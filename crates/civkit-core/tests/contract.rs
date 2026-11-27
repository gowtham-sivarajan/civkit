// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 10:58 IST · last edited 2026-11-27 11:21 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! Tests of the JSON contract itself: envelopes, error codes and descriptor checks (`docs/API_DESIGN.md`).

use civkit_core::{
    Breach, CalcError, Calculation, Context, Descriptor, InputSpec, Inputs, OutputSpec, Outputs,
    Range, Registry, ValidityRule, calcs::demo::WeightedMean, check_descriptor,
};
use civkit_units::Dimension;
use serde_json::{Value, json};

fn demo_registry() -> Registry {
    let mut r = Registry::new();
    r.register(Box::new(WeightedMean)).expect("valid");
    r
}

fn call(r: &Registry, request: Value) -> Value {
    serde_json::from_str(&r.call_json(&request.to_string())).expect("call_json always returns JSON")
}

fn single_error_code(v: &Value) -> &str {
    v["error"]["code"]
        .as_str()
        .unwrap_or_else(|| panic!("expected one error, got {v}"))
}

#[test]
fn success_response_has_every_contract_field() {
    let v = call(
        &demo_registry(),
        json!({ "id": "math.demo.weighted_mean", "inputs": { "a": { "value": 1 }, "b": { "value": 3 } } }),
    );
    for field in [
        "id",
        "version",
        "civkit_version",
        "outputs",
        "trace",
        "warnings",
        "references",
        "assumptions",
    ] {
        assert!(v.get(field).is_some(), "missing '{field}' in {v}");
    }
    assert_eq!(
        v["version"], 1,
        "an omitted version resolves to the latest and is echoed"
    );
    assert_eq!(v["outputs"]["mean"]["value"], 2.0);
}

#[test]
fn unknown_calculation_and_version() {
    let r = demo_registry();
    assert_eq!(
        single_error_code(&call(
            &r,
            json!({ "id": "math.demo.nothing", "inputs": {} })
        )),
        "UNKNOWN_CALCULATION"
    );
    assert_eq!(
        single_error_code(&call(
            &r,
            json!({ "id": "math.demo.weighted_mean", "version": 9, "inputs": {} })
        )),
        "UNKNOWN_VERSION"
    );
}

#[test]
fn malformed_json_and_unknown_fields_are_invalid_json() {
    let r = demo_registry();
    let v: Value = serde_json::from_str(&r.call_json("{ not json")).expect("JSON");
    assert_eq!(single_error_code(&v), "INVALID_JSON");
    let v = call(
        &r,
        json!({ "id": "math.demo.weighted_mean", "inputs": {}, "colour": "red" }),
    );
    assert_eq!(single_error_code(&v), "INVALID_JSON");
}

#[test]
fn several_input_errors_come_back_together() {
    let v = call(
        &demo_registry(),
        json!({ "id": "math.demo.weighted_mean", "inputs": { "a": { "value": "two" }, "c": { "value": 1 } } }),
    );
    let codes: Vec<&str> = v["errors"]
        .as_array()
        .expect("errors array")
        .iter()
        .filter_map(|e| e["code"].as_str())
        .collect();
    assert_eq!(codes, ["UNKNOWN_INPUT", "INVALID_VALUE", "MISSING_INPUT"]);
}

#[test]
fn unknown_output_in_output_units() {
    let v = call(
        &demo_registry(),
        json!({ "id": "math.demo.weighted_mean", "inputs": { "a": { "value": 1 }, "b": { "value": 1 } }, "output_units": { "median": "m" } }),
    );
    assert_eq!(single_error_code(&v), "UNKNOWN_OUTPUT");
}

/// A dimensional calculation, to test unit rules before civkit-units exists.
struct Doubler(Descriptor);

