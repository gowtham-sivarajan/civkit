# Geometry specification (Phase 2B, `civkit-geometry`)

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-25 16:10 IST · last edited 2026-11-25 16:10 IST

> **Status: Proposed. Awaiting maintainer review.** No geometry code is written until this document is accepted
> (maintainer, 2026-11-25). Open questions for the review are collected in §13.

This document defines, before any implementation, every convention `civkit-geometry` follows: coordinate systems,
axes, angles, tolerances, point equality, degeneracy, distance, length, area, orientation and centroid. Like
[API_DESIGN.md §7.1](API_DESIGN.md), it is meant to be complete: behaviour not described here is not implemented.

## 1. Scope

**In 2B** (from [MVP_SCOPE.md §3](../planning/MVP_SCOPE.md), "Geometry"), plane 2D only:

| Capability | Used by |
|---|---|
| Distance and azimuth between two points; components (Δx, Δy) | Traverse (2C), drawing analysis (Phase 4) |
| Components from azimuth and distance (latitude, departure) | Traverse (2C) |
| Polyline length, with straight and circular-arc (bulge) segments | Drawing analysis: F1, F2, F4 road centreline |
| Arc length; circle circumference | Drawing analysis: ARC and CIRCLE entities |
| Polygon area (straight and bulged edges) and orientation | Drawing analysis: F1–F5 areas |
| Polygon centroid (straight-edged polygons only; see §13 Q5) | MVP scope ("polygon area and centroid") |

**Not in 2B:**
- 3D geometry; elevations (z) are not used.
- Intersections, offsets and clipping (except the self-intersection *check* in §8.4).
- Splines and ellipses.
- Coordinate transformations: block inserts with scale, rotation and mirroring are needed by drawing analysis.
  §11 fixes their conventions now so they match, but they are implemented with `civkit-drawing` in Phase 4.
- Map projections, grid scale factors, earth curvature, geodesy.

[ARCHITECTURE.md §3.3](ARCHITECTURE.md) describes the crate's long-term role as "2D/3D … transformations". This
specification narrows 0.1 to the list above, in line with D-20.

## 2. Numbers and units

- All coordinates and lengths are IEEE 754 `f64`. Every input must be finite; a NaN or infinity is rejected
  (§8.1), never propagated.
- **The geometry crate is unit-agnostic.** All lengths in one call must be in the same length unit; results come
  back in that unit (lengths) or its square (areas).
  - Calculations call it with internal SI units (metres), per API_DESIGN §7.1.5.
  - Drawing analysis calls it in drawing units and converts afterwards (Phase 4).
- The crate does not depend on `civkit-units`; units are the caller's responsibility, which keeps the crate small.
- **Angles are always radians** inside the crate. Degrees, DMS and bearings are converted at the contract boundary (`civkit-units`).

## 3. Coordinate systems and axes

There is one mathematical frame: a **right-handed plane Cartesian frame**. The +y axis is +x rotated 90° counter-clockwise.
CivKit uses it under two names. The arithmetic is identical; only the vocabulary differs.

| Frame | x axis | y axis | Used by |
|---|---|---|---|
| Drawing (DXF world coordinates, WCS, projected on z = 0) | right | up | drawing analysis |
| Survey (plane grid) | **Easting (E)** | **Northing (N)** | traverse, levelling plan positions |

Rules:
- **Order is always (x, y) = (E, N).** Many survey texts write coordinates as (N, E); CivKit never does in its API.
  Survey calculations name their inputs `northing` and `easting` explicitly, never `x`/`y` or position 0/1, so
  order cannot be confused.
- **Latitude = ΔN = Δy; departure = ΔE = Δx** (the traverse convention of reference case C3).
- **Both frames are right-handed with the same orientation**, so "counter-clockwise" means the same thing in
  both: turning from East towards North, or from right towards up.
- **z is ignored.** Drawing entities are projected onto z = 0. Entities with non-zero z, or with a non-default
  extrusion direction (DXF OCS), are the drawing layer's concern (Phase 4, warnings).
  `civkit-geometry` only sees 2D points.
- No geodesy: distances and areas are plane values on the grid. This is stated as an assumption by every
  calculation that uses survey coordinates.

