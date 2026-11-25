// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 18:37 IST · last edited 2026-11-25 18:37 IST

//! Lengths of polylines, arcs and circles (docs/GEOMETRY.md §9, §8.2, §8.8).
//!
//! Expected values are closed forms or 60-digit values computed outside the repository with mpmath
//! (script kept with the review notes, not in the repository). They are never produced by this crate.

use std::f64::consts::{FRAC_PI_2, PI, TAU};

use civkit_geometry::{
    Arc, Circle, GeometryError, Note, Point, Vertex, arc_length, circle_area, circle_circumference,
    polyline_length,
};

fn close_rel(actual: f64, expected: f64, rel: f64) {
    let scale = expected.abs().max(f64::MIN_POSITIVE);
    assert!(
        ((actual - expected) / scale).abs() <= rel,
        "expected {expected:e}, got {actual:e} (relative tolerance {rel:e})"
    );
}

/// Exact equality, for values every step of which is exact.
fn exact(actual: f64, expected: f64) {
    assert_eq!(
        actual.to_bits(),
        expected.to_bits(),
        "expected {expected:e}, got {actual:e}"
    );
}

fn v(x: f64, y: f64) -> Vertex {
    Vertex::straight(x, y)
}

fn vb(x: f64, y: f64, bulge: f64) -> Vertex {
    Vertex::new(x, y, bulge)
}

#[test]
fn straight_open_polyline() {
    let l = polyline_length(&[v(0.0, 0.0), v(3.0, 0.0), v(3.0, 4.0)], false, None)
        .expect("valid input");
    exact(l.value, 7.0);
    assert!(l.notes.is_empty());
}

#[test]
fn f1_boundary_perimeter_is_200() {
    let ring = [
        v(0.0, 0.0),
        v(60.0, 0.0),
        v(60.0, 30.0),
        v(40.0, 45.0),
        v(0.0, 45.0),
    ];
    let l = polyline_length(&ring, true, None).expect("valid input");
    close_rel(l.value, 200.0, 1e-15);
    // Tolerance of fixture F1 (§5 example): D = 75, eps = 7.5e-8.
    close_rel(l.tolerance.size, 75.0, 1e-15); // hypot: precision is platform-dependent
    close_rel(l.tolerance.linear, 7.5e-8, 1e-15);
    close_rel(l.tolerance.area, 7.5e-8 * 75.0, 1e-15);
    assert!(!l.tolerance.explicit);
}

#[test]
fn f1_road_centreline_is_100_plus_10_pi() {
    // Fixture F1, layer ROAD_CL: straight 60, quarter arc R = 20 (bulge as stored in the file), straight 40.
    let road = [
        v(-10.0, -8.0),
        vb(50.0, -8.0, 0.414213562373095),
        v(70.0, 12.0),
        v(70.0, 52.0),
    ];
    let l = polyline_length(&road, false, None).expect("valid input");
    // mpmath, 60 digits, with the stored (rounded) bulge: 131.41592653589793167.
    close_rel(l.value, 131.415_926_535_897_93, 1e-15);
    close_rel(l.value, 100.0 + 10.0 * PI, 1e-14);
}

#[test]
fn f1_drain_and_water() {
    // DRAIN: arc centre (10, 30), R = 5, 0 -> 180 degrees, plus a line of 25.
    let arc = Arc::new(Point::new(10.0, 30.0), 5.0, 0.0, PI);
    let drain = arc_length(&arc, None).expect("valid input").value + 25.0;
    close_rel(drain, 40.707_963_267_948_97, 1e-15);
    // WATER: circle R = 3.
    let c = Circle::new(Point::new(50.0, 20.0), 3.0);
    close_rel(
        circle_circumference(&c, None).expect("valid input").value,
        6.0 * PI,
        1e-15,
    );
    close_rel(
        circle_area(&c, None).expect("valid input").value,
        9.0 * PI,
        1e-15,
    );
}

#[test]
fn quarter_arc_and_clockwise_arc() {
    let q = Arc::new(Point::new(0.0, 0.0), 20.0, 0.0, FRAC_PI_2);
    close_rel(
        arc_length(&q, None).expect("valid input").value,
        10.0 * PI,
        1e-15,
    );
    let cw = Arc::new(Point::new(0.0, 0.0), 20.0, PI, -FRAC_PI_2);
    close_rel(
        arc_length(&cw, None).expect("valid input").value,
        10.0 * PI,
        1e-15,
    );
}

/// (bulge, arc length, signed circular-segment area) for chord 10; mpmath, 60 digits.
pub const BULGE_ORACLE: [(f64, f64, f64); 15] = [
    (1e-15, 10.0, 3.333_333_333_333_333_3e-14),
    (1e-9, 10.000_000_000_000_000, 3.333_333_333_333_333_3e-8),
    (1e-7, 10.000_000_000_000_067, 3.333_333_333_333_34e-6),
    (1e-5, 10.000_000_000_666_667, 3.333_333_333_4e-4),
    (1e-3, 10.000_006_666_665_333, 3.333_333_999_999_904_8e-2),
    (0.1, 10.066_533_901_607_365, 3.339_990_507_792_980_2),
    (
        0.414_213_562_373_095_03,
        11.107_207_345_395_916,
        14.269_908_169_872_415,
    ),
    (1.0, 15.707_963_267_948_966, 39.269_908_169_872_415),
    (10.0, 148.583_895_104_677_2, 1_999.621_675_696_549_6),
    (1e3, 15_697.978_969_245_569, 19_634_993.321_530_673),
    (1e6, 15_707_953.267_964_674, 19_634_954_084_975.478),
    (1e8, 1_570_796_316.794_896_8, 1.963_495_408_493_621e17),
    (1e12, 15_707_963_267_938.966, 1.963_495_408_493_620_8e25),
    (-1e-7, 10.000_000_000_000_067, -3.333_333_333_333_34e-6),
    (-1.0, 15.707_963_267_948_966, -39.269_908_169_872_415),
];

