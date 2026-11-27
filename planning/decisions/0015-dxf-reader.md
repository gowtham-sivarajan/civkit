# ADR 0015 — The DXF reader: CivKit's own, with a reserved heap budget

> **Provenance:** Claude Sonnet 5.5 (`claude-sonnet-5-5`) · first drafted 2026-11-27 14:50 IST · last edited 2026-11-27 23:16 IST by Claude Sonnet 5.5 (`claude-sonnet-5-5`)

- **Status:** Accepted (maintainer, 2026-11-27: revision 2 of [DXF_READER_PROPOSAL.md](../DXF_READER_PROPOSAL.md),
  with two changes, both applied: Q-D tightened; the malformed-input evidence described as a smoke-test pass).
  Amended 2026-11-27 by the maintainer: the fuzz corpus is cross-checked in report-and-triage mode (X2, below).
- **Date:** 2026-11-27
- **Decision ID:** D-27
- **Relation to earlier decisions:** settles the open question of D-12 ([ADR 0002](0002-neutral-drawing-model.md)):
  *"an existing Rust DXF crate … or our own reader"*. Applies [SECURITY.md](../../docs/SECURITY.md)'s DXF row
  (hardened parser, limits, fuzzing). Phase 4 task 1.

## Context

0.1's drawing analysis needs a DXF reader that keeps HATCH boundaries and MINSERT arrays, tells "units not stated"
apart from any stated value, never crashes or hangs on hostile files, and has a memory bound that holds for any input.
Two crates were tested hands-on against the fixtures (the proposal, §3):

- `dxf` 0.6.2 drops HATCH, reports absent header variables as defaults, and brings 30 crates, including an image
  decoder.
- `undxf` 0.11.0 keeps HATCH, header absence and unrecognised entities, but reads a MINSERT as its first placement only
  (fixture F5: 11 of 13 bays), takes no limits (peak heap measured at 39 × the input for plain lines and 158 × for
  unrecognised entities), and brings its own 0.x model whose entity type is exhaustive by design.

Both passed a malformed-input **smoke test** (3 150 corrupted fixtures: no panic or hang). That is not R4's limits and
fuzzing, which neither crate was shown to meet.

## Decision

1. **Own reader (Q-A).** `civkit-drawing` gets its own DXF reader, no dependencies, `forbid(unsafe_code)`. It produces
   a parsed drawing; block expansion, transformations and results are analysis (task 3).
2. **Cross-check (Q-B; amended 2026-11-27, see below).** `undxf = "=0.11.0"` as a **dev-dependency only**, never
   shipped. Tests read every fixture and every case of a curated set of well-formed DXF files with both readers and
   compare what both support, strictly. The fuzz corpus is compared too, in **report-and-triage** mode (see the
   amendment). `undxf` cannot validate MINSERT arrays (first placement only): **the fixtures' expected values (F5: 13
   bays) are the check for those.**
3. **Scope for 0.1 (Q-C).** ASCII DXF R12–R2018; binary DXF detected and reported as not supported; UTF-8 (R2007+) and
   `ANSI_1252` with `\U+XXXX` escapes, other code pages reported as not supported; HEADER (`$ACADVER`, `$INSUNITS`,
   `$MEASUREMENT`, `$DWGCODEPAGE`, "not stated" kept distinct), TABLES (LAYER), BLOCKS, ENTITIES; LINE, LWPOLYLINE,
   POLYLINE/VERTEX/SEQEND, ARC, CIRCLE, INSERT and MINSERT (rows, columns, spacing), HATCH (polyline, line and arc
   boundaries; ellipse and spline edges reported as not supported, that hatch gets no area), TEXT, linear DIMENSION;
   every other entity counted per type and layer, never silently dropped.
