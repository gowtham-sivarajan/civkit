# Contributing to CivKit

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-25 00:29 IST (setup and git sections from HANDOFF.md, 2026-11-24 23:41 IST) · last edited 2026-11-25 00:29 IST

> **Status:** Phase 0 — Planning. Code contributions are not open yet, and no outside contributions can be
> accepted until a licence is chosen (D-2). Feedback on the planning documents is welcome.

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
| Rust via rustup | stable, `stable-msvc` toolchain, with `rustfmt` and `clippy` | Engine |
| Visual Studio 2026 Community | "Desktop development with C++" (Rust linker) and ".NET desktop development" (.NET 10 SDK) | Engine (linker), AutoCAD/Civil 3D integration |
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
| Save my work | `git add -A` then `git commit -m "docs: describe the change"` |
| See an old version of a file | `git show plan-v0.1:README.md` |
| Restore one file to an old version | `git restore --source=plan-v0.1 README.md` (then commit) |
| Undo a commit safely (keeps history) | `git revert <commit-id>` |
| Look at an old state without changing anything | `git switch --detach plan-v0.1` … return with `git switch main` |
| Mark a known-good point | `git tag -a plan-v1.0 -m "Planning complete"` |

Avoid `git reset --hard` and `git push --force` unless you are certain — they discard history.

## 6. Sign-off

Proposed: Developer Certificate of Origin (`git commit -s`), no CLA (D-11).

## 7. AI-assisted work

AI assistants are welcome as tools; humans remain responsible for what is merged.

- Before starting, an AI assistant reads [planning/AI_HANDOVER.md](planning/AI_HANDOVER.md) and follows its working rules.
- **Provenance is mandatory.** Every file an AI creates or edits carries a provenance line near the top with
  the model name, model ID and timestamp (IST), e.g.

  ```
  > **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-24 23:41 IST · last edited 2026-11-25 00:40 IST
  ```

  In files that cannot hold Markdown (e.g. `.gitignore`), use a comment line. On every edit, update "last edited".
- Commits containing AI-written content add a trailer, e.g. `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- AI-written engineering content (formulas, limits, references) always needs human engineering review.
