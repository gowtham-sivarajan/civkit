# Proposal: drawing analysis (Phase 4, task 3)

> **Provenance:** Claude Sonnet 5.5 (`claude-sonnet-5-5`) · first drafted 2026-11-27 18:43 IST · last edited 2026-11-27 18:50 IST by Claude Sonnet 5.5 (`claude-sonnet-5-5`)

> **Status:** **Accepted by the maintainer, 2026-11-27: every question Q-A … Q-I as recommended.** Recorded as
> [ADR 0016](decisions/0016-drawing-analysis.md) (D-28), the source of truth.
>
> Built on: D-12 / [ADR 0002](decisions/0002-neutral-drawing-model.md) (analysis written once, against the neutral
> model); D-27 / [ADR 0015](decisions/0015-dxf-reader.md) (the parsed drawing, the heap budget that parsing and
> expansion share, nesting depth ≤ 32); D-22 / [ADR 0010](decisions/0010-geometry-conventions.md) and
> [GEOMETRY.md](../docs/GEOMETRY.md) §11 (transformations, OCS; geometry notes and "not supported in 0.1" deferred to
> this phase, item D3); [API_DESIGN.md](../docs/API_DESIGN.md) §5 (errors and warnings), §3 (saved output keeps its
> version), §10 (MCP drawing tools read only configured locations); [SECURITY.md](../docs/SECURITY.md) (drawing text
> is data, not instructions); [MVP_SCOPE.md](MVP_SCOPE.md) §3; the fixtures F1–F5 and their expected values
> ([tests/fixtures/dxf](../tests/fixtures/dxf/README.md)).

## 1. What has to be decided

The reader (task 2) returns the file's content in DXF terms: coordinates in each entity's own coordinate system,
blocks unexpanded, MINSERT arrays as rows × columns. Task 3 turns that into results people can use:

1. how analysis is exposed (Rust, Python, MCP) and versioned (Q-A);
2. block and MINSERT expansion within the shared budget, with limits on work as well as memory (Q-B);
3. transformations and OCS in `civkit-geometry`, including the ARC start = end case GEOMETRY.md left open (Q-C);
4. what is measured, how hatches with several boundary paths get an area, and how existing geometry code fits the
   budget and a time bound (Q-D);
5. drawing units and conversion (Q-E); the unit check, F3's planted problems P1–P4 (Q-F);
6. schedules in JSON and CSV (Q-G);
7. how warnings, geometry notes, unsupported geometry and errors reach users, which ADR 0010 deferred here (Q-H);
8. the implementation gates (Q-I).

## 2. Requirements

| # | Requirement | Source |
|---|---|---|
| N1 | Entities listed by layer; lengths and areas; a drawing-unit consistency check; schedules in CSV and JSON | MVP_SCOPE §3 |
| N2 | Every fixture's expected values, within 1e-9 relative: F1, F2 (equal to F1 after conversion), F4 (same as F1 in drawing units, units not stated), F5 (13 bays, area 175, perimeter 200, stripes 52; islands 20, canopy π, circumference 4π) | fixtures README |
| N3 | F3's P1–P4 reported as warnings, not errors | fixtures README ("a drawing can be like this on purpose") |
| N4 | Block contents on layer `0` are reported under the inserting reference's layer | F5 |
| N5 | Never a silently partial or silently wrong total: anything left out is counted, named and marks the total incomplete | GEOMETRY §8 ("reported or refused, never silent"), ADR 0015 R2 |
| N6 | Parsed drawing and expansion share one heap budget; analysis time is bounded too, for any input | ADR 0015 Q-D, SECURITY.md |
| N7 | No rounding inside the engine; full `f64` in results | ENGINEERING_STANDARDS §3 |
| N8 | Read-only; drawing-sourced text is labelled data; MCP reads only configured locations | ADR 0002, SECURITY.md, API_DESIGN §10 |
| N9 | One core, many interfaces: Rust, Python and MCP give the same JSON | Phase 3 parity rule |

## 3. Interface and versioning (Q-A)

Drawing analysis does not fit the calculation contract: its input is a file, not values with units, and its output
is a report of variable size, not named outputs. Forcing it into a calculation descriptor would bend both.

**Proposed:** a separate, versioned **drawing report**, sharing the contract's conventions where they apply.

