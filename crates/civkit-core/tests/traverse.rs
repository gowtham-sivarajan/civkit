// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-25 23:56 IST · last edited 2026-11-26 00:23 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! `surveying.traverse.compass_rule` through the public contract (D-20, planning/TRAVERSE_DESCRIPTOR_PROPOSAL.md §8).
//!
//! The reference case C3 lives in `tests/reference/`; the exact adjustment arithmetic is tested next to the code
//! (`src/calcs/surveying.rs`). These tests cover closed forms, units, errors, the trace and the properties.

use civkit_core::{ErrorCode, Registry, Request, Response};
use proptest::prelude::*;
use serde_json::json;

const ID: &str = "surveying.traverse.compass_rule";

fn request(inputs: serde_json::Value) -> Request {
    serde_json::from_value(json!({ "id": ID, "inputs": inputs })).expect("request parses")
}

fn call(inputs: serde_json::Value) -> Result<Response, civkit_core::ErrorEnvelope> {
    Registry::with_builtins().call(&request(inputs))
}

fn list(r: &Response, name: &str) -> Vec<f64> {
    r.outputs[name]
        .value
        .as_list()
        .unwrap_or_else(|| panic!("'{name}' is a list"))
        .to_vec()
}

fn scalar(r: &Response, name: &str) -> f64 {
    r.outputs[name]
        .value
        .as_scalar()
        .unwrap_or_else(|| panic!("'{name}' is a scalar"))
}

fn close(got: f64, expected: f64, tol: f64, what: &str) {
    assert!(
        (got - expected).abs() <= tol,
        "{what}: got {got}, expected {expected} (tolerance {tol})"
    );
}

fn close_all(got: &[f64], expected: &[f64], tol: f64, what: &str) {
    assert_eq!(got.len(), expected.len(), "{what}: length");
    for (k, (g, e)) in got.iter().zip(expected).enumerate() {
        close(*g, *e, tol, &format!("{what}[{k}]"));
    }
}

fn degrees(values: &[f64]) -> serde_json::Value {
    json!({ "value": values, "unit": "deg" })
}

fn metres(values: &[f64]) -> serde_json::Value {
    json!({ "value": values, "unit": "m" })
}

fn square() -> serde_json::Value {
    json!({
        "bearings": degrees(&[0.0, 90.0, 180.0, 270.0]),
        "distances": metres(&[100.0, 100.0, 100.0, 100.0]),
    })
}

fn err_code(inputs: serde_json::Value) -> (ErrorCode, Option<String>, Option<usize>, String) {
    let e = call(inputs).expect_err("should fail").0;
    (
        e[0].code,
        e[0].input.clone(),
        e[0].index,
        e[0].message.clone(),
    )
}

// --- Registration ------------------------------------------------------------------------------------

#[test]
fn the_calculation_is_registered_with_the_accepted_descriptor() {
    let registry = Registry::with_builtins();
    let d = registry.describe(ID, None).expect("registered");
    assert_eq!(d.version, 1);
    assert_eq!(d.groups.len(), 1);
    assert_eq!(
        (d.groups[0].name.as_str(), d.groups[0].item.as_str()),
        ("legs", "leg")
    );
    assert_eq!(d.groups[0].min_len, 3);
    let inputs: Vec<_> = d.inputs.iter().map(|i| i.name.as_str()).collect();
    assert_eq!(inputs, ["bearings", "distances"]);
    let outputs: Vec<_> = d.outputs.iter().map(|o| o.name.as_str()).collect();
    assert_eq!(
        outputs,
        [
            "latitudes",
            "departures",
            "latitude_corrections",
            "departure_corrections",
            "adjusted_latitudes",
            "adjusted_departures",
            "perimeter",
            "latitude_misclosure",
            "departure_misclosure",
            "linear_misclosure",
            "relative_misclosure",
        ]
    );
    assert!(d.validity.is_empty(), "no validity rules in 0.1 (Q-C)");
    assert!(
        d.inputs.iter().all(|i| i.choices.is_empty()),
        "no choice input (Q-G)"
    );
}

// --- Closed forms --------------------------------------------------------------------------------------