#[test]
fn bulged_segment_length_matches_high_precision_oracle() {
    // An explicit tolerance of 0 tests the formula alone: at b = 1e12 the default tolerance (5e3)
    // would rightly make this 10 m chord ill-conditioned (§8.2a).
    for (b, s, _) in BULGE_ORACLE {
        let l = polyline_length(&[vb(0.0, 0.0, b), v(10.0, 0.0)], false, Some(0.0))
            .expect("valid input");
        close_rel(l.value, s, 1e-13);
    }
}

#[test]
fn closed_polyline_uses_the_closing_bulge() {
    // Two vertices, two semicircles: a full circle of diameter 2 (AutoCAD DONUT style).
    let l =
        polyline_length(&[vb(0.0, 0.0, 1.0), vb(2.0, 0.0, 1.0)], true, None).expect("valid input");
    close_rel(l.value, TAU, 1e-15);
}

#[test]
fn open_polyline_ignores_the_last_bulge_but_checks_it() {
    let l = polyline_length(&[v(0.0, 0.0), vb(1.0, 0.0, 5.0)], false, None).expect("valid input");
    exact(l.value, 1.0);
    assert_eq!(
        polyline_length(&[v(0.0, 0.0), vb(1.0, 0.0, f64::NAN)], false, None),
        Err(GeometryError::NonFinite)
    );
}

#[test]
fn explicit_closure_is_not_reported() {
    let ring = [v(0.0, 0.0), v(1.0, 0.0), v(1.0, 1.0), v(0.0, 0.0)];
    let l = polyline_length(&ring, true, None).expect("valid input");
    close_rel(l.value, 3.414_213_562_373_095, 1e-15);
    assert!(l.notes.is_empty());
}

#[test]
fn zero_length_segments_are_reported_and_keep_their_length() {
    let l = polyline_length(&[v(0.0, 0.0), v(0.0, 0.0), v(1.0, 0.0)], false, None)
        .expect("valid input");
    exact(l.value, 1.0);
    assert_eq!(l.notes, vec![Note::ZeroLengthSegment { segment: 0 }]);
    // A sub-tolerance segment contributes its computed length: nothing is rounded away (D8).
    let l = polyline_length(&[v(0.0, 0.0), v(1e-12, 0.0), v(1.0, 0.0)], false, None)
        .expect("valid input");
    close_rel(l.value, 1.0, 1e-15);
    assert_eq!(l.notes, vec![Note::ZeroLengthSegment { segment: 0 }]);
    // c = 0 exactly with a bulge: radius 0, so the segment is zero-length, not an error.
    let l = polyline_length(&[vb(0.0, 0.0, 5.0), v(0.0, 0.0), v(1.0, 0.0)], false, None)
        .expect("valid input");
    exact(l.value, 1.0);
}

#[test]
fn tiny_chord_with_huge_bulge_is_an_error() {
    // c = 1e-12 with b = 1e15 is an arc of radius 250: not zero-length (§8.2a).
    let p = [vb(0.0, 0.0, 1e15), v(1e-12, 0.0), v(5.0, 5.0)];
    assert_eq!(
        polyline_length(&p, false, None),
        Err(GeometryError::IllConditionedBulge { segment: 0 })
    );
}

#[test]
fn too_few_vertices() {
    assert_eq!(
        polyline_length(&[v(0.0, 0.0)], false, None),
        Err(GeometryError::TooFewVertices {
            found: 1,
            required: 2
        })
    );
    assert_eq!(
        polyline_length(&[], true, None),
        Err(GeometryError::TooFewVertices {
            found: 0,
            required: 2
        })
    );
}

#[test]
fn long_polylines_do_not_accumulate_rounding() {
    let pts: Vec<Vertex> = (0..=100_000).map(|i| v(f64::from(i) * 0.1, 0.0)).collect();
    let l = polyline_length(&pts, false, None).expect("valid input");
    close_rel(l.value, 10_000.0, 1e-13);
}

#[test]
fn arc_domain() {
    let c = Point::new(0.0, 0.0);
    let zero = arc_length(&Arc::new(c, 5.0, 1.0, 0.0), None).expect("valid input");
    exact(zero.value, 0.0);
    assert_eq!(zero.notes, vec![Note::ZeroSweep]);
    assert_eq!(
        arc_length(&Arc::new(c, 5.0, 0.0, TAU), None),
        Err(GeometryError::SweepOutOfRange)
    );
    assert_eq!(
        arc_length(&Arc::new(c, 5.0, 0.0, -7.0), None),
        Err(GeometryError::SweepOutOfRange)
    );
    assert_eq!(
        arc_length(&Arc::new(c, -1.0, 0.0, 1.0), None),
        Err(GeometryError::NegativeRadius)
    );
    assert_eq!(
        arc_length(&Arc::new(c, 0.0, 0.0, 1.0), None),
        Err(GeometryError::RadiusTooSmall)
    );
    assert_eq!(
        arc_length(&Arc::new(c, f64::NAN, 0.0, 1.0), None),
        Err(GeometryError::NonFinite)
    );
}

#[test]
fn circle_domain() {
    let c = Point::new(3.0, 4.0);
    assert_eq!(
        circle_area(&Circle::new(c, 0.0), None),
        Err(GeometryError::RadiusTooSmall)
    );
    assert_eq!(
        circle_area(&Circle::new(c, -2.0), None),
        Err(GeometryError::NegativeRadius)
    );
    assert_eq!(
        circle_area(&Circle::new(c, 1e200), None),
        Err(GeometryError::Overflow)
    );
}
