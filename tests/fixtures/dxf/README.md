# DXF test fixtures

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-25 10:38 IST · last edited 2026-11-25 10:38 IST

Sample drawings for the read-only DXF analysis in 0.1 (task 0.9, [ADR 0002](../../../planning/decisions/0002-neutral-drawing-model.md)).
Each fixture has **exact expected values fixed by construction**: the geometry was chosen so that every length and area has a closed form.
The values were then **independently re-checked**:
- The saved files were read with a different program from the one that wrote them (ezdxf 1.4.5).
- Lengths and areas were recomputed with separate bulge and arc formulas.
- Areas and lengths were also cross-checked with shapely on flattened geometry.

All checks passed on 2026-11-25. The check script is kept outside the repository (Phase 0 rule: no code yet).

Do not re-save these files in a CAD program: the checksums below would change, and so might the quirks under test.
`.gitattributes` keeps `*.dxf` byte-exact (`-text`).

## Licence

| Files | Author / writer | Licence |
|---|---|---|
| `f1` – `f3`, `f5` | CivKit project; drawn in CADCraft on 2026-11-25 | Apache-2.0 (this repository) |
| `f4` | CivKit project; written with ezdxf 1.4.5 (CADCraft cannot write R12) | Apache-2.0 (this repository) |
| `third_party/ezdxf/*.dxf` | Manfred Moitzi, from the [ezdxf](https://github.com/mozman/ezdxf) repository, `examples_dxf/`, commit `175dee00aa187c76ccebb77db9f9fea8f99b01e8` | MIT — see [`third_party/ezdxf/LICENSE-ezdxf.txt`](third_party/ezdxf/LICENSE-ezdxf.txt) (must stay with the files) |

## Fixtures

| File | DXF version · writer | Units ($INSUNITS) | Line endings | Tests |
|---|---|---|---|---|
| `f1_site_plan_m.dxf` | R2018 (AC1032) · CADCraft | 6 = metres | CRLF | Baseline: LWPOLYLINE with a bulge, CIRCLE, ARC, LINE, HATCH, TEXT, linear DIMENSION, layers |
| `f2_site_plan_mm.dxf` | R2010 (AC1024) · CADCraft | 4 = millimetres | CRLF | Same plan × 1000 — unit conversion must give the same results in metres as F1 |
| `f3_units_mismatch.dxf` | R2013 (AC1027) · CADCraft | 1 = inches (with $MEASUREMENT = 1, metric) | CRLF | Planted unit problems (below) |
| `f4_legacy_r12.dxf` | R12 (AC1009) · ezdxf | none (R12 has no $INSUNITS) | LF | Old heavy POLYLINE/VERTEX/SEQEND with a bulge; unknown units; header extents left invalid (±1e20) |
| `f5_blocks_inserts.dxf` | R2004 (AC1018) · CADCraft | 6 = metres | CRLF | Blocks: rotated, non-uniform, mirrored, MINSERT array, nested blocks |
| `third_party/ezdxf/insert_bricscad_level_1.dxf` | R2013 (AC1027) · BricsCAD (saved by the ezdxf author) | 6 | — | Another CAD program's output: nested INSERTs, mirrored (negative scale) and rotated, TEXT in blocks |
| `third_party/ezdxf/Minimal_DXF_AC1009.dxf` | no header · hand-written | none | — | Smallest valid DXF: only an empty ENTITIES section. Must load with zero entities and unknown units, not crash |

## Expected values

Units are the drawing's own units unless stated. Tolerance for the engine: 1e-9 relative (exact geometry).
For flattened-curve methods, use a stated chord tolerance instead.

### F1 `f1_site_plan_m.dxf` (metres) and F4 `f4_legacy_r12.dxf` (same geometry, no units)

| Layer | Entities | Expected |
|---|---|---|
| BOUNDARY | 1 closed polyline (0,0)–(60,0)–(60,30)–(40,45)–(0,45) | perimeter **200**; area **2550** |
| ROAD_CL | 1 open polyline: straight 60, quarter arc R = 20 (bulge tan 22.5° = 0.414213562373095), straight 40 | length **100 + 10π = 131.415926536** |
| BUILDING | 2 closed rectangles, 12 × 8 and 10 × 10 | areas **96** and **100**; perimeters **40** and **40** |
| WATER | 1 circle, centre (50,20), R = 3 | area **9π = 28.274333882**; circumference **6π = 18.849555922** |
| DRAIN | 1 arc, centre (10,30), R = 5, 0°→180° + 1 line (5,30)–(5,5) | total length **5π + 25 = 40.707963268** |
| TEXT | 1 TEXT | `PLOT 17` |
| HATCH (F1 only) | 1 ANSI31 hatch over building 1, not associative | area **96** |
| DIM (F1 only) | 1 linear dimension (0,0)–(60,0) | measurement **60** |

F4 has no HATCH or DIM layers. Its units are unknown: the engine must report "units not stated", not assume metres.

### F2 `f2_site_plan_mm.dxf` (millimetres)

Every F1 coordinate is multiplied by 1000. In drawing units, lengths are × 1000 and areas × 10⁶ (e.g. boundary area 2.55 × 10⁹ mm²).
The dimension measures 60000.
**After unit conversion, every result must equal F1's in metres.** This is the main unit-conversion test.

### F3 `f3_units_mismatch.dxf` — planted problems

| # | Problem | Data |
|---|---|---|
| P1 | Header conflict | `$INSUNITS = 1` (inches) but `$MEASUREMENT = 1` (metric) |
| P2 | One block inserted at two scales a factor 1000 apart | Block `WINDOW_MM` (1200 × 150, drawn in mm): one insert at scale 0.001 (width **1.2**), one at scale 1 (width **1200**) |
| P3 | Same-layer size outlier ≈ 1000× (pasted from a mm drawing) | BUILDING: areas **96**, **100** and **96 000 000** |
| P4 | Dimension text override disagrees with geometry | Linear dimension measures **12**; its text is overridden to `15` |

The 0.1 unit check should report all four as findings (warnings), not errors. A drawing can be like this on purpose.

### F5 `f5_blocks_inserts.dxf` (metres)

Blocks:
- `PARKING_BAY`: closed rectangle 2.5 × 5 plus a centre stripe line of length 4.
- `TREE`: a circle of R = 1.
- `ISLAND`: a 5 × 2 rectangle plus two `TREE` inserts at scale 0.5 (nested).

| Layer | References | Expected |
|---|---|---|
| PARKING | 6 × scale 1; 2 × rotated 90°; 1 × scale (2, 1); 1 × mirrored scale (−1, 1); 1 × MINSERT 1 row × 3 columns @ 2.5 | **13** bays; area **175**; total perimeter **200**; total stripe length **52** |
| LANDSCAPE | 2 × `ISLAND` | island area **20**; 4 nested trees, canopy area **π = 3.141592654**; circumference **4π = 12.566370614** |

Block contents are on layer 0 and must be reported under the inserting reference's layer (PARKING or LANDSCAPE).

## Checksums (SHA-256, first 16 hex digits)

| File | SHA-256 (prefix) |
|---|---|
| f1_site_plan_m.dxf | bfeb7dd96481f022 |
| f2_site_plan_mm.dxf | 9ae96e6b6d17a2aa |
| f3_units_mismatch.dxf | 5bd22161e75b4429 |
| f4_legacy_r12.dxf | 1b6a0ced2323f5df |
| f5_blocks_inserts.dxf | 43106506fcdbeaa4 |
| third_party/ezdxf/insert_bricscad_level_1.dxf | c2597f95c007852d |
| third_party/ezdxf/Minimal_DXF_AC1009.dxf | 7286a35c85164d01 |
