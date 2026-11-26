# Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 00:02 IST · last edited 2026-11-27 00:02 IST
"""Gates P3, P4 and P5 of ADR 0014 (D-26). P1 (offline wheel install on five targets) and P2 (CPython 3.11–3.15)
need the CI matrix and are run there."""

import json
import subprocess
import sys
import textwrap
import unittest
from pathlib import Path

import civkit
from civkit import _civkit

# The repository's reference cases: tests/reference/<id>/<case>.json, from bindings/python/tests/.
REFERENCE = Path(__file__).resolve().parents[3] / "tests" / "reference"


def reference_cases():
    files = sorted(REFERENCE.glob("*/*.json"))
    if not files:
        raise AssertionError(f"no reference cases found under {REFERENCE}; run the tests from the repository")
    for path in files:
        yield path, json.loads(path.read_text(encoding="utf-8"))


class P3Parity(unittest.TestCase):
    """For every reference case: success compares to_dict() with call_json; an error compares CivKitError's fields
    and its full .errors list with the parsed envelope."""

    def test_every_reference_case(self):
        checked = 0
        for path, case in reference_cases():
            request = case["request"]
            if request["id"].startswith("math.demo."):
                continue  # the template is registered only in Rust tests, not in the built-in registry
            with self.subTest(case=str(path.relative_to(REFERENCE))):
                engine = json.loads(_civkit.call_json(json.dumps(request)))
                kwargs = dict(request["inputs"])
                try:
                    result = civkit.call(
                        request["id"],
                        version=request.get("version"),
                        output_units=request.get("output_units"),
                        **kwargs,
                    )
                except civkit.CivKitError as e:
                    expected = engine.get("errors") or [engine["error"]]
                    self.assertEqual(e.errors, expected)
                    self.assertEqual((e.code, e.input, e.index), (expected[0]["code"], expected[0].get("input"), expected[0].get("index")))
                else:
                    self.assertEqual(result.to_dict(), engine)
                checked += 1
        self.assertGreaterEqual(checked, 13, "every built-in calculation's reference cases are compared")

    def test_error_envelopes(self):
        """The reference cases of the built-in calculations all succeed, so the error half of P3 is shown here: one
        error, several errors, a list-element error and request-level errors, each compared field by field."""
        manning = {"bottom_width": {"value": 1.5, "unit": "m"}, "side_slope": {"value": 2.0},
                   "depth": {"value": 0.6, "unit": "m"}, "manning_n": {"value": 0.013}, "bed_slope": {"value": 0.002}}
        requests = [
            {"id": "fluids.open_channel.manning_trapezoidal", "inputs": {**manning, "depth": {"value": 0.6}}},
            {"id": "fluids.open_channel.manning_trapezoidal",
             "inputs": {**manning, "bottom_width": {"value": -1, "unit": "m"}, "manning_n": {"value": 0}}},
            {"id": "surveying.traverse.compass_rule",
             "inputs": {"bearings": {"value": [0, 90, 180], "unit": "deg"},
                        "distances": {"value": [10, -1, 10], "unit": "m"}}},
            {"id": "fluids.open_channel.manning_trapezoidal", "version": 99, "inputs": manning},
            {"id": "fluids.open_channel.manning_trapezoidal", "inputs": manning, "output_units": {"nothing": "m"}},
            {"id": "fluids.nothing.here", "inputs": {}},
        ]
        for request in requests:
            with self.subTest(request=json.dumps(request)[:80]):
                engine = json.loads(_civkit.call_json(json.dumps(request)))
                expected = engine.get("errors") or [engine["error"]]
                with self.assertRaises(civkit.CivKitError) as e:
                    civkit.call(request["id"], version=request.get("version"),
                                output_units=request.get("output_units"), **request["inputs"])
                err = e.exception
                self.assertEqual(err.errors, expected)
                first = expected[0]
                self.assertEqual((err.code, err.message, err.input, err.index),
                                 (first["code"], first["message"], first.get("input"), first.get("index")))


class P4ErrorDetails(unittest.TestCase):
    def test_a_single_error_keeps_code_input_and_message(self):
        with self.assertRaises(civkit.CivKitError) as e:
            civkit.call("fluids.open_channel.manning_trapezoidal", bottom_width="1.5 m", side_slope=2.0,
                        depth="0.6 m", manning_n=0.013, bed_slope="0.002 m")
        err = e.exception
        self.assertEqual(err.code, "UNIT_MISMATCH")
        self.assertEqual(err.input, "bed_slope")
        self.assertIsNone(err.index)
        self.assertTrue(err.message)
        self.assertEqual(len(err.errors), 1)

    def test_several_errors_arrive_together_in_order(self):
        with self.assertRaises(civkit.CivKitError) as e:
            civkit.call("fluids.open_channel.manning_trapezoidal", bottom_width="-1 m", side_slope=2.0,
                        depth="0 m", manning_n=0.0, bed_slope=0.002)
        codes = [(x["code"], x["input"]) for x in e.exception.errors]
        self.assertEqual(codes, [("OUT_OF_DOMAIN", "bottom_width"), ("OUT_OF_DOMAIN", "depth"), ("OUT_OF_DOMAIN", "manning_n")])
        self.assertEqual(e.exception.code, "OUT_OF_DOMAIN")
        self.assertEqual(e.exception.input, "bottom_width")

    def test_a_list_element_error_keeps_its_index(self):
        with self.assertRaises(civkit.CivKitError) as e:
            civkit.surveying.traverse.compass_rule(bearings=([0, 90, 180, 270], "deg"), distances=([100, 0, 99, 50], "m"))
        err = e.exception
        self.assertEqual((err.code, err.input, err.index), ("OUT_OF_DOMAIN", "distances", 1))
        self.assertIn("leg 2", err.message)

    def test_an_unknown_calculation(self):
        with self.assertRaises(civkit.CivKitError) as e:
            civkit.call("fluids.nothing.here")
        self.assertEqual(e.exception.code, "UNKNOWN_CALCULATION")

    def test_the_exception_text_names_every_error(self):
        with self.assertRaises(civkit.CivKitError) as e:
            civkit.call("fluids.open_channel.manning_trapezoidal", bottom_width="-1 m", side_slope=2.0,
                        depth="0 m", manning_n=0.013, bed_slope=0.002)
        text = str(e.exception)
        self.assertIn("bottom_width", text)
        self.assertIn("depth", text)


class P5NoNetwork(unittest.TestCase):
    def test_import_and_call_with_sockets_blocked(self):
        script = textwrap.dedent(
            """
            import socket

            def refuse(*args, **kwargs):
                raise RuntimeError("network access attempted")

            socket.socket = refuse
            socket.create_connection = refuse
            socket.getaddrinfo = refuse

            import civkit
            r = civkit.call("fluids.open_channel.manning_trapezoidal", bottom_width="1.5 m", side_slope=2.0,
                            depth="0.6 m", manning_n=0.013, bed_slope=0.002)
            assert r.outputs["discharge"].value > 0
            print("ok")
            """
        )
        done = subprocess.run([sys.executable, "-c", script], capture_output=True, text=True, timeout=120)
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(done.stdout.strip(), "ok")


if __name__ == "__main__":
    unittest.main()
