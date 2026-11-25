# CivKit

**Engineering tools that work where engineers work.**

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-24 23:41 IST · last edited 2026-11-25 16:11 IST
>
> **Status:** Phase 0 — Planning. No code is written until the Phase 0 exit criteria are met
> ([planning/ACCEPTANCE_CRITERIA.md](planning/ACCEPTANCE_CRITERIA.md)).

CivKit is a free, open-source, offline-first engineering toolkit. It brings reliable civil-engineering
calculations, unit handling and drawing analysis into the tools engineers already use — AutoCAD / Civil 3D,
MATLAB, Jupyter, AI assistants (via the Model Context Protocol) — plus a small standalone desktop app.

**One computational core. Many thin interfaces.** A calculation is written once, in Rust, and becomes
available in every interface with the same inputs, the same results and the same explanation.

---

## Principles

- **Free and open source** — no subscriptions, no mandatory paid APIs, no proprietary cloud dependency.
- **Local-first** — works offline; no accounts, no servers, no telemetry, no licence checks. See [docs/OFFLINE_POLICY.md](docs/OFFLINE_POLICY.md).
- **Checkable results** — every result carries its units, steps, references, assumptions and warnings. See [docs/ENGINEERING_STANDARDS.md](docs/ENGINEERING_STANDARDS.md).
- **Extensible** — documented APIs, independent modules, contributions welcome. See [CONTRIBUTING.md](CONTRIBUTING.md).

---

## Documentation map

| Where | File | What it answers |
|---|---|---|
| Root | `README.md` | What is this project and where is everything? |
| | [`ROADMAP.md`](ROADMAP.md) | What gets built, in what order? |
| | [`CONTRIBUTING.md`](CONTRIBUTING.md) | How do I set up, work with git, and contribute? |
| | [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md) | How we treat each other; how to report a problem privately. |
| | [`ACCESSIBILITY.md`](ACCESSIBILITY.md) | Accessibility commitments, known limits, reporting barriers. |
| `docs/` — *how the software works and the rules it follows (changes slowly)* | [`ARCHITECTURE.md`](docs/ARCHITECTURE.md) | How is the system structured? |
| | [`API_DESIGN.md`](docs/API_DESIGN.md) | Calculation contract, units format, FFI, Python and MCP surfaces. |
| | [`GEOMETRY.md`](docs/GEOMETRY.md) | Geometry conventions: axes, angles, tolerances, degeneracy, length, area (proposed). |
| | [`ENGINEERING_STANDARDS.md`](docs/ENGINEERING_STANDARDS.md) | What must every calculation meet to be trusted? |
| | [`OFFLINE_POLICY.md`](docs/OFFLINE_POLICY.md) | What may and may not touch the network? |
| | [`SECURITY.md`](docs/SECURITY.md) | Threat model, supply chain, reporting a vulnerability. |
| `planning/` — *how we get there (changes often)* | [`MVP_SCOPE.md`](planning/MVP_SCOPE.md) | What is in and out of 0.1? |
| | [`IMPLEMENTATION_PLAN.md`](planning/IMPLEMENTATION_PLAN.md) | Tasks per phase, dependencies, risks. |
| | [`ACCEPTANCE_CRITERIA.md`](planning/ACCEPTANCE_CRITERIA.md) | When is a phase, release or calculation "done"? |
| | [`DECISIONS.md`](planning/DECISIONS.md) | Every decision and its status; records in `planning/decisions/`. |
| | [`REFERENCE_SOURCES.md`](planning/REFERENCE_SOURCES.md) | Openly licensed sources for reference cases. |
| | [`REFERENCE_CASES.md`](planning/REFERENCE_CASES.md) | Collected worked examples, re-derivations, source errata, 0.1 coverage. |
| | [`AI_HANDOVER.md`](planning/AI_HANDOVER.md) | Current state and next actions — **read first when resuming work** (human or AI). |
| | [`ORIGINAL_PROPOSAL.md`](planning/ORIGINAL_PROPOSAL.md) | The original concept proposal, unchanged. |
| `tests/` | [`fixtures/dxf/README.md`](tests/fixtures/dxf/README.md) | Sample drawings and their exact expected values. |

**New to the project?** README → ARCHITECTURE → ROADMAP.
**Resuming work?** Start with [planning/AI_HANDOVER.md](planning/AI_HANDOVER.md).

---

## Repository layout

```
CivKit/
├── README.md  ROADMAP.md  CONTRIBUTING.md  CODE_OF_CONDUCT.md  ACCESSIBILITY.md
├── .github/         CI workflows, Dependabot, issue forms, pull request template
├── docs/            stable design documents
├── planning/        plans, decisions, handover (decision records in planning/decisions/)
├── Cargo.toml       Rust workspace (Phase 1)
├── crates/          the engine — never depends on anything outside crates/
├── tests/           reference cases (tests/reference/), DXF fixtures (tests/fixtures/dxf/)
├── tools/           repository scripts (dependency-rule check)
│
│   created in later phases:
├── bindings/        python/
├── integrations/    mcp/  autocad/  civil3d/  matlab/  jupyter/
├── apps/            desktop/
└── (parity tests join tests/ in Phase 3)
```

Details and the dependency rule: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

---

## Licence

Copyright 2026 Gowtham and CivKit contributors.
Licensed under the [Apache License, Version 2.0](LICENSE) (D-2).

## Disclaimer

CivKit results support, and do not replace, the judgement of a qualified engineer.
