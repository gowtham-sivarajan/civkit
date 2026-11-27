# Proposal: the DXF reader (Phase 4, task 1)

> **Provenance:** Claude Sonnet 5.5 (`claude-sonnet-5-5`) · first drafted 2026-11-27 14:10 IST · last edited 2026-11-27 14:28 IST by Claude Sonnet 5.5 (`claude-sonnet-5-5`)

> **Status:** **Proposed; revision 2 after the maintainer's review (2026-11-27).** Decision ID **D-27**. Nothing is
> implemented. The maintainer **held Q-A, Q-B and the ADR**: the first revision's case for an own reader missed
> `undxf` 0.11.0. This revision adds `undxf` to the evaluation, runs it hands-on against the fixtures, malformed
> inputs and memory measurements, and revises the limits (Q-D), because file and entity caps alone do not bound peak
> memory. Q-C (scope) and Q-E (fuzzing) remain under consideration.
>
> Built on: D-12 / [ADR 0002](decisions/0002-neutral-drawing-model.md) (neutral drawing model and DXF reader in the
> Rust engine), [SECURITY.md](../docs/SECURITY.md) (hardened parser, limits, fuzzing), [MVP_SCOPE.md](MVP_SCOPE.md) §3
> (read-only analysis: entities by layer, lengths and areas, drawing-unit consistency, schedules), the fixtures in
> [tests/fixtures/dxf](../tests/fixtures/dxf/README.md), and D-22 ([GEOMETRY.md](../docs/GEOMETRY.md)).

## 1. What has to be decided

ADR 0002 left one question open: *"DXF parsing adds a dependency (an existing Rust DXF crate, to be evaluated in
Phase 4) or our own reader."* This proposal evaluates the candidates, recommends one, and fixes what the reader
covers in 0.1 and how it is kept safe, including a peak-memory bound. The analysis contract (how drawing results,
geometry notes and unsupported features reach users) is a separate proposal (Phase 4 task 3), after this one.

## 2. What the reader must do

| # | Requirement | Source |
|---|---|---|
| R1 | Read ASCII DXF from R12 to R2018, as the fixtures have (AC1009, AC1018, AC1024, AC1027, AC1032) | fixtures |
| R2 | Entities for 0.1: LINE, LWPOLYLINE (bulges, closed flag), old POLYLINE/VERTEX/SEQEND (bulges), ARC, CIRCLE, INSERT **and MINSERT arrays** (rotation, non-uniform and negative scale, nesting), HATCH (for its area), TEXT, linear DIMENSION (its measurement); every other entity counted and reported, never silently dropped | MVP_SCOPE, fixtures F1–F5 (F5 counts **13** bays, 3 of them from a MINSERT) |
| R3 | Header: `$ACADVER`, `$INSUNITS`, `$MEASUREMENT`, with **"not stated" distinguishable from any stated value** | F3 (units mismatch), F4 (R12, no units): "the engine must report *units not stated*, not assume metres"; D-19 |
| R4 | Never crash or hang on hostile input; limits; fuzzed | SECURITY.md |
| R5 | **A stated peak-memory bound** that holds for any input, hostile ones included, enforced by the reader (new in revision 2) | SECURITY.md ("memory exhaustion"); the maintainer's review of Q-D |
| R6 | Numbers read exactly (correctly rounded), as the rest of the engine | Phase 3 finding (serde_json `float_roundtrip`) |
| R7 | Read-only; no writing in 0.1 | S-5, ADR 0002 |
| R8 | Dependencies pass `deny.toml` and the dependency rule, stay small, and do not make every upgrade a code change: they ship inside the engine | ARCHITECTURE §2, SECURITY |

## 3. Candidates

Facts from crates.io and `cargo info` (2026-11-27).

