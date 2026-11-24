# Implementation Plan

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-24 23:41 IST (as part of README.md and HANDOFF.md) · last edited 2026-11-25 00:29 IST

> **Status:** Living document — tasks are ticked as they finish. Exit criteria: [ACCEPTANCE_CRITERIA.md](ACCEPTANCE_CRITERIA.md).

## Phase 0 — Planning *(current)*

| # | Task | Status |
|---|---|---|
| 0.1 | Write core plan, handover and original proposal into the repository; set up git | ✅ 2026-11-24 |
| 0.2 | Decide repository layout (D-15) and drawing model (D-12) | ✅ 2026-11-25 |
| 0.3 | Restructure documentation (D-16) | ✅ 2026-11-25 |
| 0.4 | Install and verify toolchain (Rust, VS 2026 C++/.NET, `.venv`, SQLite CLI, VS Code extensions) | 🔄 in progress |
| 0.5 | Decide name (D-1) and licence (D-2); add `LICENSE` | ⬜ |
| 0.6 | Decide D-3, D-4, D-5, D-6, D-7 | ⬜ |
| 0.7 | Finish [API_DESIGN.md](../docs/API_DESIGN.md) TBDs (contract, units) | ⬜ |
| 0.8 | Collect ≥1 published worked example per 0.1 calculation (book, edition, page, inputs, outputs) | ⬜ |
| 0.9 | Collect 3–5 licence-safe sample DXF drawings | ⬜ |
| 0.10 | Decide when to create the GitHub repository (private now vs after licence) | ⬜ |
| 0.11 | Tag `plan-v1.0` | ⬜ |

Dependencies: 0.7 needs 0.6 (D-5, D-6). 0.8 needs D-7. Phase 1 needs 0.5–0.11.

## Phase 1 — Architecture and Rust core

1. Create Cargo workspace and empty crates per [ARCHITECTURE.md §2](../docs/ARCHITECTURE.md).
2. CI (GitHub Actions): build, test, `rustfmt`, `clippy` on Windows/macOS/Linux.
3. Dependency-rule check in CI.
4. `civkit-core`: descriptor types, registry, request/response, errors, warnings, trace.
5. One dummy calculation end to end, with a reference-case data file format.
6. Calculation template for contributors.

## Phase 2 — Units, geometry, calculations

1. `civkit-units`: quantity type, dimension algebra, unit registry and parser, ambiguity rejection, DMS.
2. `civkit-geometry`: primitives and algorithms needed by 0.1.
3. The 0.1 calculations, one at a time, each through the per-calculation checklist.
4. Property tests.

## Phase 3 — Python and MCP

1. `bindings/python`: PyO3 + maturin; generic `call`, `list`, `describe`; generated typed functions.
2. `integrations/mcp`: `rmcp` server, stdio, tools generated from descriptors, read-only annotations.
3. Parity tests across Rust, Python and MCP.
4. Example notebooks in `integrations/jupyter`.

## Phase 4 — Drawing model and DXF → 0.1

1. Evaluate existing Rust DXF crates vs own reader; decide (new decision record).
2. `civkit-drawing`: neutral model, DXF import, limits.
3. Analysis: entities by layer, lengths/areas, unit consistency, schedules (CSV/JSON).
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
