# Competitive Landscape

> **Provenance:** Claude Sonnet 5.5 (`claude-sonnet-5-5`) · first drafted 2026-11-25 22:25 IST · last edited 2026-11-25 22:25 IST

> **Status:** Research report. **Not a decision.** Nothing here changes an accepted decision, the 0.1 scope
> ([MVP_SCOPE.md](MVP_SCOPE.md)) or any code. Every change to an accepted decision is labelled **Proposal** and needs
> its own decision record ([DECISIONS.md](DECISIONS.md)). Companion document: [FEATURE_PROPOSALS.md](FEATURE_PROPOSALS.md).
>
> **Research date: 2026-11-25.** Everything on the internet below was read on that day. Star counts, release
> dates and commit dates move quickly; re-check anything before relying on it.

## How to read the evidence labels

| Label | Meaning |
|---|---|
| **[V]** | **Verified.** Read on a primary source (the project's own repository, licence file, documentation or registry) on 2026-11-25. |
| **[S]** | **Summary.** From a page that was only partly read verbatim (long pages were truncated and the rest summarised by a model). Treat as probable, not certain. |
| **[I]** | **Interpretation.** A conclusion drawn from verified facts. Reasonable, but it is my reasoning. |
| **[H]** | **Hypothesis.** A guess that would need further research or a user test to confirm. |
| **[?]** | **Unverified.** Looked for it, could not confirm either way. *Not* the same as "does not exist". |

Marketing text was not accepted as evidence of a feature. Where a project's own claim is the only source
(for example its self-reported test counts), that is said. No AI-generated statement was treated as verification;
reports from research helpers were cross-checked where it mattered (§11).

---

## 1. Executive summary

**What was done.** The repository's planning and architecture documents were read first. Then 20-odd projects
were examined: the three named competitors, the named structural and GIS projects, and libraries from units,
uncertainty, geometry, CAD/BIM interoperability, calculation reporting and MCP.

**Main findings.**

1. **The field moved a lot in the last six weeks.** Most of the closest competitors are young (first commits October
   2026), have one visible maintainer and were written with AI assistance. All of them have just discovered the
   same idea CivKit started from: *Rust engine, offline, MCP-accessible* **[V]**. "Rust + offline + MCP" is therefore
   **no longer a differentiator on its own**. It is table stakes.
2. **CivilKit (civilkit.io) is a much closer neighbour than the name check in D-1 knew.** It now offers CivilKit
   Calcs (Rust → WebAssembly worksheet engine, MIT, a *hosted* read-only MCP server) and CivilKit CAD (an
   Apache-2.0 B-rep kernel in Rust/WebAssembly) **[V]**. The names differ by two letters; the audience, language and
   pitch overlap. This is the largest non-technical risk found (§10).
3. **Nobody examined offers a versioned, replayable *calculation record*.** Some tools show steps (worksheets,
   `handcalcs`), some check a hand calculation (the hosted CivilKit MCP), some publish benchmarks (Stabileo), some
   record data inside a Python optimisation graph (OpenMDAO). None of those produces a small, self-describing document
   that says "this exact calculation, at this exact version, with these exact inputs and units, produced this" and can
   be re-run offline to confirm it **[I, subject to the limits of this search; §11]**.
4. **Units are weak almost everywhere in the solver world.** `anaStruct` and `PyCBA` take bare floats (verified from
   their README examples); Stabileo's benchmark file records a case where a Young's modulus 1000× too stiff gave a
   misleading answer **[V]**. CivKit's rule "a dimensional input without a unit is an error, ambiguous units are
   rejected" has no equivalent among the projects examined **[I]**.
5. **The licence landscape rules out copying and shapes what can be reused.** AGPL-3.0 (Pondus, Stabileo, PyCBA since
   June 2026), GPL-3.0 (Open Infra Studio), LGPL (`anaStruct`, IfcOpenShell), a non-commercial-style licence
   (OpenSees) and a source-available licence that *forbids using it as a reference* (Symbolica) all mean: **independent
   re-implementation from textbooks and published methods, which D-18 already requires, is the right posture**, and
   some source should not even be read for design input (§6).
6. **AI trust is the open lane, and the hosted competitor does not occupy it.** The hosted CivilKit MCP has a
   `calcs_check_hand_calc` tool but runs on civilkit.io (rate-limited, requires a connection) **[V]**. Pondus's MCP
   writes models and files. Open Infra's MCP edits drawings. A *local, read-only, schema-typed* MCP that returns the
   engine's own trace and refuses to guess units is a distinct position **[I]**.

**The answer to the final question** (expanded in §8): CivKit can become **the verifiable, interface-independent
calculation layer for engineering**: not "more calculators", not another solver, not another worksheet language, but
the thing the other tools *could call* when a number has to be trusted. Concretely, three properties together:

- a **calculation record** (inputs with units, calculation ID + version, trace, warnings, references, engine
  fingerprint) that can be stored, diffed and **re-verified offline** by anyone (FP-01);
- a **verification ledger** published with every calculation, so "how do we know this is right?" has a machine-readable
  answer (FP-02, FP-03, FP-04);
- **one contract reachable from every host**: Python, CLI, local MCP, AutoCAD, desktop, with conformance tests anyone
  can run (FP-10, FP-19, FP-20, FP-21).

All of this fits the current architecture with small, additive changes. **None of it needs to enter 0.1.** Two
decisions are cheaper *before* Phase 3 than after (the record schema and MCP output schemas); everything else can wait
(§9, [FEATURE_PROPOSALS.md](FEATURE_PROPOSALS.md)).

**What not to do.** Do not build a worksheet language (CalcpadCE and CivilKit occupy it), a general FE solver
(six projects occupy it, three of them with large benchmark suites), a CAS (licence and scope), or anything that needs a
server. "Ambition without project suicide" means the opportunity is *depth of trust*, not *breadth of features*.

---

## 2. Market overview

### 2.1 Segments

| Segment | Examples (verified) | What it does well | Where it stops |
|---|---|---|---|
| **Calculation sheets / notebooks** | Mathcad (proprietary), CalcpadCE (MIT), CivilKit Calcs (MIT), `handcalcs` (Apache-2.0) | Readable, shows the working, report export | Each worksheet is its own program; no shared contract; verification is the author's job |
| **Structural analysis apps** | Pondus (AGPL), Stabileo (AGPL), OpenSees (restricted) | Real solvers, design codes, reports | Large, single-maintainer or institution-bound; licences prevent reuse; units mostly the caller's problem |
| **Structural Python libraries** | PyNite (MIT), anaStruct (LGPL), PyCBA (AGPL), `sectionproperties` / `concreteproperties` (MIT), `structuralcodes` (Apache-2.0) | Scriptable, free, good for research and teaching | Python-only; bare floats; no cross-language contract |
| **Infrastructure / GIS inside CAD** | Open Infra Studio (GPL-3.0) on Open CAD Studio | Surveys, alignments, terrain, corridors, LandXML/IFC output | Needs a specific host; most features call online services; Dutch focus |
| **Terrain / GIS analysis** | WhiteboxTools (MIT legacy; open-core "Next Gen") | 700+ geospatial tools, pure Rust | Not a calculation contract; open-core split |
| **Interoperability libraries** | ezdxf (MIT), IfcOpenShell (LGPL), `dxf` (MIT), `acadrust` (MPL-2.0) | Read/write real files | Libraries, not analysis; different licences and languages |
| **AI-connected engineering** | Pondus MCP (46 tools), Open Infra MCP (39), hosted CivilKit MCP (11), several FreeCAD MCP servers | Let an agent drive or query engineering software | Trust model is thin: no shared notion of "this number is verified" |

### 2.2 Trends visible in the evidence **[I]**

- **Rust + WebAssembly for engineering engines** is now common: CivilKit Calcs, CivilKit CAD kernel, Stabileo,
  Whitebox, Open Infra **[V]**.
- **MCP is becoming a standard surface.** The MCP specification now has a revision dated **2026-07-28**, which is
  stateless, adds output schemas and structured content, and deprecates roots and sampling **[V]**. The Rust SDK
  (`rmcp`) is at 3.11 (17 Nov 2026) with about 14 million downloads a month **[V]**.
- **Licences are hardening.** `PyCBA` relicensed from Apache-2.0 to AGPL-3.0 in June 2026; Calcpad itself went
  closed-source in March 2026 (CalcpadCE is the MIT fork) **[V/S]**. A permissively licensed project that others can
  safely build on is scarcer than it was.
- **Trust is the unsolved problem.** The projects that discuss verification (Stabileo's BENCHMARKS.md, Pondus's
  calibration, CivilKit's hand-check tool) do so per *project*, not per *result*.

### 2.3 Maturity warning

Of the three named competitors, none is older than about two months in its current form. CivilKit Calcs has 613
commits and 9 stars; Pondus has 32 commits and 13 stars; Open Infra Studio has 728 commits and 34 stars **[V/S]**.
Stabileo (about 10,400 commits, 128 stars) and OpenSees (9,576 commits, decades of use) are the established ones.
A "competitor" with 13 stars is not a threat today; it is evidence of **where the ideas are going**.

---

## 3. Competitor profiles

Each profile follows the questions in the research brief: identity, architecture, verified functionality, strengths
and weaknesses, licence. "Limits" lists *stated* limitations, which are confirmed, separately from things not found.

### 3.1 CivilKit Calcs and the CivilKit family **(direct, name-adjacent)**

| | |
|---|---|
| **Sources** | github.com/mageaustralia/civilkit-calcs (README, LICENSE, docs/caskit.md), civilkit.io/calcs/learn, civilkit.io/mcp, github.com/civilkit organisation page **[V/S]** |
| **Identity** | "Free engineering calculation notebook in the browser" and a self-described Mathcad alternative. Users write worksheets in the **CalcpadCE language** (equations with units, matrices, plots, forms). README says *"Status: early"* **[V]** |
| **Users** | Engineers who want free Mathcad; Excel users; students (MATLAB-homework guide) **[I]** |
| **Architecture** | Engine: **Rust compiled to WebAssembly**, a port of CalcpadCE's engine; frontend Vue + Monaco; separate wasm modules for docx/xlsx, linear algebra, a small symbolic module (`caskit`), worksheet import. Runs from static files; installable as a PWA **[V]**. CLI binary compatible with CalcpadCE batch mode. Crates published (`civilkit-calcs-engine`, 0.1.0, MIT, 38 downloads) **[V]** |
| **Offline** | Calculation runs in the page; "no calculation request leaves the browser" (README claim). Plotly loads from a CDN, so plots need a connection (project's own test log) **[S]** |
| **Functionality** | 17 fill-in calculators (beams, column buckling, principal stresses, retaining wall, footing pressure, pipe flow, pump point, PID, filters, robot arm, RSA …) described as *"generic textbook calculations, not design-code checks"* **[V]**. Mathcad `.xmcd/.mcdx` import. Export: PDF (browser print), Word, Excel with charts, HTML. **No DXF, no CSV export mentioned.** |
| **MCP** | **Hosted** at `civilkit.io/mcp`, read-only, no account, 11 tools, including **`calcs_check_hand_calc`** (recompute a hand calculation and compare), plus resources and prompts. Limits: 64 KB sheets, 120 requests/minute/IP. "No request bodies logged" is the page's own statement, not independently verifiable **[V]** |
| **Verification** | Self-reported: 6,129 cargo tests (transcribed from CalcpadCE's), about 40 browser scripts. README admits some checks failed or were not run in recent releases; Excel/Word output not opened in Microsoft Office **[S]** |
| **Strengths** | Zero install; permissive licence; Mathcad import is a clever adoption path; embeddable calculators; hand-check tool is exactly the right *idea* for AI trust |
| **Weaknesses / limits** | Early; single visible maintainer; every calculation is a *worksheet*, not a versioned contract entry; no design codes (stated); no CAD/DXF; MCP is a hosted service; symbolic module limited (degree ≤ 2 solves, no complex numbers) **[V]** |
| **Licence** | **MIT** (plain text, no added terms) **[V]**. Built on CalcpadCE (MIT). |
| **The wider family** | civilkit.io also lists **CivilKit CAD** (parametric CAD, "Preview") and the org hosts `cadkit-kernel` (B-rep kernel, Rust/WASM, **Apache-2.0**), `cadkit-kit`, `civilkit-parts` (Apache-2.0) **[V]**. The README says model bindings ("connect a structural model") do not yet work **[V]**. |
| **Open question** | The GitHub page for the repo resolved under owner `civilkit` while README and crate metadata point to `mageaustralia`; probably a transfer or mirror, not confirmed **[S]**. |

**Reading for CivKit [I].** Complementary more than head-to-head: CivilKit Calcs is a *front end* (worksheets); CivKit
is a *contract and engine*. The overlap is the pitch ("free, offline, Rust, units, MCP"). The risk is brand and
discoverability (§10), not technology.

### 3.2 Open Infra Studio **(architectural competitor)**

| | |
|---|---|
| **Sources** | open-aec.com/open-infra-studio (Dutch, translated), /roadmap, github.com/OpenAEC-Foundation/Open-Infra-Studio (README, LICENSE, releases, mcp/README.md) **[V/S]** |
| **Identity** | Two plugins (GIS tab, Civiel tab) for **Open CAD Studio**, a free community CAD that reads/writes DWG/DXF. For infrastructure engineers, surveyors, Dutch BIM/GIS users **[V]** |
| **Architecture** | One Rust workspace. Core with *no CAD dependency* (coordinates, services, import, civil maths) plus a separate layer mapping results to CAD entities. Plugin = `plugin.toml` + native library per platform; plugin API **version-pinned to host builds** ("does not load in older versions") **[V]**. MCP server in Python (stdlib only), 39 tools (README) or "about thirty" (FAQ) **[V]** |
| **Functionality** | Survey points, LandXML, TIN, contours, alignments with clothoids, profiles, cross-sections, corridors with earthwork mass balance, sewer design, KLIC/BGT/BRO Dutch datasets, 600+ map services **[V]**. Reads GeoJSON, Shapefile, GeoPackage, LAS/LAZ, LandXML 1.x, CityGML; writes IFC 4.3, LandXML 1.2, GeoPackage, PDF sheets **[V]**. Not supported: IFC import, Shapefile export **[V]** |
| **Offline** | **Mostly no.** Most GIS features call PDOK, BRO, 3D BAG, OSM, Nominatim, epsg.io, WMS/WFS; no offline section in README **[V]**. No product account. |
| **MCP safety** | Not stated as read-only; most tools modify the drawing. HTTP mode needs a bearer token off loopback **[V]** |
| **Verification** | About 29,000 lines of Rust and 120 unit tests (README); one cited cross-check: corridor end-area volumes agree with a TIN-to-TIN volume within 2 % **[V]**. Geotechnics is "a calculation aid, not a stamped design" **[V]** |
| **Strengths** | Real civil workflows (alignment, corridor, earthwork), good format coverage, signed releases, clean "no CAD dependency in core" separation (the same idea as ADR 0002) |
| **Weaknesses** | Host-dependent; online-dependent; GPL-3.0; documentation partly stale (website vs release, README contradicts itself on export); Dutch-first; macOS workbench not available **[V]** |
| **Licence** | **GPL-3.0** (unmodified FSF text) **[V]**. Cannot be linked into an Apache-2.0 library. Data licences vary (Sentinel-2 imagery is non-commercial CC BY-NC-SA) **[V]**. |

**Reading for CivKit [I].** It proves the "civil maths separate from CAD" architecture and shows what an
infrastructure engineer actually does (alignment → profile → corridor → volumes). It also shows the cost of being
online-first and plugin-version-pinned. **Idea worth taking (independently): LandXML as the bridge from Civil 3D**
(FP-15).

### 3.3 Pondus **(AI-native structural design)**

| | |
|---|---|
| **Sources** | strpondus.github.io (site and manual), github.com/Sompote/pondus (README, LICENSE, releases, commits, docs/mcp.md) **[V/S]** |
| **Identity** | "Structural analysis and design you can talk to": desktop program (macOS, Windows; Linux from source) with a built-in agent chat and an MCP server (`pondus --mcp`, 46 tools). Reinforced concrete, steel, timber, masonry, cold-formed; Thai-market focus **[V/I]** |
| **Architecture** | Rust workspace: `core/` (model, analysis, design codes, reports, MCP), `app/` (egui GUI), vendored **Stabileo** engine (AGPL), vendored agent harness (Apache-2.0). Optional CUDA. Files: `.pondus`, Excel, DXF R12 (geometry only), IFC4 (members, slabs, walls), PDF reports **[V]** |
| **Offline** | "Everything except the agent works offline and without an account." The agent needs an OpenAI-compatible provider key (local). Remote use via a public tunnel is documented with the warning that anyone with the URL can drive the program **[V]** |
| **Functionality** | Linear static 3D frames, slabs, shear walls, P-Delta, buckling, modal, response spectrum, time history, pushover (indicative), moving loads; design to ACI 318-19, AISC 360-16 / EN 1993, EN 1992 cross-check, NDS, TMS 402, AISI S100, ASCE 7-22, Thai codes **[V]** |
| **Verification** | Self-reported 470 benchmark comparisons (430 within 0.5 %) and 199 SAP2000/ETABS comparisons (177 within 0.5 %); not independently verified. Windows build "cross-compiled, not run on Windows" at 0.1.0 **[S]** |
| **Strengths** | Complete workflow from model to PDF report; broad code coverage for a two-month-old project; honest "Limits and safety" page |
| **Weaknesses** | Single maintainer co-authored with an AI account; 13 stars; internally inconsistent docs (site offers 0.1.4, repo is 0.1.5; manual and release notes disagree on punching shear); AGPL; MCP writes models and files, not read-only **[V]** |
| **Licence** | **AGPL-3.0** **[V]** |

**Reading for CivKit [I].** It validates demand for *AI + engine*, and shows the dangerous version: an agent that can
design, with verification being "check by a qualified engineer". CivKit's differentiator is the opposite trust model:
the agent never computes, the engine does, and the result carries its own evidence.

### 3.4 Stabileo **(the strongest verification story)**

AGPL-3.0, Rust → WebAssembly engine with Svelte UI, optional Elixir backend, about 10,400 commits, 128 stars, latest
commit 2026-11-25 **[V]**. Basic mode (free) covers 2D/3D frames, diagrams, buckling, modal, influence lines; PRO
mode adds plates/shells, DXF/Excel/IFC import and nonlinear/dynamic analysis **[V]**. Design checks for AISC 360-22,
ACI 318-19, EC2, EC3, NDS, AISI, TMS 402 are listed in its benchmark file **[V]**.

**Why it matters to CivKit:** `docs/BENCHMARKS.md` is a *ledger*: 5,655 engine-coupled tests, about 1,688 textbook
tests, suites from the ANSYS Verification Manual, NAFEMS, Code_Aster, SAP2000 and MASTAN2, cross-checks against
OpenSees and others, **and an explicit list of five items that do not yet match and a known twisted-beam issue**
**[V]**. The README also claims a post-solve error bound proved sound in Lean; the supporting document was not read
**[?]**. This is the best example found of publishing *what is and is not verified*. One caution: a recent commit
title suggests the editor may now require an account, contradicting "no account"; not confirmed **[?]**.

### 3.5 OpenSees / OpenSeesPy

C++ finite-element framework from UC Berkeley with Tcl and Python interfaces (OpenSeesPy 3.8.0.0, Python 3.12), latest
commit 2026-11-23, 9,576 commits **[V]**. The repository has **no LICENSE file**; terms are in `COPYRIGHT`: free for
education, research and non-profit use; others "for internal purposes only"; **incorporating it into commercial
products requires permission from the University of California's licensing office** **[V]**. OpenSeesPy docs say
commercial redistribution, including a cloud service that imports it, needs a licence **[V]**. Element and material
counts, unit handling and test coverage: **[?]** (not read).

**For CivKit:** not reusable, not copyable. Whether an independent re-implementation of published element formulations
is outside those terms is a legal question; the policy answer is the one D-18 already gives (derive from open
textbooks, never from its source).

### 3.6 PyNite, anaStruct, PyCBA, sectionproperties, concreteproperties, structuralcodes

| Project | Licence **[V]** | Activity | What it verifiably does | Units | Verification evidence |
|---|---|---|---|---|---|
| **PyNite** (PyniteFEA) | MIT | 797 stars; commit 2026-11-21 | 3D frames, P-Delta, modal, pushover (steel only), plates (DKMQ), load combinations, diagrams, HTML/PDF reports, pyvista rendering | Not mentioned **[?]** | README: tested against textbook problems in CI; tests not read |
| **anaStruct** | LGPL-3.0 | 477 stars; commit 2026-11-23 | 2D frames and trusses, diagrams, geometric non-linearity, steel section database | **Bare numbers** (README examples) | CI badge; no verification manual referenced |
| **PyCBA** | **AGPL-3.0** (was Apache-2.0 until 1.0.1, June 2026) | 130 stars; commit 2026-09-15 | Continuous beams, moving-load bridge assessment with national load models (AASHTO, Eurocode, AS 5100 …), influence lines, hinges, modal | **Unitless solver**; display unit systems for labels only | README: 370+ tests vs closed-form; not read |
| **sectionproperties** | MIT (label/README; file read failed) | 574 stars; last commit April 2026 | Section properties, torsion, shear centre, stresses, DXF/Rhino import; JOSS paper | Not stated **[?]** | User guide has a Validation section (Pilkey, Peery examples) |
| **concreteproperties** | MIT (label/README) | 255 stars; commit 2026-09-17 | RC/composite/prestressed sections, moment-curvature, M-N interaction, AS 3600 / NZS 3101 | `UnitDisplay` class; semantics **[?]** | Assumptions page; disclaimer puts verification on the user |
| **structuralcodes** (fib) | **Apache-2.0** | 313 stars; commit 2026-10-29 | Eurocode 2 (2004, 2023), fib Model Code 2010 materials, shear, cracking, creep/shrinkage; section calculator | **[?]** | tests and coverage tooling present; benchmark vs published values **[?]** |

**Reading for CivKit [I].** The *library* niche is crowded in Python and mostly licence-clean only for PyNite,
sectionproperties, concreteproperties and structuralcodes. None publishes a cross-language contract. `structuralcodes`
is the closest in spirit to CivKit's "design-code module" plan (D-8) and is the right project to **study for how a
code-clause library is organised** (it is Apache-2.0); how it treats copyrighted code text was not verified **[?]**.

### 3.7 WhiteboxTools / Whitebox Next Gen

- **Legacy repo** `jblindsay/whitebox-tools`: **MIT**, "pure Rust code"; hydrology, terrain, LiDAR tools; CLI, Python
  API, GUI runner; README marks it legacy **[V]**.
- **Whitebox Next Gen**: README says MIT; both `LICENSE-MIT` and `LICENSE-APACHE` exist; "the majority of the 700+
  tools are open source"; a **paid "Pro" extension** sits outside the workspace (annual per-user subscription, 41
  workflows) **[V]**. Backend crates (`wbgeotiff`, `wbprojection`, `wbraster`, `wbvector`, `wbtopology`, `wblidar` …)
  are published; per-crate licences and whether the open tier does any licence check at runtime were **not verified**
  **[?]**.

**For CivKit [I].** A cautionary and a positive example: an open-core split that is now a subscription (exactly what
CivKit's offline policy rules out), and a body of permissively licensed Rust geospatial crates that are candidates for
*later* optional modules (FP-16, FP-17), provided each crate's licence is checked individually.

### 3.8 Other relevant projects (briefly)

| Project | Relevance | Licence | Notes |
|---|---|---|---|
| **CalcpadCE** | The engine lineage behind CivilKit Calcs; the "worksheet language" niche | MIT **[V]** | README: fork of Calcpad 7.6.2 after the original went closed-source in March 2026 **[S]** |
| **handcalcs** | Renders Python calculations as symbolic → substituted → result; Markdown, LaTeX, HTML, Excel, Word | Apache-2.0 **[V]** | 2.0.0 on 2026-11-24. Its README warns out-of-order notebook cells can give "a beautiful but completely incorrect" calculation **[V]**. |
| **OpenMDAO** | Component graph with units, derivatives, recorders (NASA) | Apache-2.0 **[V]** | Docs index lists "Recording Data", "Units Definitions", "Total Derivatives"; contents not read **[?]** |
| **Pint** | Python runtime units | BSD-3-Clause **[V]** | Runtime Quantity objects, offset-unit support; ambiguity handling (`lb`, `ton`) **[?]** |
| **`uom`**, **`dimensioned`** | Rust *compile-time* units | Apache-2.0/MIT; MIT **[V]** | Compile-time only. Cannot serve JSON inputs from Python/MCP/C#, which is why D-5 chose runtime units **[I]** |
| **`uncertainties`**, **SALib** | First-order error propagation; global sensitivity analysis | BSD; MIT **[V]** | Python only; not integrated with any calculation contract |
| **`num-dual`**, **`inari`** | Rust automatic differentiation; interval arithmetic (IEEE 1788) | MIT/Apache; MIT **[V]** | `inari` pulls `gmp` by default (opt-out); `rug` is LGPL-3.0+ and clashes with `deny.toml` **[V]** |
| **Symbolica** | Rust CAS | **Source-available, not OSI** **[V]** | Terms forbid use "as a reference implementation… development blueprint" for competing functionality (§4(4)-(5)). **Do not read its source for design input.** |
| **ezdxf**, **`dxf`**, **`acadrust`**, **IfcOpenShell** | DXF/DWG/IFC | MIT; MIT; **MPL-2.0**; **LGPL-3.0** **[V]** | `acadrust` claims DWG R12–R2018+ read/write and is a port of an MIT C# library; 0.x **[V]** |
| **FreeCAD MCP servers** | Agents driving CAD (3.5k, 642, 281 stars) | not checked **[?]** | Generate/modify models; no verification concept seen. Tool lists not read. |
| **RO-Crate**, **W3C PROV-O** | Research provenance packaging | Apache-2.0 spec; W3C Rec. **[V]** | A folder plus one JSON-LD file is practical without an RDF stack **[I]** |

---

## 4. Technical architecture comparison

| Aspect | CivKit (repo) | CivilKit Calcs | Open Infra Studio | Pondus | Stabileo | Python solver libs |
|---|---|---|---|---|---|---|
| **Core language** | Rust | Rust → WASM | Rust | Rust | Rust → WASM | Python (C++ for OpenSees) |
| **What the "unit of reuse" is** | A *calculation* with a typed descriptor | A *worksheet* | A *CAD command* | A *model* | A *model* | A *function / class* |
| **Self-describing interface** | Yes: descriptor → JSON Schema (accepted design) | Worksheet source | `mcp-tools.json` generated and tested **[V]** | 46 tool names **[V]** | No public API described **[?]** | Python docstrings |
| **Units** | Runtime SI + dimensions; ambiguity rejected | In worksheet language | **[?]** | **[?]** | Caller's responsibility (implied) **[V]** | Mostly bare floats |
| **Interfaces from one core** | Planned: Python, local MCP, C ABI, AutoCAD, desktop | Web, CLI, crates, hosted MCP | Plugin + MCP | GUI, CLI, MCP | Web | Python only |
| **MCP** | Local stdio only, read-only (planned) | **Hosted**, read-only, 11 tools | Local stdio/HTTP, 39 tools, mostly *write* | Local stdio, 46 tools, *write* | none found | none |
| **Extensibility** | Calculation registry; no plugin runtime | Embed API; sheets | Plugin API pinned to host | Closed set | Closed | Python subclassing |
| **Data formats** | JSON contract; DXF (planned) | Worksheets, docx/xlsx/pdf | Broad GIS/LandXML/IFC | `.pondus`, DXF, IFC, Excel | DXF, Excel, IFC (PRO) | Python objects |
| **Offline** | Hard rule, CI-enforced | Calc yes; plots CDN; MCP hosted | Mostly online | Yes (except agent) | Solver yes; editor account **[?]** | Yes |
| **Provenance / replay** | Version echoed; trace; no record format yet | None **[?]** | None **[?]** | autosave file | share link | None |

**Take-away [I].** CivKit is unusual in choosing the *calculation* as the unit of reuse and giving it a typed,
versioned, self-describing contract. Every competitor examined chose the model, worksheet or command. That choice is
what makes the opportunities in §7 possible, and also what makes CivKit's solver ambitions (if any) harder; it
should be kept deliberately.

---

## 5. Feature comparison matrix

Legend: **●** documented and verified · **◐** partial / limited / self-reported only · **○** looked for, not found
· **?** unverified. "Planned" in the CivKit column means in the accepted plan, **not implemented**; the repository
currently contains the contract, units, 2D geometry, and two calculations (Manning, simply supported beam); traverse,
levelling and earthwork are waiting on the list-input extension (D-23).

### 5.1 Applications

| Capability | CivilKit Calcs | Open Infra Studio | Pondus | Stabileo | CivKit |
|---|---|---|---|---|---|
| Runs fully offline | ◐ (calc yes, plots/MCP no) | ◐ (most features online) | ● (agent excluded) | ? | ● by policy |
| Permissive licence (MIT/Apache/BSD) | ● MIT | ○ GPL-3.0 | ○ AGPL-3.0 | ○ AGPL-3.0 | ● Apache-2.0 |
| Checked units at the interface | ● (in sheets) | ? | ? | ○ | ● (implemented) |
| Dimensional-ambiguity rejection (`lb`, `ton`, `gal`) | ? | ? | ? | ? | ● (implemented) |
| Step-by-step trace | ● (sheet is the trace) | ○ | ◐ | ◐ (stiffness-method mode) | ● |
| Versioned calculation IDs; same input → same output pinned by version | ○ | ○ | ○ | ○ | ● (accepted) |
| Replayable calculation record | ○ | ○ | ○ | ○ | ○ (proposal FP-01) |
| Published verification ledger | ◐ | ◐ | ◐ | ● | ◐ (reference cases; no ledger) |
| MCP server | ● hosted, read-only | ● local, writes | ● local, writes | ○ | planned, local, read-only |
| "Check my hand calculation" tool | ● (hosted) | ○ | ○ | ○ | ○ (proposal FP-04) |
| DXF read | ○ | ● (via host) | ◐ geometry | ● (PRO) | planned (Phase 4) |
| LandXML | ○ | ● | ○ | ○ | ○ (proposal FP-15) |
| IFC | ○ | ◐ export only | ◐ | ● import (PRO) | ○ |
| Reports | ● docx/xlsx/pdf/html | ◐ | ● PDF | ? | ○ (proposal FP-12) |
| Design codes | ○ (stated) | ◐ road/rail checks | ● many | ● many | ○ (D-8, later) |
| FE solver | ○ | ○ | ● | ● | ○ (not planned) |
| Sensitivity / uncertainty | ○ | ○ | ○ | ○ | ○ (proposals FP-05, FP-06) |
| CLI | ● | ○ | ● | ○ | ○ (proposal FP-10) |
| Accounts / telemetry required | none stated | none (online services) | none (agent key) | **?** | forbidden |

### 5.2 Libraries

| Capability | OpenSees/Py | PyNite | anaStruct | PyCBA | section-/concrete-properties | structuralcodes | WhiteboxTools | CivKit |
|---|---|---|---|---|---|---|---|---|
| Licence usable from Apache-2.0 | ○ restricted | ● MIT | ◐ LGPL | ○ AGPL | ● MIT | ● Apache | ● MIT (legacy) | ● |
| Language bindings beyond Python | Tcl, Python | ○ | ○ | ○ | ○ | ○ | Python, R | planned: Python, C ABI, MCP |
| Units checked | ○? | ○? | ○ | ○ | ? | ? | n/a | ● |
| Reports | ○ | ● | ○ | ◐ | ◐ | ○ | ○ | ○ |
| Plots / diagrams | ○ | ● | ● | ● | ● | ○ | ◐ | ○ |
| Published validation | ? | ◐ | ○ | ◐ | ● (sectionproperties) | ? | ? | ◐ |
| Design-code content | ○ | ◐ | ○ | ◐ (load models) | ● AS 3600 | ● EC2, MC2010 | n/a | ○ |
| DXF import | ○ | ○ | ○ | ○ | ● (sectionproperties) | ○ | ○ | planned |

---

## 6. Licensing analysis

### 6.1 What each licence means for CivKit **[V for the licence text; interpretation is mine, not legal advice]**

| Licence | Projects | Can CivKit link or copy code? | Can CivKit independently re-implement the *idea*? |
|---|---|---|---|
| **MIT / BSD / Apache-2.0** | CivilKit Calcs, CalcpadCE, PyNite, sectionproperties, concreteproperties, structuralcodes, Whitebox legacy, geo, proj4rs, nalgebra, faer, russell, argmin, num-dual, inari, `dxf`, ezdxf, Pint, SymPy, SALib, OpenMDAO, handcalcs, `rmcp` | Yes, with notices kept; check each *dependency's* licence | Yes |
| **MPL-2.0** | `acadrust` | Linkable from Apache-2.0; changes to *its files* stay MPL; **not in `deny.toml`'s allow list**, so needs a decision record | Yes |
| **LGPL-3.0** | anaStruct, IfcOpenShell, `rug` | Only as a separate, dynamically replaceable component; **not appropriate as a Rust dependency**; `rug` is blocked by the current allow list | Yes (as test oracle: run it, don't link it) |
| **GPL-3.0** | Open Infra Studio | **No** (would make CivKit GPL) | Yes, from public specifications; do not read-and-port |
| **AGPL-3.0** | Pondus, Stabileo, PyCBA | **No**; includes network-use clause | Yes, from published methods; do not read-and-port |
| **UC Regents terms** | OpenSees | **No** commercial incorporation without permission | Legal question; policy answer: derive from open textbooks only |
| **Source-available (Symbolica)** | Symbolica | **No**; and §4(4)-(5) forbids using it as a reference for competing functionality | Avoid reading its source at all |
| **Open-core / subscription** | Whitebox Pro | **No** | n/a |
| **Standards** | OGC GeoPackage (document licence permits implementation, no patent grant); RFC 7946 GeoJSON (WGS 84 only); IFC (docs CC BY-ND 4.0; implementation reported royalty-free by a secondary source); LandXML **[?]** | Implementable | n/a |

### 6.2 Rules this research supports

1. **Never port source from a GPL/AGPL/LGPL/restricted project**, and never from Symbolica. Reading *documentation* and
   published papers is fine. D-18 already says reference cases come only from open sources; this extends the same
   discipline to *code*.
2. **Use copyleft projects as black-box test oracles only** (run them, compare numbers, record the comparison). A
   differential-testing harness is the safe way to benefit from PyNite (MIT), IfcOpenShell (LGPL, as a process),
   shapely, Pint, etc. (FP-22).
3. **Watch for licence drift.** `PyCBA` went Apache-2.0 → AGPL-3.0 in one release. Pin versions of anything used as an
   oracle and record the licence at the time of use. CivKit's own protection is the DCO-only, no-CLA policy (D-11):
   relicensing CivKit later would need every contributor's consent.
4. **`deny.toml`'s allow list is a sound filter.** It already excludes `rug` (LGPL), `acadrust` (MPL-2.0), and any
   AGPL/GPL crate. Candidates that pass it: `geo`, `proj4rs`, `nalgebra`, `faer`, `russell*`, `argmin`, `roots`,
   `num-dual`, `inari` without default `gmp`, `statrs`, `dxf`. Candidates that need a deliberate decision first:
   `acadrust` (MPL-2.0) and anything LGPL.
5. **Copyrighted design-code text.** `structuralcodes` and `concreteproperties` show that a free Apache/MIT library can
   implement equations from Eurocodes and AS codes. How they treat table data and clause text was not verified
   **[?]**; CivKit's existing rule (equations and clause numbers only; no pasted tables) stays the right default.
6. **Standards documents.** GeoPackage, GeoJSON and (reportedly) IFC allow implementation. The **LandXML schema's
   licence could not be verified**: searches returned no licence text and landxml.org could not be fetched. Check
   before implementing FP-15.

---

## 7. Identified market gaps

Each gap lists the evidence and **how sure I am that it really is open**. All "not found" statements are limited by
the breadth of this search (§11): about 25 projects were examined in depth, and many commercial tools (Mathcad
Prime, Civil 3D, Tekla, STAAD, etc.) were *not* examined.

| # | Gap | Evidence | Confidence it is open |
|---|---|---|---|
| **G1** | **A replayable, versioned calculation record that travels between tools** | Worksheets show steps; `handcalcs` renders but does not verify (and warns about out-of-order cells); CivilKit's MCP re-computes *sheets* on a hosted service; OpenMDAO records inside a Python graph; Stabileo/Pondus verify per project. No project found defines a small portable record that names calculation + version + inputs + engine and can be re-run offline. | **Medium** **[I]**. Plausible in commercial QA systems not examined. |
| **G2** | **Typed, interface-independent calculation contract** (JSON Schema from descriptors; same results through CLI, Python, MCP, C ABI) | Competitors expose either a GUI/worksheet or a tool list. `mcp-tools.json` (Open Infra) is the closest and is generated and tested. | **Medium-high** **[I]** |
| **G3** | **Unit safety at the interface, including ambiguity rejection** | PyCBA and anaStruct unitless (verified); Stabileo's own file records a 1000× modulus mistake; Pint-style runtime units exist in Python but only inside Python. | **High** for the *structural-library* niche; **low** for worksheets **[I]** |
| **G4** | **A local, read-only MCP whose answers carry their own evidence** | The only read-only MCP is hosted; the local ones write. None exposes verification status. | **Medium-high** **[I]** |
| **G5** | **Verification as data, per calculation** (cases, sources, tolerance, verifier, errata, known gaps) | Stabileo has a project-level ledger as a document; sectionproperties has a validation chapter. None machine-readable per calculation. | **Medium** **[I]** |
| **G6** | **Offline, no-account, permissively licensed infrastructure calculation with CAD-neutral drawing quantities** | Open Infra is GPL and online-heavy; Pondus/Stabileo AGPL; ezdxf is a library, not an analysis tool. | **Medium** **[H]**. Needs checking against QGIS/LibreCAD plugins, which were not examined. |
| **G7** | **Sensitivity and uncertainty attached to ordinary engineering calculations** ("which input matters, and what if it is ±5 %?") | `uncertainties` and SALib exist; `num-dual` and `inari` exist in Rust; no project integrates them with a calculation contract. | **Medium-high** **[I]** |
| **G8** | **LandXML as the open bridge from Civil 3D into a free engine** | Open Infra reads LandXML but only inside its host; no reusable library found (LandXML research incomplete). | **Medium** **[H]** |
| **G9** | **A published conformance suite so third-party wrappers can prove they match the engine** | None found. | **Medium** **[H]** |

### Things competitors do that CivKit lacks, which are *not* opportunities for CivKit

Full worksheets and Mathcad import (CivilKit), complete structural design workflows and PDF design reports (Pondus),
decades of nonlinear solver research (OpenSees), 700 terrain tools (Whitebox), national bridge-load models (PyCBA).
Each is a project of its own. Competing on these is the "project suicide" path.

---

## 8. CivKit's potential positioning

### 8.1 The answer to the final question

> **What could CivKit become that existing engineering software does not already provide effectively?**
>
> **A verifiable, interface-independent calculation layer**: the place where an engineering number is produced *with
> its evidence attached*, reachable from Python, a command line, an AI assistant, AutoCAD or a desktop window, and
> re-checkable offline by someone who was not there.

It is not "a free alternative to X". It is infrastructure that X could use. In one sentence for the README
**[proposal, not applied]**: *"CivKit makes every engineering number carry its units, its method, its version and its
evidence, and reproduces it anywhere, offline."*

### 8.2 The three pillars **[I]**

1. **Trust you can inspect.** Calculation record (FP-01), verification ledger (FP-02), built-in cross-checks (FP-03),
   hand-calculation checker (FP-04). CivKit's existing rules (reference cases mandatory, independently verified,
   errata recorded, no silent changes) are already stricter than anything found in the worksheet or library niches.
2. **Reach.** One contract, many hosts: CLI (FP-10), Python (planned), local MCP with schemas, resources and prompts
   (FP-19), typed clients (FP-20), conformance suite (FP-21). This is the "works where engineers work" promise made
   *testable*.
3. **Independence.** Offline, no accounts, permissive licence, reproducible builds (FP-26). In a market that is
   hardening into AGPL, source-available and hosted services, "you can embed this and no one can switch it off" is a
   real property. Engineering firms, universities and tool authors (including the competitors) can adopt it.

### 8.3 Who CivKit is *not* for **[I]**

- Someone who wants to draw a frame and click "design" → Pondus, Stabileo.
- Someone who wants a Mathcad replacement → CivilKit Calcs, CalcpadCE.
- Someone who needs a corridor model of a road → Open Infra Studio, Civil 3D.

CivKit can be the thing *behind* parts of those, or next to them.

### 8.4 Candidate users, in order of fit **[H]**

1. Engineers and students who script in Python or Jupyter and are tired of unit mistakes. (Phase 3 delivers this.)
2. People who use an AI assistant for engineering and want numbers that are not hallucinated. (MCP, Phase 3.)
3. Firms and courses that need *auditable* hand-calculation checks with a record. (FP-01, FP-04, FP-25.)
4. Tool authors who want a permissive, embeddable calculation engine. (C ABI, JSON contract.)

---

## 9. Architectural implications

The brief asks for a challenge, not praise. Each finding says what the evidence shows, what it would cost, and what
migration means. **Everything labelled "Proposal" needs its own decision record; none is applied.**

### 9.1 Is the calculation contract general enough?

**Finding.** The contract (descriptor, request, response, trace, warnings, errors) matches the *calculation* as the unit
of reuse, which is the right call (§4). Gaps found against the opportunities, all additive:

| Gap | Needed for | Proposal | Benefit | Cost / migration |
|---|---|---|---|---|
| **AP-1. No echo of the request, and no engine fingerprint, in the response** (`Response` carries `civkit_version` but not the normalised inputs or a build identifier) | Calculation record (FP-01) | Define a *record* document = request + response + `engine` block (version, build id, calculation descriptor hash). Prefer building the record *outside* `Response` so the accepted JSON shape stays unchanged. | A stored result becomes re-verifiable. | Low. New document type; no change to existing calls. **Decide the schema before Phase 3 parity tests** so the three interfaces emit it identically. |
| **AP-2. `compute` is `f64 → f64`** (`Calculation::compute(&Inputs, &mut Context) -> Outputs` with `f64` accessors) | Sensitivity and uncertainty (FP-05, FP-06) | *Defer*, but record the constraint: derivatives need `compute` generic over a scalar type, or a second implementation path. Prototype on one calculation (Manning) before touching the trait. | Unlocks G7. | **Medium-high.** Every existing and future calculation would be affected; this is the most expensive change in this report. It is cheaper to decide *before* many calculations exist than after. Alternative with no trait change: numerical differentiation by calling the same calculation (central differences) at the registry level, accepting lower accuracy. **Start there.** |
| **AP-3. No place for a calculation's own sanity checks** | Cross-checks (FP-03) | Optional `checks` list in the response (name, value, tolerance, passed). Pure addition per API_DESIGN §4. | Reactions sum to the load; traverse closure; volumes from two methods agree. A failed check is a warning with its own code. | Low. |
| **AP-4. Descriptor has `references` as free text** | Code-pack framework (FP-07), coverage matrix | Optional structured `clauses` (code, edition, clause) alongside text references. | Machine-checkable "which clauses are implemented". | Low. Pure addition. |
| **AP-5. Calculations cannot be composed** | Workbook (FP-08) | Do **not** let calculations call each other. Put composition in a separate, small *workbook* layer that only wires outputs to inputs (with units) and calls the registry. | Keeps `compute` simple and every step individually recorded. | Medium. New module; no core change. |

Already planned and sufficient for now: lists and choices (D-23), validity warnings, versioning rules. **Observation
[I]:** D-23's groups, index sets, offsets and item labels are a lot of machinery for three calculations. The decision
is made and accepted; the suggestion is only that *no further contract generality should be added until a second
consumer needs it* (see §9.7).

### 9.2 Crate boundaries

**Finding [I].** The boundaries are sound: `civkit-units` and `civkit-geometry` are separate, domain calculations are
modules inside `civkit-core` and are split "when they grow". Two pressure points:

1. **Reference cases live in `tests/reference/`, outside the crates.** A `civkit verify` command or a ledger
   (FP-01, FP-02) needs the cases at *run time*. Options: embed the case files into the binary at build time
   (`include_str!`, no new dependency), or ship them as a data folder. **Proposal:** embed. Cost: low; migration: the
   harness keeps reading the same files.
2. **A future `civkit-record` / `civkit-workbook` crate** would be the natural home for FP-01/FP-08 so that
   `civkit-core` stays the contract. Create it only when FP-01 is accepted, not before.

### 9.3 Neutral drawing model

**Finding [I].** ADR 0002 anticipated that Civil 3D alignments and surfaces "will need explicit extensions". The
evidence adds a cheaper route: **LandXML** carries alignments, surfaces and points in an open format that Civil 3D
exports (FP-15). Two additions to the model are worth *reserving room for*, not building: (a) a **source handle** on each
entity (DXF handle / AutoCAD ObjectId) so a schedule line can point back to its drawing entity, and (b) a
**coordinate-reference note** (the drawing's CRS or "local") so FP-16 has somewhere to attach. Neither is needed for 0.1.
3D is correctly out of scope.

### 9.4 The C interface

**Finding.** `civkit_call(json) → json`, `civkit_list`, `civkit_describe`, `civkit_free` and an ABI version number is
adequate and is exactly the shape the competitors' plugins lack (Open Infra's plugin API is pinned to host builds
and breaks when the host changes **[V]**). JSON-over-C also means the record and verify features (FP-01) cost nothing at
the ABI. Two small additions: `civkit_schema()` returning the descriptor JSON-Schema bundle (feeds FP-20), and, only
if profiling demands it, a batch call. **No ABI redesign recommended.**

### 9.5 MCP: tools only, or workflows?

**Finding.** API_DESIGN §10 specifies tools (one per calculation plus generic `list/describe/call`). The current MCP
specification (revision **2026-07-28**) also provides **resources**, **prompts** and **`outputSchema` with
`structuredContent`**, and *deprecates* sampling and roots **[V]**. Pondus implements 2025-06-18; `rmcp` 3.x targets
2026-07-28 **[V]**. **Proposal (FP-19):**

- generate `outputSchema` from descriptors, so a client can validate what it receives;
- expose descriptors, reference cases and the verification ledger as **resources** (the AI can *read* the evidence);
- offer a few **prompts** ("explain this result from its trace", "check my hand calculation") that call read-only
  tools;
- keep every tool read-only; add a tool such as `civkit_check_hand_calc` (FP-04).

D-13 ("pin `rmcp` at Phase 3 start") should be re-read against the 2026-07-28 revision when Phase 3 begins; the
protocol changed from a stateful to a stateless model, which suits a local stdio server well.

### 9.6 Python: more than function calls?

**Finding [I].** API_DESIGN §8 gives `call`, typed functions, `Q` quantities, and defers array inputs. The
competitors' Python users do *sweeps and tables*. **Proposal:** after 0.1, add `civkit.sweep(...)` and
`civkit.batch(...)` returning a table (list of dicts, optionally a DataFrame if pandas is installed; pandas must stay
optional), plus Jupyter `_repr_html_` on results (FP-09, FP-24). This is a layer over `call`; no engine change. D-23
already clarified that "array inputs" means batching.

### 9.7 Plugin system for the desktop app? A common project format?

- **Plugin runtime: not now (FP-30, rejected/deferred).** Every plugin system found (Open Infra's native libraries
  pinned to host API version 28) becomes a permanent maintenance and security surface. CivKit's extension model is
  *a calculation registered with a descriptor and reference cases*; that is already a plugin system at source level.
- **Common project/document format: only the minimum.** A general "CivKit project file" is premature. The *record*
  (one calculation) and the *workbook* (a short ordered list of requests with named links) are small, testable,
  and enough. D-14 (SQLite in Phase 6) is compatible: the journal (FP-25) can start as a JSONL file.

### 9.8 Dependency strategy

**Finding.** Strict allow-list + bans + dependency-rule script is stronger than anything seen in the competitors and
scales. Risks: (1) the rule "nothing in `crates/` depends outside `crates/`" is sound but means pure-Rust geometry
and CRS crates (`geo`, `proj4rs`) must be *wrapped* in optional crates rather than depended on by core; (2) `inari`'s
default `gmp` feature and `rug` conflict with the licence list: disable or avoid; (3) the C library `proj` adds a
native dependency: prefer `proj4rs` (pure Rust, 2D only) for offline builds **[V]**. **Proposal:** write one short
decision record when the first optional module crate is added: "optional crates may depend on named third-party crates
that pass `deny.toml`; core, units and geometry stay minimal".

### 9.9 Anything premature?

**[I]** Items that exist ahead of their first user: `civkit-ffi` (empty crate until Phase 5), `civkit-drawing`
(empty until Phase 4), the general machinery in D-23, the MATLAB wrapper and SQLite plans. None is harmful today (empty
crates cost nothing), but **no new abstraction should be added without a second concrete user**. This report's
proposals follow that rule: most are *additions* whose first user exists today (Python and MCP).

### 9.10 What *not* to change

Rust core; runtime units; JSON contract; descriptor-driven interfaces; reference cases mandatory; the "AI calls, it
does not compute" rule. Each of these is confirmed, not weakened, by the competitor evidence.

---

## 10. Name and branding risks

> **No legal conclusions.** These are risks to investigate. D-1 (name CivKit) is accepted; nothing here changes it.
> If the checks below turn up a problem, a *new* decision record would be needed.

### 10.1 Findings

| Check | Result (2026-11-25) |
|---|---|
| **crates.io** `civkit`, `civilkit`, `civkit-core`, `civkit-units` | Not found (HTTP 404, checked twice, by two methods) **[V]**. `civilkit-calcs-engine` exists (MIT, published 2026-10-22). |
| **PyPI** `civkit`, `civilkit`, `civil-kit`, `civkit-core` | JSON API returned 404 for each **[V]**. The human-readable project page for `civkit` could not be read (bot challenge), so absence is *very probable*, not certain **[?]**. |
| **npm** `civkit` | **Exists** (AGPL, "sets of tools, in typescript", maintainer `nomagick`, last modified 2026-11-03) **[V]**. Unrelated, but active. `civilkit` does not exist. |
| **GitHub org `civkit`** | Exists; a Bitcoin/Lightning peer-to-peer trading project; last visible activity Nov 2024 **[V]** (matches what D-1 recorded). |
| **GitHub org `civilkit`** | Exists, 6 repos, **active in Nov 2026**: `civilkit-calcs`, `cadkit-kit`, `cadkit-kernel`, `civilkit-parts`, forks **[V]**. |
| **civilkit.io** | Home page lists CivilKit Calcs ("Available now") and CivilKit CAD ("Preview"), MCP, guides. No operator, trademark notice or copyright holder named on the pages read **[V]**. |
| **GitHub search "civkit"** | 8 repositories, including three from the Bitcoin org, an npm toolkit, an old 2016 repo, and this project **[V]**. |
| **Search-engine results** | **Not obtained** (the search engine returned a bot challenge) **[?]**. |
| **Trademark registers** (TMview, USPTO, WIPO Global Brand Database) | **Could not be searched** (dynamic sites; no results retrievable) **[?]**. Nothing is inferred. |

### 10.2 Risks to investigate

| # | Risk | Why |
|---|---|---|
| **N1** | **Name confusion with CivilKit** | Two letters apart; same domain (civil engineering), same language (Rust), same pitch (free, offline, units, MCP, "kit"). A user, an AI assistant or a search engine can easily conflate them, *and CivilKit is now ahead in visibility and activity.* This risk has **grown since D-1**, which recorded only the Bitcoin and npm overlaps. |
| **N2** | **Search discoverability** | A query for "civkit" can be corrected to "civilkit" by search engines; the Bitcoin `civkit` org and the npm package also compete for the term **[H]**. |
| **N3** | **AI-assistant confusion** | Assistants (including coding agents reading MCP lists) may recommend CivilKit's hosted MCP when asked about CivKit, or vice versa **[H]**. This matters more because CivKit's own message is about *trusting the source of a number*. |
| **N4** | **Package-namespace squatting** | Names `civkit`, `civkit-core` etc. are currently free on crates.io/PyPI. Nothing is published (`publish = false`). Someone else could take them before 0.1. Reserving a name requires *publishing something*, which is a decision the maintainer should take deliberately. |
| **N5** | **Trademark** | Not checked. Product names in this space ("CivilKit", "CivKit", Civil 3D, Civil Tools) could be registered in some jurisdictions; searches need a human with register access. |
| **N6** | **Brand collision of the shared "kit" word** | "Kit" names are common (`cadkit-kit` in the CivilKit org). Differentiating by tagline matters. |
| **N7** | **Ownership of the CivilKit repo is ambiguous** | The repo appears under both `civilkit` and `mageaustralia` **[S]**. Not CivKit's problem, but relevant if contact is ever wanted. |

### 10.3 Suggested actions (all optional, none taken)

1. Run the trademark searches by hand (TMview, USPTO, IP Australia, India's IP India, WIPO Brand DB) for "civkit" and
   "civilkit" in classes for software. Record the result in a short note.
2. Decide *deliberately* whether to reserve `civkit` on crates.io and PyPI with a minimal placeholder release. That
   would break "nothing published before 0.1" (ADR/Cargo `publish = false`), so it is a separate decision.
3. Make the README's first line distinguish CivKit unambiguously, e.g. "CivKit (not CivilKit)". Use a distinctive
   tagline, since the current one ("Engineering tools that work where engineers work") is generic.
4. Monitor `civilkit`'s direction: if it adds a local, schema-typed MCP or a verification ledger it would collide with
   CivKit's lane, and a *conversation* with its maintainer may be better than competition. This is a judgement call
   for the maintainer.
5. If the checks reveal a conflict, bring a "reopen D-1" decision to the maintainer. Do not rename on this report alone.

---

## 11. Sources and research limitations

### 11.1 Method

- Repository documents read first (README, ROADMAP, ORIGINAL_PROPOSAL, MVP_SCOPE, IMPLEMENTATION_PLAN,
  ACCEPTANCE_CRITERIA, DECISIONS, ADRs 0002 and 0011, ARCHITECTURE, API_DESIGN, ENGINEERING_STANDARDS, OFFLINE_POLICY,
  SECURITY, AI_HANDOVER, `deny.toml`, `Cargo.toml`, `crates/civkit-core/src/contract.rs` and `registry.rs`). The
  repository was treated as the source of truth; the working tree has uncommitted D-23 changes, which were read as is.
- Web research was done by fetching public pages (project sites, GitHub pages, `raw.githubusercontent.com`, registry
  APIs, docs.rs, lib.rs, standards pages). Eight helper agents (three for the named competitors, five for adjacent projects, MCP and naming) performed most of the page fetching, each with a written
  list of questions and an instruction to separate facts from inference; their reports were read critically.
- Independently re-checked by me after the helpers reported: crates.io, PyPI JSON and npm registry results for
  `civkit` and `civilkit`; the repository's own `Response` and `Calculation::compute` definitions.

### 11.2 Primary sources (URLs read)

CivilKit: [github.com/mageaustralia/civilkit-calcs](https://github.com/mageaustralia/civilkit-calcs) ·
[README](https://raw.githubusercontent.com/mageaustralia/civilkit-calcs/main/README.md) ·
[LICENSE](https://raw.githubusercontent.com/mageaustralia/civilkit-calcs/main/LICENSE) ·
[caskit.md](https://raw.githubusercontent.com/mageaustralia/civilkit-calcs/main/docs/caskit.md) ·
[civilkit.io](https://civilkit.io/) · [civilkit.io/calcs/learn](https://civilkit.io/calcs/learn/) ·
[civilkit.io/mcp](https://civilkit.io/mcp) · [github.com/civilkit](https://github.com/civilkit) ·
[crates.io API: civilkit-calcs-engine](https://crates.io/api/v1/crates/civilkit-calcs-engine)

Open Infra Studio: [open-aec.com/open-infra-studio](https://open-aec.com/open-infra-studio/) ·
[roadmap](https://open-aec.com/roadmap/) ·
[repo](https://github.com/OpenAEC-Foundation/Open-Infra-Studio) ·
[LICENSE](https://github.com/OpenAEC-Foundation/Open-Infra-Studio/blob/main/LICENSE) ·
[releases](https://github.com/OpenAEC-Foundation/Open-Infra-Studio/releases) ·
mcp/README.md in the repo

Pondus: [strpondus.github.io](https://strpondus.github.io/index.html) ·
[manual](https://strpondus.github.io/manual.html) · [repo](https://github.com/Sompote/pondus) ·
[LICENSE](https://github.com/Sompote/pondus/blob/main/LICENSE) ·
[docs/mcp.md](https://github.com/Sompote/pondus/blob/main/docs/mcp.md)

Structural projects: [OpenSees](https://github.com/OpenSees/OpenSees) ·
[COPYRIGHT](https://raw.githubusercontent.com/OpenSees/OpenSees/master/COPYRIGHT) ·
[OpenSeesPy docs](https://openseespydoc.readthedocs.io/en/latest/) ·
[PyNite](https://github.com/JWock82/Pynite) · [anaStruct](https://github.com/ritchie46/anaStruct) ·
[PyCBA](https://github.com/ccaprani/pycba) ·
[sectionproperties](https://github.com/robbievanleeuwen/section-properties) ·
[concreteproperties](https://github.com/robbievanleeuwen/concrete-properties) ·
[structuralcodes](https://github.com/fib-international/structuralcodes) ·
[Stabileo](https://github.com/lambdaclass/stabileo) ·
[Stabileo BENCHMARKS.md](https://github.com/lambdaclass/stabileo/blob/main/docs/BENCHMARKS.md)

Geospatial and interoperability: [whitebox-tools](https://github.com/jblindsay/whitebox-tools) ·
[whitebox_next_gen](https://github.com/jblindsay/whitebox_next_gen) ·
[whiteboxgeo.com/pricing](https://www.whiteboxgeo.com/pricing.html) · [georust/geo](https://github.com/georust/geo) ·
[proj](https://github.com/georust/proj) · [proj4rs](https://github.com/3liz/proj4rs) ·
[dxf-rs](https://github.com/ixmilia/dxf-rs) · [acadrust](https://github.com/hakanaktt/acadrust) ·
[ezdxf](https://github.com/mozman/ezdxf) · [IfcOpenShell](https://github.com/IfcOpenShell/IfcOpenShell) ·
[GDAL LICENSE](https://raw.githubusercontent.com/OSGeo/gdal/master/LICENSE.TXT) ·
[RFC 7946](https://www.rfc-editor.org/rfc/rfc7946) · [OGC GeoPackage](https://www.geopackage.org/spec140/index.html)

Computation libraries: [Pint](https://github.com/hgrecco/pint) · [uom](https://github.com/iliekturtles/uom) ·
[dimensioned](https://github.com/paholg/dimensioned) · [handcalcs](https://pypi.org/project/handcalcs/) ·
[uncertainties](https://pypi.org/project/uncertainties/) · [SALib](https://github.com/SALib/SALib) ·
[num-dual](https://github.com/itt-ustutt/num-dual) · [inari](https://docs.rs/inari) · [rug](https://docs.rs/rug) ·
[Symbolica License.md](https://raw.githubusercontent.com/symbolica-dev/symbolica/main/License.md) ·
[CalcpadCE](https://github.com/imartincei/CalcpadCE) · [nalgebra](https://docs.rs/nalgebra) ·
[faer](https://docs.rs/faer) · [russell](https://github.com/cpmech/russell) · [argmin](https://github.com/argmin-rs/argmin) ·
[OpenMDAO](https://github.com/OpenMDAO/OpenMDAO) · [PROV-O](https://www.w3.org/TR/prov-o/) ·
[RO-Crate](https://www.researchobject.org/ro-crate/) · [proptest](https://github.com/proptest-rs/proptest)

MCP: [specification 2026-07-28](https://modelcontextprotocol.io/specification/2026-07-28) ·
[rust-sdk (`rmcp`)](https://github.com/modelcontextprotocol/rust-sdk) ·
topic pages for [mcp-server](https://github.com/topics/mcp-server) and [freecad](https://github.com/topics/freecad)

Secondary (used only to flag questions, not as evidence):
[LandXML search results](https://en.wikipedia.org/wiki/LandXML) ·
[buildingSMART IFC 4.3 docs](https://ifc43-docs.standards.buildingsmart.org)

### 11.3 Limitations, stated plainly

1. **Breadth.** About 25 projects were examined; none of the commercial leaders (Mathcad Prime, Civil 3D, Tekla,
   STAAD, SAP2000, ETABS, RISA, Robot, QGIS, Bentley products) were studied. "Not found" means *not found in this
   sample*. Several gaps (G1, G6, G8) could already be filled by commercial tools.
2. **Partial page reads.** Several long GitHub pages were truncated; the remainder was summarised by a model (marked
   **[S]**). That includes the CivilKit repository's commit and folder details and the Open Infra README. Counts such as
   "613 commits" come from those pages.
3. **Self-reported verification.** Test counts and benchmark percentages for CivilKit, Pondus and Stabileo are the
   projects' own statements; no tests were run. Stabileo's Lean-proof claim and `CERTIFY.md` were not read.
4. **Not read:** licence files for sectionproperties/concreteproperties (branch name error); per-crate licences of the
   Whitebox backend crates; Whitebox Pro EULA; the structuralcodes API pages; the OpenSees element list; LandXML
   (landxml.org unreachable, schema licence unknown); ODA's DWG specification terms; search-engine results;
   trademark registers; PyPI's `civkit` human page.
5. **Unit handling** was verified for PyCBA and anaStruct (unitless) and described for Pint and `uom`; for PyNite,
   sectionproperties, structuralcodes and OpenSees it is **[?]**.
6. **Dates and counts are one-day snapshots**; several projects released within the last week.
7. **AI involvement.** This report was written with an AI model and fetched through AI helpers. I treated their output
   as leads and re-verified registry and repository facts directly where a conclusion depended on them. All
   engineering and legal interpretation here still needs a human reviewer, especially §6 (licences) and §10 (naming).
8. **No code was run, no dependency installed, no source file modified, and no accepted decision changed.**
