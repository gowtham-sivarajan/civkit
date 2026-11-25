# Review of the Phase 2B geometry specification

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-25 16:37 IST · last edited 2026-11-25 16:37 IST

> **Status: Review only.** This document reviews [docs/GEOMETRY.md](../docs/GEOMETRY.md) as it stands at commit `6d3d16a`.
> It changes nothing else: no source file, specification, decision record or plan was edited. Every change it proposes
> waits for the maintainer's approval (§6 and §7).

**About independence.** The same model (Claude Opus 5.5) drafted GEOMETRY.md and wrote this review. This is not an
independent reviewer in the human sense. Independence comes from two things instead:

- **Separate numerical re-derivation.** Every formula was re-implemented in separate Python code, outside the
  repository, and checked against:
  - closed forms;
  - 60-digit arithmetic (`mpmath` 1.4.1);
  - an unrelated geometry library (`shapely` 2.2.0, on densely flattened arcs).
- **External sources** for every convention (§10).

Findings that rest only on reasoning are marked as such. Before accepting, have a human spot-check at least the Major
findings. Each one names the numbers that reproduce it.

Files read for this review (current device copies, 2026-11-25):

- docs/GEOMETRY.md, docs/API_DESIGN.md, docs/ARCHITECTURE.md, docs/ENGINEERING_STANDARDS.md
- planning/MVP_SCOPE.md, planning/DECISIONS.md, planning/IMPLEMENTATION_PLAN.md, planning/AI_HANDOVER.md
- For context: tests/fixtures/dxf/README.md and planning/ACCEPTANCE_CRITERIA.md

Naming: "D1–D6" in this review means the six open questions in GEOMETRY.md §13 (Q1–Q6), as the review request named
them. They are not the decision-register entries D-1…D-6. New questions raised here are D7–D9.

---

## 1. Executive assessment

**Verdict: sound in its foundations; not ready to accept as written.**

What holds up:

- The coordinate frame and the angle and bulge conventions are coherent, and they match DXF and survey practice.
- Scope stays inside MVP_SCOPE and D-20.
- Nothing in it breaks the dependency rule or the offline, zero-budget principles.

The formulas are mathematically correct. These were verified independently:

- azimuth from components;
- bulge sign and side;
- arc length;
- circular-segment area and its sign;
- the shoelace formula with translation to a local origin;
- the centroid formula, when it uses the signed area.

The translate-first rule works as claimed. The F1 boundary gives exactly 2550 at offsets up to 4 × 10¹². Without
translation it gives 2560 at 4 × 10⁹ and 0 at 4 × 10¹².

**Findings:** 0 Critical, 9 Major, 13 Minor, 14 Accepted (§5). The Major findings fall into four groups:

1. **Numerics the specification prescribes wrongly.**
   - The circular-segment area formula loses every digit for nearly straight bulges: b ≤ 10⁻⁹ gives 0 instead of
     the true value.
   - The length and area formulas both lose accuracy as the sweep approaches 2π.
   - The series branch is attached to the length formula, which never needed it, instead of the area formula,
     which does (M2).
   - Angle normalisation can return exactly 2π, and it can return −0 (M1).
2. **Rules that reject valid geometry or accept invalid geometry.**
   - Two-vertex closed polylines with bulged edges are rejected, yet AutoCAD's DONUT draws exactly that (M4).
   - A sub-tolerance chord with a huge bulge is treated as zero length, although it can be a 1.6 km arc (M5).
   - The self-intersection check is undefined for bulged edges. A chord-only check passes a ring whose arc crosses
     another edge, and returns a wrong area with the wrong orientation (M6).
3. **Ambiguities with a wrong obvious reading.**
   - Using the reported |A| in the centroid formula reflects the centroid of a clockwise ring through the first
     vertex (M3).
   - The collinearity rule (§8.3) and the overlap rule (§8.7) contradict each other for spikes (M7).
   - "Touch" has no numerical definition (M8).
   - The bounding box D is undefined for bulged input (M9).
4. **No Critical findings.** None of the defects silently corrupts realistic engineering results at the magnitudes in
   the fixtures. M6 comes closest, but it needs invalid input.

**Recommendation:**

1. Approve the corrections in §7 and decide D1–D9 (§6).
2. Then revise GEOMETRY.md and accept it, with a decision record (proposed below).
3. Only then implement 2B, inside the boundary in §8.

---

## 2. Detailed mathematical review

Each row lists GEOMETRY.md's statement, how it was checked, and the verdict. Numbers come from the check scripts (§9.6).

### 2.1 Numbers and units (§2)

| Statement | Check | Verdict |
|---|---|---|
| f64 throughout; non-finite input rejected | Reasoning. JSON cannot carry NaN or ∞ anyway, but the Rust API can, so the rule is needed | **Accepted** |
| Unit-agnostic; no `civkit-units` dependency; radians inside | Consistent with the dependency rule. The calculation boundary converts | **Accepted**; see D1 (no absolute default tolerance) |

### 2.2 Coordinate frame (§3)

| Statement | Check | Verdict |
|---|---|---|
| One right-handed frame; (x, y) = (E, N) | E × N = up, so the frame is right-handed with +z up. Survey texts that use x = N, y = E with clockwise azimuths are the left-handed variant; naming inputs `northing`/`easting` avoids confusion | **Accepted** |
| "+y is +x rotated 90° counter-clockwise" | "Counter-clockwise" only has meaning once the viewing side is fixed | **Minor m9**: say "about +z, seen from +z" |
| Latitude = ΔN, departure = ΔE | Matches the traverse source used for C3 (REFERENCE_SOURCES S12, NAVEDTRA 14070, ch. 7; [R13]) | **Accepted** |
| Non-zero z or non-default extrusion → "Phase 4, warnings" | Extrusion (0, 0, −1) is common after mirroring and maps exactly to WCS by negating x (Autodesk arbitrary axis algorithm, §10 [R3]). A warning-only rule would misread mirrored arcs | **Minor m12** (Phase 4 note only; no 2B effect) |

### 2.3 Angles (§4)

| Statement | Check | Verdict |
|---|---|---|
| α = (π/2 − θ) mod 2π and its inverse | For θ = E, N, W, S the error against exact values is ≤ 1.8 × 10⁻¹⁶ rad | **Accepted** |
| "values within 10⁻¹⁵ below 2π become 0" | `rem_euclid(−1e-17, TAU)`, `(−1e-16, …)` and `(−4e-16, …)` all return **exactly TAU**. Rust documents that `rem_euclid` can return the divisor itself for small negative inputs [R6]. Such values are not "below 2π", so the rule misses them. Separately, `atan2(−0.0, +y)` = −0.0, and `rem_euclid` keeps the sign, so the azimuth "−0" would reach JSON as `-0.0`. The snap also contradicts §5 ("angles are not compared with a tolerance") | **Major M1** |
| α = atan2(Δx, Δy); θ = atan2(Δy, Δx) | Argument order verified. The round trip α → (sin, cos) → atan2 → α has a worst error of 8.9 × 10⁻¹⁶ rad, which is one ulp(2π), over 20 000 random angles | **Accepted**. Compute α directly with atan2, not via θ (m15) |
| Δx = d sin α, Δy = d cos α | Correct. Cardinal azimuths are not exact zeros: a due-south leg of 100 m gets departure 1.22 × 10⁻¹⁴ m | **Accepted**; consequence for tests in m4 |
| Bulge b = tan(Δ/4); b > 0 counter-clockwise, "bulging to the right"; \|b\| = 1 semicircle | Matches the DXF definition [R1, R2]. Numerically, b = +1 on the edge (0,0)→(2,0) passes through y < 0, which is right of travel | **Accepted** |
| \|Δ\| < 2π always | 4·atan(b) lies in (−2π, 2π) | **Accepted** |

