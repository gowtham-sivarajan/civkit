# AI Handover

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-24 23:41 IST (as HANDOFF.md) · last edited 2026-11-25 11:32 IST

> **Read this first** when picking the project up — the maintainer after a break, a new collaborator, or an AI
> assistant starting a fresh session. **Update it at the end of every working session.**

**Project:** CivKit
**Maintainer:** Gowtham
**Repository:** public on GitHub from day one (D-17) · licence Apache-2.0
**Current phase:** 0 — Planning
**Code written so far:** none (intentional)

---

## 1. Read order for a new session

1. This file (state, rules, next actions).
2. [README.md](../README.md) — what the project is and where everything lives.
3. [DECISIONS.md](DECISIONS.md) — what is decided and what is open.
4. [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md) — the current phase's task list.
5. Then whatever the task needs: [ARCHITECTURE.md](../docs/ARCHITECTURE.md), [API_DESIGN.md](../docs/API_DESIGN.md), …

## 2. Working rules

1. **No code until Phase 0 is complete** ([ACCEPTANCE_CRITERIA.md](ACCEPTANCE_CRITERIA.md)). Documents only.
   In particular, **no calculation is implemented until D-3 … D-7 are resolved** and every Phase 0 criterion is met.
   The only exception is a throwaway toolchain smoke test outside the repository.
2. **Work only inside this repository folder.** Do not read or modify anything else on the maintainer's machine.
3. **Provenance on every write.** Every file an AI creates or edits carries, near the top, a provenance line
   with model name, model ID and timestamp in IST; update "last edited" on every edit. Use a comment line in
   non-Markdown files. Suggested commit messages include a `Co-Authored-By` trailer. (ADR 0003)
4. **One home per fact.** Decisions → DECISIONS.md; tasks → IMPLEMENTATION_PLAN.md; exit criteria →
   ACCEPTANCE_CRITERIA.md. Link, don't copy.
