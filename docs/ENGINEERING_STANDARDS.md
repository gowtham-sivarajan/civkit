# Engineering Standards

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-24 23:41 IST (as part of README.md) · last edited 2026-11-25 01:00 IST

> **Status:** Draft. Applies to every calculation in CivKit, whoever (or whatever) writes it.

## 1. Trust principles

- **Reference cases are not optional paperwork.** They are the test cases that stop CivKit from confidently
  producing nonsense. A calculation without verified reference cases is not merged.
- **No silent results.** Every result carries its units, the steps used, the reference it came from, and any warnings.
- **No silent changes.** A calculation's formula never changes under the same ID and version ([API_DESIGN.md §4](API_DESIGN.md#4-ids-and-versioning)).
- **Fail loudly on units.** Dimensionally inconsistent input is an error, never a guess.
- **Validity ranges are part of the calculation.** Inputs outside a method's stated range produce a warning or an error.
- **AI calls, it does not compute.** Through MCP, an AI client gets the numbers and the explanation from CivKit.

## 2. What every calculation must ship with

| # | Requirement |
|---|---|
| 1 | A descriptor with ID, version, inputs/outputs with dimensions ([API_DESIGN.md §2](API_DESIGN.md#2-calculation-descriptor)) |
| 2 | **References** — book (title, edition, page/section) or code clause number. Cited, not copied. |
| 3 | **Assumptions** in plain language |
| 4 | **Validity limits**, enforced as warnings or errors |
| 5 | **≥1 reference case** in `tests/reference/`: known input, documented method and assumptions, cited source, stated tolerance, and an expected result that is **independently verified** — re-derived by hand or confirmed by a second source, not just copied from the book (textbooks have errata) |
| 6 | **Edge-case tests**: zero, limits of validity range, invalid inputs, unit variations |
| 7 | A generated **doc page**: formula, symbols, references, assumptions, limits, example |
| 8 | **Two reviews**: one code review and one engineering review |

Full checklist: [planning/ACCEPTANCE_CRITERIA.md](../planning/ACCEPTANCE_CRITERIA.md).

## 3. Numerical practice

- Internal computation in SI, `f64`.
- **No rounding inside the engine.** Rounding happens only when displaying.
- Reference-case tolerance reflects the precision **printed in the source** (a textbook giving 3 significant
  figures is checked to that), not machine precision. Exact algebraic identities are tested tightly (relative ~1e-9).
- Iterative methods state their convergence criterion and maximum iterations, and report non-convergence as an error.
- Angles in surveying use exact DMS handling; no floating-point drift in angle sums that should close exactly.

## 4. Design codes and copyright

- Design-code modules (IS, Eurocode, ACI, AASHTO …) come **after 0.1**, one code at a time (D-8).
- Code texts and many tables are copyrighted. Implement equations and cite clause numbers; **do not paste code
  text or reproduce copyrighted tables** into the repository. Where a table's values are needed, record the
  source and seek permission or derive from the underlying equation where the code provides one.
- Each code module states the code edition and amendments it implements.

## 5. Testing strategy

- **Unit tests** in every crate.
- **Reference tests** from published worked examples (data files with source and tolerance).
- **Property tests** (`proptest`): unit round-trips, area invariant under translation/rotation, etc.
- **Parity tests**: the same request through Rust, FFI, Python and MCP gives identical output.
- **Fixture tests** on DXF files for drawing analysis.
- **CI** on GitHub Actions across Windows, macOS and Linux.
- The AutoCAD/Civil 3D plugins cannot run in CI: their logic stays thin; a manual test checklist covers in-host behaviour.
- Formatting and lints enforced: `rustfmt`, `clippy`, `ruff`, `dotnet format`.

## 6. Review

- An **engineering reviewer** checks formula, symbols, units, assumptions, limits and references against the source.
- A **code reviewer** checks implementation, tests and the dependency rule.
- AI-written engineering content always gets human engineering review.

## 7. Disclaimer (shown in every interface)

> CivKit results support, and do not replace, the judgement of a qualified engineer.
