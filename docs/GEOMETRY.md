# Geometry specification (Phase 2B, `civkit-geometry`)

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-25 16:10 IST · last edited 2026-11-27 20:24 IST by Claude Sonnet 5.5 (`claude-sonnet-5-5`)

> **Status: Accepted** (maintainer, 2026-11-25), after the review in
> [planning/GEOMETRY_SPEC_REVIEW.md](../planning/GEOMETRY_SPEC_REVIEW.md). Recorded as D-22 in
> [ADR 0010](../planning/decisions/0010-geometry-conventions.md).
> A few details were settled while implementing. They are marked **(impl.)** below and listed in ADR 0010; the
> maintainer confirmed them on 2026-11-25, adding the overflow rule for derived tolerances (§5).
> §11 (transformations, regions) and the §8.7 prefilter were added by D-28
> ([ADR 0016](../planning/decisions/0016-drawing-analysis.md)) for Phase 4.

This document defines, before any implementation, every convention `civkit-geometry` follows:

- coordinate systems, axes and angles;
- tolerances and point equality;
- degeneracy;
- distance and length;
- area, orientation and centroid.

Like [API_DESIGN.md §7.1](API_DESIGN.md), it is meant to be complete: behaviour not described here is not
implemented.

## 1. Scope

**In 2B** (from [MVP_SCOPE.md §3](../planning/MVP_SCOPE.md), "Geometry"), plane 2D only:

| Capability | Used by |
|---|---|
| Distance and azimuth between two points; components (Δx, Δy) | Traverse (2C), drawing analysis (Phase 4) |
| Components from azimuth and distance (latitude, departure) | Traverse (2C) |
| Polyline length, with straight and circular-arc (bulge) segments | Drawing analysis: F1, F2, F4 road centreline |
| Arc length; circle circumference and area | Drawing analysis: ARC and CIRCLE entities |
| Polygon area (straight and bulged edges) and orientation | Drawing analysis: F1–F5 areas |
| Polygon centroid (straight-edged polygons only, D5) | MVP scope ("polygon area and centroid") |

**Not in 2B:**

- 3D geometry. Elevations (z) are not used.
- Intersections, offsets and clipping, except the internal self-intersection *check* in §8.7. There is no public
  intersection API.
- Splines and ellipses.
- Coordinate transformations and regions. Drawing analysis needs block inserts with scale, rotation and mirroring,
  and hatch areas with holes. They are specified in §11 and implemented in Phase 4 (D-28), still part of release
  0.1, in `civkit-geometry` (D6b).
- Map projections, grid scale factors, earth curvature and geodesy.
- Any registry calculation (`geometry.*` IDs) or other change to the calculation contract.

This specification limits Phase 2B to the list above, in line with D-20. 3D remains out of 0.1.

## 2. Numbers and units

- **Floating point.** All coordinates and lengths are IEEE 754 `f64`.
  - Every input must be finite. A NaN or infinity is rejected (§8.1), never propagated.
  - A result too large for `f64` is an error, not ±∞ (§8.1).
- **The geometry crate is unit-agnostic.** All lengths in one call must be in the same length unit. Results come
  back in that unit (lengths) or its square (areas).
  - Calculations call it with internal SI units (metres), per API_DESIGN §7.1.5.
  - Drawing analysis calls it in drawing units and converts afterwards (Phase 4).
- **No units dependency.** The crate does not depend on `civkit-units`. Units are the caller's responsibility,
  which keeps the crate small.
- **Angles are always radians** inside the crate. Degrees, DMS and bearings are converted at the contract boundary
  (`civkit-units`).
- **Errors through the contract** (D3). The crate's errors and notes are Rust types. A calculation that passes them
  on uses existing codes:

  | Crate outcome | Contract code |
  |---|---|
  | non-finite input | `INVALID_VALUE` |
  | any other geometry error (undefined result) | `OUT_OF_DOMAIN` |
  | precision-limited input (§8.9) | warning `ROUNDING_SENSITIVE` (`caution`) |

  "Not supported in 0.1" (centroid of a curved polygon) has no matching code. It is not reachable through the
  contract in 0.1 (see ADR 0010).

## 3. Coordinate systems and axes

There is one mathematical frame: a **right-handed plane Cartesian frame**. The +y axis is +x rotated +90° about +z,
where +z points towards the viewer looking down on the plan. "Counter-clockwise" always means as seen from +z.

