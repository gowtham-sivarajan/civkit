# Phase 2D review: integration and regression

> **Provenance:** Claude Sonnet 5.5 (`claude-sonnet-5-5`) · first drafted 2026-11-26 00:52 IST · last edited 2026-11-26 22:56 IST by OpenAI Codex (GPT-6; runtime model ID unavailable)

> **Status:** Phase 2 review record, updated 2026-11-26. CI run #11 passed on commit 87fa3d2; the phase-2-done tag is on GitHub. Phase 2 technical work is complete. Human code review D6 and human engineering review C1–C9 remain open and are scheduled after Phase 4 implementation, before 0.1.
>
> **Evidence boundary:** The checks in this report are local Windows evidence unless marked CI. CI run #11 verifies the latest pushed commit; earlier run details are retained below as history.
>
> **Decision status:** D-24 and D-25 are recorded in ADR 0012. D1 (the Phase 2 tag) is complete; D2 is recorded in CONTRIBUTING; D5 remains optional; D6 is deferred to the bulk review.
>

## 1. The checks

| Check | Result | Evidence |
|---|---|---|
| `cargo fmt --all --check` | pass | local, 2026-11-26 |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | pass | local |
| `cargo test --locked --workspace` | **269 passed, 0 failed** | local |
| `cargo doc --locked --no-deps --workspace` | pass, no warnings | local |
| Dependency rule (`tools/check_dependency_rule.py`) | pass, 5 engine crates | local |
| Licence audit of the lockfile against `deny.toml` | pass: 27 third-party packages, none outside the allow list, no banned crate | local script (`cargo metadata`); **not** `cargo-deny` itself |
| Advisories (`cargo-deny` RustSec check) | **not run locally** (`cargo-deny` is not installed) | CI runs it; last CI run was before the newest commits |
| CI on Windows, macOS, Linux × Rust 1.98 and stable | **last green on `47a5f25` (2026-11-25 16:23 UTC)** | GitHub, read-only query; runs for `6d3d16a`, `8a93d58`, `bc06cb1`, `a49b5e8`, `e842edc`, `47a5f25` all succeeded |
| MSRV (Rust 1.98) | not tested locally (only 1.100 is installed) | CI only |
| Offline operation | every cargo command in this review ran with `--offline` or on a warm cache; no test touches the network; no network crate is in the lockfile | local |

**What CI has not seen.** `main` was four commits ahead of `origin/main` at the start of this review (`e948bc6` D-23
implementation, `58ca580` research, `e394cb8` and `ddecbfb` traverse), and levelling, earthwork and the 2D changes are
uncommitted. So the contract extension, all five calculation families and the new tests have **no CI evidence on macOS,
Linux or Rust 1.98**. They are pure Rust with no platform code, so surprises are unlikely, but "unlikely" is not "shown".

## 2. Calculations against ACCEPTANCE_CRITERIA §3

Seven calculations are registered (five D-20 families). `math.demo.weighted_mean` is the template and is registered only in tests.

| Criterion | Manning | Beam | Traverse | HI | Rise and fall | Closure adj. | Earthwork |
|---|---|---|---|---|---|---|---|
| Descriptor complete (ID, version, dimensions, references, assumptions) | ✅ test-enforced | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| ≥ 1 reference case from an open source (D-18) | ✅ 2 | ✅ 3 | ✅ 1 | ✅ 2 | ✅ 2 *(consistency)* | ✅ 1 | ✅ 2 |
| Independent verification recorded in the case | ✅ | ✅ | ✅ exact 70-digit | ✅ exact rational | ✅ *(labelled consistency)* | ✅ | ✅ exact rational |
| Edge cases: zero, limits, invalid input, unit variants | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Trace present, correct, units and values asserted | ✅ *(fixed in 2D)* | ✅ *(fixed in 2D)* | ✅ *(hardened)* | ✅ | ✅ | ✅ | ✅ |
| Mutation check recorded | ✅ 7 caught *(2D)* | ✅ 10 caught *(2D)* | ✅ 10 caught | ✅ 13 across the levelling set | ✅ | ✅ | ✅ 13 caught |
| Validity rules | none | none | none | none | none | none | none *(see F6)* |
| Generated documentation page | ❌ Phase 3 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Code review (independent) | ❌ none yet | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Engineering review (human) | ⏸ deferred by the maintainer until the product is complete | ⏸ | ⏸ | ⏸ | ⏸ | ⏸ | ⏸ |

Reading: the **technical** criteria are met for all seven. The **review** criteria are not, and cannot be by an AI
session. So "Every 0.1 calculation meets §3" in the Phase 2 exit list stays **unticked**, with a note pointing here.

## 3. Findings

Each finding says what it was, how it was found, and what was done. "Fixed" means a change in the working tree, not yet committed.

