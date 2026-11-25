# ADR 0006 — Openly licensed reference sources and the 0.1 calculation set

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-25 09:05 IST · last edited 2026-11-25 09:05 IST

- **Status:** Accepted
- **Date:** 2026-11-25
- **Decision IDs:** D-7, D-18

## Context
Every calculation needs reference cases: known input, documented method, independently verified expected
result (ENGINEERING_STANDARDS.md). The maintainer decided that these must come only from openly licensed books,
so that anyone can check CivKit's tests against the source without buying a textbook, and so the project never
depends on proprietary material.

## Decision
1. **D-18 — Open sources only.** Reference cases come only from public-domain or openly licensed sources
   (Creative Commons). Preference: public domain → CC BY → CC BY-SA → CC BY-NC(-SA) → CC BY-NC-ND. Only numbers,
   method names and citations are taken; no text, figures or tables are copied. Approved sources are listed in
   [REFERENCE_SOURCES.md](../REFERENCE_SOURCES.md), the single register.
   US federal government works (e.g. FHWA, US Army, USACE manuals) count as public domain.
2. **D-7 — 0.1 calculation set is driven by available open sources.** Candidate calculations:
   - earthwork volume (average end area; prismoidal),
   - Manning's equation for uniform open-channel flow,
   - simply supported beam (reactions, shear, moment, deflection),
   - soil phase relationships,
   - levelling reduction (rise & fall / height of instrument),
   - traverse closure with compass-rule (Bowditch) adjustment.

   **Entry rule:** a calculation is in 0.1 only once at least one worked example from an approved source has
   been collected and independently re-derived (task 0.8). Calculations without such an example by the end of
   Phase 0 move to a later release rather than being tested against unverifiable numbers.

## Consequences
- The 0.1 list may shrink: as of 2026-11-25 only the average-end-area method has a confirmed open worked
  example; traverse/Bowditch and the prismoidal method have no complete open example yet.
- Engineers can contribute by finding open worked examples, which is now an explicit, valuable task.
- Public-domain US manuals use US practice and often US units; unit handling (D-5) makes this harmless.
- NC/ND-licensed sources are usable because only facts (numbers) and citations enter the repository.