#[test]
fn a_square_traverse_closes() {
    let r = call(square()).expect("square");
    close_all(
        &list(&r, "latitudes"),
        &[100.0, 0.0, -100.0, 0.0],
        1e-12,
        "latitudes",
    );
    close_all(
        &list(&r, "departures"),
        &[0.0, 100.0, 0.0, -100.0],
        1e-12,
        "departures",
    );
    close(scalar(&r, "perimeter"), 400.0, 1e-12, "perimeter");
    close(
        scalar(&r, "linear_misclosure"),
        0.0,
        1e-12,
        "linear misclosure",
    );
    close(
        scalar(&r, "relative_misclosure"),
        0.0,
        1e-14,
        "relative misclosure",
    );
    close_all(
        &list(&r, "latitude_corrections"),
        &[0.0; 4],
        1e-12,
        "corrections",
    );
    close_all(
        &list(&r, "adjusted_latitudes"),
        &list(&r, "latitudes"),
        1e-12,
        "adjusted",
    );
    assert!(r.warnings.is_empty());
}

#[test]
fn a_rectangle_with_one_leg_short_is_adjusted_in_proportion_to_length() {
    // North 100, east 50, south 99, west 50: the traverse falls 1 m short of closing, due north.
    let r = call(json!({
        "bearings": degrees(&[0.0, 90.0, 180.0, 270.0]),
        "distances": metres(&[100.0, 50.0, 99.0, 50.0]),
    }))
    .expect("rectangle");
    let p = 299.0;
    close(scalar(&r, "perimeter"), p, 1e-12, "perimeter");
    close(scalar(&r, "latitude_misclosure"), 1.0, 1e-10, "M_N");
    close(scalar(&r, "departure_misclosure"), 0.0, 1e-10, "M_E");
    close(scalar(&r, "linear_misclosure"), 1.0, 1e-10, "E");
    close(scalar(&r, "relative_misclosure"), 1.0 / p, 1e-12, "E / P");
    // c_N,k = -M_N d_k / P, so the corrections are -d_k / 299, and the adjusted latitudes sum to zero.
    let d = [100.0, 50.0, 99.0, 50.0];
    let c: Vec<f64> = d.iter().map(|d| -d / p).collect();
    close_all(
        &list(&r, "latitude_corrections"),
        &c,
        1e-10,
        "latitude corrections",
    );
    close_all(
        &list(&r, "departure_corrections"),
        &[0.0; 4],
        1e-10,
        "departure corrections",
    );
    let adjusted = list(&r, "adjusted_latitudes");
    close(
        adjusted.iter().sum::<f64>(),
        0.0,
        1e-10,
        "sum of adjusted latitudes",
    );
    close(adjusted[0], 100.0 - 100.0 / p, 1e-10, "leg 1 adjusted");
}

#[test]
fn the_sign_of_a_correction_opposes_the_misclosure() {
    // The same shape, 1 m too long to the south: M_N = -1, so every latitude correction is positive.
    let r = call(json!({
        "bearings": degrees(&[0.0, 90.0, 180.0, 270.0]),
        "distances": metres(&[100.0, 50.0, 101.0, 50.0]),
    }))
    .expect("long south leg");
    close(scalar(&r, "latitude_misclosure"), -1.0, 1e-10, "M_N");
    assert!(list(&r, "latitude_corrections").iter().all(|c| *c > 0.0));
}

#[test]
fn latitude_and_departure_are_not_swapped() {
    // One leg due east: all departure, no latitude (leg 2 of the square).
    let r = call(square()).expect("square");
    assert!(list(&r, "departures")[1] > 99.0);
    assert!(list(&r, "latitudes")[1].abs() < 1e-12);
}

// --- Units and bearings ------------------------------------------------------------------------------

