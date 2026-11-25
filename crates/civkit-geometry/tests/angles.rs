// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 18:37 IST · last edited 2026-11-25 18:37 IST

//! Angle conventions, direction and components (docs/GEOMETRY.md §4, §7).
//! Expected values are closed forms; nothing here is computed by the code under test.

use std::f64::consts::{FRAC_PI_2, PI, TAU};

use civkit_geometry::{
    GeometryError, Point, azimuth_from_components, azimuth_from_direction, components, direction,
    direction_from_azimuth, normalize_angle,
};

fn close(actual: f64, expected: f64, abs: f64) {
    assert!(
        (actual - expected).abs() <= abs,
        "expected {expected:e}, got {actual:e} (tolerance {abs:e})"
    );
}

fn is_positive_zero(x: f64) -> bool {
    x.to_bits() == 0.0_f64.to_bits()
}

#[test]
fn tiny_negative_angles_wrap_to_zero_not_tau() {
    // rem_euclid returns exactly TAU for these inputs; the rule turns that into 0 (§4).
    for a in [-1e-17, -1e-16, -4e-16, -1e-300] {
        let r = normalize_angle(a).expect("valid input");
        assert!(is_positive_zero(r), "normalise({a:e}) = {r:e}");
    }
}

#[test]
fn negative_zero_becomes_positive_zero() {
    assert!(is_positive_zero(
        normalize_angle(-0.0).expect("valid input")
    ));
    assert!(is_positive_zero(
        azimuth_from_components(-0.0, 1.0).expect("valid input")
    ));
}

#[test]
fn normalisation_of_ordinary_values() {
    assert!(is_positive_zero(normalize_angle(TAU).expect("valid input")));
    assert_eq!(
        normalize_angle(-PI).expect("valid input").to_bits(),
        PI.to_bits()
    );
    close(
        normalize_angle(7.0 * TAU + 1.0).expect("valid input"),
        1.0,
        1e-13,
    );
    // A value just below TAU is a legitimate angle and is not snapped (no angular tolerance).
    let just_below = f64::from_bits(TAU.to_bits() - 1);
    assert_eq!(
        normalize_angle(just_below).expect("valid input").to_bits(),
        just_below.to_bits()
    );
}

#[test]
fn non_finite_angles_are_rejected() {
    for a in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(normalize_angle(a), Err(GeometryError::NonFinite));
        assert_eq!(azimuth_from_direction(a), Err(GeometryError::NonFinite));
        assert_eq!(direction_from_azimuth(a), Err(GeometryError::NonFinite));
    }
}

#[test]
fn direction_angle_and_azimuth_convert_both_ways() {
    // East: theta 0 <-> azimuth pi/2; North: theta pi/2 <-> azimuth 0.
    assert_eq!(
        azimuth_from_direction(0.0).expect("valid input").to_bits(),
        FRAC_PI_2.to_bits()
    );
    assert!(is_positive_zero(
        azimuth_from_direction(FRAC_PI_2).expect("valid input")
    ));
    close(
        azimuth_from_direction(PI).expect("valid input"),
        3.0 * FRAC_PI_2,
        1e-15,
    );
    close(
        direction_from_azimuth(PI).expect("valid input"),
        3.0 * FRAC_PI_2,
        1e-15,
    );
    close(
        direction_from_azimuth(3.0 * FRAC_PI_2).expect("valid input"),
        PI,
        1e-15,
    );
}

#[test]
fn azimuth_from_components_uses_east_first() {
    assert!(is_positive_zero(
        azimuth_from_components(0.0, 1.0).expect("valid input")
    )); // north
    close(
        azimuth_from_components(1.0, 0.0).expect("valid input"),
        FRAC_PI_2,
        1e-15,
    ); // east
    close(
        azimuth_from_components(0.0, -1.0).expect("valid input"),
        PI,
        1e-15,
    ); // south
    close(
        azimuth_from_components(-0.0, -1.0).expect("valid input"),
        PI,
        1e-15,
    ); // south, from the other side
    close(
        azimuth_from_components(-1.0, 0.0).expect("valid input"),
        3.0 * FRAC_PI_2,
        1e-15,
    ); // west
    close(
        azimuth_from_components(1.0, 1.0).expect("valid input"),
        PI / 4.0,
        1e-15,
    ); // north-east
    // Slightly west of north: atan2 gives a tiny negative number, which wraps to 0, never 2*pi.
    assert!(is_positive_zero(
        azimuth_from_components(-1e-300, 1.0).expect("valid input")
    ));
}

