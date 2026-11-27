# Proposal: the levelling descriptors

> **Provenance:** Claude Sonnet 5.5 (`claude-sonnet-5-5`) · first drafted 2026-11-26 00:05 IST · last edited 2026-11-26 00:12 IST by Claude Sonnet 5.5 (`claude-sonnet-5-5`)

> **Status:** **Accepted by the maintainer, 2026-11-26: all seven recommended answers (Q-A to Q-G, section 8).**
> This is the descriptor set reviewed under D-20 condition 4 ([ADR 0009](decisions/0009-implementation-scope-0-1-calculations.md)).
> Implementation follows section 9, tests first. The text below is unchanged from the proposal. One detail differs from
> section 9: the code lives in `crates/civkit-core/src/calcs/levelling.rs`, not `surveying.rs`, to keep each file
> readable; the IDs stay in the `surveying` domain as accepted. Once the code exists it is the source of truth.
>
> Built on: D-20 (scope and condition 2), D-23 (lists, [ADR 0011](decisions/0011-list-and-choice-inputs.md)), the traverse
> as the working pattern ([TRAVERSE_DESCRIPTOR_PROPOSAL.md](TRAVERSE_DESCRIPTOR_PROPOSAL.md), implemented), reference
> cases C4 and C9 ([REFERENCE_CASES.md](REFERENCE_CASES.md)).

## 1. Scope (D-20, unchanged)

> **Levelling:** height-of-instrument method; rise-and-fall method; closure; adjustment.
> Condition 2: the rise-and-fall reference test is labelled honestly. It verifies *numerical consistency* with C4's known
> elevations (the source reduces the same field book by the HI method). It is not independent validation of the
> rise-and-fall method from a source.

D-20 also says no bonus features and that only what a worked example verifies is implemented. Two consequences shape
everything below.

1. **Both examples contain only backsight and foresight pairs** (C4: five setups; C9: one). No example has an
   intermediate sight. So **0.1 reduces field books made of backsight/foresight pairs only**: every foresight is to a
   point whose elevation is wanted (a benchmark or a change point).
2. **On such field books the two methods give identical elevations.** This was checked in exact rational arithmetic
   (section 6). The methods differ in the working that is shown (a height of instrument per setup, or a rise or fall per
   setup) and in two of the outputs, not in the answer. A third method-independent step, closure and adjustment, follows
   either.

## 2. The decision that shapes the descriptors (Q-A)

| Option | Shape | Consequence |
|---|---|---|
| **B (recommended)** | Three calculations: `surveying.levelling.height_of_instrument`, `surveying.levelling.rise_and_fall`, `surveying.levelling.closure_adjustment` | The two reductions have fixed, unconditional outputs and need no known closing elevation, so a single open run (C9) is natural. The adjustment takes any reduced elevations. Each has its own ID, version and reference case; the rise-and-fall case carries its honest label by itself. The caller chains two calls for a full circuit (a workbook, FEATURE_PROPOSALS FP-08, would automate it later). |
| A | Two calculations (HI, rise-and-fall), each with a **required** `closing_elevation` and closure and adjustment inside | Fewer calls, but every run must have a known closing elevation, so C9 (no closing benchmark) can only be tested with a made-up closing value that forces zero closure. The adjustment is duplicated. |
| C | One calculation with a required `method` choice (`height_of_instrument`, `rise_and_fall`) | One ID, and the method is recorded as a choice step (D-23). But the outputs cannot depend on the choice, so they would be a superset (heights of instrument *and* rises and falls) in both cases, which is redundant and confusing; and closure still needs a closing elevation. |

The rest of this document describes **option B**.

## 3. `surveying.levelling.height_of_instrument` (version 1)

For setup k (k = 1 … n), with the previous point's elevation Z_(k−1) (Z_0 = the start elevation), backsight BS_k and
foresight FS_k:

| Quantity | Formula |
|---|---|
| Height of instrument | HI_k = Z_(k−1) + BS_k |
| Elevation of the point sighted | Z_k = HI_k − FS_k |
| Sums | ΣBS, ΣFS, and the net rise ΣBS − ΣFS (= Z_n − Z_0) |

**Inputs.** One group `setups` (item `setup`, `min_len` 1): *"Instrument setups in order of observation. Setup k reads a
backsight on the previous point (the start benchmark for setup 1) and a foresight on the next point."*

| Input | kind | dimension | internal unit | display | domain | notes |
|---|---|---|---|---|---|---|
| `start_elevation` | scalar | length | `m` | `m` | none (below datum is valid) | Elevation of the starting benchmark. Required. |
| `backsights` | list, group `setups` | length | `m` | `m` | ≥ 0 | Staff reading on the previous point, per setup |
| `foresights` | list, group `setups` | length | `m` | `m` | ≥ 0 | Staff reading on the next point, per setup |

