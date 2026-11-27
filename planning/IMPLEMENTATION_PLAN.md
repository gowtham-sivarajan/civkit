# Implementation Plan

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-24 23:41 IST (as part of README.md and HANDOFF.md) · last edited 2026-11-27 18:19 IST by Claude Sonnet 5.5 (`claude-sonnet-5-5`)

> **Status:** Living document — tasks are ticked as they finish. Exit criteria: [ACCEPTANCE_CRITERIA.md](ACCEPTANCE_CRITERIA.md).

## Phase 0 — Planning *(done, tag `plan-v1.0`)*

| # | Task | Status |
|---|---|---|
| 0.1 | Write core plan, handover and original proposal into the repository; set up git | ✅ 2026-11-24 |
| 0.2 | Decide repository layout (D-15) and drawing model (D-12) | ✅ 2026-11-25 |
| 0.3 | Restructure documentation (D-16) | ✅ 2026-11-25 |
| 0.4 | Install and verify toolchain (Rust, VS 2026 C++/.NET, `.venv`, SQLite CLI, VS Code extensions) | ✅ 2026-11-25 |
| 0.5 | Decide name (D-1) and licence (D-2); add `LICENSE` | ✅ 2026-11-25 |
| 0.6 | Decide D-3, D-4, D-5, D-6, D-7 (+ D-18 open sources only) | ✅ 2026-11-25 |
| 0.7 | Finish [API_DESIGN.md](../docs/API_DESIGN.md) TBDs (contract, units) | ✅ 2026-11-25 — 9 resolutions accepted (D-19, ADR 0007) |
| 0.8 | For each 0.1 candidate, collect ≥1 worked example from an **open** source ([REFERENCE_SOURCES.md](REFERENCE_SOURCES.md)), record citation and numbers, re-derive independently; drop candidates with none | ✅ 2026-11-25 — 9 cases (C1–C9) re-derived; soil phase relationships and general prismoidal deferred; human engineering review deferred until product complete |
| 0.9 | Collect 3–5 licence-safe sample DXF drawings | ✅ 2026-11-25 — 5 own fixtures (CADCraft + ezdxf R12) + 2 ezdxf MIT samples; [tests/fixtures/dxf](../tests/fixtures/dxf/README.md) |
| 0.10 | Decide when to create the GitHub repository → public from day one (D-17) | ✅ 2026-11-25 |
| 0.12 | Create public GitHub repository, push history and tags, enable private vulnerability reporting | ✅ 2026-11-25 |
| 0.11 | Tag `plan-v1.0` | ✅ 2026-11-25 |

Dependencies: 0.7 needs 0.6 (D-5, D-6). 0.8 needs D-7. Phase 1 needs 0.5–0.12.

## Phase 1 — Architecture and Rust core

