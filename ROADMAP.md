# Roadmap

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-24 23:41 IST (as part of README.md) · last edited 2026-11-28 14:15 IST by Claude Sonnet 5.5 (`claude-sonnet-5-5`)

What gets built, in what order. Task breakdown: [planning/IMPLEMENTATION_PLAN.md](planning/IMPLEMENTATION_PLAN.md).
Exit criteria for each phase: [planning/ACCEPTANCE_CRITERIA.md](planning/ACCEPTANCE_CRITERIA.md).

## Phases

| Phase | Objective | Main outputs | Release |
|---|---|---|---|
| **0** | Planning *(done — tag `plan-v1.0`)* | Decisions, API design, engineering standards, reference worked examples | — |
| **1** | Architecture and Rust core *(done — tag `phase-1-done`)* | Cargo workspace, `civkit-core` registry and contract, CI on Windows/macOS/Linux | — |
| **2** | Units, geometry, calculations, validation *(done — tag `phase-2-done`)* | civkit-units, civkit-geometry, the 0.1 calculation set with reference tests | — |
| **3** | Python bindings + local MCP server *(done — tag `phase-3-done`)* | `bindings/python`, `integrations/mcp`, cross-interface parity tests | — |
| **4** | Drawing model + DXF analysis *(current: the DXF reader is done; the analysis layer is decided (D-28) and being built, its geometry first)* | `civkit-drawing`: quantities, unit checks, schedules from DXF | **0.1** |
| **5** | AutoCAD / Civil 3D integration | `civkit-ffi`, AutoCAD 2027 plugin (read-only), Civil 3D extension | **0.2** |
| **6** | Standalone desktop app | Tauri 2 app with forms generated from the registry; SQLite history | 0.3 |
| **7** | Expansion and community | Design codes, MATLAB wrapper, more domains, external contributors | 0.4+ |

## Path to 0.1

1. Finish Phase 4: the analysis layer, decided in D-28 ([ADR 0016](planning/decisions/0016-drawing-analysis.md)):
   - done, in `civkit-geometry`: transformations, areas of regions with holes, and a bounded self-intersection
     check (bounding-box sweep with a work limit);
   - done, in `civkit-drawing`: bounded block and MINSERT expansion within the shared heap budget;
   - done, in `civkit-drawing`: quantities by layer (unsupported and invalid geometry named, never silently left
     out), drawing units, and the unit check;
   - done, in `civkit-drawing`: the JSON drawing report and the CSV schedule;
   - done: the report from Python (`civkit.drawing`) and MCP (`drawing.analyze`), with parity tests;
   - done: the measured heap check (A4) and fuzzing of read + analyse (A9); every analysis gate closed.
2. Fuzz the DXF reader for 8 hours in total (the running total is in
   [planning/ACCEPTANCE_CRITERIA.md](planning/ACCEPTANCE_CRITERIA.md)).
3. Bulk human review: the D6 code review and, separately, the engineering review of the calculations; the DXF results
   and fixtures; the generated documentation.
4. The release checklist: generated pages reviewed; the offline CI job and the disclaimer in every interface (both
   added 2026-11-28, ticked after CI); release notes and checksums; tag `v0.1.0`.

After 0.1, Phase 5 (AutoCAD / Civil 3D) leads to 0.2.

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
