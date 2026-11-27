# D6 code review: guide and record

> **Provenance:** Claude Sonnet 5.5 (`claude-sonnet-5-5`) · first drafted 2026-11-26 22:24 IST · last edited 2026-11-27 23:16 IST by Claude Sonnet 5.5 (`claude-sonnet-5-5`)

> **Status:** **Open.** This is the independent human code review that ACCEPTANCE_CRITERIA §3 requires and that the
> Phase 2D review left open ([PHASE_2D_REVIEW.md](PHASE_2D_REVIEW.md), F11 and D6). All code so far was written and checked
> by AI models; this review is the first human reading. **D6 stays open until a human reviewer completes §5 and §6.**
> An AI session may help prepare it, but must not fill in the record.
>
> Scope agreed 2026-11-26: `crates/civkit-core/src/registry.rs` and `crates/civkit-core/src/calcs/levelling.rs`.
> **Line numbers refer to commit `ed3d20f`.** If you review a later commit, write its hash in §6; the function names stay valid.

## 1. Before you start (about 30 minutes)

1. Read the contract rules: [API_DESIGN.md](../docs/API_DESIGN.md) §2 (with §2.1, lists and choices), §3, §5, §6 and §7.1.5
   (the three unit roles).
2. Read [ADR 0011](decisions/0011-list-and-choice-inputs.md) (D-23, list and choice inputs).
3. Optional: [ENGINEERING_STANDARDS.md](../docs/ENGINEERING_STANDARDS.md) §3 (numerical practice) and
   [LEVELLING_DESCRIPTOR_PROPOSAL.md](LEVELLING_DESCRIPTOR_PROPOSAL.md) §3–§5.
4. Make sure the code builds and the tests pass on your machine:

```powershell
cargo test --locked --workspace
```

## 2. `registry.rs`, in this order

| # | Area | Where (at `ed3d20f`) | What to look for |
|---|---|---|---|
| 2.1 | **Request validation and unit conversion** | `validate_inputs` (line 413), `to_si` (722), `list_to_si` (594), `choice_value` (572), `unit_error` (708), `apply_validity_rules` (810) | Every input path ends in either a value in internal SI units or a contract error with the right code (API_DESIGN §5 and §7.1.5). |
| 2.2 | **List limits and error indices** | `MAX_LIST_LEN` (42), group length and maximum checks (about 511–560), the per-element `at` helper in `list_to_si` (about 601–605), `index_set_len` (406) | Every element error carries a 0-based `index` and a 1-based "item k" in its message; group lengths and limits are checked as ADR 0011 says. |
| 2.3 | **Execution, panic handling, output checks** | `call` (276), `catch_unwind` (287), output length checks (about 305–330), `call_json` (378) | A panic or a wrong-length output becomes `INTERNAL`, never a crash or a malformed response. |
| 2.4 | **Trace checks and descriptor validation** | `split_choice_steps` (861), `check_trace` (902), `index_sets` (967), `check_descriptor` (983), `check_lists_and_choices` (1104) | The trace rules of API_DESIGN §6 (choice steps first, structured items, internal units) are enforced after `compute`; malformed descriptors are refused at registration. |

**Specific questions to answer** (write "yes", "no" or "unsure", with a line reference):

1. Can a **finite input become non-finite during unit conversion** (for example a very large value in `km` or `ksi`), and
   if so, is it caught before `compute` runs or only afterwards?
2. Is the **10 000-element limit enforced before the elements of a list are processed**, or only after every element has
   been converted? Does that matter for a hostile or accidental very long list?
3. Does `check_descriptor` **reject invalid input and output names** (empty, not `snake_case`, clashing with a group
   name)? Groups and choices are checked for `snake_case`; are inputs and outputs?
4. Are errors from several inputs **all reported together**, as API_DESIGN §5 promises, or does any path stop at the first?
5. Is there any `unwrap`, `expect`, indexing or arithmetic that a request (not a calculation bug) can make panic?

## 3. `levelling.rs`

| # | Area | Where (at `ed3d20f`) | What to check |
|---|---|---|---|
| 3.1 | **Height of instrument** | `hi_reduction` (154) | HI = previous elevation + backsight; elevation = HI − foresight; the "previous elevation" is the start for setup 1 and the last computed elevation afterwards. |
| 3.2 | **Rise and fall** | `rf_reduction` (174), `sums` (197) | rise = max(BS − FS, 0), fall = max(FS − BS, 0); elevation = previous + rise − fall; the totals satisfy last − start = ΣBS − ΣFS. |
| 3.3 | **Closure and adjustment** | `closure_adjustment` (221) | Closure error sign (computed − known, positive when the run came out too high); corrections −e·S/L proportional to the *cumulative* distance; the last adjusted elevation equals the known one. |
| 3.4 | **Pairing, trace, units, overflow** | descriptors and `compute` (276–552) | Backsight k pairs with foresight k (one group, `setups`); trace items use the right label (`setup`, `point`) and 0-based index; every trace unit is `m`; overflow becomes `OUT_OF_DOMAIN`, never `inf` or `NaN`. |
| 3.5 | **The reference cases against the source** | `tests/reference/surveying.levelling.*` and [REFERENCE_CASES.md](REFERENCE_CASES.md) C4 and C9 | Compare the inputs and expected values with NAVEDTRA 14069, ch. 14 (Fig. 14-14, pp. 14-13 to 14-15; p. 14-10), source S13 in [REFERENCE_SOURCES.md](REFERENCE_SOURCES.md). Check the recorded erratum (backsight printed 11.65, computed with 11.56) and that the rise-and-fall cases are labelled as consistency checks (D-20 condition 2). |

## 4. Severity scale

| Severity | Meaning |
|---|---|
| **Critical** | A wrong engineering result, a crash or hang from a request, or a security problem. |
| **Major** | A contract rule broken (wrong error code, missing `index`, a trace rule not enforced), or a gap a likely input can hit. |
| **Minor** | Correct but fragile, unclear or inconsistent with the documents. |
| **Note** | A question or suggestion; no defect claimed. |

## 5. Findings

One row per finding. "Reproduce" means a request, input values or a test that shows it.

| # | Severity | File:line | Finding | Why it matters | How to reproduce | Status |
|---|---|---|---|---|---|---|
| R1 | | | | | | |
| R2 | | | | | | |

Answers to the five questions in §2:

1.
2.
3.
4.
5.

## 6. Sign-off

| | |
|---|---|
| Reviewer | |
| Date | |
| Commit reviewed | |
| Scope covered | `registry.rs` §2.1–§2.4; `levelling.rs` §3.1–§3.5 (strike out anything not covered) |
| Outcome | Approved / approved with findings / changes required |

After sign-off, the maintainer marks D6 closed in [PHASE_2D_REVIEW.md](PHASE_2D_REVIEW.md) and records the outcome in
this file and in [ACCEPTANCE_CRITERIA.md](ACCEPTANCE_CRITERIA.md). Fixes for findings follow the usual rule: a test that shows the problem first, then
the change.
