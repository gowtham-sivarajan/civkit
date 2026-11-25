# Reference Cases (Phase 0 collection)

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-25 09:50 IST · last edited 2026-11-26 00:23 IST by Claude Sonnet 5.5 (`claude-sonnet-5-5`)

> **Status:** Task 0.8. Worked examples collected from open sources ([REFERENCE_SOURCES.md](REFERENCE_SOURCES.md)),
> each **re-derived independently** by script (Claude Opus 5.5, 2026-11-25). **Human engineering review: pending.**
> In Phase 1 these become data files in `tests/reference/`. Only numbers and citations are recorded here (D-18).

Legend: ✅ re-derived, matches within tolerance · ⚠️ matches with a documented source issue · ❌ not usable as written

---

## C1 — Earthwork volume, average end area ✅

> **Implemented 2026-11-26** as `tests/reference/quantities.earthwork.average_end_area/wikibooks-ex1.json` (method
> `average_end_area`; the source does not say cut or fill, so the areas are entered as cut; segments 2 050, 1 525,
> 1 325 and total 4 900 m³, tolerance 0.5).

- **Source:** S1 — *Fundamentals of Transportation*, "Earthwork", Example 1 (Wikibooks, CC BY-SA).
- **Input:** level-terrain road 150 m; end areas 40, 42, 19, 34 m² at 0, 50, 100, 150 m.
- **Expected:** 2,050 + 1,525 + 1,325 = **4,900 m³**.
- **Re-derived:** 4,900.0 m³. Tolerance 0.5 m³.

## C2 — Earthwork volume, average end area + pyramid (zero end area) ✅

> **Implemented 2026-11-26** as `…/wikibooks-solution.json` with the method `pyramid_at_zero_end` requested explicitly
> (totals fill 4 087.5, cut 1 058.3, net fill 3 029.2, tolerance 0.05; the two printed pyramids 83.3 and 25, every other
> per-segment entry `null`). The exact re-derivation shows why the method must be explicit: the same data under average
> end area gives fill 4 100, cut 1 100, net 3 000 (a unit test).

- **Source:** S1 — *Fundamentals of Transportation/Earthwork/Solution* (Wikibooks, CC BY-SA).
- **Input:** stations 0+00 fill 60; 0+50 fill 50; 0+75 cut 0 / fill 25; 1+00 cut 10 / fill 5; 1+15 cut 15 / fill 0; 1+50 cut 30 (m²).
- **Expected:** fill **4,087.5 m³**, cut **1,058.3 m³**, net **3,029.2 m³** fill required. Pyramid segments: 10×25/3 = 83.3 m³, 5×15/3 = 25 m³.
- **Re-derived:** 4,087.5 / 1,058.333 / 3,029.167 m³. Tolerance 0.05 m³.
- **Note:** the pyramid volume equals the prismoidal formula with A₂ = 0, Aₘ = A₁/4 (checked: 83.333 m³), so this is a
  worked example of the **prismoidal formula's pyramid special case only**. Source inconsistency: the Problem page
  says "0+00 to 2+50", the Solution page "0+00 and 1+50"; data end at 1+50.

## C3 — Traverse: latitudes/departures, misclosure, compass (Bowditch) rule ⚠️

- **Source:** S12 — US Navy, *Engineering Aid* (NAVEDTRA 14070), ch. 7, Figs 7-12 to 7-14, pp. 7-10 – 7-11. Public domain.
- **Input (bearings, ft):** AB N 3°45′ E 584.21 · BC S 86°15′ E 720.26 · CD S 10°49′ E 212.00 ·
  DE S 32°40′ W 292.31 · EF N 43°29′ W 278.53 · FA S 57°31′ W 527.54. Perimeter 2,614.85.
- **Expected (book):**

  | Line | Lat | Dep | Lat corr | Dep corr | Adj lat | Adj dep |
  |---|---|---|---|---|---|---|
  | AB | +582.95 | +38.20 | −0.07 | −0.50 | +582.88 | +37.70 |
  | BC | −47.10 | +718.72 | −0.09 | −0.62 | −47.19 | +718.10 |
  | CD | −208.23 | +39.77 | −0.03 | −0.18 | −208.26 | +39.59 |
  | DE | −246.07 | −157.77 | −0.04 | −0.25 | −246.11 | −158.02 |
  | EF | +202.09 | −191.67 | −0.03 | −0.24 | +202.06 | −191.91 |
  | FA | −283.31 | −445.01 | −0.07 | −0.45 | −283.38 | −445.46 |
  | Σ | +0.33 | +2.24 | −0.33 | −2.24 | 0.00 | 0.00 |

  Linear misclosure 2.26 ft; ratio ≈ 1:1,157.
