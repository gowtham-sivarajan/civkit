# Proposal: the earthwork descriptor (`quantities.earthwork.average_end_area`)

> **Provenance:** Claude Sonnet 5.5 (`claude-sonnet-5-5`) · first drafted 2026-11-26 00:15 IST · last edited 2026-11-26 00:23 IST by Claude Sonnet 5.5 (`claude-sonnet-5-5`)

> **Status:** **Accepted by the maintainer, 2026-11-26: all seven recommended answers (Q-A to Q-G, section 8.3).**
> This is the descriptor reviewed under D-20 condition 4 ([ADR 0009](decisions/0009-implementation-scope-0-1-calculations.md)).
> Implementation follows section 9, tests first. The text below is unchanged from the proposal; once the code exists
> (`crates/civkit-core/src/calcs/quantities.rs`) it is the source of truth.
>
> Built on: D-20 (scope and condition 3), D-23 (lists and **choice inputs**, [ADR 0011](decisions/0011-list-and-choice-inputs.md)),
> the traverse and levelling proposals as the working pattern, reference cases C1 and C2 ([REFERENCE_CASES.md](REFERENCE_CASES.md)).
> This is the last Phase 2C calculation. It is also the first to use a **choice input**.

## 1. Scope (D-20, unchanged)

> **Average end area:** average-end-area volumes (cut, fill, net) and the zero-end pyramid as an **explicitly requested**
> method. Not in 0.1: automatic switching between methods.
> Condition 3: the method is selected visibly in the input contract. The user always knows whether they requested average
> end area or the zero-end pyramid, and the chosen method is recorded in the calculation steps. **The engine never
> switches methods silently.**

Also out of 0.1 (MVP_SCOPE, D-7): the general prismoidal formula (no open general example), shrink and swell factors,
mass-haul ordinates, and anything else no worked example verifies.

## 2. The method

Stations i = 1 … n at chainages x_i (strictly increasing). At each station the cut area C_i and the fill area F_i
(cross-section areas, m², both ≥ 0). Segment k lies between stations k and k + 1, with length L_k = x_(k+1) − x_k.
Cut and fill are computed separately, with the same rule. For a kind with end areas A₁ and A₂:

| Method (required input `method`) | Volume of segment k |
|---|---|
| `average_end_area` | V = (A₁ + A₂)/2 · L, for every segment |
| `pyramid_at_zero_end` | If **exactly one** of A₁, A₂ is zero: V = A · L / 3, with A the non-zero end area (a pyramid or wedge ending at the zero section). Otherwise V = (A₁ + A₂)/2 · L (both ends non-zero, or both zero, which gives 0). |

Totals: total cut = Σ V_cut, total fill = Σ V_fill, **net fill = total fill − total cut** (positive: fill must be brought
in; negative: cut is in surplus). The sign follows the source, which reports C2 as "3 029.2 m³ fill required" (Q-D).

The rule is applied **per kind and per segment**. In C2, segment 3 has a fill from 25 to 5 (average end area) but a cut from
0 to 10 (pyramid); segment 4 has a fill from 5 to 0 (pyramid) but a cut from 10 to 15 (average end area).

## 3. Inputs

One group, three lists and one **choice input**.

| Group | `item` | `min_len` | Description |
|---|---|---|---|
| `stations` | `station` | 2 | Cross-section stations in order of increasing chainage. Segment k lies between station k and station k + 1. |

| Input | kind | dimension | internal unit | default display | domain | notes |
|---|---|---|---|---|---|---|
| `method` | **choice** | none | none | none | `average_end_area`, `pyramid_at_zero_end` | **Required, no default.** Recorded as the first trace step (D-23). |
| `chainages` | list, group `stations` | length | `m` | `m` | none | Must be **strictly increasing**; checked by the calculation (Q-C). Station 0+50 is 50 m. |
| `cut_areas` | list, group `stations` | area | `m^2` | `m^2` | ≥ 0 | Cut cross-section area at each station. Give 0 where there is no cut. |
| `fill_areas` | list, group `stations` | area | `m^2` | `m^2` | ≥ 0 | Fill cross-section area at each station. Give 0 where there is no fill. |

All lists are required (D-23), so a run with only cut gives `fill_areas` as zeros.

## 4. Outputs

List outputs are on (`stations`, offset **−1**), item **`segment`**: one value between consecutive stations.

