# Implementation Plan

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-24 23:41 IST (as part of README.md and HANDOFF.md) · last edited 2026-11-25 18:37 IST

> **Status:** Living document — tasks are ticked as they finish. Exit criteria: [ACCEPTANCE_CRITERIA.md](ACCEPTANCE_CRITERIA.md).

## Phase 0 — Planning *(current)*

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
| 2A | `civkit-units`: quantity type, dimension algebra, named units and compound parser, ambiguity rejection, DMS and bearings ([API_DESIGN.md §7](../docs/API_DESIGN.md#7-units), D-19) | Explicit units; dimensional validation; ambiguity rejection; round-trip, property-based and rejection tests pass | 🔄 2026-11-25: implemented and reviewed; CI green on `b4f37e2`. Follow-up enforces internal units in trace and messages (61 tests locally). Done when the maintainer's Windows check and CI on the follow-up pass |
| 2B | `civkit-geometry`: primitives needed by 0.1 | Documented coordinate and angle conventions, numerical tolerances, edge-case tests | 🔄 2026-11-25: specification accepted (D-22, [ADR 0010](decisions/0010-geometry-conventions.md)) after [review](GEOMETRY_SPEC_REVIEW.md); implemented and tested locally. Done when the maintainer confirms the items settled during implementation and CI passes |
| 2C | The five 0.1 calculations, exactly as D-20 ([ADR 0009](decisions/0009-implementation-scope-0-1-calculations.md)) defines, one at a time | All applicable reference cases (C1–C9 and the extra beam case) pass | ⬜ |
| 2D | Integration and regression | Full test suite, dependency checks and cross-platform CI pass; tag `phase-2-done` | ⬜ |

Before 2C: collect one more openly licensed worked example that numerically verifies beam maximum shear and maximum moment (D-20).

## Phase 3 — Python and MCP

1. `bindings/python`: PyO3 + maturin; generic `call`, `list`, `describe`; generated typed functions.
2. `integrations/mcp`: `rmcp` server, stdio, tools generated from descriptors, read-only annotations.
3. Parity tests across Rust, Python and MCP.
4. Example notebooks in `integrations/jupyter`.

## Phase 4 — Drawing model and DXF → 0.1

1. Evaluate existing Rust DXF crates vs own reader; decide (new decision record).
2. `civkit-drawing`: neutral model, DXF import, limits.
3. Analysis: entities by layer, lengths/areas, unit consistency, schedules (CSV/JSON).
   - Add the 2D transformations and OCS handling of [GEOMETRY.md §11](../docs/GEOMETRY.md) to `civkit-geometry` (D-22).
   - Before exposing drawing results through the contract: decide how geometry notes reach users, add "clients accept
     unknown warning codes" to API_DESIGN §5, and add a code for "not supported in 0.1" (D-22, D3).
4. Fuzzing; DXF fixtures; expose via Python and MCP.
5. Release 0.1.

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