| # | Finding | How found | Status |
|---|---|---|---|
| **F1** | **CI evidence ends at `47a5f25`.** The D-23 implementation and the five calculations have not run on macOS, Linux or Rust 1.98. | Read-only GitHub query; `git status -sb` showed `ahead 4` | **Resolved 2026-11-26:** pushed as `948a49a`; CI green on all nine jobs (run 45685894322). |
| **F2** | **Trace tests proved too little.** The registry accepts any *valid* internal unit for a trace step, not the right one. A planted `m^2` label on a volume total survived the earthwork tests; the beam test `trace_is_in_internal_units` asserted only `trace.len() == 7`; Manning's tests only counted steps. | Mutation check | **Fixed** in all seven: every numeric trace step's unit is asserted, and (Manning, beam) its value against the output. Two planted unit errors are now caught. |
| **F3** | **Manning's unit tests used y = 1**, where `(B + z·y)·y` and `(B + z)·y` are equal. The planted error was caught only by the reference case. | Mutation check | **Fixed:** a test with B = 2, z = 2, y = 0.5 from independent geometry (top width, trapezoid area, side length). |
| **F4** | **The demo template's descriptor listed one of its three reference cases.** | New integrity test | **Fixed**, and the test now guards every descriptor: each `examples` path exists, is in its own folder, and every case file on disk is listed. |
| **F5** | **Stale status text:** README said "Phase 0 — Planning"; the ROADMAP said Phase 2 "(next)"; the plan said Phase 0 "(current)"; the 2A and 2B rows were "pending"; the Phase 1 tag box was unticked although the tag exists. | Review | **Fixed.** `CONTRIBUTING.md` line 5 still says "Phase 0 — Planning. Code contributions are not open yet": **left alone**, because it states a contribution policy, which is the maintainer's call (D2). |
| **F6** | **No calculation has a validity rule.** ENGINEERING_STANDARDS §1 and §2 item 4 say "validity limits, enforced as warnings or errors". For traverse, levelling and earthwork the maintainer's accepted answers (Q-C of the traverse and levelling proposals, Q-E of the earthwork proposal) record why none exists: no source states a limit. For Manning and the beam it was never stated. | Review | **Open, wording question** (D3). Mathematical domains *are* enforced (positive depth, positions within the span, and so on). **Decided: D-24** (ADR 0012). All seven descriptors now state that no range is established; a test enforces it. |
| **F7** | **No test pushed a list or choice calculation through `call_json`**, the single path Python, MCP, the C ABI and the desktop app will use. | Review | **Fixed:** `tests/json_surface.rs` (8 tests) pins the JSON shapes: arrays with one unit, structured trace items, the choice step, `index` on element errors only, descriptor JSON for forms and tools, byte-identical repeat calls. |
| **F8** | **Descriptor text and verification records were not machine-checked.** | Review | **Fixed:** two guards. Every descriptor has a title, summary, references, assumptions, examples and a description for every input and output. Every reference case has a title, a provenance line naming a model, and a `verification` text. |
| **F9** | **The beam's maximum shear and moment are still missing.** D-20 allows them only with one more openly licensed worked example, and it allows moving them to a later release if none is found. | Known since 2C | **Open** (D4). **Decided: D-25** (ADR 0012): deferred from 0.1; the search continues. |
| **F10** | **`cargo-deny` is not installed locally**, so advisories (RustSec) were not checked here. The licence and ban rules were checked by script against the lockfile. | Review | **Resolved by CI** (cargo-deny job green on `948a49a`); a local install stays optional (D5). One legacy licence spelling, `MIT/Apache-2.0` (`core_detect`), is how `cargo-deny` itself would read "MIT or Apache-2.0", but it was not run. |
| **F11** | **All code and tests were written by an AI model, with the same model family doing the review.** The independence in this project comes from outside computations (exact rational and 70-digit arithmetic, mutation checks), not from a second reader. | Review | **Open** (D6): a human code review is still missing, as §3 requires. |

## 4. Not verified in 2D

- Anything on macOS, Linux or Rust 1.98 for the new commits (F1).
- RustSec advisories (F10).
- Numerical behaviour of the **units** and **geometry** crates beyond their own tests (they are unchanged since CI last ran).
- The Windows run of levelling, earthwork and the 2D changes by the maintainer (done locally here, but the maintainer's Windows check is the project's own gate).
- Whether the engineering content is *right* in the professional sense. The cases are checked against their sources and
  re-derived independently, which establishes arithmetic fidelity to the source, not engineering judgement
  (the disclaimer in ENGINEERING_STANDARDS §7 applies).

## 5. Decisions for the maintainer

| # | Question | Recommendation |
|---|---|---|
| **D1** | When to create the phase-2-done tag? | **Completed 2026-11-26:** CI run #11 passed on 87fa3d2 and the phase-2-done tag is present on GitHub. |
| **D2** | `CONTRIBUTING.md` still says "Phase 0 — Planning. Code contributions are not open yet." Update the phase name, the policy, or both? | Your call; the policy sentence is yours. **Decided 2026-11-26:** the phase label is updated; the policy sentence is kept as written. |
| **D3** | Validity rules (F6): accept "no stated range, so no rule" as the standard, and say so in ENGINEERING_STANDARDS §1 and ACCEPTANCE §3, or require one per calculation (for example a Manning's n range with a cited source)? | Accept the first, in writing. Inventing ranges would be unverified engineering content. **Decided 2026-11-26: D-24.** |
| **D4** | Beam maximum shear and moment (F9): keep searching for an open example, or move them explicitly to a later release now? | Move them explicitly, and keep searching in the background. The five families are otherwise complete. **Decided 2026-11-26: D-25.** |
| **D5** | Install cargo-deny locally? | Optional; CI runs cargo-deny, so local installation is not a blocker. |
| **D6** | Human code review (F11): who and when? | Before 0.1, bundled at the end of Phase 4 with the separate C1–C9 human engineering review. The reviewer uses [CODE_REVIEW_D6.md](CODE_REVIEW_D6.md); D6 stays open until a human signs it off. |

## 6. What changed in the working tree for 2D

`crates/civkit-core/src/calcs/{demo,fluids,structural}.rs` (descriptor list, two Manning tests, one beam test),
`crates/civkit-core/tests/reference_cases.rs` (three new tests), `crates/civkit-core/tests/json_surface.rs` (new, eight tests), the docs listed in F5, and
this file. No behaviour of any calculation changed.
