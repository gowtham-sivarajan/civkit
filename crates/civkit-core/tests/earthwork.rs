// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-26 00:18 IST · last edited 2026-11-26 00:23 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! `quantities.earthwork.average_end_area` through the public contract (D-20, ADR 0009 condition 3,
//! planning/EARTHWORK_DESCRIPTOR_PROPOSAL.md §8.2).
//!
//! The reference cases C1 and C2 live in `tests/reference/`. These tests cover closed forms, the visible and never
//! silent choice of method, errors, units, the trace and the properties.

use civkit_core::{ErrorCode, Registry, Request, Response};
use proptest::prelude::*;
use serde_json::json;

const ID: &str = "quantities.earthwork.average_end_area";
const AVG: &str = "average_end_area";
const PYR: &str = "pyramid_at_zero_end";

fn call(inputs: serde_json::Value) -> Result<Response, civkit_core::ErrorEnvelope> {
    let request: Request =
        serde_json::from_value(json!({ "id": ID, "inputs": inputs })).expect("request parses");
    Registry::with_builtins().call(&request)
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

fn run(method: &str, x: &[f64], cut: &[f64], fill: &[f64]) -> serde_json::Value {
    json!({
        "method": { "value": method },
        "chainages": { "value": x, "unit": "m" },
        "cut_areas": { "value": cut, "unit": "m^2" },
        "fill_areas": { "value": fill, "unit": "m^2" },
    })
}

fn failure(inputs: serde_json::Value) -> (ErrorCode, Option<String>, Option<usize>, String) {
    let e = call(inputs).expect_err("should fail").0;
    (
        e[0].code,
        e[0].input.clone(),
        e[0].index,
        e[0].message.clone(),
    )
}

// C2 (Wikibooks, Fundamentals of Transportation, Earthwork, Solution): stations 0, 50, 75, 100, 115, 150 m.
const C2_X: [f64; 6] = [0.0, 50.0, 75.0, 100.0, 115.0, 150.0];
const C2_CUT: [f64; 6] = [0.0, 0.0, 0.0, 10.0, 15.0, 30.0];
const C2_FILL: [f64; 6] = [60.0, 50.0, 25.0, 5.0, 0.0, 0.0];

// --- Registration --------------------------------------------------------------------------------

#[test]
fn the_calculation_is_registered_with_the_accepted_descriptor() {
    let registry = Registry::with_builtins();
    let d = registry.describe(ID, None).expect("registered");
    assert_eq!(d.version, 1);
    assert_eq!(d.groups.len(), 1);
    assert_eq!(
        (d.groups[0].name.as_str(), d.groups[0].item.as_str()),
        ("stations", "station")
    );
    assert_eq!(d.groups[0].min_len, 2);
    let inputs: Vec<_> = d.inputs.iter().map(|i| i.name.as_str()).collect();
    assert_eq!(inputs, ["method", "chainages", "cut_areas", "fill_areas"]);
    assert_eq!(d.inputs[0].choices, [AVG, PYR]);
    assert!(
        d.inputs[0].required && d.inputs[0].default.is_none(),
        "no default method (ADR 0009 condition 3)"
    );
    let outputs: Vec<_> = d.outputs.iter().map(|o| o.name.as_str()).collect();
    assert_eq!(
        outputs,
        [
            "cut_volumes",
            "fill_volumes",
            "total_cut",
            "total_fill",
            "net_fill"
        ]
    );
    let length = d.outputs[0].length.as_ref().expect("list output");
    assert_eq!(
        (length.group.as_str(), length.offset, length.item.as_str()),
        ("stations", -1, "segment")
    );
    assert!(d.validity.is_empty(), "no validity rules (Q-E)");
}

// --- Closed forms --------------------------------------------------------------------------------

#[test]
fn a_constant_section_gives_area_times_length_under_both_methods() {
    for method in [AVG, PYR] {
        let r = call(run(method, &[0.0, 100.0], &[10.0, 10.0], &[0.0, 0.0])).expect(method);
        close_all(&list(&r, "cut_volumes"), &[1000.0], 1e-9, "cut");
        close_all(&list(&r, "fill_volumes"), &[0.0], 1e-9, "fill");
        close(scalar(&r, "total_cut"), 1000.0, 1e-9, "total cut");
        close(scalar(&r, "net_fill"), -1000.0, 1e-9, "net fill");
    }
}

#[test]
fn a_segment_ending_at_a_zero_area_is_half_or_a_third_by_the_requested_method() {
    let avg = call(run(AVG, &[0.0, 50.0], &[0.0, 30.0], &[0.0, 0.0])).expect("avg");
    close_all(&list(&avg, "cut_volumes"), &[750.0], 1e-9, "average: A·L/2");
    let pyr = call(run(PYR, &[0.0, 50.0], &[0.0, 30.0], &[0.0, 0.0])).expect("pyramid");
    close_all(&list(&pyr, "cut_volumes"), &[500.0], 1e-9, "pyramid: A·L/3");
    // The zero end can be either end.
    let pyr = call(run(PYR, &[0.0, 50.0], &[30.0, 0.0], &[0.0, 0.0]))
        .expect("pyramid, zero at the far end");
    close_all(
        &list(&pyr, "cut_volumes"),
        &[500.0],
        1e-9,
        "pyramid, far end",
    );
}

#[test]
fn two_zero_ends_give_no_volume_under_either_method() {
    for method in [AVG, PYR] {
        let r = call(run(method, &[0.0, 50.0], &[0.0, 0.0], &[0.0, 0.0])).expect(method);
        assert_eq!(list(&r, "cut_volumes"), [0.0]);
        assert_eq!(list(&r, "fill_volumes"), [0.0]);
    }
}

#[test]
fn cut_and_fill_follow_the_rule_independently() {
    // Cut tapers from zero, fill tapers to zero, in the same segment.
    let r = call(run(PYR, &[0.0, 50.0], &[0.0, 30.0], &[20.0, 0.0])).expect("pyramid");
    close_all(&list(&r, "cut_volumes"), &[500.0], 1e-9, "cut: 30·50/3");
    close_all(
        &list(&r, "fill_volumes"),
        &[20.0 * 50.0 / 3.0],
        1e-9,
        "fill: 20·50/3",
    );
    let r = call(run(AVG, &[0.0, 50.0], &[0.0, 30.0], &[20.0, 0.0])).expect("average");
    close_all(&list(&r, "cut_volumes"), &[750.0], 1e-9, "cut: average");
    close_all(&list(&r, "fill_volumes"), &[500.0], 1e-9, "fill: average");
}

#[test]
fn net_fill_is_fill_less_cut() {
    let r = call(run(AVG, &[0.0, 10.0], &[2.0, 2.0], &[5.0, 5.0])).expect("fill exceeds cut");
    close(
        scalar(&r, "net_fill"),
        30.0,
        1e-9,
        "50 − 20: fill is required",
    );
    let r = call(run(AVG, &[0.0, 10.0], &[5.0, 5.0], &[2.0, 2.0])).expect("cut exceeds fill");
    close(scalar(&r, "net_fill"), -30.0, 1e-9, "20 − 50: surplus cut");
}

// --- The reference examples through the contract ---------------------------------------------------

#[test]
fn c1_average_end_area() {
    let r = call(run(
        AVG,
        &[0.0, 50.0, 100.0, 150.0],
        &[40.0, 42.0, 19.0, 34.0],
        &[0.0; 4],
    ))
    .expect("c1");
    close_all(
        &list(&r, "cut_volumes"),
        &[2050.0, 1525.0, 1325.0],
        1e-9,
        "segments",
    );
    close(scalar(&r, "total_cut"), 4900.0, 1e-9, "total");
}

#[test]
fn c2_pyramid_method_reproduces_the_source() {
    let r = call(run(PYR, &C2_X, &C2_CUT, &C2_FILL)).expect("c2");
    close_all(
        &list(&r, "cut_volumes"),
        &[0.0, 0.0, 250.0 / 3.0, 187.5, 787.5],
        1e-9,
        "cut segments",
    );
    close_all(
        &list(&r, "fill_volumes"),
        &[2750.0, 937.5, 375.0, 25.0, 0.0],
        1e-9,
        "fill segments",
    );
    close(
        scalar(&r, "total_fill"),
        4087.5,
        1e-9,
        "total fill (book 4 087.5)",
    );
    close(
        scalar(&r, "total_cut"),
        1058.0 + 1.0 / 3.0,
        1e-9,
        "total cut (book 1 058.3)",
    );
    close(
        scalar(&r, "net_fill"),
        3029.0 + 1.0 / 6.0,
        1e-9,
        "net fill (book 3 029.2)",
    );
}

#[test]
fn c2_data_under_average_end_area_differs_which_is_why_the_method_is_explicit() {
    let r = call(run(AVG, &C2_X, &C2_CUT, &C2_FILL)).expect("c2, average end area");
    close(
        scalar(&r, "total_fill"),
        4100.0,
        1e-9,
        "fill 12.5 m³ higher",
    );
    close(scalar(&r, "total_cut"), 1100.0, 1e-9, "cut 41.7 m³ higher");
    close(scalar(&r, "net_fill"), 3000.0, 1e-9, "net fill");
}

// --- The choice is visible and never silent ------------------------------------------------------------

#[test]
fn the_chosen_method_is_the_first_trace_step() {
    for method in [AVG, PYR] {
        let r = call(run(method, &C2_X, &C2_CUT, &C2_FILL)).expect(method);
        let first = &r.trace[0];
        assert_eq!(first.step, "method");
        assert_eq!(first.choice.as_deref(), Some(method));
        assert!(first.value.is_none());
        assert_eq!(
            r.trace.iter().filter(|s| s.choice.is_some()).count(),
            1,
            "exactly one choice step"
        );
    }
}

#[test]
fn the_two_methods_differ_exactly_on_the_zero_end_segments() {
    let avg = call(run(AVG, &C2_X, &C2_CUT, &C2_FILL)).expect("avg");
    let pyr = call(run(PYR, &C2_X, &C2_CUT, &C2_FILL)).expect("pyr");
    let differs = |name: &str| -> Vec<usize> {
        let (a, p) = (list(&avg, name), list(&pyr, name));
        (0..a.len())
            .filter(|k| (a[*k] - p[*k]).abs() > 1e-9)
            .collect()
    };
    assert_eq!(differs("cut_volumes"), [2], "cut: 0 → 10 at segment 3");
    assert_eq!(differs("fill_volumes"), [3], "fill: 5 → 0 at segment 4");
}

#[test]
fn the_trace_says_which_formula_each_segment_used() {
    let pyr = call(run(PYR, &C2_X, &C2_CUT, &C2_FILL)).expect("pyr");
    let step = |r: &Response, text: &str| -> String {
        r.trace
            .iter()
            .find(|s| s.step.starts_with(text))
            .unwrap_or_else(|| panic!("{text}"))
            .step
            .clone()
    };
    assert!(
        step(&pyr, "segment 3: cut volume").contains("pyramid"),
        "{}",
        step(&pyr, "segment 3: cut volume")
    );
    assert!(step(&pyr, "segment 4: fill volume").contains("pyramid"));
    assert!(
        !step(&pyr, "segment 3: fill volume").contains("pyramid"),
        "fill 25 → 5 is average end area"
    );
    let avg = call(run(AVG, &C2_X, &C2_CUT, &C2_FILL)).expect("avg");
    assert!(
        avg.trace.iter().all(|s| !s.step.contains("pyramid")),
        "average end area never mentions the pyramid"
    );
}

#[test]
fn an_unknown_or_missing_method_is_refused() {
    let (code, input, _, message) =
        failure(run("prismoidal", &[0.0, 1.0], &[1.0, 1.0], &[0.0, 0.0]));
    assert_eq!(
        (code, input.as_deref()),
        (ErrorCode::InvalidValue, Some("method"))
    );
    assert!(message.contains(AVG) && message.contains(PYR), "{message}");

    let (code, input, ..) = failure(json!({
        "chainages": { "value": [0.0, 1.0], "unit": "m" },
        "cut_areas": { "value": [1.0, 1.0], "unit": "m^2" },
        "fill_areas": { "value": [0.0, 0.0], "unit": "m^2" },
    }));
    assert_eq!(
        (code, input.as_deref()),
        (ErrorCode::MissingInput, Some("method"))
    );
}

// --- Errors --------------------------------------------------------------------------------------

#[test]
fn chainages_must_strictly_increase() {
    for x in [[0.0, 50.0, 50.0], [0.0, 50.0, 40.0]] {
        let (code, input, index, message) = failure(run(AVG, &x, &[1.0; 3], &[0.0; 3]));
        assert_eq!(
            (code, input.as_deref(), index),
            (ErrorCode::OutOfDomain, Some("chainages"), Some(2)),
            "{x:?}"
        );
        assert!(message.contains("station 3"), "{message}");
    }
}

#[test]
fn lists_of_different_lengths_and_a_single_station_are_refused() {
    let (code, input, _, message) =
        failure(run(AVG, &[0.0, 50.0, 100.0], &[1.0, 1.0], &[0.0, 0.0, 0.0]));
    assert_eq!(
        (code, input.as_deref()),
        (ErrorCode::InvalidValue, Some("cut_areas"))
    );
    assert!(message.contains("'stations'"), "{message}");

    let (code, _, _, message) = failure(run(AVG, &[0.0], &[1.0], &[0.0]));
    assert_eq!(code, ErrorCode::InvalidValue);
    assert!(message.contains('2'), "{message}");
}

#[test]
fn a_negative_area_names_the_station() {
    let (code, input, index, message) =
        failure(run(AVG, &[0.0, 10.0, 20.0], &[1.0, -1.0, 1.0], &[0.0; 3]));
    assert_eq!(
        (code, input.as_deref(), index),
        (ErrorCode::OutOfDomain, Some("cut_areas"), Some(1))
    );
    assert!(message.contains("station 2"), "{message}");
    let (code, input, index, _) = failure(run(AVG, &[0.0, 10.0], &[1.0, 1.0], &[0.0, -0.5]));
    assert_eq!(
        (code, input.as_deref(), index),
        (ErrorCode::OutOfDomain, Some("fill_areas"), Some(1))
    );
}

#[test]
fn units_are_required_and_must_match() {
    let (code, input, ..) = failure(json!({
        "method": { "value": AVG },
        "chainages": { "value": [0.0, 1.0], "unit": "m" },
        "cut_areas": { "value": [1.0, 1.0] },
        "fill_areas": { "value": [0.0, 0.0], "unit": "m^2" },
    }));
    assert_eq!(
        (code, input.as_deref()),
        (ErrorCode::MissingUnit, Some("cut_areas"))
    );

    let (code, input, ..) = failure(json!({
        "method": { "value": AVG },
        "chainages": { "value": [0.0, 1.0], "unit": "m" },
        "cut_areas": { "value": [1.0, 1.0], "unit": "m" },
        "fill_areas": { "value": [0.0, 0.0], "unit": "m^2" },
    }));
    assert_eq!(
        (code, input.as_deref()),
        (ErrorCode::UnitMismatch, Some("cut_areas"))
    );
}

#[test]
fn a_volume_that_overflows_is_out_of_domain_not_infinity() {
    let (code, _, _, message) = failure(run(AVG, &[0.0, 1.0e10], &[1.0e300, 1.0e300], &[0.0, 0.0]));
    assert_eq!(code, ErrorCode::OutOfDomain);
    assert!(message.contains("too large"), "{message}");
}

// --- Units and the trace ---------------------------------------------------------------------------

#[test]
fn feet_in_cubic_metres_or_cubic_yards_out() {
    let metric = call(run(
        AVG,
        &[0.0, 30.48, 60.96],
        &[0.0929030, 0.3716122, 0.0],
        &[0.0; 3],
    ))
    .expect("metres");
    let feet = call(json!({
        "method": { "value": AVG },
        "chainages": { "value": [0.0, 100.0, 200.0], "unit": "ft" },
        "cut_areas": { "value": [0.0929030 / 0.09290304, 0.3716122 / 0.09290304, 0.0], "unit": "ft^2" },
        "fill_areas": { "value": [0.0, 0.0, 0.0], "unit": "ft^2" },
    }))
    .expect("feet");
    close(
        scalar(&feet, "total_cut"),
        scalar(&metric, "total_cut"),
        1e-6,
        "total cut",
    );

    let request: Request = serde_json::from_value(json!({
        "id": ID,
        "inputs": run(AVG, &[0.0, 100.0], &[10.0, 10.0], &[0.0, 0.0]),
        "output_units": { "total_cut": "yd^3" },
    }))
    .expect("request");
    let r = Registry::with_builtins().call(&request).expect("yd³ out");
    close(
        scalar(&r, "total_cut"),
        1000.0 / 0.764_554_857_984,
        1e-9,
        "1 000 m³ in yd³",
    );
    assert_eq!(r.outputs["total_cut"].unit.as_deref(), Some("yd^3"));
}

#[test]
fn the_trace_has_a_choice_step_two_item_steps_per_segment_and_three_totals() {
    let n = 4;
    let r = call(run(
        AVG,
        &[0.0, 10.0, 20.0, 30.0],
        &[1.0, 2.0, 3.0, 4.0],
        &[0.0; 4],
    ))
    .expect("avg");
    assert_eq!(r.trace.len(), 1 + 2 * (n - 1) + 3);
    let per: Vec<_> = r.trace.iter().filter(|s| s.item.is_some()).collect();
    assert_eq!(per.len(), 2 * (n - 1));
    for step in &per {
        let item = step.item.as_ref().expect("item");
        assert_eq!(item.label, "segment");
        assert!(item.index < n - 1);
        assert!(
            step.step
                .starts_with(&format!("segment {}: ", item.index + 1)),
            "{}",
            step.step
        );
        assert_eq!(step.unit.as_deref(), Some("m^3"));
    }
    let texts: Vec<&str> = r.trace.iter().map(|s| s.step.as_str()).collect();
    assert!(
        texts.iter().any(|t| t.contains("ΣV_cut")) && texts.iter().any(|t| t.contains("net fill"))
    );
    // Every numeric step, including the totals, is a volume in m³ (the registry accepts any valid internal unit).
    let numeric: Vec<_> = r.trace.iter().filter(|s| s.value.is_some()).collect();
    assert_eq!(numeric.len(), 2 * (n - 1) + 3);
    assert!(
        numeric.iter().all(|s| s.unit.as_deref() == Some("m^3")),
        "{numeric:#?}"
    );
}

// --- Properties --------------------------------------------------------------------------------

fn area() -> BoxedStrategy<f64> {
    prop_oneof![1 => Just(0.0), 3 => 0.0f64..100.0].boxed()
}

fn nonzero_area() -> BoxedStrategy<f64> {
    (1.0f64..100.0).boxed()
}

/// Stations: a start chainage, then one (gap, cut area, fill area) row per station. Row 0's gap is unused: the
/// first station is the start chainage, and each later station is the previous one plus its positive gap.
type Run = (f64, Vec<(f64, f64, f64)>);

fn runs(areas: BoxedStrategy<f64>) -> impl Strategy<Value = Run> {
    (
        -100.0f64..100.0,
        prop::collection::vec((1.0f64..100.0, areas.clone(), areas), 2..10),
    )
}

fn unpack(run: &Run) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut x = vec![run.0];
    for (g, _, _) in &run.1[1..] {
        let last = *x.last().expect("a station");
        x.push(last + g);
    }
    (
        x,
        run.1.iter().map(|r| r.1).collect(),
        run.1.iter().map(|r| r.2).collect(),
    )
}

