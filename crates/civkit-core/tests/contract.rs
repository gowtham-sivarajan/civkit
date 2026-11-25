// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 10:58 IST · last edited 2026-11-25 10:58 IST

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
                display_unit: Some("m".into()),
                required: true,
                default: None,
                domain: Range::at_least(0.0),
            }],
            outputs: vec![OutputSpec {
                name: "double".into(),
                description: "Twice the length".into(),
                dimension: Dimension::LENGTH,
                display_unit: Some("m".into()),
            }],
            references: vec!["Test".into()],
            assumptions: vec![],
            validity: vec![],
            examples: vec![],
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

#[test]
fn units_are_not_parsed_before_phase_2() {
    let mut r = Registry::new();
    r.register(Box::new(Doubler::new())).expect("valid");
    let v = call(
        &r,
        json!({ "id": "geometry.test.double_length", "inputs": { "length": { "value": 3, "unit": "m" } } }),
    );
    assert_eq!(single_error_code(&v), "UNKNOWN_UNIT");
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
    calc.0.inputs[0].dimension = Dimension::DIMENSIONLESS; // so the rule can run before civkit-units exists
    calc.0.inputs[0].display_unit = None;
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
    let inside = call(
        &r,
        json!({ "id": "geometry.test.double_length", "inputs": { "length": { "value": 4 } } }),
    );
    assert_eq!(inside["outputs"]["double"]["value"], 8.0);
    let outside = call(
        &r,
        json!({ "id": "geometry.test.double_length", "inputs": { "length": { "value": 20 } } }),
    );
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