impl Doubler {
    fn new() -> Self {
        Self(Descriptor {
            id: "geometry.test.double_length".into(),
            version: 1,
            title: "Double a length".into(),
            summary: "Test only".into(),
            inputs: vec![InputSpec {
                name: "length".into(),
                description: "A length".into(),
                dimension: Dimension::LENGTH,
                si_unit: Some("m".into()),
                display_unit: Some("m".into()),
                required: true,
                default: None,
                accepts_bearing: false,
                domain: Range::at_least(0.0),
                ..Default::default()
            }],
            outputs: vec![OutputSpec {
                name: "double".into(),
                description: "Twice the length".into(),
                dimension: Dimension::LENGTH,
                si_unit: Some("m".into()),
                display_unit: Some("m".into()),
                ..Default::default()
            }],
            references: vec!["Test".into()],
            assumptions: vec![],
            validity: vec![],
            examples: vec![],
            ..Default::default()
        })
    }
}

impl Calculation for Doubler {
    fn descriptor(&self) -> &Descriptor {
        &self.0
    }
    fn compute(&self, inputs: &Inputs, ctx: &mut Context) -> Result<Outputs, CalcError> {
        let l = inputs.require("length")?;
        ctx.step("2L", 2.0 * l, Some("m"));
        Ok(Outputs::new().with("double", 2.0 * l))
    }
}

#[test]
fn dimensional_input_without_unit_is_missing_unit() {
    let mut r = Registry::new();
    r.register(Box::new(Doubler::new())).expect("valid");
    let v = call(
        &r,
        json!({ "id": "geometry.test.double_length", "inputs": { "length": { "value": 3 } } }),
    );
    assert_eq!(
        single_error_code(&v),
        "MISSING_UNIT",
        "CivKit never assumes a unit (D-19)"
    );
}

fn doubler() -> Registry {
    let mut r = Registry::new();
    r.register(Box::new(Doubler::new())).expect("valid");
    r
}

fn length(value: Value, unit: &str) -> Value {
    json!({ "id": "geometry.test.double_length", "inputs": { "length": { "value": value, "unit": unit } } })
}

#[test]
fn units_convert_to_si_before_compute() {
    let r = doubler();
    // 3 ft = 0.9144 m; doubled = 1.8288 m, reported in SI
    let v = call(&r, length(json!(3), "ft"));
    assert!(
        (v["outputs"]["double"]["value"].as_f64().expect("number") - 1.8288).abs() < 1e-12,
        "{v}"
    );
    assert_eq!(v["outputs"]["double"]["unit"], "m");
    // the trace is in SI
    assert!((v["trace"][0]["value"].as_f64().expect("number") - 1.8288).abs() < 1e-12);
    assert_eq!(v["trace"][0]["unit"], "m");
}

#[test]
fn unit_errors_map_to_contract_codes() {
    let r = doubler();
    assert_eq!(
        single_error_code(&call(&r, length(json!(3), "kN"))),
        "UNIT_MISMATCH"
    );
    assert_eq!(
        single_error_code(&call(&r, length(json!(3), "Mm"))),
        "UNKNOWN_UNIT"
    );
    assert_eq!(
        single_error_code(&call(&r, length(json!(3), "m^2^2"))),
        "UNKNOWN_UNIT"
    );
    assert_eq!(
        single_error_code(&call(&r, length(json!(3), "lb"))),
        "AMBIGUOUS_UNIT"
    );
    assert_eq!(
        single_error_code(&call(&r, length(json!(3), "deg"))),
        "UNIT_MISMATCH"
    );
    assert_eq!(
        single_error_code(&call(&r, length(json!(3), "dms"))),
        "UNIT_MISMATCH"
    );
    assert_eq!(
        single_error_code(&call(&r, length(json!("N 3° E"), "bearing"))),
        "UNIT_MISMATCH"
    );
    let v = call(
        &r,
        json!({ "id": "geometry.test.double_length", "inputs": { "length": { "value": 3, "unit": "m", "negative": true } } }),
    );
    assert_eq!(single_error_code(&v), "INVALID_VALUE");
}

