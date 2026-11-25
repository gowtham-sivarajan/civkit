# API Design

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-25 00:29 IST (§3 example from README.md, 2026-11-24 23:41 IST) · last edited 2026-11-25 10:03 IST

> **Status:** Phase 0 deliverable — no open TBDs. The nine former TBDs were resolved in task 0.7 and accepted by the
> maintainer on 2026-11-25 (D-19, [ADR 0007](../planning/decisions/0007-api-contract-details.md)); summary in §12.
> Related decisions: D-5 (units) and D-6 (FFI) — both accepted, ADR 0005; D-19 (contract details) — accepted, ADR 0007; D-13 (MCP SDK, proposed).

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
| `inputs` | Name → dimension, display unit, required/optional, allowed range, description. The display unit is for documentation and UIs only; it is never applied silently to a bare number (§7) |
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

**Resolved: omitting `version`.**
- A request **may omit** `version`; the engine then uses the latest version.
- The response **always** includes the resolved `version`, so the result can still be reproduced.
- Anything CivKit **saves** (exported results, quantity schedules, drawing annotations, later the desktop history)
  stores the explicit version. Loading a saved request that has no version is an error (`MISSING_INPUT` on `version`).
- Python and MCP accept an omitted version. Typed Python functions pin the version they were generated for.

