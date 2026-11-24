# CivKit — Core Plan

> **Working name.** "CivKit" is tentative (see decision D-1 in [HANDOFF.md](HANDOFF.md)).
> **Status:** Phase 0 — Planning. **No code is written until the Phase 0 exit criteria below are met.**
> **Last updated:** 2026-11-25

An open-source, offline-first engineering toolkit that brings reliable civil-engineering calculations
into the tools engineers already use: AutoCAD / Civil 3D, MATLAB, Jupyter, AI assistants (via MCP),
and a small standalone desktop app.

**One computational core. Many thin interfaces.**

---

## How this repository is organised (during planning)

| File | Purpose |
|---|---|
| `README.md` | This file — the plan: what we are building, how, and in what order. Changes slowly. |
| `HANDOFF.md` | The working state — what is decided, what is open, what to do next. Updated every session. |
| `docs/00-original-proposal.md` | The original concept proposal, kept unchanged for reference. |
| `docs/decisions/` *(to be created)* | One short Architecture Decision Record (ADR) per accepted decision. |
| `docs/spec/` *(to be created)* | Specifications, starting with the calculation contract. |

Where this plan differs from the original proposal, it is marked **[Change from proposal]** and
listed as an open decision in `HANDOFF.md` until confirmed.

---

## 1. Goals and non-goals

### Goals
1. A single, well-tested Rust core of engineering calculations, units and geometry.
2. Every calculation usable identically from Python, MCP, AutoCAD, MATLAB and the desktop app.
3. Results that an engineer can **check**: inputs, units, formula steps, references, assumptions, warnings.
4. Fully local operation — no accounts, no servers, no telemetry, no licence checks.
5. A contribution path where an engineer can add or verify a calculation without learning the whole codebase.

### Non-goals (for now)
- Replacing AutoCAD, Civil 3D, STAAD, ETABS, PLAXIS or similar. CivKit augments them.
- Full design-code compliance checking (IS 456, Eurocode, ACI …) in early releases. See §6.
- Cloud sync, collaboration features or hosted services — ever, as a requirement.
- Writing to drawings in 0.1. Drawing analysis is read-only first.

---

## 2. Principles

### From the proposal (non-negotiable)
- **Free and open source** — no paid subscriptions, no mandatory paid APIs, no proprietary cloud dependency.
- **Local-first** — core works offline; no accounts, no remote database, no hosted backend.
- **User-controlled** — data stays on the machine; no mandatory telemetry; no online licence verification.
- **Extensible** — documented APIs, modular integrations, new domains addable independently.

### Added: engineering-trust principles
- **No silent results.** Every result carries its units, the steps used, the reference it came from, and any warnings.
- **No silent changes.** A calculation's formula never changes under the same ID/version. Changes bump the version.
- **Fail loudly on units.** Dimensionally inconsistent input is an error, never a guess.
- **Validity ranges are part of the calculation.** Inputs outside a method's stated range produce a warning (or error).
- **AI calls, it does not compute.** Through MCP, an AI client gets the numbers and the explanation from CivKit, rather than doing the arithmetic itself.

### A note on the proprietary hosts
AutoCAD, Civil 3D and MATLAB are paid products. CivKit stays free, but those **integrations** can only be built
and tested by someone with access to the host (student/education licences or a university campus licence
usually cover this). The core, Python, MCP and desktop paths must never depend on them.

---

## 3. Architecture

```
 AutoCAD / Civil 3D     MATLAB            Jupyter / Python     AI clients        Desktop app
 (C# plugin, .NET)      (calls Python)    (PyO3 wheel)         (MCP, stdio)      (Tauri 2)
        │                    │                  │                   │                 │
   C ABI (P/Invoke)          └───────►──────────┤                   │                 │
        │                                       │                   │                 │
        ▼                                       ▼                   ▼                 ▼
 ┌───────────────────────────────────────────────────────────────────────────────────────┐
 │ civkit-core  calculation registry · JSON contract · versioned calc IDs · errors/trace  │
 ├───────────────────────────────────────────────────────────────────────────────────────┤
 │ civkit-units │ civkit-geometry │ civkit-drawing (neutral model + DXF) │ civkit-ffi     │
 └───────────────────────────────────────────────────────────────────────────────────────┘
```

### 3.1 The calculation contract (the central idea)
Every calculation **describes itself** to a registry: ID, version, inputs (with physical dimension),
outputs, references, assumptions and validity ranges. From that one description we generate:

- the MCP tool definition (name, description, JSON Schema),
- the Python function and its docstring,
- the desktop app's input form,
- the C ABI entry point used by AutoCAD,
- the documentation page.

So a contributor writes a calculation **once** and it appears in every interface. This is what makes
"one core, many interfaces" affordable for a small team.

Illustrative request/response (exact schema to be specified in `docs/spec/calculation-contract.md`):

```json
// request
{
  "id": "fluids.open_channel.manning",
  "version": 1,
  "inputs": {
    "n":                { "value": 0.013 },
    "area":             { "value": 2.4,   "unit": "m^2" },
    "wetted_perimeter": { "value": 4.2,   "unit": "m" },
    "bed_slope":        { "value": 0.001 }
  }
}

// response
{
  "id": "fluids.open_channel.manning",
  "version": 1,
  "outputs": {
    "velocity":  { "value": 1.675, "unit": "m/s" },
    "discharge": { "value": 4.020, "unit": "m^3/s" }
  },
  "trace": [
    { "step": "R = A / P",                   "value": 0.5714, "unit": "m"   },
    { "step": "V = (1/n) · R^(2/3) · S^(1/2)", "value": 1.675,  "unit": "m/s" },
    { "step": "Q = V · A",                   "value": 4.020,  "unit": "m^3/s" }
  ],
  "warnings": [],
  "references": ["<textbook or code clause>"],
  "assumptions": ["Uniform, steady flow", "SI form of Manning's equation"]
}
```

Calculation IDs are namespaced (`domain.topic.method`) and versioned. Old versions remain callable
so saved notebooks and drawings stay reproducible.

### 3.2 Units
- A runtime `Quantity` type: value stored in SI + dimension exponents (length, mass, time, temperature, …).
  Runtime (not only compile-time) checking is required because inputs arrive as JSON from Python, MCP and C#.
- A unit registry tuned for civil work: mm, m, km, ft, in, kN, MPa, kPa, kN/m², kN/m³, t (tonne), kip, psi,
  ha, acre, m³/s, L/s, etc.
- **Angle is treated as its own dimension** (rad, deg, grad, DMS) so surveying errors like adding a bearing
  to a length are caught.
- **Ambiguous units are rejected**, not guessed (e.g. "ton" — short, long or metric?).
- Temperature (affine °C/°F) is excluded from 0.1 unless a calculation needs it.

### 3.3 Drawing model — *accepted (D-12, ADR 0002)*
The proposal puts drawing analysis inside the AutoCAD plugin. This plan instead adds a
**neutral drawing model in Rust** (lines, polylines, arcs, circles, blocks, layers, units, dimensions)
with a **DXF reader**. The AutoCAD plugin then becomes a thin translator from AutoCAD entities into that model.

Why:
- Quantity extraction, unit-inconsistency checks and schedules are written **once** and work from AutoCAD,
  Python, MCP and the desktop app.
- It can be **tested in CI** using DXF fixture files — AutoCAD itself cannot run in CI.
- Users without AutoCAD (or with other CAD tools that export DXF) still benefit.

### 3.4 The FFI boundary
- `civkit-ffi` exposes a small, stable C ABI: essentially `call(id, json) -> json`, `list() -> json`, `free(ptr)`.
- JSON across the boundary keeps the ABI tiny and stable while calculations grow. Engineering calculations
  are not performance-bound; geometry-heavy paths can get typed entry points later if profiling shows a need.
- Python uses PyO3 directly (no C ABI) and also gets typed wrappers generated from the registry.

### 3.5 Repository layout — *accepted (D-15, ADR 0001)*

One monorepo. **The rule: nothing in `crates/` depends on anything outside `crates/`.**
Interfaces depend on the engine, never the other way round.

```
CivKit/
├── Cargo.toml                 # Rust workspace
├── crates/                    # the engine
│   ├── civkit-core/           # calculation contract, registry, trace, errors (+ domain modules at first)
│   ├── civkit-units/          # Quantity, dimensions, unit registry, parsing
│   ├── civkit-geometry/       # 2D/3D geometry primitives and algorithms
│   ├── civkit-drawing/        # neutral drawing model + DXF import
│   └── civkit-ffi/            # C ABI for C# (AutoCAD / Civil 3D) and MATLAB fallback
├── bindings/
│   └── python/                # PyO3 + maturin: the one Python package (Jupyter and MATLAB use it)
├── integrations/
│   ├── mcp/                   # MCP server binary (stdio)
│   ├── autocad/               # C# .NET plugin + shared .NET code
│   ├── civil3d/               # references autocad/ shared project, adds Civil 3D APIs
│   ├── matlab/                # thin wrapper over the Python package
│   └── jupyter/               # example notebooks, optional widgets (depend on bindings/python)
├── apps/
│   └── desktop/               # Tauri 2
├── tests/                     # cross-cutting: reference worked examples (data), parity tests, DXF fixtures
└── docs/
```