- **Rust** (`civkit-drawing`, module `analysis`):
  `analyze(drawing: &Drawing, budget: &Budget, options: &AnalysisOptions) -> Result<Report, AnalysisError>`, with
  `Report::to_json()` and `Report::to_csv()`. The budget is the one the drawing was read with (Q-B).
  `analyze_dxf(bytes, options)` reads and analyses in one call, with the default budget and limits.
- **Python:** `civkit.drawing.analyze(path_or_bytes, *, detail="layer")` returns the report as a `dict` (the same
  JSON); `civkit.drawing.schedule_csv(...)` returns the CSV text. No file is written (ADR 0014: no intentional
  writes); the caller saves it.
- **MCP:** one read-only tool, `drawing.analyze` (`path`, optional `detail`). It exists only when the server is
  started with at least one `--drawing-root <dir>` (repeatable). Paths are canonicalised (symlinks resolved) and must
  lie inside a root, otherwise a contract error. `structuredContent` is the report JSON; drawing-sourced strings sit
  only in fields documented as drawing text, and the tool description says they are data, not instructions.
- **Not in the calculation registry:** `civkit_list` does not list it. The ID domain `drawing` (API_DESIGN §4) is used
  for the report's identity: `"report": "drawing.analyze"`, `"version": "1.0.0"`, with the same SemVer rules as
  calculations (§11). Saved reports keep `version` and the CivKit version (§3).
- **Shared with the contract:** the warning object (`code`, `message`, `level`), the error envelope, and the rule
  that unknown warning codes are accepted (Q-H).

## 4. Expansion within the shared budget (Q-B)

### 4.1 Streaming, not an expanded copy

ADR 0015 requires "the parsed drawing and its expanded form" to share one budget. The proposal keeps that form
small by **not materialising an expanded drawing**: expansion walks placements and adds each placed entity's
measures to running totals. What it allocates (the walk's stack, per-block caches, transformed vertex buffers,
totals, report rows, the JSON or CSV text) reserves from the **same `Budget`** as the parsed drawing, through the
budgeted containers of task 2 (`BVec`, `BStr`, `BIndex`). A refused reservation is `AnalysisError::BudgetExhausted`,
never a panic.

Existing `civkit-geometry` functions allocate with ordinary `Vec`s (about six arrays of n entries). The analysis
**reserves a stated bound per vertex** (proposed 256 bytes × n, checked by a measured test like X3) from the budget
before each call and releases it after. That keeps the geometry crate free of the budget type and still bounds
everything analysis asks for.

### 4.2 Measured once per block, placed by transform

Lengths and areas of a block's entities are computed **once, in block coordinates**, and cached (budgeted). A
placement then applies its transform to the cached results:

- area × |det A| for any affine transform (exact for straight and, under a similarity, bulged edges);
- length × s for a similarity (uniform scale s = √|det A|); under a non-uniform scale, straight-edged lengths are
  recomputed from transformed vertices (their work is counted), and curved geometry is "not supported" (Q-C);
- validity (self-intersection) is invariant under a non-degenerate affine map, so it is decided once per block
  entity; the precision check (GEOMETRY §8.9) depends on position, so it is re-applied per placement from the
  transformed bounding box.

The result is defined as exactly this computation, so it is deterministic. Nested MINSERT arrays then cost work per
placement in proportion to the block's number of totals, not its vertices.

### 4.3 Rules

| Case | Rule |
|---|---|
| Which entities | Model space only (`paper_space = false`). Paper-space entities are counted by type and layer, not measured |
| What expands | INSERT and MINSERT, including anonymous `*U` blocks. A DIMENSION's or HATCH's own block is never expanded (it is the rendering of an entity already measured) |
| Layer `0` in blocks | Takes the effective layer of the insert that places it, through nesting (N4) |
| Frozen or off layers | Included and measured; the layer row carries `frozen` / `off` so the user decides. Not silently excluded |
| MINSERT | columns × rows placements, the product computed with checked arithmetic before any work |
| Nesting | Depth ≤ 32 (ADR 0015); deeper → `AnalysisError::Limit { what: "nesting" }` |
| Cycle (a block that reaches itself) | That INSERT is not expanded; warning `DRAWING_BLOCK_CYCLE` with the chain; totals on its layer `incomplete` |
| INSERT of an undefined block | Not expanded; warning `DRAWING_BLOCK_MISSING`; layer `incomplete` |
| Scale 0 on any axis (det A = 0) | Not expanded; warning `DRAWING_ZERO_SCALE`; layer `incomplete` |
| ATTRIB | Counted (the reader already does); attribute values are not quantities in 0.1 |

