# ADR 0007 — API contract details for 0.1

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-25 10:02 IST · last edited 2026-11-25 10:02 IST

- **Status:** Accepted
- **Date:** 2026-11-25
- **Decision ID:** D-19

## Context
After D-5 (units) and D-6 (FFI), [API_DESIGN.md](../../docs/API_DESIGN.md) still had nine open items (TBDs) in the contract, units, Python, FFI and MCP sections.
Phase 0 cannot exit with open items there (ACCEPTANCE_CRITERIA.md). Task 0.7 proposed a resolution for each; the maintainer accepted all nine.

## Decision
The nine resolutions in [API_DESIGN.md §12](../../docs/API_DESIGN.md#12-resolved-tbds--summary-task-07-accepted-2026-11-25). In short:
1. **Version:** a request may omit `version` and get the latest. The response always states the version used, and anything saved stores it.
2. **Output units:** an `output_units` map, per output. The trace stays in SI.
3. **Validity ranges:** an input outside a method's stated range gives a warning by default.
   A calculation can escalate a specific rule to an error, with a recorded reason. Mathematically invalid input is always an error.
4. **No silent default unit:** a dimensional input given as a bare number is rejected (`MISSING_UNIT`).
5. **Angles:** decimal degrees, radians or grads; DMS as `[d, m, s]` plus a `negative` flag; quadrant bearings as a structured value.
6. **Units:**
   - Named units plus compounds built with `*`, `/` and `^`; no general prefix parsing.
   - `lb` and `k` are rejected as ambiguous (use `lbm`/`lbf` and `kip`).
   - `ft` is the international foot; the US survey foot is `ft_us`.
7. **Python input:** a quantity can be a string, a tuple or `civkit.Q`. A bare number is allowed only for a dimensionless input.
8. **C interface:** an integer `civkit_abi_version()` that host programs check when they load the library.
9. **MCP:** a `--tools per-calc|generic|both` option, default `both`. Revisit the default at about 40 calculations.

## Consequences
- Rejecting a missing or ambiguous unit means some inputs need extra typing. In exchange, CivKit never makes a silent unit error,
  which is the failure engineers fear most in calculation tools.
- Surveying calculations take bearings directly, so traverse data can be entered as it appears in field books and textbooks (reference case C3).
- A calculation's version can only be omitted in interactive use. Saved results stay reproducible.
