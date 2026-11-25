# Feature Proposals

> **Provenance:** Claude Sonnet 5.5 (`claude-sonnet-5-5`) · first drafted 2026-11-25 22:25 IST · last edited 2026-11-25 22:25 IST

> **Status:** Proposals only. **Nothing here is accepted, scheduled or applied.** The 0.1 scope
> ([MVP_SCOPE.md](MVP_SCOPE.md)) and every accepted decision stand unchanged. A proposal that would change an accepted
> decision or the public contract is marked **needs a decision record**. Evidence and sources:
> [COMPETITIVE_LANDSCAPE.md](COMPETITIVE_LANDSCAPE.md) (research date **2026-11-25**; evidence labels **[V]** verified,
> **[S]** summarised, **[I]** interpretation, **[H]** hypothesis, **[?]** unverified are used the same way here).

## 0. How to read this document

- **28 serious proposals** (FP-01 … FP-28) with all required fields, then **12 rejected or deferred ideas** (R-1 … R-12)
  with reasons. The 28 are grouped by category (§2); scores and shortlists are in §3–§5.
- **Categories:** A core · B optional module · C integration · D standalone application · E developer ecosystem ·
  F reject or defer.
- **0.1 impact:** every proposal says what it would do to 0.1. The recommendation is that **0.1 stays exactly as
  planned.** Items marked "decision only" cost a paragraph now and save a migration later; none adds code to 0.1.
- **Ambition check:** the project is first-year-undergraduate-led and single-maintainer. Each proposal says whether
  it is realistic *for this team*: **yes**, **with care** (needs a prototype and a stop condition) or **no / later**.
- Dependencies named here that were *researched* carry their licence from the landscape report. Crates
  marked *(not researched)* are only candidates; their licence and maintenance must be checked against `deny.toml`
  before any use.

