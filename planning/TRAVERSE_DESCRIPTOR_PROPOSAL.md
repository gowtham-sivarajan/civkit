# Proposal: the traverse descriptor (`surveying.traverse.compass_rule`)

> **Provenance:** Claude Sonnet 5.5 (`claude-sonnet-5-5`) · first drafted 2026-11-25 23:52 IST · last edited 2026-11-25 23:52 IST

> **Status:** **Proposed, for the maintainer's review.** D-20 condition 4 ([ADR 0009](decisions/0009-implementation-scope-0-1-calculations.md)):
> a descriptor is proposed and reviewed before any code. **No code has been written.** Nothing here is accepted until
> the maintainer says so; section 9 lists the seven questions that need an answer.
>
> Built on: D-20 (scope), D-23 (list inputs, [ADR 0011](decisions/0011-list-and-choice-inputs.md), implemented in
> `e948bc6`), [GEOMETRY.md](../docs/GEOMETRY.md) §3, §4, §7, reference case C3 ([REFERENCE_CASES.md](REFERENCE_CASES.md)).

## 1. Scope (D-20, unchanged)

From bearings and distances: **latitudes, departures, misclosure, precision ratio, and compass-rule (Bowditch)
adjusted latitudes and departures** (verified by C3). **Not in 0.1:** adjusted coordinates, area by coordinates,
open (link) traverses, angular misclosure balancing, other adjustment methods, misclosure direction.
D-20 forbids bonus behaviour, so every item above the line is exactly one output of section 4.

## 2. The method

For a closed traverse of legs k = 1 … n (leg k runs from station k to station k + 1; leg n closes onto station 1),
with azimuth α_k (clockwise from north, §4 of GEOMETRY.md) and horizontal distance d_k:

| Quantity | Formula | Notes |
|---|---|---|
| Latitude (ΔN) | Lat_k = d_k · cos α_k | GEOMETRY.md §3: latitude = Δy |
| Departure (ΔE) | Dep_k = d_k · sin α_k | departure = Δx; `civkit_geometry::components` returns (Dep, Lat) |
| Perimeter | P = Σ d_k | |
| Misclosures | M_N = Σ Lat_k, M_E = Σ Dep_k | zero for a perfectly closed traverse |
| Linear misclosure | E = √(M_N² + M_E²) | `hypot`, never overflows early |
| Relative misclosure | E / P | the precision ratio is 1 : (P / E); see Q-A |
| Corrections (compass rule) | c_N,k = −M_N · d_k / P, c_E,k = −M_E · d_k / P | proportional to leg length; sign as in the book |
| Adjusted | Lat′_k = Lat_k + c_N,k, Dep′_k = Dep_k + c_E,k | Σ Lat′ = Σ Dep′ = 0 |

No geodesy: plane grid values (GEOMETRY.md §3). The sign convention is the book's: **a correction is the negative of
the misclosure, shared out by length**, and an adjusted value is computed + correction.

## 3. Inputs

One group, two list inputs, no scalar inputs and **no choice input**: the compass rule is the only adjustment in 0.1, and
a choice with one value would add a field and no information. (ADR 0009 condition 3 is about the pyramid method, not this.)

| Group | `item` | `min_len` | Description |
|---|---|---|---|
| `legs` | `leg` | 3 | Traverse legs in order of measurement. Leg k runs from station k to station k + 1; the last leg ends at station 1. |

| Input | kind | dimension | internal unit | default display | required | domain | notes |
|---|---|---|---|---|---|---|---|
| `bearings` | list, group `legs` | angle | `rad` | `deg` | yes | none | Azimuth of each leg. `accepts_bearing`: a list may use `"unit": "bearing"` (`"N 3°45' E"`) or `deg`, `rad`, `grad`. DMS lists are not supported in 0.1 (D-23). Not normalised. |
| `distances` | list, group `legs` | length | `m` | `m` | yes | `> 0` | Horizontal distance of each leg. A zero or negative leg is `OUT_OF_DOMAIN`, with `index` and "leg 3". |

Design notes:
- A traverse cannot be defined by an angle alone, so both lists are required (D-23: every list is required in 0.1).
- Lists have equal length by the group rule. A mismatch is `INVALID_VALUE` naming both lengths.
- Station names (A, B, C …) are not inputs: "leg 3" is the item label everywhere. Interfaces may show names from the
  caller's own data; the engine does not need them.

## 4. Outputs