### 2.4 Distance and direction (§7)

| Statement | Check | Verdict |
|---|---|---|
| d = hypot(Δx, Δy) | Correct; avoids overflow | **Accepted** |
| Direction undefined if d ≤ ε, "computed over the two points" | For two points D = d, so the size term 10⁻⁹·d can never exceed d unless d = 0. The test is effectively d ≤ 1024·ulp(M). For a direction from components alone, M is undefined. The text suggests a relative check that does not happen | **Minor m10** (clarify). Note for 2C: a traverse that closes exactly has an undefined misclosure direction |

### 2.5 Length (§9)

| Statement | Check | Verdict |
|---|---|---|
| s = c·(\|Δ\|/2)/sin(\|Δ\|/2) | Algebraically equal to r\|Δ\| with r = c/(2 sin(\|Δ\|/2)) | **Accepted** as mathematics |
| "used instead of r\|Δ\| because it stays accurate as b → 0" … series below \|Δ\| = 10⁻⁴ "so it never divides by a tiny sine" | Dividing a small number by a small sine is not unstable: the relative error is preserved. With no series at all, the direct form is accurate to ≤ 1.5 × 10⁻¹⁶ for every b from 10⁻³⁰⁰ to 1. The real loss is as \|Δ\| → 2π: sin(\|Δ\|/2) comes from an argument near π that carries an absolute rounding error. Relative error of s: 5 × 10⁻¹⁴ at b = 10³, 2 × 10⁻¹¹ at 10⁶, 7 × 10⁻⁵ at 10¹², 5 × 10⁻² at 10¹⁵. The form s = c(1 + b²)·atan\|b\|/\|b\| has ≤ 1.6 × 10⁻¹⁶ error everywhere | **Major M2** (the rationale is wrong; the instability is at the other end) |
| Series coefficients 1 + Δ²/24 + 7Δ⁴/5760 | x/sin x = 1 + x²/6 + 7x⁴/360 + …, with x = Δ/2, gives exactly these | **Accepted** (but unnecessary under M2) |
| Arc s = r\|Δ\|; circle 2πr; closed polyline includes the closing bulge | Correct | **Accepted** |
| Neumaier (Kahan–Babuška) summation | Correct naming [R9, R10]. Harmless. It does not touch the dominant error, which is cancellation inside each cross product; translation handles that | **Accepted** (m5: say what it does and does not fix) |

### 2.6 Area, orientation, centroid (§10)

| Statement | Check | Verdict |
|---|---|---|
| Translate to the first vertex, then shoelace | F1 (area 2550) at offsets (0,0), (5e5, 4e6), (5e8, 4e9) and (5e11, 4e12): translated result exactly 2550 every time; naive result 2550, 2550, **2560**, **0**. The translated coordinates are exact (Sterbenz lemma: x − y is exact when y/2 ≤ x ≤ 2y [R11]), so the only remaining error is the input's own representation | **Accepted**. Add the Sterbenz remark (m5) |
| Segment term ±(r²/2)(\|Δ\| − sin\|Δ\|), sign of Δ | Mathematically correct. Check example: 4 + π/2 for b = +1 and 4 − π/2 for b = −1 confirmed in closed form and with shapely on flattened arcs. **But** \|Δ\| − sin\|Δ\| cancels catastrophically for small \|Δ\|, measured against 60-digit truth with c = 10: b = 10⁻⁵ → −1.0 × 10⁻⁷; b = 10⁻⁷ → −2.4 × 10⁻³; b ≤ 10⁻⁹ → −1 (returns 0). Near 2π the r computed from sin(\|Δ\|/2) loses accuracy as in §2.5 (−9 × 10⁻¹¹ at b = 10⁸). Computing r from b and \|Δ\| − sin\|Δ\| by series below 0.5 gives ≤ 3.3 × 10⁻¹⁶ everywhere | **Major M2** |
| "A positive bulge lies to the right of its chord; outward for a counter-clockwise ring" | Correct | **Accepted** |
| Reported area \|A\|; orientation from A against ±ε_A | Correct as logic | **Accepted**; the threshold value is under D1 and D8 |
| Centroid Cx = (1/6A) Σ (xᵢ + xᵢ₊₁)(xᵢyᵢ₊₁ − xᵢ₊₁yᵢ) | Correct with the **signed** A [R7]. CCW and CW rings both give (28.627…, 21.470…), the same as shapely. With \|A\| the CW ring gives (−28.6, 68.5): reflected through the first vertex. The text says "the reported area is \|A\|" just before this formula, which invites that misreading. At (5e5, 4e6) the translated centroid matches shapely to every printed digit | **Major M3** (wording) |

### 2.7 Phase 4 conventions (§11)

| Statement | Check | Verdict |
|---|---|---|
| Negative determinant reverses orientation and bulge sign | Correct: a reflection maps CCW to CW, and b = tan(Δ/4) is odd in Δ | **Accepted** |
| Non-uniform scale turns arcs into ellipses → "not supported" | Correct | **Accepted** |
| DXF ARC is counter-clockwise; Δ = (end − start) mod 2π | Matches the DXF reference (ARC angles run counter-clockwise, in degrees) | **Accepted** (the degree → radian conversion belongs to Phase 4) |

---

## 3. Numerical stability and tolerance analysis

### 3.1 Formula stability (measured against 60-digit arithmetic)

Chord c = 10. The table gives relative errors of the length s and the circular-segment area.

| b | s: spec formula | s: b-form | segment area: spec formula | segment area: b-form + series |
|---|---|---|---|---|
| 10⁻¹⁵ | 0 | 0 | **−1 (returns 0)** | 3.3 × 10⁻¹⁶ |
| 10⁻⁹ | ≤ 10⁻¹⁶ | ≤ 10⁻¹⁶ | **−1 (returns 0)** | 1.6 × 10⁻¹⁶ |
| 10⁻⁷ | ≤ 10⁻¹⁶ | ≤ 10⁻¹⁶ | **−2.4 × 10⁻³** | 5 × 10⁻¹⁷ |
| 10⁻⁵ | 5.5 × 10⁻¹⁸ | 5.5 × 10⁻¹⁸ | −1.0 × 10⁻⁷ | 1.6 × 10⁻¹⁶ |
| 10⁻³ | 2.2 × 10⁻¹⁷ | 2.2 × 10⁻¹⁷ | −3.3 × 10⁻¹¹ | 3.3 × 10⁻¹⁶ |
| 1 | 3.9 × 10⁻¹⁷ | 3.9 × 10⁻¹⁷ | 6 × 10⁻¹⁸ | 6 × 10⁻¹⁸ |
| 10⁶ | **2.1 × 10⁻¹¹** | 7 × 10⁻¹⁹ | — | — |
| 10⁸ | **−4.6 × 10⁻¹¹** | −1.1 × 10⁻¹⁶ | **−9.2 × 10⁻¹¹** | −1.9 × 10⁻¹⁶ |
| 10¹² | **7.2 × 10⁻⁵** | −8.6 × 10⁻¹⁷ | — | — |

