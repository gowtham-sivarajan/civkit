# ADR 0010 — Geometry conventions for 0.1

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-25 18:37 IST · last edited 2026-11-27 18:50 IST by Claude Sonnet 5.5 (`claude-sonnet-5-5`)

- **Status:** Accepted (maintainer, 2026-11-25 18:10 IST: "everything is accepted"). The items under
  "Settled during implementation" were added afterwards and **confirmed** by the maintainer on 2026-11-25
  (18:58 IST), item 6 with an added rule.
- **Date:** 2026-11-25
- **Decision ID:** D-22
- **Relation to earlier decisions:**
  - Implements the geometry part of MVP_SCOPE within D-20 ([ADR 0009](0009-implementation-scope-0-1-calculations.md)).
  - Does not change the calculation contract (D-19, [ADR 0007](0007-api-contract-details.md)).

## Context

[docs/GEOMETRY.md](../../docs/GEOMETRY.md) was proposed with six open questions (Q1–Q6). The review
[planning/GEOMETRY_SPEC_REVIEW.md](../GEOMETRY_SPEC_REVIEW.md) checked every formula numerically, against 60-digit
arithmetic and an unrelated geometry library. It found 0 Critical, 9 Major and 13 Minor issues, and raised three
more questions (D7–D9). The maintainer accepted all of the review's recommendations and corrections.

## Decision

| # | Question | Decision |
|---|---|---|
| D1 | Default tolerance | ε = max(10⁻⁹·D, 1024·ulp(M)), computed per call. D includes arc extremes. No default absolute (unit-bearing) tolerance in the geometry crate: an engineering tolerance is passed explicitly by the caller |
| D2 | Tolerance internal or input? | Internal in 0.1, returned with every Rust result. A calculation records ε (and ε_A) as a trace step only when a tolerance decision affected the result. No new contract inputs. A later optional `tolerance` input is a compatible addition (API_DESIGN §4) |
| D3 | `DEGENERATE_GEOMETRY` warning code | **Not added.** The crate returns typed Rust notes and errors. Calculations use existing codes: non-finite → `INVALID_VALUE`; undefined results → `OUT_OF_DOMAIN`; precision-limited → `ROUNDING_SENSITIVE`. How informational notes reach users is decided when Phase 4 first exposes drawing results through the contract. That change must also add "clients accept unknown warning codes" to API_DESIGN §5 |
| D4 | Self-intersecting polygons | Error for area and centroid; length unaffected. "Touch" means within ε. Pinch, spike and keyhole rings are refused too |
| D5 | Centroid scope | Straight-edged polygons only. Any bulge gives "not supported in 0.1" |
| D6 | ARCHITECTURE.md §3.3 | Amended: 2D in 0.1. Transformations are specified in GEOMETRY.md §11 and added in Phase 4 (part of 0.1). 3D is not in 0.1 |
| D6b | Where transformations live | `civkit-geometry`, added in Phase 4 |
| D7 | Self-intersection with bulged edges | Exact closed-form point–arc, segment–arc and arc–arc relations, internal to the check. No public intersection API |
| D8 | Zero-area value | Return the computed area, not 0, with orientation "undefined" and a note (ENGINEERING_STANDARDS §3: no rounding inside the engine) |
| D9 | Review corrections C-1 … C-25 | Applied to GEOMETRY.md and ARCHITECTURE.md §3.3. C-24 is applied as D3 describes. No API_DESIGN change, because no warning code is added |

## Settled during implementation (confirmed 2026-11-25)

Writing the code exposed points the accepted text did not settle. Each is marked **(impl.)** in GEOMETRY.md.

1. **8.6 is now defined geometrically and evaluated before 8.7.** A degenerate ring is one whose edges are all
   straight or have sagitta ≤ ε, with every vertex within ε of the line through the first vertex and the vertex
   farthest from it, so the whole ring lies within 2ε of one line. This replaces "|A| ≤ ε_A". Why:
   - A collinear ring always folds back on itself, so with 8.7 first, 8.6 could never be reached.
   - Under the old definition, a figure-8 whose lobes cancel (net shoelace exactly 0) would have been "zero area",
     with no error. That is a silently wrong result.
   - A simple, non-degenerate ring with |A| ≤ ε_A still gets orientation "undefined" and a note.
2. **Adjacent-edge conflict (8.7).** Two adjacent edges conflict when either holds:
   - a far endpoint lies within ε of the other edge; or
   - they cross, or reach a local minimum distance ≤ ε, more than ε from the shared vertex.

   Tangential contact only at the shared vertex (a cusp) is not a conflict. "Overlap along more than ε" in C-15
   could not be implemented without also flagging every sharp acute corner.
3. **A bulged edge whose sagitta is at most ulp(M) is checked as its chord.** Below the representation limit, the
   arc and its chord cannot be told apart, and for such bulges the radius can overflow.
4. **Results that overflow `f64` are an error** (8.1), never ±∞.
5. **Area and centroid are computed on the vertices as given.** The cleaned ring is used only for validity checks
   (8.5–8.7). Zero-length segments therefore keep their exact tiny contribution to length and area. This follows
   D8.
6. **Explicit tolerance:**
   - It must be finite and ≥ 0 (0 is allowed).
   - D and M are still computed, and ε_A = ε·D.
   - The precision-limited check (8.9) does not depend on ε.
   - **Added by the maintainer on confirmation:** a derived tolerance that is not finite is an overflow error
     (8.1). A huge but finite explicit ε with a large geometry could otherwise produce an infinite ε_A and make
     the orientation undefined. The rule applies to every tolerance computation, explicit or default; the
     default overflows only for a size above about 4 × 10¹⁵⁸.
7. **Centroid:** any non-zero bulge on any edge gives "not supported", including the bulge of a zero-length
   segment.
8. **Open polylines:** the last vertex's bulge is not used, but it must be finite.

## Consequences

- Phase 2B is implemented to GEOMETRY.md. Every public function cites its section.
- No contract change in 0.1 from geometry.
- Phase 4 inherits these tasks:
  - transformations in `civkit-geometry`;
  - OCS handling (GEOMETRY.md §11);
  - the contract representation of geometry notes, plus the "unknown warning codes" rule;
  - a code for "not supported in 0.1".

  *Settled 2026-11-27 by D-28 ([ADR 0016](0016-drawing-analysis.md)): GEOMETRY §11 rewritten, the §8.7 sweep
  prefilter, `DRAWING_GEOMETRY_NOTE`, `NOT_SUPPORTED` and the unknown-codes rule in API_DESIGN §5.*
- 2C note: traverse must decide for itself when a misclosure counts as zero before it asks for a misclosure
  direction (GEOMETRY.md §7).
