# ADR 0008 — Rust edition, minimum Rust version and contribution sign-off

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-25 10:48 IST · last edited 2026-11-25 10:48 IST

- **Status:** Accepted
- **Date:** 2026-11-25
- **Decision IDs:** D-10, D-11

## Context
Both decisions must be made before Phase 1 creates the Cargo workspace and accepts its first code.

## Decision
1. **D-10 — Rust edition and MSRV.**
   - All crates use the **2024 edition**.
   - The minimum supported Rust version (MSRV) is **latest stable − 2**, measured at Phase 1 start.
   - At that point stable is 1.100.0 (verified on the maintainer's machine), so the workspace declares `rust-version = "1.98"`.
   - The MSRV is raised deliberately, never by accident. CI builds with both the MSRV and the latest stable.
2. **D-11 — Sign-off.**
   - Every commit carries a `Signed-off-by:` line (`git commit -s`), certifying the
     [Developer Certificate of Origin 1.1](https://developercertificate.org/).
   - There is **no CLA**: contributors keep their copyright and license their work under Apache-2.0.
   - When an AI tool wrote part of a commit, the human who commits signs off and remains responsible.
     The AI is credited with a `Co-Authored-By:` trailer, not a sign-off.

## Consequences
- Contributors on a slightly older Rust can still build, while the project can use recent language features.
- The DCO is lightweight: no legal paperwork. It is checked in CI from Phase 1 (a commit without `Signed-off-by:` fails the check).
