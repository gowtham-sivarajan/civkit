# Proposal: list and choice inputs in the calculation contract (D-23)

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-25 19:15 IST · last edited 2026-11-25 21:50 IST

> **Status:** Direction decided by the maintainer (2026-11-25 21:32 IST):
> 1. option A, parallel lists with one unit per list;
> 2. required choice inputs for method selection, with the selected method in the trace;
> 3. "Array (vectorised) inputs are out of scope for 0.1" means Python batching and is to be clarified. The sentence
>    is in API_DESIGN §8 (Python API); the earlier proposal wrongly cited §7.
>
> §3 is the full design. The maintainer **confirmed** it on 2026-11-25 (21:49 IST) and asked for four edge rules
> to be written down before the contract changes. They are marked **Clarified** below:
> - list presence (§3.1);
> - limits on outputs too, and descriptor checks for list outputs (§3.3, §3.8);
> - exactly one choice step per choice input (§3.6);
> - structured item indices in the trace (§3.6).
>
> Next: ADR 0011, the API_DESIGN text (§5 of this document), then the contract extension, tests first. Nothing in
> the contract or code depends on it yet.

## 1. The gap

The accepted contract ([API_DESIGN.md §2–§3](../docs/API_DESIGN.md); `InputSpec`, `OutputSpec`, `InputValue`,
`TraceStep` in `civkit-core`) has only **scalar** inputs, outputs and trace values. Three of the five D-20
calculations are lists by nature, and one needs a method choice:

| Calculation (D-20) | Needs | Reference case |
|---|---|---|
| Traverse | ordered legs (bearing, distance); per-leg latitudes, departures, corrections, adjusted values | C3 (6 legs) |
| Levelling (HI, rise and fall) | ordered backsight/foresight readings; per-point reduced levels | C4 (5 setups), C9 |
| Average end area | ordered stations and end areas (cut and fill); per-segment volumes | C1 (4 stations), C2 (6) |
| Average end area + pyramid | a **visible, required method input** (ADR 0009, condition 3) | C2 |

## 2. Options considered

| Option | Shape | Outcome |
|---|---|---|
| **A. Parallel lists** | `"areas": { "value": [40, 42, 19, 34], "unit": "m^2" }`; related lists share an index | **Chosen** |
| B. Records | `"legs": [{ "bearing": …, "distance": { … } }, …]` | Not chosen: a second, nested unit model |
| C. Numbered scalars | `area_1` … `area_n` | Not chosen: fixed length, poor descriptors |
| D. One step per call | the caller loops | Not chosen: closure and adjustment need all values |

The known weakness of A, noted by the maintainer: equal lengths cannot catch two lists whose entries were paired in
the wrong order. The design answers this by making the shared index explicit everywhere (§3.2, §3.5–§3.7). The
error itself cannot be detected.

## 3. Design

### 3.1 Input kinds

`InputSpec` gains `kind`; the default `scalar` keeps every existing descriptor valid unchanged.

| `kind` | Value in a request | Extra descriptor fields |
|---|---|---|
| `scalar` | as today: a number, a DMS array, or a bearing string | — |
| `list` | a JSON array of numbers, or of bearing strings (`"unit": "bearing"`); **one unit for the whole list** | `group` (required) |
| `choice` | a string | `choices`: the allowed values (non-empty, unique, lower-case `snake_case`) |

- **List elements.** Each element is checked as a scalar input would be: finite, then unit conversion, then
  `domain`. Lists of DMS arrays are not supported in 0.1, because no 0.1 calculation needs them.
- **Choice inputs.** In 0.1 a choice input must be **required with no default**, so the caller always states the
  method; `check_descriptor` enforces this. Choice values are stable identifiers: renaming or removing one is a
  breaking change (version bump, API_DESIGN §4), and adding one is not.
- **Presence (Clarified).** In 0.1 every list input is **required**, with no default, and `check_descriptor`
  enforces this. A group's length is therefore always the common length of all its lists; no rule is needed for
  omitted lists. Optional lists would be a later decision.

### 3.2 Groups: the index relationship, declared once

A descriptor gains `groups`, a list of named index sets:

```json
"groups": [
  { "name": "legs", "item": "leg", "min_len": 3,
    "description": "Traverse legs in order of measurement; leg k runs from station k to station k + 1" }
]
```

