# ADR 0005 — 0.1 scope, build order, units model and FFI contract

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-25 09:04 IST · last edited 2026-11-25 09:04 IST

- **Status:** Accepted
- **Date:** 2026-11-25
- **Decision IDs:** D-3, D-4, D-5, D-6

## Context
The original proposal placed the AutoCAD plugin, MCP server and Python bindings all in 0.1, and left the units
model and the interface between C# and the Rust engine open. AutoCAD is the riskiest integration and depends on
a proprietary host; the units model and FFI shape affect every interface.

## Decision
1. **D-3 — AutoCAD plugin in 0.2.** Release 0.1 ships the engine, DXF-based drawing analysis, the Python
   package and the MCP server. The AutoCAD 2027 plugin (0.2) reuses the DXF analysis through the neutral
   drawing model (ADR 0002).
2. **D-4 — Build order:** Rust core → units, geometry and calculations → Python and MCP together → drawing
   model and DXF analysis (release 0.1) → AutoCAD / Civil 3D (0.2) → desktop app (0.3).
3. **D-5 — Units:** every input carries its unit; the engine stores SI values with dimension exponents and
   checks dimensions at run time; angle is its own dimension; ambiguous units (e.g. "ton") are rejected.
   Specification: [API_DESIGN.md §7](../../docs/API_DESIGN.md#7-units).
4. **D-6 — FFI:** non-Rust hosts call a small C ABI that passes JSON requests and responses in the same
   contract used by Python and MCP (`call`, `list`, `describe`, `free`, `version`).
   Specification: [API_DESIGN.md §9](../../docs/API_DESIGN.md#9-ffi-cratescivkit-ffi).

## Consequences
- 0.1 can be built, tested and released without any proprietary software.
- Python and MCP arrive early, so engineers can exercise calculations before any CAD work starts.
- Unit errors become explicit errors instead of wrong numbers; every interface must pass units with values.
- One stable ABI serves AutoCAD, Civil 3D and any future non-Rust host; JSON parsing overhead is accepted
  because engineering calculations are not performance-bound (typed fast paths only if profiling shows a need).