| Crate | Latest | First release | Releases | Downloads | Licence | Normal dependencies |
|---|---|---|---|---|---|---|
| **`dxf`** (IxMilia) | 0.6.2, 2026-10-03 | 2016-09 | 10 | ~394 000 | MIT | byteorder, chrono, encoding_rs, enum_primitive, image, itertools, num, uuid (30 crates in all) |
| **`undxf`** (iyulab) | 0.11.0, 2026-11-05 | 2026-09-29 | 13 in 5 weeks, all 0.x minor bumps | ~790 | MIT, Rust 1.88 | encoding_rs, serde, **uncad-model** 0.12 (serde, serde_json; 13 releases in lockstep) |
| `cadkit-dxf` | 0.5.4, 2026-11-16 | 2026-10-01 | 17 | ~3 100 | MIT OR Apache-2.0 | cadkit-core, encoding_rs (its own drawing model) |
| `dxf-tools-rs` | 0.1.4, 2026-01-20 | 2026-01-20 | 3 | ~700 | MIT | ahash, anyhow, flate2, nalgebra, nom, rayon, … |
| `cadrender-dxf` | 0.2.0, 2026-11-16 | 2026-11-14 | 2 | ~70 | MIT OR Apache-2.0 | cadrender-core, cadrender-acis, memchr, memmap2 |
| **Own reader** in `civkit-drawing` | — | — | — | — | Apache-2.0 | none |

`cadkit-dxf`, `dxf-tools-rs` and `cadrender-dxf` are too new or too heavy to rest the engine on and were not tested.
`dxf` and `undxf` were tested hands-on, in a scratch crate outside the repository, with the same harness.

### 3.1 The fixtures

| Check | `undxf` 0.11.0 | `dxf` 0.6.2 |
|---|---|---|
| All seven fixtures load | ✅ | ✅ |
| F1 HATCH (expected area 96) | ✅ polyline boundary path, 4 vertices, shoelace area **96** | ❌ no HATCH entity type: dropped |
| F4 (R12) units not stated (R3) | ✅ `$INSUNITS` `None`; `$MEASUREMENT` absent from the header | ❌ reported as `Unitless` and **`English`** (defaults) |
| F3 header conflict | ✅ `$INSUNITS` 1, `$MEASUREMENT` 1 | ✅ |
| F5 transformed inserts | ✅ rotation 90°, scale (2, 1), mirror (−1, 1); blocks resolved; nested blocks present | ✅ |
| F5 MINSERT, 1 row × 3 columns | ⚠️ **first placement only**, with a diagnostic ("only its first placement is read"); its model has no row, column or spacing fields: **11 of 13 bays** | ✅ read |
| Unrecognised entities | ✅ kept as `Unknown`, by name | ❌ dropped |
| Binary DXF, code pages | ✅ both (binary converted to ASCII text first) | ✅ |
| `unsafe` | `forbid(unsafe_code)` in `undxf` and `uncad-model` | dependencies use `unsafe` (`image`, …) |

### 3.2 Malformed input (R4)

The same 3 150 corrupted variants of the fixtures (truncation, byte flips, numbers such as `1e308`, `NaN`,
`-2147483649`, deleted lines, repeated slices), a smoke test, not fuzzing:

| | `undxf` | `dxf` |
|---|---|---|
| Panics | **0** | **0** |
| Parses slower than 2 s | 0 | 0 |
| Clean errors | 2 333 | 2 352 |

Both are robust at this level.

### 3.3 Peak memory (R5)

Peak heap measured with a counting allocator, input already in memory:

| Input | Size | `undxf` peak | `dxf` peak |
|---|---|---|---|
| The fixtures | 6–89 KiB | 5–13 × input | — |
| 200 000 LINEs | 9.5 MiB | 371 MiB (**39 ×**) | 521 MiB (**55 ×**) |
| One LWPOLYLINE, 500 000 vertices | 9.4 MiB | 81 MiB (8.5 ×) | 45 MiB (4.8 ×) |
| 300 000 unrecognised entities | 4.0 MiB | 631 MiB (**158 ×**) | 4 MiB (1 ×: dropped) |

Neither crate takes a memory or count limit, and both build the whole drawing before returning. With the first
revision's 256 MiB file cap, the bound would be about 10 GiB for plain lines and about 40 GiB for a hostile file of
unrecognised entities through `undxf`. **File-size and entity-count caps do not bound memory**, as the maintainer
noted; a bound needs either a much smaller file cap with a pre-scan before the crate runs, or a reader that accounts
for what it stores.

### 3.4 Maintenance and fit (R8)

