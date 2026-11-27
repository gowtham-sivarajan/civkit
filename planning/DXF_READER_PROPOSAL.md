# Proposal: the DXF reader (Phase 4, task 1)

> **Provenance:** Claude Sonnet 5.5 (`claude-sonnet-5-5`) · first drafted 2026-11-27 14:10 IST · last edited 2026-11-27 14:10 IST

> **Status:** **Proposed, for the maintainer's review.** Decision ID **D-27**. Nothing is implemented. Built on:
> D-12 / [ADR 0002](decisions/0002-neutral-drawing-model.md) (neutral drawing model and DXF reader in the Rust engine),
> [SECURITY.md](../docs/SECURITY.md) (hardened parser, limits, fuzzing), [MVP_SCOPE.md](MVP_SCOPE.md) §3 (read-only
> analysis: entities by layer, lengths and areas, drawing-unit consistency, schedules), the fixtures in
> [tests/fixtures/dxf](../tests/fixtures/dxf/README.md), and D-22 ([GEOMETRY.md](../docs/GEOMETRY.md)).

## 1. What has to be decided

ADR 0002 left one question open: *"DXF parsing adds a dependency (an existing Rust DXF crate, to be evaluated in
Phase 4) or our own reader."* This proposal evaluates the candidates, recommends one, and fixes what the reader
covers in 0.1 and how it is kept safe. The analysis contract (how drawing results, geometry notes and unsupported
features reach users) is a separate proposal (Phase 4 task 3), written after this one is decided.

## 2. What the reader must do

| # | Requirement | Source |
|---|---|---|
| R1 | Read ASCII DXF from R12 to R2018, as the fixtures have (AC1009, AC1018, AC1024, AC1027, AC1032) | fixtures |
| R2 | Entities for 0.1: LINE, LWPOLYLINE (bulges, closed flag), old POLYLINE/VERTEX/SEQEND (bulges), ARC, CIRCLE, INSERT and MINSERT (rotation, non-uniform and negative scale, nesting), HATCH (for its area), TEXT, linear DIMENSION (its measurement); every other entity counted and reported, never silently dropped | MVP_SCOPE, fixtures F1–F5 |
| R3 | Header: `$ACADVER`, `$INSUNITS`, `$MEASUREMENT`, with **"not stated" distinguishable from any stated value** | F3 (units mismatch), F4 (R12, no units): "the engine must report *units not stated*, not assume metres"; D-19 (no silent default unit) |
| R4 | Never crash, hang or exhaust memory on hostile input; limits on size, entity count and nesting; fuzzed | SECURITY.md |
| R5 | Numbers read exactly (correctly rounded), as the rest of the engine now does | Phase 3 finding (serde_json `float_roundtrip`) |
| R6 | Read-only; no writing in 0.1 | S-5, ADR 0002 |
| R7 | Dependencies pass `deny.toml`, the dependency rule, and stay small: everything here ships inside the engine | ARCHITECTURE §2, SECURITY |

## 3. Candidates evaluated (2026-11-27)

Facts from crates.io and `cargo info`; the `dxf` crate was also tested hands-on (below).

| Crate | Latest | First release | Downloads | Licence | Normal dependencies | Notes |
|---|---|---|---|---|---|---|
| **`dxf`** (IxMilia) | 0.6.2, 2026-10-03 | 2016 | ~394 000 | MIT | byteorder, chrono, encoding_rs, enum_primitive, image, itertools, num, uuid (30 crates in all) | Mature; reads and writes DXF and DXB; entity types generated from the DXF reference |
| `cadkit-dxf` | 0.5.4, 2026-11-16 | 2026-10-01 | ~3 100 | MIT OR Apache-2.0 | cadkit-core, encoding_rs | Six weeks old, 17 releases; brings its own drawing model, so we would translate model to model |
| `dxf-tools-rs` | 0.1.4, 2026-01-20 | 2026-01-20 | ~700 | MIT | ahash, anyhow, bitflags, flate2, indexmap, nalgebra, nom, rayon, … | Little use; heavy tree (rayon, nalgebra) |
| `cadrender-dxf` | 0.2.0, 2026-11-16 | 2026-11-14 | ~70 | MIT OR Apache-2.0 | cadrender-core, cadrender-acis, memchr, memmap2 | Two weeks old; a renderer's reader; memory-maps files |
| **Own reader** in `civkit-drawing` | — | — | — | Apache-2.0 | none | Scoped to R1–R7 |