**Practical size.**

- The absolute error of the spec's segment area stays tiny. Its worst case for c = 10 and b ≤ 2 × 10⁻¹³ is
  6.7 × 10⁻¹² (in the units squared), well below ε_A.
- Totals will rarely be visibly wrong. But the prescribed formulas would fail closed-form unit tests at the
  ENGINEERING_STANDARDS tolerance for exact identities (10⁻⁹ relative), as soon as they are tested at small or very
  large bulges.
- Tiny bulges such as 10⁻¹⁵ do appear in real CAD data as round-off.

**Stable forms** (all in terms of b; exact algebra from b = tan(Δ/4)):

| Quantity | Stable form | Notes |
|---|---|---|
| sin(Δ/2) | 2b/(1 + b²) | |
| r | c(1 + b²)/(4\|b\|) | |
| Sagitta | h = \|b\|c/2 | |
| Arc length | s = c(1 + b²)·atan\|b\|/\|b\| | b ≠ 0 |
| Segment area | sign(b)·(r²/2)·g(\|Δ\|) | g(x) = x − sin x |

How to compute g(x):

- For x < 0.5, use the series x³/6 − x⁵/120 + x⁷/5040 − …, summed until a term falls below 2⁻⁵³ of the sum.
  This takes at most about 9 terms.
- Otherwise use x − sin x directly.
- The 0.5 threshold was chosen numerically. Any value from about 0.1 to 1 works.

### 3.2 Tolerance at different magnitudes (F1: 60 m × 45 m, so D = 75 m)

| Placement | M | ulp(M) | ε | ε_A = ε·D | §8.9 "precision-limited"? |
|---|---|---|---|---|---|
| near origin (m) | 60 | 7.1 × 10⁻¹⁵ | 7.5 × 10⁻⁸ | 5.6 × 10⁻⁶ | no |
| state plane (m), (5e5, 4e6) | 4.0 × 10⁶ | 4.66 × 10⁻¹⁰ | 4.77 × 10⁻⁷ | 3.6 × 10⁻⁵ | no |
| same size in mm at (5e8, 4e9), i.e. a 75 mm object | 4.0 × 10⁹ | 4.77 × 10⁻⁷ | 4.9 × 10⁻⁴ | 3.7 × 10⁻² | **yes** |

Verified claims in GEOMETRY.md §5:

- ulp(4 × 10⁶) = 4.66 × 10⁻¹⁰ and ε ≈ 4.8 × 10⁻⁷: correct.
- "10⁻⁹ is about 4.5 × 10⁶ times machine precision": correct for machine epsilon 2⁻⁵² (4.50 × 10⁶). It is
  9.0 × 10⁶ times the unit roundoff 2⁻⁵³. Say which (m2).
- "a 60 m site plan … D ≈ 100 m": F1's D is 75 m, so ε = 7.5 × 10⁻⁸. Harmless, but use the fixture's real number (m2).

Properties of the formula:

- **Scale invariance.** When the size term dominates, ε scales with the object. F2 (F1 × 1000, in mm) makes exactly
  the same degeneracy decisions as F1. This is what the F2 "same results after conversion" test needs.
- **Shoelace error budget.** After translation each cross-product term has magnitude ≤ D², and the summed rounding
  error is at most about n·2⁻⁵³·D². ε_A = 10⁻⁹·D² is larger than that for any n below about 9 × 10⁶ vertices.
  So orientation is never decided by rounding noise.
- **Non-locality.** ε depends on the whole input. A 5 × 10⁻⁷ segment is zero-length inside a 1 km polyline
  (ε = 10⁻⁶), but not inside a 1 µm one. This is the intended consequence of a relative tolerance and should be
  stated (m3).
- **§8.9 noise.** At M = 4 × 10⁶ m, every object smaller than 0.48 m is flagged as precision-limited. That covers
  manhole symbols, text-sized blocks and similar. The results are still accurate to about 10⁻⁶ relative, far beyond
  printed precision. Expect many such flags in Phase 4 drawings in state-plane coordinates (m8).
- **Bounding box of bulged input** is undefined (M9). With vertices only, a 1 mm chord carrying a near-full 250 m
  circle gets D = 1 mm, and ε is 10⁹ times too small for the actual object.

### 3.3 Platform determinism

Rust documents that the precision of `sin`, `cos`, `atan2` and `hypot` is non-deterministic: it can vary by
platform, by Rust version, and even between calls in one run [R6]. Consequences:

- Expected values that pass through these functions must be compared with a tolerance, never bit for bit. CI runs on
  three operating systems (m4).
- Parity tests between interfaces of one build are unaffected (same binary).
- Exact equality is fine for pure +, −, ×, ÷ and `sqrt`, which IEEE 754 rounds correctly. One example is the
  translated shoelace on integer coordinates.

---

## 4. Edge-case review

Each case was run or reasoned through. "Defined?" means the specification gives one behaviour for the case.

| Case | GEOMETRY.md behaviour | Defined? | Finding |
|---|---|---|---|
| NaN / ±∞ anywhere | error | yes | Map to `INVALID_VALUE` (D3) |
| Two coincident points: direction | error | yes | Tolerance wording (m10) |
| Direction exactly along an axis from the negative side (−0) | not addressed | **no** | M1 |
| Tiny negative angle normalised | "within 10⁻¹⁵ below 2π → 0" | **wrong** | M1: `rem_euclid` returns exactly 2π |
| Negative distance for components | error | yes | — |
| Azimuth input outside [0, 2π) | "every finite α" | yes | Fine. sin and cos accept it |
| Zero-length straight segment | 0, reported | yes | — |
| Zero chord with non-zero bulge (c = 0) | 0, reported | yes | Correct: r = 0 |
| Chord ≤ ε with a huge bulge | 0, reported | **wrong** | M5: c = 10⁻⁹, b = 10¹² is an arc of r = 250 and length 1570.8 |
| \|b\| → ∞ (very large finite) | formulas as given | yes | M2: accuracy loss |
| b → 0 | series for length | yes | M2: area, not length, needs it |
| Collinear middle vertex between its neighbours | allowed | yes | — |
| Collinear middle vertex outside its neighbours (spike) | 8.3 "allowed" **and** 8.7 "overlap → error" | **conflict** | M7 |
| Neighbours of a vertex coincide (a→b→a) | 8.3: line through equal points undefined | **no** | M7 |
| Explicit closure; bulge on the duplicate vertex | duplicate dropped | partly | m6: say what happens to that bulge |
| Closed ring, 2 vertices, bulged edges (DONUT, polyline circle) | error (< 3 vertices) | **wrong** | M4: area π for b = (1, 1) and π/2 for (1, 0), both valid |
| Zero-area ring | area 0, orientation undefined, centroid error | yes | D8: "0" is engine-side rounding |
| Figure-8 | error | yes | Net shoelace 0 against a true 2 (D4) |
| Pinch (two lobes share a vertex) | error ("touch") | yes | Net 2.0 is correct here; OGC calls it invalid (D4) |
| Keyhole (hole joined by a zero-width bridge) | error (overlap) | yes | Net 96 is correct here (D4) |
| Bulged edge crossing another edge | **undefined** | **no** | M6: rectangle 10 × 2 with top bulge −0.6 gives signed A = −1.37 (orientation flips) on an invalid shape |
| "Touch" within rounding | **undefined** | **no** | M8 |
| Arc with \|Δ\| ≥ 2π, r < 0, circle r ≤ ε | not stated | **no** | m7 |
| Centroid, bulged polygon | "not supported in 0.1" error | yes | D5 |
| Precision-limited input | reported | yes | m8 (noise) |
| Bounding box with arcs | not stated | **no** | M9 |