**What the proposals add up to** (the answer to the brief's final question): CivKit becomes the *verifiable,
interface-independent calculation layer*: records, ledgers and cross-checks (trust); CLI, MCP, typed clients and
conformance tests (reach); offline and permissive (independence). More calculators are deliberately *not* on the list.

---

## 1. The proposals

### Trust and provenance

#### FP-01 · Calculation Record and offline replay (`verify`)

| Field | |
|---|---|
| **Category** | A (record schema, replay) with E (CLI command) |
| **Description** | A small, canonical JSON document that wraps one request and its response: the *normalised* inputs in SI, calculation ID and version, outputs, trace, warnings, references, plus an `engine` block (CivKit version, build identifier, hash of the calculation's descriptor). `civkit verify record.json` re-runs the request on the local engine and reports "identical", "differs within tolerance" or "differs", and says if the calculation version is no longer available. |
| **User problem** | "Which version of which method gave this number, and can I get it again next year?" Today a result is a number pasted into a report. QA reviewers, students submitting work, and AI-assisted workflows all need to re-check without trusting the author. |
| **Inspiration** | Stabileo's ledger (per-project, not per-result) **[V]**; OpenMDAO recorders **[V titles]**; RO-Crate (one JSON file plus optional payload) **[V]**; `handcalcs` renders but warns about "beautiful but completely incorrect" notebooks **[V]**; the repo's own rule "no silent changes" (API_DESIGN §4). No competitor found with this **[I]**. |
| **CivKit fit** | Excellent. Versioned IDs, SI-normalised inputs and a self-describing descriptor already exist; `Response` already carries `civkit_version`. A record is built *around* a response, so the accepted JSON shape of calls does not change (AP-1 in the landscape report). |
| **Dependencies** | `serde_json` (already used). Hashing: a small, permissively licensed hash crate *(not researched)*, or none for v1 (omit the hash and use the version only). |
| **Offline** | Fully offline. |
| **Difficulty** | Medium (schema design is the hard part; code is small). |
| **Maintenance** | Low once the schema is stable. A schema change is a version bump like any other contract change. |
| **Verification** | Property test: for every reference case, `record → verify → "identical"`. Mutation tests: edit one input or one output in a record and confirm `verify` flags it. Cross-interface parity: Rust, Python and MCP produce byte-identical records. |
| **Strategic value** | The single most distinctive idea found. It turns "trust us" into "check us". Makes CivKit usable in QA, teaching and audit settings where competing tools offer nothing portable. **[I]** |
| **0.1 impact** | **Decision only** (the schema) before the Phase 3 parity tests lock the response shape. Implementation after 0.1. |
| **Needs decision record** | Yes (new document type; touches API_DESIGN §3, §11). |
| **Ambition check** | Yes. Small code, careful design. |

#### FP-02 · Machine-readable verification ledger

| Field | |
|---|---|
| **Category** | A (descriptor/test metadata) with E (generated docs) |
| **Description** | For every calculation, a generated ledger: each reference case (source, edition, page, tolerance, who and how independently verified, errata found), the edge-case tests, known limits and *known gaps*, and the date last run. Exposed as JSON (for MCP and the CLI) and rendered into the doc page. `civkit ledger` prints it. |
| **User problem** | "How do we know this is right?" is currently answered in prose scattered across REFERENCE_CASES.md, tests and the handover file. |
| **Inspiration** | Stabileo's `BENCHMARKS.md` (suites, counts, *and an explicit list of items that do not yet match*) **[V]**; sectionproperties' Validation chapter **[V titles]**; the repo's own REFERENCE_CASES.md and ENGINEERING_STANDARDS. |
| **CivKit fit** | The reference-case JSON format and harness exist (`tests/reference/`). The ledger is a *view* over them plus a few extra fields. |
| **Dependencies** | None new. |
| **Offline** | Fully. |
| **Difficulty** | Low–medium. |
| **Maintenance** | Low (generated; the discipline already exists). |
| **Verification** | The harness already fails if a registered calculation has no case; extend it to fail if a case lacks a ledger field. |
| **Strategic value** | Makes the project's strictest habit (every case independently verified, errata recorded) *visible and queryable*. Differentiates against competitors whose verification is a self-reported number. |
| **0.1 impact** | Fits the existing "generated doc page for every calculation" criterion; recommend as a *shape* for that page, not extra scope. |
| **Needs decision record** | No for generation; yes if new mandatory case fields are required. |
| **Ambition check** | Yes. |

#### FP-03 · Built-in cross-checks (`checks`)

| Field | |
|---|---|
| **Category** | A |
| **Description** | Each calculation may return a list of *internal consistency checks* it ran: reactions sum to applied load; traverse closure within stated tolerance; cut and fill from two methods agree; Manning velocity × area equals discharge. Each check has a name, residual, tolerance and pass/fail. A failure becomes a warning with its own code. |
| **User problem** | A confident wrong answer is the worst failure. Cheap independent checks catch programming errors and bad inputs. |
| **Inspiration** | Open Infra's corridor volumes vs independent TIN-to-TIN volume within 2 % **[V]**; the hosted CivilKit hand-check idea **[V]**; the repo's own habit of re-deriving every reference value with a second method. |
| **CivKit fit** | Pure addition to the response (API_DESIGN §4: additions do not bump versions). The registry already post-checks trace steps (`INTERNAL` on violation), so a checks list follows the same pattern. |
| **Dependencies** | None. |
| **Offline** | Fully. |
| **Difficulty** | Low per calculation. |
| **Maintenance** | Low–medium (each new calculation should define its checks). |
| **Verification** | Plant an error into a calculation in a test and confirm the relevant check fails (mutation-style, as already done for geometry). |
| **Strategic value** | Raises trust at near-zero user cost; checks appear in records (FP-01) and MCP answers. |
| **0.1 impact** | None required. Could be added to a calculation *as a trace step* today. |
| **Needs decision record** | Yes if a `checks` field is added to the contract. |
| **Ambition check** | Yes. |

#### FP-04 · Local hand-calculation checker

| Field | |
|---|---|
| **Category** | A (comparison logic) exposed through E (CLI), C (Python, MCP) |
| **Description** | Input: a calculation ID, inputs, and *the user's own answer with its unit*. Output: the engine's value, the relative deviation, whether it is within a stated tolerance, and a **deterministic list of likely slips**: unit factor off by 10ⁿ, 25.4, 0.3048, 9.80665, 2π, π/180, diameter vs radius, inch vs foot. |
| **User problem** | Students, reviewers and AI users want "is my number right?" without a lecture. |
| **Inspiration** | The hosted CivilKit tool `calcs_check_hand_calc` and its prompt **[V]** (cloud, rate-limited); the original proposal's "AI calls, it does not compute". |
| **CivKit fit** | Reuses the engine and units. The "likely slip" table is data. Needs no AI. |
| **Dependencies** | None. |
| **Offline** | Fully. This is the point: the same function as the hosted one, local. |
| **Difficulty** | Low. |
| **Maintenance** | Low. |
| **Verification** | Table-driven tests: apply each known slip to a reference result and check the diagnosis names it. Negative tests: a genuinely different result gives no false diagnosis. |
| **Strategic value** | Directly answers a use case the competitor has validated, without the cloud. Good demo of the MCP story. |
| **0.1 impact** | None. After 0.1. |
| **Needs decision record** | No. |
| **Ambition check** | Yes. |

### Reach and developer ecosystem

#### FP-10 · `civkit` command-line tool

| Field | |
|---|---|
| **Category** | E |
| **Description** | One binary: `civkit list`, `describe`, `call request.json`, `verify record.json` (FP-01), `ledger` (FP-02), later `report` (FP-12) and `batch` (FP-11). JSON on stdin/stdout; exit codes map to error classes. |
| **User problem** | Scripts, CI pipelines, MATLAB fallback, firms with batch jobs, and testers who do not want Python. |
| **Inspiration** | CalcpadCE's batch CLI, Pondus `design`/`bench`, ezdxf's CLI **[V]**. |
| **CivKit fit** | A thin wrapper like every other interface; lives in `integrations/cli` (or `apps/`), depends on `civkit-core` only. |
| **Dependencies** | A small argument parser *(not researched)*. |
| **Offline** | Fully. |
| **Difficulty** | Low. |
| **Maintenance** | Low. |
| **Verification** | The same parity harness as Python and MCP; golden request/response files (FP-21). |
| **Strategic value** | Cheap and widely useful; becomes the reference client for the conformance suite and the easiest way for a reviewer to try CivKit. |
| **0.1 impact** | Not in the accepted 0.1 scope. A tiny CLI could ride along because Phase 3's parity tests need a harness anyway; **recommendation: do not add it to 0.1 unless the maintainer decides so.** |
| **Needs decision record** | Yes if added to a release scope (layout: ADR 0001 lists no `cli` folder). |
| **Ambition check** | Yes. |

#### FP-19 · MCP resources, prompts and output schemas

| Field | |
|---|---|
| **Category** | C |
| **Description** | Beyond one tool per calculation: (1) generate `outputSchema` from descriptors and return `structuredContent`; (2) expose **resources**: descriptors, reference cases, the ledger (FP-02), unit table; (3) a few **prompts** ("explain this result from its trace", "check my hand calculation"); (4) tools remain read-only; add `civkit_check_hand_calc` (FP-04) and `civkit_verify_record` (FP-01). |
| **User problem** | AI clients need to *read the evidence*, not just call tools; typed outputs prevent the client from mis-parsing numbers. |
| **Inspiration** | MCP spec 2026-07-28: resources, prompts, output schemas, structured content **[V]**; hosted CivilKit MCP uses resources and prompts **[V]**; Open Infra generates and tests `mcp-tools.json` **[V]**. |
| **CivKit fit** | API_DESIGN §10 already says tools are generated from descriptors and return the full contract response; this extends the generation to schemas and resources. |
| **Dependencies** | `rmcp` (Apache-2.0, 3.11 on 2026-11-17 **[V]**), already planned (D-13). |
| **Offline** | Fully (stdio only). |
| **Difficulty** | Low–medium. |
| **Maintenance** | Medium: the MCP specification moved a major revision in 2026; isolation in `integrations/mcp` is the right mitigation and already planned. |
| **Verification** | Validate every response against its generated `outputSchema`; parity test against Rust results; test with MCP Inspector (already in Phase 3 criteria). |
| **Strategic value** | Positions CivKit as the *trustworthy* MCP in a field of write-capable ones. Directly supports the "AI calls, it does not compute" rule. |
| **0.1 impact** | Part of the Phase 3 design; recommend deciding the surface (tools only vs tools + resources + prompts) at Phase 3 start. Resources and prompts can follow after 0.1. |
| **Needs decision record** | Yes: amends API_DESIGN §10 and revisits D-13 against the 2026-07-28 revision. |
| **Ambition check** | Yes. |

#### FP-20 · Typed client generation and a JSON-Schema bundle

| Field | |
|---|---|
| **Category** | E |
| **Description** | `civkit_schema()` (FFI) and `civkit schema` (CLI) export one JSON-Schema bundle for all descriptors. Generators produce: Python stubs/typed functions (already planned), C# records and enums for the AutoCAD plugin, TypeScript types for the desktop UI, and a docs data file. |
| **User problem** | The AutoCAD plugin and the desktop form must not hand-copy input names and units; drift between interfaces is the failure the architecture exists to prevent. |
| **Inspiration** | Open Infra's generated and tested tool manifest **[V]**; ARCHITECTURE §3.1 ("interfaces are generated from the descriptor"). |
| **CivKit fit** | Direct. |
| **Dependencies** | None new for the Rust side; generators can be scripts in `tools/`. |
| **Offline** | Fully. |
| **Difficulty** | Low–medium. |
| **Maintenance** | Low–medium. |
| **Verification** | CI regenerates and fails on diff (the Open Infra pattern). |
| **Strategic value** | Makes "one contract, many interfaces" mechanical. |
| **0.1 impact** | Python typed functions are already in Phase 3; the rest follows. |
| **Needs decision record** | No (implements ARCHITECTURE §3.1). |
| **Ambition check** | Yes. |

#### FP-21 · Golden-vector conformance suite

| Field | |
|---|---|
| **Category** | E |
| **Description** | A published folder of request/response vectors (including errors, warnings, units edge cases, ambiguity rejections, list inputs) with a tiny runner spec. Any wrapper (Python, MCP, .NET, MATLAB, a third party's) must reproduce them byte for byte after normalisation. |
| **User problem** | Third parties and future interfaces need a way to *prove* they match. |
| **Inspiration** | The repo's own planned parity tests; no equivalent found in competitors **[I]**. |
| **CivKit fit** | The reference-case harness generalises; vectors can be generated from the Rust tests. |
| **Dependencies** | None. |
| **Offline** | Fully. |
| **Difficulty** | Low. |
| **Maintenance** | Low (vectors change only with contract versions). |
| **Verification** | The suite is itself the test; mutation of a wrapper must fail it. |
| **Strategic value** | Turns the contract from documentation into something enforceable by outsiders. |
| **0.1 impact** | Already implied by the Phase 3 "parity test" criterion; this proposes *publishing* it in a reusable form. |
| **Needs decision record** | No. |
| **Ambition check** | Yes. |

#### FP-22 · Differential-testing harness against permissive oracles

| Field | |
|---|---|
| **Category** | E |
| **Description** | Scripts in `tools/` that compare CivKit results to independent implementations on random inputs: shapely (area/centroid/length), Pint (unit conversion), ezdxf (DXF entities), SciPy (root finding), PyNite (MIT; beam deflection). Copyleft tools (IfcOpenShell, anaStruct) run only as separate processes. Findings go to a log, not to tests that need those tools in CI. |
| **User problem** | Reference cases cover chosen points; random comparison catches regions nobody thought of. |
| **Inspiration** | The 2B geometry review that re-derived every formula against shapely and 60-digit arithmetic (GEOMETRY_SPEC_REVIEW.md); `proptest` already in use. |
| **CivKit fit** | Dev-time only; keeps the engine free of Python (dependency rule). |
| **Dependencies** | Python oracles only in a dev virtualenv (`.venv` exists). No runtime or CI requirement. |
| **Offline** | Offline after install. |
| **Difficulty** | Low–medium. |
| **Maintenance** | Low–medium; oracle versions drift. |
| **Verification** | Self-checking; a planted bug in CivKit must be found within N random cases. |
| **Strategic value** | Strengthens the trust story at little cost; respects licence rules (§6 of the landscape report). |
| **0.1 impact** | None required; fits Phase 2D regression. |
| **Needs decision record** | No. |
| **Ambition check** | Yes. |

#### FP-26 · Reproducible builds, SBOM and engine fingerprint

| Field | |
|---|---|
| **Category** | E |
| **Description** | Release builds are reproducible where practical; every release ships a software bill of materials and SHA-256 checksums (already in SECURITY.md); the binary reports a fingerprint that FP-01 records embed. |
| **User problem** | "Is this the engine that produced that record?" and supply-chain scrutiny for firms. |
| **Inspiration** | Open Infra signs releases (Authenticode, notarisation, OpenPGP) **[V]**; SECURITY.md supply-chain section. |
| **CivKit fit** | Extends existing CI and `cargo-deny`. |
| **Dependencies** | An SBOM generator *(not researched)*; code-signing certificates cost money and are out of scope ("must be free", SECURITY.md). |
| **Offline** | Fully. |
| **Difficulty** | Medium (reproducibility is fiddly). |
| **Maintenance** | Medium. |
| **Verification** | Two clean builds produce identical hashes in CI. |
| **Strategic value** | Supports the independence pillar. |
| **0.1 impact** | Checksums already required for 0.1. SBOM and reproducibility after. |
| **Needs decision record** | No. |
| **Ambition check** | With care (stop condition: if reproducibility costs more than a week, ship checksums and SBOM only). |

### Calculation ergonomics

#### FP-09 · Parameter sweeps and batch calls (Python and CLI)

| Field | |
|---|---|
| **Category** | C |
| **Description** | `civkit.sweep(id, base_inputs, vary={"depth": range})` and `civkit.batch([...])` returning a table; CLI `civkit batch file.jsonl`. Each row keeps its own trace and warnings; rows are independent calls, so results are identical to single calls. |
| **User problem** | Engineers explore ranges ("what depth gives Q = 4 m³/s?") and run many cases. |
| **Inspiration** | PyCBA's `to_dataframe`/`to_csv` **[V]**; SALib sampling; OpenMDAO drivers. |
| **CivKit fit** | D-23 already clarified "array inputs" means batching; a loop over `call`. |
| **Dependencies** | None required; optional pandas conversion if installed. |
| **Offline** | Fully. |
| **Difficulty** | Low. |
| **Maintenance** | Low. |
| **Verification** | `batch(n) == [call(i) for i in n]` as a property test. |
| **Strategic value** | Removes the main reason to leave CivKit for a spreadsheet. |
| **0.1 impact** | Out of the 0.1 scope (API_DESIGN §8). After 0.1. |
| **Needs decision record** | No. |
| **Ambition check** | Yes. |

#### FP-11 · Spreadsheet-driven batch (CSV first, XLSX later)

| Field | |
|---|---|
| **Category** | B |
| **Description** | `civkit batch --csv inputs.csv --calc fluids.open_channel.manning_trapezoidal` with a header row of input names and units; writes results and warnings to a new file. XLSX later. |
| **User problem** | Engineers live in Excel; this is the shortest bridge. |
| **Inspiration** | Stabileo and CalcpadCE both read/write Excel **[V]**; the original proposal's "export CSV/JSON". |
| **CivKit fit** | Pure wrapper over `call`. |
| **Dependencies** | CSV: `csv` crate *(not researched)*. XLSX read/write crates *(not researched)*; check licences first. |
| **Offline** | Fully. |
| **Difficulty** | Low (CSV); medium (XLSX). |
| **Maintenance** | Low (CSV); medium (XLSX). |
| **Verification** | Round-trip tests; units in headers use the unit parser. |
| **Strategic value** | Practical adoption path. |
| **0.1 impact** | Drawing schedules already export CSV/JSON (in scope); batch input is separate. |
| **Needs decision record** | No. |
| **Ambition check** | Yes for CSV. |

#### FP-12 · Report generation from records

| Field | |
|---|---|
| **Category** | B (`civkit-report`, optional crate) |
| **Description** | Turns a record (FP-01) into a self-contained HTML page and Markdown: inputs with units; the formula, then values substituted, then the result (from the trace); warnings; references; the verification ledger entry; the disclaimer. PDF is a later stage (print-to-PDF from the HTML is enough at first). |
| **User problem** | A result is useless to a client or reviewer until it is on paper with its working. |
| **Inspiration** | `handcalcs` (symbolic → substituted → result) **[V]**; CalcpadCE/CivilKit Word/Excel/PDF output **[V]**; Pondus PDF reports **[V]**. |
| **CivKit fit** | The trace already stores expression strings, value and unit per step. |
| **Dependencies** | None for HTML/Markdown. A PDF engine *(not researched; Typst is a candidate to evaluate)*. |
| **Offline** | Fully; the HTML must load no remote fonts, scripts or images (OFFLINE_POLICY). |
| **Difficulty** | Medium. |
| **Maintenance** | Medium (layout, locales). |
| **Verification** | Golden-file tests; check every number in the report equals the record. |
| **Strategic value** | Highest *practical* value in this list; every competitor has reports; CivKit's differ by embedding the record so the printout can be re-verified. |
| **0.1 impact** | None. After 0.1; desktop (Phase 6) needs it. |
| **Needs decision record** | No. |
| **Ambition check** | With care (HTML first). |

#### FP-24 · Jupyter rich display

| Field | |
|---|---|
| **Category** | C |
| **Description** | `Result._repr_html_` renders inputs, trace, warnings, units; `Q` objects print nicely; a small notebook helper shows a record and `verify` status. |
| **User problem** | Notebook users should see the working, not a dict. |
| **Inspiration** | `handcalcs` rendering **[V]**. |
| **CivKit fit** | Python layer only (`integrations/jupyter` holds notebooks and widgets already). |
| **Dependencies** | None required; IPython optional. |
| **Offline** | Fully. |
| **Difficulty** | Low. |
| **Maintenance** | Low. |
| **Verification** | Snapshot tests of the HTML. |
| **Strategic value** | Makes Python adoption pleasant. |
| **0.1 impact** | Phase 3 already lists example notebooks; this is a nicety. |
| **Needs decision record** | No. |
| **Ambition check** | Yes. |

#### FP-28 · Unit and presentation profiles

| Field | |
|---|---|
| **Category** | A |
| **Description** | Named sets of `output_units`: `si`, `us_customary`, `metric_civil` (mm, kN, MPa, L/s, ha). Selected once per session or request. Defaults still come from each descriptor's `display_unit`. |
| **User problem** | A US firm or a metric civil firm wants all outputs in its conventions without listing each. |
| **Inspiration** | PyCBA's display unit systems (labels only) **[V]**. |
| **CivKit fit** | Builds on the accepted three-role unit design (§7.1.5). Presentation only; the trace stays internal. |
| **Dependencies** | None. |
| **Offline** | Fully. |
| **Difficulty** | Low. |
| **Maintenance** | Low. |
| **Verification** | Each profile's units match the output's dimension for every registered calculation (a registry test). |
| **Strategic value** | Small, but removes friction. |
| **0.1 impact** | None. |
| **Needs decision record** | Yes if added to the contract (a request field). |
| **Ambition check** | Yes. |

### Engineering depth (optional modules and long-term)

#### FP-05 · Sensitivity outputs (derivatives)

| Field | |
|---|---|
| **Category** | A (opt-in) |
| **Description** | On request, return the partial derivative of each output with respect to each numeric input, in units, and a ranked "what matters most" list. Implementation options: (1) numerical central differences at registry level, no trait change; (2) automatic differentiation by making `compute` generic over a scalar type. |
| **User problem** | "Which input should I measure more carefully?" "How wrong is my velocity if my slope is 5 % off?" |
| **Inspiration** | `uncertainties` (derivatives computed automatically) **[V]**; `num-dual` (MIT/Apache, exact derivatives) **[V]**; OpenMDAO's derivative contract **[V titles]**; SALib for global methods **[V]**. No integration with a calculation contract found **[I]**. |
| **CivKit fit** | Option 1 fits today. Option 2 changes the `Calculation::compute` signature (it is `f64`-based today), so it should be decided *early or not at all*. |
| **Dependencies** | None (option 1); `num-dual` + `nalgebra` (option 2). |
| **Offline** | Fully. |
| **Difficulty** | Medium (option 1) to high (option 2). |
| **Maintenance** | Medium; every new calculation must work under the chosen scheme. |
| **Verification** | Compare to analytic derivatives of the closed-form calculations (Manning has simple ones) and to central differences; derivative of a linear calculation must be exact. |
| **Strategic value** | Differentiates sharply (G7) and teaches engineers *why* a number moves. |
| **0.1 impact** | None. Do not change the trait now. |
| **Needs decision record** | Yes, for either option once prototyped. |
| **Ambition check** | Option 1: with care. Option 2: **not yet**. |

#### FP-06 · Input-uncertainty propagation

| Field | |
|---|---|
| **Category** | A (opt-in), with C for Monte Carlo in Python |
| **Description** | Inputs may carry a tolerance (±absolute or ±relative). The response adds an output range from first-order propagation (needs FP-05) and optionally rigorous interval bounds. Monte Carlo and global sensitivity (Sobol) are left to Python with SALib as an optional extra, calling the batch API. |
| **User problem** | Survey and soil inputs are never exact; engineers want "the answer, and how sure". |
| **Inspiration** | `uncertainties` **[V]**, `inari` (IEEE 1788 intervals; MIT; default `gmp` feature to disable) **[V]**, SALib **[V]**. |
| **CivKit fit** | Additive input form; requires FP-05. |
| **Dependencies** | `inari` or first-order only. Avoid `rug` (LGPL). |
| **Offline** | Fully. |
| **Difficulty** | Medium–high. |
| **Maintenance** | Medium. |
| **Verification** | Interval results must *contain* sampled results; first-order matches analytic variance for linear cases. |
| **Strategic value** | Honest handling of uncertainty is rare in free tools; supports "standards-aware" quality. **[I]** |
| **0.1 impact** | None. |
| **Needs decision record** | Yes. |
| **Ambition check** | Later; only after FP-05 option 1 proves useful. |

#### FP-07 · Clause-addressed design-code framework and coverage matrix

| Field | |
|---|---|
| **Category** | A (descriptor metadata) → B (code packs) |
| **Description** | Optional structured `clauses` on descriptors (code, edition, jurisdiction, clause, formula numbers) and a generated **coverage matrix** ("which clauses of IS 456:2000 are implemented, verified, or not yet"). Packs follow D-8 one code at a time. |
| **User problem** | Engineers need to know exactly what is and is not covered, and which edition. |
| **Inspiration** | `structuralcodes` (Apache-2.0; EC2 2004/2023, fib MC 2010, clause-organised) **[V]**; `concreteproperties` AS 3600/NZS 3101 **[V]**. How they avoid copyrighted text **[?]**. |
| **CivKit fit** | The descriptor already holds references; make them structured. ENGINEERING_STANDARDS already says "implement equations, cite clause numbers". |
| **Dependencies** | None. |
| **Offline** | Fully. |
| **Difficulty** | Low (metadata); high (each code). |
| **Maintenance** | High per code (editions, amendments). |
| **Verification** | Coverage matrix is generated and fails CI on unreferenced clause IDs. |
| **Strategic value** | The only route to design-code credibility without a clone of Pondus; valuable to the Indian-code users D-8 mentions. |
| **0.1 impact** | None. |
| **Needs decision record** | Yes for the metadata; D-8 governs the first code. |
| **Ambition check** | Metadata: yes. Packs: one code, slowly. |

#### FP-08 · Calculation workbook (a small chain of calculations)

| Field | |
|---|---|
| **Category** | A/B (`civkit-workbook`) |
| **Description** | A JSON document listing named steps. Each step is a normal request; an input may be a reference to an earlier output (`"$step1.discharge"`), carrying its unit. No expressions, no loops, no conditionals. Each step produces its own record; the workbook records the chain. |
| **User problem** | Real work is chains: channel depth → discharge → culvert check → earthwork. Today each link is copied by hand. |
| **Inspiration** | OpenMDAO components/groups **[V titles]**; worksheets in Mathcad/CalcpadCE/CivilKit **[V]** (but those are programs, not data). |
| **CivKit fit** | Composition outside `compute` (AP-5 in the landscape report), so each step stays individually verifiable. |
| **Dependencies** | None. |
| **Offline** | Fully. |
| **Difficulty** | Medium. |
| **Maintenance** | Medium (pressure to add expressions: resist). |
| **Verification** | A workbook equals the same calls made by hand; unit mismatches across a link are reported at the link. |
| **Strategic value** | Gets most of the value of a worksheet without a language. The explicit "no expressions" line avoids competing with CalcpadCE and CivilKit. |
| **0.1 impact** | None. |
| **Needs decision record** | Yes (new format). |
| **Ambition check** | With care (stop condition: no expression syntax). |

#### FP-13 · Plot-ready series and SVG diagrams

| Field | |
|---|---|
| **Category** | A (series in responses) + B (SVG renderer) |
| **Description** | Responses may include named series (e.g. V(x), M(x), δ(x) along a beam; the channel section outline) with units. A small renderer turns series into self-contained SVG for reports and the desktop. |
| **User problem** | Engineers think in diagrams; every Python structural library plots. |
| **Inspiration** | anaStruct, PyCBA, PyNite diagrams **[V]**. |
| **CivKit fit** | Needs a new output type (list outputs exist after D-23; "series" is a pair of lists with units). |
| **Dependencies** | None for SVG. |
| **Offline** | Fully. |
| **Difficulty** | Medium. |
| **Maintenance** | Medium. |
| **Verification** | Series values equal pointwise calls to the calculation; SVG snapshot tests. |
| **Strategic value** | Medium; table stakes for desktop and reports. |
| **0.1 impact** | None (and the 0.1 beam omits max shear/moment for lack of a qualifying example). |
| **Needs decision record** | Yes (response shape). |
| **Ambition check** | With care. |

### Drawing, survey and civil modules

#### FP-14 · Rule-driven quantity take-off profiles and drawing audit

| Field | |
|---|---|
| **Category** | B (`civkit-drawing` extension) |
| **Description** | A TOML profile maps layers/blocks/attributes to quantity items ("layer `C-ROAD-CURB` polylines → length m; closed polylines on `C-AREA-*` → area m²"), checks drawing units, flags zero-length or duplicate entities, open polylines that should be closed, and mixed units, and writes CSV/JSON schedules plus an issue list with entity handles. |
| **User problem** | Take-offs are repeated by hand and differ firm to firm; unit mistakes between drawings are common. |
| **Inspiration** | The original proposal's quantity schedules; ezdxf `audit` CLI **[V]**; Pondus/Stabileo read DXF only for geometry **[V]**. |
| **CivKit fit** | Already Phase 4; the profile is configuration, not code. |
| **Dependencies** | The DXF reader decision (own, `dxf` MIT, or `acadrust` MPL-2.0 needing a decision record). |
| **Offline** | Fully. |
| **Difficulty** | Medium. |
| **Maintenance** | Medium (DXF corner cases). |
| **Verification** | The five DXF fixtures with exact expected values (already built); ezdxf as differential oracle (FP-22). |
| **Strategic value** | Strong practical value and a natural lead into AutoCAD (Phase 5). |
| **0.1 impact** | The 0.1 scope lists entity listing, lengths/areas, unit checks and schedules. Profiles are the *next* step. |
| **Needs decision record** | No (config); yes for the DXF dependency. |
| **Ambition check** | Yes. |

#### FP-15 · LandXML reader into the neutral model

| Field | |
|---|---|
| **Category** | B |
| **Description** | Read LandXML points, alignments (lines, curves, spirals) and surfaces into `civkit-drawing`'s model; run length, station, area and volume analyses. Read-only. |
| **User problem** | Civil 3D's alignments and surfaces have no DXF equivalent (ADR 0002 consequence); LandXML is the open way out of Civil 3D without its API. |
| **Inspiration** | Open Infra reads and writes LandXML 1.x/1.2 **[V]**. |
| **CivKit fit** | Honours ADR 0002's "explicit extensions later". |
| **Dependencies** | An XML parser *(not researched)*. **LandXML's schema licence could not be verified** (landxml.org unreachable) **[?]**. |
| **Offline** | Fully. |
| **Difficulty** | Medium–high (geometry types, versions). |
| **Maintenance** | Medium. |
| **Verification** | Fixture files from open sources; compare lengths and areas with the file's own declared values. |
| **Strategic value** | Medium–high; reduces dependence on the riskiest integration (Civil 3D). **[H]** |
| **0.1 impact** | None. Candidate for 0.3+. |
| **Needs decision record** | Yes (model extension; licence check first). |
| **Ambition check** | Later. |

#### FP-16 · CRS transformations and ground-to-grid

| Field | |
|---|---|
| **Category** | B (`civkit-crs`, optional) |
| **Description** | 2D coordinate transformations between common systems and a ground-to-grid calculation (combined scale factor) using a pure-Rust projection crate, with CivKit's own tolerance and verification discipline. |
| **User problem** | Survey data in a local grid versus a national projection is a daily source of errors. |
| **Inspiration** | `proj4rs` (pure Rust, 2D only, MIT/Apache; accuracy target 1e-6 m vs PROJ) **[V]**; `proj` needs C libproj **[V]**; Open Infra's 7-parameter RD transform with a 0.2 m caveat **[V]**. |
| **CivKit fit** | Optional crate; core stays clean (dependency rule). |
| **Dependencies** | `proj4rs`; grid files for NTV2/GeoTIFF (large data files: *offline needs bundled grids*, a real cost). |
| **Offline** | Fully, if grids are bundled by the user. |
| **Difficulty** | Medium–high (correctness, datums, grids). |
| **Maintenance** | Medium–high (EPSG data changes). |
| **Verification** | Published geodetic test points; differential test against `pyproj` (dev-time). |
| **Strategic value** | Real but domain-specific; not the core story. |
| **0.1 impact** | None. |
| **Needs decision record** | Yes. |
| **Ambition check** | Later; only with a user who needs it. |

#### FP-17 · TIN, contours and surface volumes

| Field | |
|---|---|
| **Category** | B |
| **Description** | Build a TIN from points, extract contours, compute volumes between surfaces; the general case of earthwork where average-end-area (in 0.1) is the simplified case. |
| **User problem** | Real earthwork quantities come from surfaces. |
| **Inspiration** | Open Infra TIN, contours, volume, and a 2 % cross-check against end-area volumes **[V]**; WhiteboxTools terrain tools **[V]**; `geo` offers triangulation (earcut, spade) **[V]**. |
| **CivKit fit** | Extends quantities; needs robust geometry. |
| **Dependencies** | `geo`'s triangulation or `spade` *(not researched)*. |
| **Offline** | Fully. |
| **Difficulty** | High. |
| **Maintenance** | High (robustness, large data). |
| **Verification** | Closed-form surfaces (planes, pyramids); cross-check TIN against grid volume. |
| **Strategic value** | High value, but a project in itself. |
| **0.1 impact** | None; and 0.1 deliberately defers the general prismoidal case. |
| **Needs decision record** | Yes. |
| **Ambition check** | **No / much later.** Revisit only with experienced contributors. |

#### FP-18 · Continuous beam and 2D frame stiffness module

| Field | |
|---|---|
| **Category** | B |
| **Description** | `structural.beam.continuous` (stiffness method, Euler-Bernoulli) and later plane frames. Verified against textbook worked examples and, as a dev-time oracle, PyNite (MIT). |
| **User problem** | The natural next step after the simply supported beam. |
| **Inspiration** | PyCBA, anaStruct, PyNite **[V]**. |
| **CivKit fit** | A calculation like any other, with lists (D-23). |
| **Dependencies** | `nalgebra` (Apache-2.0) or a small hand-written solver. |
| **Offline** | Fully. |
| **Difficulty** | High (continuous beam: medium; frames: high). |
| **Maintenance** | High; invites feature requests (nonlinearity, 3D, plates). |
| **Verification** | D-18 reference cases; textbook beam tables; equilibrium cross-checks (FP-03). |
| **Strategic value** | Low–medium: the niche is crowded and CivKit's edge is not solver depth. |
| **0.1 impact** | None. |
| **Needs decision record** | Yes. |
| **Ambition check** | Continuous beam only, later. Frames: **no**. |

### Integrations and experience

#### FP-23 · Optional Python interoperability extras

| Field | |
|---|---|
| **Category** | C |
| **Description** | An optional package (not part of the engine) that converts between CivKit's neutral model and ezdxf, shapely and IfcOpenShell objects. IfcOpenShell (LGPL) is imported only if the user installs it. |
| **User problem** | Notebook users already hold these objects. |
| **Inspiration** | ezdxf's `geo` add-on and `__geo_interface__` **[V]**. |
| **CivKit fit** | Lives in `bindings/python` extras or `integrations/jupyter`; never in `crates/`. |
| **Dependencies** | ezdxf (MIT), shapely, IfcOpenShell (LGPL-3.0-or-later). |
| **Offline** | Fully once installed. |
| **Difficulty** | Medium. |
| **Maintenance** | Medium (third-party API drift). |
| **Verification** | Round trips on fixtures. |
| **Strategic value** | Medium; gives IFC access without a Rust IFC reader (see R-1). |
| **0.1 impact** | None. |
| **Needs decision record** | Yes (licence boundary note). |
| **Ambition check** | With care. |

#### FP-25 · Local calculation journal

| Field | |
|---|---|
| **Category** | D (with E) |
| **Description** | An append-only local file (one record per line, each carrying the previous line's hash) that the desktop app and CLI can write if the user asks. Tamper-*evident*, not secure signing. SQLite only if/when D-14 is accepted. |
| **User problem** | Audit trail for a project without a server. |
| **Inspiration** | W3C PROV idea in JSON, not RDF **[I]**; RO-Crate **[V]**. |
| **CivKit fit** | Builds on FP-01; satisfies "data stays local" and "write only where the user asks". |
| **Dependencies** | None beyond a hash. |
| **Offline** | Fully. |
| **Difficulty** | Low–medium. |
| **Maintenance** | Low. |
| **Verification** | Alter any line: chain verification fails. |
| **Strategic value** | Supports "calculation provenance and audit trails" in the brief. |
| **0.1 impact** | None. |
| **Needs decision record** | Yes (interacts with D-14). |
| **Ambition check** | Yes. |

#### FP-27 · `wasm32` portability guard (and, later, a static offline web page)

| Field | |
|---|---|
| **Category** | E (guard) / D (page) |
| **Description** | CI builds the engine crates for `wasm32-unknown-unknown` so portability is not lost by accident. Later, an optional static page (no server, no CDN) could run calculations in the browser from the same engine. |
| **User problem** | Lets someone try CivKit with nothing installed; CivilKit and Stabileo show this works. |
| **Inspiration** | CivilKit Calcs, Stabileo, russell's WebAssembly targets **[V]**. |
| **CivKit fit** | The engine is pure Rust with few dependencies; the guard is nearly free. A web page adds a surface that must obey the offline and CSP rules. |
| **Dependencies** | The `wasm32` target; for the page, `wasm-bindgen` *(not researched)*. |
| **Offline** | Fully, if served as a static, self-contained page. Hosting a public demo is *optional* and free (static) but it is a distribution decision. |
| **Difficulty** | Low (guard); medium (page). |
| **Maintenance** | Low (guard); medium (page). |
| **Verification** | Run the contract test vectors in a WASM runtime (FP-21). |
| **Strategic value** | Adoption and reach; sits between "installable software" and "toy". |
| **0.1 impact** | None for the guard; the page is not planned. |
| **Needs decision record** | Yes for the page (distribution; OFFLINE_POLICY §7 concerns the desktop app). |
| **Ambition check** | Guard: yes. Page: later. |

---

## 2. Categorisation

| Category | Proposals |
|---|---|
| **A — Core improvements** (belong in the Rust engine) | FP-01 record · FP-02 ledger · FP-03 cross-checks · FP-04 hand-check logic · FP-05 sensitivity · FP-06 uncertainty · FP-07 clause metadata · FP-28 unit profiles · (FP-08, FP-13 partly) |
| **B — Optional modules** (no bloat in the core) | FP-08 workbook · FP-11 CSV batch · FP-12 report · FP-13 series and SVG · FP-14 take-off profiles · FP-15 LandXML · FP-16 CRS · FP-17 TIN and volumes · FP-18 continuous beam |
| **C — Integrations** | FP-09 sweeps and batch · FP-19 MCP resources, prompts, schemas · FP-23 Python interop extras · FP-24 Jupyter display |
| **D — Standalone application** | FP-25 local journal · (FP-27 web page, later) |
| **E — Developer ecosystem** | FP-10 CLI · FP-20 typed clients and schema bundle · FP-21 conformance suite · FP-22 differential testing · FP-26 reproducible builds and SBOM · FP-27 `wasm32` guard |
| **F — Reject or defer** | R-1 … R-12 below |

---

## 3. Scoring and prioritisation

### 3.1 Scores (1 = poor, 5 = excellent; judgemental **[I]**)

Columns: **EU** engineering usefulness · **SD** strategic differentiation · **AF** architectural fit · **OC** offline
compatibility · **DF** development feasibility (5 = easy) · **MS** long-term maintenance sustainability (5 = light
burden). Total out of 30. *A high total never overrides a safety or architecture constraint*: FP-17 and FP-18 score
lower on feasibility and sustainability exactly because the constraints, not the usefulness, are the issue.

| ID | Feature | EU | SD | AF | OC | DF | MS | **Total** |
|---|---|---|---|---|---|---|---|---|
| FP-01 | Calculation record + replay | 4 | 5 | 5 | 5 | 4 | 4 | **27** |
| FP-02 | Verification ledger | 4 | 4 | 5 | 5 | 4 | 4 | **26** |
| FP-03 | Built-in cross-checks | 5 | 4 | 4 | 5 | 4 | 4 | **26** |
| FP-04 | Hand-calculation checker | 4 | 4 | 5 | 5 | 4 | 4 | **26** |
| FP-09 | Sweeps and batch | 4 | 2 | 5 | 5 | 5 | 5 | **26** |
| FP-10 | CLI | 4 | 2 | 5 | 5 | 5 | 5 | **26** |
| FP-19 | MCP resources, prompts, schemas | 4 | 4 | 5 | 5 | 4 | 4 | **26** |
| FP-21 | Conformance suite | 3 | 4 | 5 | 5 | 4 | 5 | **26** |
| FP-11 | CSV/XLSX batch | 4 | 3 | 5 | 5 | 4 | 4 | **25** |
| FP-20 | Typed clients, schema bundle | 4 | 3 | 5 | 5 | 4 | 4 | **25** |
| FP-28 | Unit profiles | 3 | 2 | 5 | 5 | 5 | 5 | **25** |
| FP-08 | Workbook | 5 | 4 | 4 | 5 | 3 | 3 | **24** |
| FP-14 | Take-off profiles + audit | 5 | 3 | 5 | 5 | 3 | 3 | **24** |
| FP-22 | Differential testing | 4 | 2 | 5 | 5 | 4 | 4 | **24** |
| FP-24 | Jupyter display | 3 | 2 | 5 | 5 | 5 | 4 | **24** |
| FP-07 | Clause framework + coverage | 4 | 3 | 4 | 5 | 4 | 3 | **23** |
| FP-12 | Reports | 5 | 3 | 4 | 5 | 3 | 3 | **23** |
| FP-15 | LandXML reader | 4 | 4 | 4 | 5 | 3 | 3 | **23** |
| FP-25 | Local journal | 3 | 3 | 4 | 5 | 4 | 4 | **23** |
| FP-26 | Reproducible builds, SBOM | 3 | 3 | 5 | 5 | 3 | 4 | **23** |
| FP-27 | `wasm32` guard (+ page) | 3 | 3 | 4 | 5 | 4 | 4 | **23** |
| FP-05 | Sensitivity | 4 | 5 | 3 | 5 | 2 | 3 | **22** |
| FP-06 | Uncertainty | 4 | 4 | 3 | 5 | 3 | 3 | **22** |
| FP-13 | Plot series + SVG | 4 | 2 | 4 | 5 | 3 | 4 | **22** |
| FP-16 | CRS, ground-to-grid | 4 | 3 | 3 | 5 | 3 | 3 | **21** |
| FP-23 | Python interop extras | 3 | 3 | 4 | 5 | 3 | 3 | **21** |
| FP-17 | TIN and surface volumes | 5 | 3 | 3 | 5 | 2 | 2 | **20** |
| FP-18 | Continuous beam / frames | 4 | 2 | 3 | 5 | 2 | 2 | **18** |

### 3.2 Complexity, dependencies, testing, maintenance

| ID | Complexity | New dependencies | Testing needs | Maintenance burden |
|---|---|---|---|---|
| FP-01 | Medium | none (hash optional) | parity across interfaces; tamper tests | Low |
| FP-02 | Low–medium | none | harness extension | Low |
| FP-03 | Low per calc | none | planted-error tests | Low–medium |
| FP-04 | Low | none | slip table tests | Low |
| FP-05 | Medium–high | `num-dual` (option 2) | analytic derivatives | Medium |
| FP-06 | Medium–high | `inari` (no `gmp`) | containment tests | Medium |
| FP-07 | Low (meta) / high (packs) | none | coverage CI | High per code |
| FP-08 | Medium | none | equals hand-chained calls | Medium |
| FP-09 | Low | none (pandas optional) | `batch == loop` | Low |
| FP-10 | Low | arg parser | golden vectors | Low |
| FP-11 | Low / medium | `csv`, XLSX crates (not researched) | round trips | Low / medium |
| FP-12 | Medium | PDF engine later (not researched) | golden files | Medium |
| FP-13 | Medium | none | snapshots | Medium |
| FP-14 | Medium | DXF reader (Phase 4) | fixtures + ezdxf oracle | Medium |
| FP-15 | Medium–high | XML parser | open fixtures | Medium |
| FP-16 | Medium–high | `proj4rs` + grids | geodetic test points | Medium–high |
| FP-17 | High | triangulation crate | closed-form surfaces | High |
| FP-18 | High | `nalgebra` | textbook cases | High |
| FP-19 | Low–medium | `rmcp` | schema validation, Inspector | Medium |
| FP-20 | Low–medium | none | regenerate-and-diff | Low–medium |
| FP-21 | Low | none | itself | Low |
| FP-22 | Low–medium | dev-time Python only | self-detecting planted bugs | Low–medium |
| FP-23 | Medium | ezdxf, shapely, IfcOpenShell (optional) | round trips | Medium |
| FP-24 | Low | none | snapshots | Low |
| FP-25 | Low–medium | none | chain verify | Low |
| FP-26 | Medium | SBOM tool (not researched) | rebuild equality | Medium |
| FP-27 | Low / medium | `wasm32` target | vectors in WASM | Low / medium |
| FP-28 | Low | none | per-dimension registry test | Low |

---

## 4. The three shortlists

### 4.1 High value, low complexity: candidates for relatively early releases

*Early* means "after 0.1 is out", not "inside 0.1".

| Order | Proposal | Why it comes early |
|---|---|---|
| 1 | **FP-01 record schema** (decision before Phase 3; implementation soon after 0.1) | Everything else in the trust pillar builds on it; the schema is cheaper to fix before three interfaces emit different shapes |
| 2 | **FP-19 MCP design** (resources, prompts, output schemas) | Decide the surface at Phase 3 start; the MCP spec moved a revision in 2026 |
| 3 | **FP-10 CLI** | Tiny; becomes the reference client and a scripting entry point |
| 4 | **FP-21 conformance vectors** and **FP-20 schema bundle** | Make the contract enforceable; nearly free once parity tests exist |
| 5 | **FP-04 hand-calculation checker** and **FP-03 cross-checks** | Visible trust wins; directly address the AI-trust lane |
| 6 | **FP-02 ledger** | Shape for the generated doc page that 0.1 already requires |
| 7 | **FP-09 sweeps / batch**, **FP-24 Jupyter display**, **FP-28 unit profiles** | Small ergonomic gains |
| 8 | **FP-22 differential testing**, **FP-27 `wasm32` guard** | Cheap insurance for quality and portability |
| 9 | **FP-11 CSV batch**, **FP-14 take-off profiles** (with Phase 4) | Practical adoption |

### 4.2 High value, long-term: worth planning, not building yet

| Proposal | Prerequisite and stop condition |
|---|---|
| **FP-12 reports** | After FP-01. HTML and Markdown first; PDF only if a free, offline route is found. |
| **FP-08 workbook** | After FP-01. *Stop if anyone asks for expression syntax.* |
| **FP-05 → FP-06 sensitivity and uncertainty** | Prototype option 1 (numerical differences) on one calculation first. *Do not change `compute` until the prototype proves its worth.* |
| **FP-07 clause framework** | Metadata now is cheap; packs follow D-8, one code. |
| **FP-15 LandXML** | After Phase 5; **verify the schema licence first**. |
| **FP-13 series and SVG** | When the desktop app needs diagrams. |
| **FP-25 journal**, **FP-26 reproducible builds**, **FP-23 Python interop extras** | When there are users to ask. |
| **FP-16 CRS** | Only when a survey user asks. |
| **FP-17 TIN and volumes**, **FP-18 continuous beam** | **Needs experienced contributors.** Not for the current team. |

### 4.3 Interesting but not worth pursuing (see §5)

R-1 … R-12 below. Headline: no solver, no CAS, no worksheet language, no plugin runtime, no cloud, no embedded AI.

### 4.4 A suggested sequence **[I]**

1. **Before Phase 3 starts** (decisions, no code): record schema (FP-01); MCP surface (FP-19, D-13 against the new spec
   revision); whether the CLI is part of 0.1 (FP-10); optional-crate dependency rule.
2. **0.1:** unchanged scope. Generated doc pages shaped as a ledger (FP-02).
3. **0.2–0.3:** FP-01 implementation, FP-10, FP-20, FP-21, FP-04, FP-03, FP-09, FP-24, FP-11, FP-14 profiles.
4. **0.3–0.4:** FP-12 reports, FP-08 workbook, FP-25 journal, FP-07 metadata and first code pack.
5. **Beyond:** FP-05/06, FP-15, FP-13, FP-16 as users arrive.

### 4.5 Decision records this would eventually need

| Topic | Proposals | Touches |
|---|---|---|
| Calculation Record schema | FP-01, FP-26 | API_DESIGN §3, §11 |
| `checks` in the response | FP-03 | API_DESIGN §4–§6 |
| Structured clauses in descriptors | FP-07 | API_DESIGN §2 |
| MCP surface | FP-19, FP-04 | API_DESIGN §10, D-13 |
| Optional-crate dependency rule | FP-11, 12, 15, 16, 17 | ARCHITECTURE §2, `deny.toml` |
| `compute` genericity (only after a prototype) | FP-05, FP-06 | D-19, `civkit-core` |
| Name checks (reopen D-1 only if risks materialise) | none | D-1, landscape §10 |

---

## 5. Rejected and deferred ideas (Category F)

| # | Idea | Where it came from | Why it is rejected or deferred | Revisit when |
|---|---|---|---|---|
| **R-1** | **Pure-Rust IFC reader/writer** | Stabileo, Pondus and Open Infra all touch IFC **[V]** | IFC is huge; no pure-Rust IFC crate was found (the search page was blocked: **[?]**); IfcOpenShell is LGPL (cannot link); the IFC docs are CC BY-ND 4.0 (implementation reported royalty-free, from a secondary source). Years of work for a thin slice. Use IfcOpenShell *as an optional Python extra* (FP-23) instead. | A volunteer with BIM experience and a stable IFC4.3 subset (quantities only). |
| **R-2** | **Declarative calculation definition language** (users write calculations without Rust) | CalcpadCE, CivilKit worksheets; OpenMDAO components | Needs an expression evaluator with units, i.e. a second CivKit inside CivKit; the niche is occupied by CalcpadCE (MIT) and CivilKit (MIT); trust model gets harder (who verifies a user's expression?). The existing contributor path (Rust + reference cases first) already protects trust. | After FP-08 proves chains are not enough, and a safe subset can be listed. |
| **R-3** | **Plugin runtime** (dynamic libraries, WASM plugins) for third-party calculations | Open Infra's plugin API (version-pinned to host builds, a maintenance cost the project itself shows) **[V]** | Permanent ABI and security surface; extension already exists at source level (registered descriptor + reference cases). | A real third party wants out-of-tree calculations and has a sandboxing story. |
| **R-4** | **General FE solver** (3D, plates, nonlinear, dynamic) | OpenSees, Stabileo, Pondus, PyNite | Six projects occupy it; Stabileo alone lists 5,655 engine tests and five known mismatches. A first-year project cannot verify one. Wrong results here are a safety risk. | Never as a goal. The *contract* could front someone else's solver. |
| **R-5** | **Built-in symbolic CAS** | CivilKit `caskit`; Symbolica | Scope explosion. Symbolica is source-available and forbids use as a reference (§4(4)–(5)); `caskit`'s own limits (degree ≤ 2, no complex) show the cost. SymPy exists in Python; use it as an oracle. | Never in the engine. |
| **R-6** | **Cloud, accounts, hosted calculators, real-time collaboration** | Open Infra "SuperCloud" plans; hosted CivilKit MCP | Violates OFFLINE_POLICY and "no ongoing paid infrastructure". | Not applicable; any exception needs a decision record by policy. |
| **R-7** | **Embedded LLM / in-app AI assistant** | Pondus, Stabileo AI | Makes AI a runtime dependency (forbidden), or pays for one. MCP already lets the *user's* AI call CivKit. | Never. |
| **R-8** | **DWG read/write** | `acadrust` (MPL-2.0, DWG R12–R2018+, a port of an MIT C# project; 0.x) **[V]** | The format is proprietary; ODA terms unverified **[?]**; MPL-2.0 is outside `deny.toml`; maturity low. DXF and a DXF export from AutoCAD is enough. | After the DXF reader is stable and a licence review is done. |
| **R-9** | **Many design codes early** | Pondus (9+ codes), Stabileo | Each code is a long-term maintenance and copyright-handling commitment; D-8 says one at a time after 0.1. | After 0.1 and with a user who needs a specific code. |
| **R-10** | **Writing to drawings** | Open Infra writes entities; FreeCAD MCPs | S-5 and SECURITY: any write needs host confirmation. | After read-only analysis is trusted; separate decision. |
| **R-11** | **Mathcad import / worksheet language** | CivilKit Calcs, CalcpadCE | Occupied, MIT-licensed and young; CivKit's differentiator is the contract, not a worksheet. | Never as a goal; a *workbook* (FP-08) is the intentionally smaller alternative. |
| **R-12** | **Hosted public demo / web calculators as a product** | CivilKit Calcs | The WASM guard (FP-27) keeps the option open; a *hosted product* becomes an operating duty, a brand collision risk (landscape §10) and a second front. | When the name question is settled and a maintainer can own it. |

---

## 6. Caveats

- Scores and "strategic value" statements are **interpretation [I]**; they would change with user research. The most
  valuable single next step is probably not a feature but **five conversations with real engineers or lecturers** about
  FP-01, FP-04 and FP-12.
- Several gaps behind the proposals (G1, G4, G6, G8 in the landscape report) were assessed against ~25 projects,
  not the commercial market.
- No proposal here has been prototyped. Difficulty ratings are estimates; the stop conditions in §4.2 are the
  safeguard.
- Nothing in this document is legal advice (licences, trademarks, standards).
- No code was written, no dependency installed, no accepted decision modified and no change made to MVP_SCOPE.md.
