# Architecture

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-24 23:41 IST (as part of README.md) · last edited 2026-11-27 19:02 IST by Claude Sonnet 5.5 (`claude-sonnet-5-5`)

> **Status:** Design — accepted parts are marked with their decision ID; the rest is proposed.
> Contract and API details: [API_DESIGN.md](API_DESIGN.md).

## 1. Overview

**One computational core. Many thin interfaces.**

```
 AutoCAD / Civil 3D     MATLAB            Jupyter / Python     AI clients        Desktop app
 (C# plugin, .NET 10)   (calls Python)    (PyO3 wheel)         (MCP, stdio)      (Tauri 2)
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

## 2. Repository layout — *accepted (D-15, ADR 0001)*

```
CivKit/
├── Cargo.toml                 # Rust workspace
├── crates/                    # the engine
│   ├── civkit-core/           # calculation contract, registry, trace, errors (+ domain modules at first)
│   ├── civkit-units/          # Quantity, dimensions, unit registry, parsing
│   ├── civkit-geometry/       # 2D geometry primitives and algorithms (GEOMETRY.md)
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
├── tests/                     # cross-cutting: reference worked examples, parity tests, DXF fixtures
├── tools/                     # repository scripts (e.g. the dependency-rule check)
├── docs/                      # stable design documents
└── planning/                  # plans, decisions, handover
```

### The dependency rule
**Nothing in `crates/` depends on anything outside `crates/`.** Interfaces depend on the engine, never the
reverse. In particular, the engine has:

- **no AutoCAD / .NET dependency** — AutoCAD is an integration target, not a dependency;
- **no Python dependency** — PyO3 lives only in `bindings/python`;
- **no network dependency** — see [OFFLINE_POLICY.md](OFFLINE_POLICY.md);
- **no AI dependency** — no model, AI service or MCP code; MCP lives only in `integrations/mcp`.

Checked in review and in CI:
- `tools/check_dependency_rule.py`: no path dependency leaves `crates/`, and no engine crate reaches a Python, MCP, AI,
  network, async-runtime or GUI crate, even transitively.
- `cargo-deny` (`deny.toml`): network clients banned everywhere; PyO3, `rmcp` and Tauri allowed only in their own interface package.

Further rules:
- Domain calculations (geotech, structural, surveying, fluids, quantities, math) start as modules inside
  `civkit-core` and are split into their own crates only when they grow.
- Rust unit tests live inside each crate; root `tests/` holds only what spans the whole project.
- `civil3d/` never copies code from `autocad/`; shared .NET code lives in the autocad shared project.
- There is exactly one Python package (`bindings/python`).

## 3. Components

### 3.1 `civkit-core` — the calculation registry
Every calculation **describes itself** to a registry: ID, version, inputs (with physical dimension),
outputs, references, assumptions and validity ranges. From that one description we generate the MCP tool
definitions, the Python functions and docstrings, the desktop app's forms, the FFI entry points and the
documentation pages. A contributor writes a calculation **once** and it appears in every interface.
Specification: [API_DESIGN.md](API_DESIGN.md).

### 3.2 `civkit-units`
Runtime dimensional analysis: values stored in SI with dimension exponents; angle as its own dimension;
ambiguous units rejected. Runtime (not only compile-time) checking is required because inputs arrive as JSON
from Python, MCP and C#. Details: [API_DESIGN.md §7](API_DESIGN.md#7-units).

### 3.3 `civkit-geometry` — *accepted (D-22, ADR 0010)*
2D primitives and algorithms used by calculations and drawing analysis: distance and azimuth, polyline length
(straight and bulged segments), arc length, polygon area and orientation, and centroid of straight-edged polygons
([GEOMETRY.md](GEOMETRY.md)). Phase 4 (D-28) adds 2D affine transformations and block placement (`Affine2`) and the
area of regions combined by even-odd nesting (`region_area`), as GEOMETRY.md §11 specifies. 3D is not in 0.1.

### 3.4 `civkit-drawing` — *accepted (D-12, ADR 0002)*
A CAD-neutral drawing model (lines, polylines, arcs, circles, blocks, layers, dimensions, drawing units) plus a
DXF reader. All drawing analysis — quantity extraction, unit-consistency checks, schedules — is written against
this model (the drawing report of [API_DESIGN.md §13](API_DESIGN.md), D-28), so it works from every interface, is
testable in CI with DXF fixtures, and helps users without
AutoCAD. The AutoCAD/Civil 3D integrations translate their entities into this model.

### 3.5 `civkit-ffi`
A small, stable C ABI for non-Rust hosts: essentially `call(id, json) → json`, `list() → json`, `free(ptr)`.
JSON keeps the ABI tiny while calculations grow. Typed fast paths only if profiling shows a need.

## 4. Integrations

| Target | Approach | Main risk | Notes |
|---|---|---|---|
| **Python / Jupyter** | PyO3 + maturin wheel (`bindings/python`); notebooks in `integrations/jupyter` | Low | Cheapest interface; doubles as the test harness engineers will use. |
| **MCP** | Official Rust SDK (`rmcp`), stdio transport only (`integrations/mcp`) | Spec churn | Tools generated from the registry. All 0.1 tools read-only. No network listener. |
| **AutoCAD** | C# .NET plugin → P/Invoke into `civkit-ffi` | Highest | AutoCAD 2027 uses .NET 10 (AutoCAD 2025/2026 use .NET 8). Start with 2027 only (D-9). Keep the plugin dependency-light to avoid DLL conflicts with other plugins. |
| **Civil 3D** | Extends the AutoCAD shared project | High | Alignments, surfaces, corridors need neutral-model extensions later. |
| **MATLAB** | MATLAB's Python interface calling the CivKit Python package | Medium | Zero extra native code; `loadlibrary` on the C ABI only as fallback. |
| **Desktop app** | Tauri 2 + Rust backend; forms generated from the registry | Medium | SQLite arrives here (history/saved calculations), not before (D-14). |

## 5. Drawing modifications

Out of scope for 0.1 (S-5). When added, every write requires explicit user confirmation in the host, and MCP
tools that write are clearly marked as non-read-only. Security implications: [SECURITY.md](SECURITY.md).

## 6. Proprietary hosts

AutoCAD, Civil 3D and MATLAB are paid products. CivKit stays free, but those integrations can only be built
and tested by someone with access to the host (education or campus licences usually cover this). The engine,
Python, MCP and desktop paths never depend on them.
