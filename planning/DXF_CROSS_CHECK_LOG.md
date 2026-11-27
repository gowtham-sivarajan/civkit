# DXF cross-check log: triaged disagreements with `undxf`

> **Provenance:** Claude Sonnet 5.5 (`claude-sonnet-5-5`) · first drafted 2026-11-27 18:18 IST · last edited 2026-11-27 18:23 IST by Claude Sonnet 5.5 (`claude-sonnet-5-5`)

Gate X2 of [ADR 0015](decisions/0015-dxf-reader.md) (amended 2026-11-27) compares CivKit's DXF reader with `undxf`
0.11.0. It is **strict** for the fixtures and the curated well-formed corpus (`crates/civkit-drawing/tests/well_formed/`).
On the **fuzz corpus** it runs in **report-and-triage** mode: each disagreement on an input both readers accept is
listed in the Fuzz job's summary (input name, FNV-1a 64 hash, first difference) and the job still passes. Each one is
triaged here:

- **Promoted:** a disagreement on a valid DXF; the file goes into the strict corpus and the bug is fixed.
- **Refused:** an ambiguity CivKit should not resolve (a value stated twice); CivKit now refuses it, with a test.
- **Tolerance difference:** malformed input the two readers recover from differently. Reviewed; CivKit does not copy
  `undxf`'s recovery choice. Malformed-input behaviour is covered by CivKit's own refusal and safety tests (X3–X6).

Inputs below came from Fuzz #2's failure (RA-4, from the job log the maintainer read) and a local stand-in corpus (the
fixtures mutated with a fixed seed). Hashes are FNV-1a 64 of the input bytes.

## Refused (ambiguities)

| ID | Input (hash) | What | Before | Now |
|---|---|---|---|---|
| RA-1 | `80074998347c8f80` | An LWPOLYLINE vertex with two y values (`20` twice after one `10`) | CivKit kept the last, `undxf` the first | `Malformed` (test `a_repeated_value_is_refused_not_chosen`) |
| RA-2 | `731243014465f39b` | `$INSUNITS` with two values (`70=6`, then `70=1`) | CivKit kept the last, `undxf` the first | `Malformed` (same test) |
| RA-3 | `0f5c4f8956defecb` | Layer `0` defined twice in the LAYER table | CivKit kept both records, `undxf` one | `Malformed`; blocks likewise (test `a_layer_or_block_defined_twice_is_refused`) |
| RA-4 | CI Fuzz #2: `2712f99c02149f916ce1cbd1770f5ea2952c775b` (SHA-1 file name) | The fuzzer spliced header values (`$LIMMAX` 420, 297) and an empty text into TEXT `11E` of the BricsCAD fixture: position, height and text stated twice | CivKit kept the last values, `undxf` the first (the original) | `Malformed` under the RA-1 rule (test `a_text_with_spliced_second_values_is_refused`) |

The rule behind RA-1, RA-2 and RA-4: a group code that is single-valued for an entity or record (the common codes 5, 8, 67,
210/220/230 and each entity's own list), a vertex's 20 or 42 (only after its 10), and each header variable CivKit reads
appear at most once. Application groups (102) and extended data (1000 and up) are not checked.

## Reviewed tolerance differences

| ID | Inputs (hash) | Damage | `undxf` | CivKit | Decision |
|---|---|---|---|---|---|
| TD-1 | `df13a38fc9cbd613`, `22b74b743b74cef7`, `3839bb625e311f4f` | The `100 AcDbEntity` subclass marker corrupted (`1e5`, a stray byte, `2.5`) | Reads group codes by subclass; drops the layer (`8`) after an unknown subclass, so the entity lands on layer `0` | Ignores subclass markers (R12 files have none; the codes it reads mean the same in every subclass) and keeps the layer | Reviewed 2026-11-27 (maintainer): not copied |
| TD-2 | `be5fff157ac1fbd3` | `$DWGCODEPAGE`'s value pair deleted, so a coordinate (`10 -10000.0`) follows it | Takes the first group after the name as the value (`-10000.0`) | Reads a variable only from its defined group code (3), so the code page is not stated | Reviewed 2026-11-27: not copied |
