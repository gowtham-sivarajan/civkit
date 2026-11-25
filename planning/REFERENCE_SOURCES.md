# Reference Sources

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-25 09:05 IST · last edited 2026-11-25 10:08 IST

> **Status:** Living register. The only place where reference sources are listed (ADR 0003 "one home per fact").
> Policy: D-18 ([ADR 0006](decisions/0006-open-sources-and-0-1-calculations.md)). Use: [ENGINEERING_STANDARDS.md](../docs/ENGINEERING_STANDARDS.md).

## 1. Rule

Reference cases (worked examples used as tests) may only come from **openly licensed or public-domain** sources.
Proprietary textbooks are not used, even if the maintainer owns a copy.

**Preference order:** public domain → CC BY → CC BY-SA → CC BY-NC / BY-NC-SA → CC BY-NC-ND.

**What we take from a source:** only the numbers (inputs, expected outputs), the method name and a citation
(title, edition/version, section, example number, URL). Facts are not copyrightable; we never copy text, figures
or tables into the repository. This keeps even NC/ND sources usable without licence conflict with Apache-2.0.

**Every expected result is independently re-derived** before it becomes a test ([ENGINEERING_STANDARDS.md §2](../docs/ENGINEERING_STANDARDS.md)).
Errata are checked where the publisher lists them.

## 2. Approved sources

Licence and content were checked on 2026-11-25 from the publisher pages. "Example confirmed" means a worked
numerical example was seen; "to confirm" means the topic is covered but a specific worked example still has to
be located in task 0.8.

| # | Source | Licence | Covers (0.1 relevance) | Worked example status |
|---|---|---|---|---|
| S1 | *Fundamentals of Transportation*, "Earthwork" (Wikibooks; mirrored on Engineering LibreTexts §7.3) — https://en.wikibooks.org/wiki/Fundamentals_of_Transportation/Earthwork | CC BY-SA (Wikibooks site licence) | Average-end-area and prismoidal formulas | **Confirmed:** Example 1, 150 m road, areas 40/42/19/34 m² at 50 m spacing → 4,900 m³ (average end area). Re-derived: 2,050 + 1,525 + 1,325 = 4,900 ✔. No prismoidal example. |
| S2 | FHWA, *Introduction to Highway Hydraulics*, HDS-4 — 1997 metric edition used: https://www.fhwa.dot.gov/engineering/hydraulics/pubs/hds4intr.pdf (2008 edition FHWA-NHI-08-090 has a publisher errata list) | Public domain (US federal government work) | Ch. 4 open-channel flow; Manning's equation | **Confirmed:** Examples 4.1 and 4.2 → cases C5. |
| S3 | B. Dupen, *Applied Strength of Materials for Engineering Technology* (v14, 2018) — http://www.etcs.pfw.edu/~dupenb/ET_200/Applied%20Str%20of%20Mat%20for%20ET%20v14%20July%202018.pdf | CC BY-SA 4.0 | Beams: reactions, shear/moment, bending, deflection; all examples fully worked, SI and US units | Host unreachable on 2026-11-25; not needed (S4 + S5 cover beams). |
| S4 | J. Lord et al. (Virginia Tech), *Strength of Materials* — https://engineeringmechanicsoer.github.io/StrengthBook | CC BY-NC-SA 4.0 (confirmed on site) | Ch. 7–11 beams, bending, beam deflection | **Confirmed:** Examples 11.1, 11.5 → cases C7, C8. |
| S5 | F. Udoeyo, *Structural Analysis* (Temple University, 2020) — https://temple.manifoldapp.org/projects/structural-analysis | CC BY-NC-ND 4.0 | Reactions, shear and moment diagrams, deflection of determinate beams | **Confirmed:** Example 7.2 → case C6. Numbers only (ND). |
| S6 | G. Kouretzis, *Fundamentals of Foundation Engineering and their Applications*, 2nd ed. (Univ. of Newcastle / CAUL OER Collective, 2025) — https://oercollective.caul.edu.au/fundamentals-of-foundation-engineering | CC BY 4.0 (Australian Standards excerpts excluded) | Appendix A: soil phase relationships (water content, degree of saturation, void ratio …) | Checked: **formulas only, no worked example.** Erratum: ρw "≈ 10 Mg/m³" (should be ≈ 1). |
| S7 | M. S. Hossain et al., *Properties and Behavior of Soil — Online Lab Manual* (UT Arlington, 2021) — https://open.umn.edu/opentextbooks/formats/2973 | CC BY-NC 4.0 | Lab determination of water content, specific gravity, unit weight | Checked: sample data sheets only, no worked phase-relationship example. |
| S8 | C. Tiberius et al., *Surveying and Mapping* (TU Delft OPEN, 2021/2022), doi:10.5074/T.2021.007 | CC BY-NC-SA 4.0 | Part I land surveying: levelling (Ch. 3), total station | Not needed for 0.1 (S13 covers levelling; S12 covers traverse). |
| S9 | US Army, FM 3-34.331 *Topographic Surveying* (2001) — https://www.globalsecurity.org/military/library/policy/army/fm/3-34-331/index.html | Public domain (US federal; "approved for public release; distribution is unlimited") | Ch. 6 traverse: angular and linear misclosure, compass-rule (Bowditch) adjustment; level reductions | **Partial:** numeric examples for coordinate differences only (para 6-28); no complete adjusted traverse. Superseded for 0.1 by S12. |
| S10 | USACE, EM 1110-1-1005 *Control and Topographic Surveying* — https://www.publications.usace.army.mil/portals/76/publications/engineermanuals/em_1110-1-1005.pdf | Public domain (US federal) | Traverse computations and adjustments; levelling | Not needed for 0.1. |
| S11 | C. Rapp, *Hydraulics in Civil Engineering* (Springer, 2024) — https://library.oapen.org/handle/20.500.12657/90899 | Open access (exact CC licence to confirm) | Channel flow, exercises with solutions | Not needed for 0.1 (S2 has two examples). |
| S12 | US Navy, *Engineering Aid* (NAVEDTRA 14070; tpub label "Engineering Aid 1 – Advanced"), ch. 7 — mirror: https://engineeringtraining.tpub.com/14070/css/Linear-Error-Of-Closure-130.htm | Public domain (US government work; mirror site's compilation notice does not cover the underlying text) | Traverse: latitudes/departures, misclosure, compass rule; earthwork end areas | **Confirmed:** Figs 7-12 – 7-14 → case C3 (one source erratum). |
| S13 | US Navy, *Engineering Aid* (NAVEDTRA 14069; tpub label "Engineering Aid 3 – Beginning"), ch. 14 — mirror: https://engineeringtraining.tpub.com/14069/css/Level-Computations-480.htm | Public domain (US government work) | Differential levelling, level circuit closure and adjustment | **Confirmed:** Fig 14-14 → case C4; p. 14-10 → case C9 (one source erratum). |
| S14 | H. Chappidi, *Phase Relations* (slides, RGUKT, 2017) — https://www.slideshare.net/slideshow/phase-relations/82088099 | CC BY-SA 4.0 | Phase relationships (definitions, derivations) | Checked: no worked numerical example. |