CivKit uses this frame under two names. The arithmetic is identical; only the vocabulary differs.

| Frame | x axis | y axis | Used by |
|---|---|---|---|
| Drawing (DXF world coordinates, WCS, projected on z = 0) | right | up | drawing analysis |
| Survey (plane grid) | **Easting (E)** | **Northing (N)** | traverse, levelling plan positions |

Rules:

- **Order is always (x, y) = (E, N).** Many survey texts write coordinates as (N, E); CivKit never does in its API.
  Survey calculations name their inputs `northing` and `easting` explicitly, never `x`/`y` or position 0/1, so the
  order cannot be confused.
- **Latitude = ΔN = Δy; departure = ΔE = Δx.** This is the traverse convention of reference case C3.
- **Both frames are right-handed with the same orientation**, so "counter-clockwise" means the same thing in both:
  turning from East towards North, or from right towards up.
- **z is ignored.** Drawing entities are projected onto z = 0. Entities with non-zero z, or with a non-default
  extrusion direction (DXF OCS), are the drawing layer's concern (Phase 4; see §11). `civkit-geometry` only sees
  2D points.
- **No geodesy.** Distances and areas are plane values on the grid. Every calculation that uses survey coordinates
  states this as an assumption.

## 4. Angle conventions

Three angles occur. Each has one name, one zero direction and one positive sense.

| Name | Zero direction | Positive sense | Range returned | Where |
|---|---|---|---|---|
| **Direction angle θ** | +x (East / right) | counter-clockwise | [0, 2π) | geometry internals; DXF arc angles |
| **Azimuth α** | +y (North / up) | **clockwise** | [0, 2π) | surveying; bearings convert to it |
| **Sweep Δ** (arcs and bulges) | the start direction | counter-clockwise positive, clockwise negative | (−2π, 2π) | arcs, bulged segments |

**Normalisation** of any finite angle a:

1. r = a.rem_euclid(TAU), with TAU the `f64` constant for 2π.
2. If r ≥ TAU, r becomes 0. `rem_euclid` can return exactly TAU when a is a tiny negative number.
3. Negative zero becomes +0. `atan2` returns −0 for directions along the zero axis approached from the negative
   side.

No other value is changed: angles are never compared with a tolerance (§5).

**Conversion:** α = normalise(π/2 − θ), and θ = normalise(π/2 − α).

**Direction from components:**

- α = atan2(Δx, Δy). Note the order: East component first.
- θ = atan2(Δy, Δx).
- Each is then normalised.
- α is computed directly with atan2, never by converting θ.

**Components from azimuth:** Δx = d · sin α (departure), Δy = d · cos α (latitude). A component of −0 is returned as +0.

**Quadrant bearings** are converted to azimuths by `civkit-units` ([API_DESIGN.md §7.1.4](API_DESIGN.md)). The
geometry crate only sees azimuths.

**Arcs** are defined by centre, radius r and start direction θ₀, plus a signed sweep Δ.

- The arc runs from θ₀ to θ₀ + Δ: Δ > 0 is counter-clockwise, Δ < 0 clockwise.
- Valid input: r > 0 and 0 ≤ |Δ| < 2π. Anything else follows §8.8.
- The DXF ARC entity is always counter-clockwise from start to end angle. That mapping belongs to Phase 4 (§11).

**Bulge** (DXF polyline arcs): b = tan(Δ/4) for the segment from vertex i to vertex i+1.

- b = 0 is a straight segment.
- b > 0 is a counter-clockwise arc, bulging to the right when travelling along the segment. b < 0 is clockwise.
- |b| = 1 is a semicircle; |b| > 1 is an arc larger than a semicircle.
- Δ = 4·atan(b), so |Δ| < 2π always.
- Quantities derived from a bulge are computed from b, never from sin(Δ/2), which loses accuracy as |Δ| → 2π.
  With chord length c and half-chord a = c/2:
  - radius r = (a/2)·(|b| + 1/|b|), which equals c(1 + b²)/(4|b|);
  - signed offset of the centre from the chord midpoint, along the chord's left normal:
    k = (a/2)·(1/b − b), which equals c(1 − b²)/(4b);
  - sagitta h = |b|·a;
  - sin(Δ/2) = 2b/(1 + b²).

## 5. Tolerances

**No hidden epsilon.** Every operation that compares lengths either takes a tolerance explicitly or uses the
default below. The value used is returned in its result. A calculation records it in its trace only when a
tolerance decision affected the result (D2).