### 4.4 Limits (all in `AnalysisOptions`, documented defaults)

| Limit | Default | When exceeded |
|---|---|---|
| Heap: the shared `Budget` | 512 MiB, parsing + analysis together | `BudgetExhausted` |
| Placement work: 1 per placed entity, + n per vertex transformed | 50 000 000 units | `Limit { what: "placements" }` |
| Validity-check work: edge-pair tests (Q-D) | 100 000 000 pairs per analysis | that entity's area not computed; `NOT_SUPPORTED` (`feature: "validity check work limit"`); `incomplete` |
| Report rows at `detail = "entity"` | 100 000 | rows stop; warning `DRAWING_REPORT_TRUNCATED`; totals stay complete |
| Handles listed per warning | 20 | the rest counted in `count` |
| Distinct warnings | 1 000 | one final `DRAWING_REPORT_TRUNCATED` |

An analysis that hits a hard limit returns an error, not a partial report: a total that stops halfway would break N5.

## 5. Transformations and OCS (Q-C)

Added to `civkit-geometry` (ADR 0010 D6b) and specified in GEOMETRY.md §11, which this replaces with full text:

- `Affine2 { a, b, c, d, tx, ty }` with `apply`, `then` (composition), `det`, `is_similarity()` (|σ₁ − σ₂| ≤ 10⁻⁹·σ₁)
  and `transform_vertex` (negates the bulge when det < 0).
- **INSERT placement**, block point p to the insert's OCS:
  `p' = P + R(θ)·( (c·Δc, r·Δr) + S(sx, sy)·(p − B) )`, with B the block base point, P the insertion point, θ the
  rotation, Δc and Δr the column and row spacing (unscaled, along the rotated axes), c and r the column and row
  index. A single INSERT has c = r = 0. The composed transform of nested inserts is the product, outermost last.
- **OCS:** the insert's extrusion, and each entity's, maps OCS to world. (0, 0, 1) within 10⁻⁹ is the identity;
  (0, 0, −1) negates x (GEOMETRY §11: a mirror, bulges negated). Any other extrusion is not a plan entity: not
  measured; `NOT_SUPPORTED` (`feature: "non-plan extrusion"`), counted, `incomplete`.
- **z is not used** for plan quantities: elevations and `sz` are ignored (sz < 0 with the default extrusion does not
  mirror the plan). The report says that quantities are plan quantities.
- **Non-uniform scale:** lines and straight polyline segments are transformed; circles, arcs and bulged segments are
  `NOT_SUPPORTED` (`feature: "curve under non-uniform scale"`), never approximated (GEOMETRY §11).
- **ARC start = end** (left open by GEOMETRY §11): Δ = (end − start) mod 360°, computed in degrees as stated,
  exactly. Δ = 0 is a **zero-sweep arc: length 0**, with the geometry note `ZeroSweep` (GEOMETRY §8.8) reported as
  warning `DRAWING_GEOMETRY_NOTE`. It is not read as a full circle: the file does not say which, and CAD programs do
  not create such arcs. *(Alternative: refuse it, `DRAWING_GEOMETRY_INVALID`.)*

## 6. Quantities (Q-D)

| Entity | Measures | Notes |
|---|---|---|
| LINE | length | |
| ARC | length | Q-C for start = end |
| CIRCLE | circumference, area | |
| LWPOLYLINE / POLYLINE, open | length | |
| LWPOLYLINE / POLYLINE, closed | perimeter, area | Self-intersecting → area not computed, `DRAWING_GEOMETRY_INVALID`, `incomplete`; perimeter still given (GEOMETRY D4) |
| HATCH | area | Below |
| TEXT | listed (value, layer, handle, position); no quantity | Drawing text, labelled |
| DIMENSION | listed (type, stated measurement, override text) | Used by the P4 check |

**Kinds are never summed together.** A layer row has separate columns for line length, arc length, open-polyline
length, closed-polyline perimeter and area, circle circumference and area, and hatch area. A closed polyline and
the hatch over it are both areas of the same building; adding them would count it twice.