#[test]
fn units_and_bearing_notation_do_not_change_the_result() {
    let metric = call(json!({
        "bearings": degrees(&[30.0, 135.0, 285.0]),
        "distances": metres(&[30.48, 60.96, 45.72]),
    }))
    .expect("degrees and metres");
    let other = call(json!({
        "bearings": { "value": ["N 30°00'00\" E", "S 45°00'00\" E", "N 75°00'00\" W"], "unit": "bearing" },
        "distances": { "value": [100.0, 200.0, 150.0], "unit": "ft" },
    }))
    .expect("bearings and feet");
    for name in [
        "latitudes",
        "departures",
        "latitude_corrections",
        "departure_corrections",
        "adjusted_latitudes",
        "adjusted_departures",
    ] {
        close_all(&list(&other, name), &list(&metric, name), 1e-9, name);
    }
    for name in ["perimeter", "linear_misclosure", "relative_misclosure"] {
        close(scalar(&other, name), scalar(&metric, name), 1e-9, name);
    }
}

#[test]
fn outputs_can_be_requested_in_feet_and_are_converted_for_every_entry() {
    let mut req = request(square());
    for name in ["latitudes", "perimeter"] {
        req.output_units.insert(name.into(), "ft".into());
    }
    let r = Registry::with_builtins().call(&req).expect("feet out");
    close(
        scalar(&r, "perimeter"),
        400.0 / 0.3048,
        1e-9,
        "perimeter in ft",
    );
    close(
        list(&r, "latitudes")[0],
        100.0 / 0.3048,
        1e-9,
        "latitude in ft",
    );
    assert_eq!(r.outputs["latitudes"].unit.as_deref(), Some("ft"));
    // The trace stays in metres.
    assert!(
        r.trace
            .iter()
            .filter_map(|s| s.unit.as_deref())
            .all(|u| u == "m" || u == "rad")
    );
}

#[test]
fn relative_misclosure_is_dimensionless() {
    let r = call(square()).expect("square");
    assert_eq!(r.outputs["relative_misclosure"].unit, None);
    assert_eq!(r.outputs["perimeter"].unit.as_deref(), Some("m"));
}

// --- Errors ------------------------------------------------------------------------------------------

#[test]
fn lists_of_different_lengths_are_refused() {
    let (code, input, _, message) = err_code(json!({
        "bearings": degrees(&[0.0, 90.0, 180.0, 270.0]),
        "distances": metres(&[100.0, 100.0, 100.0]),
    }));
    assert_eq!(code, ErrorCode::InvalidValue);
    assert_eq!(input.as_deref(), Some("distances"));
    assert!(
        message.contains("'legs'") && message.contains('4') && message.contains('3'),
        "{message}"
    );
}

#[test]
fn fewer_than_three_legs_are_refused() {
    let (code, _, _, message) = err_code(json!({
        "bearings": degrees(&[0.0, 180.0]),
        "distances": metres(&[100.0, 100.0]),
    }));
    assert_eq!(code, ErrorCode::InvalidValue);
    assert!(message.contains('3'), "{message}");
}

#[test]
fn a_zero_or_negative_distance_names_the_leg() {
    for bad in [0.0, -5.0] {
        let (code, input, index, message) = err_code(json!({
            "bearings": degrees(&[0.0, 90.0, 180.0, 270.0]),
            "distances": metres(&[100.0, bad, 100.0, 100.0]),
        }));
        assert_eq!(code, ErrorCode::OutOfDomain, "{bad}");
        assert_eq!(input.as_deref(), Some("distances"));
        assert_eq!(index, Some(1));
        assert!(message.contains("leg 2"), "{message}");
    }
}

#[test]
fn a_distance_without_a_unit_is_refused() {
    let (code, input, ..) = err_code(json!({
        "bearings": degrees(&[0.0, 90.0, 180.0, 270.0]),
        "distances": { "value": [100.0, 100.0, 100.0, 100.0] },
    }));
    assert_eq!(code, ErrorCode::MissingUnit);
    assert_eq!(input.as_deref(), Some("distances"));
}

#[test]
fn a_bearing_without_a_unit_is_refused() {
    let (code, input, ..) = err_code(json!({
        "bearings": { "value": [0.0, 90.0, 180.0, 270.0] },
        "distances": metres(&[100.0, 100.0, 100.0, 100.0]),
    }));
    assert_eq!(code, ErrorCode::MissingUnit);
    assert_eq!(input.as_deref(), Some("bearings"));
}

