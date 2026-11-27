# ADR 0003 — Documentation structure and AI provenance

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-25 00:29 IST · last edited 2026-11-27 23:16 IST by Claude Sonnet 5.5 (`claude-sonnet-5-5`)

- **Status:** Accepted. *Amended 2026-11-27 (maintainer): `AI_HANDOVER.md` and the working logs `FUZZ_LOG.md` and
  `DXF_CROSS_CHECK_LOG.md` are kept locally and updated, but are not in the repository. Decision records, plans,
  proposals and reviews stay public (D-17 unchanged).*
- **Date:** 2026-11-25
- **Decision ID:** D-16

## Context
The first planning pass put almost everything into one long `README.md` and a `HANDOFF.md`. As the project
grows, design rules, plans, decisions and session state change at very different speeds and are read by
different people (newcomers, contributors, the maintainer, AI assistants). Much of the work is drafted with AI
assistance, and the maintainer wants it always visible which model wrote what, and when.

## Decision
Organise documentation from the start as:

```
README.md  ROADMAP.md  CONTRIBUTING.md        # root: front page, order of work, how to contribute
docs/                                          # stable design: how it works, rules it follows
  ARCHITECTURE.md  API_DESIGN.md  ENGINEERING_STANDARDS.md  OFFLINE_POLICY.md  SECURITY.md
planning/                                      # changes often: how we get there
  MVP_SCOPE.md  IMPLEMENTATION_PLAN.md  ACCEPTANCE_CRITERIA.md
  DECISIONS.md  decisions/NNNN-*.md            # register + individual decision records
  AI_HANDOVER.md                               # current state; read first when resuming
  ORIGINAL_PROPOSAL.md                         # the concept proposal, unchanged
```

Refinements to the maintainer's proposed structure:
1. Individual decision records live in `planning/decisions/`; `DECISIONS.md` is the register and index.
2. The original proposal is kept, unchanged, as `planning/ORIGINAL_PROPOSAL.md`.
3. The units specification lives in `docs/API_DESIGN.md` §7 rather than a separate file.
4. `LICENSE` will sit at the root once D-2 is decided.
5. Each fact has one home: phase exit criteria live only in `ACCEPTANCE_CRITERIA.md`; tasks only in
   `IMPLEMENTATION_PLAN.md`; decisions only in `DECISIONS.md`. Other documents link rather than repeat.

**Provenance rule:** every file created or edited by an AI assistant carries a provenance line near the top
with model name, model ID and timestamp in IST (comment line where Markdown is not possible), updated on every
edit. Commits containing AI-written content carry a `Co-Authored-By` trailer.

`HANDOFF.md` is replaced by `planning/AI_HANDOVER.md`; `docs/00-original-proposal.md` and `docs/decisions/`
move into `planning/`.

## Consequences
- Newcomers get a short README; deep material is one link away.
- Stable design (`docs/`) is not churned by day-to-day planning (`planning/`).
- More files to keep consistent; the "one home per fact" rule limits drift.
- Provenance is visible in every file, not only in git history.
