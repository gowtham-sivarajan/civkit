# ADR 0004 — Name, licence and public repository

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-25 00:36 IST · last edited 2026-11-25 00:36 IST

- **Status:** Accepted
- **Date:** 2026-11-25
- **Decision IDs:** D-1, D-2, D-17

## Context
Before any code is written the maintainer wants the project public on GitHub, so that development is
transparent from the first commit. Publishing makes three things effectively permanent: the name people find,
the licence terms of everything pushed, and the commit history (including author details).

## Decision
1. **Name (D-1): CivKit.** Package name `civkit`.
   Checked on 2026-11-25: `civkit` is unclaimed on crates.io and PyPI. Known overlaps, accepted by the maintainer:
   a dormant Bitcoin-related open-source project using the same name (GitHub organisation `civkit`, last active
   2024) and an unrelated npm package `civkit`. Web searches for "CivKit" also return *Civilization*-game projects.
2. **Licence (D-2): Apache License 2.0.** Free, OSI-approved, permissive, with an explicit patent grant and
   patent-retaliation clause. The full text is in `LICENSE`, unmodified. Contributions are licensed under the
   same terms (Apache-2.0, section 5).
3. **Public from day one (D-17).** The GitHub repository is public from its first push, including all planning
   history. The maintainer accepts that the author email in commit history is public.

## Consequences
- Everything pushed — planning documents included — is available under Apache-2.0.
- Planning documents must not contain personal or machine-specific details; AI_HANDOVER.md was cleaned before
  the first push.
- GitHub Actions minutes are free for public repositories, which suits the CI plan.
- Apache-2.0 is incompatible with GPL-2.0-only projects (compatible with GPL-3.0). This only matters if CivKit
  code is ever combined into a GPL-2.0-only project.
- If the name collision causes confusion later, renaming remains possible until packages are published to
  crates.io or PyPI; after that it becomes costly.
- Security reports go through GitHub private vulnerability reporting (docs/SECURITY.md), which must be enabled
  after the repository is created.