- **`undxf`:** a careful design ("nothing silently dropped", deterministic, `forbid(unsafe_code)`, small tree whose
  licences all pass `deny.toml`). But it is young: 13 releases in five weeks, each a 0.x minor bump, and
  `uncad-model` makes `Entity` deliberately exhaustive, so **every new entity type is a breaking release** that needs a
  code change here. Its drawing is `uncad-model`'s, so CivKit would either translate `uncad-model` into its own neutral
  model (two models to keep in step) or adopt `uncad-model` as the engine's model (coupling the engine's model to an
  external 0.x crate, against ADR 0002's intent of a CivKit-owned neutral model). The maintainer's reading is that the
  project calls itself a 0.x spike.
- **`dxf`:** mature, but HATCH and R3 fail, and 30 crates (with an image decoder) ship for features 0.1 does not use.

## 4. Recommendation (revision 2)

**Still an own reader in `civkit-drawing`, now with `undxf` as the primary test-only cross-check.** The comparison
changes the reasons, not the conclusion:

- `undxf` meets R1, R3 and R4 and most of R2, which the first revision's case did not reflect. It fails two
  requirements that matter for 0.1: **R2 (MINSERT arrays reduced to one placement: F5 gives 11 bays, not 13)** and
  **R5 (no way to bound memory: up to 158 × the input on hostile content)**. Meeting both while using it means a
  CivKit pre-scan of the raw file (to count and budget before `undxf` allocates) and a second pass for MINSERT rows
  and columns: a tokenizer of our own, plus an adapter from `uncad-model`, plus breaking upgrades to track.
- An own reader for the 0.1 subset is about the size of that pre-scan plus the entity handling (estimated 1 500–2 500
  lines with tests), with no dependencies, `forbid(unsafe_code)`, and the memory budget of §5 built in.
- **`undxf` as the cross-check (dev-dependency, never shipped):** it covers HATCH, header presence and unknown
  entities, which `dxf` does not, so it checks more of what our reader does. Tests read every fixture and the fuzz
  corpus with both readers and compare what both support: entity counts by type and layer, coordinates, bulges, hatch
  boundaries, insert transforms, header values and their absence. `dxf` is not needed as well.
- **Revisit later:** if `undxf` gains MINSERT arrays and caller-set limits and settles its API, replacing our reader
  with it becomes a contained change, because analysis is written against CivKit's model, not a reader.

Cost: we maintain a parser. Mitigations: the subset is fixed and versioned, fuzzing and the cross-check run in CI, and
the fixtures' expected values were set by construction and checked with ezdxf.

## 5. Proposed scope and limits for 0.1

**Scope (Q-C, unchanged):**
- **Input:** ASCII DXF, R12–R2018. **Binary DXF** is detected by its sentinel and reported as not supported in 0.1.
- **Encoding:** R2007 and later are UTF-8. Earlier files use `$DWGCODEPAGE`; 0.1 decodes `ANSI_1252` (a built-in
  table) and `\U+XXXX` escapes, and reports any other code page as not supported rather than guessing.
- **Sections:** HEADER (`$ACADVER`, `$INSUNITS`, `$MEASUREMENT`, `$DWGCODEPAGE`, each with "not stated" kept
  distinct), TABLES (LAYER), BLOCKS, ENTITIES. Others are skipped by structure, without interpreting their contents.
- **Entities:** as R2, including MINSERT rows, columns and spacing, each entity's layer, extrusion (OCS, for
  GEOMETRY.md §11) and elevation. HATCH: boundary paths of polyline type and of line and arc edges; ellipse and spline
  edges are reported as not supported in 0.1 (that hatch gets no area). Every other entity type is counted per type and
  layer, not stored.
- **The reader produces a parsed drawing, not results.** Block expansion, transformations, lengths and areas are
  analysis (task 3).

**Limits (Q-D, revised).** The bound is on memory itself, not only on counts:

1. **Input size:** at most **256 MiB** (checked before reading; the input is held once, as bytes).
2. **No whole-file copies:** the reader works over the input bytes and decodes one value at a time; it never builds
   a decoded copy of the file.
3. **A storage budget the reader enforces:** every element it keeps is charged its size as it is stored (a fixed
   record per entity, 24 bytes per vertex, the length of each string, each table entry). Unrecognised entities are
   counted per type and layer, not kept. When the charge would pass the budget, reading stops with an error naming the
   limit. Default budget **512 MiB**, a named constant, lowerable by the caller.
