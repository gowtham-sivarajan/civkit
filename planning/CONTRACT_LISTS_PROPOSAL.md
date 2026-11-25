# Proposal: list and choice inputs in the calculation contract (D-23)

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-25 19:15 IST · last edited 2026-11-25 19:15 IST

> **Status: Proposed — awaiting the maintainer's decision.** Nothing in the contract or code depends on it yet.
> Found at the start of Phase 2C (2026-11-25), while preparing the D-20 calculations.

## 1. The gap

The accepted contract ([API_DESIGN.md §2–§3, §7](../docs/API_DESIGN.md); `InputSpec`, `OutputSpec`, `InputValue` in
`civkit-core`) has only **scalar** inputs and outputs: one number (or one DMS or bearing value) per name. Three of
the five D-20 calculations are lists by nature, and one needs a method choice:

| Calculation (D-20) | Needs | Reference case |
|---|---|---|
| Traverse | ordered legs (bearing, distance); per-leg latitudes, departures, corrections, adjusted values | C3 (6 legs) |
| Levelling (HI, rise and fall) | ordered backsight/foresight readings and setup distances; per-point elevations | C4 (5 setups), C9 |
| Average end area | ordered stations and end areas (cut and fill); per-segment volumes | C1 (4 stations), C2 (6) |
| Average end area + pyramid | a **visible method input** (ADR 0009, condition 3) | C2 |

Manning (trapezoid) and the beam fit the scalar contract and are implemented.

API_DESIGN §7 says "Array (vectorised) inputs are out of scope for 0.1". I read that as Python *vectorisation*
(calling one calculation over many values). It does not cover a single calculation whose input is a list. Please
confirm or correct this reading.

## 2. Options

| Option | Shape | For | Against |
|---|---|---|---|
| **A. Parallel lists** (recommended) | An input may be a list of numbers (or bearings) with **one unit for the whole list**: `"areas": { "value": [40, 42, 19, 34], "unit": "m^2" }`. The descriptor declares which lists must have the same length. Outputs may be lists the same way. | Smallest change; reuses the unit model unchanged (one unit per input); easy JSON Schema, Python and MCP mapping; existing calculations untouched | Related columns (bearing and distance of one leg) are separate lists, matched by position |
| B. Records | An input may be a list of objects: `"legs": [{ "bearing": "N 3 45 E", "distance": { "value": 584.21, "unit": "ft" } }, …]` | Keeps each leg together | A second, nested unit model; much larger change to validation, errors, schemas and every interface |
| C. Numbered scalars | `area_1` … `area_n` | No contract change | Fixed maximum length; unusable descriptors; poor for AI clients |
| D. One step per call | Calculations take one leg, setup or segment; the caller loops | No contract change | Closure, Bowditch adjustment and level-circuit adjustment need all values at once: D-20 cannot be met |

**Method choice.** Option A adds a second kind of input: a **choice**, a string from a fixed list declared in the
descriptor. For example: `"method": { "value": "pyramid_at_zero_end" }` with
`choices: ["average_end_area", "pyramid_at_zero_end"]`. The alternative is separate calculation IDs per method. That
also needs no new input kind, but it is less visible than "an explicit method input" (condition 3).

## 3. Recommended design (option A + choices)

- **Descriptor.** `InputSpec` gains `kind`:
  - `scalar`: the default, so today's calculations are unchanged.
  - `list`: has `min_len` and an optional `same_length_as` group.
  - `choice`: has `choices`.

  `OutputSpec` gains `kind`: `scalar` or `list`.
- **Request.**
  - A list's `value` is a JSON array.
  - Numbers carry one `unit` for the whole list. Bearings use `"unit": "bearing"`, with each element a bearing string.
  - Lists of DMS values are not supported in 0.1, because no 0.1 calculation needs them.
- **Validation.**
  - Every element is checked as a scalar would be: finite, unit converted, then the domain.
  - An element error names its position, e.g. `"input": "areas[2]"`.
  - Lists in one length group must match, otherwise `INVALID_VALUE`.
  - A list that is too short gives `INVALID_VALUE`.
  - An unknown choice gives `INVALID_VALUE`, and the message lists the allowed values. **No new error codes.**
- **Response.** A list output is `{ "value": [ … ], "unit": "ft" }`, with requested `output_units` applied to every
  element.
- **Trace.** Steps stay scalar; per-element steps carry the index (e.g. `Lat_3 = d_3·cos α_3`).
- **Compatibility.** This is a pure addition (API_DESIGN §4). Scalar calculations and their reference cases are
  unaffected.
  - Python (Phase 3) maps lists to lists of `civkit.Q`.
  - MCP schemas use JSON arrays.
  - The FFI is unchanged: JSON in, JSON out.
- **Work.**
  - Code: API_DESIGN §2, §3, §5, §7; `InputSpec`, `OutputSpec`, `InputValue`, the registry's validation and
    output conversion; `check_descriptor`; the reference-case harness (list values compared element by element).
  - Tests first, as for 2A and 2B.

## 4. What I need from you

1. Option A, B, C or D (or another).
2. Choice inputs, or separate calculation IDs, for the method selection.
3. Whether "Array (vectorised) inputs are out of scope" (API_DESIGN §7) is to be read as above.

On acceptance: an ADR (0011, D-23), the API_DESIGN changes, then the contract extension (tests first), then the
three list calculations one at a time. Before coding each, I propose its exact descriptor for your review, as
D-20 condition 4 asks: inputs, outputs, options, validation, warnings and trace steps.