#[test]
fn a_missing_list_is_reported() {
    let (code, input, ..) = err_code(json!({ "bearings": degrees(&[0.0, 90.0, 180.0]) }));
    assert_eq!(code, ErrorCode::MissingInput);
    assert_eq!(input.as_deref(), Some("distances"));
}

#[test]
fn a_bad_bearing_names_the_leg() {
    let (code, input, index, message) = err_code(json!({
        "bearings": { "value": ["N 10° E", "E 10° N", "S 10° W"], "unit": "bearing" },
        "distances": metres(&[100.0, 100.0, 100.0]),
    }));
    assert_eq!(code, ErrorCode::InvalidValue);
    assert_eq!(input.as_deref(), Some("bearings"));
    assert_eq!(index, Some(1));
    assert!(message.contains("leg 2"), "{message}");
}

#[test]
fn a_perimeter_that_overflows_is_out_of_domain_not_nan() {
    let (code, input, _, message) = err_code(json!({
        "bearings": degrees(&[0.0, 120.0, 240.0]),
        "distances": metres(&[1.0e308, 1.0e308, 1.0e308]),
    }));
    assert_eq!(code, ErrorCode::OutOfDomain);
    assert_eq!(input.as_deref(), Some("distances"));
    assert!(message.contains("too large"), "{message}");
}

// --- The trace ---------------------------------------------------------------------------------------

#[test]
fn the_trace_has_one_item_step_per_leg_for_each_per_leg_expression() {
    let n = 4;
    let r = call(square()).expect("square");
    // 6n per-leg steps and 5 summary steps; degrees need no conversion step.
    assert_eq!(
        r.trace.len(),
        6 * n + 5,
        "{:#?}",
        r.trace.iter().map(|s| &s.step).collect::<Vec<_>>()
    );
    let per_leg: Vec<_> = r.trace.iter().filter(|s| s.item.is_some()).collect();
    assert_eq!(per_leg.len(), 6 * n);
    for step in &per_leg {
        let item = step.item.as_ref().expect("item");
        assert_eq!(item.label, "leg");
        assert!(item.index < n);
        assert!(
            step.step.starts_with(&format!("leg {}: ", item.index + 1)),
            "{}",
            step.step
        );
        assert_eq!(step.unit.as_deref(), Some("m"));
    }
    // Every leg appears in each of the six per-leg expressions.
    for expression in ["Lat = ", "Dep = ", "c_N = ", "c_E = ", "Lat′ = ", "Dep′ = "] {
        let count = per_leg
            .iter()
            .filter(|s| s.step.contains(expression))
            .count();
        assert_eq!(count, n, "{expression}");
    }
    // The summary steps, in order, after the per-leg latitudes and departures.
    let texts: Vec<&str> = r.trace.iter().map(|s| s.step.as_str()).collect();
    let pos = |needle: &str| texts.iter().position(|t| t.contains(needle)).expect(needle);
    assert!(pos("P = Σ") < pos("M_N = Σ"));
    assert!(pos("M_N = Σ") < pos("E = √"));
    assert!(pos("E = √") < pos("c_N = "));
    assert!(pos("c_N = ") < pos("Lat′ = "));
    assert!(
        r.trace.iter().all(|s| s.choice.is_none()),
        "no choice steps"
    );
    // Every step is a length in metres, except the dimensionless ratio E/P (the registry accepts any valid
    // internal unit, so a wrong one would otherwise pass).
    for step in &r.trace {
        let expected = if step.step == "E/P" { None } else { Some("m") };
        assert_eq!(step.unit.as_deref(), expected, "{}", step.step);
    }
}

#[test]
fn bearing_strings_add_one_conversion_step_per_leg_before_the_calculation() {
    let r = call(json!({
        "bearings": { "value": ["N 30° E", "S 45° E", "N 75° W"], "unit": "bearing" },
        "distances": metres(&[30.0, 60.0, 45.0]),
    }))
    .expect("bearings");
    assert_eq!(r.trace.len(), 3 + 6 * 3 + 5);
    for (k, step) in r.trace[..3].iter().enumerate() {
        assert!(
            step.step
                .starts_with(&format!("leg {}: bearings: bearing ", k + 1)),
            "{}",
            step.step
        );
        assert_eq!(step.unit.as_deref(), Some("rad"));
    }
}

