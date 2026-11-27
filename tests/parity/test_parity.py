# Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 10:50 IST · last edited 2026-11-27 10:50 IST
"""Phase 3 parity: every reference case, and a set of error and rounding cases, gives the same result through
Rust, Python and MCP (IMPLEMENTATION_PLAN Phase 3, task 3).

Each interface runs in its own process, and every answer is read with Python's `json`, which rounds correctly, so
a reader that loses a bit anywhere shows up. (A Rust-only comparison missed exactly that: serde_json without
`float_roundtrip` misread `1.8276944379471094`, and the MCP server changed last digits.)

- **Rust:** `Registry::call_json`'s text, from `crates/civkit-core/examples/call_json.rs` (env `CIVKIT_CALL_JSON`).
- **Python:** the installed `civkit` package, `civkit.call(...)`; `to_dict()`, or the envelope rebuilt from
  `CivKitError.errors`.
- **MCP:** `civkit-mcp` over stdio (env `CIVKIT_MCP`), in both protocol eras, through the calculation's own tool and
  through `civkit_call`: `structuredContent`, the text block, and `isError`.

Run from the repository root after building:
    cargo build --release --locked -p civkit-mcp
    cargo build --release --locked -p civkit-core --example call_json
    python -m unittest discover -s tests/parity -v
"""

import json
import os
import subprocess
import sys
import unittest
from pathlib import Path

import civkit

ROOT = Path(__file__).resolve().parents[2]
EXE = ".exe" if sys.platform == "win32" else ""


def _binary(env: str, *candidates: str) -> Path:
    if os.environ.get(env):
        return Path(os.environ[env])
    for c in candidates:
        for profile in ("release", "debug"):
            p = ROOT / "target" / profile / (c + EXE)
            if p.exists():
                return p
    raise unittest.SkipTest(f"set {env} or build the binary (see this file's docstring)")


MANNING = "fluids.open_channel.manning_trapezoidal"
TRAVERSE = "surveying.traverse.compass_rule"
EARTHWORK = "quantities.earthwork.average_end_area"
LEVELLING_HI = "surveying.levelling.height_of_instrument"

MANNING_INPUTS = {
    "bottom_width": {"value": 1.5, "unit": "m"}, "side_slope": {"value": 2.0}, "depth": {"value": 0.6, "unit": "m"},
    "manning_n": {"value": 0.013}, "bed_slope": {"value": 0.002},
}

# A 17-significant-digit value that serde_json's default (not correctly rounded) parser reads one ulp high.
HARD_FLOAT_TEXT = "1.8276944379471094"


def reference_requests():
    cases = []
    for path in sorted((ROOT / "tests" / "reference").glob("*/*.json")):
        request = json.loads(path.read_text(encoding="utf-8"))["request"]
        if request["id"].startswith("math.demo."):
            continue  # registered only in civkit-core's own tests
        cases.append((f"reference {path.parent.name}/{path.stem}", request))
    return cases


def error_requests():
    with_inputs = lambda **changes: {**MANNING_INPUTS, **changes}  # noqa: E731
    return [
        ("error MISSING_UNIT", {"id": MANNING, "inputs": with_inputs(depth={"value": 0.6})}),
        ("error several OUT_OF_DOMAIN",
         {"id": MANNING, "inputs": with_inputs(bottom_width={"value": -1, "unit": "m"}, manning_n={"value": 0})}),
        ("error list element index",
         {"id": TRAVERSE, "inputs": {"bearings": {"value": [0, 90, 180], "unit": "deg"},
                                     "distances": {"value": [10, -1, 10], "unit": "m"}}}),
        ("error bad choice",
         {"id": EARTHWORK, "inputs": {"method": {"value": "prismoidal"}, "chainages": {"value": [0, 50], "unit": "m"},
                                      "cut_areas": {"value": [0, 30], "unit": "m^2"},
                                      "fill_areas": {"value": [0, 0], "unit": "m^2"}}}),
        ("error UNIT_MISMATCH", {"id": MANNING, "inputs": with_inputs(bed_slope={"value": 0.002, "unit": "m"})}),
        ("error UNKNOWN_OUTPUT", {"id": MANNING, "inputs": MANNING_INPUTS, "output_units": {"nothing": "m"}}),
        ("error UNKNOWN_UNIT", {"id": MANNING, "inputs": with_inputs(depth={"value": 0.6, "unit": "furlongs"})}),
    ]


def rounding_requests():
    """The regression of 2026-11-27: n = 0.013 gave velocity 1.8276944379471094 from Rust and Python but
    ...096 from MCP, which re-read the engine's JSON with a lossy parser."""
    return [
        ("rounding: manning n = 0.013 (velocity 1.8276944379471094)", {"id": MANNING, "inputs": MANNING_INPUTS}),
        ("rounding: manning n = 0.013, velocity in ft/s",
         {"id": MANNING, "inputs": MANNING_INPUTS, "output_units": {"velocity": "ft/s", "discharge": "L/s"}}),
        ("rounding: an input with 17 significant digits",
         {"id": LEVELLING_HI, "inputs": {"start_elevation": {"value": 100, "unit": "m"},
                                         "backsights": {"value": [float(HARD_FLOAT_TEXT), 0.0], "unit": "m"},
                                         "foresights": {"value": [1.0, 0.5], "unit": "m"}}}),
    ]