**Resolved: output units.** A request may contain `"output_units": { "velocity": "ft/s" }`.
- Outputs not listed are returned in SI (the descriptor's display unit).
- Naming an unknown output gives `UNKNOWN_OUTPUT`. A unit of the wrong dimension gives `UNIT_MISMATCH`.
- The trace always stays in SI, so that it reads the same way for everyone.

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

Codes:
- Request problems: `UNKNOWN_CALCULATION`, `UNKNOWN_VERSION`, `MISSING_INPUT`, `UNKNOWN_INPUT`, `UNKNOWN_OUTPUT`, `INVALID_JSON`, `INVALID_VALUE`.
- Unit problems: `MISSING_UNIT` (a dimensional input given as a bare number), `UNIT_MISMATCH`, `UNKNOWN_UNIT`, `AMBIGUOUS_UNIT`.
- Calculation limits: `OUT_OF_DOMAIN` (mathematically invalid, e.g. a negative area), `OUTSIDE_VALIDITY_RANGE` (when escalated — see below).
- Other: `INTERNAL`.

`INVALID_VALUE` covers a malformed DMS, bearing or other structured value.
Several input errors may be reported at once as `{ "errors": [ … ] }`.

**Warnings** do not stop it. Each has a `code`, `message` and `level` (`info` | `caution`). Typical:
`OUTSIDE_VALIDITY_RANGE` (input outside the method's stated range), `ROUNDING_SENSITIVE`.

**Resolved: validity breaches.**
- A mathematically invalid input is **always** an error (`OUT_OF_DOMAIN`).
- An input outside a method's *stated* validity range is, by default, a **warning** (`OUTSIDE_VALIDITY_RANGE`, level `caution`).
- A calculation may escalate a specific rule to an **error** with the same code. It does this by setting
  `"on_breach": "error"` on that rule in its `validity` descriptor, with a one-line reason.
- Escalate only when the result outside the range is meaningless rather than merely less accurate.
- Every escalation is listed in the calculation's descriptor and documentation.

## 6. Trace

An ordered list of steps: symbolic expression, value, unit, and optionally a reference for that step. The trace
is what lets an engineer — or an AI client — explain a result without recomputing it.

## 7. Units

- **Internal representation:** value in SI + exponents over base dimensions
  (length, mass, time, temperature, amount, current, luminous intensity) **plus angle** as its own dimension.
- **Unit strings:** ASCII, `*` or `·` for products, `/` for division, `^` for powers: `kN/m^2`, `m^3/s`, `kN*m`.
- **Dimensionless inputs** omit `unit` (a plain ratio) or use `%`: `{ "value": 0.2, "unit": "%" }` = 0.002.
- **No silent default unit.** A dimensional input given without a unit is rejected with `MISSING_UNIT`.
  It is never assumed to be in the descriptor's display unit, because a wrong guess there is a silent error.
- **Ambiguous units are rejected** with `AMBIGUOUS_UNIT`:
  - `ton` → use `t` (tonne), `ton_short` or `ton_long`
  - `gal` → use `gal_us` or `gal_imp`
  - `lb` → use `lbm` (mass) or `lbf` (force)
  - bare `cal`
  - `k` → use `kip`
- **Feet:** `ft` is the international foot (0.3048 m exactly). The US survey foot (1200/3937 m) is `ft_us`; it is
  deprecated in the US since 2023 but still appears in older survey data. The two differ by 2 ppm, which matters only on long survey lines.
- **Output units:** see §3.
- **Temperature** (affine °C/°F) excluded from 0.1.

**Resolved: angle formats.** Angles are never dimensionless. A request may give an angle in any of these forms:
- **Decimal:** `{ "value": 12.5822, "unit": "deg" }`. Also `rad`, `grad`.
- **DMS (canonical JSON form):** `{ "value": [12, 34, 56.7], "unit": "dms" }`.
  - Degrees are an integer; minutes and seconds are in [0, 60).
  - A negative angle adds `"negative": true`. Signs inside the array are rejected, which removes the −0° ambiguity.
- **Quadrant bearing** (surveying inputs that accept it): `{ "value": "N 3°45'00\" E", "unit": "bearing" }`.
  - The ASCII form `"N 3d45m00s E"` is also accepted.
  - The engine converts it to an azimuth and records the conversion in the trace.
- Malformed values give `INVALID_VALUE`. The Python and MCP layers may also accept the strings `"12°34'56.7\""`
  and `"12d34m56.7s"` and convert them to the canonical form before calling the engine.

**Resolved: unit list for 0.1.**
- These are the **named units**. Compound units are built from them with `*`, `/` and `^`
  (e.g. `kN/m`, `kip/ft`, `in^4`, `kN*m`), so second moments of area, line loads and similar need no separate entries.
- Prefixes are **not** parsed generally (no `mN`, `Mm`). Only the names listed are recognised, which keeps the parser unambiguous.
- The list covers every unit used in reference cases C1–C9.

| Quantity | Named units |
|---|---|
| Length | mm, cm, m, km, in, ft, ft_us, yd, mi |
| Area | ha, acre (others are compounds: m^2, ft^2, mm^2, …) |
| Volume | L, gal_us, gal_imp (others are compounds: m^3, ft^3, yd^3) |
| Mass | g, kg, t, lbm |
| Force | N, kN, MN, kgf, lbf, kip |
| Pressure / stress | Pa, kPa, MPa, GPa, bar, psi, ksi, psf, ksf (compounds such as N/mm^2 also work) |
| Time | s, min, h, day |
| Angle | rad, deg, grad, dms (structured, above), bearing (structured, above) |
| Dimensionless | % (plus omitted unit = plain ratio) |
| Aliases | cumec = m^3/s |

Second moment of area (`mm^4`, `in^4`), line load (`kN/m`, `kip/ft`), unit weight (`kN/m^3`, `lbf/ft^3`),
moment (`kN*m`, `kip*ft`), velocity and flow all follow from the compound rule.

## 8. Python API (`bindings/python`)

- Generic: `civkit.call("fluids.open_channel.manning", n=0.013, area="2.4 m^2", ...)` returning a result object
  with `.outputs`, `.trace`, `.warnings`, `.to_dict()`.
- Generated typed functions per calculation, e.g. `civkit.fluids.open_channel.manning(...)`, with docstrings
  built from the descriptor.
- Discovery: `civkit.list()`, `civkit.describe(id)`.
- **Resolved: quantity input forms.** Python accepts all three:
  - a string: `"2.4 m^2"`
  - a tuple: `(2.4, "m^2")`
  - a quantity object: `civkit.Q(2.4, "m^2")`

  A plain Python number is accepted only for dimensionless inputs. For a dimensional input it raises `MISSING_UNIT` (§7).
  Result outputs are `civkit.Q` objects with `.value`, `.unit` and `.to("ft/s")`.
  Array (vectorised) inputs are out of scope for 0.1.

## 9. FFI (`crates/civkit-ffi`)

- Functions: `civkit_call(const char* request_json) → char*`, `civkit_list() → char*`,
  `civkit_describe(const char* id) → char*`, `civkit_free(char*)`, `civkit_version() → const char*`.
- All strings UTF-8, NUL-terminated. Every returned `char*` is owned by CivKit and must be released with `civkit_free`.
- **No panic crosses the boundary**: panics are caught and returned as `INTERNAL` errors.
- Thread-safe: calls may be made from any thread.
- **Resolved: ABI versioning.**
  - `uint32_t civkit_abi_version(void)` returns the ABI major number, starting at 1.
  - A host checks it once at load and refuses to run on a mismatch.
  - The number changes only when a function signature, a memory-ownership rule or the error envelope changes incompatibly.
  - Adding functions does not change it.
  - The ABI number is separate from `civkit_version()` (the CivKit semver string) and from calculation versions.

## 10. MCP (`integrations/mcp`)

- Transport: stdio only (see [OFFLINE_POLICY.md](OFFLINE_POLICY.md)).
- For 0.1, one MCP tool per calculation, generated from descriptors, all annotated read-only.
- Plus generic tools so the surface scales as calculations grow: `civkit_list`, `civkit_describe`, `civkit_call`.
  **Resolved: tool surface.**
  - A server option `--tools per-calc|generic|both` sets which tools are exposed. The default for 0.1 is `both`.
  - The default is revisited when the registry passes about 40 calculations, because large tool lists degrade some clients.
    At that point the default may switch to `generic`, with per-calculation tools left as an option.
- Tool results return the full contract response (outputs, trace, warnings, references) so the AI client
  explains CivKit's numbers instead of computing its own.
- Drawing tools accept file paths only within locations the user configures (see [SECURITY.md](SECURITY.md)).

## 11. Compatibility policy

- Rust crates follow semantic versioning; before 1.0, minor versions may break the Rust API.
- The **JSON contract** is more stable than the Rust API: calculation results are governed by §4 even pre-1.0.
- Every interface reports the CivKit version and the calculation version in its output.

## 12. Resolved TBDs — summary (task 0.7, accepted 2026-11-25)

| # | Topic | Resolution | Section |
|---|---|---|---|
| 1 | Omitting `version` | Optional in requests (latest); always echoed in the response; required in anything saved | §3 |
| 2 | Output units | `output_units` map per output; trace stays SI | §3 |
| 3 | Validity breaches | Warning by default; per-rule escalation to error with a stated reason; math-invalid is always an error | §5 |
| 4 | Bare numbers | Dimensional input without a unit → `MISSING_UNIT`; no silent default unit | §7 |
| 5 | Angle formats | Decimal deg/rad/grad; DMS as `[d, m, s]` + `negative`; quadrant bearings as a structured value | §7 |
| 6 | Unit list | Named units + compounds; no general prefix parsing; `lb` and `k` ambiguous; `ft` vs `ft_us` | §7 |
| 7 | Python quantities | String, tuple or `civkit.Q`; bare numbers only for dimensionless | §8 |
| 8 | FFI ABI | `civkit_abi_version()` integer, checked at load | §9 |
| 9 | MCP tool surface | `--tools` option, default `both`; revisit at ~40 calculations | §10 |
