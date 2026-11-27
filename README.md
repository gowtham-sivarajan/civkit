# CivKit

**Engineering tools that work where engineers work.**

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-24 23:41 IST · last edited 2026-11-27 23:49 IST by Claude Sonnet 5.5 (`claude-sonnet-5-5`)
>
> **Status:** Phase 3 (Python and MCP) is complete: tag `phase-3-done` on `a94bf3f`, with CI and the Python workflow
> green. The engine has seven registered calculations in five families, each tested against published reference
> cases. They can be used from Rust, from the `civkit` Python package and from the `civkit-mcp` server for AI
> assistants, with identical results (both are built from source for now; nothing is on PyPI or crates.io yet).
> **Current: Phase 4** (drawing model and DXF), which completes release 0.1:
> - CivKit's own DXF reader is in `main` (D-27, [ADR 0015](planning/decisions/0015-dxf-reader.md)).
> - The drawing analysis is decided (D-28, [ADR 0016](planning/decisions/0016-drawing-analysis.md)) and is being
>   built, tests first. Its geometry (transformations, areas of regions with holes, a bounded intersection check)
>   bounded block and MINSERT expansion, quantities by layer, drawing units, the unit check, and the JSON report and
>   CSV schedule are written, in Rust, Python (`civkit.drawing`) and MCP (`drawing.analyze`, behind `--drawing-root`).
> - Every analysis gate (ADR 0016 A1–A9) passes; A5 and A6 are closed with their own end-to-end tests.
> - Fuzzing stands at 2 h 10 min of the 8 hours required before 0.1.
>
> There is no release yet. Exit criteria and current status: [planning/ACCEPTANCE_CRITERIA.md](planning/ACCEPTANCE_CRITERIA.md).

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
| | [`API_DESIGN.md`](docs/API_DESIGN.md) | Calculation contract, units format, the drawing report, FFI, Python and MCP surfaces. |
| | [`GEOMETRY.md`](docs/GEOMETRY.md) | Geometry conventions: axes, angles, tolerances, degeneracy, length, area, transformations, regions (accepted: D-22, D-28). |
| | [`ENGINEERING_STANDARDS.md`](docs/ENGINEERING_STANDARDS.md) | What must every calculation meet to be trusted? |
| | [`OFFLINE_POLICY.md`](docs/OFFLINE_POLICY.md) | What may and may not touch the network? |
| | [`SECURITY.md`](docs/SECURITY.md) | Threat model, supply chain, reporting a vulnerability. |
| | [`calculations/`](docs/calculations/README.md) | One generated page per calculation: inputs, method, limits, references, worked example. |
| `integrations/` — *interfaces* | [`mcp/`](integrations/mcp/README.md) | The MCP server: tools, protocol versions, client set-up. |
| | [`jupyter/`](integrations/jupyter/README.md) | Example notebooks for the Python package. |
| `planning/` — *how we get there (changes often)* | [`MVP_SCOPE.md`](planning/MVP_SCOPE.md) | What is in and out of 0.1? |
| | [`IMPLEMENTATION_PLAN.md`](planning/IMPLEMENTATION_PLAN.md) | Tasks per phase, dependencies, risks. |
| | [`ACCEPTANCE_CRITERIA.md`](planning/ACCEPTANCE_CRITERIA.md) | When is a phase, release or calculation "done"? |
| | [`DECISIONS.md`](planning/DECISIONS.md) | Every decision and its status; records in `planning/decisions/`. |
| | [`REFERENCE_SOURCES.md`](planning/REFERENCE_SOURCES.md) | Openly licensed sources for reference cases. |
| | [`REFERENCE_CASES.md`](planning/REFERENCE_CASES.md) | Collected worked examples, re-derivations, source errata, 0.1 coverage. |
| | [`PHASE_2D_REVIEW.md`](planning/PHASE_2D_REVIEW.md) | Phase 2 integration and regression review: checks, findings, decisions needed (2026-11-26). |
| | [`CODE_REVIEW_D6.md`](planning/CODE_REVIEW_D6.md) | Guide and record for the first human code review (`registry.rs`, `levelling.rs`); open. |
| | [`MCP_SDK_PROPOSAL.md`](planning/MCP_SDK_PROPOSAL.md) | D-13: the MCP SDK, its pinned version and how the contract maps onto MCP (accepted, ADR 0013). |
| | [`PYTHON_BINDINGS_PROPOSAL.md`](planning/PYTHON_BINDINGS_PROPOSAL.md) | D-26: Python versions, wheels and the thin binding layer (accepted, ADR 0014). |
| | [`DXF_READER_PROPOSAL.md`](planning/DXF_READER_PROPOSAL.md) | D-27: CivKit's own DXF reader, its limits and heap budget (accepted, ADR 0015). |
| | [`DRAWING_ANALYSIS_PROPOSAL.md`](planning/DRAWING_ANALYSIS_PROPOSAL.md) | D-28: drawing analysis: expansion, transformations, quantities, unit check, schedules, warnings (accepted, ADR 0016). |
| | [`EARTHWORK_DESCRIPTOR_PROPOSAL.md`](planning/EARTHWORK_DESCRIPTOR_PROPOSAL.md) | The earthwork calculation's descriptor (accepted; implemented in Phase 2, D-20). |
| | [`LEVELLING_DESCRIPTOR_PROPOSAL.md`](planning/LEVELLING_DESCRIPTOR_PROPOSAL.md) | The levelling calculations' descriptors (accepted; implemented in Phase 2, D-20). |
| | [`TRAVERSE_DESCRIPTOR_PROPOSAL.md`](planning/TRAVERSE_DESCRIPTOR_PROPOSAL.md) | The traverse calculation's descriptor (accepted; implemented in Phase 2, D-20). |
| | [`COMPETITIVE_LANDSCAPE.md`](planning/COMPETITIVE_LANDSCAPE.md) | Who else is building this, licences, gaps, naming risks (research, 2026-11-25). |
| | [`FEATURE_PROPOSALS.md`](planning/FEATURE_PROPOSALS.md) | Feature ideas from that research, scored; none accepted. |
| | [`ORIGINAL_PROPOSAL.md`](planning/ORIGINAL_PROPOSAL.md) | The original concept proposal, unchanged. |
| `tests/` | [`fixtures/dxf/README.md`](tests/fixtures/dxf/README.md) | Sample drawings and their exact expected values. |

