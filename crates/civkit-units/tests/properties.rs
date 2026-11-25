// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 15:36 IST · last edited 2026-11-25 15:36 IST

//! Property-based tests: invariants that must hold for every input, not just chosen examples.

use civkit_units::{Dimension, Unit, bearing_to_azimuth, dms_to_radians, table::NAMED};
use proptest::prelude::*;

/// Named units usable in compounds (`%` cannot be combined).
fn names() -> Vec<&'static str> {
    NAMED
        .iter()
        .map(|(n, _, _)| *n)
        .filter(|n| *n != "%")
        .collect()
}

fn name() -> impl Strategy<Value = &'static str> {
    proptest::sample::select(names())
}

fn exponent() -> impl Strategy<Value = i8> {
    prop_oneof![Just(1i8), -3i8..=4].prop_filter("non-zero", |e| *e != 0)
}

/// Two named units of the same dimension.
fn same_dimension_pair() -> impl Strategy<Value = (&'static str, &'static str)> {
    let pairs: Vec<(&'static str, &'static str)> = NAMED
        .iter()
        .flat_map(|(a, _, da)| {
            NAMED
                .iter()
                .filter(move |(_, _, db)| db == da)
                .map(move |(b, _, _)| (*a, *b))
        })
        .collect();
    proptest::sample::select(pairs)
}

fn term_text(name: &str, e: i8) -> String {
    if e == 1 {
        name.to_owned()
    } else {
        format!("{name}^{e}")
    }
}

/// A random valid compound: up to three numerator terms and an optional single denominator term,
/// together with its factor and dimension computed independently from the table.
fn compound() -> impl Strategy<Value = (String, f64, Dimension)> {
    (
        proptest::collection::vec((name(), exponent()), 1..=3),
        proptest::option::of((name(), 1i8..=3)),
    )
        .prop_map(|(num, den)| {
            let lookup = |n: &str| {
                NAMED
                    .iter()
                    .find(|(x, _, _)| *x == n)
                    .map(|(_, f, d)| (*f, *d))
                    .expect("named")
            };
            let mut text = Vec::new();
            let (mut factor, mut dim) = (1.0f64, Dimension::DIMENSIONLESS);
            for (n, e) in &num {
                let (f, d) = lookup(n);
                text.push(term_text(n, *e));
                factor *= f.powi(i32::from(*e));
                dim = dim
                    .checked_mul(d.checked_powi(*e).expect("small"))
                    .expect("small");
            }
            let mut s = text.join("*");
            if let Some((n, e)) = den {
                let (f, d) = lookup(n);
                s = format!("{s}/{}", term_text(n, e));
                factor /= f.powi(i32::from(e));
                dim = dim
                    .checked_div(d.checked_powi(e).expect("small"))
                    .expect("small");
            }
            (s, factor, dim)
        })
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 2000, ..ProptestConfig::default() })]

    /// Every grammatical compound parses to the factor and dimension implied by the table.
    #[test]
    fn compounds_parse_to_their_algebra((text, factor, dim) in compound()) {
        let unit = Unit::parse(&text).expect("valid compound");
        prop_assert_eq!(unit.dimension, dim);
        prop_assert!((unit.factor - factor).abs() <= 1e-12 * factor.abs(), "{} {} {}", text, unit.factor, factor);
        // '·' is equivalent to '*'
        let dot = Unit::parse(&text.replace('*', "·")).expect("valid with ·");
        prop_assert_eq!(dot, unit);
    }

    /// Converting to SI and back returns the original value.
    #[test]
    fn round_trip_through_si((text, _, _) in compound(), value in -1e9f64..1e9) {
        let unit = Unit::parse(&text).expect("valid");
        let back = unit.from_si(unit.to_si(value));
        prop_assert!((back - value).abs() <= 1e-12 * value.abs().max(1e-300), "{} {} {}", text, value, back);
    }

    /// Converting between two units of the same dimension and back returns the original value.
    #[test]
    fn round_trip_between_units((a, b) in same_dimension_pair(), value in -1e6f64..1e6) {
        let (ua, ub) = (Unit::parse(a).expect("a"), Unit::parse(b).expect("b"));
        prop_assert_eq!(ua.dimension, ub.dimension);
        let in_b = ub.from_si(ua.to_si(value));
        let back = ua.from_si(ub.to_si(in_b));
        prop_assert!((back - value).abs() <= 1e-12 * value.abs().max(1e-300));
    }

    /// The parser never panics, whatever the input.
    #[test]
    fn parser_never_panics(text in "\\PC{0,24}") {
        let _ = Unit::parse(&text);
        let _ = bearing_to_azimuth(&text);
    }

    /// Strings near the grammar never panic and never parse to a unit with a non-finite factor.
    #[test]
    fn near_misses_never_panic(text in "[a-zA-Z_%·*/^0-9 .-]{0,16}") {
        if let Ok(unit) = Unit::parse(&text) {
            prop_assert!(unit.factor.is_finite() && unit.factor > 0.0);
        }
    }

    /// DMS → radians matches the decimal-degree value, and the sign flag only flips the sign.
    #[test]
    fn dms_round_trip(d in 0u32..720, m in 0u32..60, s in 0.0f64..60.0) {
        let rad = dms_to_radians(&[f64::from(d), f64::from(m), s], false).expect("valid");
        let expected = f64::from(d) + f64::from(m) / 60.0 + s / 3600.0;
        prop_assert!((rad.to_degrees() - expected).abs() < 1e-9);
        let neg = dms_to_radians(&[f64::from(d), f64::from(m), s], true).expect("valid");
        prop_assert_eq!(neg.to_bits(), (-rad).to_bits());
    }

    /// Bearing → azimuth → bearing reproduces the quadrant and angle.
    #[test]
    fn bearing_round_trip(ns in prop_oneof![Just('N'), Just('S')], ew in prop_oneof![Just('E'), Just('W')],
                          d in 0u32..90, m in 0u32..60, s in 0u32..60) {
        let text = format!("{ns} {d}°{m}'{s}\" {ew}");
        let az = bearing_to_azimuth(&text).expect("valid").to_degrees();
        prop_assert!((0.0..360.0).contains(&az), "{} -> {}", text, az);
        let angle = f64::from(d) + f64::from(m) / 60.0 + f64::from(s) / 3600.0;
        // invert: azimuth -> quadrant bearing
        let (q_ns, q_ew, q_angle) = if az <= 90.0 { ('N', 'E', az) }
            else if az <= 180.0 { ('S', 'E', 180.0 - az) }
            else if az <= 270.0 { ('S', 'W', az - 180.0) }
            else { ('N', 'W', 360.0 - az) };
        prop_assert!((q_angle - angle).abs() < 1e-9, "{} -> {} -> {}", text, az, q_angle);
        // quadrant letters must match except on the axes, where two spellings name one direction
        if angle > 1e-9 && angle < 90.0 - 1e-9 {
            prop_assert_eq!((q_ns, q_ew), (ns, ew));
        }
    }
}