proptest! {
    #[test]
    fn totals_are_the_sums_and_net_fill_is_fill_less_cut(scn in runs(area())) {
        let (x, cut, fill) = unpack(&scn);
        for method in [AVG, PYR] {
            let r = call(run(method, &x, &cut, &fill)).expect("valid run");
            let (cv, fv) = (list(&r, "cut_volumes"), list(&r, "fill_volumes"));
            prop_assert_eq!(cv.len(), x.len() - 1);
            prop_assert!((scalar(&r, "total_cut") - cv.iter().sum::<f64>()).abs() <= 1e-9);
            prop_assert!((scalar(&r, "total_fill") - fv.iter().sum::<f64>()).abs() <= 1e-9);
            prop_assert!((scalar(&r, "net_fill") - (scalar(&r, "total_fill") - scalar(&r, "total_cut"))).abs() <= 1e-9);
        }
    }

    #[test]
    fn swapping_cut_and_fill_swaps_the_totals_and_flips_the_net(scn in runs(area())) {
        let (x, cut, fill) = unpack(&scn);
        for method in [AVG, PYR] {
            let a = call(run(method, &x, &cut, &fill)).expect("a");
            let b = call(run(method, &x, &fill, &cut)).expect("b");
            prop_assert!((scalar(&a, "total_cut") - scalar(&b, "total_fill")).abs() <= 1e-9);
            prop_assert!((scalar(&a, "total_fill") - scalar(&b, "total_cut")).abs() <= 1e-9);
            prop_assert!((scalar(&a, "net_fill") + scalar(&b, "net_fill")).abs() <= 1e-9);
        }
    }

    #[test]
    fn scaling_the_stations_scales_every_volume(scn in runs(area()), scale in 0.5f64..4.0) {
        let (x, cut, fill) = unpack(&scn);
        let x2: Vec<f64> = x.iter().map(|v| v * scale).collect();
        for method in [AVG, PYR] {
            let a = call(run(method, &x, &cut, &fill)).expect("a");
            let b = call(run(method, &x2, &cut, &fill)).expect("b");
            prop_assert!((scalar(&b, "total_cut") - scale * scalar(&a, "total_cut")).abs() <= 1e-7);
            prop_assert!((scalar(&b, "total_fill") - scale * scalar(&a, "total_fill")).abs() <= 1e-7);
        }
    }

    #[test]
    fn without_a_zero_end_the_methods_are_identical(scn in runs(nonzero_area())) {
        let (x, cut, fill) = unpack(&scn);
        let a = call(run(AVG, &x, &cut, &fill)).expect("a");
        let b = call(run(PYR, &x, &cut, &fill)).expect("b");
        for name in ["cut_volumes", "fill_volumes"] {
            let (p, q) = (list(&a, name), list(&b, name));
            for k in 0..p.len() {
                prop_assert!((p[k] - q[k]).abs() <= 1e-12, "{name}[{k}]");
            }
        }
    }

    #[test]
    fn a_pyramid_is_two_thirds_of_the_average_exactly_where_one_end_is_zero(scn in runs(area())) {
        let (x, cut, fill) = unpack(&scn);
        let a = call(run(AVG, &x, &cut, &fill)).expect("a");
        let b = call(run(PYR, &x, &cut, &fill)).expect("b");
        for (name, areas) in [("cut_volumes", &cut), ("fill_volumes", &fill)] {
            let (p, q) = (list(&a, name), list(&b, name));
            for k in 0..p.len() {
                let one_zero = (areas[k] <= 0.0) != (areas[k + 1] <= 0.0);
                let expected = if one_zero { p[k] * 2.0 / 3.0 } else { p[k] };
                prop_assert!((q[k] - expected).abs() <= 1e-9, "{name}[{k}]");
            }
        }
        prop_assert!(scalar(&b, "total_cut") <= scalar(&a, "total_cut") + 1e-9);
        prop_assert!(scalar(&b, "total_fill") <= scalar(&a, "total_fill") + 1e-9);
    }
}
