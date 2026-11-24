# HANDOFF

> Read this first when picking the project up — whether you are the maintainer after a break,
> a new collaborator, or an AI assistant starting a fresh session.
> **Update it at the end of every working session.**

**Project:** CivKit *(working name — see D-1)*
**Current phase:** 0 — Planning
**Code written so far:** none (intentional)
**Last updated:** 2026-11-24

---

## 1. Working rules

1. **No code until Phase 0 is complete.** Planning documents only. Exit criteria are in §4.
2. **Work only inside this repository folder.** Do not read or modify files elsewhere on the machine
   (in particular, the maintainer's University folder is off-limits).
3. Every meaningful change is a git commit. Every phase end (and the end of planning) gets a git tag.
4. Decisions are recorded here as *Proposed* → *Accepted*. Accepted decisions also get a short ADR in
   `docs/decisions/NNNN-title.md`.
5. `README.md` = the plan. `HANDOFF.md` = the current state. Keep them consistent.

---

## 2. Where things stand

- The original concept proposal is preserved in `docs/00-original-proposal.md`.
- `README.md` contains the core plan, built from the proposal, with a few changes marked
  **[Change from proposal]**. These changes are *proposed*, not accepted — see D-3, D-4, D-12.
- Git is set up for this folder (`.gitignore`, `.gitattributes`); the repository itself is created by
  running the one-time commands in §5. The first commit is tagged `plan-v0.1`.
- Nothing has been published anywhere. No GitHub remote yet.

---

## 3. Decisions

### Already settled by the proposal
| # | Decision |
|---|---|
| S-1 | Rust is the core implementation language. |
| S-2 | Free, open source, local-first, no accounts, no telemetry, no cloud dependency. |
| S-3 | Interfaces: Python (PyO3), MCP, AutoCAD/Civil 3D (C#/.NET), MATLAB, Tauri desktop app. |
| S-4 | Civil engineering first; other disciplines later. |
| S-5 | Drawing analysis is read-only first; any modification needs explicit user confirmation. |
| S-6 | An OSI-approved licence will be used. |

### Open — needs the maintainer's decision
Priority **P0** = must be decided to leave Phase 0. **P1** = before the relevant phase starts.

| # | Pri | Question | Recommendation | Status |
|---|---|---|---|---|
| D-1 | P0 | Final project name? | Decide before any package/repo is published. Check GitHub, crates.io, PyPI, trademark. "CivKit" competes with *Civilization*-game projects in search results. | Proposed |
| D-2 | P0 | Licence? | **Apache-2.0 OR MIT** (dual, the Rust-ecosystem norm; permissive; Apache adds a patent grant). Avoid GPL: linking a GPL plugin against AutoCAD's proprietary libraries is legally awkward. MPL-2.0 is the middle option if you want modifications to CivKit files shared back. | Proposed |
| D-3 | P0 | AutoCAD plugin in 0.1 or 0.2? | **0.2.** Put DXF-based drawing analysis in 0.1 instead; the plugin then reuses it. | Proposed |
| D-4 | P0 | Roadmap order? | Python bindings in Phase 3 **with** MCP (originally Phase 5). Drawing model Phase 4, AutoCAD Phase 5. | Proposed |
| D-5 | P0 | Units approach? | Runtime `Quantity` (SI value + dimension exponents), angle as its own dimension, reject ambiguous units. | Proposed |
| D-6 | P0 | FFI contract? | JSON over a minimal C ABI (`call`, `list`, `free`). Typed fast paths only if profiling demands. | Proposed |
| D-7 | P0 | Which calculations in 0.1? | First-principles set in README §5 (traverse/Bowditch, levelling, soil phase relations, SS beam, Manning, earthwork volumes). Adjust to what you can verify from your textbooks. | Proposed |
| D-8 | P1 | Which design code first (after 0.1)? | Whichever your first users work with — e.g. IS codes (IS 456 / IS 800) if they are in India. One code at a time. | Proposed |
| D-9 | P1 | Supported AutoCAD versions? | 2025+ only (.NET 8; 2027 is .NET 10). Skip the .NET Framework 4.8 era (2024 and older). | Proposed |
| D-10 | P1 | Rust edition / minimum Rust version? | 2024 edition, latest stable at Phase 1 start; MSRV policy "latest stable − 2". | Proposed |
| D-11 | P1 | Contribution sign-off? | DCO (`git commit -s`), no CLA. | Proposed |
| D-12 | P0 | Add a neutral drawing model + DXF reader in Rust? | **Yes** — testable in CI, reusable by every interface, useful without AutoCAD. | Proposed |
| D-13 | P1 | MCP SDK? | Official `rmcp` crate, stdio transport only; pin version at Phase 3 start. | Proposed |
| D-14 | P1 | When does SQLite enter? | Phase 6 (desktop history). Not needed earlier. | Proposed |

---

## 4. Next actions (to exit Phase 0)

- [ ] Review README.md end to end; accept, change or reject each item in §3 "Open".
- [ ] Choose the name (D-1) and licence (D-2). Add `LICENSE` file(s).
- [ ] Write ADRs for accepted decisions in `docs/decisions/`.
- [ ] Write `docs/spec/calculation-contract.md`: request/response schema, error format, trace format,
      ID and versioning rules, warning levels.
- [ ] Write `docs/spec/units.md`: the full list of supported units, symbols, and rejected/ambiguous ones.
- [ ] For each 0.1 calculation: collect ≥1 published worked example (book, edition, page) with inputs and
      expected outputs. These become the first reference tests.
- [ ] Collect 3–5 small sample DXF drawings (own work, licence-safe) for drawing-analysis fixtures.
- [ ] Install the free toolchain (Git, Rust, Python + maturin) and confirm versions.
- [ ] Decide: create the GitHub repository now (private) or after the licence is chosen.
- [ ] Tag `plan-v1.0` when all P0 decisions are accepted → Phase 1 can start.

**Phase 0 exit criteria:** all P0 decisions accepted · contract and units specs written ·
reference examples collected for every 0.1 calculation.

---

## 5. Git: conventions and rolling back

Branch: `main`. Commit messages: short imperative summary, optionally prefixed (`docs:`, `feat:`, `fix:`).
Tags mark known-good points: `plan-v0.1`, `plan-v1.0`, `phase-1-done`, …

### One-time setup (if `git status` says "not a git repository")
Install Git for Windows if needed, open a terminal **in this folder**, and run:

```
git config --global user.name "Your Name"
git config --global user.email "you@example.com"
git init -b main
git add -A
git commit -m "docs: initial core plan, handoff and original proposal"
git tag -a plan-v0.1 -m "Planning draft 0.1"
```

(The first two lines are only needed once per computer.)

### Everyday commands
Run these from inside this folder (Git for Windows → *Git Bash*, or any terminal with `git`):

| I want to… | Command |
|---|---|
| See history | `git log --oneline --decorate` |
| See what changed since last commit | `git status` then `git diff` |
| Save my current work | `git add -A` then `git commit -m "docs: describe the change"` |
| See an old version of a file | `git show plan-v0.1:README.md` |
| Restore one file to an old version | `git restore --source=plan-v0.1 README.md` (then commit) |
| Undo a specific commit safely (keeps history) | `git revert <commit-id>` |
| Look around an old state without changing anything | `git switch --detach plan-v0.1` … return with `git switch main` |
| Mark a known-good point | `git tag -a plan-v1.0 -m "Planning complete"` |

Avoid `git reset --hard` and `git push --force` unless you are certain — they discard history.

---

## 6. Session log

Newest first. One short entry per session: what changed, what was decided, what is next.

### 2026-11-24 — Planning kick-off
- Created repository: `README.md` (core plan), `HANDOFF.md`, `docs/00-original-proposal.md`, `.gitignore`, `.gitattributes`.
- Git configuration files added; repository to be initialised by the maintainer with the §5 one-time
  commands (remote tools cannot write inside `.git`). First commit to be tagged `plan-v0.1`.
- Proposed changes vs. original proposal: Phase 0 added; AutoCAD moved to 0.2; neutral drawing model + DXF
  in Rust; Python moved up beside MCP; engineering-trust rules; SQLite deferred to the desktop phase.
- Facts checked: AutoCAD 2025/2026 plugins target .NET 8, AutoCAD 2027 targets .NET 10;
  the official Rust MCP SDK is the `rmcp` crate.
- **Next:** maintainer reviews README and decides D-1 … D-7, D-12.