---

## 5. Findings

Classes:

- **Critical:** wrong engineering results for realistic input, or a conflict with an accepted principle. Must fix.
- **Major:** a wrong formula or rule, a contradiction, or an ambiguity whose obvious reading is wrong. Must fix
  before acceptance.
- **Minor:** clarity, completeness or test guidance. Should fix; low impact.
- **Accepted:** verified correct as written.

### Critical

None.

### Major

| ID | GEOMETRY.md | Finding | Evidence | Correction |
|---|---|---|---|---|
| M1 | §4 | Normalisation can return exactly 2π and can return −0. The 10⁻¹⁵ snap misses the real failure case and adds a hidden angular tolerance, which contradicts §5 | `rem_euclid(−1e-16, TAU) == TAU`; `atan2(−0.0, 1)` → −0.0 kept; Rust documentation [R6] | C-5 |
| M2 | §9, §10 | Bulge numerics. The series sits on the length formula, which is stable at small b and has a wrong rationale. The area term loses all digits at small b. Both lose accuracy as \|Δ\| → 2π | Table §3.1 | C-6, C-17, C-18 |
| M3 | §10 | The centroid must use the signed A. The nearby "\|A\|" invites a reflected centroid for clockwise rings | (28.6, 21.5) against (−28.6, 68.5) | C-19 |
| M4 | §8.5 | Rejects valid closed rings of two vertices with bulged edges. AutoCAD's DONUT makes "two arc polylines … joined end-to-end" [R4] | Area π for (1, 1), as expected | C-13 |
| M5 | §8.2 | A sub-tolerance chord with a large bulge is not zero length | c = 10⁻⁹, b = 10¹² gives r = 250, s = 1570.8 | C-10 |
| M6 | §8.7 | Self-intersection undefined for bulged edges. A chord-only check misses arc crossings | Signed A = −1.37 for an invalid ring | C-15 + D7 |
| M7 | §8.3/§8.7 | Spikes are "collinear, allowed" and also "overlap, error". 8.3 is undefined when a vertex's neighbours coincide | Spike ring: shoelace 12.5 | C-11 |
| M8 | §8.7 | "Intersect or touch" has no numerical definition. Exact floating-point orientation tests are not robust near degeneracy [R12] | Reasoning + [R12] | C-15 |
| M9 | §5 | D (bounding box) is undefined for bulged input and unclear per call | §3.2 example | C-7 |

### Minor

| ID | GEOMETRY.md | Finding | Correction |
|---|---|---|---|
| m1 | §1 | "§8.4" should be §8.7 | C-1 |
| m2 | §5 | "machine precision": say machine epsilon 2⁻⁵². Use F1's real D = 75 m | C-7 |
| m3 | §5 | Define ulp(x) (including x = 0 and powers of two). State that ε is per call and therefore non-local | C-7 |
| m4 | §12 | Trigonometric functions are not deterministic across platforms. Cardinal components are not exactly zero. Tests need tolerances | C-22, §9 |
| m5 | §9, §10 | Say what compensated summation does and does not fix. Add the Sterbenz exactness remark | C-18 |
| m6 | §8.4 | Fate of the duplicate vertex's bulge | C-12 |
| m7 | §4, §8.8 | Arc and circle input domain (\|Δ\| ≥ 2π, r < 0, circle r ≤ ε) | C-16 |
| m8 | §8.9 | Precision-limited flag will be frequent for small objects in state-plane drawings | Noted under D1 and D3; no text change required |
| m9 | §3 | Fix the viewing side for "counter-clockwise" | C-3 |
| m10 | §7 | Two-point tolerance reduces to the precision floor; the component-only case is undefined | C-9 |
| m11 | §8 | No mapping from geometry errors to contract error codes | D3, C-24 |
| m12 | §3 | "(Phase 4, warnings)" prejudges OCS handling; extrusion (0, 0, −1) is exact | C-4, C-21 |
| m13 | §6, §13 | §6 "used for three things only" omits collinearity, self-intersection and zero area. Typo "an "not supported"" in §13 Q5 | C-8, C-23 |

m14 and m15 are recorded as Accepted with a note (A6, A2).

### Accepted (verified)

| ID | Item | How verified |
|---|---|---|
| A1 | Right-handed frame, (E, N), latitude = ΔN, departure = ΔE | Reasoning; [R13] |
| A2 | α = atan2(Δx, Δy); components d·sin α, d·cos α. Note m15: compute α directly, not via θ | 20 000-angle round trip, ≤ 1 ulp(2π) |
| A3 | α ↔ θ conversion formula | Cardinal checks, ≤ 1.8 × 10⁻¹⁶ |
| A4 | Bulge definition, sign and side; \|b\| = 1 semicircle | DXF reference [R1, R2]; numerical side test |
| A5 | Arc length formula (mathematics) and series coefficients | Symbolic series |
| A6 | Segment-area formula (mathematics) and sign rule; 4 ± π/2. Note m14: wording with \|Δ\| is correct | Closed form and shapely |
| A7 | Translate-then-shoelace | Exact 2550 up to 4 × 10¹² offsets |
| A8 | Centroid formula with signed A | shapely, both orientations, at state plane |
| A9 | Tolerance example values (ulp(4e6), ε) | Recomputed |
| A10 | `hypot` for distance | Reasoning |
| A11 | §11 mirror, non-uniform scale and DXF ARC rules | Reasoning; DXF reference |
| A12 | Neumaier summation | Correct, harmless |
| A13 | Scope inside MVP_SCOPE and D-20; no new dependencies; offline | Document comparison |
| A14 | Point equality is non-transitive and used only between neighbours | Reasoning |

---

## 6. Decisions

None of these is decided by this review. For each: recommendation, justification, advantages, disadvantages,
consequences for 0.1, and whether a decision record is needed.

**One record for the lot (recommended).** Most of these are conventions that change results. They belong in
**one new decision record** accepting the revised GEOMETRY.md, with D1, D2, D4, D5, D7 and D8 as its sections.
It would take the next free IDs, D-22 / ADR 0010. D3 gets its own record only if it changes the contract. D6 needs
none.

### D1 — Default tolerance ε = max(10⁻⁹·D, 1024·ulp(M))

**Recommendation:** accept the formula, with the clarifications in C-7: D from the curve's bounding box including
arc extremes, computed per call; ulp defined; the two-point case explained. Do **not** add a default absolute
tolerance in 2B.

**Justification**

- The crate is unit-agnostic. An absolute default such as "1 µm" has no meaning without units: it differs by a
  factor of 1000 between a metre drawing and a millimetre drawing. The relative term keeps decisions identical
  under scaling, which is what the F1/F2 test requires.
- The ulp term is a representation floor, not an engineering tolerance. It prevents a relative tolerance from
  dropping below what the coordinates can express at large offsets.