- Every list input names its group (`"group": "legs"`). **All lists in one group must have the same length.**
- `item` is the singular word used to number entries in messages and traces ("leg 3"). Numbering starts at 1.
- `min_len` is the smallest allowed length; it must be at least 1.
- `check_descriptor` rejects a group with no list input, a list naming an unknown group, and an empty `item`.

### 3.3 List outputs and their length relation to the inputs

`OutputSpec` gains `kind` (`scalar` by default, or `list`). A list output declares its length **relative to a
group**:

```json
{ "name": "volumes", "kind": "list",
  "length": { "group": "stations", "offset": -1, "item": "segment" },
  "description": "Volume of each segment; segment k lies between station k and station k + 1" }
```

- **Length rule:** length = length of the group + `offset`, with `offset` ∈ {−1, 0, +1}.
  - 0: one value per group item, e.g. one adjusted latitude per leg.
  - −1: one value between consecutive items, e.g. one volume per segment between stations.
  - +1: one value per item plus one, e.g. reduced levels for the starting point and each foresight point.
- **`item` names the output's entries** ("segment 2"). When it differs from the group's item, the `description`
  must say how they pair, as in the example.
- **Descriptor checks for list outputs (Clarified).** `check_descriptor` rejects a list output whose:
  - `length.group` does not name a declared group;
  - `offset` is not −1, 0 or +1;
  - smallest possible length, `min_len` of its group + `offset`, is below 1 (no list output may be empty);
  - `item` or `description` is empty;
  - `item` label is already used for a different index set. Item labels must be unique across groups and list
    outputs; the one exception is a list output with `offset` 0, which may reuse its group's label.
- **The registry checks the actual length after `compute`.** A mismatch is an `INTERNAL` error: it is a defect in
  the calculation, not in the input.

### 3.4 Requests and responses

```json
"inputs": {
  "bearings":  { "value": ["N 3 45 E", "S 89 12 E"], "unit": "bearing" },
  "distances": { "value": [584.21, 211.04], "unit": "ft" },
  "method":    { "value": "pyramid_at_zero_end" }
}
```

- **List output in a response:** `"volumes": { "value": [3570, 2510.5, 2032.5], "unit": "m^3" }`.
- `output_units` applies to every element of a list output.
- A choice is never an output. Its value appears in the trace (§3.6).

### 3.5 Validation and errors

No new error codes. Each case below is listed with its contract code:

| Case | Code | `input` | Message names |
|---|---|---|---|
| element not finite, or outside `domain` | `OUT_OF_DOMAIN` / `INVALID_VALUE` as for scalars | list name | the item: "leg 3: distance must be greater than 0" |
| element unit or bearing malformed | as for scalars (`UNIT_MISMATCH`, `INVALID_VALUE`, …) | list name | the item |
| a value is not an array, or an element is the wrong JSON type | `INVALID_VALUE` | list name | the item, where known |
| lists in one group differ in length | `INVALID_VALUE` | the first list that differs | both lengths and the group: "group 'legs': bearings has 6 legs, distances has 5" |
| list shorter than the group's `min_len` | `INVALID_VALUE` | list name | the minimum |
| unknown choice | `INVALID_VALUE` | choice name | every allowed value |

- The error object gains an optional **`index`** field: the 0-based array position, for programs.
- Messages use the 1-based item number for people. Both appear together, e.g.
  `{ "code": "OUT_OF_DOMAIN", "input": "distances", "index": 2, "message": "leg 3: distances[2] = -5 ft must be greater than 0" }`.
- `index` is an addition to the error object (API_DESIGN §5); clients that ignore it keep working.
- As today, every input problem is collected and returned together.

### 3.6 Trace

A trace step is one of two forms; a step never has both `value` and `choice`.