| Output | kind | dimension | internal unit | default display | Meaning |
|---|---|---|---|---|---|
| `cut_volumes` | list | volume | `m^3` | `m^3` | Cut volume of each segment |
| `fill_volumes` | list | volume | `m^3` | `m^3` | Fill volume of each segment |
| `total_cut` | scalar | volume | `m^3` | `m^3` | Σ cut volumes |
| `total_fill` | scalar | volume | `m^3` | `m^3` | Σ fill volumes |
| `net_fill` | scalar | volume | `m^3` | `m^3` | total fill − total cut (positive: fill required) |

Segment lengths are not outputs: they are derived from the input and no source prints them (D-20, no bonus).

## 5. Validity, domain and errors

- **Domain:** areas ≥ 0 (per element). Chainages: any finite number.
- **Ordering (calculation-level check):** the contract has no rule across the elements of one list, so the calculation
  rejects a chainage that is not greater than the previous one: `OUT_OF_DOMAIN`, `input` = `chainages`, `index` = the
  offending station, message "station 3: chainage 40 m must be greater than station 2 (50 m)".
- **Method:** a value other than the two choices is `INVALID_VALUE` listing both (D-23, by the registry). A missing method is `MISSING_INPUT`.
- **Validity rules: none.** No cited limit exists for when average end area is inadequate, and the explicit method input
  already makes the user choose the formula.
- **Overflow:** a volume or total that is not finite is `OUT_OF_DOMAIN`, never a `NaN`.

## 6. Assumptions and references

1. End areas are cross-sectional areas perpendicular to the centreline at the stations, and the section changes linearly
   between stations.
2. Average end area is an approximation, exact for prismatic and linearly varying sections; **it overstates a segment
   that tapers to a zero end** (a pyramid has A·L/3, not A·L/2), which is why that method is offered, never applied
   automatically.
3. No shrink or swell, no curvature correction, no allowance for the general prismoidal correction (not in 0.1).
4. Cut and fill are not netted within a station.

References: *Fundamentals of Transportation*, "Earthwork", Example 1 and the Solution page (Wikibooks; mirrored on
Engineering LibreTexts §7.3), CC BY-SA (S1 in [REFERENCE_SOURCES.md](REFERENCE_SOURCES.md)). Only numbers, method names and
citations are taken (D-18).

## 7. The trace

1. **Choice step first** (D-23, exactly one): `{ "step": "method", "choice": "pyramid_at_zero_end" }`.
2. For each segment k, two steps (item label `segment`), the text showing **which formula was used**:
   - `segment k: cut volume = (A₁ + A₂)/2·L` or `segment k: cut volume = A·L/3 (zero end, pyramid)`;
   - `segment k: fill volume = …` likewise.
3. `ΣV_cut`, `ΣV_fill`, `net fill = ΣV_fill − ΣV_cut`.

That is 1 + 2(n − 1) + 3 steps. A user can read from the trace exactly which segments were treated as pyramids.

## 8. Verification plan and questions

### 8.1 Independent re-derivation (done 2026-11-26)

C1 and C2 were recomputed in exact rational arithmetic (Python `fractions`, outside the repository, no packages).

| | Result | Book |
|---|---|---|
| **C1** `average_end_area`, areas 40, 42, 19, 34 m² at 0, 50, 100, 150 m (entered as cut) | segments 2 050, 1 525, 1 325; total **4 900** m³ | 4 900 |
| **C2** `pyramid_at_zero_end` | cut 0, 0, **83.333**, 187.5, 787.5 → **1 058.333**; fill 2 750, 937.5, 375, **25**, 0 → **4 087.5**; net fill **3 029.167** | 1 058.3; 4 087.5; 3 029.2; pyramids 83.3 and 25 |
| **C2 data with `average_end_area`** | cut **1 100**, fill **4 100**, net fill **3 000**; the two zero-end segments become 125 and 37.5 | not in the source |

The last row is the reason for the explicit method: the same field data gives 12.5 m³ more fill and 41.7 m³ more cut when
the wrong formula is used, and nothing in the result would say so. It becomes a unit test.

### 8.2 Tests, in this order (tests first)

1. **Closed forms:** constant area A over length L gives A·L under both methods; a segment with one zero end gives A·L/2
   (average) and A·L/3 (pyramid); both ends zero gives 0; cut and fill are independent.
