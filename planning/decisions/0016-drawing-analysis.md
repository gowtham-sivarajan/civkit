# ADR 0016 — Drawing analysis: a versioned report, streaming expansion in the shared budget

> **Provenance:** Claude Sonnet 5.5 (`claude-sonnet-5-5`) · first drafted 2026-11-27 18:50 IST · last edited 2026-11-27 21:10 IST by Claude Sonnet 5.5 (`claude-sonnet-5-5`)

- **Status:** Accepted (maintainer, 2026-11-27: every question Q-A … Q-I of
  [DRAWING_ANALYSIS_PROPOSAL.md](../DRAWING_ANALYSIS_PROPOSAL.md) as recommended). Details settled while implementing
  are listed below, **awaiting the maintainer's confirmation**.
- **Date:** 2026-11-27
- **Decision ID:** D-28
- **Relation to earlier decisions:**
  - Phase 4 task 3. It builds on D-12 ([ADR 0002](0002-neutral-drawing-model.md)) and D-27
    ([ADR 0015](0015-dxf-reader.md): the shared heap budget, nesting ≤ 32).
  - It settles the items D-22 ([ADR 0010](0010-geometry-conventions.md)) deferred to Phase 4:
    - transformations and OCS;
    - the ARC start = end case;
    - how geometry notes reach users;
    - the unknown-warning-codes rule;
    - a code for "not supported in 0.1".
  - It amends [GEOMETRY.md](../../docs/GEOMETRY.md) §11 and §8.7, and [API_DESIGN.md](../../docs/API_DESIGN.md) §5.

## Context

The reader returns a drawing in DXF terms:
- coordinates in each entity's own system;
- blocks unexpanded;
- MINSERT arrays as rows × columns.

0.1 needs these results from it (MVP_SCOPE §3):
- entities by layer;
- lengths and areas;
- a unit check;
- schedules.

They must meet the fixtures' exact values, report F3's planted problems as warnings, and never give a silently
partial total. Analysis must stay within the heap budget the parsed drawing uses, and within a time bound for any
input.

While drafting, one gap was found: `polygon_area`'s self-intersection check tests every pair of edges. Under the
reader's 1 000 000-vertex cap, a single ring would take about 5 × 10¹¹ pair tests.

## Decision

The proposal's §3–§11 are the specification. In summary:

1. **Interface (Q-A).** A separate, versioned drawing report, `drawing.analyze` 1.0.0. It is not a registry
   calculation and is not listed by `civkit_list`. It is available three ways:
   - Rust: `analyze` / `analyze_dxf`, with `to_json` and `to_csv`.
   - Python: `civkit.drawing.analyze` and `schedule_csv`. Neither writes a file.
   - MCP: one read-only tool, `drawing.analyze`, which exists only with `--drawing-root`. Paths are canonicalised
     and must lie inside a root, and drawing text is labelled as data.

   Saved reports keep the report version and the CivKit version.
2. **Expansion (Q-B).**
   - Expansion streams placements; it does not build an expanded copy of the drawing.
   - Measures are cached per block entity in block coordinates, then scaled per placement: area × |det|, and
     length × s under a similarity. Straight-edged lengths are recomputed under a non-uniform scale. Validity is
     decided once per block entity; precision is checked per placement.
   - Every allocation reserves from the parsed drawing's `Budget`. Geometry calls are bracketed by a stated
     per-vertex reservation.
   - Model space only. Layer `0` inherits the inserting layer. Frozen and off layers are measured and flagged.
   - DIMENSION and HATCH blocks are not expanded.
   - A cycle, a missing block or a zero scale is not expanded; a warning is given and the total is marked
     `incomplete`.
   - Limits:
     - hard, giving an error with no report: nesting 32; placement work 50 000 000; the budget;
     - soft, giving a warning: validity work of 100 000 000 pair tests; 100 000 entity rows; 20 handles per warning;
       1 000 warnings.
3. **Transformations and OCS (Q-C).**
   - `Affine2` in `civkit-geometry`. An INSERT or MINSERT places a block point by
     `p' = P + R(θ)·((c·Δc, r·Δr) + S(sx, sy)·(p − B))`, composed through nesting.
   - Extrusion (0, 0, −1) negates x. Any other non-plan extrusion is `NOT_SUPPORTED`.
   - z is not used.
   - Curves under a non-uniform scale are `NOT_SUPPORTED`.
   - An ARC with start = end, computed exactly in degrees, is a zero-sweep arc of length 0, with a note.
4. **Quantities (Q-D).**
   - Lengths, perimeters and areas by layer and kind. Different kinds are never summed together.
   - Self-intersecting closed polylines: perimeter only, with a warning.
   - HATCH: edge paths are joined into rings. Several paths are combined by even-odd nesting in a new
     `civkit_geometry::region_area`, which refuses rings that cross.
   - The all-pairs validity check gets a bounding-box sweep prefilter, giving identical results, plus the work limit.
   - Neumaier summation; no rounding.
5. **Units (Q-E).**
   - `$INSUNITS` codes converted: 1, 2, 3, 4, 5, 6, 7, 10, 14 and 21 (US survey foot).
   - "Not stated" and 0 (unitless) are reported, never assumed.
   - Other codes give drawing units only and `NOT_SUPPORTED`.
   - Results are given in drawing units, and in m and m² when the units are known.
6. **Unit check (Q-F).** P1–P4 are caution warnings, using the rules and thresholds of the proposal's §8.
   They are documented as heuristics. F3 gives exactly these four; the other fixtures give none.
