# Acceptance Criteria

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-24 23:41 IST (as part of README.md) · last edited 2026-11-25 11:32 IST

> **Status:** Draft. The single source of truth for "done". [ROADMAP.md](../ROADMAP.md) and
> [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md) point here.

## 1. Phase exit criteria

### Phase 0 — Planning
- [x] All **P0** decisions in [DECISIONS.md](DECISIONS.md) are Accepted (or explicitly deferred with a reason).
      *All P0 accepted as of D-19, 2026-11-25; D-8 – D-11, D-13, D-14 are P1.*
- [x] [API_DESIGN.md](../docs/API_DESIGN.md) has no remaining **TBD** for the contract (§3–§6) and units (§7).
      *Resolved 2026-11-25 (D-19, ADR 0007).*
- [x] The 0.1 calculation set is fixed (D-7), and **each calculation has ≥1 published worked example** collected
      (source, edition, page, inputs, expected outputs). *[REFERENCE_CASES.md](REFERENCE_CASES.md) C1–C9; prismoidal (general)
      and soil phase relationships deferred; human engineering review deferred until the product is complete.*
- [x] 3–5 licence-safe sample DXF drawings collected for fixtures. *[tests/fixtures/dxf](../tests/fixtures/dxf/README.md): F1–F5 (Apache-2.0) + 2 ezdxf samples (MIT); expected values independently checked.*
- [x] Toolchain installed and verified (Rust builds and links a test program; Python `.venv` works) — 2026-11-25.
- [x] Licence chosen and `LICENSE` added (Apache-2.0, 2026-11-25).
- [x] Public GitHub repository created and pushed; private vulnerability reporting enabled (2026-11-25).
- [x] Tag `plan-v1.0`. *2026-11-25 — Phase 0 complete.*

### Phase 1 — Architecture and Rust core
- [ ] Cargo workspace builds in CI on Windows, macOS and Linux.
- [x] `civkit-core` registry and contract implemented; one dummy calculation runs end to end (request → response JSON).
      *2026-11-25: `math.demo.weighted_mean` (template), 23 tests incl. reference-case harness; local run with Rust 1.97.*
- [ ] Dependency rule checked in CI; `rustfmt` and `clippy` clean.
- [ ] Tag `phase-1-done`.

### Phase 2 — Units, geometry, calculations
- [ ] Every 0.1 calculation meets §3 below.
- [ ] Property tests for units and geometry pass.
- [ ] Tag `phase-2-done`.

### Phase 3 — Python + MCP
- [ ] `pip install` of the built wheel works offline on Windows, macOS, Linux.
- [ ] MCP server works with MCP Inspector and at least one MCP client, fully offline, stdio only.
- [ ] Parity test: identical JSON output via Rust, Python and MCP for every reference case.
- [ ] Tag `phase-3-done`.

### Phase 4 — Drawing model + DXF → **Release 0.1**
- [ ] Quantity extraction, unit checks and schedule export pass on all DXF fixtures.
- [ ] DXF parser fuzzed for a defined minimum duration with no crashes.
- [ ] Everything in §2 holds. Tag `v0.1.0`.

### Phase 5 — AutoCAD / Civil 3D → **Release 0.2**
- [ ] Plugin loads in AutoCAD 2027, runs read-only analysis on the open drawing, matches DXF results for the same drawing.
- [ ] Manual in-host test checklist completed and recorded.

### Phase 6 — Desktop app
- [ ] A generated form for every calculation; works with networking disabled; no remote content.

## 2. Release 0.1 — definition of done

- [ ] Every calculation in [MVP_SCOPE.md](MVP_SCOPE.md) meets §3.
- [ ] Python and MCP interfaces pass parity tests.
- [ ] Offline CI job passes ([OFFLINE_POLICY.md](../docs/OFFLINE_POLICY.md)).
- [ ] `cargo-audit` / `cargo-deny` clean ([SECURITY.md](../docs/SECURITY.md)).
- [ ] Every public calculation has a generated doc page.
- [ ] Disclaimer visible in every interface.
- [ ] Release notes and checksums published.

## 3. Per-calculation checklist

- [ ] Descriptor complete: ID, version, inputs/outputs with dimensions, references, assumptions, validity.
- [ ] ≥1 reference case: known input, documented method and assumptions, cited source, result within the source's printed precision.
- [ ] Expected result **independently verified** (hand re-derivation or second source), with who verified it recorded in the case file.
- [ ] Edge cases: zero, validity-range limits, invalid inputs (correct error codes), alternative units.
- [ ] Trace steps present and correct.
- [ ] Generated doc page reviewed.
- [ ] Code review **and** engineering review approved.

## 4. Per-integration checklist

- [ ] Uses only the public contract — no engine internals.
- [ ] Respects the dependency rule.
- [ ] Error and warning output shown to the user, not swallowed.
- [ ] Works offline.
- [ ] Shows the disclaimer.