#[test]
fn domain_is_checked_after_conversion() {
    let v = call(&doubler(), length(json!(-1), "mm"));
    assert_eq!(single_error_code(&v), "OUT_OF_DOMAIN");
}

#[test]
fn output_units_are_honoured() {
    let r = doubler();
    let v = call(
        &r,
        json!({ "id": "geometry.test.double_length", "inputs": { "length": { "value": 1, "unit": "m" } }, "output_units": { "double": "ft" } }),
    );
    let ft = v["outputs"]["double"]["value"].as_f64().expect("number");
    assert!((ft - 2.0 / 0.3048).abs() < 1e-12, "{v}");
    assert_eq!(v["outputs"]["double"]["unit"], "ft");
    assert_eq!(v["trace"][0]["unit"], "m", "the trace stays SI");
    let bad = call(
        &r,
        json!({ "id": "geometry.test.double_length", "inputs": { "length": { "value": 1, "unit": "m" } }, "output_units": { "double": "kg" } }),
    );
    assert_eq!(single_error_code(&bad), "UNIT_MISMATCH");
}

/// An angle calculation, to test DMS and bearings.
struct Azimuth(Descriptor);

impl Calculation for Azimuth {
    fn descriptor(&self) -> &Descriptor {
        &self.0
    }
    fn compute(&self, inputs: &Inputs, _: &mut Context) -> Result<Outputs, CalcError> {
        Ok(Outputs::new().with("angle_out", inputs.require("angle")?))
    }
}

fn azimuth_registry(accepts_bearing: bool) -> Registry {
    let mut d = Doubler::new().0;
    d.id = "surveying.test.echo_angle".into();
    d.inputs[0].name = "angle".into();
    d.inputs[0].dimension = Dimension::ANGLE;
    d.inputs[0].si_unit = Some("rad".into());
    d.inputs[0].display_unit = Some("deg".into());
    d.inputs[0].domain = Range::UNBOUNDED;
    d.inputs[0].accepts_bearing = accepts_bearing;
    d.outputs[0].name = "angle_out".into();
    d.outputs[0].dimension = Dimension::ANGLE;
    d.outputs[0].si_unit = Some("rad".into());
    d.outputs[0].display_unit = None;
    let mut r = Registry::new();
    r.register(Box::new(Azimuth(d))).expect("valid");
    r
}

fn angle_out(v: &Value) -> f64 {
    v["outputs"]["angle_out"]["value"]
        .as_f64()
        .unwrap_or_else(|| panic!("no output in {v}"))
}

#[test]
fn dms_and_bearing_inputs() {
    let r = azimuth_registry(true);
    let dms = call(
        &r,
        json!({ "id": "surveying.test.echo_angle", "inputs": { "angle": { "value": [12, 30, 0], "unit": "dms", "negative": true } } }),
    );
    assert!((angle_out(&dms) + 12.5_f64.to_radians()).abs() < 1e-12);
    let bad = call(
        &r,
        json!({ "id": "surveying.test.echo_angle", "inputs": { "angle": { "value": [-12, 30, 0], "unit": "dms" } } }),
    );
    assert_eq!(single_error_code(&bad), "INVALID_VALUE");

    let b = call(
        &r,
        json!({ "id": "surveying.test.echo_angle", "inputs": { "angle": { "value": "S 32°40' W", "unit": "bearing" } } }),
    );
    let expected = (180.0 + 32.0 + 40.0 / 60.0_f64).to_radians();
    assert!((angle_out(&b) - expected).abs() < 1e-12);
    assert!(
        b["trace"][0]["step"]
            .as_str()
            .expect("step")
            .contains("S 32°40' W"),
        "the conversion is in the trace: {b}"
    );
    let deg = call(
        &r,
        json!({ "id": "surveying.test.echo_angle", "inputs": { "angle": { "value": 90, "unit": "deg" } } }),
    );
    assert!((angle_out(&deg) - std::f64::consts::FRAC_PI_2).abs() < 1e-12);

    let not_accepted = azimuth_registry(false);
    let v = call(
        &not_accepted,
        json!({ "id": "surveying.test.echo_angle", "inputs": { "angle": { "value": "N 3° E", "unit": "bearing" } } }),
    );
    assert_eq!(single_error_code(&v), "UNIT_MISMATCH");
}