## 4. Angle conventions

Three angles occur. Each has one name, one zero direction and one positive sense.

| Name | Zero direction | Positive sense | Range returned | Where |
|---|---|---|---|---|
| **Direction angle θ** | +x (East / right) | counter-clockwise | [0, 2π) | geometry internals; DXF arc angles |
| **Azimuth α** | +y (North / up) | **clockwise** | [0, 2π) | surveying; bearings convert to it |
| **Sweep Δ** (arcs and bulges) | the start direction | counter-clockwise positive, clockwise negative | (−2π, 2π) | arcs, bulged segments |

- **Conversion:** α = (π/2 − θ) mod 2π, and θ = (π/2 − α) mod 2π. The result is normalised into [0, 2π); values
  within 1e-15 rad below 2π become 0 so that "north" is never reported as 2π.
- **Direction from components:** α = atan2(Δx, Δy) (note the order: East component first) and θ = atan2(Δy, Δx),
  each then normalised to [0, 2π).
- **Components from azimuth:** Δx = d · sin α (departure), Δy = d · cos α (latitude).
- **Quadrant bearings** are converted to azimuths by `civkit-units` ([API_DESIGN.md §7.1.4](API_DESIGN.md)); the
  geometry crate only sees azimuths.
- **Arcs** are defined by centre, radius r > 0, start direction θ₀ and signed sweep Δ. The arc runs from θ₀ to
  θ₀ + Δ: Δ > 0 counter-clockwise, Δ < 0 clockwise. (The DXF ARC entity is always counter-clockwise from start to
  end angle; that mapping belongs to Phase 4.)
- **Bulge** (DXF polyline arcs): b = tan(Δ/4) for the segment from vertex i to vertex i+1.
  - b = 0 is a straight segment.
  - b > 0 is a counter-clockwise arc (bulging to the right when travelling along the segment), b < 0 clockwise.
  - |b| = 1 is a semicircle; |b| > 1 is an arc larger than a semicircle.
  - Δ = 4·atan(b), so |Δ| < 2π always.

## 5. Tolerances

- **No hidden epsilon.** Every operation that compares lengths takes a tolerance explicitly or uses the default
  below, and the value used is recorded in its result (and, for calculations, in the trace).
- **Linear tolerance ε (length units of the input).** Default:

  ε = max( 10⁻⁹ × D , 1024 × ulp(M) )

  where D is the diagonal of the input's bounding box (its size), M the largest absolute coordinate, and
  ulp(M) the spacing of `f64` values near M (the representation limit of the input itself).
  - **Size term** (10⁻⁹ × D): relative to the size of the object. It is about 4.5 × 10⁶ times machine
    precision, which leaves room for rounding in sums of many terms.
  - **Precision floor** (1024 × ulp(M)): coordinates far from the origin are themselves only known to ulp(M).
    Areas and centroids are computed after translating to a local origin (§10), so this floor, not M itself, is
    the real limit.
  - Examples:
    - a 60 m site plan near the origin: D ≈ 100 m → ε ≈ 1 × 10⁻⁷ m (0.1 µm)
    - the same plan at state-plane coordinates near (500 000, 4 000 000) m: ulp(4 × 10⁶) ≈ 4.7 × 10⁻¹⁰ m, so
      ε ≈ 4.8 × 10⁻⁷ m (about 0.5 µm). Scaling by M instead would have given about 4 mm, needlessly coarse.
- **Area tolerance:** ε_A = ε × D.
- **Angles** are not compared with a tolerance in 2B. Every angular decision is made through lengths, e.g.
  collinearity via perpendicular distance (§8.3).
- Reference-case tolerances (in `tests/reference/`) are a separate thing: they reflect a source's printed
  precision and are not geometry tolerances.

## 6. Point equality

- **Two points are equal when their distance is ≤ ε** (Euclidean, §7). Exact `==` is never used on computed
  coordinates.
- This relation is **not transitive** (p ≈ q and q ≈ r do not imply p ≈ r). No algorithm relies on transitivity:
  points are compared only with their neighbours in sequence, and there is no global clustering or snapping.
