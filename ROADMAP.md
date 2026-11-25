# Roadmap

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-24 23:41 IST (as part of README.md) · last edited 2026-11-25 09:08 IST

What gets built, in what order. Task breakdown: [planning/IMPLEMENTATION_PLAN.md](planning/IMPLEMENTATION_PLAN.md).
Exit criteria for each phase: [planning/ACCEPTANCE_CRITERIA.md](planning/ACCEPTANCE_CRITERIA.md).

## Phases

| Phase | Objective | Main outputs | Release |
|---|---|---|---|
| **0** | Planning *(current)* | Decisions, API design, engineering standards, reference worked examples | — |
| **1** | Architecture and Rust core | Cargo workspace, `civkit-core` registry and contract, CI on Windows/macOS/Linux | — |
| **2** | Units, geometry, calculations, validation | `civkit-units`, `civkit-geometry`, the 0.1 calculation set with reference tests | — |
| **3** | Python bindings + local MCP server | `bindings/python`, `integrations/mcp`, cross-interface parity tests | — |
| **4** | Drawing model + DXF analysis | `civkit-drawing`: quantities, unit checks, schedules from DXF | **0.1** |
| **5** | AutoCAD / Civil 3D integration | `civkit-ffi`, AutoCAD 2027 plugin (read-only), Civil 3D extension | **0.2** |
| **6** | Standalone desktop app | Tauri 2 app with forms generated from the registry; SQLite history | 0.3 |
| **7** | Expansion and community | Design codes, MATLAB wrapper, more domains, external contributors | 0.4+ |

## Changes from the original proposal

| Change | Why | Decision |
|---|---|---|
| Phase 0 (planning) added | Decide before building | — |
| Python moved from Phase 5 to Phase 3, beside MCP | Both are thin layers over the registry; Python is the fastest way to get engineers testing | D-4 (accepted) |
| Drawing model + DXF added as Phase 4 | Drawing analysis written once, testable in CI, usable without AutoCAD | D-12 (accepted) |
| AutoCAD plugin moved to 0.2 | Riskiest piece; depends on a proprietary host | D-3 (accepted) |
| SQLite deferred to Phase 6 | Only the desktop app needs it | D-14 (proposed) |

## Beyond 0.x

- Design-code modules, one code at a time (D-8).
- Other disciplines — mechanical, electrical — once the civil core is proven.
- Every phase ends with a git tag (`phase-1-done`, …) so the project can always roll back to a known-good point.
