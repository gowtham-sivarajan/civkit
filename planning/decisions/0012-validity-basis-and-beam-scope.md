# ADR 0012 — Validity ranges need a reliable basis; beam maximum shear and moment deferred from 0.1

> **Provenance:** Claude Sonnet 5.5 (`claude-sonnet-5-5`) · first drafted 2026-11-26 20:58 IST · last edited 2026-11-26 20:58 IST

- **Status:** Accepted (maintainer, 2026-11-26 20:56 IST: the answers to D3 and D4 of
  [PHASE_2D_REVIEW.md](../PHASE_2D_REVIEW.md))
- **Date:** 2026-11-26
- **Decision IDs:** D-24 (validity ranges), D-25 (beam maximum shear and moment)
- **Relation to earlier decisions:**
  - D-24 clarifies how [ENGINEERING_STANDARDS.md](../../docs/ENGINEERING_STANDARDS.md) §1 and §2 item 4 and
    [ACCEPTANCE_CRITERIA.md](../ACCEPTANCE_CRITERIA.md) §3 are read. It keeps D-19's mechanics (a breach is a warning by
    default, escalated per rule with a stated reason; a mathematically invalid input is always an error).
  - D-25 applies the consequence that [ADR 0009](0009-implementation-scope-0-1-calculations.md) (D-20) already foresaw:
    "If none is found, they move to a later release." ADR 0009 is not edited. The MVP_SCOPE table stays as the
    destination, as before; this record explains where 0.1 ships less.

## Context

The Phase 2D review found that none of the seven registered calculations has a validity rule, while the standards ask for
"validity limits, enforced as warnings or errors". For the traverse, levelling and earthwork, the accepted descriptor
proposals already record why: no source in the register states a limit. For Manning and the beam it was never written down.

The beam's maximum shear and moment have waited since Phase 0 for one more openly licensed worked example (D-20
condition 1). None has been found.

## Decision

### D-24 — Validity ranges only with a reliable basis

1. **Mathematical domain checks always apply.** A mathematically invalid input is always `OUT_OF_DOMAIN` (D-19).
2. **A validity range is documented only when there is a reliable basis for it**: a cited source that states the range
   or limit for the method. The rule's `reason` names the source.
3. **If no reliable basis is established, the calculation says so.** Its descriptor carries the assumption
   *"Validity range: none is stated, because no reliable source for one has been established. The mathematical domain
   of every input is still enforced."* A test enforces it for every calculation without a validity rule.
4. **No range is invented to fill the field.**
5. Adding a validity rule later changes the response for some inputs (a new warning or error), so it **bumps the
   calculation's version** (API_DESIGN §4).

### D-25 — Beam maximum shear and moment deferred from 0.1

1. `structural.beam.simply_supported` ships in 0.1 with reactions, deflection at a point and end slope (verified by
   C6–C8), and **without** maximum shear and maximum moment.
2. The search for a qualifying openly licensed worked example **continues** (D-18; D-20 condition 1: verified numerically,
   a diagram alone does not count).
3. When one is found, checked and accepted, maximum shear and moment come back as **new outputs**, a pure addition under
   API_DESIGN §4, in a later release.

## Consequences

- ENGINEERING_STANDARDS §1 and §2 item 4, and ACCEPTANCE_CRITERIA §3, say "a stated range with its basis, or an explicit
  statement that none is established".
- All seven calculations carry the D-24 statement in their assumptions; no result changes, so no version changes.
- The beam's summary cites D-25. IMPLEMENTATION_PLAN and REFERENCE_CASES record the deferral and the open search.
- PHASE_2D_REVIEW D3 and D4 are closed; Phase 2 can be tagged (D1).
