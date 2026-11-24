# Decisions

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-24 23:41 IST (as part of HANDOFF.md) · last edited 2026-11-25 00:29 IST

The register of every project decision. Status flow: **Proposed → Accepted** (or Rejected / Superseded).
Every Accepted decision gets a short record in [`decisions/`](decisions/) (`NNNN-title.md`, template at the end).

Priority: **P0** = must be decided to leave Phase 0 · **P1** = before the relevant phase starts.

## Settled by the original proposal

| # | Decision |
|---|---|
| S-1 | Rust is the core implementation language. |
| S-2 | Free, open source, local-first, no accounts, no telemetry, no cloud dependency. |
| S-3 | Interfaces: Python (PyO3), MCP, AutoCAD/Civil 3D (C#/.NET), MATLAB, Tauri desktop app. |
| S-4 | Civil engineering first; other disciplines later. |
| S-5 | Drawing analysis is read-only first; any modification needs explicit user confirmation. |
| S-6 | An OSI-approved licence will be used. |

## Register

| # | Pri | Question | Recommendation / outcome | Status |
|---|---|---|---|---|
| D-1 | P0 | Final project name? | Decide before any package or repository is published. Check GitHub, crates.io, PyPI, trademark. "CivKit" competes with *Civilization*-game projects in search results. | Proposed |
| D-2 | P0 | Licence? | **Apache-2.0 OR MIT** (dual; Rust-ecosystem norm; permissive; Apache adds a patent grant). Avoid GPL: a GPL plugin linked against AutoCAD's proprietary libraries is legally awkward. MPL-2.0 is the middle option. Must be chosen before the first outside contribution. | Proposed |
| D-3 | P0 | AutoCAD plugin in 0.1 or 0.2? | **0.2**; DXF-based drawing analysis in 0.1 instead. | Proposed |
| D-4 | P0 | Roadmap order? | Python in Phase 3 **with** MCP (was Phase 5); drawing model Phase 4; AutoCAD Phase 5. | Proposed |
| D-5 | P0 | Units approach? | Runtime quantities (SI value + dimension exponents), angle as its own dimension, ambiguous units rejected. See [API_DESIGN.md §7](../docs/API_DESIGN.md#7-units). | Proposed |
| D-6 | P0 | FFI contract? | JSON over a minimal C ABI. See [API_DESIGN.md §9](../docs/API_DESIGN.md#9-ffi-cratescivkit-ffi). | Proposed |
| D-7 | P0 | Which calculations in 0.1? | First-principles set in [MVP_SCOPE.md §3](MVP_SCOPE.md#3-in-scope). Adjust to what can be verified from available textbooks. | Proposed |
| D-8 | P1 | First design code after 0.1? | Whichever the first users work with — e.g. IS 456 / IS 800 for users in India. One code at a time. | Proposed |
| D-9 | P1 | Supported AutoCAD versions? | **AutoCAD 2027 only to start** (.NET 10, the installed version). Add 2025/2026 (.NET 8) only if users need it. | Proposed |
| D-10 | P1 | Rust edition / minimum Rust version? | 2024 edition; latest stable at Phase 1 start; MSRV "latest stable − 2". | Proposed |
| D-11 | P1 | Contribution sign-off? | DCO (`git commit -s`), no CLA. | Proposed |
| D-12 | P0 | Neutral drawing model + DXF reader in Rust? | Yes. | **Accepted** 2026-11-25 · [ADR 0002](decisions/0002-neutral-drawing-model.md) |
| D-13 | P1 | MCP SDK? | Official `rmcp` crate, stdio only; pin version at Phase 3 start. | Proposed |
| D-14 | P1 | When does SQLite enter? | Phase 6 (desktop history). | Proposed |
| D-15 | P0 | Repository layout? | Monorepo: `crates/` · `bindings/` · `integrations/` · `apps/` · `tests/`; engine never depends outside `crates/`. | **Accepted** 2026-11-25 · [ADR 0001](decisions/0001-monorepo-layout.md) |
| D-16 | P0 | Documentation structure and AI provenance? | Root README/ROADMAP/CONTRIBUTING · `docs/` (stable design) · `planning/` (plans, decisions, handover); provenance line on every AI-written file. | **Accepted** 2026-11-25 · [ADR 0003](decisions/0003-documentation-structure.md) |

## Decision record index

| ADR | Title | Decision |
|---|---|---|
| [0001](decisions/0001-monorepo-layout.md) | Monorepo with an interface-independent engine | D-15 |
| [0002](decisions/0002-neutral-drawing-model.md) | Neutral drawing model and DXF reader in the Rust engine | D-12 |
| [0003](decisions/0003-documentation-structure.md) | Documentation structure and AI provenance | D-16 |

## Template for a new decision record

```markdown
# ADR NNNN — Title

> **Provenance:** <author or model (model ID)> · first drafted YYYY-MM-DD HH:MM IST · last edited YYYY-MM-DD HH:MM IST

- **Status:** Accepted
- **Date:** YYYY-MM-DD
- **Decision ID:** D-NN

## Context
## Decision
## Consequences
```
