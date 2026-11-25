# Contributing to CivKit

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-25 00:29 IST (setup and git sections from HANDOFF.md, 2026-11-24 23:41 IST) · last edited 2026-11-25 10:52 IST

> **Status:** Phase 0 — Planning. Code contributions are not open yet; feedback on the planning documents is
> welcome through GitHub issues. CivKit is licensed under [Apache-2.0](LICENSE); by contributing you agree that
> your contribution is licensed under the same terms (Apache-2.0, section 5).

---

## 1. Ways to contribute (once open)

| Contribution | Needs Rust? | Where |
|---|---|---|
| **Reference cases** — published worked examples (inputs, expected outputs, source) | No | `tests/reference/` |
| **Engineering review** of a calculation's formula, assumptions and limits | No | Pull-request review |
| **Calculations** in a domain module | Yes | `crates/civkit-core/` (domain modules) |
| **Integrations** — Python, MCP, AutoCAD/Civil 3D, MATLAB, Jupyter, desktop | Depends | `bindings/`, `integrations/`, `apps/` |
| **Documentation** | No | `docs/`, `planning/` |

Reference cases and reviews are often the most valuable contribution: they are how results earn trust.

## 2. Adding a calculation (planned process)

1. Read [docs/ENGINEERING_STANDARDS.md](docs/ENGINEERING_STANDARDS.md) and [docs/API_DESIGN.md](docs/API_DESIGN.md).
2. Copy the calculation template (created in Phase 1): one Rust file, one reference-case data file, one doc stub.
3. Fill in the descriptor: ID, inputs with dimensions, outputs, references, assumptions, validity range.
4. Add at least one reference case from a cited published worked example, plus edge cases.
5. Open a pull request. It needs one code review **and** one engineering review.

## 3. Architecture rule you must not break

Nothing in `crates/` may depend on anything outside `crates/`. Interfaces depend on the engine, never the
reverse. See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## 4. Development setup (Windows; all free)

| Tool | Version / notes | Needed for |
|---|---|---|
| Git for Windows | 2.55+ | Everything |
| Rust via rustup | stable, `stable-msvc` toolchain, with `rustfmt` and `clippy` (verified with 1.100.0) | Engine |
| Visual Studio 2026 Community | "Desktop development with C++" (Rust linker) and ".NET desktop development" (brings a .NET SDK; the AutoCAD 2027 plugin targets `net10.0`) | Engine (linker), AutoCAD/Civil 3D integration |
| Python | 3.14, project virtual environment in `.venv` with `maturin`, `jupyterlab`, `ipykernel` | Python bindings, Jupyter |
| Node.js | 26 LTS | Desktop app (Tauri 2), MCP Inspector (`npx @modelcontextprotocol/inspector`) |
| VS Code | Extensions: `rust-lang.rust-analyzer`, `tamasfe.even-better-toml`, `ms-vscode.cpptools`, `ms-python.python`, `ms-toolsai.jupyter`, `ms-dotnettools.csdevkit`, `tauri-apps.tauri-vscode` | Editing |
| SQLite CLI | optional | Inspecting desktop-app data (Phase 6) |
| AutoCAD 2027 / Civil 3D 2027 | proprietary, only for those integrations | Phase 5 |

Libraries such as PyO3, `rmcp`, Tauri and the AutoCAD .NET packages are project dependencies — they are
downloaded by the build, not installed separately.

Python environment (from the repository folder):

```powershell
py -3.14 -m venv .venv
.venv\Scripts\python -m pip install --upgrade pip
.venv\Scripts\python -m pip install maturin jupyterlab ipykernel
```

If `npm` / `npx` report that running scripts is disabled, use `npm.cmd` / `npx.cmd`.

## 5. Working with git

Branch: `main`. Commit messages: short imperative summary with a prefix (`docs:`, `feat:`, `fix:`, `test:`,
`chore:`). Tags mark known-good points: `plan-v0.1`, `plan-v1.0`, `phase-1-done`, …

### One-time setup per computer

```powershell
git config --global user.name "Your Name"
git config --global user.email "you@example.com"
```

### Everyday commands

| I want to… | Command |
|---|---|
| See history | `git log --oneline --decorate` |
| See what changed | `git status` then `git diff` |
| Save my work | `git add -A` then `git commit -s -m "docs: describe the change"` |
| Commit with a message an AI prepared | `git add -A` then `git commit -s -F _commit_msg.txt` (the file is git-ignored; avoids long messages being cut off when pasted) |
| See an old version of a file | `git show plan-v0.1:README.md` |
| Restore one file to an old version | `git restore --source=plan-v0.1 README.md` (then commit) |
| Undo a commit safely (keeps history) | `git revert <commit-id>` |
| Look at an old state without changing anything | `git switch --detach plan-v0.1` … return with `git switch main` |
| Mark a known-good point | `git tag -a plan-v1.0 -m "Planning complete"` |

Avoid `git reset --hard` and `git push --force` unless you are certain — they discard history.

## 6. Sign-off

Every commit must be signed off: use `git commit -s`. This adds a `Signed-off-by: Your Name <email>` line.
It certifies the [Developer Certificate of Origin 1.1](https://developercertificate.org/): you wrote the change, or
otherwise have the right to submit it under the project's licence. There is no CLA; you keep your copyright
(D-11, [ADR 0008](planning/decisions/0008-rust-edition-and-sign-off.md)). CI rejects commits without a sign-off from Phase 1.

Rust: all crates use the **2024 edition**; the minimum supported Rust version is **1.98** (latest stable − 2; D-10).

## 7. AI-assisted work

AI assistants are welcome as tools; humans remain responsible for what is merged.

- Before starting, an AI assistant reads [planning/AI_HANDOVER.md](planning/AI_HANDOVER.md) and follows its working rules.
- **Provenance is mandatory.** Every file an AI creates or edits carries a provenance line near the top with
  the model name, model ID and timestamp (IST), e.g.

  ```
  > **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-24 23:41 IST · last edited 2026-11-25 00:40 IST
  ```

  In files that cannot hold Markdown (e.g. `.gitignore`), use a comment line. On every edit, update "last edited".
  Exception: verbatim third-party texts such as `LICENSE` are never modified.
- Commits containing AI-written content add a trailer, e.g. `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- AI-written engineering content (formulas, limits, references) always needs human engineering review.
