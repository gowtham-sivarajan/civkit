# ADR 0002 — Neutral drawing model and DXF reader in the Rust engine

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-25 00:14 IST · last edited 2026-11-25 00:28 IST

- **Status:** Accepted
- **Date:** 2026-11-25
- **Decision ID:** D-12

## Context
The original proposal placed drawing analysis (reading entities, extracting lengths and areas, checking
drawing units, producing quantity schedules) inside the AutoCAD plugin. That would make the analysis:
- usable only inside AutoCAD,
- impossible to test in CI (AutoCAD cannot run there),
- something to re-implement for Civil 3D, Python, MCP and the desktop app.

## Decision
Add `crates/civkit-drawing`: a CAD-neutral drawing model (lines, polylines, arcs, circles, blocks, layers,
dimensions, drawing units) plus a DXF reader. All drawing analysis is written against this model.

The AutoCAD and Civil 3D integrations become thin translators: they convert AutoCAD entities into the neutral
model (through `civkit-ffi`) and display the results. Python, MCP and the desktop app load DXF files directly.

Analysis stays read-only (principle S-5). Writing to drawings is a separate, later decision.

## Consequences
- Quantity extraction and unit checks are written once and used by every interface.
- Drawing analysis is tested in CI with DXF fixture files in `tests/fixtures/dxf/`.
- Users without AutoCAD, or with other CAD tools that export DXF, can still use drawing analysis.
- The neutral model must be designed carefully up front; AutoCAD/Civil 3D objects with no DXF equivalent
  (e.g. Civil 3D alignments and surfaces) will need explicit extensions later.
- DXF parsing adds a dependency (an existing Rust DXF crate, to be evaluated in Phase 4) or our own reader.