4. **Memory (Q-D).**
   - Input at most **256 MiB**, read in place; no whole-file copies.
   - **One heap budget** (default **512 MiB**, a named constant, lowerable by the caller). **Every heap allocation the
     reader makes reserves its bytes from the budget first** and releases them when freed: container capacity (not
     length; growth by `try_reserve_exact`), strings, temporary buffers, and maps and sets (including the counts of
     unrecognised entities by type and layer) at a stated per-entry cost. A reservation past the budget, or a refused
     allocation, ends reading with a clean error naming the limit.
   - Per-item caps: 1 000 000 vertices per polyline or hatch path; 64 KiB per string; 256 bytes per unrecognised type
     name. Iterative parsing, no unbounded recursion.
   - **The bound is on reader-requested heap**: at any moment, the heap the reader has requested is at most the budget,
     with the input bounded separately. It is **not a process-memory ceiling** (allocator overhead and fragmentation,
     the caller's data, stacks and the rest of the program are outside it), and interfaces document it as such.
   - **Analysis shares the budget:** the parsed drawing and its expanded form (task 3) reserve from **one combined
     budget**; block nesting depth at most 32.
5. **Fuzzing (Q-E).** A `cargo-fuzz` target on the reader, seeded with the fixtures, under the default budget with the
   heap check; a nightly CI job of 10 minutes per run; 8 hours in total recorded before 0.1. Crashes become regression
   tests.

## Implementation gates (the reader is not done until all pass in CI)

| Gate | What must be shown |
|---|---|
| **X1 — fixtures** | Every fixture reads with its expected entities and header values, including F4's units not stated and F5's MINSERT (13 bays once expanded in task 3; the reader keeps rows, columns and spacing). |
| **X2 — cross-check** | **Strict:** for every fixture and every file of the curated well-formed corpus (`crates/civkit-drawing/tests/well_formed/`), what both readers support agrees with `undxf` 0.11.0 (MINSERT compared on its first placement only). **Report and triage:** for every fuzz-corpus file both readers accept, disagreements are recorded (input hash and differing fields) and the job passes; it fails only on a crash, a hang, a budget overrun or an allocation failure. |
| **X3 — budget accounting** | A counting allocator (dev-dependency) measures the heap requested during each read of every fixture and of adversarial inputs (many tiny entities; unrecognised entities with many distinct types and layers; long polylines; long strings; many layers and blocks). **Measured peak ≤ reserved peak + a small fixed slack** in every case, showing no allocation escapes the reservation. |
| **X4 — budget exhaustion** | Each fixture and adversarial input, read under budgets from a few bytes upward, ends in success or the budget error: never a panic, abort or overrun (measured peak ≤ budget + slack). |
| **X5 — malformed input** | Truncation, wrong group codes, non-numeric values, NaN and infinity, missing EOF and every limit exceeded give clean errors. |
| **X6 — fuzzing** | The CI fuzz job runs as Q-E states, under the budget and heap check; the 8-hour record exists before 0.1. |

## Amendment (2026-11-27): the fuzz corpus is compared in report-and-triage mode

The maintainer decided this after Fuzz #2. Fuzz-generated inputs are mostly malformed, and two readers can be
differently tolerant of the same damage without either being wrong (example: a corrupted subclass marker, where
`undxf` drops the layer that follows and CivKit keeps it). A strict comparison would push CivKit to copy `undxf`'s
recovery choices. So:

- **Strict X2** covers the hand-authored fixtures and a curated set of well-formed DXF files. Any disagreement there
  fails.
- **Fuzz corpus:** every file both readers accept is compared; each disagreement is recorded with the input's hash and
  the differing fields (in the CI job summary), and the job still passes. It fails on a crash, a hang, a budget overrun
  or an allocation failure.
- **Triage:** a disagreement on a valid DXF is promoted into the strict corpus (and fixed). A disagreement caused by
  ambiguity CivKit should refuse (a value stated twice) is fixed by refusing it, with a test. A tolerance difference on
  malformed input is logged as reviewed in the maintainer's triage log (kept locally since 2026-11-27); CivKit does not copy
  `undxf`'s recovery choice. Malformed-input behaviour stays covered by CivKit's own refusal and safety tests (X3–X6).

## Consequences

- ADR 0002's open question is settled; ADR 0002 points here.
- `civkit-drawing` stays dependency-free in normal builds; `undxf` and a counting-allocator crate are dev-dependencies,
  still checked by cargo-deny.
- The task 3 proposal (expansion, transformations, lengths and areas, the unit check, schedules, and how geometry notes
  and "not supported in 0.1" reach users) must fit within the combined budget.
- If `undxf` later gains MINSERT arrays and caller-set limits and its API settles, replacing the reader is a contained
  change, since analysis is written against CivKit's model.
