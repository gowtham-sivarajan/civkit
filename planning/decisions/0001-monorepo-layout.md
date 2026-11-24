# ADR 0001 — Monorepo with an interface-independent engine

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-25 00:14 IST · last edited 2026-11-25 00:29 IST

- **Status:** Accepted
- **Date:** 2026-11-25
- **Decision ID:** D-15

## Context
CivKit has one computational engine and many interfaces (Python, MCP, AutoCAD, Civil 3D, MATLAB, Jupyter,
desktop app). Early on, the engine's API will change often, and every change has to be reflected in all
interfaces. The engine must also stay usable without any particular interface.

## Decision
Keep everything in one repository with this top-level structure:

```
crates/        the engine (Rust): civkit-core, civkit-units, civkit-geometry, civkit-drawing, civkit-ffi
bindings/      language bindings: python/ (PyO3 + maturin)
integrations/  mcp/, autocad/, civil3d/, matlab/, jupyter/
apps/          desktop/ (Tauri 2)
tests/         cross-cutting tests: reference worked examples, parity tests, DXF fixtures
docs/          stable design documents
planning/      plans, decisions, handover
```

Rules:
1. Nothing in `crates/` depends on anything outside `crates/`.
2. MCP is an interface, so it lives in `integrations/mcp`, not in `crates/`.
3. Non-Rust hosts (C#) reach the engine only through `crates/civkit-ffi`.
4. There is one Python package (`bindings/python`). `integrations/jupyter` holds only notebooks and optional
   widgets that depend on it; `integrations/matlab` wraps it.
5. `integrations/civil3d` references the shared .NET project in `integrations/autocad`; it never copies code.
6. Domain calculations start as modules inside `civkit-core` and become separate crates only when they grow.

## Consequences
- An engine change and its interface updates land in one commit, and CI tests them together.
- One version history and one tag per release across all interfaces.
- The dependency rule must be checked in review (and later in CI) so the engine stays interface-free.
- The repository will need Rust, Python, .NET and Node toolchains; contributors to one area only need that
  area's toolchain plus Rust.
- If an integration later needs its own release cadence, it can be split out; the dependency rule keeps that cheap.
