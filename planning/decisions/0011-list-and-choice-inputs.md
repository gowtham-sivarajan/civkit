# ADR 0011 — List and choice inputs in the calculation contract

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-25 22:03 IST · last edited 2026-11-25 22:03 IST

- **Status:** Accepted (maintainer, 2026-11-25: direction 21:32 IST, design 21:49 IST, final correction 21:54 IST)
- **Decision ID:** D-23
- **Relation to earlier decisions:**
  - Extends the contract of D-19 ([ADR 0007](0007-api-contract-details.md)) by pure addition.
  - Needed by D-20 ([ADR 0009](0009-implementation-scope-0-1-calculations.md)) for traverse, levelling and
    earthwork, and for its condition 3: an explicit method input for the pyramid.

## Context

The contract carried only scalar inputs and outputs. Three of the five 0.1 calculations take ordered lists:
traverse legs, levelling readings, earthwork stations. One needs a visible method choice. Options and the full
design are in [CONTRACT_LISTS_PROPOSAL.md](../CONTRACT_LISTS_PROPOSAL.md).

## Decision

1. **Parallel lists, one unit per list (option A).** A list input belongs to a named **group**. All lists in a group
   have the same length, and every list input is required in 0.1.
2. **Index sets and item labels.** An index set is a pair (group, offset), with offset −1, 0 or +1. Its item label
   ("leg", "segment") numbers entries from 1 in messages and trace text. Errors carry a 0-based `index` and trace
   steps a structured `item` (label, index). List outputs declare their index set; several outputs may share one.
3. **Choice inputs.** A choice input is a string from a declared list, required, with no default. Each choice is
   recorded exactly once as a `choice` trace step, before any numeric step, in declaration order.
4. **Limits.** At most 10 000 elements in any list, input or output. A group's maximum leaves room for its +1
   outputs.
5. **Errors.** No new error codes. Malformed or mismatched requests are `INVALID_VALUE` (or the scalar code for an
   element). Violations by a calculation (wrong output length, bad choice or item steps) are `INTERNAL`.
6. **Compatibility.** A pure addition. New fields are omitted from JSON when unused, so scalar calculations keep
   their exact JSON shape.
7. **Wording.** API_DESIGN §8's "array (vectorised) inputs" means batch calls, e.g. from Python. It does not rule
   out a list inside one calculation's input.

## Consequences

- API_DESIGN §2, §3, §5, §6, §7, §8 and §11 and `tests/reference/README.md` are updated.
- `civkit-core` is extended first, tests first. Then traverse, levelling and earthwork, one at a time, each with
  its descriptor proposed for review before coding.
- Not in 0.1: optional lists, list defaults, DMS lists, nested lists, records, and machine-readable conditional
  requirements (the beam's `point_load_position` stays in prose; Phase 3 shows it prominently).
