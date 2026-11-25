# ADR 0009 — Implementation scope of the 0.1 calculations

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-25 15:31 IST · last edited 2026-11-25 15:31 IST

- **Status:** Accepted (maintainer, 2026-11-25 15:31 IST)
- **Date:** 2026-11-25
- **Decision ID:** D-20
- **Relation to earlier decisions:** narrows *implementation* within D-7 / [ADR 0006](0006-open-sources-and-0-1-calculations.md) and
  [MVP_SCOPE.md](../MVP_SCOPE.md) §3. It does not rewrite them; the accepted MVP scope stays as recorded and is the destination.

## Context
MVP_SCOPE.md names five calculations for 0.1 at the level of calculation families. Comparing that wording with
the worked examples C1–C9 ([REFERENCE_CASES.md](../REFERENCE_CASES.md)) showed outputs in the scope that no example verifies
(beam maximum shear and moment), outputs verified but not listed (beam end slope), methods with no source worked
example (rise and fall), and unstated behaviour (when the zero-end pyramid applies).

Principle (maintainer): *the accepted scope defines the destination; the worked examples define what we can responsibly ship.*
0.1 exposes only capabilities whose behaviour we can explain and verify.

## Decision

| Calculation | 0.1 implements | Not in 0.1 |
|---|---|---|
| **Simply supported beam** | Reactions; deflection at a requested point; end slope θ_A (verified by C6). Maximum shear and maximum moment **only once** an additional openly licensed worked example is collected that verifies them numerically. | Anything a worked example does not verify. |
| **Levelling** | Height-of-instrument method; rise-and-fall method; closure; adjustment. | — |
| **Manning's equation** | Trapezoidal channel with known depth: area, wetted perimeter, hydraulic radius, velocity, discharge (verified by C5). | Normal-depth solver; circular pipes; other sections. |
| **Traverse** | From bearings and distances: latitudes, departures, misclosure, precision ratio, compass-rule (Bowditch) adjusted latitudes and departures (verified by C3). | Adjusted coordinates; area by coordinates. |
| **Average end area** | Average-end-area volumes (cut, fill, net) and the zero-end pyramid as an **explicitly requested** method. | Automatic switching between methods. |

### Conditions
1. **Beam:** the additional source must verify maximum shear and maximum moment **numerically**. A shear or moment
   diagram alone is not a substitute.
2. **Rise and fall:** its reference test is labelled honestly. It verifies *numerical consistency* with C4's known
   elevations (the source reduces the same field book by the HI method). It is not independent validation of the rise-and-fall method from a source.
3. **Pyramid:** the method is selected visibly in the input contract (an explicit method input). The user always knows whether
   they requested average end area or the zero-end pyramid, and the chosen method is recorded in the calculation steps.
   The engine never switches methods silently.
4. **All five:** documented inputs, outputs, options, validation rules, warnings and calculation steps (the descriptor and trace
   of [API_DESIGN.md](../../docs/API_DESIGN.md)). **No undocumented convenience behaviour, and no bonus features in 0.1.**

## Consequences
- Phase 2C has an exact checklist. Anything outside the table above needs a new decision before it is implemented.
- Beam maximum shear and maximum moment depend on one more reference case. If none is found, they move to a later
  release under the D-7 entry rule; reactions, deflection and end slope ship regardless.
- The accepted MVP scope remains the record of what 0.1 aims for. Where 0.1 ships less, this ADR is the explanation.