- Domain calculations (geotech, structural, surveying, fluids, quantities, math) start as modules inside
  `civkit-core` and are split into their own crates only when they grow.
- Rust unit tests live inside each crate; the root `tests/` folder holds only what spans the whole project.
- `civil3d/` never copies code from `autocad/`; shared .NET code lives in the autocad shared project.

---

## 4. Integrations

| Target | Approach | Main risk | Notes |
|---|---|---|---|
| **Python / Jupyter** | PyO3 + maturin wheel | Low | Cheapest interface; doubles as the test harness engineers will actually use. |
| **MCP** | Official Rust SDK (`rmcp`), stdio transport only | Spec churn | Tools generated from the registry. All 0.1 tools read-only. No network listener (local-first). |
| **AutoCAD / Civil 3D** | C# .NET plugin → P/Invoke into `civkit-ffi` | Highest | AutoCAD 2025/2026 use .NET 8; AutoCAD 2027 moves to .NET 10, so the plugin must multi-target. Keep it dependency-light to avoid DLL conflicts with other plugins. Civil 3D adds its own API on top (alignments, surfaces, corridors) — later phase. |
| **MATLAB** | Call the CivKit Python package from MATLAB's Python interface | Medium | Zero extra native code. Fall back to `loadlibrary` on the C ABI only if needed. |
| **Desktop app** | Tauri 2 + Rust backend; forms generated from the registry | Medium | SQLite only arrives here (saved calculations / history). Not needed before Phase 6. |

**Drawing modifications** (any interface): out of scope for 0.1. When added, every write requires explicit
user confirmation in the host, and MCP tools that write must be clearly marked as non-read-only.

---

## 5. Scope of 0.1 (MVP)

### Proposed 0.1 — **[Change from proposal]**
The proposal puts core + AutoCAD + MCP + Python all in 0.1. AutoCAD is the riskiest piece and depends on
a proprietary host, so this plan moves the **AutoCAD plugin to 0.2** and replaces it in 0.1 with
**DXF-based drawing analysis** that the plugin will later reuse.

**0.1 includes**
- Units: conversion, parsing, dimensional validation.
- Geometry: polyline length, polygon area and centroid, arc length, bearing/distance between points.
- Contract: registry, JSON request/response, trace, warnings, versioned IDs.
- A small set of **first-principles** calculations, each verifiable against a textbook worked example. Candidates:
  - Surveying — traverse closure error and Bowditch adjustment; levelling reduction (rise & fall / HI).
  - Geotechnical — soil phase relationships (void ratio, porosity, degree of saturation, unit weights).
  - Structural — simply supported beam: reactions, max shear/moment, deflection (UDL and point load).
  - Fluids — Manning's equation for open channels.
  - Quantities — earthwork volume by average-end-area and prismoidal methods.
- Drawing (from DXF, read-only): entity listing by layer, lengths/areas, drawing-unit consistency check,
  quantity schedule export (CSV/JSON).
- Python package exposing all of the above.
- MCP server exposing all of the above.

**0.1 explicitly excludes**: AutoCAD/Civil 3D plugin, MATLAB, desktop app, SQLite, design-code checks,
any drawing writes, temperature units.

### 0.1 definition of done
- Every calculation has ≥1 reference test from a cited published worked example, plus edge-case tests.
- The same input gives byte-identical JSON output via Rust, Python and MCP (parity test).
- `pip install` and the MCP server work offline on Windows, macOS and Linux.
- Every calculation has a generated doc page: formula, references, assumptions, validity range.

---

## 6. Engineering correctness and trust

Each calculation must ship with:
1. **References** — textbook (title, edition, page/section) or code clause number. Cited, not copied.
2. **Assumptions** and **validity limits**, enforced as warnings or errors.
3. **Reference tests** — worked examples stored as data in `tests/reference/` with the source and a stated tolerance.
4. **Review** — at least one reviewer with engineering background in addition to code review.

Design codes (IS, Eurocode, ACI, AASHTO …) come **after 0.1**, one code at a time. Code texts and some
tables are copyrighted: implement equations and cite clauses; do not paste code text into the repository.