- An engineering tolerance that knows its units (for example "gaps under 1 mm in a drawing in metres") is a
  different thing. It belongs to the caller (a calculation, or Phase 4 drawing analysis). The caller can pass it
  explicitly, because §5 already lets every operation take an explicit tolerance.
- So yes, separate absolute and relative tolerances are needed. Both already exist: the representation floor
  (absolute, unit-free) and the size term (relative) are inside the formula. The engineering absolute tolerance
  is explicit and belongs to the caller.

**Behaviour across magnitudes:** see §3.2. It is scale-invariant until the floor takes over. The floor takes over
only when the object is smaller than about 10⁶ × ulp(M), for example 0.48 m at state-plane magnitudes in metres.

**Could it reject valid geometry?** Only features smaller than 10⁻⁹ of the object's size are treated as degenerate.
That is far below survey or drafting precision (1 µm on a 1 km object).
Rejections happen only where the specification says error:

- too few vertices;
- a zero-area centroid;
- self-intersection within ε;
- an ill-conditioned arc.

Each needs true or near-exact degeneracy.

**Could it accept invalid geometry?** Yes, by design: a 1 µm sliver on a 100 m plan is treated as real. Geometry
should not guess drafting intent. Drawing-quality checks with real units belong to Phase 4.

| Advantages | Disadvantages |
|---|---|
| Unit-free; scale-invariant; one formula; explainable in a trace | Non-local: the same segment can be judged differently in different calls. The constant 10⁻⁹ is a judgement, not derived. §8.9 can be noisy |

**Consequences for 0.1:**

- Traverse (2C) uses only distance, azimuth and components. ε affects it only through the coincident-point rule.
- Most effects land in Phase 4 drawing analysis.

**Decision record:** yes (a section of the proposed geometry record).

### D2 — Tolerance internal, or a contract input?

**Recommendation:** keep it **internal** in 0.1 and record it. Expose no new inputs.

- In the Rust API, the value used is returned with each result, as §5 already says.
- Through the contract, a calculation whose result depended on a tolerance decision records ε (and ε_A when it was
  used) as one trace step. The step is in internal units (m, m²). It is finite, so it passes the registry's
  trace-unit check.

**Justification:**

- API_DESIGN §4 states that adding an optional input whose default preserves old results does not bump the
  version. Adding a `tolerance` input later is therefore fully compatible, so deferring costs nothing.
- Exposing it now adds an input that no 0.1 reference case exercises.

| Advantages | Disadvantages |
|---|---|
| No contract growth; reversible; results stay reproducible because the trace shows ε | Users cannot override ε through Python or MCP in 0.1. One more trace step |

**Consequences for 0.1:** none for the five calculations, except that traverse would record ε only if it uses the
coincident-point rule. Phase 4 results carry ε in their result structures.

**Record in steps?** Yes, only when a tolerance decision affected the result. Otherwise the trace becomes noise.

**Decision record:** a section of the geometry record; no API_DESIGN change.

### D3 — A `DEGENERATE_GEOMETRY` warning code

**Recommendation:** **do not add it in 2B.** Instead:

1. `civkit-geometry` returns a typed Rust report: an enum of degeneracy notes plus a Rust error enum. This has no
   contract impact.
2. When a calculation surfaces geometry problems, it uses **existing** codes:
   - non-finite input → `INVALID_VALUE`;
   - coincident-point direction, too few vertices, zero-area centroid, self-intersection, invalid arc →
     `OUT_OF_DOMAIN` ("mathematically invalid");
   - §8.9 precision-limited → the existing warning `ROUNDING_SENSITIVE` (level `caution`), which describes
     exactly that.
3. Decide how the remaining informational notes reach users (zero-length segments, zero-sweep arcs, zero area)
   when Phase 4 first exposes drawing results through the contract. Record this as a Phase 4 task in
   IMPLEMENTATION_PLAN.

**Warning or error?**

- **Warning (`info`)** where the result is still well defined: a zero-length segment contributes exactly 0, and
  so does a zero-sweep arc.
- **Error** where no meaningful result exists: undefined direction, centroid of zero area, self-intersection.
- **`caution`** for precision-limited input.

**Conflicts:**

- None with `OUT_OF_DOMAIN` or `ROUNDING_SENSITIVE`.
- `OUTSIDE_VALIDITY_RANGE` must not be reused: it means a method's stated range, not degenerate data.
- A real gap: "**not supported in 0.1**" (a bulged-polygon centroid under D5, arcs under non-uniform scale in
  Phase 4) fits no existing error code. It is not mathematically invalid. This gap only matters once such a case
  is reachable through the contract, which is not before Phase 4.

**Compatibility:**

- API_DESIGN §5 lists warnings as "typical", but it does not say clients must tolerate unknown warning codes.
- Generated Python enums or MCP schemas that enumerate codes could break when a code is added.
- Whenever the first new code is added, add the rule "clients must accept unknown warning codes" in the same
  change.

| Option | Advantages | Disadvantages |
|---|---|---|
| Defer (recommended) | No contract change without a consumer; 2C unaffected; uses accepted codes | The decision returns in Phase 4; until then, info notes are Rust-only |
| Add `DEGENERATE_GEOMETRY` now | One code fixed early | Contract change with no 0.1 consumer in 2C; untested surface; still leaves the "not supported" gap |

**Decision record:** none if deferred (note it in IMPLEMENTATION_PLAN). If a code is added, it needs a record,
because it amends the D-19 contract resolutions.

### D4 — Self-intersecting polygons

**Recommendation:** **reject** (error) for area and centroid, as proposed. Length is unaffected. Define "touch" with
ε (C-15). Decide bulged edges under D7.

**Are there safe cases?** Yes. The net shoelace value is the correct area for:

- a **pinch**: lobes of the same orientation sharing one vertex (net 2.0 = true 2.0);
- a **spike** (zero-width fold-back): it contributes 0;
- a **keyhole**: a hole joined by a zero-width bridge (net 96 = 10 × 10 − 2 × 2). Older CAD hatch boundaries use it.

Unsafe cases:

- a **figure-8**: net 0, true 2;
- **crossing arcs**: −1.37 against an undefined true area.

Telling the safe cases from the unsafe ones needs a full topological analysis (winding numbers over the planar
arrangement). That is beyond a 2B "check" and close to a CAD engine.

The standards line up with rejection:

- OGC Simple Features requires rings to be simple. GEOS reports a pinch as "Ring Self-intersection".
- GDAL notes that the Esri model accepts self-touching rings [R5].
- So refusing is the conservative, widely shared reading.

**Consequences of "warn and continue":**

- A figure-8 lot would enter a quantity schedule with area 0, or any other net value.
- The warning might not survive export: the schedule format is not yet defined.
- Orientation would also be wrong (it flips in the M6 example).

**Consequences of rejecting:**

- Real drawings with keyholes, pinches or spikes get an error.
- Phase 4 must report them as findings, and users may need to clean them up.

Rejecting is acceptable for read-only analysis that "reports, never guesses". It also matches D-20: only what can be
verified.

| Advantages | Disadvantages |
|---|---|
| Never a silently wrong area; matches OGC/GEOS; simple to explain | Rejects some shapes whose net area is right (pinch, keyhole, spike); O(n²) check (fine at 0.1 sizes) |

**Decision record:** yes (a section of the geometry record).

### D5 — Centroid for straight-edged polygons only

**Recommendation:** accept the deferral.

