# AI Handover

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-24 23:41 IST (as HANDOFF.md) · last edited 2026-11-27 10:51 IST by Claude Sonnet 5.5 (`claude-sonnet-5-5`) by OpenAI Codex (GPT-6; runtime model ID unavailable)

> **Read this first** when picking the project up — the maintainer after a break, a new collaborator, or an AI
> assistant starting a fresh session. **Update it at the end of every working session.**

**Project:** CivKit
**Maintainer:** Gowtham
**Repository:** public on GitHub from day one (D-17) · licence Apache-2.0
**Current phase:** Phase 2 is complete; CI run #11 passed on 87fa3d2 and the phase-2-done tag is on GitHub. **Next: Phase 3** (Python and MCP). D6 and engineering review remain open for the bulk review before 0.1.
**Code written so far:** `civkit-core` (contract), `civkit-units` (2A), `civkit-geometry` (2B); seven registered engineering calculations covering the five D-20 families (2C complete; the beam's maximum shear and moment are deferred from 0.1, D-25)

---

## 1. Read order for a new session

1. This file (state, rules, next actions).
2. [README.md](../README.md) — what the project is and where everything lives.
3. [DECISIONS.md](DECISIONS.md) — what is decided and what is open.
4. [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md) — the current phase's task list.
5. Then whatever the task needs: [ARCHITECTURE.md](../docs/ARCHITECTURE.md), [API_DESIGN.md](../docs/API_DESIGN.md), …

## 2. Working rules

1. **Phases in order.** Phase 0 (planning) is complete (`plan-v1.0`); Phase 1 is complete. Work only on the current
   phase's tasks ([IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md)); every calculation needs its reference cases first.
2. **Work only inside this repository folder.** Do not read or modify anything else on the maintainer's machine.
3. **Provenance on every write.** Every file an AI creates or edits carries, near the top, a provenance line
   with model name, model ID and timestamp in IST; update "last edited" on every edit. Use a comment line in
   non-Markdown files. Suggested commit messages include a `Co-Authored-By` trailer. (ADR 0003)
   **Generate every timestamp with the shell at the moment of writing** (`TZ=Asia/Kolkata date '+%Y-%m-%d %H:%M IST'`
   inside the same command that writes the file). **Never type or estimate a time:**
   a stamp must never be later than the moment the file is actually written. (Estimated stamps ran ahead of the
   clock four times on 2026-11-25 and had to be corrected from file modification times.)
4. **One home per fact.** Decisions → DECISIONS.md; tasks → IMPLEMENTATION_PLAN.md; exit criteria →
   ACCEPTANCE_CRITERIA.md. Link, don't copy.
5. **Decisions** go Proposed → Accepted only when the maintainer says so; accepted ones get an ADR.
6. **Git:** the maintainer runs git commands; AI tools cannot write inside `.git`. After changes, give the
   maintainer the exact commands. Put the commit message in the git-ignored `_commit_msg.txt` and have the maintainer
   run `git commit -s -F _commit_msg.txt` (pasted multi-part `-m` lines get cut off; `-s` is the D-11 sign-off). Commands reference: [CONTRIBUTING.md §5](../CONTRIBUTING.md#5-working-with-git).
   **GitHub pushes: allowed again** (resumed by the maintainer on 2026-11-26 after a pause). The AI never runs
   `git push` itself; it gives the exact commands after `git fetch` and `git status -sb` checks. CI runs only on pushed
   commits, so tags and phase gates that name CI wait for the maintainer to report the CI result. The maintainer can
   pause pushing again at any time; if so, record it here.
7. Never present unverified engineering content as fact; flag it for engineering review.
8. **Protect the engine's independence:** no AutoCAD, Python, network or AI dependency in `crates/` ([ARCHITECTURE.md](../docs/ARCHITECTURE.md)).
9. **Reference cases are mandatory:** known input, documented method, independently verified result, edge-case tests ([ENGINEERING_STANDARDS.md](../docs/ENGINEERING_STANDARDS.md)).

## 3. Where things stand (snapshot 2026-11-26)

**Phases**
- Phase 0 (planning): ✅, tag `plan-v1.0`.
- Phase 1 (workspace, `civkit-core` contract, CI): ✅, tag `phase-1-done`.
- Phase 2 runs in stages 2A → 2D ([IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md)).
  - **2A (units)**: ✅. `6d3d16a`, CI green ([run #4](https://github.com/gowtham-sivarajan/civkit/actions/runs/45598540213)).
  - **2B (geometry)**: ✅. Specification accepted (D-22, [ADR 0010](decisions/0010-geometry-conventions.md)) after the
    [review](GEOMETRY_SPEC_REVIEW.md); implemented in `8a93d58`, CI green
    ([run #5](https://github.com/gowtham-sivarajan/civkit/actions/runs/45622288253)); the eight implementation items
    confirmed by the maintainer. The follow-up (derived tolerances must be finite) passed the maintainer's Windows
    check; it is committed together with 2C part 1.
  - **2C (calculations)**: ✅ complete. Done: `fluids.open_channel.manning_trapezoidal` (C5) and
    `structural.beam.simply_supported` (C6–C8: reactions, deflection, end slope). D-23 (list and choice inputs)
    is accepted and committed (`e948bc6`, [ADR 0011](decisions/0011-list-and-choice-inputs.md)).
    **`surveying.traverse.compass_rule` (C3) is implemented and committed** (`ddecbfb`). Levelling and earthwork are
    committed and pushed (`948a49a`, CI green on all nine jobs). **The five D-20 calculation families are complete.**
    The beam's maximum shear and moment are deferred from 0.1 (D-25). 2D review done; D-24 and D-25 decided
    ([ADR 0012](decisions/0012-validity-basis-and-beam-scope.md)).

**Decisions**
- All P0 and P1 decisions needed so far are accepted: D-1 … D-7, D-10 … D-13, D-15 … D-20, D-22 … D-26.
- Proposed: D-8, D-9, D-14; D-21 (SI prefixes and Unicode units, after 0.1).
  See [DECISIONS.md](DECISIONS.md).

**Engine state**
- `civkit-units`: named units, the formal unit grammar (one `/`), DMS (whole minutes) and bearings, all per
  [API_DESIGN.md §7.1](../docs/API_DESIGN.md).
- `civkit-core`: contract with three unit roles (internal SI, default presentation, requested). Internal units are
  enforced in values, trace and messages.
- `civkit-geometry`: angles and direction, tolerance, polyline/arc/circle lengths, polygon area, orientation,
  centroid (straight edges), the self-intersection check with exact arc relations. Every item cites
  [GEOMETRY.md](../docs/GEOMETRY.md). Rust only: no contract calculations (D-22).
- Registered built-ins: `fluids.open_channel.manning_trapezoidal`, `structural.beam.simply_supported`,
  `surveying.traverse.compass_rule` (first list calculation; `civkit-core` now depends on `civkit-geometry`),
  `surveying.levelling.height_of_instrument`, `surveying.levelling.rise_and_fall`,
  `surveying.levelling.closure_adjustment`, `quantities.earthwork.average_end_area` (first choice input). The template
  `math.demo.weighted_mean` is registered only in tests.

**Repository**
- Public at github.com/gowtham-sivarajan/civkit; private vulnerability reporting on.
- CI: fmt; build/test on Windows/macOS/Linux × Rust 1.98/stable; clippy; dependency rule; cargo-deny; DCO.
  Dependabot is weekly. Actions are pinned by SHA.
- `planning/ui-concepts/` exists on the maintainer's machine, untracked. It was not created by an AI session; leave it alone.
- `.venv/` and `.vs/` (Jupyter environment, Visual Studio state) exist on the maintainer's machine; both are git-ignored.

**Machine:** Windows 11, PowerShell 5.1, Rust 1.100.0 (MSRV 1.98), .NET SDK 11.0.100, `.venv` (maturin,
JupyterLab). Details: [CONTRIBUTING.md §4](../CONTRIBUTING.md#4-development-setup-windows-all-free). Note for Phase 5:
only the .NET 11 SDK is installed; pin with `global.json` if `net10.0` builds misbehave.

**Verification commands** (all must pass before a stage is called done; the same checks run in CI):

```powershell
cd C:\Users\Gowtham\Documents\03_Projects\CivKit
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
python tools/check_dependency_rule.py
cargo deny check          # if cargo-deny is installed locally; CI always runs it
cargo doc --no-deps --workspace
git status                # confirm planning/ui-concepts/ is not staged
```

## 4. Next actions

1. **Phase 3 preconditions: settled (2026-11-26).** D-13 ([ADR 0013](decisions/0013-mcp-sdk-and-surface.md): `rmcp` `=3.12.0`, both protocol eras, tool names and the result mapping, gates G1–G3) and D-26 ([ADR 0014](decisions/0014-python-package.md): CPython 3.11–3.15, PyO3 0.30.1 abi3 wheels for five targets, the thin binding layer, gates P1–P5). D-26 was the last open precondition.
2. **Phase 3 implementation:** Python package code is done (`bindings/python`, see the log); **its gates P1–P5 are
   met only when the `Python` workflow is green on GitHub.** The MCP server (`integrations/mcp`) is written, its G1–G3
   tests pass locally, and MCP Inspector and the Python MCP SDK agree with the engine in both eras. G1–G3 are met
   when `ci.yml` is green: **met** (CI #17 and Python #2 green on `0013681`). Parity test written (`tests/parity`),
   met when its CI job is green. Next: notebooks and generated documentation, then `phase-3-done` (tag only after
   green CI), then Phase 4.
3. **Phase 4 to complete 0.1:** implement the neutral drawing model and DXF reader, drawing limits, quantity extraction and unit checks, schedules, geometry transformations and OCS, DXF fixtures and fuzzing. Resolve geometry-note and unsupported-feature contract details before exposure.
4. **Bulk human review before 0.1:** after Phase 4 code is complete, bundle the D6 code review, the separate engineering review of C1–C9, Phase 4 DXF results and fixtures, and generated documentation review. Keep D6 and engineering approval as separate records. D6 remains open until a human signs off.
5. D5 (local cargo-deny) is optional; CI already checks it.
6. Beam maximum shear and moment remain deferred from 0.1 (D-25); continue looking for a suitable open worked example. General prismoidal and soil phase relationships remain later-release work.
7. The phase-2-done tag and CI gate are complete: run #11 passed on 87fa3d2 and the tag is on GitHub. Pushing is resumed; the maintainer runs Git commands and reports CI.
8. Preserve planning/ui-concepts/; it is maintainer-owned and untracked.

## 5. Session log

Newest first. One short entry per session: what changed, what was decided, what is next.

### 2026-11-27 10:51 IST — Phase 3: cross-interface parity (Rust, Python, MCP) *(Claude Sonnet 5.5)*
- `0013681` (MCP server) was pushed by the maintainer, with the `.pdb` untracked; CI #17 and Python #2 both green
  (https://github.com/gowtham-sivarajan/civkit/actions/runs/45944436704, https://github.com/gowtham-sivarajan/civkit/actions/runs/45944436666). The maintainer set the order: parity only after both pass. Acceptance items for gates G1–G3 and
  P1–P5 are ticked, citing those runs.
- **Parity** (`tests/parity/test_parity.py`, unittest): 13 reference cases + 7 error cases + 3 rounding cases, each
  through Rust `Registry::call_json` (new `crates/civkit-core/examples/call_json.rs`, JSON lines in and out), the
  installed Python package, and `civkit-mcp` in both eras via its own tool and `civkit_call` (a minimal stdio
  JSON-RPC client, no SDK). Separate processes; every answer read with Python's `json` (correctly rounding).
- **The 0.013 regression:** `0.013` itself parses correctly with either serde_json parser (short decimals take an
  exact path); the bug was the output: with n = 0.013 the velocity is `1.8276944379471094`, which MCP re-read and
  emitted as `…096`. Cases: that Manning call (SI and ft/s), plus a 17-digit input that must come back bit-exact
  (`sum_backsights` of `[1.8276944379471094, 0.0]`, exact on every platform).
- **Shown to catch it:** with `float_roundtrip` removed from both workspaces and all three interfaces rebuilt, parity
  failed 15 times (both rounding cases in both eras, the input case, and MCP on 5 reference cases: Manning ×2, beam,
  rise-and-fall, traverse). Restored; green.
- **CI:** a `parity` job (Ubuntu, Windows, macOS) added to `python.yml`, written to `_move_to_dot_github/` for the
  maintainer to move.

### 2026-11-27 00:24 IST — Phase 3: the MCP server (`integrations/mcp`), tests first *(Claude Sonnet 5.5)*
- The maintainer reported the Python workflow green (24/24 jobs) after `929c052` and asked for the MCP server next.
- **Mistake in `929c052`:** `bindings/python/python/civkit/_civkit.pdb` (Windows debug symbols left by `maturin
  develop`) was committed, because the AI did not list untracked files inside `bindings/` before giving the add
  command. `.gitignore` now covers `*.pdb`, `*.so`, `*.dylib`; the file is untracked in the next commit (history
  keeps it, no force push). **Rule:** before giving `git add` for a new directory, list its untracked files.
- **G2 first** (`tests/contract.rs`, red then green): `check_descriptor` refuses IDs over 128 characters
  (`MAX_ID_LEN`, the MCP tool-name limit) and the input names `version` and `output_units`
  (`RESERVED_INPUT_NAMES`, control arguments of Python `call` and the MCP tools); AI addition, recorded in
  API_DESIGN §10.1.
- **Stdio tests before the server** (`integrations/mcp/tests/stdio.rs`, 8 tests, all red against an empty binary):
  the real binary in both eras: 2025-11-25 `initialize`, and 2026-07-28 `_meta` per request plus
  `server/discover`. Every reference case through its tool and through `civkit_call` equals `call_json`
  (structuredContent and the text block); contract errors (single, several, list index, bad choice, unknown output,
  unknown version/calculation, malformed request) keep `isError` and the envelope; unknown tool = -32602;
  `--tools`; stdout carries only JSON-RPC; exit when stdin closes.
- **Server:** rmcp `=3.12.0` (`server`, `transport-io`), handler written directly (no macros), one current-thread
  tokio runtime. rmcp picks the lifecycle from the first message; stdin closing before any lifecycle (only a
  discover probe) is treated as a normal exit (found by a test). Schemas from descriptors: inputs
  `{value, unit, negative?}`, `unit` required for dimensional inputs, lists with min/max items, choices as `enum`;
  `outputSchema` = `oneOf` [response, envelope]. Schemas guide only; the engine validates.
- **Mutation check** (11 mutants): 10 killed, the survivor (`unit` not required in the schema) got a test; 11/11.
- `tools/check_dependency_rule.py` rule 3: no HTTP/TLS/WebSocket/SSE crate in `civkit-mcp`'s tree (shown to fail
  when a present crate is forbidden). Licences of the new crates all have an allowed option in `deny.toml`
  (cargo-deny is not installed locally; CI runs it).
- Root workspace: fmt, clippy clean; 280 tests; `civkit-mcp` builds on Rust 1.98.
- **Interoperability (maintainer approved both downloads; scratchpad only, not in the repo):**
  - **MCP Inspector 2.16.0** CLI (`--config` read-only file, `--protocol-era legacy|modern`): tools/list with
    `--strict`, a scalar, a list, a choice call and a contract error, against the debug and release binaries.
    `--strict` flagged a `type` array (`["number","string","array"]`) on angle inputs as unportable (some
    clients accept one type only): now `anyOf`, and a test asserts every `type` in every schema is one string.
    Run it with `node <inspector>/clients/launcher/build/index.js`: the Windows `.cmd` shim passes arguments
    through cmd.exe, which eats `^`, so `"m^2"` arrived as `"m2"` (`UNKNOWN_UNIT`; a harness problem).
  - **Python MCP SDK 2.3.0** (`initialize` and `discover` sessions): 10 tools, all read-only; scalar, list,
    choice and a list-index error each equal `call_json`; the server process had 0 TCP and 0 UDP endpoints.
    (Windows cannot block one process's network without firewall changes, so endpoints were observed instead.)
- **Bug found by the cross-client comparison, fixed:** serde_json's default float parser is not correctly rounded
  (`1.8276944379471094` parsed as `…096`). The MCP server re-reads `call_json`'s output, so it changed last digits;
  the engine's request parsing could also take inputs one ulp off. The stdio tests missed it because their
  reference parse used the same lossy parser. Fix: `serde_json` feature `float_roundtrip` in the workspace (and
  bindings/python). Test first: `json_numbers_are_parsed_exactly` (contract.rs; known values + 20 000 random
  bit patterns). All 281 Rust and 38 Python tests still pass; reference tolerances unaffected.
- **Lesson for the Phase 3 parity task:** compare interfaces through a correctly-rounding JSON reader (Python's
  `json`), or across processes, not only inside one Rust process.

### 2026-11-27 00:02 IST — Phase 3: the Python package (`bindings/python`), tests first *(Claude Sonnet 5.5)*
- **Tests first** (red before any package existed), `unittest` only: `tests/test_api.py` (discovery, the input
  forms, results, `.to()`, typed functions), `tests/test_gates.py` (P3 every reference case + error envelopes field
  by field, P4, P5 socket-blocked subprocess), `tests/test_stub.py` (Q-E drift). 38 tests.
- **Rust layer:** `src/lib.rs`, exactly `call_json`, `list_json`, `describe_json`, `version` over a `LazyLock`
  registry. Builds under `unsafe_code = "forbid"` (PyO3 0.30.1 macros are compatible). Its own Cargo workspace,
  excluded from the root (`exclude` in the root Cargo.toml), so root CI never builds PyO3; own `Cargo.lock`.
- **Python layer** (`python/civkit/`): `Q`, `Result`, `CivKitError`, `call`/`list`/`describe`, typed functions as
  `civkit.<domain>.<topic>.<method>` with keyword-only signatures (inputs + `output_units`) and docstrings from the
  descriptor. It only reshapes; the engine validates. A list of `Q` in mixed units is refused with `ValueError`
  (the one Python-side check: the contract has one unit per list). `Q.to()` reruns the request with
  `output_units`; a hand-made `Q` cannot convert. Stub `__init__.pyi` from `tools/gen_stub.py`.
- **Mutation check** (16 mutants of the Python layer): 13 killed at first; the 3 survivors exposed gaps (warnings,
  `.to()` beyond index 0, tuple values in the request), tests added, now 16/16.
- **Defect fixed in the engine:** `calcs::NO_VALIDITY_RANGE` held a literal `
` and five spaces (a broken line
  continuation), so every descriptor's D-24 statement differed from ADR 0012's sentence. The validity-basis test now
  checks the exact sentence (red first, then fixed). Root workspace: fmt, clippy and 270 tests green.
- **P1 locally (Windows x86-64 only):** release abi3 wheel built, installed with `pip --no-index` into a fresh venv,
  offline check and all 38 tests pass against the installed copy.
- **CI:** `python.yml` written to `_move_to_dot_github/workflows/` for the maintainer to move (AI tools cannot write
  `.github/`): fmt/clippy/MSRV and cargo-deny for `bindings/python` (closing the cargo-deny gap), and 13 wheel jobs
  (5 targets; 3.11–3.15 on Linux x86-64, 3.11 and 3.15 elsewhere): build, offline install (Linux in a network
  namespace), offline check, full test suite. New pins: setup-python v7.1.0 `6d4fd22b`, maturin-action v1.51.0
  `e83996d1` (resolved with `git ls-remote`). **Unverified until it runs:** runner labels `macos-15-intel` and
  `ubuntu-24.04-arm`; that current stable Rust still builds for manylinux2014 (glibc 2.17); `sudo unshare --net`.

### 2026-11-26 23:44 IST — D-26 accepted (ADR 0014); Phase 3 preconditions settled *(Claude Sonnet 5.5)*
- The maintainer accepted Q-A … Q-G of PYTHON_BINDINGS_PROPOSAL.md with two clarifications, both applied:
  - **Q-F:** the package makes no network calls and no intentional file writes; Python's own bytecode caches (`.pyc`)
    are excluded, and no zero-write test is required.
  - **P3:** success results compare `Result.to_dict()` with `call_json`; errors compare `CivKitError`'s fields and full
    `.errors` list with the parsed envelope (P4 covers the details).
- Facts now marked verified, citing the official pages the maintainer supplied: Python 3.10 end of life 2026-10-01 and
  3.15 stable (devguide versions page); `abi3-py311` targets the stable ABI from 3.11, free-threaded uses `abi3t`
  (PyO3 0.30.1 feature guide).
- Recorded: [ADR 0014](decisions/0014-python-package.md); DECISIONS (D-26 Accepted, index); API_DESIGN §8.1; IMPLEMENTATION_PLAN and
  ACCEPTANCE_CRITERIA Phase 3 (gates P1–P5).
- **Stale text fixed at the maintainer's request:** the plan and this handover no longer say D-13 or the MCP schemas
  need settling. Phase 3 preconditions are complete; Phase 3 code is next, tests first.

### 2026-11-26 23:34 IST — Python package proposal (D-26), still Proposed *(Claude Sonnet 5.5)*
- The maintainer gave the baseline: CPython 3.11–3.15; PyO3 0.30.1 with `abi3-py311`; maturin 1.16.0; five abi3
  wheels (Windows x86-64, macOS x86-64 and ARM64, Linux x86-64 and ARM64); manylinux2014; free-threaded, PyPy and GraalPy
  deferred; tests across 3.11–3.15 plus offline install and a call on each target; no offline promise from the sdist.
- Checked: PyO3 0.30.1 is the latest (MIT OR Apache-2.0, MSRV 1.85), `abi3-py311` exists and the chain reaches
  `abi3-py315`; `abi3t-py315` exists for a later free-threaded wheel; maturin 1.16.0 and Python 3.14.10 are in `.venv`.
- [PYTHON_BINDINGS_PROPOSAL.md](PYTHON_BINDINGS_PROPOSAL.md) adds: a thin Rust layer (`call_json`, `list_json`,
  `describe_json`, `version` only; everything else plain Python, so parity is structural); `CivKitError` with
  `code`/`input`/`index`/`errors`; typed functions generated at import plus a drift-checked `.pyi`; no runtime
  dependencies; CI build and test matrices; gates P1–P5. Seven questions, Q-A … Q-G. D-26 row added as Proposed.

### 2026-11-26 23:15 IST — D-13 accepted (ADR 0013) *(Claude Sonnet 5.5)*
- The maintainer answered "yes to Q-A through Q-G" of MCP_SDK_PROPOSAL.md and added two implementation gates. Recorded
  as [ADR 0013](decisions/0013-mcp-sdk-and-surface.md): `rmcp` `=3.12.0`, `default-features = false`, `server` +
  `transport-io`; both protocol eras; tool name = calculation ID; full response as `structuredContent` plus text;
  contract errors as `isError` with the envelope; `civkit-mcp` in `integrations/mcp`.
- Gates (the server is not done until all pass in CI): **G1** both eras over real stdio; **G2** `check_descriptor`
  enforces MCP tool-name rules (128 characters, allowed set), tests first; **G3** round trip equal to `call_json`,
  errors included, in both eras. Written into API_DESIGN §10.1, IMPLEMENTATION_PLAN Phase 3, ACCEPTANCE_CRITERIA Phase 3.
- Next Phase 3 precondition: propose the Python side (supported versions, wheel approach) before any code.

### 2026-11-26 23:05 IST — D-13 proposal written; still Proposed *(Claude Sonnet 5.5)*
- Checked first: `813b26b` (the maintainer's plan update: D6 and the engineering review move to a bulk review after
  Phase 4, before 0.1) and `87fa3d2` are pushed; CI #11 and #12 green on all nine jobs (including macOS stable);
  `phase-2-done` is on GitHub at `87fa3d2`.
- [MCP_SDK_PROPOSAL.md](MCP_SDK_PROPOSAL.md) for D-13, from the MCP specification 2026-07-28 pages and the `rmcp`
  manifests and releases (and `cargo info rmcp`; nothing added to the workspace):
  - `rmcp` 3.12.0 (released 2026-11-26), Apache-2.0, MSRV 1.88; HTTP features optional and off by default.
  - Recommended: `=3.12.0`, `default-features = false`, `server` + `transport-io`, no `macros` (tools are generated at
    run time from the registry).
  - **Finding:** the 2026-07-28 revision removed the `initialize` handshake; a modern-only server fails with
    2025-11-25 clients. Recommend dual-era support, tested with one client of each era.
  - CivKit IDs are valid MCP tool names as they are (dots allowed). Mapping: full contract response as
    `structuredContent` plus text; contract errors as `isError: true` with the envelope; `outputSchema` = `oneOf`.
  - Seven questions, Q-A … Q-G. D-13 row in DECISIONS.md points to the proposal and stays Proposed.

### 2026-11-26 22:56 IST — Review timing moved to the 0.1 bulk gate *(OpenAI Codex; GPT-6, runtime model ID unavailable)*
- The maintainer will complete Phase 3 and Phase 4 code before the human review session. D6 code review and the separate C1–C9 engineering review are bundled after Phase 4 and before 0.1; generated documentation and Phase 4 DXF outputs are included.
- Phase 3 begins with the D-13 and interface preconditions; D6 no longer blocks Phase 3 coding. D6 remains open until a human signs the review record.
- CI run #11 is green on 87fa3d2 and the phase-2-done tag is on GitHub. Other code still needed for 0.1 is in Phase 4.

### 2026-11-26 22:34 IST — Pushing resumed *(Claude Sonnet 5.5)*
- The maintainer committed the D6 guide as `9eb22a3` and resumed pushing: "actually this time we are gonna commit to
  github". Working rule 6 and the next actions updated; `762c70d`, `ed3d20f`, `9eb22a3` and this note go up together.
- After CI is green on the pushed `main`: create and push the annotated tag `phase-2-done` (held until now for CI).
- Unchanged order of work: D6 human review first ([CODE_REVIEW_D6.md](CODE_REVIEW_D6.md)), then the Phase 3
  preconditions starting with D-13, which stays Proposed until the MCP SDK is pinned.

### 2026-11-26 22:24 IST — D6 review guide written; Phase 3 preconditions recorded *(Claude Sonnet 5.5)*
- The maintainer committed `ed3d20f` (D2, tag held); `main` is two commits ahead of `origin/main`; pushing stays paused.
- The maintainer supplied the D6 review plan and the Phase 3 approach. Recorded:
  - [CODE_REVIEW_D6.md](CODE_REVIEW_D6.md): reading order, `registry.rs` and `levelling.rs` sections with line anchors at
    `ed3d20f`, five specific questions (non-finite values from unit conversion; whether the 10 000 limit applies before
    list processing; descriptor name validation; errors reported together; request-triggered panics), a severity scale, a
    findings table and a sign-off block. The review itself is not done: D6 is open until a human signs it.
  - IMPLEMENTATION_PLAN Phase 3 gains a "before any Phase 3 code" list (D6 first; D-13 stays Proposed until the SDK is
    pinned against MCP 2026-07-28; thin interfaces over `Registry::call_json`; what to settle and propose; checks to build).
- Documentation only; no code changed.

### 2026-11-26 21:09 IST — D2 settled; the phase tag waits for CI *(Claude Sonnet 5.5)*
- The maintainer committed D-24 and D-25 as `762c70d` (`main` is one commit ahead of `origin/main`) and **did not create
  the tag**: `phase-2-done` is held until CI has checked the latest commit, because the green run on `948a49a` predates
  D-24 and D-25. Pushing stays paused. ACCEPTANCE_CRITERIA, IMPLEMENTATION_PLAN, ROADMAP and PHASE_2D_REVIEW updated.
- **D2:** `CONTRIBUTING.md` now says Phase 2 is complete and Phase 3 is next; the policy sentence ("Code contributions are
  not open yet; …") is kept exactly as written.
- Maintainer's order of work: commit locally (no push); hold the tag; arrange the human code review of `registry.rs` and
  `levelling.rs` before 0.1 (D6); `cargo-deny` locally is optional (D5); then Phase 3 planning, starting with D-13.

### 2026-11-26 20:58 IST — D-24 and D-25 decided; Phase 2 closed; pushing paused again *(Claude Sonnet 5.5)*
- The maintainer answered D3 and D4 of PHASE_2D_REVIEW.md and asked for D1's tag. Recorded as **D-24** and **D-25** in
  [ADR 0012](decisions/0012-validity-basis-and-beam-scope.md) (register and index in DECISIONS.md):
  - **D-24:** mathematical domain checks always apply; a validity range is documented only with a reliable basis (a cited
    source); otherwise the calculation says so; nothing is invented. ENGINEERING_STANDARDS §1 and §2 item 4 and
    ACCEPTANCE_CRITERIA §3 reworded. All seven descriptors now carry the statement through one constant,
    `calcs::NO_VALIDITY_RANGE`; the new test `every_calculation_states_its_validity_basis` was red for exactly those
    seven before the change. No result changed, so no version changed.
  - **D-25:** the beam's maximum shear and moment are deferred from 0.1; the search continues; they return as new outputs.
    The beam's summary cites it. MVP_SCOPE and ADR 0009 are not edited (ADR 0009 already foresaw this; ADR 0012 records it).
- ACCEPTANCE_CRITERIA Phase 2: "every calculation meets §3" ticked **with three recorded exceptions** (doc pages in
  Phase 3; code review D6; engineering review deferred by the maintainer), as the maintainer chose to close Phase 2.
  The tag box stays unticked until the tag exists.
- **Pushing is paused again** (20:56 IST). Memory and working rule 6 updated; no push or tag push in any instruction.
- Checks: fmt, clippy `-D warnings`, all tests, `cargo doc`, dependency rule (local only; CI last ran on `948a49a`).
- Commit and the local tag: the maintainer runs them from the commands given in the session.

### 2026-11-26 20:51 IST — Pushed; CI green on everything *(Claude Sonnet 5.5)*
- The maintainer committed the levelling, earthwork and 2D work as `948a49a` and pushed (the push pause was lifted the
  same day). CI run 45685894322: **all nine jobs green**: rustfmt; tests on Windows, macOS, Linux × Rust 1.98 and stable;
  dependency rule and cargo-deny; DCO. Findings F1 (no CI evidence) and F10 (advisories unchecked) are resolved.
- Remaining before `phase-2-done`: the maintainer's decisions D1 … D6 in PHASE_2D_REVIEW.md. Working tree clean apart
  from the untracked `planning/ui-concepts/`.

### 2026-11-26 00:55 IST — Phase 2D review done locally; new rule: no GitHub push *(Claude Sonnet 5.5)*
- **New maintainer rule: no push to GitHub for a while.** Saved as a memory and written into working rule 6 above. CI
  has therefore not run on anything after `47a5f25` (GitHub read-only query: six green runs, the last for `47a5f25`);
  `main` is four commits ahead of `origin/main`, and levelling, earthwork and these 2D changes are uncommitted.
- Review record: [PHASE_2D_REVIEW.md](PHASE_2D_REVIEW.md): the checks (all local, 269 tests), a per-calculation matrix
  against ACCEPTANCE_CRITERIA §3, eleven findings and six decisions for the maintainer (D1 … D6). **It does not close
  Phase 2** (cross-platform CI cannot be shown; code and engineering reviews are outstanding), so `phase-2-done` is not created.
- **Fixed in the working tree:**
  - the trace unit blind spot (a wrong but valid unit on a trace step passed; the beam test only counted steps) is closed
    in all seven calculations;
  - Manning's unit tests used y = 1, where the area formula's slope term is invisible (new test with y = 0.5);
  - the demo template's descriptor listed one of its three cases (new integrity test: every descriptor's `examples` and
    the case files on disk must be the same set);
  - descriptor text and `verification` records are now machine-checked;
  - `tests/json_surface.rs` (8 tests) pins the JSON shapes of the list and choice calculations, the path Python, MCP and the C ABI will use;
  - stale status text in README, ROADMAP, the plan and the Phase 1 tag box.
- **Mutation check for the two older calculations** (Manning 7, beam 10 planted errors): all caught.
- Licence audit of the lockfile by script: 27 third-party packages, none outside `deny.toml`'s allow list, no banned
  crate. `cargo-deny` itself is not installed here, so advisories were not checked.
- Left for the maintainer: `CONTRIBUTING.md` still says "Phase 0 — Planning. Code contributions are not open yet" (policy,
  D2); wording of "validity limits" (D3); the beam's maximum shear and moment (D4); optional local `cargo-deny` (D5); a human code review (D6).
- **Uncommitted:** everything in the 00:40 entry plus `crates/civkit-core/src/calcs/{demo,fluids,structural}.rs`,
  `crates/civkit-core/tests/{reference_cases,json_surface}.rs`, `planning/PHASE_2D_REVIEW.md`, `planning/ACCEPTANCE_CRITERIA.md`,
  `ROADMAP.md`, `README.md`.

### 2026-11-26 00:40 IST — Earthwork implemented, tests first; the five D-20 families are complete; nothing committed *(Claude Sonnet 5.5)*
- The maintainer accepted all seven answers (Q-A … Q-G). `quantities.earthwork.average_end_area` v1 is in
  `crates/civkit-core/src/calcs/quantities.rs`, registered in `Registry::with_builtins`. It is the first calculation with a
  **choice input**: `method` is required, has no default and is the first trace step (D-23, ADR 0009 condition 3).
- **Tests first** (25 in `tests/earthwork.rs`, red before the code; plus 5 unit tests next to the code): registered
  descriptor, closed forms (constant section, zero end under both methods, cut and fill independent, net sign), C1 and
  C2 through the contract, the choice visible and never silent (the methods differ exactly on the zero-end segments;
  the trace names the formula used per segment), errors naming "station k", units including m³ to yd³, the trace
  (1 + 2(n−1) + 3 steps, all in m³), and five proptest properties.
- **Two reference cases** in `tests/reference/quantities.earthwork.average_end_area/`: C1 (`wikibooks-ex1`) and C2
  (`wikibooks-solution`; `null` for every per-segment value the source does not print; the only two printed are the
  pyramids 83.3 and 25). C1 areas are entered as cut because the source does not say.
- **Exact re-derivation confirms why the method is explicit:** C2 under the pyramid method gives the book's 4 087.5 /
  1 058.3 / 3 029.2; the same data under average end area gives 4 100 / 1 100 / 3 000.
- **Mutation check:** thirteen planted errors. **One survived the first time**: a wrong unit label (`m^2`) on a total
  step. The registry only checks that a trace step's unit is a valid internal unit, not that it is the *right* one,
  and my trace test checked units only on per-item steps. Fixed by asserting every numeric step's unit, and the same
  hardening added to the traverse and levelling trace tests. All thirteen now caught against a verified green baseline.
  (The script also had a PowerShell 5.1 pitfall: a `Σ` in a script file is read through the ANSI codepage, so the string
  was "not found"; the character is now built from its code point.)
- Lints: `!(a > b)` on floats (now `<=`, equivalent because the registry guarantees finite values), and an identity
  `prop_map` in a test strategy. `proptest` regressions files from the mutation runs were deleted (trivial seeds).
- **Checks (Windows, Rust 1.100):** fmt, clippy `-D warnings`, 256 tests, `cargo doc`, dependency rule: pass.
  `cargo-deny` is not installed locally; CI runs it. No new crate and no `Cargo.lock` change.
- **Uncommitted** (together with levelling): `crates/civkit-core/src/calcs/{mod,levelling,quantities}.rs`,
  `crates/civkit-core/tests/{levelling,earthwork,traverse}.rs`, `tests/reference/surveying.levelling.*/`,
  `tests/reference/quantities.earthwork.average_end_area/`,
  `planning/{LEVELLING,EARTHWORK}_DESCRIPTOR_PROPOSAL.md`, `planning/{AI_HANDOVER,IMPLEMENTATION_PLAN,REFERENCE_CASES}.md`, `README.md`.
- Not done: 2D; the beam's maximum shear and moment; Python or MCP exposure (Phase 3).

### 2026-11-26 00:25 IST — Earthwork descriptor proposed; no code *(Claude Sonnet 5.5)*
- [EARTHWORK_DESCRIPTOR_PROPOSAL.md](EARTHWORK_DESCRIPTOR_PROPOSAL.md) written for review (D-20 condition 4); seven
  questions, Q-A … Q-G. It is the last 2C calculation and the first with a **choice input** (`method`: `average_end_area`
  or `pyramid_at_zero_end`, required, recorded as the first trace step per D-23). ID `quantities.earthwork.average_end_area`.
- **Found by exact rational re-derivation of C1 and C2:** C1 gives 2 050, 1 525, 1 325 = 4 900 m³; C2 under the pyramid
  method gives fill 4 087.5, cut 1 058.333, net fill 3 029.167 (the book's numbers). **C2's data under plain average end
  area gives fill 4 100, cut 1 100, net 3 000**: wrong by 12.5 and 41.7 m³ with nothing in the result to say so. That is
  why ADR 0009 condition 3 demands an explicit method, and it becomes a unit test.
- The pyramid rule is per kind (cut and fill separately) and per segment, and applies only where **exactly one** end area
  is zero. Chainage ordering is checked by the calculation (the contract has no cross-element rule). `net_fill` = fill −
  cut follows the source's sign (Q-D).
- Levelling is implemented but still uncommitted (see the 00:20 entry); one commit is intended to carry the levelling
  work and this proposal.
- Open: the maintainer's answers. No earthwork code until then.

### 2026-11-26 00:20 IST — Levelling implemented, tests first; nothing committed *(Claude Sonnet 5.5)*
- The maintainer accepted all seven answers (Q-A … Q-G) and committed the traverse (`ddecbfb`). The proposal's status
  line records this.
- Three calculations in `crates/civkit-core/src/calcs/levelling.rs` (a new file rather than `surveying.rs`, noted in the
  proposal; the IDs stay in the `surveying` domain), registered in `Registry::with_builtins`:
  `surveying.levelling.height_of_instrument`, `.rise_and_fall`, `.closure_adjustment`. Backsight/foresight pairs only;
  no intermediate sights (D-20).
- **Tests first** (28 in `tests/levelling.rs`, red before the code): registered descriptors, closed forms, units, errors
  naming "setup k" and "point k", overflow, trace counts (2n+3, 3n+3, 2n+2) with structured items, a reduction chained
  into the adjustment, and seven proptest properties (the two reductions always agree, last elevation = start + net
  rise, rises and falls never negative, start shift, adjusted run ends on the known elevation, corrections proportional
  to cumulative length, offset and scale invariance). Seven unit tests next to the code check the book's printed numbers
  and the exact rational values.
- **Five reference cases** in `tests/reference/surveying.levelling.*`: HI for C4 and C9, rise-and-fall for C4 and C9,
  closure and adjustment for C4. Tolerance 0.0005 ft (half the last printed digit; C9: 0.005). The rise-and-fall titles
  and `verification` texts say **consistency, not independent validation** (D-20 condition 2). The adjustment case has
  `null` for the four adjusted values the book does not print. C9 uses backsight 11.56 with the erratum recorded.
- **Mutation check:** thirteen planted errors (sign slips, swapped rise and fall, count instead of distance, wrong
  cumulative length, removed overflow check, unbounded domain, wrong item label …), each caught; the script verified a
  green baseline first and the rerun used a corrected compile-error detector (my first detector matched cargo's
  "error: test failed" line and labelled everything a compile error; fixed before trusting the result).
- **Lints caught three things the tests could not:** a too-complex tuple type (now `RiseFallLists`), a no-op
  `..Default::default()`, and a strict `f64` equality in a property test (now a tolerance).
- `proptest` saved a regressions file during the mutation runs; deleted (trivial seeds that only planted bugs fail).
- **Checks (Windows, Rust 1.100):** fmt, clippy `-D warnings`, 226 tests, `cargo doc`, dependency rule: pass.
  `cargo-deny` is not installed locally; CI runs it. No new crate and no `Cargo.lock` change this time.
- **Uncommitted:** `crates/civkit-core/src/calcs/{mod,levelling}.rs`, `crates/civkit-core/tests/levelling.rs`,
  `tests/reference/surveying.levelling.height_of_instrument/`, `.rise_and_fall/`, `.closure_adjustment/`,
  `planning/{LEVELLING_DESCRIPTOR_PROPOSAL,AI_HANDOVER,IMPLEMENTATION_PLAN,REFERENCE_CASES}.md`.
- Not done: earthwork (the last 2C calculation); Python or MCP exposure (Phase 3).

### 2026-11-26 00:05 IST — Levelling descriptors proposed; no code *(Claude Sonnet 5.5)*
- [LEVELLING_DESCRIPTOR_PROPOSAL.md](LEVELLING_DESCRIPTOR_PROPOSAL.md) written for review (D-20 condition 4); seven
  questions, Q-A … Q-G. Traverse is committed (`ddecbfb`, Windows run passed).
- **Recommended structure (Q-A, option B): three calculations**, `surveying.levelling.height_of_instrument`,
  `.rise_and_fall` and `.closure_adjustment`. Reductions have fixed outputs and need no closing elevation (so C9 is
  natural); the adjustment takes any reduced elevations, `section_lengths` and a known `closing_elevation`.
- **Found by exact rational re-derivation of C4 and C9:** the HI reduction reproduces every printed number; **rise-and-fall
  gives identical elevations** when a field book has only backsight/foresight pairs (no example has an intermediate
  sight, so 0.1 has none: Q-B); adjusted TBM 16 = 134.84792 (book 134.848), the last adjusted point equals the known
  elevation exactly; with the printed C9 backsight 11.65 the result would be 375.24, which confirms the 11.56 erratum.
- The book's permissible closure (0.05·√M, a class constant in miles) is not in D-20 and is omitted (Q-C).
- Open: the maintainer's answers. No levelling code until then.

### 2026-11-26 00:00 IST — Traverse implemented, tests first; nothing committed *(Claude Sonnet 5.5)*
- The maintainer accepted all seven answers (Q-A … Q-G) and committed D-23 (`e948bc6`). The proposal's status line
  records this; its descriptor is now the code.
- `surveying.traverse.compass_rule` v1 in `crates/civkit-core/src/calcs/surveying.rs`, registered in
  `Registry::with_builtins`. First calculation with list inputs and outputs; `civkit-core` now depends on
  `civkit-geometry` (engine-internal; `Cargo.lock` gains only the two edges `civkit-geometry` and the dev-dependency
  `proptest`, already in the lock).
- **Tests first** (23 in `tests/traverse.rs`, red before the code): registered descriptor, closed forms (square,
  one short leg, sign, lat/dep not swapped), units and bearing notation, errors with `index` and "leg k", overflow,
  the trace (6n + 5 steps, structured items, conversion steps), and four proptest properties (adjusted traverse closes,
  misclosure parts, cyclic shift of the legs, scaling). Seven unit tests next to the code apply the compass rule to the
  book's **own printed** latitudes, departures and distances: worst difference 0.00515 ft.
- **Reference case** `tests/reference/surveying.traverse.compass_rule/navedtra-14070-fig7-12.json` (C3): 0.01 ft
  tolerance (the book's printed precision, no widening), `null` for CD departure and CD adjusted departure (the source
  erratum), the book's misclosure figures not asserted; the exact values are in its `verification` text.
- **Mutation check:** ten planted errors (missing minus, lat/dep swapped, wrong perimeter, |M| sum instead of hypot,
  equal shares, subtracted correction, halved ratio, wrong share denominator, removed overflow check, wrong sum), each
  caught.
  - Slip found and fixed: the first mutation run was not meaningful, because `scalar_calculations_keep_their_exact_json_shape`
    (D-23) iterated every built-in and failed on traverse's legitimate list fields. It now skips calculations with
    groups. The rerun, with a green baseline, caught all ten, the lat/dep swap by traverse-specific tests.
  - `proptest` also saved a regressions file during the mutation runs; deleted (trivial seeds that only a planted bug
    fails; the correct code passes all properties).
- **Checks (Windows, Rust 1.100):** fmt, clippy `-D warnings`, 191 tests, `cargo doc`, dependency rule: pass.
  `cargo-deny` is not installed locally; CI runs it.
- **Uncommitted:** `Cargo.toml`, `Cargo.lock`, `crates/civkit-core/Cargo.toml`, `crates/civkit-core/src/calcs/{mod,surveying}.rs`,
  `crates/civkit-core/tests/{traverse,contract_lists}.rs`, `tests/reference/surveying.traverse.compass_rule/`,
  `planning/{TRAVERSE_DESCRIPTOR_PROPOSAL,AI_HANDOVER,IMPLEMENTATION_PLAN,REFERENCE_CASES}.md`.
- Not done: levelling and earthwork; Python or MCP exposure (Phase 3); human engineering review of C1–C9 (deferred by
  the maintainer).

### 2026-11-25 23:52 IST — Traverse descriptor proposed; no code *(Claude Sonnet 5.5)*
- The maintainer committed D-23 as `e948bc6` (Windows run passed).
- [TRAVERSE_DESCRIPTOR_PROPOSAL.md](TRAVERSE_DESCRIPTOR_PROPOSAL.md) written for review (D-20 condition 4): ID
  `surveying.traverse.compass_rule`; group `legs` (min 3); list inputs `bearings` and `distances`; 6 list and 5 scalar
  outputs; no choice input; no validity rules; trace plan; test plan; seven questions (Q-A … Q-G, section 9).
- **C3 re-derived independently** (70-digit decimal arithmetic, own sin/cos series, no packages): every entry is within
  0.0092 ft of the book except the two touched by the CD departure erratum (0.0154 and 0.0118 ft). The book's
  misclosures inherit that erratum: exact E = 2.2869 ft, 1 : 1 143 (book 2.26 ft, 1 : 1 157). The compass rule applied to
  the book's own latitudes and departures matches the book to 0.00515 ft. Proposed reference case: tolerance 0.01 ft,
  `null` for the two erratum entries, the book's misclosure figures not asserted.
- Found: `civkit-core` does not depend on `civkit-geometry` yet; traverse would add that engine-internal edge (Q-D).
- Open: the maintainer's answers to Q-A … Q-G. No traverse code until then.

### 2026-11-25 23:48 IST — D-23 implementation finished and green; nothing committed *(Claude Sonnet 5.5)*
- The working tree held a half-done D-23 extension (the previous Opus session wrote ADR 0011, the new descriptor and
  contract types, a large registry change and `tests/contract_lists.rs`, but not the callers; the 21:50 log entry said
  "no code changed yet"). `cargo test` did not compile: 14 library and 25 test errors.
- Fixed, with the registry design left as written:
  - the three built-in descriptors and two test fixtures now use `..Default::default()` for the new fields
    (scalar input and output, no groups); two registry slips (`&&str`, and the bearing trace step needed `Some(value)`,
    `item` and `choice`);
  - tests that read `outputs[..].value` as `f64` now use `Number::as_scalar()`.
- **Two bugs were in the new test file, not the engine:** the test-only `spacing` calculation produced 4 marks for an
  output declared "points + 1" (the expected `[0, 2, 5, 9, 9]` shows the intent, so it now closes with the total); and
  the "scalar JSON shape" test searched for the substring `"length"`, which every dimension serialises, so it now
  checks descriptor fields structurally.
- **Reference-case harness (tests first):** list expectations with `null` entries per proposal §3.7; mismatch messages
  name the entry ("segment 2: got …"); unit checked once; at least one entry must be checked. 6 new tests.
- **Docs:** API_DESIGN §2.1 (new), §3, §5, §6, §7, §8, §11 and `tests/reference/README.md`, as ADR 0011 promised. The
  descriptor rules written there were checked against `check_descriptor`.
- **Checks (Windows, Rust 1.100, run from the working folder):** fmt, clippy `-D warnings`, 161 tests, `cargo doc`,
  dependency rule: all pass. `cargo-deny` is not installed on this machine; CI runs it.
- **Uncommitted files:** `crates/civkit-core/src/{contract,descriptor,error,lib,registry}.rs`,
  `src/calcs/{demo,fluids,structural}.rs`, `tests/{contract,reference_cases,contract_lists}.rs`,
  `docs/API_DESIGN.md`, `tests/reference/README.md`, `planning/{CONTRACT_LISTS_PROPOSAL,DECISIONS,IMPLEMENTATION_PLAN,AI_HANDOVER}.md`,
  `planning/decisions/0011-list-and-choice-inputs.md`. `planning/ui-concepts/` stays unstaged.
- Not done: no traverse, levelling or earthwork code; no new reference cases with list outputs yet (the harness is
  ready for them).

### 2026-11-25 23:07 IST — Competitive research; two reports, no decisions *(Claude Sonnet 5.5)*
- Research-only task from the maintainer. New files: [COMPETITIVE_LANDSCAPE.md](COMPETITIVE_LANDSCAPE.md) (11 sections) and
  [FEATURE_PROPOSALS.md](FEATURE_PROPOSALS.md) (28 proposals FP-01 … FP-28; 12 rejected or deferred, R-1 … R-12).
  No code, accepted decision or MVP_SCOPE changed; 0.1 stays as planned.
- Headline: "Rust + offline + MCP" is now common among new competitors. The open lane is a verifiable calculation layer:
  a replayable calculation record (FP-01), a verification ledger (FP-02), cross-checks (FP-03), a local hand-calculation
  checker (FP-04), and a conformance suite and CLI (FP-21, FP-10).
- **Risk found:** CivilKit (civilkit.io; Calcs, plus a CAD kernel under Apache-2.0) is two letters from CivKit, same
  language and pitch, active in Nov 2026. D-1 only recorded the Bitcoin and npm overlaps. Trademark registers and search
  results could not be checked. Landscape §10 lists risks and optional actions; D-1 is not reopened.
- **Proposals needing a decision record (none written):** record schema and MCP surface are cheapest to decide *before*
  Phase 3 parity tests; MCP spec has a 2026-07-28 revision, so re-read D-13 at Phase 3 start. `compute` is `f64`-only,
  so sensitivity outputs (FP-05) would be the most expensive change; prototype first, do not change the trait now.
  Full list: FEATURE_PROPOSALS §4.5.
- Licence rules reinforced: never port from GPL/AGPL/LGPL/OpenSees/Symbolica; use them only as test oracles.
  `deny.toml` already blocks `rug` (LGPL) and `acadrust` (MPL-2.0); LandXML schema licence is **unverified**.
- Limits: ~25 projects, no commercial tools studied; some pages only partly read; vendor test counts are self-reported.
  Human review needed for the licence and naming sections.
- **Timestamp trap:** on this machine `TZ=Asia/Kolkata date` in Git Bash prints UTC labelled "IST" (5 h 30 min behind;
  16:36 when the real time was 22:16). Stamp from PowerShell:
  `[TimeZoneInfo]::ConvertTimeBySystemTimeZoneId([DateTime]::UtcNow,'UTC','India Standard Time')`.
  The §2 rule 3 example command is wrong for Git Bash on Windows.
- Maintainer: the two reports are untracked; commit with `git commit -s -F _commit_msg.txt` when ready
  (`planning/ui-concepts/` stays unstaged).

### 2026-11-25 21:50 IST — D-23 design confirmed; four edge rules clarified *(Claude Opus 5.5)*
- Maintainer's Windows run passed. Pushed as `e842edc`. That commit holds only the four D-23 documents; its message
  also re-describes the 2B follow-up and 2C part 1, which were already committed in `a49b5e8`. History left as is.
- The maintainer confirmed design §3 and asked for four edge rules before any contract change. Now written:
  - every list input is required in 0.1, so a group's length is the common length of its lists;
  - the 10 000 cap covers outputs: a group's maximum is 10 000 minus its largest positive output offset, and an
    over-long or wrongly sized output is `INTERNAL`;
  - `check_descriptor` validates list outputs (known group, offset, never empty, unique item labels);
  - exactly one choice step per choice input, before numeric steps and in declaration order; any violation is
    `INTERNAL`;
  - structured trace `item` (label, 0-based index), with the text prefix generated from it and checked.
- No code changed yet.

### 2026-11-25 21:35 IST — D-23 direction decided; full design written *(Claude Opus 5.5)*
- Maintainer reviewed the Manning and beam descriptors: no scope or formula mismatch with D-20. Asked that the
  beam's conditional `point_load_position` stay prominent in generated forms and docs (Phase 3 task recorded).
- D-23 direction decided by the maintainer:
  - option A (parallel lists, one unit per list);
  - required choice inputs, recorded in the trace;
  - "array (vectorised) inputs" means Python batching.
- The maintainer asked that list-output lengths be settled before implementation. The proposal §3 now has the
  full design:
  - groups with item labels;
  - list outputs with length = group + offset (−1/0/+1);
  - 1-based item numbering in messages and traces, plus a 0-based `index` field in errors;
  - `choice` trace steps;
  - `null` elements in reference cases;
  - a 10 000-element limit.
- Correction: the "array inputs" sentence is in API_DESIGN §8, not §7 as the first proposal said.
- No code changed. Waiting for confirmation of §3.

### 2026-11-25 19:15 IST — 2C part 1: Manning and beam; contract gap for list inputs *(Claude Opus 5.5)*
- Maintainer confirmed the 2B follow-up locally (127 tests) and said go for 2C.
- `fluids.open_channel.manning_trapezoidal`:
  - area, wetted perimeter, hydraulic radius, velocity and discharge at a known depth (D-20 scope);
  - reference cases HDS-4 Examples 4.2 and 4.1 (C5);
  - five unit tests.
- `structural.beam.simply_supported`:
  - full-span UDL plus one point load; reactions, deflection at x and end slope θ_A;
  - sign conventions as in the sources: downward deflection and clockwise slope are negative;
  - reference cases Udoeyo 7.2 (C6) and Lord 11.1 and 11.5 (C7, C8);
  - nine unit tests.
  - Maximum shear and moment are not included: no qualifying example yet. S4 ch. 7 has none that fits, and its
    Example 7.2 has a moment erratum (50.4 vs 52.02 kip·ft), now recorded.
- **Gap found:** the contract has only scalar inputs. Traverse, levelling and earthwork need lists, and earthwork
  needs a visible method input (ADR 0009 condition 3). Proposed as D-23 with options; not decided, no code.
- Found: the output mirror's network mount does not update file times, so a cargo build there was stale once.
  A clean rebuild confirmed every result. Builds now run from a content-synced copy.
- Local checks (from the synced copy): fmt, clippy -D warnings, 141 tests, dependency rule, docs. All clean.

### 2026-11-25 19:00 IST — 2B confirmed; derived tolerances must be finite *(Claude Opus 5.5)*
- Maintainer confirmed all eight implementation items of ADR 0010. Item 6 (explicit tolerance) was confirmed with
  a caveat: ε_A = ε × D could overflow for a huge finite ε and large geometry, leaving an infinite tolerance and an
  undefined orientation.
- Fix: every tolerance computation now fails with `Overflow` when ε or ε_A is not finite (GEOMETRY.md §5, §8.1).
  This also covers the default, which overflows only for D above about 4e158. Two tests were added first and
  failed before the fix: polygon, centroid, polyline and direction cases.
- Maintainer's Windows run passed on `8a93d58` (Rust 1.100). CI green on `8a93d58` (run #5) and `6d3d16a` (run #4),
  including the Rust 1.98 MSRV jobs.
- Local checks after the fix: fmt, clippy -D warnings, 127 tests, dependency rule. All clean.

### 2026-11-25 18:37 IST — 2B geometry: review, acceptance, implementation *(Claude Opus 5.5)*
- Review-only task: [GEOMETRY_SPEC_REVIEW.md](GEOMETRY_SPEC_REVIEW.md). Every formula was re-derived in separate
  Python code against 60-digit arithmetic and shapely. Result: 0 Critical, 9 Major, 13 Minor findings, and D7–D9.
  The same model wrote the spec and the review; independence came from the numerical checks, not a second reviewer.
- Maintainer accepted everything (18:10 IST). GEOMETRY.md revised and marked Accepted; D-22 /
  [ADR 0010](decisions/0010-geometry-conventions.md); ARCHITECTURE §3.3 amended; IMPLEMENTATION_PLAN updated
  (2B status; Phase 4 inherits transformations, OCS and the contract side of geometry notes).
- Found while implementing (not caught by the review): with "|A| ≤ ε_A" as the degeneracy rule, a collinear ring
  could never reach 8.6 and a figure-8 with cancelling lobes would have passed as "zero area". 8.6 is now
  geometric and evaluated before 8.7. This and seven smaller points await confirmation (ADR 0010).
- `civkit-geometry` implemented, tests first. Expected values come from closed forms, exact rationals or 60-digit
  values computed outside the repository.
  - 64 tests: 6 unit tests, 11 on angles, 15 on lengths, 11 on areas, 15 on degeneracy, 6 property tests.
  - Mutation check: 11 planted errors (bulge sign, |A| in the centroid, no translation, old g(x), TAU wrap,
    −0, disabled self-intersection, wrong arc wedge, old length formula, area-based 8.6, D without arc
    extremes). All were caught.
- Full local checks: fmt, clippy -D warnings, 125 tests, dependency rule, cargo-deny. All clean (Linux, Rust 1.97
  with `--ignore-rust-version`; the Windows run and CI on 1.98 are the authority).
- Cargo.lock changes only by `proptest` as a dev-dependency of `civkit-geometry` (already used by `civkit-units`).

### 2026-11-25 16:11 IST — 2A verification; 2B geometry specification proposed *(Claude Opus 5.5)*
- Maintainer approved whole-number DMS minutes (fractional seconds) and the three unit roles.
- **Verified in the pushed commit, not the working copy:** cloned github.com/gowtham-sivarajan/civkit at `b4f37e2`.
  - All five restored contract tests exist once each.
  - Four are identical to `05b1637`, ignoring whitespace. The fifth (escalated validity) differs only as intended:
    it now passes `4 m`/`20 m` through real units instead of a dimensionless workaround.
  - The working copy matched the pushed commit before new edits. CI on `b4f37e2` was green.
- Internal units enforced:
  - domain and validity messages name the internal unit (`'length' = 15.24 m … [0, 10] m`)
  - after `compute` the registry rejects trace steps in non-internal units or non-finite values (`INTERNAL`)
  - two new tests; documented in API_DESIGN §7.1.5
- Full local checks: fmt, clippy -D warnings, 61 tests, dependency rule, cargo-deny, docs. All clean (Linux, Rust 1.97).
- 2B: [docs/GEOMETRY.md](../docs/GEOMETRY.md) drafted. Its formulas and figures were checked numerically before
  hand-over; that caught a wrong bulge-area sentence, a wrong ulp figure and a too-coarse tolerance rule, all corrected.
  No geometry code was written.

### 2026-11-25 15:59 IST — Maintainer review of the 2A rules applied *(Claude Opus 5.5)*
- Approved: `ton_short`/`ton_long`; no spaces in unit strings (spaces stay allowed in human-readable bearings).
- Division: only one `/` with a single factor after it; canonical spelling `W*m^-2*K^-1`. The full grammar is now
  written formally (EBNF) in API_DESIGN §7.1, including what is *not* supported (parentheses, prefixes, Unicode
  superscripts), so users never learn rules from error messages.
- Bearings:
  - degrees 0–90, minutes 0–59, seconds 0 ≤ s < 60; 90° only with zero minutes and seconds
  - N/S…E/W combinations only; lower case **rejected, not normalised**
  - all four quadrants and all eight cardinal spellings tested
  - fixed: spaces could split a number (`N 4 5° E` read as 45°); now rejected
- DMS: canonical form; minutes are now whole numbers 0–59 (so `[12, 30.5, 0]` and `[12, 30, 30]` cannot both exist),
  seconds 0 ≤ s < 60; sign only via `negative`; negative zero is zero.
- Unit roles: descriptors now carry `si_unit` (internal, coherent SI without prefix) **and** `display_unit` (default
  presentation, may be e.g. `mm`); requests may override with `output_units`. The trace stays in internal units.
- D-21 proposed (after 0.1): general SI prefixes and Unicode notation. Exactly what 0.1 accepts is listed in §7.1.2.
- Git: my earlier instruction listed two alternative `git add` commands as if consecutive. They are alternatives, and the
  second does not unstage. Nothing had been pushed: `ui-concepts` is not on GitHub.
- My own slip while editing: a slice removed five existing contract tests; caught by the test count and restored.
- 59 tests pass (Linux, Rust 1.97); clippy, fmt, cargo-deny and the dependency rule are clean.

### 2026-11-25 15:39 IST — D-20 accepted; Phase 2A units *(Claude Opus 5.5)*
- Maintainer approved D-20 ([ADR 0009](decisions/0009-implementation-scope-0-1-calculations.md)): 0.1 implements only what
  worked examples verify. The MVP_SCOPE table is unchanged and links to ADR 0009. Phase 2 runs in stages 2A → 2D.
- 2A `civkit-units`:
  - named-unit table from defining relations; compound parser (one `/`, single term after it)
  - ambiguity rejection; DMS and quadrant bearings
  - core integration: real unit conversion, `MISSING_UNIT`/`AMBIGUOUS_UNIT`/`UNIT_MISMATCH`, `output_units`,
    bearing steps in the trace, descriptor unit checks
- Tests:
  - definition relations (mutation-checked: three planted factor errors all caught)
  - rejections; angles (C3 bearings by hand)
  - 7 property tests (2000 cases each); proptest added as a dev-dependency only
  - 56 tests pass; clippy, fmt, cargo-deny and the dependency rule are clean (Linux, Rust 1.97)
- Rules written into [API_DESIGN.md §7.1](../docs/API_DESIGN.md) for maintainer review. Includes one spec fix: `ton_short`/`ton_long`
  added because the ambiguity message for `ton` names them.
- Timestamp slip again: `civkit-units` files were stamped up to 7 minutes ahead and were corrected from file times.
  Stamps are now generated by the shell at write time.
- New folder `planning/ui-concepts/` (Figcraft/HTML/PDF) appeared on the device; it was not created by this session and was left untouched.

### 2026-11-25 11:15 IST — Phase 1 complete *(Claude Opus 5.5)*
- Maintainer ran `cargo test --workspace` on Windows with Rust 1.100: 23/23 pass.
- First CI [run 45555065260](https://github.com/gowtham-sivarajan/civkit/actions/runs/45555065260) on 0e07eed: 9/9 jobs green (fmt; tests on Windows/macOS/Linux × 1.98/stable; clippy; dependency rule;
  cargo-deny; DCO). Dependabot started its first checks.
- Cargo 1.100 warned about unused workspace dependencies; removed (re-added when crates depend on each other).
- Correction: the Phase 1 files and the entry below were stamped 11:00–11:32 IST from estimates; the files were
  actually written 10:54–11:01 IST. Every stamp later than its file's real modification time was reset to that time, across the repository
  (48 files, plus three test JSON files, the Code of Conduct note and four log headings from 10:02 onward).

### 2026-11-25 11:00 IST — Phase 1 code *(Claude Opus 5.5)*
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

### 2026-11-25 10:48 IST — Phase 0 complete; D-10, D-11 accepted *(Claude Opus 5.5)*
- D-10 (2024 edition, MSRV 1.98) and D-11 (DCO, no CLA) accepted → ADR 0008; CONTRIBUTING updated.
- Commit messages: pasted multi-part `git commit -m … -m …` lines were cut off twice in the maintainer's terminal.
  New method: the AI writes a git-ignored `_commit_msg.txt`; the maintainer runs `git commit -s -F _commit_msg.txt`.
- `plan-v1.0` was first pushed on a commit that lacked task 0.9 and the community files (the commit had failed);
  it is re-created on the Phase 0 completion commit.

### 2026-11-25 10:35 IST — GitHub community files *(Claude Opus 5.5)*
- `CODE_OF_CONDUCT.md`: Contributor Covenant 3.0 (CC BY-SA 4.0) copied from source; reporting contact = maintainer's
  commit email (already public in commits; maintainer agreed to a public email).
- `.github/ISSUE_TEMPLATE/`: wrong-result form (treated as the most serious bug), bug report, calculation/feature request
  (asks for an openly licensed worked example, D-18), config (blank issues off; private security link).
- `.github/PULL_REQUEST_TEMPLATE.md` (dependency rule, offline, reference-case and AI-provenance checks); `ACCESSIBILITY.md`.
- Remote AI file tools cannot write inside `.github/` (protected, like `.git/`). Files were written to `_move_to_dot_github/`
  and the maintainer moves them into `.github/`.

### 2026-11-25 10:34 IST — Task 0.9: DXF fixtures *(Claude Opus 5.5)*
- `tests/fixtures/dxf/`: F1 metric site plan, F2 same in mm, F3 planted unit problems, F4 R12 legacy, F5 blocks
  (rotated, stretched, mirrored, MINSERT, nested). F1–F3/F5 drawn in CADCraft; F4 written with ezdxf.
- Two MIT samples from ezdxf (`insert_bricscad_level_1.dxf` from BricsCAD; `Minimal_DXF_AC1009.dxf`), licence file kept with them.
- Expected values fixed by construction and re-checked with ezdxf + shapely (0 failures); script kept outside the repo.
- University folders offered by the maintainer were not opened: not needed, and they may hold personal or coursework data.

### 2026-11-25 10:02 IST — D-19 accepted *(Claude Opus 5.5)*
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