2. **The choice is visible and never silent:** the first trace step is the choice; the same data under the two methods
   differs exactly on the zero-end segments and nowhere else; an unknown method is `INVALID_VALUE` listing both; a missing
   method is `MISSING_INPUT`.
3. **Errors:** non-increasing chainages (equal and decreasing) with `index` and "station k"; unequal lists; a negative
   area naming "station k"; fewer than 2 stations; missing units; overflow.
4. **Units:** feet and square feet in, cubic metres out (and `output_units` for `yd^3`).
5. **Properties** (proptest): the totals equal the sum of the segment volumes; net fill = fill − cut; swapping the cut and
   fill lists swaps the totals and flips the sign of the net; scaling every chainage gap by s scales every volume by s;
   when no area is zero the two methods are identical; with a zero end the pyramid total is never larger than the
   average total, and for a single zero-end segment equals 2/3 of it.
6. **Reference cases** (tolerances as REFERENCE_CASES.md): `wikibooks-ex1` (C1; `method` `average_end_area`; the source does
   not say cut or fill, so the areas are entered as cut; `cut_volumes` [2 050, 1 525, 1 325] ±0.5, `total_cut` 4 900 ±0.5);
   `wikibooks-solution` (C2; `pyramid_at_zero_end`; `total_fill` 4 087.5, `total_cut` 1 058.3, `net_fill` 3 029.2, each
   ±0.05; the only printed per-segment values are the two pyramids: `cut_volumes` `[null, null, 83.3, null, null]` and
   `fill_volumes` `[null, null, null, 25, null]`, ±0.05). The source erratum (Problem page "0+00 to 2+50", Solution page
   "0+00 and 1+50") is recorded in `verification`; the data end at 1+50.
7. **Mutation check** as before, with a verified green baseline: pyramid factor 1/2, pyramid applied when both ends are
   non-zero, pyramid applied to the wrong kind, net with the wrong sign, method ignored, non-increasing chainage accepted,
   segment length from the wrong station, totals missing the last segment.

### 8.3 Questions for the maintainer (all answered: the recommendation was accepted for every one, 2026-11-26)

| # | Question | Recommendation |
|---|---|---|
| **Q-A** | The method: a **required choice input** `method` with the values `average_end_area` and `pyramid_at_zero_end`; the pyramid applies, per kind and per segment, only where **exactly one** end area is zero; never chosen by the engine. | **As proposed** (ADR 0009 condition 3) |
| **Q-B** | Inputs: `chainages`, `cut_areas` and `fill_areas`, all required, with zeros for a kind that does not occur. C1 is entered as cut because the source does not say. | **As proposed** |
| **Q-C** | Strictly increasing chainages are checked by the calculation (`OUT_OF_DOMAIN` on `chainages` with `index`), because the contract has no cross-element rule. | **As proposed** |
| **Q-D** | Outputs and sign: per-segment `cut_volumes` and `fill_volumes`, `total_cut`, `total_fill`, and **`net_fill` = fill − cut (positive: fill required)**, matching how the source reports C2. The alternative is `net_cut` (cut − fill, the mass-haul convention). | **`net_fill`**, so the printed 3 029.2 is asserted as printed |
| **Q-E** | Out of 0.1, as stated: general prismoidal formula, shrink and swell, mass-haul ordinates, per-segment net, curvature correction. No validity rules. | **As proposed** |
| **Q-F** | The verification plan in 8.2, including the C2-under-average-end-area unit test and the `null` entries in the reference case. | **As proposed** |
| **Q-G** | ID `quantities.earthwork.average_end_area` (domain `quantities` exists), version 1, with the names above; group `stations`, item `station`, outputs' item `segment`. Alternative: `quantities.earthwork.end_area`. | **`average_end_area`**, the name D-20 uses |

## 9. After approval

1. Record the answers in the status line and in [AI_HANDOVER.md](AI_HANDOVER.md).
2. Tests first (red); then `crates/civkit-core/src/calcs/quantities.rs`, registered in `Registry::with_builtins`; then the
   two reference cases; then the mutation check and the full local checks; then the maintainer's Windows run and commit.
3. That completes the five D-20 calculations in 2C. Phase **2D** (integration and regression, tag `phase-2-done`) follows.
