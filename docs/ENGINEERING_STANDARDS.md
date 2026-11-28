# Engineering Standards

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-24 23:41 IST (as part of README.md) · last edited 2026-11-28 14:15 IST by Claude Sonnet 5.5 (`claude-sonnet-5-5`)

> **Status:** Draft. Applies to every calculation in CivKit, whoever (or whatever) writes it.

## 1. Trust principles

- **Reference cases are not optional paperwork.** They are the test cases that stop CivKit from confidently
  producing nonsense. A calculation without verified reference cases is not merged.
- **No silent results.** Every result carries its units, the steps used, the reference it came from, and any warnings.
- **No silent changes.** A calculation's formula never changes under the same ID and version ([API_DESIGN.md §4](API_DESIGN.md#4-ids-and-versioning)).
- **Fail loudly on units.** Dimensionally inconsistent input is an error, never a guess.
- **Validity ranges are part of the calculation, when there is a basis for them** (D-24, ADR 0012). Mathematical domain
  checks always apply. A method's validity range is stated only when a reliable source gives it, and inputs outside it
  produce a warning or an error. When no reliable basis is established, the calculation says so explicitly; a range is
  never invented to fill the field.
- **AI calls, it does not compute.** Through MCP, an AI client gets the numbers and the explanation from CivKit.

## 2. What every calculation must ship with

| # | Requirement |
|---|---|
| 1 | A descriptor with ID, version, inputs/outputs with dimensions ([API_DESIGN.md §2](API_DESIGN.md#2-calculation-descriptor)) |
| 2 | **References** — book (title, edition, page/section) or code clause number. Cited, not copied. |
| 3 | **Assumptions** in plain language |
| 4 | **Validity limits**, enforced as warnings or errors, each citing its basis; or, when no reliable basis is established, an explicit statement that none is stated (D-24). Mathematical domain checks always apply. |
| 5 | **≥1 reference case** in `tests/reference/`: known input, documented method and assumptions, cited source, stated tolerance, and an expected result that is **independently verified** — re-derived by hand or confirmed by a second source, not just copied from the book (textbooks have errata) |
| 6 | **Edge-case tests**: zero, limits of validity range, invalid inputs, unit variations |
| 7 | A generated **doc page**: formula, symbols, references, assumptions, limits, example |
| 8 | **Two reviews**: one code review and one engineering review |

Full checklist: [planning/ACCEPTANCE_CRITERIA.md](../planning/ACCEPTANCE_CRITERIA.md).

## 3. Numerical practice

- Internal computation in SI, `f64`.
- **No rounding inside the engine.** Rounding happens only when displaying.
- When a source rounds an intermediate value (e.g. R to two figures) the tolerance may widen to cover that rounding;
  the reason is recorded with the case. Source errata are recorded, never silently "fixed" into expected values.
- Reference-case tolerance reflects the precision **printed in the source** (a textbook giving 3 significant
  figures is checked to that), not machine precision. Exact algebraic identities are tested tightly (relative ~1e-9).
- Iterative methods state their convergence criterion and maximum iterations, and report non-convergence as an error.
- Angles in surveying use exact DMS handling; no floating-point drift in angle sums that should close exactly.

## 4. Sources, design codes and copyright

- **Reference cases come only from openly licensed or public-domain sources** (D-18). Approved sources and
  the licence policy are in [planning/REFERENCE_SOURCES.md](../planning/REFERENCE_SOURCES.md). Only numbers,
  method names and citations are taken from a source — never text, figures or tables.

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

This exact sentence is one constant in the engine (`civkit_core::DISCLAIMER`, checked against this section by a test).
It is shown by the interfaces and is never part of the JSON contract, so responses stay identical across interfaces
(D-13):

- **Generated calculation pages and notebooks:** at the top of every page and in each notebook's first cell.
- **Python:** `civkit.DISCLAIMER`; the documentation of the package, `call`, `list`, `describe`, every typed
  calculation function, `civkit.drawing`, `analyze` and `schedule_csv`; and the display of a `Result` or a drawing
  report (`repr`, and Jupyter's rich display). `Result.to_dict()` and the report dict are the engine's JSON, unchanged.
- **MCP:** the server instructions (which ask the client to show it with any CivKit result) and the end of every tool
  description: per-calculation, generic and drawing tools. Tool results are the contract JSON, unchanged. CivKit gives
  the text to the client; it cannot make a third-party client display it.