**`dxf` 0.6.2, hands-on** (scratch crate outside the repository, all seven fixtures):

- **Reads every fixture** without error, with the right versions, layers, blocks, LWPOLYLINE, R12 POLYLINE, ARC,
  CIRCLE, INSERT (13 in F5), TEXT and DIMENSION.
- **Robust in a smoke test:** 3 150 corrupted variants of the fixtures (truncation, byte flips, odd numbers such as
  `1e308`, `NaN`, `-2147483649`, deleted lines, repeated slices): **0 panics, 0 parses slower than 2 s**; every bad
  input gave an error. (A smoke test, not fuzzing.)
- **Gap 1, HATCH:** the crate has **no HATCH entity**; F1's hatch is silently dropped (only hatch-related enums
  exist in its source). Hatch area is part of 0.1's "lengths and areas".
- **Gap 2, header defaults:** an absent header variable comes back as a default. F4 (R12, no `$INSUNITS`, no
  `$MEASUREMENT`) reads as `Unitless` and **`English`**. A unit-consistency check built on that would flag every file
  that simply omits `$MEASUREMENT` as imperial (R3 fails).
- **Weight:** 30 crates, including the `image` decoder with colour management (`moxcms`, `pxfm`; used for the DXF
  thumbnail), `uuid` with `getrandom`, `chrono`, the `num` family and the unmaintained-looking `enum_primitive`
  (0.1.1). All licences would pass `deny.toml`; the concern is attack surface and upkeep inside the engine, for
  features 0.1 does not use (writing, DXB, images).

The other three are too new or too heavy to rest the engine on.

## 4. Recommendation

**Write our own reader in `civkit-drawing`, scoped to R1–R7, with no dependencies, and use the `dxf` crate as a
test-only cross-check.**

- R2 and R3 fail with `dxf` as it is, and both sit at the centre of 0.1 (areas; the unit check). Working around them
  means reading the raw file a second time, at which point we own a reader anyway.
- The 0.1 subset of ASCII DXF is small: the format is pairs of lines (group code, value), and the needed entities
  are listed in R2. Estimated 1 500–2 500 lines of Rust with tests, no `unsafe` (the crate keeps
  `unsafe_code = "forbid"`), and limits built in rather than bolted on.
- Owning the reader lets it record **what it did not understand**: every unsupported entity type and count, every
  ignored header variable, binary DXF, unknown encodings. That record is what task 3 needs to tell users "not
  supported in 0.1" instead of silently dropping geometry.
- **`dxf` as a dev-dependency oracle:** tests read every fixture (and fuzz corpus files) with both readers and
  compare the entities both support: counts, layers, coordinates, bulges, insert transforms. It does not ship; its
  tree is still checked by cargo-deny in CI. This gives a second, independent implementation, as the reference
  cases do for calculations.

Cost: we maintain a parser. Mitigations: the subset is fixed and versioned (new entity types are additions), fuzzing
and the oracle run in CI, and the fixtures' expected values were set by construction and checked with ezdxf.

## 5. Proposed scope for 0.1

- **Input:** ASCII DXF, R12–R2018. **Binary DXF** is detected by its sentinel and reported as not supported in 0.1.
- **Encoding:** R2007 and later are UTF-8. Earlier files use `$DWGCODEPAGE`; 0.1 decodes `ANSI_1252` (a built-in
  table) and `\U+XXXX` escapes, and reports any other code page as not supported rather than guessing.
