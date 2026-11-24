# AI Handover

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-24 23:41 IST (as HANDOFF.md) · last edited 2026-11-25 00:29 IST

> **Read this first** when picking the project up — the maintainer after a break, a new collaborator, or an AI
> assistant starting a fresh session. **Update it at the end of every working session.**

**Project:** CivKit *(working name — D-1)*
**Maintainer:** Gowtham
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
2. **Work only inside this repository folder.** Do not read or modify anything else on the machine —
   in particular, the maintainer's University folder is off-limits.
3. **Provenance on every write.** Every file an AI creates or edits carries, near the top, a provenance line
   with model name, model ID and timestamp in IST; update "last edited" on every edit. Use a comment line in
   non-Markdown files. Suggested commit messages include a `Co-Authored-By` trailer. (ADR 0003)
4. **One home per fact.** Decisions → DECISIONS.md; tasks → IMPLEMENTATION_PLAN.md; exit criteria →
   ACCEPTANCE_CRITERIA.md. Link, don't copy.
5. **Decisions** go Proposed → Accepted only when the maintainer says so; accepted ones get an ADR.
6. **Git:** the maintainer runs git commands; AI tools cannot write inside `.git`. After changes, give the
   maintainer the exact `git add` / `git commit` commands. Commands reference: [CONTRIBUTING.md §5](../CONTRIBUTING.md#5-working-with-git).
7. Never present unverified engineering content as fact; flag it for engineering review.

## 3. Where things stand

- Planning documents restructured into root / `docs/` / `planning/` (D-16, ADR 0003).
- Accepted: D-12 (drawing model), D-15 (repository layout), D-16 (documentation structure).
- Open P0 decisions: D-1 name, D-2 licence, D-3, D-4, D-5, D-6, D-7.
- Toolchain install in progress on the maintainer's PC (Windows 11, PowerShell 5.1):
  - Done: Rust via rustup (winget, 2026-11-25 ~00:20 IST).
  - In progress: Visual Studio 2026 Community, C++ + .NET desktop workloads (4.38 GB download).
  - To verify: `rustup default stable-msvc` + rustfmt/clippy, SQLite CLI, `.venv` (maturin, jupyterlab, ipykernel),
    VS Code extensions. `ms-vscode.cpptools` is already installed.
  - Already present: Git 2.55, Python 3.14.10 (plus an old Anaconda 3.7 — do not use), Node.js 26 LTS,
    VS Code, WebView2, AutoCAD 2027 (Civil 3D presence unchecked: `Test-Path "C:\Program Files\Autodesk\AutoCAD 2027\C3D"`).
- Git tags: `plan-v0.1`. Nothing published; no GitHub remote.

## 4. Next actions

1. Verify the toolchain (check command in the 2026-11-25 session log below).
2. Decide D-1 (name) and D-2 (licence).
3. Decide D-3 … D-7, then finish the TBDs in API_DESIGN.md.
4. Collect worked examples for each 0.1 calculation.

Full Phase 0 task list: [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md).

---

## 5. Session log

Newest first. One short entry per session: what changed, what was decided, what is next.

### 2026-11-25 00:25 IST — Documentation restructure, provenance, install progress *(Claude Opus 5.5)*
- **Accepted D-16** (ADR 0003): documentation split into root (`README`, `ROADMAP`, `CONTRIBUTING`),
  `docs/` (`ARCHITECTURE`, `API_DESIGN`, `ENGINEERING_STANDARDS`, `OFFLINE_POLICY`, `SECURITY`) and
  `planning/` (`MVP_SCOPE`, `IMPLEMENTATION_PLAN`, `ACCEPTANCE_CRITERIA`, `DECISIONS` + `decisions/`,
  `AI_HANDOVER`, `ORIGINAL_PROPOSAL`). `HANDOFF.md` replaced by this file.
- New content drafted: API design (contract, errors, units, Python/FFI/MCP surfaces), offline policy with
  enforcement, security threat model.
- **Provenance rule adopted** at the maintainer's request; provenance lines added to all files written so far.
- An overnight attempt to delegate installation to another AI agent (Codex) did nothing: its sandbox could not
  run winget or reach installer windows. No changes were made by it.
- Rust installed via winget; Visual Studio 2026 still installing.
- Toolchain check command (new PowerShell window):
  `& "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe" -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property displayName; rustc --version; cargo clippy --version; dotnet --list-sdks; sqlite3 --version; .venv\Scripts\maturin --version`
- **Next:** verify toolchain; decide D-1 and D-2.

### 2026-11-25 00:14 IST — Repository layout and toolchain check *(Claude Opus 5.5)*
- **Accepted D-15** (monorepo layout, ADR 0001) with four refinements: MCP moved from `crates/` to
  `integrations/mcp`; `crates/civkit-ffi` added; `crates/civkit-drawing` added; `integrations/jupyter` holds
  notebooks/widgets only and `civil3d/` references `autocad/`'s shared project.
- **Accepted D-12** (neutral drawing model + DXF in Rust, ADR 0002).
- Crate names now `civkit-core` (was `civkit-api`) and `civkit-geometry` (was `civkit-geom`).
- D-9 recommendation changed to "AutoCAD 2027 only to start" — AutoCAD 2027 (.NET 10) is installed.
- Environment check: Git 2.55, Python 3.14.10, Node.js 26 LTS, VS Code, WebView2, AutoCAD 2027 present;
  654 GB free on C:. Missing: Rust, Visual Studio C++/.NET (no .NET SDK), maturin, JupyterLab, SQLite CLI,
  VS Code extensions.
- Commit `572b34b`.

### 2026-11-24 23:41 IST — Planning kick-off *(Claude Opus 5.5)*
- Created `README.md` (core plan), `HANDOFF.md`, `docs/00-original-proposal.md`, `.gitignore`, `.gitattributes`.
- Repository initialised by the maintainer (AI tools cannot write inside `.git`); first commit `00e7db6`
  tagged `plan-v0.1`.
- Proposed changes vs. the original proposal: Phase 0 added; AutoCAD moved to 0.2; neutral drawing model + DXF
  in Rust; Python moved up beside MCP; engineering-trust rules; SQLite deferred to the desktop phase.
- Facts checked: AutoCAD 2025/2026 plugins target .NET 8, AutoCAD 2027 targets .NET 10;
  the official Rust MCP SDK is the `rmcp` crate.