## 3. Rejected or on hold

| Source | Reason |
|---|---|
| A. Verruijt, *An Introduction to Soil Mechanics* (Springer, 2018) | Licence unclear: a download site lists it as CC BY, but Springer's own page shows it as subscription content. Not used unless an open licence is confirmed from the publisher. |
| NPTEL, IIT Guwahati *Soil Mechanics* web course (105103097), ch. 2 worked examples | Licence unclear: NPTEL FAQ says copyright is held jointly by MoE, IITs and faculty, and CC licensing is only "likely … in the future". |
| N. Sivakugan, *Phase Relations* (1998 notes, hosted on geoengineer.org) | Worked examples, but **no licence stated**. |
| FHWA NHI-06-088 *Soils and Foundations* Vol. I (public domain), ch. 2 | Checked: phase relationships as formulas only; no worked example. |
| USACE EM 1110-2-1906 *Laboratory Soils Testing* (public domain), App. II | Checked: forms are blank in the text; no worked example found. |
| J. Mahun, *Traverse Computations* (jerrymahun.com "open access") | Complete compass-rule example, but **no licence stated** — not openly licensed. |
| Commercial textbooks (Pearson, Elsevier, McGraw Hill, Indian university texts, etc.) | Not openly licensed (D-18). |
| Lecture slides and notes found on file-sharing sites | Licence unknown or not open. |

## 4. Coverage for 0.1 (feeds D-7)

See [REFERENCE_CASES.md](REFERENCE_CASES.md) — the case-by-case record and the coverage table now live there.
