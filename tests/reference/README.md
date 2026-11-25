# Reference cases

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-25 11:00 IST · last edited 2026-11-25 11:00 IST

A reference case is a published worked example turned into a test. It has a known input, a cited method, and an independently verified expected result ([ENGINEERING_STANDARDS.md](../../docs/ENGINEERING_STANDARDS.md)).
Every registered calculation must have at least one; CI fails otherwise.

The harness `crates/civkit-core/tests/reference_cases.rs` runs every `.json` file in this folder on every build.
A file that does not parse is a failure, not a skip.

## Layout

```
tests/reference/
└── <calculation id>/              e.g. fluids.open_channel.manning/
    ├── <case>.json                one file per worked example
    └── …
```

The folder name must equal the calculation ID in the request.

## File format

```json
{
  "provenance": "who wrote this file, model or person, and when",
  "case": "hds4-ex4-2",
  "title": "Trapezoidal channel, uniform flow (FHWA HDS-4 Example 4.2)",
  "source": {
    "kind": "published",
    "register_id": "S2",
    "citation": "FHWA, Introduction to Highway Hydraulics, HDS-4 (1997, metric), Example 4.2",
    "location": "§4.3, PDF pp. 50–52",
    "licence": "Public domain (US federal government work)",
    "url": "https://www.fhwa.dot.gov/engineering/hydraulics/pubs/hds4intr.pdf"
  },
  "request": { "id": "…", "version": 1, "inputs": { "…": { "value": 1.5, "unit": "m" } } },
  "expected": {
    "outputs": { "discharge": { "value": 2.96, "unit": "m^3/s", "tolerance": { "relative": 0.005 } } },
    "warnings": [],
    "errors": []
  },
  "verification": "How the expected values were checked independently, and any source erratum."
}
```

| Field | Rules |
|---|---|
| `source.kind` | `published` for real calculations. `demo` is allowed only for `math.demo.*` (the template). |
| `source.register_id` | Required for `published`: the source's S-number in [REFERENCE_SOURCES.md](../../planning/REFERENCE_SOURCES.md). Only openly licensed or public-domain sources (D-18). |
| `request` | Exactly what an interface would send ([API_DESIGN.md §3](../../docs/API_DESIGN.md)). |
| `expected.outputs` | Each with `value`, `unit` (omit for dimensionless) and a `tolerance`: `{ "relative": r }` or `{ "absolute": a }`. |
| Tolerance | Reflects the precision printed in the source. Widen it only to cover the source's own intermediate rounding, and explain why in `verification`. |
| `expected.warnings` / `expected.errors` | Warning or error codes, in order. A case expecting errors has no `outputs`. |
| `verification` | Required. Say how the result was re-derived (hand calculation, script, second source) and record any erratum. |

Source errata are recorded, never silently "fixed" into the expected values.
Collected cases waiting for their calculations (C1–C9) are listed in [REFERENCE_CASES.md](../../planning/REFERENCE_CASES.md).
They move here in Phase 2.

The `math.demo.weighted_mean/` cases demonstrate the format. They exercise the registry with the template calculation and are not engineering results.
