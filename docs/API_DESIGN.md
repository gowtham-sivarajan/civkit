# API Design

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-25 00:29 IST (§3 example from README.md, 2026-11-24 23:41 IST) · last edited 2026-11-25 00:29 IST

> **Status:** Draft — a Phase 0 deliverable. Sections marked **TBD** must be settled before Phase 0 can exit.
> Related decisions: D-5 (units), D-6 (FFI), D-13 (MCP SDK).

## 1. Design goals

1. **One contract, every interface.** Python, MCP, FFI and the desktop app all speak the same request/response shape.
2. **Self-describing.** Every calculation publishes a descriptor; interfaces are generated from it.
3. **Language-neutral.** The contract is JSON; nothing in it is Rust-specific.
4. **Stable and versioned.** A calculation ID + version always returns the same result for the same input.
5. **Explainable.** Every response carries a trace, references, assumptions and warnings.

## 2. Calculation descriptor

Each calculation registers a descriptor with:

| Field | Meaning |
|---|---|
| `id` | Namespaced identifier, `domain.topic.method` (e.g. `fluids.open_channel.manning`) |
| `version` | Integer, starts at 1 |
| `title`, `summary` | Human-readable name and one-paragraph description |
| `inputs` | Name → dimension, default unit, required/optional, allowed range, description |
| `outputs` | Name → dimension, default unit, description |
| `references` | Citations (book, edition, page/section; or code clause number) |
| `assumptions` | Plain-language list |
| `validity` | Machine-checkable input limits and what happens outside them (warning or error) |
| `examples` | Links to reference cases in `tests/reference/` |

The descriptor is exported as JSON Schema for MCP and for validation in other languages.

## 3. Request and response

```json
// request
{
  "id": "fluids.open_channel.manning",
  "version": 1,
  "inputs": {
    "n":                { "value": 0.013 },
    "area":             { "value": 2.4,   "unit": "m^2" },
    "wetted_perimeter": { "value": 4.2,   "unit": "m" },
    "bed_slope":        { "value": 0.001 }
  }
}

// response
{
  "id": "fluids.open_channel.manning",
  "version": 1,
  "outputs": {
    "velocity":  { "value": 1.675, "unit": "m/s" },
    "discharge": { "value": 4.020, "unit": "m^3/s" }
  },
  "trace": [
    { "step": "R = A / P",                     "value": 0.5714, "unit": "m"     },
    { "step": "V = (1/n) · R^(2/3) · S^(1/2)", "value": 1.675,  "unit": "m/s"   },
    { "step": "Q = V · A",                     "value": 4.020,  "unit": "m^3/s" }
  ],
  "warnings": [],
  "references": ["<textbook or code clause>"],
  "assumptions": ["Uniform, steady flow", "SI form of Manning's equation"]
}
```

Values in this example are rounded for display; the engine returns full `f64` precision.

**TBD:** whether `version` may be omitted (meaning "latest") in requests. Proposed: optional in interactive
use (Python, MCP), required in saved files and drawings so they stay reproducible.

## 4. IDs and versioning

- IDs are lowercase, dot-separated, `domain.topic.method`. Domains: `geotech`, `structural`, `surveying`,
  `fluids`, `quantities`, `math`, `units`, `geometry`, `drawing`.
- **Any change to a result for the same input bumps the version.** Old versions stay callable.
- Pure additions (new optional input with a default that preserves old results, new output) do not bump the version.
- Removing a version requires a decision record and a deprecation period of at least one release.

## 5. Errors and warnings

**Errors** stop the calculation. Draft shape:

```json
{ "error": { "code": "UNIT_MISMATCH", "message": "Input 'area' expects dimension L^2, got L", "input": "area" } }
```

Draft codes: `UNKNOWN_CALCULATION`, `UNKNOWN_VERSION`, `MISSING_INPUT`, `UNKNOWN_INPUT`, `UNIT_MISMATCH`,
`UNKNOWN_UNIT`, `AMBIGUOUS_UNIT`, `OUT_OF_DOMAIN` (mathematically invalid, e.g. negative area),
`INVALID_JSON`, `INTERNAL`.