Every interface shows the same disclaimer: *results support, and do not replace, the judgement of a
qualified engineer.*

---

## 7. Roadmap

Phases from the proposal, with exit criteria added. **[Change from proposal]**: Phase 0 added; Python moved
up next to MCP (both are thin layers over the registry, and Python is the cheapest way to get engineers testing).

| Phase | Objective | Exit criteria |
|---|---|---|
| **0** | Planning *(current)* | P0 decisions accepted (HANDOFF.md); calculation contract spec written; unit list agreed; reference worked examples collected for every 0.1 calculation. |
| **1** | Architecture and Rust core | Workspace builds in CI on Win/macOS/Linux; `civkit-core` registry and contract implemented with one dummy calculation end-to-end. |
| **2** | Units, geometry, calculations, validation | All 0.1 calculations pass reference tests; property tests for units and geometry. |
| **3** | Python bindings + local MCP server | Parity test passes; usable from Jupyter and an MCP client fully offline. |
| **4** | Drawing model + DXF analysis | Quantity extraction and unit checks pass on DXF fixtures. → **Release 0.1** |
| **5** | AutoCAD / Civil 3D plugin | Plugin loads in supported AutoCAD versions, reuses drawing analysis, read-only. → **Release 0.2** |
| **6** | Standalone desktop app (+ SQLite history) | Generated forms for every calculation; works offline. |
| **7** | Expanded modules, design codes, MATLAB, community | Contribution guide proven by at least one external contribution. |

Each phase ends with a git tag (e.g. `phase-1-done`) so the project can always be rolled back to a known-good point.

---

## 8. Quality and testing

- **Unit tests** for every function; **reference tests** from published worked examples.
- **Property tests** (`proptest`): unit round-trips, area invariant under translation/rotation, etc.
- **Parity tests**: same request through Rust, FFI, Python and MCP → identical output.
- **Fixture tests** on DXF files for drawing analysis.
- **CI** on GitHub Actions (free for public repositories) across Windows, macOS and Linux.
- The AutoCAD plugin cannot run in CI: its C# logic is kept thin and tested separately; a manual test
  checklist covers in-host behaviour.
- Formatting and lints enforced: `rustfmt`, `clippy`, `ruff`, `dotnet format`.

---

## 9. Contributing model (planned)

- A **calculation template**: one Rust file + one reference-test data file + one doc stub.
- Engineers who do not write Rust can still contribute **reference cases** (worked examples as YAML/TOML)
  and **reviews** — often the most valuable contribution.
- Each domain module is independent; contributors do not need to understand the whole codebase.
- Developer Certificate of Origin (sign-off) rather than a CLA — to be confirmed (D-11).

---

## 10. Licensing and naming

- **Licence** must be chosen **before the first outside contribution**, not just before the first release —
  relicensing later needs every contributor's consent. Options and recommendation in HANDOFF.md (D-2).
- **Name**: before anything is published, check GitHub, crates.io, PyPI and a basic trademark search.
  Also note discoverability: a web search for "CivKit project" currently returns mostly *Civilization*-game
  projects, so the name may compete for attention with games. Choosing early avoids renaming published packages.

---

## 11. Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Scope too broad for a small team | Nothing ships | Narrow 0.1 (§5); AutoCAD to 0.2; phase exit criteria. |
| Wrong engineering result | Loss of trust; potential harm | §6 process; versioned calculations; warnings; disclaimer. |
| AutoCAD .NET runtime churn (.NET 8 → .NET 10) | Plugin breaks each release | Thin plugin; logic in Rust; multi-targeting; per-version test checklist. |
| MCP specification changes | Adapter rework | MCP isolated in `integrations/mcp`; pin SDK version per release. |
| Rust is a barrier for engineering contributors | Few contributors | Templates, macros, data-driven reference tests, Python-side contributions. |
| Single maintainer burnout | Project stalls | Small releases; clear docs; HANDOFF.md always current. |
| Copyright in design-code material | Legal exposure | Cite clauses, implement equations, never paste code text. |

---

## 12. Toolchain (all free)

Git · Rust (rustup, stable, 2024 edition) · Python 3 + maturin · .NET SDK (8 and 10, for the plugin later) ·
Node.js (for Tauri later) · VS Code or any editor. AutoCAD/MATLAB only for their respective integrations.

---

*Status: proposed project. Architecture, scope and implementation details are subject to validation during development.*