All list outputs belong to the group `legs` with **offset 0** and item label `leg` (the group's own index set).

| Output | kind | dimension | internal unit | default display | Meaning |
|---|---|---|---|---|---|
| `latitudes` | list (legs, 0) | length | `m` | `m` | Lat_k |
| `departures` | list (legs, 0) | length | `m` | `m` | Dep_k |
| `latitude_corrections` | list (legs, 0) | length | `m` | `m` | c_N,k |
| `departure_corrections` | list (legs, 0) | length | `m` | `m` | c_E,k |
| `adjusted_latitudes` | list (legs, 0) | length | `m` | `m` | Lat′_k |
| `adjusted_departures` | list (legs, 0) | length | `m` | `m` | Dep′_k |
| `perimeter` | scalar | length | `m` | `m` | P |
| `latitude_misclosure` | scalar | length | `m` | `m` | M_N (signed) |
| `departure_misclosure` | scalar | length | `m` | `m` | M_E (signed) |
| `linear_misclosure` | scalar | length | `m` | `m` | E |
| `relative_misclosure` | scalar | dimensionless | none | none | E / P |

A caller wanting feet names each output in `output_units` (the C3 case does, eleven entries). That is verbose, and a
presentation profile would remove it ([FEATURE_PROPOSALS.md](FEATURE_PROPOSALS.md) FP-28); it is not proposed for 0.1.

## 5. Validity, domain and errors

- **Domain:** `distances` > 0 (per element). `bearings`: any finite angle. Every element is checked as a scalar would be.
- **Validity rules: none.** No source in the register states a misclosure threshold for the compass rule. Inventing one
  would be unverified engineering content. A poor closure is visible in `relative_misclosure`; if the maintainer wants a
  caution threshold later, it needs a cited source first (Q-C).
- **Intermediate overflow:** a sum or hypot that is not finite (distances near 1e308 m) is `OUT_OF_DOMAIN` on
  `distances`, with the message "the perimeter is too large to compute". It is a caller problem, never a `NaN` output.
- **A perfectly closed traverse** (M_N = M_E = 0, or rounding noise): corrections are zero, `relative_misclosure` is
  0, and nothing divides by E. Because the direction of the misclosure is **not** an output (Q-B), the GEOMETRY.md §7
  question of when a misclosure counts as zero never arises.
- All other errors are the generic ones (D-19, D-23): `MISSING_UNIT`, `UNIT_MISMATCH`, `UNKNOWN_UNIT`, `INVALID_VALUE`.

## 6. Assumptions and references

Assumptions (the descriptor text):
1. The traverse is **closed**: the last leg ends at the first station. An open (link) traverse is not supported, and its
   "misclosure" would be meaningless.
2. Bearings are already balanced: angular misclosure is outside this calculation.
3. Plane grid, no geodesy: distances are horizontal and on the grid (GEOMETRY.md §3).
4. The compass rule distributes the misclosure in proportion to leg length; it does not model error sources.

References:
- US Navy, *Engineering Aid*, NAVEDTRA 14070, ch. 7 (Figs 7-12 to 7-14, pp. 7-10 to 7-11): the worked example and the
  compass-rule procedure. Public domain (S12 in [REFERENCE_SOURCES.md](REFERENCE_SOURCES.md)).

## 7. The trace

Per-leg steps carry the structured `item` (label `leg`, 0-based index); the text starts with "leg k:". No choice steps.

1. *Registry (not the calculation):* one conversion step per bearing element, `leg k: bearings: bearing N 3°45' E → azimuth …` (rad).
2. For each leg k: `leg k: Lat = d·cos α` and `leg k: Dep = d·sin α` (m).
3. `P = Σ d`, `M_N = Σ Lat`, `M_E = Σ Dep`, `E = √(M_N² + M_E²)`, `E / P`.
4. For each leg k: `leg k: c_N = −M_N·d/P` and `leg k: c_E = −M_E·d/P`.
5. For each leg k: `leg k: Lat′ = Lat + c_N` and `leg k: Dep′ = Dep + c_E`.

For n legs that is 6n + 5 numeric steps plus n conversion steps, so the 10 000-element limit is nowhere near.

## 8. Verification plan (tests first)

**Independent re-derivation (done 2026-11-25).** C3 was recomputed in 70-digit decimal arithmetic with a series for
sin and cos written for the purpose, outside the repository (no packages). Results, in feet:

| Leg | Lat | Dep | c_N | c_E | Lat′ | Dep′ |
|---|---|---|---|---|---|---|
| AB | 582.9592 | 38.2092 | −0.0718 | −0.5059 | 582.8874 | 37.7033 |
| BC | −47.1073 | 718.7179 | −0.0885 | −0.6237 | −47.1958 | 718.0942 |
| CD | −208.2333 | 39.7854 | −0.0261 | −0.1836 | −208.2594 | 39.6018 |
| DE | −246.0738 | −157.7745 | −0.0359 | −0.2531 | −246.1098 | −158.0276 |
| EF | 202.0943 | −191.6686 | −0.0342 | −0.2412 | 202.0601 | −191.9098 |
| FA | −283.3176 | −445.0052 | −0.0648 | −0.4568 | −283.3824 | −445.4619 |

P = 2 614.85 ft; M_N = 0.3214 ft; M_E = 2.2642 ft; **E = 2.2869 ft; E / P = 0.00087456 (1 : 1 143)**.

**Compared with the book** (printed to 0.01 ft): the largest difference in any entry is **0.0092 ft**, except the two
entries touched by the known CD departure erratum (book 39.77, exact 39.7854): **CD departure 0.0154** and **CD adjusted
departure 0.0118**. The book's own misclosures (Σ lat 0.33, Σ dep 2.24, E 2.26, 1 : 1 157) are therefore slightly off,
because they inherit the erratum. The computed values above are the independently derived ones.

**Tests, in this order:**

1. **Adjustment-only unit test, exact:** the compass rule applied to the *book's own* printed latitudes, departures and
   distances reproduces the book's corrections and adjusted values to 0.00515 ft (worst case, checked), which is below
   the book's rounding. This needs the adjustment as a pure function inside the calculation module, tested directly
   (the contract takes bearings, not latitudes).
2. **Closed forms:** a 4-leg square (N, E, S, W, equal lengths) has E = 0 and zero corrections; a rectangle with one
   leg 1 m short has M_N = 1 m (up to sign) and exact corrections; a single-direction pair of legs checks signs.
3. **Properties** (proptest, in the style of `civkit-geometry`): Σ adjusted latitudes and departures = 0 within a few
   ulps; the adjusted values do not depend on the starting leg (a cyclic shift of the legs shifts the outputs); doubling
   every distance doubles every length output and leaves `relative_misclosure` unchanged.
4. **Contract tests:** unit independence (feet in, metres out equals metres in); bearings as strings and as decimal
   degrees give identical results; element errors carry `index` and "leg k"; lists of unequal length, fewer than 3 legs
   and a zero distance each give the right code; the trace has one `item` step per leg for each per-leg expression.
5. **Reference case `tests/reference/surveying.traverse.compass_rule/navedtra-14070-fig7-12.json` (C3):**
   - request: the six bearings as `"unit": "bearing"` strings and the six distances in `ft`; `output_units` in `ft`;
   - expected list outputs checked with `absolute: 0.01` ft (the book's printed precision, no widening needed);
   - **`null` for CD departure and CD adjusted departure** (the erratum; recorded in `verification`);
   - `perimeter` 2 614.85 ft (the book gives 2 614.85) is checked; the book's `latitude_misclosure`, `departure_misclosure`,
     `linear_misclosure` and the ratio are **not** put in `expected`, because they inherit the erratum. They are covered
     by test 1 and by the closed forms. The exact values above are written into `verification`, with the erratum, not
     into the expected outputs ("recorded, never silently fixed").
   - `verification`: this section's re-derivation, including the script's method and the worst differences.
6. **Mutation check** (as for geometry): plant sign errors (correction without the minus), swap lat and dep, use
   P of the wrong list, and confirm each is caught.

## 9. Questions for the maintainer

| # | Question | Recommendation |
|---|---|---|
| **Q-A** | The precision ratio is conventionally written 1 : N with N = P / E, but N is infinite for a perfect closure and JSON cannot carry that. Return **`relative_misclosure` = E / P** (always finite, 0 when closed) and let interfaces show "1 : 1 143"? | **Yes.** N is derived from it, and it is never undefined. |
| **Q-B** | Omit the **misclosure direction** (a bearing)? D-20 does not list it, no worked example prints it, and omitting it removes the "when is a misclosure zero" problem (GEOMETRY.md §7). | **Omit** in 0.1. Add later with its own case. |
| **Q-C** | **No validity rules**: no cited threshold exists in the register. A poor closure is shown, not warned. | **No rule** until a source is added. |
| **Q-D** | `civkit-core` must depend on `civkit-geometry` (for `components`). It is an engine-internal path dependency, so the dependency rule holds and no external crate is added (`Cargo.lock` gains only the edge). Alternatively the calculation could use `sin_cos` directly and skip the dependency. | **Add the dependency**, so there is one tested trigonometry convention (negative zero, finiteness, GEOMETRY.md §4). |
| **Q-E** | Names and units of section 4 (eleven outputs, all lengths in `m` by default, `output_units` for feet). | **As proposed.** |
| **Q-F** | The reference-case handling in section 8, test 5: tolerance 0.01 ft, `null` for the two erratum-affected entries, the book's misclosure figures not asserted. | **As proposed.** |
| **Q-G** | ID `surveying.traverse.compass_rule`, version 1 (domain `surveying` already exists). Alternative: `surveying.traverse.closed`, which leaves room for a second adjustment method under one ID later. | **`compass_rule`**: D-20 fixes the method; a second method would be a new ID or a choice input (a version change). |

## 10. After approval

1. Record the answers here (status line) and in [AI_HANDOVER.md](AI_HANDOVER.md).
2. Tests 1 to 4 and the mutation check first (red), then `surveying.traverse.compass_rule` in
   `crates/civkit-core/src/calcs/surveying.rs`, then the reference case. Registered in `Registry::with_builtins`.
3. Full local checks (fmt, clippy `-D warnings`, tests, doc, dependency rule), then the maintainer's Windows run and commit.
4. Levelling and earthwork follow, each with its own descriptor proposal.
