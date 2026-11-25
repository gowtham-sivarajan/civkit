// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-26 00:09 IST · last edited 2026-11-26 00:23 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! The three levelling calculations through the public contract (D-20, planning/LEVELLING_DESCRIPTOR_PROPOSAL.md §7):
//! `surveying.levelling.height_of_instrument`, `.rise_and_fall` and `.closure_adjustment`.
//!
//! The reference cases C4 and C9 live in `tests/reference/`; the exact arithmetic against the book's printed numbers
//! is tested next to the code (`src/calcs/levelling.rs`). These tests cover closed forms, units, errors, the trace and
//! the properties, including that the two reductions agree.

use civkit_core::{ErrorCode, Registry, Request, Response};
use proptest::prelude::*;
use serde_json::json;

const HI: &str = "surveying.levelling.height_of_instrument";
const RF: &str = "surveying.levelling.rise_and_fall";
const ADJ: &str = "surveying.levelling.closure_adjustment";

fn call(id: &str, inputs: serde_json::Value) -> Result<Response, civkit_core::ErrorEnvelope> {
    let request: Request =
        serde_json::from_value(json!({ "id": id, "inputs": inputs })).expect("request parses");
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

fn m(values: &[f64]) -> serde_json::Value {
    json!({ "value": values, "unit": "m" })
}

fn reduction(start: f64, bs: &[f64], fs: &[f64]) -> serde_json::Value {
    json!({
        "start_elevation": { "value": start, "unit": "m" },
        "backsights": m(bs),
        "foresights": m(fs),
    })
}

fn adjustment(z: &[f64], lengths: &[f64], closing: f64) -> serde_json::Value {
    json!({
        "elevations": m(z),
        "section_lengths": m(lengths),
        "closing_elevation": { "value": closing, "unit": "m" },
    })
}

fn failure(
    id: &str,
    inputs: serde_json::Value,
) -> (ErrorCode, Option<String>, Option<usize>, String) {
    let e = call(id, inputs).expect_err("should fail").0;
    (
        e[0].code,
        e[0].input.clone(),
        e[0].index,
        e[0].message.clone(),
    )
}

// --- Registration --------------------------------------------------------------------------------

#[test]
fn the_three_calculations_are_registered_with_the_accepted_descriptors() {
    let registry = Registry::with_builtins();
    for (id, group, item, inputs, outputs) in [
        (
            HI,
            "setups",
            "setup",
            vec!["start_elevation", "backsights", "foresights"],
            vec![
                "heights_of_instrument",
                "elevations",
                "sum_backsights",
                "sum_foresights",
                "net_rise",
            ],
        ),
        (
            RF,
            "setups",
            "setup",
            vec!["start_elevation", "backsights", "foresights"],
            vec![
                "rises",
                "falls",
                "elevations",
                "sum_backsights",
                "sum_foresights",
                "net_rise",
            ],
        ),
        (
            ADJ,
            "points",
            "point",
            vec!["elevations", "section_lengths", "closing_elevation"],
            vec![
                "closure_error",
                "total_length",
                "corrections",
                "adjusted_elevations",
            ],
        ),
    ] {
        let d = registry
            .describe(id, None)
            .unwrap_or_else(|| panic!("{id} registered"));
        assert_eq!(d.version, 1, "{id}");
        assert_eq!(d.groups.len(), 1, "{id}");
        assert_eq!(
            (d.groups[0].name.as_str(), d.groups[0].item.as_str()),
            (group, item),
            "{id}"
        );
        assert_eq!(d.groups[0].min_len, 1, "{id}");
        let got_in: Vec<_> = d.inputs.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(got_in, inputs, "{id} inputs");
        let got_out: Vec<_> = d.outputs.iter().map(|o| o.name.as_str()).collect();
        assert_eq!(got_out, outputs, "{id} outputs");
        assert!(d.validity.is_empty(), "{id}: no validity rules (Q-C)");
        assert!(
            d.inputs.iter().all(|i| i.choices.is_empty()),
            "{id}: no choice input (Q-A)"
        );
    }
}

// --- Height of instrument ------------------------------------------------------------------------

#[test]
fn hi_closed_form() {
    // Start 100.000; setup 1: BS 1.5, FS 0.5; setup 2: BS 2.0, FS 3.0.
    let r = call(HI, reduction(100.0, &[1.5, 2.0], &[0.5, 3.0])).expect("hi");
    close_all(
        &list(&r, "heights_of_instrument"),
        &[101.5, 103.0],
        1e-12,
        "HI",
    );
    close_all(
        &list(&r, "elevations"),
        &[101.0, 100.0],
        1e-12,
        "elevations",
    );
    close(scalar(&r, "sum_backsights"), 3.5, 1e-12, "sum BS");
    close(scalar(&r, "sum_foresights"), 3.5, 1e-12, "sum FS");
    close(scalar(&r, "net_rise"), 0.0, 1e-12, "net rise");
    assert!(r.warnings.is_empty());
}

#[test]
fn a_negative_or_zero_start_elevation_is_valid() {
    let r = call(HI, reduction(-12.5, &[1.0], &[4.0])).expect("below datum");
    close_all(&list(&r, "elevations"), &[-15.5], 1e-12, "elevation");
    let r = call(HI, reduction(0.0, &[1.0], &[1.0])).expect("at datum");
    close_all(&list(&r, "elevations"), &[0.0], 1e-12, "elevation");
}

#[test]
fn hi_units_do_not_change_the_result() {
    let metric = call(HI, reduction(40.0, &[1.524, 3.048], &[0.3048, 0.6096])).expect("metres");
    let feet = call(
        HI,
        json!({
            "start_elevation": { "value": 40.0 / 0.3048, "unit": "ft" },
            "backsights": { "value": [5.0, 10.0], "unit": "ft" },
            "foresights": { "value": [1.0, 2.0], "unit": "ft" },
        }),
    )
    .expect("feet");
    for name in ["heights_of_instrument", "elevations"] {
        close_all(&list(&feet, name), &list(&metric, name), 1e-9, name);
    }
    close(
        scalar(&feet, "net_rise"),
        scalar(&metric, "net_rise"),
        1e-9,
        "net rise",
    );
}

#[test]
fn outputs_in_feet_convert_every_entry() {
    let mut req: Request = serde_json::from_value(json!({
        "id": HI,
        "inputs": reduction(0.0, &[0.3048, 0.6096], &[0.0, 0.0]),
        "output_units": { "elevations": "ft", "net_rise": "ft" },
    }))
    .expect("request");
    req.version = Some(1);
    let r = Registry::with_builtins().call(&req).expect("feet out");
    close_all(
        &list(&r, "elevations"),
        &[1.0, 3.0],
        1e-12,
        "elevations in ft",
    );
    close(scalar(&r, "net_rise"), 3.0, 1e-12, "net rise in ft");
    assert_eq!(r.outputs["elevations"].unit.as_deref(), Some("ft"));
}

#[test]
fn hi_errors_name_the_setup() {
    let (code, input, index, message) =
        failure(HI, reduction(100.0, &[1.0, -2.0, 1.0], &[1.0, 1.0, 1.0]));
    assert_eq!(code, ErrorCode::OutOfDomain);
    assert_eq!(input.as_deref(), Some("backsights"));
    assert_eq!(index, Some(1));
    assert!(message.contains("setup 2"), "{message}");

    let (code, input, index, _) = failure(HI, reduction(100.0, &[1.0, 1.0], &[1.0, -0.001]));
    assert_eq!(
        (code, input.as_deref(), index),
        (ErrorCode::OutOfDomain, Some("foresights"), Some(1))
    );
}

#[test]
fn hi_lists_of_different_lengths_and_empty_lists_are_refused() {
    let (code, input, _, message) = failure(HI, reduction(100.0, &[1.0, 1.0, 1.0], &[1.0, 1.0]));
    assert_eq!(code, ErrorCode::InvalidValue);
    assert_eq!(input.as_deref(), Some("foresights"));
    assert!(
        message.contains("'setups'") && message.contains('3') && message.contains('2'),
        "{message}"
    );

    let (code, ..) = failure(HI, reduction(100.0, &[], &[]));
    assert_eq!(code, ErrorCode::InvalidValue);
}

#[test]
fn hi_missing_input_and_missing_unit() {
    let (code, input, ..) = failure(
        HI,
        json!({ "backsights": m(&[1.0]), "foresights": m(&[1.0]) }),
    );
    assert_eq!(
        (code, input.as_deref()),
        (ErrorCode::MissingInput, Some("start_elevation"))
    );

    let (code, input, ..) = failure(
        HI,
        json!({
            "start_elevation": { "value": 100.0 },
            "backsights": m(&[1.0]),
            "foresights": m(&[1.0]),
        }),
    );
    assert_eq!(
        (code, input.as_deref()),
        (ErrorCode::MissingUnit, Some("start_elevation"))
    );

    let (code, input, ..) = failure(
        HI,
        json!({
            "start_elevation": { "value": 100.0, "unit": "m" },
            "backsights": { "value": [1.0] },
            "foresights": m(&[1.0]),
        }),
    );
    assert_eq!(
        (code, input.as_deref()),
        (ErrorCode::MissingUnit, Some("backsights"))
    );
}

#[test]
fn a_height_of_instrument_that_overflows_is_out_of_domain_not_infinity() {
    let (code, _, _, message) = failure(HI, reduction(1.0e308, &[1.0e308], &[0.0]));
    assert_eq!(code, ErrorCode::OutOfDomain);
    assert!(message.contains("too large"), "{message}");
}

#[test]
fn hi_trace_has_two_item_steps_per_setup_and_three_summary_steps() {
    let n = 3;
    let r = call(HI, reduction(10.0, &[1.0, 2.0, 3.0], &[0.5, 0.5, 0.5])).expect("hi");
    assert_eq!(r.trace.len(), 2 * n + 3);
    let per: Vec<_> = r.trace.iter().filter(|s| s.item.is_some()).collect();
    assert_eq!(per.len(), 2 * n);
    for step in &per {
        let item = step.item.as_ref().expect("item");
        assert_eq!(item.label, "setup");
        assert!(
            step.step
                .starts_with(&format!("setup {}: ", item.index + 1)),
            "{}",
            step.step
        );
        assert_eq!(step.unit.as_deref(), Some("m"));
    }
    assert_eq!(per.iter().filter(|s| s.step.contains("HI = ")).count(), n);
    assert_eq!(
        per.iter()
            .filter(|s| s.step.contains("elevation = HI"))
            .count(),
        n
    );
    assert!(r.trace.iter().all(|s| s.choice.is_none()));
    // Every step, including the sums, is a length in metres (the registry accepts any valid internal unit).
    assert!(
        r.trace.iter().all(|s| s.unit.as_deref() == Some("m")),
        "{:#?}",
        r.trace
    );
}

// --- Rise and fall -------------------------------------------------------------------------------

#[test]
fn rf_closed_form() {
    // Start 100; setup 1 rises 1.0, setup 2 falls 2.0.
    let r = call(RF, reduction(100.0, &[2.0, 1.0], &[1.0, 3.0])).expect("rf");
    close_all(&list(&r, "rises"), &[1.0, 0.0], 1e-12, "rises");
    close_all(&list(&r, "falls"), &[0.0, 2.0], 1e-12, "falls");
    close_all(&list(&r, "elevations"), &[101.0, 99.0], 1e-12, "elevations");
    close(scalar(&r, "sum_backsights"), 3.0, 1e-12, "sum BS");
    close(scalar(&r, "sum_foresights"), 4.0, 1e-12, "sum FS");
    close(scalar(&r, "net_rise"), -1.0, 1e-12, "net rise");
}

#[test]
fn rf_equal_readings_give_neither_rise_nor_fall() {
    let r = call(RF, reduction(5.0, &[1.25], &[1.25])).expect("flat");
    assert_eq!(list(&r, "rises"), [0.0]);
    assert_eq!(list(&r, "falls"), [0.0]);
    assert_eq!(list(&r, "elevations"), [5.0]);
}

#[test]
fn rf_trace_has_three_item_steps_per_setup_and_three_summary_steps() {
    let n = 4;
    let r = call(RF, reduction(0.0, &[1.0; 4], &[2.0, 0.5, 1.0, 3.0])).expect("rf");
    assert_eq!(r.trace.len(), 3 * n + 3);
    let per: Vec<_> = r.trace.iter().filter(|s| s.item.is_some()).collect();
    assert_eq!(per.len(), 3 * n);
    for expression in ["rise = ", "fall = ", "elevation = previous"] {
        assert_eq!(
            per.iter().filter(|s| s.step.contains(expression)).count(),
            n,
            "{expression}"
        );
    }
    assert!(
        r.trace.iter().all(|s| s.unit.as_deref() == Some("m")),
        "{:#?}",
        r.trace
    );
}

#[test]
fn rf_shares_the_reduction_errors() {
    let (code, input, index, message) = failure(RF, reduction(1.0, &[1.0, -1.0], &[1.0, 1.0]));
    assert_eq!(
        (code, input.as_deref(), index),
        (ErrorCode::OutOfDomain, Some("backsights"), Some(1))
    );
    assert!(message.contains("setup 2"), "{message}");
}

// --- Closure and adjustment ----------------------------------------------------------------------

#[test]
fn adjustment_closed_form() {
    // Two points; the run ends 0.04 too high over 400 m (100 m, then 300 m).
    let r = call(ADJ, adjustment(&[10.0, 20.04], &[100.0, 300.0], 20.0)).expect("adjust");
    close(scalar(&r, "closure_error"), 0.04, 1e-12, "closure error");
    close(scalar(&r, "total_length"), 400.0, 1e-12, "total length");
    close_all(
        &list(&r, "corrections"),
        &[-0.01, -0.04],
        1e-12,
        "corrections",
    );
    close_all(
        &list(&r, "adjusted_elevations"),
        &[9.99, 20.0],
        1e-12,
        "adjusted",
    );
}

#[test]
fn a_run_that_came_out_too_low_is_corrected_upwards() {
    let r = call(ADJ, adjustment(&[10.0, 19.96], &[100.0, 300.0], 20.0)).expect("low");
    close(scalar(&r, "closure_error"), -0.04, 1e-12, "closure error");
    assert!(list(&r, "corrections").iter().all(|c| *c > 0.0));
}

#[test]
fn zero_closure_needs_no_correction_and_a_loop_is_the_same_calculation() {
    // A loop: the closing elevation is the start elevation.
    let r = call(
        ADJ,
        adjustment(&[101.0, 99.5, 100.0], &[50.0, 70.0, 80.0], 100.0),
    )
    .expect("loop");
    assert_eq!(scalar(&r, "closure_error"), 0.0);
    assert!(list(&r, "corrections").iter().all(|c| *c == 0.0));
    close_all(
        &list(&r, "adjusted_elevations"),
        &[101.0, 99.5, 100.0],
        0.0,
        "unchanged",
    );
}

#[test]
fn adjustment_units_do_not_change_the_result() {
    let metric = call(ADJ, adjustment(&[3.048, 6.1], &[30.48, 60.96], 6.0)).expect("metres");
    let feet = call(
        ADJ,
        json!({
            "elevations": { "value": [10.0, 6.1 / 0.3048], "unit": "ft" },
            "section_lengths": { "value": [100.0, 200.0], "unit": "ft" },
            "closing_elevation": { "value": 6.0 / 0.3048, "unit": "ft" },
        }),
    )
    .expect("feet");
    close(
        scalar(&feet, "closure_error"),
        scalar(&metric, "closure_error"),
        1e-9,
        "closure",
    );
    close_all(
        &list(&feet, "adjusted_elevations"),
        &list(&metric, "adjusted_elevations"),
        1e-9,
        "adjusted",
    );
}

#[test]
fn adjustment_errors_name_the_point() {
    let (code, input, index, message) = failure(ADJ, adjustment(&[1.0, 2.0], &[10.0, 0.0], 2.0));
    assert_eq!(
        (code, input.as_deref(), index),
        (ErrorCode::OutOfDomain, Some("section_lengths"), Some(1))
    );
    assert!(message.contains("point 2"), "{message}");

    let (code, input, ..) = failure(ADJ, adjustment(&[1.0, 2.0, 3.0], &[10.0, 10.0], 3.0));
    assert_eq!(
        (code, input.as_deref()),
        (ErrorCode::InvalidValue, Some("section_lengths"))
    );

    let (code, input, ..) = failure(
        ADJ,
        json!({ "elevations": m(&[1.0]), "section_lengths": m(&[1.0]) }),
    );
    assert_eq!(
        (code, input.as_deref()),
        (ErrorCode::MissingInput, Some("closing_elevation"))
    );

    let (code, input, ..) = failure(
        ADJ,
        json!({
            "elevations": m(&[1.0]),
            "section_lengths": m(&[1.0]),
            "closing_elevation": { "value": 1.0 },
        }),
    );
    assert_eq!(
        (code, input.as_deref()),
        (ErrorCode::MissingUnit, Some("closing_elevation"))
    );
}

#[test]
fn a_total_length_that_overflows_is_out_of_domain_not_nan() {
    let (code, input, _, message) = failure(ADJ, adjustment(&[1.0, 2.0], &[1.0e308, 1.0e308], 2.0));
    assert_eq!(
        (code, input.as_deref()),
        (ErrorCode::OutOfDomain, Some("section_lengths"))
    );
    assert!(message.contains("too large"), "{message}");
}

#[test]
fn adjustment_trace_has_two_item_steps_per_point_and_two_summary_steps() {
    let n = 3;
    let r = call(ADJ, adjustment(&[1.0, 2.0, 3.0], &[10.0, 20.0, 30.0], 2.9)).expect("adjust");
    assert_eq!(r.trace.len(), 2 * n + 2);
    let per: Vec<_> = r.trace.iter().filter(|s| s.item.is_some()).collect();
    assert_eq!(per.len(), 2 * n);
    for step in &per {
        let item = step.item.as_ref().expect("item");
        assert_eq!(item.label, "point");
        assert!(
            step.step
                .starts_with(&format!("point {}: ", item.index + 1)),
            "{}",
            step.step
        );
    }
    let texts: Vec<&str> = r.trace.iter().map(|s| s.step.as_str()).collect();
    let pos = |needle: &str| texts.iter().position(|t| t.contains(needle)).expect(needle);
    assert!(pos("L = Σ") < pos("e = Z"));
    assert!(pos("e = Z") < pos("correction"));
    assert!(
        r.trace.iter().all(|s| s.unit.as_deref() == Some("m")),
        "{:#?}",
        r.trace
    );
}

// --- Chaining: the reduction feeds the adjustment (C4's shape) -------------------------------------------

#[test]
fn a_reduction_chains_into_an_adjustment() {
    let reduced = call(
        HI,
        reduction(
            133.163,
            &[6.659, 4.968, 4.508, 1.412, 7.073],
            &[4.971, 7.452, 5.857, 1.261, 1.785],
        ),
    )
    .expect("reduce");
    let z = list(&reduced, "elevations");
    let r = call(
        ADJ,
        adjustment(&z, &[440.0, 500.0, 620.0, 200.0, 380.0], 136.442),
    )
    .expect("adjust");
    close(scalar(&r, "closure_error"), 0.015, 1e-9, "closure error");
    close(
        list(&r, "adjusted_elevations")[4],
        136.442,
        1e-9,
        "last adjusted point is the known elevation",
    );
}

// --- Properties ----------------------------------------------------------------------------------

fn field_book() -> impl Strategy<Value = (f64, Vec<(f64, f64)>)> {
    (
        -500.0f64..2000.0,
        prop::collection::vec((0.0f64..5.0, 0.0f64..5.0), 1..12),
    )
}

fn split(book: &[(f64, f64)]) -> (Vec<f64>, Vec<f64>) {
    (
        book.iter().map(|b| b.0).collect(),
        book.iter().map(|b| b.1).collect(),
    )
}

proptest! {
    #[test]
    fn the_two_reductions_always_agree((start, book) in field_book()) {
        let (bs, fs) = split(&book);
        let a = call(HI, reduction(start, &bs, &fs)).expect("hi");
        let b = call(RF, reduction(start, &bs, &fs)).expect("rf");
        let (x, y) = (list(&a, "elevations"), list(&b, "elevations"));
        for k in 0..x.len() {
            prop_assert!((x[k] - y[k]).abs() <= 1e-9, "setup {}: {} vs {}", k + 1, x[k], y[k]);
        }
        for name in ["sum_backsights", "sum_foresights", "net_rise"] {
            prop_assert!((scalar(&a, name) - scalar(&b, name)).abs() <= 1e-12, "{}", name);
        }
    }

    #[test]
    fn the_last_elevation_is_the_start_plus_the_net_rise((start, book) in field_book()) {
        let (bs, fs) = split(&book);
        let r = call(HI, reduction(start, &bs, &fs)).expect("hi");
        let last = *list(&r, "elevations").last().expect("at least one setup");
        prop_assert!((last - (start + scalar(&r, "net_rise"))).abs() <= 1e-9);
        let (sb, sf): (f64, f64) = (bs.iter().sum(), fs.iter().sum());
        prop_assert!((scalar(&r, "net_rise") - (sb - sf)).abs() <= 1e-9);
    }

    #[test]
    fn rises_and_falls_are_never_negative_and_net_to_the_difference((start, book) in field_book()) {
        let (bs, fs) = split(&book);
        let r = call(RF, reduction(start, &bs, &fs)).expect("rf");
        let (up, down) = (list(&r, "rises"), list(&r, "falls"));
        for k in 0..up.len() {
            prop_assert!(up[k] >= 0.0 && down[k] >= 0.0);
            prop_assert!(up[k] == 0.0 || down[k] == 0.0, "a setup cannot both rise and fall");
            prop_assert!((up[k] - down[k] - (bs[k] - fs[k])).abs() <= 1e-12);
        }
    }

    #[test]
    fn shifting_the_start_shifts_every_elevation((start, book) in field_book(), shift in -100.0f64..100.0) {
        let (bs, fs) = split(&book);
        let a = call(HI, reduction(start, &bs, &fs)).expect("a");
        let b = call(HI, reduction(start + shift, &bs, &fs)).expect("b");
        let (x, y) = (list(&a, "elevations"), list(&b, "elevations"));
        for k in 0..x.len() {
            prop_assert!((y[k] - x[k] - shift).abs() <= 1e-9);
        }
    }

    #[test]
    fn the_adjusted_run_ends_on_the_known_elevation(
        pts in prop::collection::vec((-100.0f64..500.0, 1.0f64..400.0), 1..10),
        closing in -100.0f64..500.0,
    ) {
        let z: Vec<f64> = pts.iter().map(|p| p.0).collect();
        let s: Vec<f64> = pts.iter().map(|p| p.1).collect();
        let r = call(ADJ, adjustment(&z, &s, closing)).expect("adjust");
        let adjusted = list(&r, "adjusted_elevations");
        prop_assert!((adjusted[adjusted.len() - 1] - closing).abs() <= 1e-9);
        let corrections = list(&r, "corrections");
        let e = scalar(&r, "closure_error");
        prop_assert!((corrections[corrections.len() - 1] + e).abs() <= 1e-9, "the last correction is minus the error");
        // Proportional to the distance run: c_k / S_k is the same for every point.
        let total = scalar(&r, "total_length");
        let mut cumulative = 0.0;
        for k in 0..s.len() {
            cumulative += s[k];
            prop_assert!((corrections[k] + e * cumulative / total).abs() <= 1e-9, "point {}", k + 1);
        }
    }

    #[test]
    fn adjustment_ignores_a_common_offset_and_a_common_length_scale(
        pts in prop::collection::vec((-100.0f64..500.0, 1.0f64..400.0), 1..10),
        closing in -100.0f64..500.0,
        offset in -1000.0f64..1000.0,
        scale in 0.1f64..10.0,
    ) {
        let z: Vec<f64> = pts.iter().map(|p| p.0).collect();
        let s: Vec<f64> = pts.iter().map(|p| p.1).collect();
        let base = call(ADJ, adjustment(&z, &s, closing)).expect("base");
        let z2: Vec<f64> = z.iter().map(|v| v + offset).collect();
        let s2: Vec<f64> = s.iter().map(|v| v * scale).collect();
        let moved = call(ADJ, adjustment(&z2, &s2, closing + offset)).expect("moved");
        let (a, b) = (list(&base, "corrections"), list(&moved, "corrections"));
        for k in 0..a.len() {
            prop_assert!((a[k] - b[k]).abs() <= 1e-8, "correction {}", k + 1);
        }
    }
}