- Equality is used for three things only:
  - detecting zero-length segments (§8.2)
  - detecting an explicitly closed ring, where the last vertex equals the first (§8.4)
  - refusing a direction between coincident points (§7)

## 7. Distance and direction between two points

- **Distance:** d = hypot(Δx, Δy). Uses `hypot` to avoid overflow and underflow; always ≥ 0.
- **Direction:** azimuth α (and θ) as in §4.
  - If d ≤ ε the direction is undefined (ε from §5, computed over the two points). Asking for it is an error, never 0 or NaN.
  - The distance itself is still returned (it is ≈ 0).
- **Components from azimuth and distance** (§4): defined for every finite α and d ≥ 0. A negative distance is an error.

## 8. Degeneracy and invalid input

Every degenerate or invalid case has one defined behaviour. None is handled silently.

| # | Case | Definition | Behaviour |
|---|---|---|---|
| 8.1 | Non-finite input | any coordinate, radius, bulge or angle is NaN or ±∞ | **error** (invalid value) |
| 8.2 | Zero-length segment | consecutive vertices equal within ε (§6) | contributes 0 length and 0 area; **reported** as a degeneracy note (§13 Q3) |
| 8.3 | Collinear vertices | perpendicular distance of the middle vertex from the line through its neighbours ≤ ε | **allowed**; no special handling (it changes nothing in length or area) |
| 8.4 | Explicit closure | a ring whose last vertex equals its first within ε | treated as closed; the duplicate closing vertex is dropped, so the closing edge is not counted twice |
| 8.5 | Too few vertices | polyline: < 2 vertices; polygon: < 3 vertices remaining after 8.2 and 8.4 | **error** |
| 8.6 | Zero-area polygon | \|signed area\| ≤ ε_A (e.g. all vertices collinear) | area returned as 0, orientation **undefined**, centroid **error**; reported (§13 Q3) |
| 8.7 | Self-intersecting polygon | any two non-adjacent edges intersect or touch, or adjacent edges overlap | area and centroid: **error** (the shoelace sum of a figure-8 is a net value, not an area); length is unaffected (§13 Q4) |
| 8.8 | Degenerate arc | radius ≤ ε, or sweep = 0 | zero sweep: length 0, reported; radius ≤ ε with non-zero sweep: **error** |
| 8.9 | Precision-limited input | the precision floor dominates badly: 1024 × ulp(M) > 10⁻⁶ × D (coordinates so large, relative to the object's size, that a millionth of its size is below what `f64` can represent there) | allowed but **reported**; results are still computed in the translated frame (§10) |

## 9. Length

- **Straight segment:** chord c = distance between its vertices.
- **Bulged segment:** with Δ = 4·atan(b), the arc length is s = c · (Δ/2) / sin(Δ/2), using the absolute value of Δ.
  - This form is used instead of r·|Δ| because it stays accurate as b → 0, where s → c and r → ∞.
  - For |Δ| < 1 × 10⁻⁴ it uses the series s = c · (1 + Δ²/24 + 7Δ⁴/5760), so it never divides by a tiny sine.
- **Polyline length:** the sum of its segment lengths. A closed polyline includes the closing segment, with the
  closing vertex's bulge.
- **Arc:** s = r · |Δ|. **Circle:** 2πr.
- Summation uses compensated (Kahan–Babuška/Neumaier) addition, so long polylines do not accumulate rounding.

## 10. Area, orientation and centroid

**Signed area** of a closed ring:
- **Precision step:** all vertices are first translated so the first vertex is the origin. Area is
  translation-invariant, and this removes the cancellation that would otherwise cost about half the significant
  digits at state-plane coordinates.
- **Straight edges:** shoelace sum, A = ½ Σ (xᵢ·yᵢ₊₁ − xᵢ₊₁·yᵢ).
- **Bulged edges:** each adds the signed area of its circular segment, ± (r²/2)·(|Δ| − sin |Δ|) with
  r = c / (2 sin(|Δ|/2)), taking the sign of Δ.
  - A positive bulge lies to the right of its chord. For a counter-clockwise ring that is outward, so it adds area;
    a negative bulge cuts area away.
  - Check: a 2 × 2 counter-clockwise square whose bottom edge is a semicircle (b = +1) has area 4 + π/2.
- Compensated summation, as for length.

**Area:** the reported area is |A|.

**Orientation:**
- A > ε_A means **counter-clockwise**; A < −ε_A means **clockwise**; otherwise undefined (8.6).
- Because both frames are right-handed (§3), counter-clockwise means East → North in survey coordinates and
  right → up in a drawing.
- Survey texts often describe traverses as clockwise. The reported area is the same either way; only the
  orientation flag differs.

**Circle area:** πr².

**Centroid** (straight-edged polygons only in 0.1; see §13 Q5):
- Cx = (1/6A) Σ (xᵢ + xᵢ₊₁)(xᵢ·yᵢ₊₁ − xᵢ₊₁·yᵢ), and likewise for Cy, computed in the translated frame and translated back.
- Defined only when the area is not degenerate (8.6) and the polygon is simple (8.7). Otherwise it is an error.

## 11. Conventions fixed now for Phase 4 (not implemented in 2B)

These are recorded so the drawing layer cannot drift from the geometry conventions:
- **Block inserts:** each insert applies a 2D affine transform: scale (sx, sy), rotation θ (counter-clockwise),
  translation, plus `MINSERT` arrays.
  - A transform whose linear part has a negative determinant (a mirror, e.g. sx < 0 as in fixture F5) reverses
    orientation. Bulge signs are negated, and signed areas change sign.
  - Lengths scale with the transform. A non-uniform scale turns a circular arc into an elliptical one, so arcs
    under non-uniform scale are reported as not supported in 0.1, never approximated silently.
- **DXF ARC** (always counter-clockwise from start to end angle): Δ = (end − start) mod 2π, with start = end treated as defined in Phase 4.

## 12. How 2B will be verified

Tests are designed so the code does not grade its own homework:

1. **Closed forms:**
   - rectangles, right triangles, circles, semicircles (|b| = 1), quarter arcs
   - the hand-derived values of fixtures F1–F5 (boundary area 2550, road length 100 + 10π, …), already
     independently checked with ezdxf and shapely ([tests/fixtures/dxf/README.md](../tests/fixtures/dxf/README.md))
2. **Invariants (property tests):**
   - translation and rotation leave length and area unchanged
   - scaling by k multiplies length by k and area by k²
   - reversing vertex order keeps length, negates signed area and swaps orientation
   - splitting a segment at an interior point keeps length and area
   - α ↔ θ ↔ components round-trip
3. **Precision far from the origin:** the F1 site plan translated to (500 000, 4 000 000) gives the same area and
   length within the documented tolerance.
4. **Every row of §8:** each degenerate case has a test that it produces exactly the stated behaviour.
5. **Bulge edge cases:** b → 0 (the series branch), |b| = 1, |b| ≫ 1 and negative bulges, each against closed forms.

## 13. Open questions for the review

1. **Default tolerance.** Is ε = max(10⁻⁹ × D, 1024 × ulp(M)) (§5) acceptable? It is 0.1 µm on a 100 m plan near
   the origin and about 0.5 µm at state-plane magnitudes.
2. **Explicit tolerance in the contract.** Should calculations that use geometry expose the tolerance as an
   optional input, or keep it internal and record it in the trace only? The proposal is internal plus trace
   (no new inputs in 0.1).
3. **How degeneracy reaches users.** 8.2, 8.6, 8.8 and 8.9 "report" a degeneracy. The proposal is one new warning code,
   `DEGENERATE_GEOMETRY` (level `info` or `caution`), added to API_DESIGN §5. This is a contract change and needs your approval.
4. **Self-intersecting polygons (8.7).** The proposal is an error for area and centroid (refuse, rather than return a
   net shoelace value). The check is O(n²) segment tests, adequate for 0.1 sizes. The alternative is a caution
   warning that still returns the net value.
5. **Centroid scope.** The proposal is centroids for straight-edged polygons only in 0.1. Bulged polygons get an
   "not supported in 0.1" error, because no fixture or reference case verifies a bulged centroid.
6. **ARCHITECTURE.md §3.3** says "2D/3D … transformations". The proposal is to amend it to "2D in 0.1;
   transformations specified here (§11) and implemented with civkit-drawing in Phase 4".