#[test]
fn descriptor_units_are_checked() {
    let mut d = Doubler::new().0;
    d.outputs[0].si_unit = Some("mm".into());
    assert!(
        check_descriptor(&d).is_err(),
        "internal units must be coherent SI without prefix"
    );
    let mut d = Doubler::new().0;
    d.outputs[0].si_unit = None;
    assert!(
        check_descriptor(&d).is_err(),
        "dimensional outputs need an internal SI unit"
    );
    let mut d = Doubler::new().0;
    d.inputs[0].display_unit = Some("kN".into());
    assert!(
        check_descriptor(&d).is_err(),
        "display unit must match the dimension"
    );
    let mut d = Doubler::new().0;
    d.inputs[0].accepts_bearing = true;
    assert!(check_descriptor(&d).is_err(), "only angles accept bearings");
    let mut d = Doubler::new().0;
    d.outputs[0].display_unit = Some("mm".into());
    assert!(
        check_descriptor(&d).is_ok(),
        "a prefixed presentation unit is fine"
    );
}

#[test]
fn three_unit_roles_are_distinct() {
    // internal unit m, default presentation mm, user request ft
    let mut d = Doubler::new().0;
    d.outputs[0].display_unit = Some("mm".into());
    let mut r = Registry::new();
    r.register(Box::new(Doubler(d))).expect("valid");
    let v = call(&r, length(json!(1), "m"));
    assert_eq!(
        v["outputs"]["double"]["unit"], "mm",
        "default presentation unit"
    );
    assert!((v["outputs"]["double"]["value"].as_f64().expect("number") - 2000.0).abs() < 1e-9);
    assert_eq!(v["trace"][0]["unit"], "m", "trace in the internal SI unit");
    assert!((v["trace"][0]["value"].as_f64().expect("number") - 2.0).abs() < 1e-12);
    let asked = call(
        &r,
        json!({ "id": "geometry.test.double_length", "inputs": { "length": { "value": 1, "unit": "m" } }, "output_units": { "double": "ft" } }),
    );
    assert_eq!(
        asked["outputs"]["double"]["unit"], "ft",
        "user-requested unit wins"
    );
}

#[test]
fn panicking_calculation_becomes_internal_error() {
    struct Panics(Descriptor);
    impl Calculation for Panics {
        fn descriptor(&self) -> &Descriptor {
            &self.0
        }
        fn compute(&self, _: &Inputs, _: &mut Context) -> Result<Outputs, CalcError> {
            panic!("deliberate test panic")
        }
    }
    let mut d = Doubler::new().0;
    d.id = "geometry.test.panics".into();
    d.inputs.clear();
    let mut r = Registry::new();
    r.register(Box::new(Panics(d))).expect("valid");
    let v = call(&r, json!({ "id": "geometry.test.panics" }));
    assert_eq!(single_error_code(&v), "INTERNAL");
}

#[test]
fn escalated_validity_rule_is_an_error_and_needs_a_reason() {
    let mut calc = Doubler::new();
    calc.0.validity.push(ValidityRule {
        input: "length".into(),
        range: Range::closed(0.0, 10.0),
        on_breach: Breach::Error,
        reason: "test limit".into(),
    });
    let mut no_reason = calc.0.clone();
    no_reason.validity[0].reason.clear();
    assert!(
        check_descriptor(&no_reason).is_err(),
        "an escalated rule needs a reason"
    );

    let mut r = Registry::new();
    r.register(Box::new(calc)).expect("valid");
    let inside = call(&r, length(json!(4), "m"));
    assert_eq!(inside["outputs"]["double"]["value"], 8.0);
    let outside = call(&r, length(json!(20), "m"));
    assert_eq!(single_error_code(&outside), "OUTSIDE_VALIDITY_RANGE");
}