7. **Schedules (Q-G).**
   - The JSON report as the proposal's §9, at `detail` `layer` or `entity`.
   - A CSV schedule per layer and kind:
     - RFC 4180, UTF-8, full precision;
     - formula-injection neutralising: a leading `=`, `+`, `-`, `@`, tab or CR is prefixed with `'`;
     - version columns.
8. **Warnings and errors (Q-H).**
   - API_DESIGN §5: clients accept unknown warning codes and treat them as `caution`.
   - One `NOT_SUPPORTED` code, with fields `feature`, `count`, `layer` and `handles`.
   - Geometry notes become `DRAWING_GEOMETRY_NOTE` (info). Precision limited becomes `ROUNDING_SENSITIVE`.
   - Drawing warning codes as the proposal's §10, aggregated per code, feature and layer, and bounded.
   - Totals that leave anything out carry `incomplete` and `omitted`.
   - Two error codes: `DRAWING_UNREADABLE` and `DRAWING_LIMIT`. MCP path problems are `INVALID_VALUE`.
   - Calculations keep ADR 0010 D3's rule.

## Implementation gates (task 3 is not done until all pass in CI)

| Gate | What must be shown |
|---|---|
| **A1 — fixtures** | F1–F5 and the BricsCAD fixture give their expected values within 1e-9 relative. F2 equals F1 in metres. F4 gives "units not stated" and F1's values in drawing units |
| **A2 — unit check** | F3 gives exactly P1–P4; the other fixtures give none |
| **A3 — transforms** | Property tests of composition, mirror bulge sign and similarity detection. MINSERT and nested placements match independently computed positions |
| **A4 — budget accounting** | Measured peak heap of read + analyse ≤ reserved peak + slack (as X3), including the geometry reservation |
| **A5 — limits** | A 10⁹ × 10⁹ MINSERT, 33-deep nesting, a self-inserting block and a 1 000 000-vertex zig-zag ring each give the stated error or warning quickly. Never a panic, an overrun or a hang |
| **A6 — no silent loss** | Every unsupported case has a test showing its total is `incomplete` and a warning names it |
| **A7 — region area** | `region_area` matches an independent computation on nested and disjoint rings, and refuses crossing rings. The sweep prefilter matches the all-pairs check on random rings |
| **A8 — parity and safety** | Rust, Python and MCP give identical JSON for every fixture. MCP refuses paths outside the roots, including through symlinks. CSV formula injection is neutralised |
| **A9 — fuzzing** | The fuzz target runs read + analyse under the default budget. Its time counts toward ADR 0015's 8 hours |

## Settled during implementation (awaiting confirmation)

Writing the code exposed points the accepted text did not settle. Each is implemented and tested; none changes a
decision above.

1. **Where uninterpreted entities count** (refines §10 "make their layer's row incomplete"). The reader now records
   where each one is: model space, paper space, or a block definition.
   - In model space, or in a block that is placed, it makes the (effective) layer incomplete, once per placement.
   - In paper space, or in a block never placed (a dimension's or hatch's own block, an unused definition), it is
     listed at level info and leaves totals complete. Otherwise every dimensioned drawing (MTEXT and SOLID in the
     dimension's block) and every layout (VIEWPORT) would show false "incomplete" totals.
   - Types that carry no length or area (ATTDEF, ATTRIB, MTEXT, POINT, VIEWPORT) are listed, never "incomplete".
2. **LINE and DIMENSION points are world (block) coordinates** in DXF, so their extrusion does not move them; the
   other plan entities go through their OCS (`Placed::plan_transform`).
3. **Hatch edge paths:**
   - consecutive edges must join within ε (GEOMETRY §5 over the path's points); otherwise `NOT_SUPPORTED`
     (`feature: "hatch boundary not joined"`);
   - clockwise arc edges (group 73 = 0) are `NOT_SUPPORTED` (`"hatch clockwise arc edge"`) in 0.1: how their angles
     are stated differs between writers, and no fixture has one to check against;
   - an arc edge becomes two half-arcs, so a full circle (360°) is representable.
4. **Hard and soft limits added:** 200 000 000 vertices measured or transformed (hard, `DRAWING_LIMIT` "vertices"),
   and the size check (P3) keeps at most 1 000 000 areas (beyond, an info `NOT_SUPPORTED` "size check sample
   limit").
5. **Placement factor:** under any non-degenerate placement, area is the local area × |det|; under a similarity,
   lengths are the local length × s. Straight-edged geometry under a non-uniform scale is re-measured from its
   placed vertices.
6. **P4 formatting:** braces, `\X…;` codes and the one-letter codes `\P \L \O \K \X \~` are removed before the
   text is read as a number; `DIMLFAC` is not read (as §8 says).

## Consequences

- GEOMETRY.md §11 is replaced by the full transformation and OCS text, and §8.7 gains the sweep prefilter.
- API_DESIGN §5 gains:
  - the unknown-codes rule;
  - `NOT_SUPPORTED`;
  - the drawing warning and error codes;
  - a drawing-report section.
- ADR 0010's Phase 4 items are closed.
- `civkit-geometry` gains `Affine2`, `region_area` and the prefilter. This is the first change to that crate since
  Phase 2B, made tests first. Its existing results are unchanged.
- The heuristics of P1–P4 can give false positives and miss some cases (for example, mixes of inches and feet).
  Their thresholds are stated and can be tuned at the bulk review without changing this decision.
- New warning codes after 0.1 are compatible additions, because clients accept unknown codes.