**Linear tolerance ε** (in the input's length unit). Default:

  ε = max( 10⁻⁹ × D , 1024 × ulp(M) )

where:

- **D** is the diagonal of the bounding box of the geometry passed in one call. The box includes the extreme points
  of each arc, not only the vertices.
- **M** is the largest absolute vertex coordinate. For an arc or circle, the points used are its centre and the
  points that define its bounding box.
- **ulp(x)** is the distance from |x| to the next larger `f64`. ulp(0) is the smallest positive subnormal.

The two terms:

- **Size term** (10⁻⁹ × D): relative to the size of the object. It is about 4.5 × 10⁶ times machine epsilon (2⁻⁵²),
  which leaves room for rounding in sums of many terms.
- **Precision floor** (1024 × ulp(M)): coordinates far from the origin are themselves only known to ulp(M). Areas
  and centroids are computed after translating to a local origin (§10), so this floor, not M itself, is the real
  limit.

Examples:

- Fixture F1 (60 m × 45 m) near the origin: D = 75 m → ε = 7.5 × 10⁻⁸ m.
- The same plan at state-plane coordinates near (500 000, 4 000 000) m: ulp(4 × 10⁶) ≈ 4.66 × 10⁻¹⁰ m, so
  ε ≈ 4.77 × 10⁻⁷ m (about 0.5 µm).

**ε is computed per call.** It depends on the extent of the whole input, so the same short segment can be
zero-length in a large polyline and not in a small one. This is intended.

**Explicit tolerance.** A caller may pass ε explicitly, for example an engineering tolerance in known units.

- It must be finite and ≥ 0, otherwise it is an error. It may be 0.
- D and M are still computed. ε_A uses the explicit ε.
- The precision-limited check (§8.9) does not depend on ε.

**Area tolerance:** ε_A = ε × D.

**Derived tolerances must be finite.** If ε × D overflows `f64`, the call fails with an overflow error (§8.1); it
never returns an infinite tolerance or an orientation made "undefined" by one. This holds for every operation that
computes a tolerance, including lengths and direction. With an explicit ε it can happen for any large ε; with the
default (ε_A ≈ 10⁻⁹·D²) only for a size D above about 4 × 10¹⁵⁸.

**Angles** are not compared with a tolerance. Every angular decision is made through lengths.

**Reference-case tolerances** (in `tests/reference/`) are a separate thing. They reflect a source's printed
precision and are not geometry tolerances.

## 6. Point equality and the uses of ε

- **Two points are equal when their distance is ≤ ε** (Euclidean, §7). Exact `==` is never used on computed
  coordinates.
- This relation is **not transitive**: p ≈ q and q ≈ r do not imply p ≈ r. No algorithm relies on transitivity.
  Points are compared only with their neighbours in sequence, and there is no global clustering or snapping.
- ε and ε_A are used only for:
  - zero-length segments (§8.2);
  - explicit closure (§8.4);
  - degenerate rings (§8.6);
  - coincident points for direction (§7);
  - self-intersection (§8.7);
  - orientation (§10).

## 7. Distance and direction between two points

- **Distance:** d = hypot(Δx, Δy). `hypot` avoids overflow and underflow; the result is always ≥ 0.
- **Direction:** azimuth α (and θ) as in §4.
  - If d ≤ ε the direction is undefined. Asking for it is an error, never 0 or NaN.
  - **The two-point rule.** For two points D = d, so only the precision floor can decide. The test is
    d ≤ 1024·ulp(M), with M the largest absolute coordinate of the two points.
  - A caller may pass ε explicitly (§5).
- **Direction from components** (Δx, Δy) alone is undefined only when both are exactly 0. Whether a computed
  difference is zero in engineering terms (for example a traverse misclosure) is decided by the calling
  calculation, not by this crate.
- **Components from azimuth and distance** (§4): defined for every finite α and d ≥ 0. A negative distance is an
  error.

## 8. Degeneracy and invalid input

Every degenerate or invalid case has one defined behaviour. None is handled silently. "Reported" means a note in the
result (Rust only in 0.1, D3).

| # | Case | Definition | Behaviour |
|---|---|---|---|
| 8.1 | Non-finite input or result | any coordinate, radius, bulge, angle, distance or explicit tolerance is NaN or ±∞; or a result, or a derived tolerance (ε_A, §5), overflows `f64` | **error** |
| 8.2 | Zero-length segment | straight segment with chord c ≤ ε; bulged segment with c ≤ ε and arc length s ≤ ε (§9). c = 0 exactly with b ≠ 0 has r = 0 and belongs here | contributes its computed length (≤ ε) and area (no rounding); **reported**, except the explicit closing duplicate (8.4) |
| 8.2a | Ill-conditioned bulge | c ≤ ε < s: the radius would be set by a chord below tolerance | **error** |
| 8.3 | Collinear vertices | perpendicular distance of the middle vertex from the line through its neighbours ≤ ε, **and** the middle vertex lies between them | **allowed**; no special handling (it changes nothing in length or area). A collinear vertex outside its neighbours folds the ring back (a spike) and is handled by 8.7 |
| 8.4 | Explicit closure | a ring whose last vertex equals its first within ε | treated as closed. For validity checks the duplicate is dropped; its bulge describes the zero-length closing segment and is checked as in 8.2/8.2a. Not reported |
| 8.5 | Too few vertices | polyline: fewer than 2 vertices. Polygon, on the cleaned ring (§10): fewer than 3 vertices if every edge is straight, or fewer than 2 if any edge is bulged. A closed two-vertex polyline with bulged edges is valid; AutoCAD's DONUT draws one | **error** |
| 8.6 | Degenerate ring **(impl.)** | every edge of the cleaned ring is straight or has sagitta ≤ ε, and every vertex lies within ε of the line through the first vertex and the vertex farthest from it (so the whole ring lies within 2ε of one line) | area returned **as computed** (D8), orientation **undefined**, centroid **error**; reported. 8.7 is not applied: a degenerate ring always folds back on itself |
| 8.7 | Self-intersecting ring | on the cleaned ring: two **non-adjacent** edges come within ε of each other anywhere; or two **adjacent** edges conflict **(impl.)** | area and centroid: **error**; length is unaffected. This refuses some rings whose net area would be right (a pinch, a keyhole; D4) |
| 8.8 | Invalid or degenerate arc or circle | r < 0; arc with \|Δ\| ≥ 2π; r ≤ ε; sweep 0 | zero sweep (with r ≥ 0): length 0, **reported**. \|Δ\| ≥ 2π: **error** (use a circle). r < 0, or r ≤ ε with non-zero sweep: **error**. Circle with r ≤ ε: **error** |
| 8.9 | Precision-limited input | D > 0 and 1024 × ulp(M) > 10⁻⁶ × D: the coordinates are so large, relative to the object's size, that a millionth of its size is below what `f64` can represent there | allowed but **reported**; results are still computed in the translated frame (§10) |

**What "conflict" means for adjacent edges in 8.7 (impl.).** Two adjacent edges share a vertex v (in a two-vertex
ring they share both). They conflict when either:

- the far endpoint of one lies within ε of the other edge; or
- they cross, or reach a local minimum of their distance that is ≤ ε, at a point more than ε from every shared
  vertex.

Tangential contact that exists only at v is not a conflict.

**Bulged edges in 8.7** (D7) are tested with exact closed-form point–arc, segment–arc and arc–arc relations. These
are computed in each arc's own frame (chord midpoint, chord direction), so a nearly straight arc of huge radius loses
no accuracy.

- A bulged edge whose sagitta is at most ulp(M) is checked as its chord **(impl.)**. Below the representation
  limit, the arc and its chord cannot be told apart.
- Every ring edge is still tested. **(D-28, Phase 4)** A sweep over the edges' bounding boxes, each inflated by ε,
  is a prefilter: only pairs whose boxes overlap are tested exactly. A pair whose boxes do not overlap cannot
  conflict, so results are identical to testing all pairs. The worst case stays O(n²); callers that need a time
  bound (drawing analysis) pass a limit on pair tests, and exceeding it is reported, not a validity result.
  - A box covers a line's ends, or an arc's ends and its extremes (§5), widened by ε on every side. Two edges whose
    widened boxes do not overlap are more than 2ε apart.
  - **Work** is the number of edge pairs the sweep examines: pairs whose widened x-ranges overlap (the sweep runs
    in x). It bounds the sweep's time even when many boxes overlap in x but not in y. **(impl.)**
  - `polygon_area_limited` and `region_area_limited` draw on a caller's `WorkLimit`, shared across calls;
    `polygon_area`, `region_area` and `polygon_centroid` are unlimited. A region is charged each ring's own check,
    then the pairs of edges from different rings.
  - When the work would pass the limit, the result is `WorkLimitExceeded`: the check was not completed, so the
    geometry is reported neither valid nor self-intersecting, even if a conflict was already found.
  - The error names the first conflicting pair in (i, j) order (the first pair of rings, for a region), as testing
    every pair in order does, whatever order the sweep meets them in.

**Order of evaluation for a polygon:**

1. 8.1, non-finite input.
2. 8.2a, ill-conditioned bulges.
3. Cleaning: 8.2 and 8.4.
4. 8.5, too few vertices.
5. 8.6, degenerate ring.
6. 8.7, self-intersection.
7. Orientation (§10).

The first error found is returned.

## 9. Length

- **Straight segment:** chord c = distance between its vertices.
- **Bulged segment:** s = c·(1 + b²)·atan|b| / |b| (b ≠ 0).
  - This equals r·|Δ|, has no cancellation for any finite b, and needs no series.
  - In code: c·(1 + b²)·(atan|b|/|b|) for |b| ≤ 1, and c·atan|b|·(|b| + 1/|b|) for |b| > 1. Neither form
    overflows before the result itself does.
  - The form c·(Δ/2)/sin(Δ/2) is not used. Its relative error grows to 2 × 10⁻¹¹ at b = 10⁶ and 7 × 10⁻⁵ at
    b = 10¹².
- **Polyline length:** the sum of its segment lengths.
  - A closed polyline includes the closing segment, with the last vertex's bulge.
  - An open polyline does not use the last vertex's bulge, but it must still be finite.
- **Arc:** s = r · |Δ|. **Circle:** circumference 2πr.
- **Summation** uses compensated (Neumaier) addition. It removes accumulation across many segments. It does not
  remove cancellation inside a term; §10's translation does that.

## 10. Area, orientation and centroid

**Cleaned ring.** For validity checks (8.5–8.7) a ring is cleaned:

- The start vertex of each zero-length segment (8.2), including the explicit closing duplicate (8.4), is removed.
- The segment before it then ends at the next kept vertex and keeps its own bulge.
- Cleaning repeats until no zero-length segment remains.
- **(impl., D-28)** Cleaning is one linear pass with the same result as repeating "remove the first zero-length
  segment's start vertex": removing a vertex changes only the segment before it, so the scan steps back one vertex
  (or restarts at the new first vertex). At most 2n steps; a property test compares it with the repeated search.

Area and centroid are computed on the vertices **as given**. A zero-length segment contributes its tiny exact share,
so nothing is rounded away (D8).

**Signed area** of a closed ring:

- **Precision step:** all vertices are first translated so the first vertex is the origin. Area is
  translation-invariant, and this removes the cancellation that would otherwise cost about half the significant
  digits at state-plane coordinates. The translated coordinates are exact whenever a vertex and the first vertex
  are within a factor of 2 of each other (Sterbenz lemma), as at state-plane magnitudes. The only remaining error
  is the input's own representation.
- **Straight edges:** shoelace sum, A = ½ Σ (xᵢ·yᵢ₊₁ − xᵢ₊₁·yᵢ).
- **Bulged edges:** each adds the signed area of its circular segment, sign(b)·(r²/2)·g(|Δ|), where
  g(x) = x − sin x.
  - For x ≥ 0.5, it is computed as (r²/2)·(x − sin x), with r from §4.
  - For x < 0.5, it is computed as (c²(1 + b²)²/32)·(x/|b|)²·x·T(x), with
    T(x) = 1/3! − x²/5! + x⁴/7! − …, summed until a term falls below 2⁻⁵³ of the sum. This is the same quantity,
    with no cancellation and no overflow for tiny b. Direct subtraction loses all significant digits for x ≲ 10⁻⁵.
  - A positive bulge lies to the right of its chord. For a counter-clockwise ring that is outward, so it adds area;
    a negative bulge cuts area away.
  - Check: a 2 × 2 counter-clockwise square whose bottom edge is a semicircle (b = +1) has area 4 + π/2.
- Compensated summation, as for length.

**Area:** the reported area is |A|.

**Orientation:**

- A > ε_A means **counter-clockwise**; A < −ε_A means **clockwise**. Otherwise it is undefined and reported.
- Because both frames are right-handed (§3), counter-clockwise means East → North in survey coordinates and
  right → up in a drawing.
- Survey texts often describe traverses as clockwise. The reported area is the same either way; only the
  orientation flag differs.

**Circle area:** πr².

**Centroid** (straight-edged polygons only in 0.1; D5):

- Cx = (1/6A) Σ (xᵢ + xᵢ₊₁)(xᵢ·yᵢ₊₁ − xᵢ₊₁·yᵢ), and likewise for Cy. It is computed in the translated frame
  and translated back.
- Here A is the **signed** area from the same translated sum, not the reported |A|. Using |A| would reflect the
  centroid of a clockwise ring through the first vertex.
- **Any non-zero bulge** on an edge of the ring gives a "not supported in 0.1" error. This is checked right after
  8.1, before the other checks.
- It is also an error when the ring is degenerate (8.6), not simple (8.7), or has undefined orientation.

## 11. Transformations and OCS (Phase 4, D-28)

Fixed in 2B so the drawing layer could not drift; specified in full by [ADR 0016](../planning/decisions/0016-drawing-analysis.md)
(D-28). Implemented in `civkit-geometry` in Phase 4.

- **`Affine2`** `{a, b, c, d, tx, ty}` maps (x, y) to (a·x + b·y + tx, c·x + d·y + ty). It has composition, the
  determinant, and a similarity test: the linear part's singular values σ₁ ≥ σ₂ satisfy σ₁ − σ₂ ≤ 10⁻⁹·σ₁. Under a
  similarity, the scale is s = √|det|.
- **Orientation:** a transform with det < 0 is a mirror. It negates bulges, and signed areas change sign. det = 0
  is degenerate and is not applied (the drawing layer reports it).
- **Block inserts:** a block point p is placed by `p' = P + R(θ)·((c·Δc, r·Δr) + S(sx, sy)·(p − B))`, where:
  - B is the block base point and P the insertion point;
  - θ is the rotation, counter-clockwise;
  - Δc and Δr are the MINSERT column and row spacing, unscaled and along the rotated axes;
  - c and r are the column and row indices (0 for a single INSERT).

  Nested inserts compose, the outermost applied last.
- **Lengths and areas under a transform:**
  - area scales by |det| for any affine map;
  - length scales by s under a similarity;
  - under a non-uniform scale, straight segments are transformed and measured, but a circular arc would become
    elliptical, so arcs, circles and bulged segments are **not supported in 0.1**, never approximated.
- **Extrusion (OCS):**
  - (0, 0, 1), within 10⁻⁹, is the plan itself.
  - (0, 0, −1), common after mirroring, maps to world coordinates by negating x (Autodesk arbitrary axis
    algorithm). Like a mirror, it reverses orientation and bulge signs.
  - Any other extrusion is not a plan entity, and the drawing layer reports it.
  - z (elevation, sz) is not used: quantities are plan quantities.
- **DXF ARC** (always counter-clockwise from start to end angle): Δ = (end − start) mod 360°, computed exactly in
  degrees as stated, then converted. Δ = 0 is a **zero-sweep arc**: length 0 with the note of §8.8, not a full
  circle.
- **`apply_vertex`** maps a polyline vertex. det = 0 is an error (degenerate transform); a bulged vertex under a
  transform that is not a similarity is an error (curve under non-uniform scale); results that overflow are errors
  (§8.1).
- **Regions (`region_area`):** rings combined by even-odd nesting. A ring at nesting depth k counts with sign (−1)ᵏ.
  Rings that cross or touch each other (within ε, by the relations of §8.7) are not supported. A ring's depth is
  found by testing a point of it against the other rings.
  - One tolerance for the region: D from all rings (arc extremes included), M from all coordinates. Each ring is
    checked on its own (§8) with that ε; the first invalid ring, in input order, gives its error.
  - Containment **(impl.)**: the parity of a ray from the point equals the parity of the chord polygon, flipped once
    for each circular segment (between a bulged edge's chord and its arc) that contains the point. That rule is
    ill-defined on a bulged chord, so the point tested is a vertex or edge midpoint of the ring that lies more than ε
    from every other ring's bulged chords (every point of a ring that crosses no other ring has the same
    containment).

## 12. How 2B is verified

Expected values never come from the code under test. Each one is a closed form, a 60-digit value computed outside
the repository, or a result from another program. Each records where it came from.

1. **Closed forms:**
   - rectangles and right triangles (exact on integer coordinates);
   - F1 boundary: area 2550, perimeter 200, centroid (1460/51, 365/17);
   - F1 road centreline 100 + 10π, drain 5π + 25, circle R = 3 (9π and 6π), quarter arc R = 20 (10π);
   - square with a semicircular edge: 4 ± π/2;
   - two-vertex rings: bulges (1, 1) give π, bulges (1, 0) give π/2.
2. **High-precision oracle for bulges** (chord 10; b from 10⁻¹⁵ to 10¹², both signs), for length and segment
   area, at 10⁻¹³ relative.
3. **Angles:**
   - normalisation of −10⁻¹⁷, −4 × 10⁻¹⁶, −0.0 (check the sign bit), TAU, 7·TAU + 1 and −π;
   - all `atan2` sign cases, including ±0;
   - cardinal azimuths;
   - the α ↔ θ ↔ components round trip.
4. **Large magnitudes:**
   - F1 translated by (5 × 10⁵, 4 × 10⁶) gives area exactly 2550;
   - F1 × 1000 at (5 × 10⁸, 4 × 10⁹) gives exactly 2.55 × 10⁹;
   - a guard at 4 × 10¹² that fails without translation.
5. **Every row of §8**, including:
   - two-vertex rings;
   - the tiny chord with a huge bulge;
   - spike, pinch, keyhole, figure-8;
   - a bulged edge crossing another edge;
   - near misses just outside and inside ε.
6. **Property tests:**
   - exact invariance under dyadic translations;
   - reversal negates the signed area;
   - scaling by powers of 2 multiplies length and area exactly and keeps every decision;
   - rotation within 10⁻¹²;
   - the arc distance routines against dense sampling.
7. **No bit-exact expectations** for anything through `sin`, `cos`, `atan2` or `hypot`, whose precision Rust
   documents as platform-dependent.
8. **Mutation check:** planted errors must make a test fail:
   - a flipped bulge sign;
   - |A| in the centroid;
   - a skipped translation;
   - the old g(x);
   - a missing TAU wrap.
9. **§11, Phase 4** (gates A3 and A7 of ADR 0016), `tests/transforms.rs` and `tests/regions.rs`:
   - the insert formula, MINSERT spacing and nested placement, worked by hand step by step;
   - exact quarter turns; composition order; mirrors negate bulges (square with a semicircular edge stays 4 + π/2);
   - areas scale by |det|; singular values of a shear (φ and 1/φ); the similarity threshold either side;
   - regions: holes and islands with every orientation and input order; discs with square and disc holes
     (25π − 4, 24π); holes inside an outward bulge and islands inside an inward bulge (containment follows the arc);
     crossing and touching rings refused;
   - property tests: composition equals sequential application (exact on integers); concentric squares alternate;
   - mutation check: dropping the bulge negation, reversing the composition, or ignoring circular segments in
     containment each make a test fail.
10. **§8.7 prefilter, Phase 4** (gate A7), `src/sweep.rs` (unit tests) and `tests/sweep.rs`:
    - property tests: on random rings (integer coordinates nudged by 0, ±ε/2 or ±5ε; minor and major arcs of both
      signs), the sweep finds exactly the conflicting pairs that testing every pair finds, and is charged exactly
      the pairs whose widened x-ranges overlap; pairs whose widened boxes do not overlap never conflict;
    - widened boxes: rings δ apart are refused for δ = 10⁻⁹ < ε and accepted for δ = 10⁻⁶;
    - arc extremes: a ring under an arc's lowest point, whose box the chord's box would miss;
    - the work limit at its boundary and one below (a square costs 5, a ring with a hole 18), shared across calls;
      over the limit, a bow tie is `WorkLimitExceeded`, not self-intersecting; a 40 000-vertex meander of
      overlapping rows stops at its limit; a regular 50 000-gon costs under 10 pairs per edge;
    - the first crossing in order is reported for a ring and for a region with two crossings;
    - mutation check: unwidened boxes, ignored arc extremes, charging only y-overlaps, an unenforced limit, and
      reporting the first crossing found (ring or region) each make a test fail.
11. **Linear cleaning (§10, D-28):** a property test against the repeated-search loop (few points, many exact and
    near repeats, bulges); a triangle with every vertex repeated 300 000 times cleans to area 1/2. Stepping forward
    instead of back after a removal makes the property test fail.

## 13. Decisions

The open questions of the proposed version (Q1–Q6), and D7–D9 raised in the review, are decided in
[ADR 0010](../planning/decisions/0010-geometry-conventions.md) (D-22).