#[test]
fn malformed_descriptors_are_rejected() {
    let base = Doubler::new().0;
    let bad_ids = [
        "Geometry.test.x",
        "geometry.test",
        "geometry.test.x.y",
        "bridges.test.x",
        "geometry.te-st.x",
    ];
    for id in bad_ids {
        let mut d = base.clone();
        d.id = id.into();
        assert!(check_descriptor(&d).is_err(), "'{id}' should be rejected");
    }
    let mut d = base.clone();
    d.version = 0;
    assert!(check_descriptor(&d).is_err(), "version 0");
    let mut d = base.clone();
    d.references.clear();
    assert!(check_descriptor(&d).is_err(), "no references");
    let mut d = base.clone();
    d.outputs[0].name = "length".into();
    assert!(check_descriptor(&d).is_err(), "duplicate name");
}

#[test]
fn duplicate_registration_is_rejected() {
    let mut r = demo_registry();
    assert!(r.register(Box::new(WeightedMean)).is_err());
}

#[test]
fn builtins_are_valid() {
    // Panics if any built-in descriptor is malformed.
    let r = Registry::with_builtins();
    assert!(
        r.list().iter().all(|d| !d.id.starts_with("math.demo.")),
        "the demo must not be a built-in"
    );
}

/// Writes its trace step in millimetres: a calculation bug the registry must catch.
struct TraceInMillimetres(Descriptor);

impl Calculation for TraceInMillimetres {
    fn descriptor(&self) -> &Descriptor {
        &self.0
    }
    fn compute(&self, inputs: &Inputs, ctx: &mut Context) -> Result<Outputs, CalcError> {
        let l = inputs.require("length")?;
        ctx.step("2L", 2000.0 * l, Some("mm"));
        Ok(Outputs::new().with("double", 2.0 * l))
    }
}

#[test]
fn trace_steps_must_use_internal_units() {
    let mut r = Registry::new();
    r.register(Box::new(TraceInMillimetres(Doubler::new().0)))
        .expect("valid");
    let v = call(&r, length(json!(1), "m"));
    assert_eq!(single_error_code(&v), "INTERNAL", "{v}");
}

#[test]
fn messages_name_the_internal_unit() {
    let mut calc = Doubler::new();
    calc.0.validity.push(ValidityRule {
        input: "length".into(),
        range: Range::closed(0.0, 10.0),
        on_breach: Breach::Warning,
        reason: "test limit".into(),
    });
    let mut r = Registry::new();
    r.register(Box::new(calc)).expect("valid");
    // 20 ft = 6.096 m is inside; 50 ft = 15.24 m is outside, reported in metres
    let inside = call(&r, length(json!(20), "ft"));
    assert_eq!(inside["warnings"].as_array().map(Vec::len), Some(0));
    let v = call(&r, length(json!(50), "ft"));
    let msg = v["warnings"][0]["message"].as_str().expect("a warning");
    assert!(
        msg.contains("15.24") && msg.contains(" m ") && msg.contains("[0, 10] m"),
        "{msg}"
    );
    let domain = call(&doubler(), length(json!(-2), "ft"));
    let msg = domain["error"]["message"].as_str().expect("an error");
    assert!(
        msg.contains("-0.6096 m") && msg.contains("[0, ∞) m"),
        "{msg}"
    );
}

