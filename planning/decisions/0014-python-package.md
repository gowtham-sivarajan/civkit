# ADR 0014 — Python package: versions, wheels and the binding layer

> **Provenance:** Claude Sonnet 5.5 (`claude-sonnet-5-5`) · first drafted 2026-11-26 23:44 IST · last edited 2026-11-26 23:44 IST

- **Status:** Accepted (maintainer, 2026-11-26: Q-A through Q-G of [PYTHON_BINDINGS_PROPOSAL.md](../PYTHON_BINDINGS_PROPOSAL.md),
  with two clarifications, recorded below)
- **Date:** 2026-11-26
- **Decision ID:** D-26
- **Relation to earlier decisions:** implements D-4 (Python in Phase 3) and API_DESIGN §8; parallels D-13
  ([ADR 0013](0013-mcp-sdk-and-surface.md)). With this decision the Phase 3 preconditions are complete.

## Context

API_DESIGN §8 fixed the Python API. What remained was which Pythons and platforms, how the package is built and
shipped, how thin the Rust layer is, and which gates prove it. The maintainer set the baseline; the proposal added the
binding layer and the gates.

Facts, verified:
- Python version status, from the official page (https://devguide.python.org/versions/), confirmed by the maintainer:
  Python 3.10 reached end of life on 2026-10-01; Python 3.15 is stable.
- PyO3 0.30.1 (`cargo info`, 2026-11-26): the latest release, MIT OR Apache-2.0, minimum Rust 1.85. PyO3's feature
  guide (https://pyo3.rs/v0.30.1/features), confirmed by the maintainer: `abi3-py311` targets the stable ABI from
  Python 3.11 onward; free-threaded Python uses the separate `abi3t` ABI.
- maturin 1.16.0 and Python 3.14.10 are in the project's local `.venv`.

## Decision

1. **Versions (Q-A).** CPython **3.11–3.15** for the first release. Free-threaded CPython, PyPy and GraalPy are
   deferred from 0.1.
2. **Build (Q-A).** PyO3 **`=0.30.1`** with **`abi3-py311`**; maturin **1.16.0**. **One abi3 wheel per target**, five
   targets: Windows x86-64; macOS x86-64 and ARM64; Linux x86-64 and ARM64, built for **manylinux2014**. No promise of
   offline installation from the source archive (it needs a Rust toolchain, build tools and the crate sources).
3. **Names (Q-B).** Distribution and import name `civkit`; crate `civkit-python` in `bindings/python`; extension module
   `civkit._civkit`. The PyPI name is decided with the 0.1 release (naming risk, COMPETITIVE_LANDSCAPE §10).
4. **Thin Rust layer (Q-C).** The extension exposes only `call_json`, `list_json`, `describe_json` and `version`.
   Input parsing, `Result` and `Q` objects, errors and typed functions are plain Python; the Python layer does no
   validation of its own.
5. **Errors (Q-D).** One exception class, `CivKitError`, with `.code`, `.input`, `.index`, `.message`, and `.errors`
   listing every error of a multi-error response in order. Warnings stay in `Result.warnings`.
6. **Typed functions (Q-E).** Generated at import time from the installed descriptors, each sending the descriptor
   version it was generated from; a generated `.pyi` stub is checked for drift in CI.
7. **No runtime dependencies, no network, no intentional writes (Q-F, clarified).** The package depends on nothing
   outside the standard library. **The package itself makes no network calls and no intentional file writes**, at import
   or when calculating (OFFLINE_POLICY rule 8 bans fetching data at import). **Normal interpreter bytecode caches**
   (`.pyc` files in `__pycache__`, created by Python's import system) **are not the package's writes and are excluded.**
   A zero-file-write test is not required.

## Implementation gates (Phase 3 Python is not done until all pass in CI) (Q-G, P3 clarified)

| Gate | What must be shown |
|---|---|
| **P1 — offline install** | On each of the five targets: the wheel installs with networking disabled and runs a calculation. |
| **P2 — versions** | The Python test suite passes on CPython 3.11, 3.12, 3.13, 3.14 and 3.15. |
| **P3 — parity** | For every reference case in `tests/reference/`: **on success**, `civkit.call(...).to_dict()` equals `json.loads(call_json(request))` exactly; **on error**, the raised `CivKitError`'s fields and its full `.errors` list equal the parsed error envelope. |
| **P4 — error details** | `CivKitError` keeps `code`, `input`, `index` and the full list for multi-error responses, including list-element errors. |
| **P5 — no network** | Importing `civkit` and calling a calculation succeeds with sockets blocked. |

## Consequences

- API_DESIGN §8 gains §8.1 summarising this decision.
- IMPLEMENTATION_PLAN and ACCEPTANCE_CRITERIA Phase 3 carry gates P1–P5 beside ADR 0013's G1–G3.
- **The Phase 3 preconditions are settled** (D-13 and D-26). Phase 3 code starts, tests first: the Python package,
  then the MCP server, then the parity tests across Rust, Python and MCP.