- **Re-derived:**
  - Compass-rule adjustment applied to the book's latitudes/departures: **all 12 adjusted values match within 0.006 ft**.
  - Latitudes/departures from bearings: 11 of 12 match within 0.011 ft.
  - **Source discrepancy:** CD departure — book 39.77, but 212.00 × sin 10°49′ (book's own table value 0.18767) = **39.79**.
    Consequently the true departure misclosure is 2.26 ft (book 2.24) and linear misclosure 2.29 ft (book 2.26).
- **Test design:** (a) adjustment test uses the book's lat/dep as inputs — exact match expected; (b) lat/dep-from-bearing
  test excludes CD departure, which is recorded as a known source erratum.
- **Implemented 2026-11-26:** (a) is a unit test in `crates/civkit-core/src/calcs/surveying.rs` (worst difference
  0.00515 ft); (b) is `tests/reference/surveying.traverse.compass_rule/navedtra-14070-fig7-12.json` (tolerance 0.01 ft;
  `null` for CD departure and CD adjusted departure; the book's misclosure figures are not asserted because they inherit
  the erratum). Exact values from an independent 70-digit re-derivation: M_N 0.3214 ft, M_E 2.2642 ft, E 2.2869 ft,
  E/P 0.00087456 (1 : 1 143); every other entry within 0.0092 ft of the book.

## C4 — Differential levelling (HI method), closure and proportional adjustment ✅

> **Implemented 2026-11-26** as `tests/reference/surveying.levelling.height_of_instrument/navedtra-14069-fig14-14.json`
> (HIs, elevations, ΣBS, ΣFS, difference; tolerance 0.0005 ft), `…closure_adjustment/…` (closure error, total length,
> adjusted TBM 16; the four unprinted adjusted values are `null`) and, as a **consistency check only** (D-20 condition 2),
> `…rise_and_fall/…`. Exact rational re-derivation: the book is reproduced to its last digit; adjusted values
> 134.84792, 132.36041, 131.00707, 131.15666, 136.44200 (the last equals the known BM 19).

- **Source:** S13 — US Navy, *Engineering Aid* (NAVEDTRA 14069), ch. 14, Fig. 14-14 and text pp. 14-13 – 14-15. Public domain.
- **Input:** BM 35 = 133.163 ft. BS / FS (ft): 6.659 / 4.971 (to 16), 4.968 / 7.452 (to 17), 4.508 / 5.857 (to 18),
  1.412 / 1.261 (to TP 1), 7.073 / 1.785 (to BM 19). Setup distances BS/FS (ft): 220/220, 250/250, 310/310, 100/100, 190/190.
  Known BM 19 = 136.442 ft.
- **Expected:** HIs 139.822, 139.819, 136.875, 132.430, 138.242; elevations 16 = 134.851, 17 = 132.367, 18 = 131.018,
  TP1 = 131.169, BM19 = 136.457. ΣBS 24.620, ΣFS 21.326, difference 3.294. Closure error +0.015 ft over 2,140 ft.
  Adjusted TBM 16 = **134.848 ft**. Permissible 0.05√(2140/5280) = 0.032 ft.
- **Re-derived:** every value matches to 0.0005 ft. Adjusted 17 and 18 (not printed in source): 132.360 and 131.007 ft.
- **Note:** rise-and-fall reduction of the same readings must give identical elevations; the same data serve both methods.

## C5 — Manning's equation, trapezoidal channel ✅

- **Source:** S2 — FHWA, *Introduction to Highway Hydraulics*, HDS-4 (June 1997, metric), §4.3, PDF pp. 50–52. Public domain.
- **Example 4.2 (primary):** B = 1.5 m, side slopes 1V:2H, n = 0.013, S = 0.002, Q = 3 m³/s. Book: y = 0.60 m gives
  Q ≈ 2.96 m³/s (A = 1.62 m², P = 4.18 m), V = Q/A = 1.85 m/s.
  **Re-derived:** A 1.620, P 4.183, Q(0.60) 2.961, V 1.852. ✅
- **Example 4.1 (secondary):** B = 2 m, 1V:2H, S = 0.003, y = 0.5 m, n = 0.02. Book: P = 4.24 m, R = 0.35 m,
  V = 1.36 m/s, Q ≈ 2.0. **Re-derived:** P 4.236, R 0.3541, V 1.371, Q 2.056. The book rounds R to 0.35 before
  computing V, which accounts for the 0.8 % difference. Usable with a stated 1 % relative tolerance. Printed unit
  "m²/s" for Q is a typo (should be m³/s).
  *(An earlier note in this session wrongly called Example 4.1 an arithmetic error; the script re-derivation corrected that.)*

## C6 — Simply supported beam: UDL + midspan point load ✅

- **Source:** S5 — F. Udoeyo, *Structural Analysis*, §1.7, Example 7.2 (Engineering LibreTexts, CC BY-NC-ND).
- **Input:** L = 8 ft, w = 2 kip/ft (full span), P = 10 kip at midspan, E = 10,000 ksi, I = 1,000 in⁴.
- **Expected:** reactions 13 kips each; EIθ_A = −82.67 kip·ft² → θ_A = −0.0012 rad; EIy_C = −213.35 kip·ft³ → y_C = −0.0031 ft ≈ −0.04 in.
- **Re-derived:** 13.0; 82.667; 213.333 (book 213.35 from rounded intermediate); θ_A 0.00119 rad; y_C 0.0369 in. ✅

## C7 — Simply supported beam, UDL: midspan deflection ✅

- **Source:** S4 — J. Lord et al., *Strength of Materials* (Virginia Tech), Example 11.1 (CC BY-NC-SA).
- **Input:** L = 10 m, w = 20 kN/m, E = 200 GPa, I = 350 × 10⁶ mm⁴.
- **Expected:** y(5 m) = −37.2 mm. **Re-derived:** 37.202 mm. ✅

## C8 — Simply supported beam, UDL + off-centre point load (superposition) ✅

- **Source:** S4 — Example 11.5.
- **Input:** L = 8 m, w = 20 kN/m, F = 50 kN at a = 6 m (b = 2 m), E = 210 GPa, I = 275 × 10⁶ mm⁴; deflection at x = 4 m.
- **Expected:** UDL −18.47 mm, point load −6.35 mm, total −24.8 mm. **Re-derived:** 18.470, 6.349, 24.820 mm. ✅

## C9 — Single-setup levelling (HI) ⚠️

> **Implemented 2026-11-26** as `…height_of_instrument/navedtra-14069-p14-10.json` (backsight 11.56, erratum recorded in
> its `verification`) and, as a consistency check only, `…rise_and_fall/navedtra-14069-p14-10.json`. With the printed
> 11.65 the result would be 376.66 and 375.24, not the book's 376.57 and 375.15.

- **Source:** S13 — NAVEDTRA 14069, p. 14-10.
- **Input/Expected:** BM 365.01 ft; HI 376.57; FS 1.42 → elevation B **375.15 ft**.
- **Source discrepancy:** the text states the backsight as **11.65 ft** but computes with **11.56** (365.01 + 11.56 = 376.57).
  The case is usable with BS = 11.56 (consistent with HI and result); the erratum is recorded.

---

## Coverage of the 0.1 candidate list (D-7 entry rule)

| Calculation | Cases | Result |
|---|---|---|
| Earthwork volume — average end area | C1, C2 | ✅ In 0.1 |
| Earthwork volume — prismoidal (general Aₘ) | C2 (pyramid special case only) | ❎ Deferred to a later release (maintainer, 2026-11-25) — no open general example after two searches. The zero-end pyramid (V = AL/3) stays in 0.1 with the average-end-area module. |
| Manning's equation | C5 (two examples) | ✅ In 0.1 |
| Simply supported beam | C6, C7, C8 (two independent sources) | ✅ In 0.1 |
| Levelling reduction | C4, C9 | ✅ In 0.1 |
| Traverse closure + compass (Bowditch) rule | C3 | ✅ In 0.1 (with documented source erratum) |
| Soil phase relationships | none | ❎ Deferred to a later release (maintainer, 2026-11-25) — no openly licensed worked example after two searches |

Second search (2026-11-25 09:56 IST) checked FHWA NHI-06-088 ch. 2 (formulas only), USACE EM 1110-2-1906 App. II
(blank forms), NPTEL IIT Guwahati soil mechanics (has examples, but licence unclear) and Sivakugan 1998 notes
(has examples, no licence). None qualifies under D-18. See REFERENCE_SOURCES.md §3.

**Human engineering review of C1–C9:** deferred by the maintainer until the product is complete (2026-11-25).

## Source issues found (to report upstream where possible)

| Source | Issue |
|---|---|
| NAVEDTRA 14070 Fig 7-13 | CD departure 39.77; should be 39.79 (212.00 × 0.18767) |
| NAVEDTRA 14069 p. 14-10 | Backsight printed 11.65; arithmetic uses 11.56 |
| FHWA HDS-4 (1997) Ex 4.1 | R rounded to 0.35 before use (V 1.36 vs 1.37); discharge unit printed m²/s |
| Wikibooks Earthwork Problem page | Station range 2+50 vs 1+50 on Solution page |
| Kouretzis, Appendix A | ρw "≈ 10 Mg/m³" — should be ≈ 1 Mg/m³ |
| Lord et al. (S4) Example 7.2 | Maximum moment printed as 50.4 kip·ft at C (x = 6 ft); zero shear is at x = 5.1 ft, where M = 52.02 kip·ft |