/// ADR 0013 gate G2: a calculation ID is also an MCP tool name, so it is at most 128 characters from
/// `A–Z a–z 0–9 _ - .`. CivKit's own ID rule is a subset of that set; the length limit is the new part.
#[test]
fn ids_fit_mcp_tool_names() {
    let with_method_len = |n: usize| {
        let mut d = Doubler::new().0;
        d.id = format!("geometry.test.{}", "m".repeat(n));
        d
    };
    let prefix = "geometry.test.".len();
    let longest = with_method_len(128 - prefix);
    assert_eq!(longest.id.len(), 128);
    assert!(
        check_descriptor(&longest).is_ok(),
        "128 characters is allowed"
    );
    let too_long = with_method_len(129 - prefix);
    let message = check_descriptor(&too_long)
        .expect_err("129 characters")
        .to_string();
    assert!(
        message.contains("128"),
        "the error names the limit: {message}"
    );
    for id in [
        "geometry.test.Double",
        "geometry.test.double-length",
        "geometry.test.dö",
    ] {
        let mut d = Doubler::new().0;
        d.id = id.into();
        assert!(check_descriptor(&d).is_err(), "{id}");
    }
    for d in Registry::with_builtins().list() {
        assert!(
            d.id.len() <= 128
                && d.id
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "_-.".contains(c)),
            "{}",
            d.id
        );
    }
}

/// `version` and `output_units` are control arguments of the Python `call` and of the MCP per-calculation tools,
/// so no calculation may name an input that way.
#[test]
fn reserved_input_names_are_refused() {
    for name in ["version", "output_units"] {
        let mut d = Doubler::new().0;
        d.inputs[0].name = name.into();
        let message = check_descriptor(&d).expect_err("reserved").to_string();
        assert!(message.contains("reserved"), "{name}: {message}");
    }
}

/// Numbers in requests and responses keep every bit: serde_json must parse floats correctly rounded (its
/// `float_roundtrip` feature). Without it, `1.8276944379471094` parses one ulp off, so an interface that re-reads
/// `call_json`'s output (the MCP server) changed the last digit, and request inputs could arrive one ulp off.
/// Found by comparing MCP Inspector's results with the Python package's.
#[test]
fn json_numbers_are_parsed_exactly() {
    for text in [
        "1.8276944379471094",
        "0.013",
        "0.002",
        "0.3048",
        "2.2250738585072014e-308",
        "1.7976931348623157e308",
        "5e-324",
    ] {
        let via_json: f64 = serde_json::from_str(text).expect("number");
        let exact: f64 = text.parse().expect("number"); // the standard library rounds correctly
        assert_eq!(via_json.to_bits(), exact.to_bits(), "{text}");
    }
    // Every f64 written by the engine reads back to the same bits.
    let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
    for _ in 0..20_000 {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        let v = f64::from_bits(x);
        if !v.is_finite() {
            continue;
        }
        let text = serde_json::to_string(&v).expect("finite");
        let back: f64 = serde_json::from_str(&text).expect("number");
        assert_eq!(back.to_bits(), v.to_bits(), "{text}");
    }
}

/// Descriptor text is shown as written by every interface (docstrings, MCP tool descriptions, generated docs), so it
/// has no line breaks or doubled spaces. A broken Rust line continuation (`\n` and indentation inside a string)
/// slipped into two descriptors before this test existed.
#[test]
fn descriptor_text_is_one_clean_line() {
    fn walk(v: &Value, path: &str, problems: &mut Vec<String>) {
        match v {
            Value::String(s) if s.contains('\n') || s.contains("  ") => {
                problems.push(format!("{path}: {s:?}"));
            }
            Value::Array(items) => {
                for (i, x) in items.iter().enumerate() {
                    walk(x, &format!("{path}[{i}]"), problems);
                }
            }
            Value::Object(map) => {
                for (k, x) in map {
                    walk(x, &format!("{path}.{k}"), problems);
                }
            }
            _ => {}
        }
    }
    let mut problems = Vec::new();
    for d in Registry::with_builtins().list() {
        let value = serde_json::to_value(d).expect("descriptor");
        walk(&value, &d.id, &mut problems);
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