> Status 2026-11-25: ✅ steps 1–6 done. CI green on Windows, macOS and Linux with Rust 1.98 and stable ([run 45555065260](https://github.com/gowtham-sivarajan/civkit/actions/runs/45555065260)).
> Tag `phase-1-done` on the commit that records this.

1. Create Cargo workspace and empty crates per [ARCHITECTURE.md §2](../docs/ARCHITECTURE.md).
2. CI (GitHub Actions): build, test, `rustfmt`, `clippy` on Windows/macOS/Linux.
3. Dependency-rule check in CI.
4. `civkit-core`: descriptor types, registry, request/response, errors, warnings, trace.
5. One dummy calculation end to end, with a reference-case data file format.
6. Calculation template for contributors.

## Phase 2 — Units, geometry, calculations

Order agreed 2026-11-25. The units parser is infrastructure: it is finished and tested before any calculation uses it.

| Stage | Work | Exit condition | Status |
|---|---|---|---|
| 2A | `civkit-units`: quantity type, dimension algebra, named units and compound parser, ambiguity rejection, DMS and bearings ([API_DESIGN.md §7](../docs/API_DESIGN.md#7-units), D-19) | Explicit units; dimensional validation; ambiguity rejection; round-trip, property-based and rejection tests pass | ✅ 2026-11-25: implemented and reviewed; CI green on `b4f37e2` and on the internal-units follow-up `6d3d16a` |
| 2B | `civkit-geometry`: primitives needed by 0.1 | Documented coordinate and angle conventions, numerical tolerances, edge-case tests | ✅ 2026-11-25: specification accepted (D-22, [ADR 0010](decisions/0010-geometry-conventions.md)) after [review](GEOMETRY_SPEC_REVIEW.md); implemented in `8a93d58`, CI green; implementation items confirmed; CI green on the overflow-rule follow-up `bc06cb1` |
| 2C | The five 0.1 calculations, exactly as D-20 ([ADR 0009](decisions/0009-implementation-scope-0-1-calculations.md)) defines, one at a time | All applicable reference cases (C1–C9 and the extra beam case) pass | ✅ 2026-11-26: **all five D-20 families are implemented**, as seven registered calculations: Manning (C5), beam reactions, deflection and end slope (C6–C8), traverse (C3, [proposal](TRAVERSE_DESCRIPTOR_PROPOSAL.md)), levelling in three calculations (C4, C9, [proposal](LEVELLING_DESCRIPTOR_PROPOSAL.md)) and earthwork (C1, C2, [proposal](EARTHWORK_DESCRIPTOR_PROPOSAL.md)). D-23 (lists and choices) accepted, [ADR 0011](decisions/0011-list-and-choice-inputs.md). Every descriptor was proposed and accepted before its code (D-20 condition 4). The beam's maximum shear and moment are deferred from 0.1 (D-25, [ADR 0012](decisions/0012-validity-basis-and-beam-scope.md)). Committed and pushed as `948a49a`, CI green. Details: [AI_HANDOVER.md](AI_HANDOVER.md) |
| 2D | Integration and regression | Full test suite, dependency checks and cross-platform CI pass; tag `phase-2-done` | ✅ 2026-11-26: review done ([PHASE_2D_REVIEW.md](PHASE_2D_REVIEW.md)); CI green on `948a49a` (all nine jobs); D3 and D4 decided (D-24, D-25, [ADR 0012](decisions/0012-validity-basis-and-beam-scope.md)). Tag `phase-2-done` held until CI has checked the latest commit (the green run on `948a49a` predates D-24 and D-25), which waits for pushing to resume (the maintainer, 2026-11-26) |

Beam maximum shear and moment: **deferred from 0.1** (D-25, [ADR 0012](decisions/0012-validity-basis-and-beam-scope.md)). The search for an openly licensed worked example that numerically verifies them continues (D-20 condition 1); when one is accepted they return as new outputs, a pure addition, in a later release. Searched 2026-11-25 (S4 ch. 7, an openly licensed statics text): none fits a full-span UDL with one point load.

## Phase 3 — Python and MCP

**Phase 3: complete (2026-11-27).** Tag `phase-3-done` on `a94bf3f`, CI #19 and Python #4 green; every Phase 3
acceptance item is ticked with its evidence.

**Phase 3 preconditions: settled (2026-11-26).** D-13 ([ADR 0013](decisions/0013-mcp-sdk-and-surface.md)) and D-26 ([ADR 0014](decisions/0014-python-package.md)) are accepted; D-26 was the last open one. Phase 3 code can start, tests first. The D6 code review and human engineering review are bulk review gates after Phase 4 and before 0.1; they do not block Phase 3 coding.

- **D-13 accepted** (2026-11-26, [ADR 0013](decisions/0013-mcp-sdk-and-surface.md)): `rmcp` `=3.12.0`, stdio, both protocol eras. The MCP server is done only
  when its three gates pass in CI: **G1** real stdio calls from a 2026-07-28 client and a 2025-11-25 client; **G2**
  `check_descriptor` enforces MCP's tool-name rules (at most 128 characters, allowed set), tests first; **G3** in both
  eras, `structuredContent` equals `Registry::call_json`'s output and contract errors keep `isError` and their envelope.
- **Thin interfaces:** Python and MCP both go through the existing JSON contract (`Registry::call_json`), with parity
  checks against it. Nothing re-implements validation, units or the trace.
- **As API_DESIGN already says:** PyO3 and maturin for Python; MCP over stdio only and read-only; per-calculation and
  generic tools (`--tools both`, §10).
- **D-26 accepted** (2026-11-26, [ADR 0014](decisions/0014-python-package.md)): CPython 3.11–3.15; PyO3 `=0.30.1` with `abi3-py311`; maturin 1.16.0;
  five abi3 wheels (Windows x86-64, macOS x86-64 and ARM64, Linux x86-64 and ARM64, manylinux2014); thin Rust layer
  (`call_json`, `list_json`, `describe_json`, `version`). The Python package is done only when **P1** offline install
  and a call on all five targets, **P2** tests on 3.11–3.15, **P3** parity (success: `to_dict()` equals `call_json`;
  error: `CivKitError` fields and `.errors` equal the envelope), **P4** error details and **P5** no network pass in CI.
- **Checks to build:** wheels that install offline on Windows, macOS and Linux; real stdio MCP calls; list and choice
  inputs; units; trace fields; error `index`. Plus the planned notebooks and generated documentation pages (task 4 and
  ACCEPTANCE_CRITERIA §3).

1. `bindings/python`: PyO3 + maturin; generic `call`, `list`, `describe`; generated typed functions. **Code done
   (2026-11-27); gates pending CI:** 38 Python tests (API, P3 parity incl. error envelopes, P4, P5, stub drift) pass
   locally against an offline-installed Windows wheel; P1 on all five targets and P2 on 3.11–3.15 need the new
   `Python` workflow (`.github/workflows/python.yml`) to run green.
2. `integrations/mcp`: `rmcp` server, stdio, tools generated from descriptors, read-only annotations. **Code done
   (2026-11-27):** G1–G3 tests over real stdio in both eras pass locally and run in `ci.yml` (the crate is a workspace
   member); the dependency rule now also checks that `civkit-mcp` has no network crate. Checked with MCP Inspector
   2.16.0 and the Python MCP SDK 2.3.0 in both eras: every result equals `call_json`, no network endpoint. G1–G3
   are met when `ci.yml` is green with this code.
3. Parity tests across Rust, Python and MCP. **Written (2026-11-27):** `tests/parity/test_parity.py`, every reference
   case plus error and rounding cases through Rust `call_json` (`examples/call_json.rs`), the Python package and
   the MCP server in both eras, each in its own process, all read with Python's `json`. Passes locally; shown to
   fail (15 failures) without serde_json's `float_roundtrip`. Met when the `parity` job of `python.yml` is green.
4. Example notebooks in `integrations/jupyter`. **Done (2026-11-27):** three notebooks (getting started; surveying;
   beam and earthwork) from the published reference cases, stored without outputs; `test_notebooks.py` runs every
   code cell in each Python CI job. Executed in Jupyter locally; results match the sources.
5. Generated forms, typed functions and API docs show conditional requirements prominently, e.g. the beam's
   `point_load_position`, required only when `point_load > 0`. Such conditions stay in prose in 0.1 (D-23 §3.9).
   **Done for Phase 3 (2026-11-27):** the convention is "...; required when <condition>" at the end of an optional
   input's description; Python docstrings and the `.pyi` stub, MCP tool and input descriptions, and the generated
   pages (`docs/calculations`, `tools/gen_calc_docs.py`, drift-checked in CI) all show it. Forms are the Phase 6
   desktop app's and will read the same convention.

## Phase 4 — Drawing model and DXF → 0.1

**Current phase (from 2026-11-27).**

1. Evaluate existing Rust DXF crates vs own reader; decide (new decision record).
   **Decided (2026-11-27):** D-27, [ADR 0015](decisions/0015-dxf-reader.md): CivKit's own reader; `undxf` =0.11.0 as a
   test-only cross-check; a 512 MiB heap budget that every allocation reserves from; gates X1–X6 (fixtures,
   cross-check, budget accounting, budget exhaustion, malformed input, fuzzing).
2. `civkit-drawing`: neutral model, DXF import, limits. Tests first, to ADR 0015's gates X1–X6.
   **Reader written (2026-11-27):** X1 fixtures, X2 `undxf` cross-check, X3 budget accounting, X4 budget exhaustion and
   X5 malformed input pass locally (34 tests, plus property tests); X3 was shown to catch three accounting faults.
   In `main` as `241b44c` (2026-11-27): CI #23, Python #8 and Fuzz #1 green, so X6's 10-minute run has passed once. Review
   follow-ups are in `4e41e7a` (CI #24, Python #9 green). Fuzz #2: fuzzing green, its corpus cross-check red. ADR 0015
   amended (maintainer): X2 is strict on the fixtures and a curated well-formed corpus (`tests/well_formed/`) and
   reports-and-triages the fuzz corpus ([DXF_CROSS_CHECK_LOG.md](DXF_CROSS_CHECK_LOG.md)). Fuzz time:
   [FUZZ_LOG.md](FUZZ_LOG.md). Then the task 3 proposal.
3. Analysis: entities by layer, lengths/areas, unit consistency, schedules (CSV/JSON).
   - Add the 2D transformations and OCS handling of [GEOMETRY.md §11](../docs/GEOMETRY.md) to `civkit-geometry` (D-22).
   - Before exposing drawing results through the contract: decide how geometry notes reach users, add "clients accept
     unknown warning codes" to API_DESIGN §5, and add a code for "not supported in 0.1" (D-22, D3).
   - Expansion (block instances, MINSERT arrays, transformed geometry) reserves from the **same combined budget** as
     the parsed drawing (ADR 0015), not a separate one; the task 3 proposal fixes its details within that rule.
4. Fuzzing; DXF fixtures; expose via Python and MCP.
5. Bulk human review before 0.1: complete D6, the separate C1–C9 engineering review, Phase 4 DXF results and fixtures, and generated documentation review; record each outcome. Then release 0.1.

## Phase 5 — AutoCAD / Civil 3D → 0.2

1. `civkit-ffi` C ABI.
2. AutoCAD 2027 plugin (.NET 10): entity → neutral model translation, read-only commands, results display.
3. Civil 3D project referencing the AutoCAD shared project.
4. Manual in-host test checklist.

## Phase 6 — Desktop app

Tauri 2 app; forms generated from descriptors; SQLite history; offline and CSP checks.

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Scope too broad for a small team | Nothing ships | Narrow 0.1; AutoCAD to 0.2; phase exit criteria |
| Wrong engineering result | Loss of trust; potential harm | [ENGINEERING_STANDARDS.md](../docs/ENGINEERING_STANDARDS.md); versioned calculations; warnings; disclaimer |
| AutoCAD .NET runtime churn (.NET 8 → .NET 10) | Plugin breaks on new releases | Thin plugin; logic in Rust; start with 2027 only; per-version checklist |
| MCP specification changes | Integration rework | MCP isolated in `integrations/mcp`; pin SDK version per release |
| Rust is a barrier for engineering contributors | Few contributors | Templates; data-driven reference cases; review-only contributions |
| Single maintainer burnout | Project stalls | Small releases; clear docs; AI_HANDOVER.md always current |
| Copyright in design-code material | Legal exposure | Cite clauses, implement equations, never paste code text |
| Hostile DXF files | Crashes, hangs | Parser limits and fuzzing ([SECURITY.md](../docs/SECURITY.md)) |