**New to the project?** README → ARCHITECTURE → ROADMAP.
**Resuming work?** Start with [planning/IMPLEMENTATION_PLAN.md](planning/IMPLEMENTATION_PLAN.md) and
[planning/ACCEPTANCE_CRITERIA.md](planning/ACCEPTANCE_CRITERIA.md).

---

## Repository layout

```
CivKit/
├── README.md  ROADMAP.md  CONTRIBUTING.md  CODE_OF_CONDUCT.md  ACCESSIBILITY.md
├── .github/         CI workflows, Dependabot, issue forms, pull request template
├── docs/            stable design documents
├── planning/        plans, decisions, handover (decision records in planning/decisions/)
├── Cargo.toml       Rust workspace
├── crates/          the engine — never depends on anything outside crates/
├── bindings/        python/ — the `civkit` Python package (PyO3; its own Cargo workspace)
├── integrations/    mcp/ — the MCP server   jupyter/ — example notebooks
├── tests/           reference cases (tests/reference/), Rust/Python/MCP parity (tests/parity/),
│                    DXF fixtures (tests/fixtures/dxf/)
├── tools/           repository scripts: dependency rule, calculation-page generator
│
│   created in later phases:
├── integrations/    autocad/  civil3d/  matlab/
└── apps/            desktop/
```

Details and the dependency rule: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

---

## Licence

Copyright 2026 Gowtham and CivKit contributors.
Licensed under the [Apache License, Version 2.0](LICENSE) (D-2).

## Disclaimer

CivKit results support, and do not replace, the judgement of a qualified engineer.
