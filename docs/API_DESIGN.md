# API Design

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-25 00:29 IST (§3 example from README.md, 2026-11-24 23:41 IST) · last edited 2026-11-25 15:59 IST

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
| `inputs` | Name → dimension, internal SI unit (`si_unit`), default presentation unit (`display_unit`, for docs and forms; never applied silently to a bare number), required/optional, default, domain, `accepts_bearing`, description (§7.1.5) |
| `outputs` | Name → dimension, internal SI unit (`si_unit`), default presentation unit (`display_unit`), description (§7.1.5) |
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
- Outputs not listed are returned in the descriptor's default presentation unit (§7.1.5), which may differ from the internal SI unit.
- Naming an unknown output gives `UNKNOWN_OUTPUT`. A unit of the wrong dimension gives `UNIT_MISMATCH`.
- The trace always stays in the internal SI units, so that it reads the same way for everyone.

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

### 7.1 Unit notation, angle notation and unit roles (Phase 2A, `civkit-units`)

This section is the complete, formal definition of what the engine accepts in 0.1. **Users should never need
to discover a rule from an error message**: if a spelling is not listed here, it is not accepted.
Written 2026-11-25 by Claude Opus 5.5; reviewed by the maintainer the same day (decisions recorded in AI_HANDOVER.md).

#### 7.1.1 Unit expressions (the `unit` field)

Formal grammar (ISO/IEC 14977 EBNF). The whole string must match `unit`; nothing else is accepted.

```ebnf
unit         = percent | product , [ "/" , factor ] ;
percent      = "%" ;
product      = factor , { multiply , factor } ;
multiply     = "*" | "·" ;                   (* U+002A or U+00B7 MIDDLE DOT; equivalent *)
factor       = unit-name , [ "^" , exponent ] ;
exponent     = [ "-" ] , nonzero-digit ;
nonzero-digit = "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;
unit-name    = (* exactly one entry of table 7.1.2, matched case-sensitively *) ;
```

Meaning:
- A `product` multiplies its factors. `name^e` raises the unit to the integer power `e` (−9 … 9, never 0).
- **Division:** at most one `/`. It divides the product on its left by the **single factor** on its right.
  - There is no operator precedence to remember: `kN*m/s^2` means (kN·m)/s².
  - Anything with a second `/`, or with `*` after the `/`, is rejected.
  - The canonical spelling for more than one factor below the line uses negative powers: **`W*m^-2*K^-1`**,
    never `W/m^2*K`.
- `%` stands alone (= 0.01, dimensionless). It cannot be combined.

Not part of the grammar in 0.1 (rejected with `UNKNOWN_UNIT`): parentheses; whitespace anywhere in the string;
numeric factors (`1000*m`, `1e3m`); `**` or `.` as operators; implicit multiplication (`kN m`); Unicode
superscripts (`m²`, `s⁻¹`); SI prefixes on names not in table 7.1.2 (`mN`, `µm`, `Mm`); any name not in the table.
Supporting SI prefixes and common Unicode notation later is proposed as D-21 (not 0.1).

#### 7.1.2 Supported unit names (complete list, case-sensitive)

| Quantity | Names | Definition |
|---|---|---|
| Length | `mm` `cm` `m` `km` | SI |
| | `in` `ft` `yd` `mi` | 0.0254 m; 12 in; 3 ft; 5280 ft (international, 1959) |
| | `ft_us` | US survey foot, 1200/3937 m |
| Area | `ha` | 10 000 m² |
| | `acre` | 43 560 ft² (international) |
| Volume | `L` | 0.001 m³ (upper-case L only) |
| | `gal_us` `gal_imp` | 231 in³; 4.546 09 L |
| Mass | `g` `kg` `t` | SI (`t` = tonne, 1000 kg) |
| | `lbm` `ton_short` `ton_long` | 0.453 592 37 kg; 2000 lbm; 2240 lbm |
| Force | `N` `kN` `MN` | SI |
| | `kgf` `lbf` `kip` | kg × 9.806 65 m/s²; lbm × 9.806 65 m/s²; 1000 lbf |
| Pressure / stress | `Pa` `kPa` `MPa` `GPa` `bar` | SI; bar = 100 kPa |
| | `psi` `ksi` `psf` `ksf` | lbf/in²; 1000 psi; lbf/ft²; 1000 psf |
| Time | `s` `min` `h` `day` | 60 s; 60 min; 24 h |
| Angle | `rad` `deg` `grad` | π/180 rad; π/200 rad |
| Dimensionless | `%` | 0.01 |
| Alias | `cumec` | m³/s |

Rejected as ambiguous (`AMBIGUOUS_UNIT`), with the suggested replacement in the message:
- `ton` → `t`, `ton_short` or `ton_long`
- `gal` → `gal_us` or `gal_imp`
- `lb` → `lbm` or `lbf`
- `k` → `kip`
- `cal` → none (energy units are not in 0.1)