#[test]
fn list_outputs_have_one_entry_per_leg() {
    let r = call(json!({
        "bearings": degrees(&[10.0, 100.0, 200.0, 290.0, 330.0]),
        "distances": metres(&[10.0, 20.0, 30.0, 40.0, 50.0]),
    }))
    .expect("five legs");
    for name in [
        "latitudes",
        "departures",
        "latitude_corrections",
        "departure_corrections",
        "adjusted_latitudes",
        "adjusted_departures",
    ] {
        assert_eq!(list(&r, name).len(), 5, "{name}");
    }
}

// --- Properties --------------------------------------------------------------------------------------

fn legs() -> impl Strategy<Value = Vec<(f64, f64)>> {
    prop::collection::vec((0.0f64..360.0, 1.0f64..1000.0), 3..9)
}

fn run(legs: &[(f64, f64)], scale: f64) -> Response {
    let bearings: Vec<f64> = legs.iter().map(|l| l.0).collect();
    let distances: Vec<f64> = legs.iter().map(|l| l.1 * scale).collect();
    call(json!({ "bearings": degrees(&bearings), "distances": metres(&distances) }))
        .expect("valid traverse")
}

proptest! {
    #[test]
    fn the_adjusted_traverse_always_closes(legs in legs()) {
        let r = run(&legs, 1.0);
        let p = scalar(&r, "perimeter");
        let lat: f64 = list(&r, "adjusted_latitudes").iter().sum();
        let dep: f64 = list(&r, "adjusted_departures").iter().sum();
        prop_assert!(lat.abs() <= 1e-10 * p, "sum of adjusted latitudes {lat}");
        prop_assert!(dep.abs() <= 1e-10 * p, "sum of adjusted departures {dep}");
    }

    #[test]
    fn the_misclosure_matches_its_parts(legs in legs()) {
        let r = run(&legs, 1.0);
        let e = scalar(&r, "latitude_misclosure").hypot(scalar(&r, "departure_misclosure"));
        prop_assert!((scalar(&r, "linear_misclosure") - e).abs() <= 1e-12 * e.max(1.0));
        let ratio = scalar(&r, "linear_misclosure") / scalar(&r, "perimeter");
        prop_assert!((scalar(&r, "relative_misclosure") - ratio).abs() <= 1e-15 + 1e-12 * ratio);
    }

    #[test]
    fn starting_at_another_leg_shifts_the_outputs(legs in legs(), shift in 0usize..8) {
        let n = legs.len();
        let r = shift % n;
        let mut rotated = legs.clone();
        rotated.rotate_left(r);
        let (a, b) = (run(&legs, 1.0), run(&rotated, 1.0));
        for name in ["latitudes", "departures", "adjusted_latitudes", "adjusted_departures",
                     "latitude_corrections", "departure_corrections"] {
            let (x, y) = (list(&a, name), list(&b, name));
            for k in 0..n {
                prop_assert!((y[k] - x[(k + r) % n]).abs() <= 1e-9, "{name}[{k}]");
            }
        }
        for name in ["perimeter", "linear_misclosure", "relative_misclosure"] {
            prop_assert!((scalar(&a, name) - scalar(&b, name)).abs() <= 1e-9, "{name}");
        }
    }

    #[test]
    fn doubling_every_distance_doubles_the_lengths_and_keeps_the_ratio(legs in legs()) {
        let (a, b) = (run(&legs, 1.0), run(&legs, 2.0));
        for name in ["latitudes", "departures", "adjusted_latitudes", "adjusted_departures",
                     "latitude_corrections", "departure_corrections"] {
            let (x, y) = (list(&a, name), list(&b, name));
            for k in 0..x.len() {
                prop_assert!((y[k] - 2.0 * x[k]).abs() <= 1e-9, "{name}[{k}]");
            }
        }
        for name in ["perimeter", "linear_misclosure"] {
            prop_assert!((scalar(&b, name) - 2.0 * scalar(&a, name)).abs() <= 1e-9, "{name}");
        }
        prop_assert!((scalar(&b, "relative_misclosure") - scalar(&a, "relative_misclosure")).abs() <= 1e-12);
    }
}