- **Sections:** HEADER (`$ACADVER`, `$INSUNITS`, `$MEASUREMENT`, `$DWGCODEPAGE`, each with "not stated" kept
  distinct), TABLES (LAYER), BLOCKS, ENTITIES. Others (CLASSES, OBJECTS, THUMBNAILIMAGE, ACDSDATA) are skipped by
  structure, without interpreting their contents.
- **Entities:** as R2, including each entity's layer, extrusion (OCS, for GEOMETRY.md §11) and elevation. HATCH:
  boundary paths of polyline type and of line and arc edges; ellipse and spline edges are reported as not supported
  in 0.1 (the hatch is then not given an area). Every other entity type is counted per type and layer.
- **The reader produces a parsed drawing, not results.** Block expansion, transformations and lengths and areas are
  analysis (task 3), in `civkit-drawing` on the neutral model and `civkit-geometry`.

**Limits** (proposed defaults; each refusal is an error naming the limit, never a crash):

| Limit | Default | Why |
|---|---|---|
| File size | 256 MiB | Larger than typical civil drawings; caps memory before parsing |
| Entities (model space plus all blocks) | 2 000 000 | Caps memory |
| Vertices in one polyline or hatch path | 1 000 000 | One entity cannot exhaust memory |
| Block nesting depth (at expansion, task 3) | 32 | Stops recursive or cyclic blocks |
| Expanded entities after INSERT/MINSERT (task 3) | 10 000 000 | Stops "block bombs" (a MINSERT of nested arrays) |

## 6. Tests and fuzzing

- **Tests first:** fixture-driven tests for every R2 entity and R3 header case, using the values in the fixtures
  README; malformed-input tests (truncation, wrong group codes, non-numeric values, NaN and infinity, missing EOF,
  each limit exceeded); the `dxf` oracle comparison.
- **Property tests** (proptest, as elsewhere): writing a random small drawing in the 0.1 subset and reading it back
  gives the same entities.
- **Fuzzing:** a `cargo-fuzz` target on the reader, seeded with the fixtures. cargo-fuzz needs nightly Rust, so it
  runs as its own CI job (Linux, nightly) for a fixed time per run, and for a longer recorded run before 0.1 (the
  acceptance criterion asks for "a defined minimum duration"; proposed: 10 minutes per CI run, and 8 hours in total
  recorded before 0.1). Crashes become regression tests.

## 7. Questions for the maintainer

| # | Question | Recommendation |
|---|---|---|
| **Q-A** | Own reader in `civkit-drawing` (§4), rather than adopting `dxf` 0.6.2? | **Own reader.** HATCH and the header defaults rule `dxf` out as the production reader. |
| **Q-B** | Use `dxf` 0.6.2 as a **dev-dependency** cross-check (test only, never shipped)? | **Yes.** An independent implementation for the entities both support. |
| **Q-C** | 0.1 scope of §5: ASCII R12–R2018; binary DXF and code pages other than 1252 reported as not supported; the R2 entities; HATCH with polyline, line and arc boundaries. | **As proposed.** |
| **Q-D** | The limits of §5 (256 MiB, 2 000 000 entities, 1 000 000 vertices, nesting 32, 10 000 000 expanded). | **As proposed**, adjustable later; each limit is a named constant with a test. |
| **Q-E** | Fuzzing: a nightly `cargo-fuzz` CI job, 10 minutes per run; 8 hours recorded before 0.1. | **As proposed.** |
| **Q-F** | Record as D-27 in a new ADR (0015), amending ADR 0002's open question. | **Yes.** |

## 8. After acceptance

1. ADR 0015 (D-27); DECISIONS, IMPLEMENTATION_PLAN Phase 4 task 1 marked decided; ACCEPTANCE_CRITERIA Phase 4 gains
   the fuzzing duration.
2. Task 2, tests first: the reader in `civkit-drawing` (the neutral model's parsed form, limits, the oracle).
3. Then the analysis proposal (task 3: block expansion, OCS and transformations, lengths and areas, the unit check,
   schedules, and how geometry notes and "not supported in 0.1" reach users) before its code.
