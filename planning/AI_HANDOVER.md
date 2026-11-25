# AI Handover

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-24 23:41 IST (as HANDOFF.md) · last edited 2026-11-25 21:35 IST

> **Read this first** when picking the project up — the maintainer after a break, a new collaborator, or an AI
> assistant starting a fresh session. **Update it at the end of every working session.**

**Project:** CivKit
**Maintainer:** Gowtham
**Repository:** public on GitHub from day one (D-17) · licence Apache-2.0
**Current phase:** 2 — Units, geometry, calculations (stage 2C)
**Code written so far:** `civkit-core` (contract), `civkit-units` (2A), `civkit-geometry` (2B); two engineering calculations (2C, in progress)

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
7. Never present unverified engineering content as fact; flag it for engineering review.
8. **Protect the engine's independence:** no AutoCAD, Python, network or AI dependency in `crates/` ([ARCHITECTURE.md](../docs/ARCHITECTURE.md)).
9. **Reference cases are mandatory:** known input, documented method, independently verified result, edge-case tests ([ENGINEERING_STANDARDS.md](../docs/ENGINEERING_STANDARDS.md)).

## 3. Where things stand (snapshot 2026-11-25 19:00 IST)

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
  - **2C (calculations)**: 🔄. Done: `fluids.open_channel.manning_trapezoidal` (C5) and
    `structural.beam.simply_supported` (C6–C8: reactions, deflection, end slope). **Blocked on D-23**: traverse,
    levelling and earthwork need list inputs and a method choice. Direction decided (option A, required
    choices, §8 wording); full design in [the proposal](CONTRACT_LISTS_PROPOSAL.md) §3 awaits confirmation. Beam maximum shear and moment: still no qualifying example.

**Decisions**
- All P0 and P1 decisions needed so far are accepted: D-1 … D-7, D-10 … D-12, D-15 … D-20, D-22.
- Proposed: D-8, D-9, D-13, D-14; D-21 (SI prefixes and Unicode units, after 0.1); **D-23 (list and choice inputs, blocks 2C)**.
  See [DECISIONS.md](DECISIONS.md).

**Engine state**
- `civkit-units`: named units, the formal unit grammar (one `/`), DMS (whole minutes) and bearings, all per
  [API_DESIGN.md §7.1](../docs/API_DESIGN.md).
- `civkit-core`: contract with three unit roles (internal SI, default presentation, requested). Internal units are
  enforced in values, trace and messages.
- `civkit-geometry`: angles and direction, tolerance, polyline/arc/circle lengths, polygon area, orientation,
  centroid (straight edges), the self-intersection check with exact arc relations. Every item cites
  [GEOMETRY.md](../docs/GEOMETRY.md). Rust only: no contract calculations (D-22).
- Registered built-ins: `fluids.open_channel.manning_trapezoidal`, `structural.beam.simply_supported`. The template
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

1. **Maintainer:** confirm D-23 design §3 ([CONTRACT_LISTS_PROPOSAL.md](CONTRACT_LISTS_PROPOSAL.md)): groups,
   list-output lengths (offset −1/0/+1), item numbering, `index` in errors, `choice` trace steps, 10 000-element limit.
2. Manning and beam descriptors: reviewed by the maintainer, no mismatch with D-20. Conditional requirements
   (beam `point_load_position`) stay in prose; Phase 3 task recorded.
3. **Maintainer:** Windows check, commit with `git commit -s -F _commit_msg.txt`, push, confirm CI.
4. After D-23: ADR 0011 and the API_DESIGN change, then the contract extension (tests first), then traverse,
   levelling and earthwork one at a time, each with its descriptor proposed for review first (D-20 condition 4).
   Traverse decides for itself when a misclosure counts as zero before asking for its direction (GEOMETRY.md §7).
   Then 2D.
5. Beam maximum shear and moment: still need an openly licensed example with a full-span UDL and one point load
   (S4 ch. 7 and one statics text checked; none fits).
6. **AI sessions:** the output mirror is a network mount that does not update file times on in-place edits, so
   cargo can reuse stale builds. Build from a content-synced local copy.
5. Review Dependabot pull requests as they arrive (each must pass CI).
6. Later: an AutoCAD-written DXF fixture once a licence that allows publishing exists. Phase 4 inherits the
   geometry tasks listed in IMPLEMENTATION_PLAN (transformations, OCS, contract representation of geometry notes).

Deferred by the maintainer: human engineering review of C1–C9 (until the product is complete);
general prismoidal and soil phase relationships (later release).

## 5. Session log

Newest first. One short entry per session: what changed, what was decided, what is next.

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
