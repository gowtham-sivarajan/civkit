# MVP Scope — CivKit 0.1

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-24 23:41 IST (as part of README.md) · last edited 2026-11-25 09:08 IST

> **Status:** Accepted (D-3, D-7). The calculation list is conditional: a calculation stays in 0.1 only once an
> open worked example is collected and re-derived ([REFERENCE_SOURCES.md](REFERENCE_SOURCES.md), task 0.8).

## 1. Goal of 0.1

A reliable foundation, not every utility: the engine, its contract, a small set of verifiable calculations,
DXF drawing analysis, and two interfaces (Python and MCP) that prove "one core, many interfaces".

## 2. Change from the original proposal

The proposal put core + AutoCAD + MCP + Python all in 0.1. AutoCAD is the riskiest piece and depends on a
proprietary host, so this scope moves the **AutoCAD plugin to 0.2** and puts **DXF-based drawing analysis**
in 0.1 instead. The plugin then reuses that analysis (ADR 0002).

## 3. In scope

**Engine**
- Units: conversion, parsing, dimensional validation ([API_DESIGN.md §7](../docs/API_DESIGN.md#7-units)).
- Geometry: polyline length, polygon area and centroid, arc length, bearing/distance between points.
- Contract: registry, JSON request/response, trace, warnings, versioned IDs.

**Calculations** (first-principles; reference cases only from open sources — D-18). Status as of 2026-11-25:

| Domain | Calculation | Open sources | Status |
|---|---|---|---|
| Quantities | Earthwork volume — average end area | S1 | ✅ Example confirmed and re-derived |
| Quantities | Earthwork volume — prismoidal | S1 (formula only) | ⚠️ Needs an open worked example |
| Fluids | Manning's equation for uniform open-channel flow | S2, S11 | 🔎 Example to locate |
| Structural | Simply supported beam: reactions, max shear/moment, deflection (UDL and point load) | S3, S4, S5 | 🔎 Examples to locate |
| Geotechnical | Soil phase relationships (void ratio, porosity, degree of saturation, unit weights) | S6, S7 | 🔎 Example to locate |
| Surveying | Levelling reduction (rise & fall; height of instrument) | S8, S9, S10 | 🔎 Example to locate |
| Surveying | Traverse closure and compass-rule (Bowditch) adjustment | S9 (partial), S10 | ⚠️ No complete open example yet — moves to a later release if none is found |

Source IDs (S1 …) refer to [REFERENCE_SOURCES.md](REFERENCE_SOURCES.md).

**Drawing analysis (DXF, read-only)**
- Entity listing by layer; lengths and areas; drawing-unit consistency check; quantity schedule export (CSV/JSON).

**Interfaces**
- Python package exposing everything above.
- MCP server (stdio) exposing everything above.

## 4. Out of scope for 0.1

AutoCAD / Civil 3D plugin (0.2) · MATLAB · desktop app · SQLite · design-code checks · any drawing writes ·
temperature units · network features of any kind.

## 5. Later releases (indicative)

| Release | Adds |
|---|---|
| 0.2 | `civkit-ffi`; AutoCAD 2027 plugin (read-only), reusing drawing analysis; Civil 3D extension |
| 0.3 | Tauri desktop app; SQLite history |
| 0.4+ | First design-code module (D-8); MATLAB wrapper; more domains |

Definition of done for 0.1: [ACCEPTANCE_CRITERIA.md](ACCEPTANCE_CRITERIA.md).
