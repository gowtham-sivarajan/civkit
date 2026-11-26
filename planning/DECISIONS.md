# Decisions

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-24 23:41 IST (as part of HANDOFF.md) · last edited 2026-11-26 23:44 IST by Claude Sonnet 5.5 (`claude-sonnet-5-5`)

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
| D-1 | P0 | Final project name? | **CivKit.** `civkit` is free on crates.io and PyPI (checked 2026-11-25). Known overlaps: a dormant Bitcoin project of the same name (GitHub org `civkit`, last active 2024) and an unrelated npm package `civkit`. | **Accepted** 2026-11-25 · [ADR 0004](decisions/0004-name-licence-public-repo.md) |
| D-2 | P0 | Licence? | **Apache-2.0** — free, permissive, includes a patent grant. | **Accepted** 2026-11-25 · [ADR 0004](decisions/0004-name-licence-public-repo.md) |
| D-3 | P0 | AutoCAD plugin in 0.1 or 0.2? | **0.2**; DXF-based drawing analysis in 0.1 instead. | **Accepted** 2026-11-25 · [ADR 0005](decisions/0005-scope-order-units-ffi.md) |
| D-4 | P0 | Roadmap order? | Python in Phase 3 **with** MCP (was Phase 5); drawing model Phase 4; AutoCAD Phase 5. | **Accepted** 2026-11-25 · [ADR 0005](decisions/0005-scope-order-units-ffi.md) |
| D-5 | P0 | Units approach? | Runtime quantities (SI value + dimension exponents), angle as its own dimension, ambiguous units rejected. See [API_DESIGN.md §7](../docs/API_DESIGN.md#7-units). | **Accepted** 2026-11-25 · [ADR 0005](decisions/0005-scope-order-units-ffi.md) |
| D-6 | P0 | FFI contract? | JSON over a minimal C ABI. See [API_DESIGN.md §9](../docs/API_DESIGN.md#9-ffi-cratescivkit-ffi). | **Accepted** 2026-11-25 · [ADR 0005](decisions/0005-scope-order-units-ffi.md) |
| D-7 | P0 | Which calculations in 0.1? | Driven by open sources: candidates in [MVP_SCOPE.md §3](MVP_SCOPE.md#3-in-scope); a calculation enters 0.1 only once an open worked example is collected and re-derived (task 0.8). | **Accepted** 2026-11-25 · [ADR 0006](decisions/0006-open-sources-and-0-1-calculations.md) |
| D-8 | P1 | First design code after 0.1? | Whichever the first users work with — e.g. IS 456 / IS 800 for users in India. One code at a time. | Proposed |
| D-9 | P1 | Supported AutoCAD versions? | **AutoCAD 2027 only to start** (.NET 10, the installed version). Add 2025/2026 (.NET 8) only if users need it. | Proposed |
| D-10 | P1 | Rust edition / minimum Rust version? | 2024 edition; latest stable at Phase 1 start; MSRV "latest stable − 2" → `rust-version = "1.98"` (stable 1.100.0 at Phase 1 start). | **Accepted** 2026-11-25 · [ADR 0008](decisions/0008-rust-edition-and-sign-off.md) |
| D-11 | P1 | Contribution sign-off? | DCO (`git commit -s`), no CLA. | **Accepted** 2026-11-25 · [ADR 0008](decisions/0008-rust-edition-and-sign-off.md) |
| D-12 | P0 | Neutral drawing model + DXF reader in Rust? | Yes. | **Accepted** 2026-11-25 · [ADR 0002](decisions/0002-neutral-drawing-model.md) |
| D-13 | P1 | MCP SDK? | `rmcp` pinned `=3.12.0`, `default-features = false`, `server` + `transport-io`; both protocol eras (2026-07-28 and 2025-11-25); tool name = calculation ID; full contract response as `structuredContent` plus text; contract errors as `isError` with the envelope. Gates G1–G3 before the server counts as done. | **Accepted** 2026-11-26 · [ADR 0013](decisions/0013-mcp-sdk-and-surface.md) |
| D-14 | P1 | When does SQLite enter? | Phase 6 (desktop history). | Proposed |
| D-15 | P0 | Repository layout? | Monorepo: `crates/` · `bindings/` · `integrations/` · `apps/` · `tests/`; engine never depends outside `crates/`. | **Accepted** 2026-11-25 · [ADR 0001](decisions/0001-monorepo-layout.md) |
| D-16 | P0 | Documentation structure and AI provenance? | Root README/ROADMAP/CONTRIBUTING · `docs/` (stable design) · `planning/` (plans, decisions, handover); provenance line on every AI-written file. | **Accepted** 2026-11-25 · [ADR 0003](decisions/0003-documentation-structure.md) |
| D-17 | P0 | When does the code become public? | **Public GitHub repository from day one**, including the planning history. | **Accepted** 2026-11-25 · [ADR 0004](decisions/0004-name-licence-public-repo.md) |
| D-18 | P0 | Where may reference cases come from? | **Only openly licensed or public-domain sources.** Register: [REFERENCE_SOURCES.md](REFERENCE_SOURCES.md). | **Accepted** 2026-11-25 · [ADR 0006](decisions/0006-open-sources-and-0-1-calculations.md) |
| D-19 | P0 | API contract details (version omission, output units, validity breaches, bare numbers, angle formats, unit list, Python quantities, ABI version, MCP tool surface)? | The nine resolutions in [API_DESIGN.md §12](../docs/API_DESIGN.md#12-resolved-tbds--summary-task-07-accepted-2026-11-25). | **Accepted** 2026-11-25 · [ADR 0007](decisions/0007-api-contract-details.md) |
| D-20 | P1 | Exactly what does each 0.1 calculation implement? | Only what a worked example verifies; beam max shear/moment need one more open numerical example; rise and fall tested for consistency against C4; Manning trapezoid with known depth; traverse to adjusted latitudes/departures; pyramid only when explicitly requested. Narrows implementation within D-7; MVP_SCOPE unchanged. | **Accepted** 2026-11-25 · [ADR 0009](decisions/0009-implementation-scope-0-1-calculations.md) |
| D-22 | P0 | Geometry conventions for 0.1 (tolerance, degeneracy, self-intersection, centroid scope, error mapping)? | [GEOMETRY.md](../docs/GEOMETRY.md) as revised after [GEOMETRY_SPEC_REVIEW.md](GEOMETRY_SPEC_REVIEW.md): relative tolerance with a precision floor, internal and returned; no new warning code; self-intersection refused (arcs checked exactly); straight-edged centroids only; computed area kept. Items settled during implementation confirmed 2026-11-25, with an overflow rule for derived tolerances. | **Accepted** 2026-11-25 · [ADR 0010](decisions/0010-geometry-conventions.md) |
| D-23 | P0 | List and choice inputs in the calculation contract (traverse, levelling, earthwork; the pyramid method input)? | Direction decided 2026-11-25: parallel lists with one unit per list (option A), required choice inputs recorded in the trace, API_DESIGN §8 wording clarified. Full design (groups, list-output lengths, item numbering, `index` in errors, `choice` trace steps) in [CONTRACT_LISTS_PROPOSAL.md](CONTRACT_LISTS_PROPOSAL.md) §3. | **Accepted** 2026-11-25 · [ADR 0011](decisions/0011-list-and-choice-inputs.md) |
| D-24 | P1 | When does a calculation carry a validity range? | Mathematical domain checks always apply. A validity range only with a reliable basis (a cited source stating it); otherwise the descriptor says so explicitly; never invent one. Adding a rule later bumps the version. | **Accepted** 2026-11-26 · [ADR 0012](decisions/0012-validity-basis-and-beam-scope.md) |
| D-25 | P1 | Beam maximum shear and moment in 0.1? | **Deferred from 0.1.** The beam ships with reactions, deflection and end slope; the search for an open worked example continues; they return as new outputs (a pure addition) in a later release. | **Accepted** 2026-11-26 · [ADR 0012](decisions/0012-validity-basis-and-beam-scope.md) |
| D-26 | P1 | Python package: versions, ABI, wheels, API layer? | CPython 3.11–3.15; PyO3 `=0.30.1` with `abi3-py311`; maturin 1.16.0; five abi3 wheels (manylinux2014 on Linux); thin Rust layer over `call_json`; `CivKitError`; no network calls or intentional writes (bytecode caches excluded); gates P1–P5. | **Accepted** 2026-11-26 · [ADR 0014](decisions/0014-python-package.md) |
| D-21 | P2 | Accept SI prefixes generally (`mN`, `µm`) and common Unicode notation (`m²`, `s⁻¹`) in unit strings? | Maintainer would eventually support both. **Not in 0.1**: [API_DESIGN.md §7.1](../docs/API_DESIGN.md) lists exactly what 0.1 accepts. Needs a grammar extension that stays unambiguous (e.g. `mm` vs milli-metre, `m` vs milli). | Proposed (after 0.1) |

## Decision record index

| ADR | Title | Decision |
|---|---|---|
| [0001](decisions/0001-monorepo-layout.md) | Monorepo with an interface-independent engine | D-15 |
| [0002](decisions/0002-neutral-drawing-model.md) | Neutral drawing model and DXF reader in the Rust engine | D-12 |
| [0003](decisions/0003-documentation-structure.md) | Documentation structure and AI provenance | D-16 |
| [0004](decisions/0004-name-licence-public-repo.md) | Name, licence and public repository | D-1, D-2, D-17 |
| [0005](decisions/0005-scope-order-units-ffi.md) | 0.1 scope, build order, units model and FFI contract | D-3, D-4, D-5, D-6 |
| [0006](decisions/0006-open-sources-and-0-1-calculations.md) | Openly licensed reference sources and the 0.1 calculation set | D-7, D-18 |
| [0007](decisions/0007-api-contract-details.md) | API contract details for 0.1 | D-19 |
| [0008](decisions/0008-rust-edition-and-sign-off.md) | Rust edition, MSRV and contribution sign-off | D-10, D-11 |
| [0009](decisions/0009-implementation-scope-0-1-calculations.md) | Implementation scope of the 0.1 calculations | D-20 |
| [0010](decisions/0010-geometry-conventions.md) | Geometry conventions for 0.1 | D-22 |
| [0011](decisions/0011-list-and-choice-inputs.md) | List and choice inputs in the calculation contract | D-23 |
| [0012](decisions/0012-validity-basis-and-beam-scope.md) | Validity ranges need a reliable basis; beam maximum shear and moment deferred from 0.1 | D-24, D-25 |
| [0013](decisions/0013-mcp-sdk-and-surface.md) | MCP SDK, pinned version and how the contract maps onto MCP | D-13 |
| [0014](decisions/0014-python-package.md) | Python package: versions, wheels and the binding layer | D-26 |

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