def with_version(request):
    """Every interface gets the same request: the descriptor's version made explicit, as the MCP tools send it."""
    r = dict(request)
    r.setdefault("version", civkit.describe(r["id"])["version"])
    return r


ALL = [(name, with_version(r)) for name, r in reference_requests() + error_requests() + rounding_requests()]


def rust_answers(requests):
    exe = _binary("CIVKIT_CALL_JSON", "examples/call_json")
    text = "\n".join(json.dumps(r) for r in requests) + "\n"
    done = subprocess.run([str(exe)], input=text, capture_output=True, text=True, encoding="utf-8", timeout=120,
                          check=True)
    lines = done.stdout.splitlines()
    assert len(lines) == len(requests), (len(lines), len(requests))
    return [json.loads(line) for line in lines]


def python_answer(request):
    try:
        return civkit.call(request["id"], version=request.get("version"), output_units=request.get("output_units"),
                           **request.get("inputs", {})).to_dict()
    except civkit.CivKitError as e:
        return {"error": e.errors[0]} if len(e.errors) == 1 else {"errors": e.errors}


class McpClient:
    """A minimal stdio JSON-RPC client, so the check reads raw lines with Python's json and needs no MCP SDK."""

    def __init__(self, era):
        self.era = era
        self.proc = subprocess.Popen([str(_binary("CIVKIT_MCP", "civkit-mcp"))], stdin=subprocess.PIPE,
                                     stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True, encoding="utf-8")
        self.next_id = 0
        if era == "2025-11-25":
            init = self.request("initialize", {"protocolVersion": era, "capabilities": {},
                                               "clientInfo": {"name": "civkit-parity", "version": "0"}})
            assert init["result"]["protocolVersion"] == era, init
            self.send({"jsonrpc": "2.0", "method": "notifications/initialized"})

    def send(self, message):
        self.proc.stdin.write(json.dumps(message) + "\n")
        self.proc.stdin.flush()

    def request(self, method, params):
        if self.era == "2026-07-28":
            params = {**params, "_meta": {"io.modelcontextprotocol/protocolVersion": self.era,
                                          "io.modelcontextprotocol/clientCapabilities": {}}}
        self.next_id += 1
        self.send({"jsonrpc": "2.0", "id": self.next_id, "method": method, "params": params})
        while True:
            line = self.proc.stdout.readline()
            if not line:
                raise AssertionError(f"civkit-mcp closed stdout during {method}")
            message = json.loads(line)
            if message.get("id") == self.next_id:
                return message

    def call(self, tool, arguments):
        response = self.request("tools/call", {"name": tool, "arguments": arguments})
        assert "error" not in response, response
        result = response["result"]
        from_text = json.loads(result["content"][0]["text"])
        assert from_text == result["structuredContent"], f"{tool}: the text block differs from structuredContent"
        return result

    def close(self):
        self.proc.stdin.close()
        code = self.proc.wait(timeout=30)
        self.proc.stdout.close()
        assert code == 0, f"civkit-mcp exited with {code}"


def per_calc_arguments(request):
    args = dict(request.get("inputs", {}))
    if "output_units" in request:
        args["output_units"] = request["output_units"]
    return args


def is_error(answer):
    return "error" in answer or "errors" in answer


class Parity(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.rust = rust_answers([r for _, r in ALL])

    def test_the_case_set(self):
        ids = {r["id"] for _, r in ALL}
        self.assertEqual(ids, {d["id"] for d in civkit.list()}, "every calculation is covered")
        self.assertGreaterEqual(sum(1 for n, _ in ALL if n.startswith("reference")), 13)

    def test_python_equals_rust(self):
        for (name, request), rust in zip(ALL, self.rust):
            with self.subTest(name):
                self.assertEqual(python_answer(request), rust)

    def test_mcp_equals_rust_in_both_eras(self):
        for era in ("2025-11-25", "2026-07-28"):
            client = McpClient(era)
            try:
                for (name, request), rust in zip(ALL, self.rust):
                    with self.subTest(f"{era} {name}"):
                        own = client.call(request["id"], per_calc_arguments(request))
                        self.assertEqual(own["structuredContent"], rust)
                        self.assertEqual(own.get("isError", False), is_error(rust))
                        generic = client.call("civkit_call", request)
                        self.assertEqual(generic["structuredContent"], rust)
                        self.assertEqual(generic.get("isError", False), is_error(rust))
            finally:
                client.close()

    def test_inputs_keep_every_bit(self):
        """The 17-digit backsight comes back unchanged (x + 0.0 is exact) from every interface."""
        name, request = next((n, r) for n, r in ALL if n == "rounding: an input with 17 significant digits")
        rust = self.rust[ALL.index((name, request))]
        exact = float(HARD_FLOAT_TEXT)
        self.assertEqual(rust["outputs"]["sum_backsights"]["value"], exact)
        self.assertEqual(python_answer(request)["outputs"]["sum_backsights"]["value"], exact)
        client = McpClient("2026-07-28")
        try:
            result = client.call(request["id"], per_calc_arguments(request))
        finally:
            client.close()
        self.assertEqual(result["structuredContent"]["outputs"]["sum_backsights"]["value"], exact)


if __name__ == "__main__":
    unittest.main()