#[test]
fn direction_from_zero_components_is_undefined() {
    assert_eq!(
        azimuth_from_components(0.0, 0.0),
        Err(GeometryError::UndefinedDirection)
    );
    assert_eq!(
        azimuth_from_components(-0.0, 0.0),
        Err(GeometryError::UndefinedDirection)
    );
    // Any non-zero component defines a direction; whether it is "zero" in engineering terms is the caller's call (§7).
    assert!(azimuth_from_components(1e-300, 0.0).is_ok());
}

#[test]
fn components_from_azimuth() {
    let (dep, lat) = components(0.0, 100.0).expect("valid input");
    assert!(is_positive_zero(dep));
    assert_eq!(lat.to_bits(), 100.0_f64.to_bits());
    // Due south: the departure is not an exact zero (sin(pi) != 0 in f64) — tests must use a tolerance.
    let (dep, lat) = components(PI, 100.0).expect("valid input");
    close(dep, 0.0, 1e-13);
    close(lat, -100.0, 1e-13);
    // Negative zero is returned as +0.
    let (dep, lat) = components(PI, 0.0).expect("valid input");
    assert!(is_positive_zero(dep) && is_positive_zero(lat));
    assert_eq!(components(0.0, -1.0), Err(GeometryError::NegativeDistance));
    assert_eq!(components(f64::NAN, 1.0), Err(GeometryError::NonFinite));
    assert_eq!(
        components(0.0, f64::INFINITY),
        Err(GeometryError::NonFinite)
    );
}

#[test]
fn direction_between_points() {
    let d = direction(Point::new(0.0, 0.0), Point::new(60.0, 0.0), None).expect("valid input");
    close(d.azimuth, FRAC_PI_2, 1e-15);
    assert!(is_positive_zero(d.theta));
    assert_eq!(d.distance.to_bits(), 60.0_f64.to_bits());
    // Two-point rule (§7): eps = max(1e-9 * d, 1024 * ulp(M)); here 6e-8.
    close(d.tolerance, 6e-8, 1e-22);

    let d = direction(Point::new(10.0, 10.0), Point::new(7.0, 6.0), None).expect("valid input");
    close(d.distance, 5.0, 1e-15); // hypot: precision is platform-dependent
    close(d.azimuth, (-3.0_f64).atan2(-4.0) + TAU, 1e-15);
}

#[test]
fn coincident_points_have_no_direction() {
    let p = Point::new(1.0, 1.0);
    assert_eq!(direction(p, p, None), Err(GeometryError::CoincidentPoints));
    // 1e-14 apart near magnitude 1: below 1024 * ulp(1) = 2.3e-13, so undefined.
    assert_eq!(
        direction(p, Point::new(1.0 + 1e-14, 1.0), None),
        Err(GeometryError::CoincidentPoints)
    );
    // 1e-12 apart: above the precision floor, so the direction is defined (the size term never decides).
    assert!(direction(p, Point::new(1.0 + 1e-12, 1.0), None).is_ok());
}

#[test]
fn explicit_tolerance_for_direction() {
    let a = Point::new(0.0, 0.0);
    let b = Point::new(1e-4, 0.0);
    assert_eq!(
        direction(a, b, Some(1e-3)),
        Err(GeometryError::CoincidentPoints)
    );
    assert_eq!(
        direction(a, b, Some(1e-6))
            .expect("valid input")
            .tolerance
            .to_bits(),
        1e-6_f64.to_bits()
    );
    assert_eq!(
        direction(a, b, Some(-1.0)),
        Err(GeometryError::InvalidTolerance)
    );
    assert_eq!(
        direction(a, b, Some(f64::NAN)),
        Err(GeometryError::InvalidTolerance)
    );
    assert_eq!(
        direction(Point::new(f64::NAN, 0.0), b, None),
        Err(GeometryError::NonFinite)
    );
}