4. **Per-item caps,** so one entity cannot take the whole budget: 1 000 000 vertices in one polyline or hatch path;
   64 KiB in one string.
5. **Peak memory bound:** input + budget + a fixed overhead, so at the defaults **under 1 GiB for any input**. It is
   shown, not assumed: tests measure the actual peak (a counting allocator from a dev-dependency, as `civkit-drawing`
   itself forbids `unsafe`) on adversarial synthetic inputs (many tiny entities, unrecognised entities, long
   polylines, long strings, many layers) and require peak ≤ input + budget + overhead, and that the budget error comes
   before the bound is reached.
6. **Expansion (task 3), its own budget:** block nesting depth 32; expanded entities charged against a second budget
   (default 512 MiB), so a MINSERT of nested arrays cannot multiply past it. Its numbers are part of the task 3
   proposal.

The entity-count cap of revision 1 is dropped: the storage budget bounds it more directly.

## 6. Tests and fuzzing (Q-E, unchanged)

- **Tests first:** fixture-driven tests for every R2 entity and R3 header case, using the values in the fixtures
  README (including F5's 13 bays); malformed-input tests (truncation, wrong group codes, non-numeric values, NaN and
  infinity, missing EOF, each limit exceeded); the memory tests of §5; the `undxf` cross-check.
- **Property tests** (proptest): writing a random small drawing in the 0.1 subset and reading it back gives the same
  entities.
- **Fuzzing:** a `cargo-fuzz` target on the reader, seeded with the fixtures, run under the default budget so memory
  exhaustion counts as a failure. cargo-fuzz needs nightly Rust, so it runs as its own CI job (Linux, nightly), 10
  minutes per run, and for a longer recorded run before 0.1 (8 hours in total). Crashes become regression tests.

## 7. Questions for the maintainer

| # | Question | Recommendation | State |
|---|---|---|---|
| **Q-A** | Own reader in `civkit-drawing` (§4), rather than adopting `undxf` 0.11.0 or `dxf` 0.6.2? | **Own reader**: `undxf` fails R2 (MINSERT) and R5 (memory) and would bring an external 0.x model; `dxf` fails R2 (HATCH) and R3. | Held; revised |
| **Q-B** | Use **`undxf` 0.11.0** as the test-only cross-check (instead of `dxf`, as revision 1 proposed)? | **Yes**: it covers more of what the reader does. | Held; revised |
| **Q-C** | 0.1 scope of §5. | **As proposed.** | Under consideration |
| **Q-D** | Limits of §5: 256 MiB input; no whole-file copies; a 512 MiB storage budget charged per stored element; per-item caps; peak under 1 GiB at the defaults, measured in tests; expansion budgeted separately in task 3. | **As proposed.** | Revised |
| **Q-E** | Fuzzing: nightly `cargo-fuzz` CI job, 10 minutes per run, 8 hours recorded before 0.1, under the default budget. | **As proposed.** | Under consideration |
| **Q-F** | Record as D-27 in a new ADR (0015), amending ADR 0002's open question. | **Yes**, once Q-A to Q-E are settled. | Held |

## 8. After acceptance

1. ADR 0015 (D-27); DECISIONS, IMPLEMENTATION_PLAN Phase 4 task 1 marked decided; ACCEPTANCE_CRITERIA Phase 4 gains
   the fuzzing duration and the memory bound.
2. Task 2, tests first: the reader in `civkit-drawing` (the parsed drawing, the storage budget, the cross-check).
3. Then the analysis proposal (task 3: block expansion and its budget, OCS and transformations, lengths and areas, the
   unit check, schedules, and how geometry notes and "not supported in 0.1" reach users) before its code.

## Appendix: how the measurements were taken

A scratch crate outside the repository (Rust 1.100, release build, Windows 11) depending on `dxf = "=0.6.2"`,
`undxf = "=0.11.0"` and `serde_json`. Fixtures: the seven files in `tests/fixtures/dxf`. Peak heap: a counting global
allocator, peak bytes above the level before the call, input bytes already allocated. Corruptions: a fixed-seed
xorshift generator, 450 variants per fixture in five kinds. Synthetic inputs: 200 000 `LINE` entities; one
`LWPOLYLINE` with 500 000 vertices; 300 000 entities of an unknown type `FOO`. The harness can be re-created from this
description; its source is not committed (no DXF code before the decision).