**Warnings** do not stop it. Each has a `code`, `message` and `level` (`info` | `caution`). Typical:
`OUTSIDE_VALIDITY_RANGE` (input outside the method's stated range), `ROUNDING_SENSITIVE`.

**TBD:** whether some validity breaches should be errors rather than warnings (per-calculation choice, recorded in the descriptor).

## 6. Trace

An ordered list of steps: symbolic expression, value, unit, and optionally a reference for that step. The trace
is what lets an engineer — or an AI client — explain a result without recomputing it.

## 7. Units

- **Internal representation:** value in SI + exponents over base dimensions
  (length, mass, time, temperature, amount, current, luminous intensity) **plus angle** as its own dimension.
- **Unit strings:** ASCII, `*` or `·` for products, `/` for division, `^` for powers: `kN/m^2`, `m^3/s`, `kN*m`.
- **Dimensionless inputs** omit `unit`. Angles are never dimensionless: `rad`, `deg`, `grad`, and DMS
  (format TBD, e.g. `"12°34'56\""` or `{ "d": 12, "m": 34, "s": 56 }`).
- **Ambiguous units are rejected** with `AMBIGUOUS_UNIT`: `ton` (use `t` for tonne, `ton_short`, `ton_long`),
  `gal` (use `gal_us` / `gal_imp`), bare `cal`.
- **Output units:** SI defaults per output; a request may ask for specific output units (field name TBD).
- **Temperature** (affine °C/°F) excluded from 0.1.

**Initial unit list (TBD — to be finalised in Phase 0):**

| Quantity | Units |
|---|---|
| Length | mm, cm, m, km, in, ft, yd, mi |
| Area | mm^2, m^2, ha, km^2, ft^2, acre |
| Volume | m^3, L, ft^3, yd^3 |
| Mass | g, kg, t |
| Force | N, kN, MN, kgf, lbf, kip |
| Pressure / stress | Pa, kPa, MPa, GPa, N/mm^2, kN/m^2, psi, ksi, bar |
| Unit weight | kN/m^3, lbf/ft^3 |
| Moment | N*m, kN*m, kip*ft |
| Flow | m^3/s (cumec), L/s, ft^3/s |
| Velocity | m/s, km/h, ft/s |
| Angle | rad, deg, grad, DMS |
| Time | s, min, h, day |

## 8. Python API (`bindings/python`)

- Generic: `civkit.call("fluids.open_channel.manning", n=0.013, area="2.4 m^2", ...)` returning a result object
  with `.outputs`, `.trace`, `.warnings`, `.to_dict()`.
- Generated typed functions per calculation, e.g. `civkit.fluids.open_channel.manning(...)`, with docstrings
  built from the descriptor.
- Discovery: `civkit.list()`, `civkit.describe(id)`.
- **TBD:** quantity input forms accepted (string `"2.4 m^2"`, tuple `(2.4, "m^2")`, `civkit.Q(2.4, "m^2")`).

## 9. FFI (`crates/civkit-ffi`)

- Functions: `civkit_call(const char* request_json) → char*`, `civkit_list() → char*`,
  `civkit_describe(const char* id) → char*`, `civkit_free(char*)`, `civkit_version() → const char*`.
- All strings UTF-8, NUL-terminated. Every returned `char*` is owned by CivKit and must be released with `civkit_free`.
- **No panic crosses the boundary**: panics are caught and returned as `INTERNAL` errors.
- Thread-safe: calls may be made from any thread.
- **TBD:** ABI versioning scheme (e.g. `civkit_abi_version()` checked by hosts at load time).

## 10. MCP (`integrations/mcp`)

- Transport: stdio only (see [OFFLINE_POLICY.md](OFFLINE_POLICY.md)).
- For 0.1, one MCP tool per calculation, generated from descriptors, all annotated read-only.
- Plus generic tools so the surface scales as calculations grow: `civkit_list`, `civkit_describe`, `civkit_call`.
  **TBD:** whether to switch to generic-only once the calculation count makes per-calculation tools unwieldy for clients.
- Tool results return the full contract response (outputs, trace, warnings, references) so the AI client
  explains CivKit's numbers instead of computing its own.
- Drawing tools accept file paths only within locations the user configures (see [SECURITY.md](SECURITY.md)).

## 11. Compatibility policy

- Rust crates follow semantic versioning; before 1.0, minor versions may break the Rust API.
- The **JSON contract** is more stable than the Rust API: calculation results are governed by §4 even pre-1.0.
- Every interface reports the CivKit version and the calculation version in its output.