`dms` and `bearing` are not unit names. They mark structured angle values (7.1.3 and 7.1.4).

Tests check every factor through its defining relation (e.g. 1 lbf = 1 lbm × 9.806 65 m/s²), not by restating
the number. A deliberately wrong factor, even under 1 ppm, fails them.

#### 7.1.3 DMS angles (`"unit": "dms"`)

```ebnf
dms-value    = "[" , degrees , "," , minutes , "," , seconds , "]" ;  (* a JSON array of three numbers *)
negative     = "negative" , ":" , ( "true" | "false" ) ;              (* optional sibling of "value" *)
```

Canonical rules, so every angle has exactly one spelling:
- **degrees:** a whole number ≥ 0. There is no upper limit; angles are not normalised, and the input's domain applies.
- **minutes:** a whole number, 0 ≤ m ≤ 59. `[12, 60, 0]` is rejected; write `[13, 0, 0]`.
- **seconds:** a real number, 0 ≤ s < 60.

Negative angles:
- The sign is carried **only** by `"negative": true`. Signs inside the array, including −0, are rejected, so a
  negative degree with positive minutes cannot occur.
- `"negative": true` with a zero angle gives zero.
- `"negative"` with any unit other than `dms` is `INVALID_VALUE`.
- `dms` is accepted only for inputs whose dimension is angle (otherwise `UNIT_MISMATCH`).

#### 7.1.4 Quadrant bearings (`"unit": "bearing"`)

```ebnf
bearing      = ns , angle , ew ;          (* spaces may separate tokens (letters, numbers, marks), never split a number *)
ns           = "N" | "S" ;
ew           = "E" | "W" ;
angle        = whole , degree-mark , [ whole , minute-mark , [ number , second-mark ] ] ;
degree-mark  = "°" | "d" ;
minute-mark  = "'" | "′" | "m" ;          (* apostrophe or U+2032 PRIME *)
second-mark  = '"' | "″" | "s" ;          (* quotation mark or U+2033 DOUBLE PRIME *)
whole        = digit , { digit } ;
number       = whole , [ "." , digit , { digit } ] ;
```

Limits:
- degrees 0–90 (whole)
- minutes 0–59 (whole)
- seconds 0 ≤ s < 60
- **exactly 90° must have zero minutes and seconds**: `N 90°00'00" E` is valid; `N 90°00'00.001" E` is not.
- Seconds may appear only after minutes.

Letters and directions:
- Letters are **upper case only**. Lower-case `n 3° e` is **rejected, not normalised**, so that what is
  recorded is exactly what was entered.
- Valid direction combinations are N…E, N…W, S…E and S…W only. `E 3° N`, `NE 3° E` and similar are rejected.

Result:
- An azimuth clockwise from north, 0 ≤ azimuth < 2π:
  - N θ E = θ
  - S θ E = 180° − θ
  - S θ W = 180° + θ
  - N θ W = 360° − θ
- Each cardinal direction has two valid spellings giving the same azimuth: `N 0° E` = `N 0° W` = 0°; `N 90° E` = `S 90° E` = 90°; and so on.
  Tests cover all four quadrants and all eight cardinal spellings.

Where bearings are accepted:
- Only inputs whose descriptor sets `accepts_bearing` (which must be angles); otherwise `UNIT_MISMATCH`.
- Each conversion is a trace step: `<input>: bearing <text> → azimuth (clockwise from north)`, value in rad.

#### 7.1.5 The three unit roles

The contract keeps three units apart. They are often the same, but nothing assumes they are.

| Role | Where | Rule |
|---|---|---|
| **Internal calculation unit** | descriptor `si_unit` (inputs and outputs) | Canonical coherent SI **without prefix**, wherever applicable: `m`, `m^2`, `m^3/s`, `Pa`, `N`, `N*m`, `rad`. It is the unit `compute` works in, the unit domains and validity ranges are stated in, and the unit of every trace step. `kg` is accepted as the SI base unit of mass. Required for every dimensional input and output; checked at registration (coherent, i.e. factor 1). |
| **Default presentation unit** | descriptor `display_unit` | What the response uses when the request names no unit for that output, and what forms and docs show. May differ from `si_unit` (e.g. `mm` for a deflection, `kN` for a reaction). Must match the dimension. If absent, the response uses `si_unit`. |
| **User-requested output unit** | request `output_units` | Overrides the default presentation unit for that output. Must match the dimension. |

Request processing:
- Each input is converted from its given unit to the internal unit before domain checks, validity rules and `compute`.
- Each output is converted from the internal unit to its presentation unit (requested, else default, else internal).
  The response states the unit with every value.
- The trace always stays in internal units.
- Error mapping:

  | Condition | Code |
  |---|---|
  | Unknown name, grammar error, empty unit | `UNKNOWN_UNIT` |
  | Ambiguous name | `AMBIGUOUS_UNIT` |
  | Wrong dimension | `UNIT_MISMATCH` |
  | Malformed DMS or bearing | `INVALID_VALUE` |

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