**What curved boundaries would need.** Each bulged edge adds the first moment of its circular segment: the segment's
signed area times its centroid. The centroid lies on the arc's mid-radius, at distance
4r·sin³(|Δ|/2)/(3(|Δ| − sin|Δ|)) from the centre. This was checked here to 9 digits against flattened shapely
centroids for sweeps of 0.6, π and 5.0 rad.

That distance has the same small-|Δ| cancellation as the area (M2). A stable implementation would instead integrate
the moments along the arc (Green's theorem, closed form in r, θ₀ and Δ). Either way it needs its own
independently verified case. There is none: no fixture or reference case has a bulged centroid.

**Why deferral is safer:**

- D-20's principle is "only what a worked example verifies".
- No 0.1 consumer uses a centroid. Drawing analysis lists "lengths and areas"; no 2C calculation needs one.
  MVP_SCOPE names "polygon area and centroid" in the engine, so straight-edged centroids still satisfy it.

| Advantages | Disadvantages |
|---|---|
| Nothing unverified ships; small code | A curved lot's centroid is an error in 0.1; needs the "not supported" code gap (D3) closed before any contract exposure |

**Consequences for 0.1:** none for 2C. Phase 4 can still report areas of curved boundaries.

**Decision record:** a section of the geometry record.

### D6 — ARCHITECTURE.md §3.3 wording

**Recommendation:** amend §3.3 (exact text in C-25). Two corrections to the wording proposed in GEOMETRY.md §13 Q6:

- **Transformations are in 0.1.** Phase 4 (DXF drawing analysis, fixture F5) is part of release 0.1, as
  IMPLEMENTATION_PLAN says: "Phase 4 — Drawing model and DXF → 0.1". The wording must say "not in 2B", not
  "not in 0.1".
- **Crate placement is ambiguous.** "Implemented with civkit-drawing" does not say which crate holds the code.

**Sub-decision D6b (placement):** recommend `civkit-geometry`, added in Phase 4 when `civkit-drawing` first needs
it, because:

- it keeps all geometric mathematics in one crate;
- §3.3 already lists transformations there;
- the dependency direction stays drawing → geometry.

The alternative is placing transforms in `civkit-drawing`. That keeps geometry smaller but splits conventions across
crates.

| Advantages | Disadvantages |
|---|---|
| Architecture matches the 0.1 plan; no hidden scope | None material |

**Decision record:** not needed for the wording, which narrows within D-20 and changes no behaviour. D6b can be one
line in the geometry record.

### D7 (new) — Self-intersection check for bulged edges

The options:

| Option | Description | Advantages | Disadvantages |
|---|---|---|---|
| **a (recommended)** | Exact distance tests between segment and arc, and between arc and arc (closed-form line–circle and circle–circle relations, restricted to the arcs' sweeps), with the same ε rule as straight edges. Internal only, no public intersection API | Exact; consistent with straight edges; no new dependencies | More code and tests in 2B (still 2D and still a "check") |
| b | Flatten arcs to chords within a stated chord tolerance, for the check only | Simple | Adds a second tolerance; can miss near-tangent contact or report false contact |
| c | Chord-only check, and refuse area for any bulged ring | Smallest | Rejects F1-type boundaries with curved edges; likely too strict for drawing analysis |
| d | Chord-only check (implicit today) | Simplest | **Wrong areas for invalid input** (M6). Not acceptable |

**Consequences for 0.1:** option a adds a moderate amount of code and tests to 2B and closes M6. Option c would make curved site
boundaries unusable in Phase 4.

**Decision record:** a section of the geometry record.

### D8 (new) — Value reported for a zero-area polygon

GEOMETRY.md §8.6 returns 0 when |A| ≤ ε_A. That is rounding inside the engine. ENGINEERING_STANDARDS §3 says
"No rounding inside the engine. Rounding happens only when displaying."

| Option | Advantages | Disadvantages |
|---|---|---|
| **Return the computed \|A\|** (≤ ε_A) with orientation "undefined" and the degeneracy note (recommended) | Consistent with ENGINEERING_STANDARDS; no information lost | Displays a tiny non-zero area such as 3 × 10⁻⁷ m² for a "degenerate" ring |
| Return exactly 0 (as proposed) | Clean output | Contradicts ENGINEERING_STANDARDS §3 unless explicitly excepted |

**Decision record:** a section of the geometry record. If 0 is kept, ENGINEERING_STANDARDS needs a stated exception.

### D9 (new) — Approval of the corrections

The corrections C-1 … C-25 in §7 are fixes, not new conventions. They still change an accepted-to-be document, so
they need approval as a set, or item by item.

---

## 7. Exact proposed document corrections

All in **docs/GEOMETRY.md** unless stated. Wording is proposed; nothing has been applied.

### §1 Scope

**C-1.** In "Not in 2B", bullet 2, replace

> (except the self-intersection *check* in §8.4)

with

> (except the self-intersection *check* in §8.7)

**C-2.** In "Not in 2B", bullet 4, replace

> §11 fixes their conventions now so they match, but they are implemented with `civkit-drawing` in Phase 4.

with

> §11 fixes their conventions now. They are implemented in Phase 4, which is still part of release 0.1, in
> `civkit-geometry` (D6b), when `civkit-drawing` first needs them.

In the paragraph after the list, replace "This specification narrows 0.1 to the list above, in line with D-20." with

> This specification limits Phase 2B to the list above, in line with D-20; 3D remains out of 0.1.

### §3 Coordinate systems

**C-3.** Replace

> The +y axis is +x rotated 90° counter-clockwise.

with

> The +y axis is +x rotated +90° about +z, where +z points towards the viewer looking down on the plan.
> "Counter-clockwise" always means as seen from +z.

**C-4.** In the rule "z is ignored", replace "(Phase 4, warnings)" with "(Phase 4; see §11)".

### §4 Angle conventions

**C-5.** Replace the sentence

> The result is normalised into [0, 2π); values within 1e-15 rad below 2π become 0 so that "north" is never
> reported as 2π.

with

> **Normalisation** of any angle a: r = a.rem_euclid(TAU), with TAU the `f64` constant for 2π.
> - If r ≥ TAU, r becomes 0: `rem_euclid` can return exactly TAU when a is a tiny negative number.
> - Negative zero becomes +0: `atan2` returns −0 for directions along the zero axis approached from the negative side.
>
> No other value is changed: angles are never compared with a tolerance (§5).

In "Direction from components", append:

> α is computed directly as atan2(Δx, Δy), never by converting θ.

**C-6.** In "Bulge", append a sub-bullet:

> - Quantities derived from a bulge are computed from b, never from sin(Δ/2), which loses accuracy as |Δ| → 2π:
>   - radius r = c(1 + b²)/(4|b|)
>   - sagitta h = |b|·c/2
>   - sin(Δ/2) = 2b/(1 + b²)
>
>   Here c is the chord length.

### §5 Tolerances

**C-7.** Replace the definitions paragraph ("where D is the diagonal … the input itself).") with

> where:
> - **D** is the diagonal of the bounding box of the geometry passed in one call. The box includes each arc's
>   extreme points, not only the vertices.
> - **M** is the largest absolute vertex coordinate.
> - **ulp(x)** is the distance from |x| to the next larger `f64`; ulp(0) is the smallest positive subnormal.
>
> ε is computed per call. It depends on the extent of the whole input, so the same short segment can be zero-length
> in a large polyline and not in a small one. This is intended.

In "Size term", replace "about 4.5 × 10⁶ times machine precision" with "about 4.5 × 10⁶ times machine epsilon (2⁻⁵²)".

In "Examples", replace the first example with

> fixture F1 (60 m × 45 m) near the origin: D = 75 m → ε = 7.5 × 10⁻⁸ m

### §6 Point equality

**C-8.** Replace "Equality is used for three things only:" and its list with

> ε and ε_A are used only for:
> - zero-length segments (§8.2)
> - explicit closure (§8.4)
> - collinearity (§8.3)
> - coincident points for direction (§7)
> - self-intersection (§8.7)
> - zero area (§8.6)

### §7 Distance and direction

**C-9.** Replace "If d ≤ ε the direction is undefined (ε from §5, computed over the two points)." with

> If d ≤ ε the direction is undefined.
> - For two points, D = d, so only the precision floor can decide: the test is d ≤ 1024·ulp(M), with M the larger
>   absolute coordinate of the two points.
> - For a direction from components (Δx, Δy) alone, it is undefined only when both are exactly 0.
> - Whether a computed difference is zero in engineering terms (for example a traverse misclosure) is decided by the
>   calling calculation, not by this crate.

### §8 Degeneracy table

**C-10.** Row 8.2. Definition becomes:

> straight segment with chord c ≤ ε; bulged segment with arc length s ≤ ε (§9)

Behaviour: unchanged. Add a new row 8.2a:

> **Ill-conditioned bulge** | c ≤ ε < s (the radius would be set by a chord below tolerance) | **error**

Add the note: "c = 0 exactly with b ≠ 0 has r = 0 and falls under 8.2."

**C-11.** Row 8.3. Definition becomes:

> perpendicular distance of the middle vertex from the line through its neighbours ≤ ε, **and** the middle vertex
> lies between them (projection parameter in [0, 1])

Add:

> A collinear vertex outside its neighbours folds the ring back (a spike) and is handled by 8.7. If the neighbours
> are equal within ε, 8.3 does not apply; the round trip is handled by 8.7.

**C-12.** Row 8.4. Append:

> The dropped vertex's bulge describes the zero-length closing segment and is checked as in 8.2/8.2a.

**C-13.** Row 8.5. Replace "polygon: < 3 vertices remaining after 8.2 and 8.4" with

> polygon: after 8.2 and 8.4, fewer than 3 vertices if every edge is straight, or fewer than 2 if any edge is bulged
> (a closed two-vertex polyline with bulged edges is valid; AutoCAD's DONUT draws one)

**C-14.** Row 8.6. Behaviour per D8. If the recommended option is chosen, it becomes:

> area returned as computed (≤ ε_A), orientation **undefined**, centroid **error**; reported

**C-15.** Row 8.7. Definition becomes:

> On the ring after 8.2 and 8.4:
> - two non-adjacent edges are at a distance ≤ ε anywhere; or
> - two adjacent edges overlap along a length > ε (fold-back).
>
> Bulged edges are included; their distances are computed as decided in D7.

Add to the behaviour:

> This refuses some rings whose net area would be right (a pinch, a keyhole); see D4.

**C-16.** Row 8.8. Replace with:

> **Invalid or degenerate arc or circle** | r ≤ ε or r < 0; sweep 0; |Δ| ≥ 2π | zero sweep: length 0, reported;
> |Δ| ≥ 2π: **error** (use a circle); negative radius or r ≤ ε with non-zero sweep: **error**; circle with r ≤ ε: **error**

### §9 Length

**C-17.** Replace the "Bulged segment" bullet and its two sub-bullets with

> **Bulged segment:** s = c·(1 + b²)·atan|b| / |b| (b ≠ 0). This equals r·|Δ|, has no cancellation for any finite b,
> and needs no series.
>
> The form c·(Δ/2)/sin(Δ/2) is avoided: its relative error grows to 2 × 10⁻¹¹ at b = 10⁶ and 7 × 10⁻⁵ at b = 10¹².

In the summation bullet, append:

> It removes accumulation across many segments. It does not remove cancellation inside a term; §10's translation
> does that.

### §10 Area, orientation and centroid

**C-18.** Replace the "Bulged edges" bullet's formula sentence with

> each adds the signed area of its circular segment, sign(b)·(r²/2)·g(|Δ|), with:
> - r computed from b (§4);
> - g(x) = x − sin x, computed directly for x ≥ 0.5 and by its series x³/6 − x⁵/120 + x⁷/5040 − … for x < 0.5.
>   Direct subtraction loses all significant digits for x ≲ 10⁻⁵.

In "Precision step", append:

> The translated coordinates are exact whenever a vertex and the first vertex are within a factor of 2 of each other
> (Sterbenz), as at state-plane magnitudes. The only remaining error is the input's own representation.

**C-19.** In "Centroid", after the formula, insert:

> Here A is the **signed** area from the same translated sum, not the reported |A|. Using |A| would reflect the
> centroid of a clockwise ring through the first vertex.

**C-20.** In "Orientation", no change unless D8 changes the 8.6 wording. Then replace "otherwise undefined (8.6)"
with "otherwise undefined (8.6); the area is still reported as computed".

### §11 Phase 4 conventions

**C-21.** Add a bullet:

> **Extrusion (OCS):** a planar entity with extrusion (0, 0, −1), common after mirroring, maps to world coordinates
> by negating x (Autodesk arbitrary axis algorithm). Like a mirror, it reverses orientation and bulge signs. Other
> extrusions are not plan entities and are reported.

### §12 Verification

**C-22.** Replace items 1–5 with the plan in §9 of this review, or append its new items:
- the 60-digit oracle values;
- the angle-normalisation cases;
- the two-vertex rings;
- the spike, pinch and keyhole cases and the crossing arc;
- the no-bit-exact rule for trigonometric results.

### §13

**C-23.** Q5: "an "not supported in 0.1" error" → "a "not supported in 0.1" error". After the decisions, replace §13
with a pointer to the decision record.

### §2 Numbers and units (only after D3)

**C-24.** Append to the first bullet:

> Through the contract: non-finite input → `INVALID_VALUE`; undefined results (coincident-point direction, too few
> vertices, zero-area centroid, self-intersection, invalid arcs) → `OUT_OF_DOMAIN`; precision-limited input → warning
> `ROUNDING_SENSITIVE` (`caution`).

### docs/ARCHITECTURE.md §3.3 (D6)

**C-25.** Replace

> 2D/3D primitives and algorithms used by calculations and drawing analysis: polyline length, polygon area/centroid,
> arcs, bearings and distances, transformations.

with

> 2D primitives and algorithms used by calculations and drawing analysis: distance and azimuth, polyline length
> (straight and bulged segments), arc length, polygon area and orientation, and centroid of straight-edged polygons
> ([GEOMETRY.md](GEOMETRY.md)). The 2D transformations needed for block inserts are specified in GEOMETRY.md §11 and
> added here in Phase 4 (release 0.1). 3D is not in 0.1.

### docs/API_DESIGN.md §5 (only if D3 adds a code)

Add after the warnings paragraph:

> Clients must accept warning codes they do not know. New warning codes may be added in a minor release.

---

## 8. Recommended final 2B implementation boundary

**In 2B (`civkit-geometry`, std only, no new dependencies; `proptest` stays a dev-dependency):**

- 2D point and vector values (f64, finite-checked).
- Angles:
  - normalisation (C-5);
  - θ ↔ α conversion;
  - α from components;
  - components from α and d.
- Distance (`hypot`); direction with the coincident-point rule (§7).
- Tolerance:
  - ε, ε_A, D (with arc extremes) and M;
  - an explicit override argument in the Rust API;
  - the value used returned with each result.
- Segments:
  - straight;
  - bulged, using the stable b-forms;
  - arc (centre, r, θ₀, Δ);
  - circle (circumference, area).
- Polyline length (open and closed) with compensated summation.
- Ring preparation (8.2, 8.2a, 8.4), validity (8.5, 8.7 incl. D7), signed area, orientation, reported area.
- Centroid for straight-edged simple rings.
- A typed degeneracy report and a typed error enum (Rust only).

**Not in 2B:**

- any registry calculation (`geometry.*` IDs) or other contract change;
- a public intersection, offset or clipping API;
- transformations, OCS and DXF (Phase 4);
- 3D, splines, ellipses, geodesy, projections;
- the bulged-polygon centroid;
- a units dependency;
- robust-predicate or geometry crates;
- network, AI or paid anything.

**Done when:** every item in §9 passes on all three CI platforms, the dependency rule and `cargo-deny` pass, and
GEOMETRY.md matches the code (every function cites its section).

---

## 9. Verification and testing strategy

The principle: **expected values never come from the code under test.** Each one is a closed form, a 60-digit value
computed outside the repository, or a result from another program (ezdxf/shapely, as the fixtures already did). Each
records where it came from.

1. **Closed forms** (tolerance 10⁻¹² relative unless exact):
   - rectangles and right triangles (exact on integer coordinates);
   - F1 boundary: area 2550, perimeter 200 (exact);
   - road: 100 + 10π; drain: 5π + 25; circle R = 3: area 9π, circumference 6π;
   - square with a semicircular edge: 4 ± π/2;
   - two-vertex rings: b = (1, 1) gives π; b = (1, 0) gives π/2;
   - quarter arc R = 20: 10π.
2. **High-precision oracle for bulges:** b ∈ {±10⁻¹⁵, 10⁻⁹, 10⁻⁷, 10⁻⁵, 10⁻³, 0.1, tan(π/8), 1, 10, 10³, 10⁶, 10⁸, 10¹²},
   c = 10, for both length and segment area. Expected values come from 60-digit arithmetic and are recorded with
   their provenance. Tolerance 10⁻¹³ relative. These tests fail on the formulas M2 replaces.
3. **Angles:**
   - normalisation of −10⁻¹⁷, −4 × 10⁻¹⁶, −0.0 (check the sign bit), TAU, 7·TAU + 1 and −π;
   - all four `atan2` sign combinations, including ±0;
   - cardinal azimuths: components within 10⁻¹³·d of exact;
   - α ↔ θ ↔ components round trip within 4 ulp(2π).
4. **Large magnitudes:**
   - F1 translated by (5 × 10⁵, 4 × 10⁶) m and F2 by (5 × 10⁸, 4 × 10⁹) mm: area exactly 2550 (× 10⁶ for F2);
   - a regression guard at offset 4 × 10¹² that fails without translation (the naive result is 0).
5. **Every degeneracy row**, including the new ones:
   - tiny chord with a large bulge (8.2a);
   - two-vertex rings;
   - spike, pinch, keyhole and figure-8;
   - the crossing-arc ring of M6 (rectangle 10 × 2, top bulge −0.6, which must be an error);
   - arc domain errors.
6. **Property tests** (proptest), each with an explicit tolerance:
   - integer or dyadic translations leave area and length **exactly** unchanged;
   - rotations within 10⁻¹² relative;
   - scaling by k: length × k, area × k²;
   - reversal negates the signed area;
   - splitting a straight segment preserves both;
   - degeneracy decisions identical under scaling by powers of 2.
7. **Platform rule:** no bit-exact expectations for anything through `sin`, `cos`, `atan2` or `hypot` [R6]. Exact
   equality is used only for +, −, ×, ÷ and `sqrt` results.
8. **Mutation check** (as in 2A). Plant each of these errors and confirm a test fails:
   - flip the bulge sign;
   - use |A| in the centroid;
   - skip the translation;
   - use the spec's old g(x);
   - remove the TAU wrap.
9. **Reproducing this review.** The check scripts used here (Python, `mpmath`, `shapely`) are outside the repository.
   They can be added under `tools/` if the maintainer wants them, but not as part of this review-only task.

---

## 10. Sources

- **[R1]** Autodesk, *DXF Reference*, LWPOLYLINE/VERTEX group code 42 (bulge): the tangent of one quarter of the
  included angle, negative when the arc runs clockwise; 0 is straight and 1 a semicircle. Quoted in ezdxf's
  documentation, which links the Autodesk page, and in AfraLISP's reproduction of AutoCAD help.
- **[R2]** ezdxf documentation, *LWPolyline — Bulge value*: positive bulge counter-clockwise, to the right of the
  line (sources.debian.org, ezdxf 1.4.1).
- **[R3]** Autodesk, *About Arbitrary Axis Algorithm (DXF)* and *Object Coordinate Systems (OCS) in DXF*,
  help.autodesk.com (AutoCAD DXF reference).
- **[R4]** Autodesk, *DONUT (Command)*, AutoCAD LT for Mac help, help.autodesk.com: a donut is two arc polylines
  joined end to end.
- **[R5]** GDAL, *Geometry Validity* (gdal.org): validity per the OGC Simple Features model; self-touching rings are
  invalid in GEOS but valid in the Esri model. OGC 06-103r4, *Simple Feature Access — Part 1*.
- **[R6]** Rust standard library, `f64::rem_euclid` (can return the divisor for small negative inputs) and
  `f64::sin`, `cos`, `atan2`, `hypot` ("precision … is non-deterministic"), doc.rust-lang.org.
- **[R7]** P. Bourke, *Calculating the area and centroid of a polygon* (1988), paulbourke.net/geometry/polygonmesh.
  Restated by R. Nürnberg (Imperial College, 2013) with the 1/(6A) centroid using the signed area.
- **[R8]** IEEE 754-2019, *Standard for Floating-Point Arithmetic* (correct rounding of +, −, ×, ÷, sqrt).
- **[R9]** A. Neumaier, "Rundungsfehleranalyse einiger Verfahren zur Summation endlicher Summen", *ZAMM* 54 (1974)
  39–51.
- **[R10]** N. J. Higham, *Accuracy and Stability of Numerical Algorithms*, 2nd ed., SIAM, 2002, ch. 4 (summation).
- **[R11]** D. Goldberg, "What every computer scientist should know about floating-point arithmetic", *ACM Computing
  Surveys* 23(1), 1991 (exact subtraction, Sterbenz lemma).
- **[R12]** J. R. Shewchuk, "Adaptive Precision Floating-Point Arithmetic and Fast Robust Geometric Predicates",
  *Discrete & Computational Geometry* 18 (1997) 305–363.
- **[R13]** US Navy, *Engineering Aid* (NAVEDTRA 14070), ch. 7: latitudes and departures. This is planning/REFERENCE_SOURCES.md S12.

Publications R8–R12 are cited for well-known results. They were not re-read for this review.