| Form | Fields |
|---|---|
| numeric (as today) | `step`, `value`, `unit` (omitted when dimensionless), optional `reference`, optional **`item`** |
| choice (new) | `step` (the choice input's name), **`choice`** (its value) |

**Item indices are structured (Clarified).** A per-item step carries `"item": { "label": "leg", "index": 2 }`.
- `index` is 0-based, like the error `index` (§3.5).
- The step text starts with the 1-based label for people, e.g. `"leg 3: Lat = d·cos α"`.
- Clients read `item` and never need to parse the text.
- In `civkit-core`, per-item steps go through one helper that writes the text prefix from `item`, so the two cannot
  disagree.
- After `compute`, the registry returns `INTERNAL` when:
  - an `item` label is not a declared group or list-output item;
  - its `index` is not below that list's length;
  - the step text does not start with `"<label> <index + 1>: "`.

**Choice steps: exactly one per choice input (Clarified).** `compute` records every choice input exactly once, as
a choice step, before its first numeric step and in the order the inputs are declared. Example:

```json
{ "step": "method", "choice": "pyramid_at_zero_end" }
```

After `compute`, the registry returns `INTERNAL` for a choice step that is:
- missing;
- duplicated;
- naming an undeclared input or a non-choice input;
- carrying a value different from the request;
- placed after a numeric step, or out of declaration order.

Numeric steps keep today's checks: finite value, internal unit.

### 3.7 Reference-case files

- An expected list output gives `"value": [ … ]`. The output's `tolerance` applies to each element.
- An element may be `null`, meaning "not printed by the source, not checked". At least one element must be checked.
- The harness reports a mismatch by item: "segment 2: got 2510.5, expected 2511 ± 0.5".

### 3.8 Compatibility and interfaces

- A **pure addition** under API_DESIGN §4 and §11: `kind` defaults to `scalar`, and `groups`, `index` and `choice`
  steps are new optional fields. Existing calculations, their reference cases and stored requests are unaffected.
- **Python (Phase 3):** list inputs take a list of numbers with one `civkit.Q` unit, or a list of `Q` of one
  dimension. List outputs come back as lists of `Q`.
- **MCP (Phase 5):** list inputs become JSON arrays in the tool schema, with `minItems`, and choices become `enum`.
- **FFI:** unchanged; JSON in, JSON out.
- **Limits (Clarified).** Every list, input **or output**, has at most 10 000 elements. This bounds response and
  trace size; 0.1 cases use at most 7.
  - **The group's limit.** A group's maximum length is 10 000 minus the largest positive `offset` among its list
    outputs, e.g. 9 999 when a +1 output exists. Request validation enforces it: a longer list is `INVALID_VALUE`,
    and the message gives the maximum. A correct calculation therefore never produces an output over the cap.
  - **After `compute`.** A list output that is longer than 10 000 elements, or whose length breaks its rule
    (§3.3), is `INTERNAL`.

### 3.9 Not in D-23

- **Conditional requirements stay in prose in 0.1.** Example: the beam's `point_load_position`, required only when
  `point_load > 0`. The maintainer noted that generated forms and API docs (Phases 3–5) must show such conditions
  prominently.
  - Recorded as a Phase 3 task in IMPLEMENTATION_PLAN.
  - A machine-readable rule (e.g. `required_when`) would be a separate, later decision.
- Optional lists and list defaults; lists of DMS values; nested lists; records (option B); lists of choices.

## 4. Work after confirmation

1. ADR 0011 (D-23), and the API_DESIGN text of §5 of this document.
2. Tests first:
   - descriptor checks (groups, list outputs, choices);
   - request validation and error shapes;
   - output length checks;
   - choice trace steps;
   - harness support for list expectations.
3. Then `civkit-core` (`InputSpec`, `OutputSpec`, `InputValue`, `TraceStep`, `CalcError`, registry, `check_descriptor`)
   and the reference harness.
4. Then traverse, levelling and earthwork, one at a time. Each descriptor is proposed for review before coding
   (D-20 condition 4).

## 5. API_DESIGN changes (applied only after confirmation)

| Section | Change |
|---|---|
| §2 Descriptor | `InputSpec.kind`, `group`, `choices`; `OutputSpec.kind`, `length`; `Descriptor.groups` — as §3.1–§3.3 |
| §3 Request and response | list and choice values; list outputs — as §3.4 |
| §5 Errors | the optional `index` field; the list and choice cases of §3.5 (no new codes) |
| §6 Trace | numeric and choice step forms; structured `item`; choice-step rules — as §3.6 |
| §7 Units | one unit per list; `output_units` applies to every element |
| §8 Python API | Replace "Array (vectorised) inputs are out of scope for 0.1" with: "Batch (vectorised) calls — one calculation applied to many independent input sets, e.g. from Python — are out of scope for 0.1. This is different from a list that is part of one calculation's input (§2, `kind: list`), which Python passes as a list." |
| §11 Compatibility | note that list and choice kinds are additive |
| `tests/reference/README.md` | list expectations and `null` elements — as §3.7 |