5. **Decisions** go Proposed → Accepted only when the maintainer says so; accepted ones get an ADR.
6. **Git:** the maintainer runs git commands; AI tools cannot write inside `.git`. After changes, give the
   maintainer the exact `git add` / `git commit` commands. Commands reference: [CONTRIBUTING.md §5](../CONTRIBUTING.md#5-working-with-git).
7. Never present unverified engineering content as fact; flag it for engineering review.
8. **Protect the engine's independence:** no AutoCAD, Python, network or AI dependency in `crates/` ([ARCHITECTURE.md](../docs/ARCHITECTURE.md)).
9. **Reference cases are mandatory:** known input, documented method, independently verified result, edge-case tests ([ENGINEERING_STANDARDS.md](../docs/ENGINEERING_STANDARDS.md)).

## 3. Where things stand

- Planning documents restructured into root / `docs/` / `planning/` (D-16, ADR 0003).
- Accepted: D-1 name (CivKit), D-2 licence (Apache-2.0), D-3 … D-6 (scope, order, units, FFI — ADR 0005),
  D-7 + D-18 (open sources only; 0.1 set driven by them — ADR 0006), D-12 drawing model, D-15 repository layout,
  D-16 documentation structure, D-17 public repository from day one.
- All P0 decisions accepted (D-1 … D-7, D-12, D-15 … D-18).
- Maintainer's development machine: Windows 11, PowerShell 5.1. **Toolchain verified 2026-11-25:** Rust 1.100.0
  (`stable-x86_64-pc-windows-msvc`, rustfmt 1.11.0, clippy), MSVC linker works (smoke test built and ran),
  .NET SDK 11.0.100, SQLite 3.54.1, `.venv` with maturin 1.16.0 and JupyterLab 4.6.6, VS Code extensions.
  Note for Phase 5: only the .NET **11** SDK is installed; it can build `net10.0` targets for AutoCAD 2027,
  but pin the SDK with `global.json` (or add the .NET 10 SDK) if builds misbehave.
  Required tools: [CONTRIBUTING.md §4](../CONTRIBUTING.md#4-development-setup-windows-all-free).
  Use the project's `.venv` for Python, not any system-wide distribution.
- Git tags: `plan-v0.1`. Public at github.com/gowtham-sivarajan/civkit (pushed 2026-11-25 ~00:47 IST);
  private vulnerability reporting enabled. Dependency graph / Dependabot to be enabled in Phase 1.

## 4. Next actions

1. Maintainer: move `_move_to_dot_github/` contents into `.github/`, commit, push; watch the first CI run.
   Fix anything CI finds on Windows/macOS or Rust 1.98 (only Linux + Rust 1.97 was available to the AI).
2. When CI is green: tag `phase-1-done`, then Phase 2 (civkit-units, civkit-geometry, the five 0.1 calculations; C1–C9 → tests/reference).
2. Later (maintainer has no AutoCAD 2027 licence yet): add an AutoCAD-written fixture once a licence that allows publishing exists.

Deferred by the maintainer: human engineering review of C1–C9 (until the product is complete);
general prismoidal and soil phase relationships (later release).

Full Phase 0 task list: [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md).

---

## 5. Session log

Newest first. One short entry per session: what changed, what was decided, what is next.

### 2026-11-25 11:32 IST — Phase 1 code *(Claude Opus 5.5)*
- Cargo workspace (2024 edition, MSRV 1.98, workspace lints: unsafe forbidden, missing docs warn), five crates.
- `civkit-units`: `Dimension` only (angle separate). `civkit-core`: descriptor, registry, request/response, errors and
  envelope, warnings, trace, validity rules (warn/escalate), domain checks, `MISSING_UNIT`, panic → `INTERNAL`, `call_json`.
  Units given in requests return `UNKNOWN_UNIT` until Phase 2.
- Template calculation `math.demo.weighted_mean` (never a built-in); reference-case format (`tests/reference/README.md`)
  and harness; check that every registered calculation has a case.
- CI: fmt; build/test on Windows/macOS/Linux × Rust 1.98/stable; clippy -D warnings; dependency-rule script (proved to
  catch a planted path dependency and a planted tokio); cargo-deny (passes locally with 0.22.1); DCO check (logic tested).
  Actions pinned by SHA: checkout v7.0.1, cargo-deny-action v2.1.4. Dependabot weekly for cargo and actions.
- Verified locally on Linux with Rust 1.97 using `--ignore-rust-version`. Rust 1.98+ and Windows/macOS untested until CI.
- actionlint could not be run (downloading and running its binary was blocked); workflow YAML parsed and SHA pins checked instead.

### 2026-11-25 10:52 IST — Phase 0 complete; D-10, D-11 accepted *(Claude Opus 5.5)*
- D-10 (2024 edition, MSRV 1.98) and D-11 (DCO, no CLA) accepted → ADR 0008; CONTRIBUTING updated.
- Commit messages: pasted multi-part `git commit -m … -m …` lines were cut off twice in the maintainer's terminal.
  New method: the AI writes a git-ignored `_commit_msg.txt`; the maintainer runs `git commit -s -F _commit_msg.txt`.
- `plan-v1.0` was first pushed on a commit that lacked task 0.9 and the community files (the commit had failed);
  it is re-created on the Phase 0 completion commit.

### 2026-11-25 10:50 IST — GitHub community files *(Claude Opus 5.5)*
- `CODE_OF_CONDUCT.md`: Contributor Covenant 3.0 (CC BY-SA 4.0) copied from source; reporting contact = maintainer's
  commit email (already public in commits; maintainer agreed to a public email).
- `.github/ISSUE_TEMPLATE/`: wrong-result form (treated as the most serious bug), bug report, calculation/feature request
  (asks for an openly licensed worked example, D-18), config (blank issues off; private security link).
- `.github/PULL_REQUEST_TEMPLATE.md` (dependency rule, offline, reference-case and AI-provenance checks); `ACCESSIBILITY.md`.
- Remote AI file tools cannot write inside `.github/` (protected, like `.git/`). Files were written to `_move_to_dot_github/`
  and the maintainer moves them into `.github/`.

### 2026-11-25 10:40 IST — Task 0.9: DXF fixtures *(Claude Opus 5.5)*
- `tests/fixtures/dxf/`: F1 metric site plan, F2 same in mm, F3 planted unit problems, F4 R12 legacy, F5 blocks
  (rotated, stretched, mirrored, MINSERT, nested). F1–F3/F5 drawn in CADCraft; F4 written with ezdxf.
- Two MIT samples from ezdxf (`insert_bricscad_level_1.dxf` from BricsCAD; `Minimal_DXF_AC1009.dxf`), licence file kept with them.
- Expected values fixed by construction and re-checked with ezdxf + shapely (0 failures); script kept outside the repo.
- University folders offered by the maintainer were not opened: not needed, and they may hold personal or coursework data.

### 2026-11-25 10:03 IST — D-19 accepted *(Claude Opus 5.5)*
- Maintainer accepted all nine API resolutions → D-19, ADR 0007; API_DESIGN.md has no open TBDs; task 0.7 ✅.
- Correction: the two previous log entries were stamped 10:08 and 10:16 IST — times estimated ahead of the real clock.
  Re-stamped to 09:57 and 09:59 IST (checked against the current time). Lesson: read the clock, never estimate.

### 2026-11-25 09:59 IST — Task 0.7: API TBDs resolved (proposed) *(Claude Opus 5.5)*
- API_DESIGN.md: every TBD replaced by a proposed resolution; summary table §12.
- Notable choices: no silent default unit (`MISSING_UNIT`); `lb` and `k` rejected as ambiguous; `ft` vs `ft_us`;
  DMS and quadrant bearings as structured values; validity breaches warn by default, escalation per rule.
- Unit list checked against every unit used in C1–C9.

### 2026-11-25 09:57 IST — 0.1 calculation set fixed *(Claude Opus 5.5)*
- Second search for prismoidal and soil phase examples found nothing openly licensed (NPTEL: licence unclear;
  Sivakugan: no licence; FHWA NHI-06-088 and EM 1110-2-1906: no worked example). Maintainer: ship without them.
- 0.1 calculations: average end area (+ zero-end pyramid), Manning, simply supported beam, levelling, traverse/compass rule.
- Maintainer deferred the human engineering review of C1–C9 until the product is complete.
  ENGINEERING_STANDARDS is unchanged: each calculation still ships with its reference-case tests.

### 2026-11-25 09:52 IST — Task 0.8: reference cases collected and re-derived *(Claude Opus 5.5)*
- New `planning/REFERENCE_CASES.md`: 9 cases (C1–C9) from open sources, all re-derived by script.
- Gaps filled: traverse + compass/Bowditch (US Navy NAVEDTRA 14070, complete worked adjustment) and levelling
  (NAVEDTRA 14069, 5-setup circuit with closure and adjustment). Beams from two independent sources (Udoeyo; Virginia Tech).
- Still open: soil phase relationships (no open worked example found in 5 sources checked) and the general
  prismoidal case (only the pyramid special case found).
- Source errata found and recorded: NAVEDTRA 14070 CD departure; NAVEDTRA 14069 backsight 11.65/11.56;
  HDS-4 Ex 4.1 rounding and unit typo; Wikibooks station range; Kouretzis ρw.
- Correction: during research Claude first called HDS-4 Ex 4.1 an arithmetic error (mental arithmetic slip);
  the script re-derivation showed the book is right apart from rounding. Lesson recorded: verify by computation only.
- Sources rejected: Mahun (no licence stated); Verruijt (licence unclear).
- **Next:** maintainer decides prismoidal + soil phase; human review of C1–C9; then API_DESIGN TBDs.

### 2026-11-25 09:08 IST — D-3 … D-7 decided; open-source reference policy *(Claude Opus 5.5)*
- **Accepted D-3 … D-6** (ADR 0005) as recommended.
- **Accepted D-18:** reference cases only from openly licensed or public-domain sources; and **D-7:** the 0.1
  calculation set is driven by those sources, with an entry rule (ADR 0006).
- Source search done: register in `planning/REFERENCE_SOURCES.md` (11 approved sources, 1 rejected for unclear
  licence). Confirmed and re-derived: average-end-area example (Wikibooks, 4,900 m³). Gaps: traverse/Bowditch,
  prismoidal.
- **Next:** task 0.8 — locate the remaining worked examples.

### 2026-11-25 08:58 IST — Toolchain verified *(Claude Opus 5.5)*
- Task 0.4 done: Rust 1.100.0 stable-msvc with rustfmt and clippy; smoke test compiled, linked and ran
  (outside the repository); .NET SDK 11.0.100; SQLite 3.54.1; `.venv` with maturin 1.16.0, JupyterLab 4.6.6,
  ipykernel 7.4.0; all VS Code extensions installed.
- **Next:** decide D-3 … D-7.

### 2026-11-25 01:00 IST — Principles locked in; repository public *(Claude Opus 5.5)*
- Repository pushed to GitHub (`main` at `8de55c2`, tag `plan-v0.1`); private vulnerability reporting enabled.
- Maintainer's protected principles written into the docs: reference cases are mandatory and independently
  verified; the engine has no AutoCAD, Python, network or AI dependency; no calculation is implemented until
  D-3 … D-7 are resolved and Phase 0 is complete.
- Status: Rust (rustup) and `.venv` installed but **not yet verified**; Visual Studio 2026 still installing.
- **Next:** toolchain checkpoint (see §4), then D-3 … D-7.

### 2026-11-25 00:40 IST — Name, licence, public repository *(Claude Opus 5.5)*
- **Accepted D-1** (name: CivKit), **D-2** (licence: Apache-2.0), **D-17** (public GitHub repository from day one) — ADR 0004.
- `LICENSE` added (verbatim Apache-2.0 text). Name check: `civkit` free on crates.io and PyPI; a dormant
  Bitcoin project uses the name on GitHub, and an unrelated npm package exists.
- Personal machine details removed from this file before publishing.
- **Next:** create the GitHub repository and push; verify toolchain.

### 2026-11-25 00:25 IST — Documentation restructure, provenance, install progress *(Claude Opus 5.5)*
- **Accepted D-16** (ADR 0003): documentation split into root (`README`, `ROADMAP`, `CONTRIBUTING`),
  `docs/` (`ARCHITECTURE`, `API_DESIGN`, `ENGINEERING_STANDARDS`, `OFFLINE_POLICY`, `SECURITY`) and
  `planning/` (`MVP_SCOPE`, `IMPLEMENTATION_PLAN`, `ACCEPTANCE_CRITERIA`, `DECISIONS` + `decisions/`,
  `AI_HANDOVER`, `ORIGINAL_PROPOSAL`). `HANDOFF.md` replaced by this file.
- New content drafted: API design (contract, errors, units, Python/FFI/MCP surfaces), offline policy with
  enforcement, security threat model.
- **Provenance rule adopted** at the maintainer's request; provenance lines added to all files written so far.
- Toolchain installation started (Rust installed; Visual Studio 2026 in progress).
- **Next:** verify toolchain; decide D-1 and D-2.

### 2026-11-25 00:14 IST — Repository layout and toolchain check *(Claude Opus 5.5)*
- **Accepted D-15** (monorepo layout, ADR 0001) with four refinements: MCP moved from `crates/` to
  `integrations/mcp`; `crates/civkit-ffi` added; `crates/civkit-drawing` added; `integrations/jupyter` holds
  notebooks/widgets only and `civil3d/` references `autocad/`'s shared project.
- **Accepted D-12** (neutral drawing model + DXF in Rust, ADR 0002).
- Crate names now `civkit-core` (was `civkit-api`) and `civkit-geometry` (was `civkit-geom`).
- D-9 recommendation changed to "AutoCAD 2027 only to start" — AutoCAD 2027 (.NET 10) is installed.
- Environment check on the maintainer's machine; missing tools identified (see CONTRIBUTING.md §4).
- Commit `572b34b`.

### 2026-11-24 23:41 IST — Planning kick-off *(Claude Opus 5.5)*
- Created `README.md` (core plan), `HANDOFF.md`, `docs/00-original-proposal.md`, `.gitignore`, `.gitattributes`.
- Repository initialised by the maintainer (AI tools cannot write inside `.git`); first commit `00e7db6`
  tagged `plan-v0.1`.
- Proposed changes vs. the original proposal: Phase 0 added; AutoCAD moved to 0.2; neutral drawing model + DXF
  in Rust; Python moved up beside MCP; engineering-trust rules; SQLite deferred to the desktop phase.
- Facts checked: AutoCAD 2025/2026 plugins target .NET 8, AutoCAD 2027 targets .NET 10;
  the official Rust MCP SDK is the `rmcp` crate.
