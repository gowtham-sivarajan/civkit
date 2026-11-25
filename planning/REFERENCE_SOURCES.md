# Reference Sources

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-25 09:05 IST · last edited 2026-11-25 09:05 IST

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
| S2 | FHWA, *Introduction to Highway Hydraulics*, HDS-4, FHWA-NHI-08-090 (2008) — https://rosap.ntl.bts.gov/view/dot/41354 | Public domain (US federal government work) | Ch. 4 open-channel flow; Manning's equation; SI and US units | To confirm example numbers. Publisher errata list exists (Ex. 4.8) — check before use. |
| S3 | B. Dupen, *Applied Strength of Materials for Engineering Technology* (v14, 2018) — http://www.etcs.pfw.edu/~dupenb/ET_200/Applied%20Str%20of%20Mat%20for%20ET%20v14%20July%202018.pdf | CC BY-SA 4.0 | Beams: reactions, shear/moment, bending, deflection; all examples fully worked, SI and US units | To confirm example numbers. |
| S4 | J. Lord et al. (Virginia Tech), *Strength of Materials* — https://engineeringmechanicsoer.github.io/StrengthBook | CC BY-NC-SA | Ch. 7–11 beams, bending, beam deflection | To confirm. Second source for beam cases. |
| S5 | F. Udoeyo, *Structural Analysis* (Temple University, 2020) — https://temple.manifoldapp.org/projects/structural-analysis | CC BY-NC-ND 4.0 | Reactions, shear and moment diagrams, deflection of determinate beams | To confirm. Numbers only (ND). |
| S6 | G. Kouretzis, *Fundamentals of Foundation Engineering and their Applications*, 2nd ed. (Univ. of Newcastle / CAUL OER Collective, 2025) — https://oercollective.caul.edu.au/fundamentals-of-foundation-engineering | CC BY 4.0 (Australian Standards excerpts excluded) | Appendix A: soil phase relationships (water content, degree of saturation, void ratio …) | To confirm a worked numerical example. |
| S7 | M. S. Hossain et al., *Properties and Behavior of Soil — Online Lab Manual* (UT Arlington, 2021) — https://open.umn.edu/opentextbooks/formats/2973 | CC BY-NC 4.0 | Lab determination of water content, specific gravity, unit weight | To confirm. Second source for phase relationships. |
| S8 | C. Tiberius et al., *Surveying and Mapping* (TU Delft OPEN, 2021/2022), doi:10.5074/T.2021.007 | CC BY-NC-SA 4.0 | Part I land surveying: levelling (Ch. 3), total station | To confirm a levelling reduction example. Traverse/Bowditch coverage not confirmed. |
| S9 | US Army, FM 3-34.331 *Topographic Surveying* (2001) — https://www.globalsecurity.org/military/library/policy/army/fm/3-34-331/index.html | Public domain (US federal; "approved for public release; distribution is unlimited") | Ch. 6 traverse: angular and linear misclosure, compass-rule (Bowditch) adjustment; level reductions | **Partial:** numeric examples for coordinate differences only (para 6-28); no complete adjusted traverse. |
| S10 | USACE, EM 1110-1-1005 *Control and Topographic Surveying* — https://www.publications.usace.army.mil/portals/76/publications/engineermanuals/em_1110-1-1005.pdf | Public domain (US federal) | Traverse computations and adjustments; levelling | To check for a complete worked traverse. |
| S11 | C. Rapp, *Hydraulics in Civil Engineering* (Springer, 2024) — https://library.oapen.org/handle/20.500.12657/90899 | Open access (exact CC licence to confirm) | Channel flow, exercises with solutions | Licence and example to confirm. Second source for Manning. |

## 3. Rejected or on hold

| Source | Reason |
|---|---|
| A. Verruijt, *An Introduction to Soil Mechanics* (Springer, 2018) | Licence unclear: a download site lists it as CC BY, but Springer's own page shows it as subscription content. Not used unless an open licence is confirmed from the publisher. |
| Commercial textbooks (Pearson, Elsevier, McGraw Hill, Indian university texts, etc.) | Not openly licensed (D-18). |
| Lecture slides and notes found on file-sharing sites | Licence unknown or not open. |

## 4. Coverage for 0.1 (feeds D-7)

| 0.1 calculation | Sources | Status |
|---|---|---|
| Earthwork volume — average end area | S1 | ✅ Example confirmed and re-derived |
| Earthwork volume — prismoidal | S1 (formula only) | ⚠️ Needs an open worked example |
| Manning's equation | S2, S11 | 🔎 Example to locate in S2 |
| Simply supported beam (reactions, shear, moment, deflection) | S3, S4, S5 | 🔎 Examples to locate; three independent sources |
| Soil phase relationships | S6, S7 | 🔎 Example to locate |
| Levelling reduction (rise & fall / HI) | S8, S9, S10 | 🔎 Example to locate |
| Traverse closure + Bowditch (compass rule) | S9 (method, partial numbers), S10 | ⚠️ No complete open worked example found yet |