**Outputs.** List outputs are on (`setups`, offset 0), item `setup`.

| Output | kind | unit | Meaning |
|---|---|---|---|
| `heights_of_instrument` | list | `m` | HI_k |
| `elevations` | list | `m` | Z_k, the elevation of the point sighted by setup k's foresight |
| `sum_backsights` | scalar | `m` | ΣBS |
| `sum_foresights` | scalar | `m` | ΣFS |
| `net_rise` | scalar | `m` | ΣBS − ΣFS (positive: the last point is higher than the start) |

**Trace:** per setup `setup k: HI = previous elevation + BS`, `setup k: elevation = HI − FS`; then `ΣBS`, `ΣFS`,
`net rise = ΣBS − ΣFS`. That is 2n + 3 steps; no choice steps.

## 4. `surveying.levelling.rise_and_fall` (version 1)

Same inputs and group. With Δ_k = BS_k − FS_k: rise_k = max(Δ_k, 0), fall_k = max(−Δ_k, 0),
**Z_k = Z_(k−1) + rise_k − fall_k**.

**Outputs:** `rises`, `falls`, `elevations` (lists on `setups`, offset 0, `m`; rises and falls are never negative), and
the same three scalars `sum_backsights`, `sum_foresights`, `net_rise`.

**Trace:** per setup `setup k: rise = max(BS − FS, 0)`, `setup k: fall = max(FS − BS, 0)`,
`setup k: elevation = previous elevation + rise − fall`; then the three sums. That is 3n + 3 steps.

**Honest label (D-20 condition 2).** Its reference cases use the same field books as the HI cases, the expected
elevations are the book's own (reduced there by the HI method), and every title and `verification` text says so:
*numerical consistency, not independent validation of the method.* The descriptor's `references` cite the source for the
field books and say the method is the standard textbook procedure; they do not claim the source worked it.

## 5. `surveying.levelling.closure_adjustment` (version 1)

For points k = 1 … n in order, with computed elevations Z_k, section lengths s_k (the length of the route from point
k − 1, or the start benchmark for k = 1, to point k) and the known elevation Z_known of the **last** point:

| Quantity | Formula | Notes |
|---|---|---|
| Closure error | e = Z_n − Z_known | Computed minus known: **positive means the run came out too high** (C4: +0.015 ft) |
| Total length | L = Σ s_k | |
| Cumulative length | S_k = s_1 + … + s_k | |
| Correction | c_k = −e · S_k / L | proportional to the distance run; opposite in sign to the error |
| Adjusted elevation | Z′_k = Z_k + c_k | Z′_n = Z_known exactly |

A closed loop (back to the start benchmark) is the same calculation with `closing_elevation` equal to the start elevation.

**Inputs.** One group `points` (item `point`, `min_len` 1): *"Points whose elevations were computed from the start
benchmark, in order of observation. The last point is the one whose elevation is also known."*

| Input | kind | dimension | internal unit | display | domain |
|---|---|---|---|---|---|
| `elevations` | list, group `points` | length | `m` | `m` | none |
| `section_lengths` | list, group `points` | length | `m` | `m` | > 0 |
| `closing_elevation` | scalar | length | `m` | `m` | none |

For C4 the section lengths are the setup distances (backsight distance + foresight distance), which is how the book
gets 2 140 ft.

**Outputs:** scalars `closure_error` and `total_length` (`m`); lists on (`points`, offset 0, item `point`)
`corrections` and `adjusted_elevations` (`m`).

**Trace:** `L = Σs`, `e = Z_n − Z_known`, then per point `point k: correction = −e·S/L` and
`point k: adjusted elevation = Z + correction`. That is 2n + 2 steps.

## 6. Independent re-derivation (done 2026-11-26)

C4 and C9 were recomputed in exact rational arithmetic (Python `fractions`, outside the repository, no packages).

- **HI reduction of C4 reproduces the book to the last printed digit:** HIs 139.822, 139.819, 136.875, 132.430, 138.242;
  elevations 134.851, 132.367, 131.018, 131.169, 136.457; ΣBS 24.620, ΣFS 21.326, difference 3.294; last − start = 3.294.
- **Rise-and-fall on the same field book gives exactly the same elevations** (identical rational values).
  Rises 1.688, 0, 0, 0.151, 5.288; falls 0, 2.484, 1.349, 0, 0.
- **Closure and adjustment:** total length 2 140 ft; closure error +0.0150; corrections −0.00308, −0.00659, −0.01093,
  −0.01234, −0.01500; **adjusted 134.84792 (book: 134.848), 132.36041, 131.00707, 131.15666, 136.44200** (= the known
  elevation exactly). The book prints only the first adjusted value.