**Hatch area.** A boundary path becomes a ring: polyline paths as stated; edge paths joined in order (consecutive
ends within ε, else `DRAWING_GEOMETRY_INVALID`), each arc edge turned into a bulge (negative when clockwise). Paths
with an unsupported edge (ellipse, spline) give no area (`NOT_SUPPORTED`, `incomplete`), as the reader's note says.

- One path: its area.
- Several paths: **even-odd nesting** (the DXF default style): a ring at nesting depth k counts with sign (−1)ᵏ. A
  new function `civkit_geometry::region_area(rings)` computes it. It refuses rings that cross or touch each other
  (exact edge relations, kept internal as ADR 0010 D7 requires) and returns "not supported" for them. Depth is
  decided by a point of each ring tested against the others. *(Alternative for 0.1: sum only paths whose bounding
  boxes are disjoint, and report nested islands as not supported. Simpler, but islands are common in site hatches.)*

**Time bound on validity.** `polygon_area` tests every pair of edges (GEOMETRY §8.7, "O(n²), adequate for 0.1
sizes"). The reader allows 1 000 000 vertices per polyline, which is 5 × 10¹¹ pair tests: an effective hang.
Proposed change to `civkit-geometry`: a **sweep over edge bounding boxes** (inflated by ε) as a prefilter, so that
only pairs whose boxes overlap are tested exactly. Results are identical by construction (a property test compares
it with the all-pairs check). Because a hostile ring can make every box overlap, pair tests also draw on the
analysis's validity-work limit (Q-B).

**Totals** use compensated (Neumaier) summation, in drawing units and, when units are known, in metres and square
metres (Q-E). No rounding.

## 7. Drawing units (Q-E)

- `$INSUNITS` codes CivKit converts in 0.1: 1 in, 2 ft, 3 mi, 4 mm, 5 cm, 6 m, 7 km, 10 yd, 14 dm, and
  **21 US survey foot** (1200/3937 m, exact; `ft_us` in `civkit-units`), common in US civil drawings. The factors are a
  private table in the analysis, checked against `civkit-units`; the public unit list (API_DESIGN §7.1.2) is
  not changed.
- **Not stated** (no `$INSUNITS`, as R12 files and F4): values in drawing units only, metres absent; warning
  `DRAWING_UNITS_NOT_STATED` (level caution). Never assumed.
- **0 = unitless:** the same, with a message that says the drawing declares itself unitless.
- **Any other code** (µin, Å, light years, US survey inch/yard/mile, …): drawing units only; `NOT_SUPPORTED`
  (`feature: "drawing unit code N"`).
- The report header carries `units: { "insunits": 6, "name": "m", "to_metres": 1.0 }` or
  `{ "insunits": null, "stated": false }`.

## 8. The unit check (Q-F)

All four are **warnings, level caution**, and are heuristics: the documentation states each rule and threshold. Each
names the entities by handle.

| # | Check | Rule |
|---|---|---|
| P1 | `DRAWING_UNITS_CONFLICT` | `$INSUNITS` is an imperial unit (1, 2, 3, 10, 21) and `$MEASUREMENT` = 1 (metric), or a metric unit (4, 5, 6, 7, 14) and `$MEASUREMENT` = 0 |
| P2 | `DRAWING_BLOCK_SCALE_MISMATCH` | One block placed at uniform scales whose ratio (largest ÷ smallest, over all placements, nesting included) is within 1 % of a ratio between two length units the table knows and at least 10 (10, 12, 25.4, 100, 304.8, 1000, …). The message names the likely pair (mm and m) |
| P3 | `DRAWING_SIZE_OUTLIER` | On one layer, among at least 3 closed areas of one kind, an area whose ratio to the median is at least 10³ (or at most 10⁻³). Catches pastes across metric scales (mm/m, cm/m); it does not detect in/ft mixes, and the documentation says so |
| P4 | `DRAWING_DIMENSION_OVERRIDE` | A dimension's override text, after removing formatting codes, is a number that differs from the stated measurement by more than 10⁻⁹ relative. `<>` (measured value shown) and no override are not findings; a non-numeric override is level info. CivKit does not read DIMLFAC in 0.1, and the message says a dimension scale could explain it |

F3 must give exactly these four and no others; F1, F2, F4 and F5 must give none of P1–P4.

## 9. Schedules (Q-G)

**JSON** (the report, `detail` = `layer` or `entity`):

```json
{
  "report": "drawing.analyze", "version": "1.0.0", "civkit_version": "0.1.0",
  "drawing": { "acadver": "AC1032", "codepage": "ANSI_1252",
               "units": { "insunits": 6, "name": "m", "to_metres": 1.0, "stated": true } },
  "plan_quantities": true,
  "layers": [ { "name": "BOUNDARY", "frozen": false, "off": false,
                "counts": { "polyline_closed": 1 },
                "drawing_units": { "polyline_closed_perimeter": 200.0, "polyline_closed_area": 2550.0 },
                "si": { "polyline_closed_perimeter_m": 200.0, "polyline_closed_area_m2": 2550.0 },
                "incomplete": false, "omitted": 0 } ],
  "text": [ { "layer": "TEXT", "handle": "2A", "value": "PLOT 17", "x": 0.0, "y": 0.0 } ],
  "dimensions": [ { "layer": "DIM", "handle": "2B", "type": "rotated", "measurement": 60.0, "override": null } ],
  "not_interpreted": [ { "type": "3DFACE", "layer": "X", "count": 2 } ],
  "warnings": [ { "code": "DRAWING_UNITS_NOT_STATED", "level": "caution", "message": "…" } ]
}
```

At `detail = "entity"` a bounded `entities` array adds one row per placed entity (layer, kind, handle, the chain of
insert handles, measures).

**CSV** (the schedule): one row per layer and kind, columns `layer, kind, count, length, area, unit, length_m,
area_m2, incomplete, omitted, report_version, civkit_version`. RFC 4180, UTF-8 without BOM, `.` decimal, numbers as
shortest round-trip text (full precision). **Formula injection:** a cell beginning with `=`, `+`, `-`, `@`, tab or
carriage return is prefixed with `'`, because layer names come from the drawing and spreadsheets execute such cells;
the documentation says so. The version columns satisfy API_DESIGN §3 (saved output keeps its version) without a
non-CSV header line.

## 10. Warnings, notes, unsupported geometry and errors (Q-H)

Settles ADR 0010 D3's deferred items.

- **API_DESIGN §5 gains the rule: clients accept unknown warning codes** (and treat an unknown code as level
  `caution`). New codes are then compatible additions (§11).
- **"Not supported in 0.1" is one code, `NOT_SUPPORTED`**, with a `feature` string (for example `"curve under
  non-uniform scale"`, `"hatch spline edge"`, `"non-plan extrusion"`, `"entity type 3DFACE"`), `count`, `layer`
  and up to 20 `handles`. It is a warning in drawing reports (the rest of the drawing is still analysed) and may
  later be used by calculations as an error with the same code.
- **Geometry notes** (GEOMETRY §8: zero-length segment, zero sweep, degenerate ring, orientation undefined) become
  `DRAWING_GEOMETRY_NOTE`, level info, with `note`, `count` and handles; **precision limited** becomes the existing
  `ROUNDING_SENSITIVE`, level caution. Calculations keep D3's rule (no new codes for them).
- **Drawing-specific warnings:** `DRAWING_UNITS_NOT_STATED`, `DRAWING_UNITS_CONFLICT`,
  `DRAWING_BLOCK_SCALE_MISMATCH`, `DRAWING_SIZE_OUTLIER`, `DRAWING_DIMENSION_OVERRIDE`, `DRAWING_GEOMETRY_INVALID`,
  `DRAWING_BLOCK_MISSING`, `DRAWING_BLOCK_CYCLE`, `DRAWING_ZERO_SCALE`, `DRAWING_TEXT_NOT_DECODED` (the reader's
  note), `DRAWING_HATCH_NOT_READ` (the reader's note), `DRAWING_REPORT_TRUNCATED`.
- **Aggregation:** one warning per (code, feature or note, layer), with `count` and at most 20 handles, so the
  number of warnings is bounded by codes × layers, and capped (Q-B).
- **Incomplete totals:** a layer row whose totals leave anything out has `incomplete: true` and `omitted: n`, and a
  warning says why. Entities the reader did not interpret are listed in `not_interpreted` and also make their
  layer's row incomplete.
- **Errors** (no report): `DRAWING_UNREADABLE` (the reader's `Malformed`, `BadGroupCode`, `InvalidNumber`,
  `NonFinite`, `UnexpectedEnd`, `BinaryNotSupported`; message gives the line), `DRAWING_LIMIT` (`InputTooLarge`,
  `TooManyVertices`, `StringTooLong`, `BudgetExhausted`, `AllocationFailed`, nesting, placements; message names the
  limit), and
  for MCP `INVALID_VALUE` on `path` (outside the roots, not found). Added to API_DESIGN §5 as drawing error codes.

## 11. Gates (Q-I)

Named A1–A9 (analysis). Tests are written first.

| Gate | What must be shown |
|---|---|
| A1 — fixtures | F1–F5 and the BricsCAD fixture give the expected values (N2), 1e-9 relative; F2 = F1 in metres; F4 units not stated, values in drawing units equal F1's |
| A2 — unit check | F3 gives exactly P1–P4; the other fixtures none |
| A3 — transforms | Property tests: composition, mirror bulge sign, similarity detection; MINSERT and nested placements match independently computed positions (F5, BricsCAD) |
| A4 — budget accounting | Measured peak heap of read + analyse ≤ reserved peak + slack, as X3, including the per-vertex reservation for geometry calls |
| A5 — limits | Hostile MINSERT (10⁹ × 10⁹), 33-deep nesting, a self-inserting block, a 1 000 000-vertex zig-zag ring: each gives the stated error or warning quickly, never a panic, overrun or hang |
| A6 — no silent loss | Every unsupported case of §4–§7 has a test that its total is `incomplete` and a warning names it |
| A7 — region area | `region_area` against an independent shapely computation on nested and disjoint rings; crossing rings refused; sweep prefilter identical to all-pairs on random rings |
| A8 — parity and safety | Rust, Python and MCP reports are identical JSON for every fixture; MCP refuses paths outside the roots (including via symlinks); CSV formula injection neutralised |
| A9 — fuzzing | The fuzz target runs read + analyse under the default budget; its time counts toward ADR 0015's 8 hours |

## 12. Questions for the maintainer

| # | Question | Recommendation | Decision (2026-11-27) |
|---|---|---|---|
| **Q-A** | A separate, versioned drawing report (`drawing.analyze` 1.0.0), not a registry calculation; Rust, Python and one MCP tool gated by `--drawing-root` (§3) | **Yes** | **Accepted** |
| **Q-B** | Streaming expansion with per-block cached measures, all allocations from the shared budget (a per-vertex reservation around geometry calls), the rules of §4.3 and the limits of §4.4 | **As proposed** | **Accepted** |
| **Q-C** | Transformations and OCS as §5, replacing GEOMETRY §11; ARC start = end is a zero-sweep arc of length 0 with a note | **As proposed** (alternative: refuse it) | **Accepted** |
| **Q-D** | Quantities as §6: kinds never summed together; even-odd `region_area` for multi-path hatches; a bounding-box sweep prefilter plus a work limit for validity | **As proposed** (alternative: disjoint paths only) | **Accepted** |
| **Q-E** | Unit table of §7, including US survey foot; "not stated" and "unitless" never assumed | **As proposed** | **Accepted** |
| **Q-F** | P1–P4 rules and thresholds of §8 (heuristics, documented) | **As proposed**; thresholds can be tuned at review | **Accepted** |
| **Q-G** | JSON report and CSV schedule as §9, with formula-injection neutralising and version columns | **As proposed** | **Accepted** |
| **Q-H** | §10: the unknown-codes rule in API_DESIGN §5; one `NOT_SUPPORTED` code with `feature`; geometry notes as `DRAWING_GEOMETRY_NOTE`; aggregated, bounded warnings; `incomplete` totals; two drawing error codes | **As proposed** | **Accepted** |
| **Q-I** | Gates A1–A9; record as D-28 in ADR 0016, amending GEOMETRY §11 and API_DESIGN §5 and closing ADR 0010's Phase 4 items | **Yes** | **Accepted**: [ADR 0016](decisions/0016-drawing-analysis.md) |

## 13. After acceptance

1. ADR 0016 (D-28); DECISIONS; GEOMETRY.md §11 and §8.7 (sweep prefilter); API_DESIGN §5 (unknown codes, new codes)
   and a drawing-report section; IMPLEMENTATION_PLAN task 3 and ACCEPTANCE_CRITERIA Phase 4 updated.
2. Tests first, in order: `Affine2` and `region_area` in `civkit-geometry` (A3, A7); expansion and quantities
   (A1, A4–A6); unit check (A2); schedules; Python and MCP (A8); the fuzz target (A9).
3. Then task 4's remaining items and the bulk human review before 0.1.