- **C9:** HI 376.57, elevation B 375.15, with the backsight **11.56** that the book actually used. With the printed
  11.65 the result would be 376.66 and 375.24, which confirms the erratum recorded in REFERENCE_CASES.md.
- The book's permissible closure, 0.05·√(2140/5280) = 0.0318 ft, is source-specific (a class constant and miles) and is
  **not** reproduced (Q-C).

## 7. Verification plan (tests first)

**Per reduction:** closed forms (a rising staircase, a descent, equal readings giving zero rise); `Z_n − Z_0 = ΣBS − ΣFS`;
units (feet in, metres out equals metres in); errors with `index` and "setup k" (unequal lists, a negative reading,
missing input, missing unit); the trace counts and structured items; negative elevations accepted.
**Cross-method (property, proptest):** the HI and rise-and-fall calculations give the same elevations for any
field book, to 1e-12; shifting the start elevation shifts every elevation by the same amount.
**Adjustment:** zero closure gives zero corrections; the last adjusted elevation equals `closing_elevation`; the last
correction equals −e and every correction is proportional to the cumulative length; adding a constant to all elevations and the
closing elevation leaves the corrections unchanged; multiplying all lengths by a constant leaves them unchanged;
a zero or negative section length is `OUT_OF_DOMAIN` naming "point k"; overflow of the total length is an error,
never `NaN`.
**Mutation check** as for geometry and traverse: wrong sign of the closure error, correction by count instead of length,
HI computed from the wrong previous elevation, rise and fall swapped, sums off by one setup; each must be caught, with a
green baseline first.

**Reference cases** (tolerance 0.0005 ft, half the book's last printed digit; exact arithmetic reproduces the book, so no
widening is needed):

| File | Calculation | Checks | Label |
|---|---|---|---|
| `navedtra-14069-fig14-14` | HI | HIs, elevations, ΣBS, ΣFS, net rise (C4) | published worked example |
| `navedtra-14069-p14-10` | HI | HI 376.57, elevation B 375.15 (C9, tolerance 0.005) | published; `verification` records the 11.65 / 11.56 erratum |
| `navedtra-14069-fig14-14` | rise-and-fall | the book's elevations (C4) | **consistency with the HI reduction, not independent validation** |
| `navedtra-14069-p14-10` | rise-and-fall | elevation B (C9) | same label |
| `navedtra-14069-fig14-14` | closure_adjustment | closure error 0.015, total length 2 140 ft, adjusted TBM 16 = 134.848; the other four adjusted entries are `null` (not printed by the source) | published; the exact unprinted values are in `verification` |

Rises, falls, HIs under rise-and-fall and corrections are not printed by the book and are not asserted in reference
cases; closed-form tests cover them.

## 8. Questions for the maintainer (all answered: the recommendation was accepted for every one, 2026-11-26)

| # | Question | Recommendation |
|---|---|---|
| **Q-A** | Structure: three calculations (section 2, option B), two with a required closing elevation (A), or one with a method choice (C)? | **B**, for the reasons in section 2 |
| **Q-B** | **No intermediate sights in 0.1**: no example verifies them (D-20). Both methods then give identical elevations, so the two calculations differ in working and in HI versus rise and fall outputs. Accept this and add intermediate sights later, to both? | **Accept**; D-20 requires both methods, and the difference becomes real once intermediate sights exist |
| **Q-C** | **No permissible-closure output and no warning** (the book's 0.05·√M is a class constant in miles, an extra input, and is not listed in D-20) | **Omit** in 0.1 |
| **Q-D** | The adjustment takes `elevations`, `section_lengths` and `closing_elevation` (not cumulative distances, not the raw field book). | **As proposed**; the caller chains the reduction's `elevations` into it |
| **Q-E** | Domains: readings ≥ 0, section lengths > 0, elevations and the start and closing elevations unbounded. | **As proposed** |
| **Q-F** | Reference-case handling in section 7: tolerance 0.0005 ft, `null` for the unprinted adjusted values, the rise-and-fall label, C9 with BS = 11.56 and the erratum recorded. | **As proposed** |
| **Q-G** | IDs `surveying.levelling.height_of_instrument`, `.rise_and_fall`, `.closure_adjustment`; output names as in sections 3 to 5; the group items `setup` and `point`. | **As proposed** |

## 9. After approval

1. Record the answers in the status line and in AI_HANDOVER.md.
2. Tests first (red), then `surveying.rs` gains the three calculations, sharing small pure functions; then the five
   reference cases; then the mutation check and the full local checks; then the maintainer's Windows run and commit.
3. Earthwork (average end area, with an explicit pyramid method, cases C1 and C2) follows with its own proposal.
