// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 18:37 IST · last edited 2026-11-25 18:37 IST

//! Polygon area, orientation and centroid (docs/GEOMETRY.md §10).
//!
//! Expected values are closed forms, exact rationals, or 60-digit values computed outside the
//! repository with mpmath. They are never produced by this crate.

use std::f64::consts::{FRAC_PI_2, PI};

use civkit_geometry::{GeometryError, Note, Orientation, Vertex, polygon_area, polygon_centroid};

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

fn ring(points: &[(f64, f64)]) -> Vec<Vertex> {
    points
        .iter()
        .map(|&(x, y)| Vertex::straight(x, y))
        .collect()
}

fn shifted(points: &[(f64, f64)], dx: f64, dy: f64, k: f64) -> Vec<Vertex> {
    points
        .iter()
        .map(|&(x, y)| Vertex::straight(x * k + dx, y * k + dy))
        .collect()
}

const F1: [(f64, f64); 5] = [
    (0.0, 0.0),
    (60.0, 0.0),
    (60.0, 30.0),
    (40.0, 45.0),
    (0.0, 45.0),
];

#[test]
fn f1_boundary_area_and_orientation() {
    let a = polygon_area(&ring(&F1), None).expect("valid input");
    exact(a.area, 2550.0);
    exact(a.signed_area, 2550.0);
    assert_eq!(a.orientation, Orientation::CounterClockwise);
    assert!(a.notes.is_empty());

    let mut rev = F1;
    rev.reverse();
    let a = polygon_area(&ring(&rev), None).expect("valid input");
    exact(a.area, 2550.0);
    exact(a.signed_area, -2550.0);
    assert_eq!(a.orientation, Orientation::Clockwise);
}

#[test]
fn far_from_the_origin_the_area_is_exact() {
    // State plane in metres, the same plan in millimetres, and a guard that fails without translation.
    let area = |dx: f64, dy: f64, k: f64| {
        polygon_area(&shifted(&F1, dx, dy, k), None)
            .expect("valid input")
            .area
    };
    exact(area(5e5, 4e6, 1.0), 2550.0);
    exact(area(5e8, 4e9, 1000.0), 2.55e9);
    exact(area(5e11, 4e12, 1.0), 2550.0);
    assert!(
        polygon_area(&shifted(&F1, 5e5, 4e6, 1.0), None)
            .expect("valid input")
            .notes
            .is_empty()
    );
}

#[test]
fn f1_centroid_is_exact_rational() {
    // Exact: (1460/51, 365/17).
    let cx = 1460.0 / 51.0;
    let cy = 365.0 / 17.0;
    let c = polygon_centroid(&ring(&F1), None).expect("valid input");
    close_rel(c.point.x, cx, 1e-15);
    close_rel(c.point.y, cy, 1e-15);
    exact(c.signed_area, 2550.0);

    // Clockwise ring: the same centroid. Using |A| here would reflect it through the first vertex.
    let mut rev = F1;
    rev.reverse();
    let c = polygon_centroid(&ring(&rev), None).expect("valid input");
    close_rel(c.point.x, cx, 1e-15);
    close_rel(c.point.y, cy, 1e-15);

    // At state-plane coordinates the translated-back centroid is limited only by ulp(4e6).
    let c = polygon_centroid(&shifted(&F1, 5e5, 4e6, 1.0), None).expect("valid input");
    assert!((c.point.x - (5e5 + cx)).abs() <= 1e-9);
    assert!((c.point.y - (4e6 + cy)).abs() <= 1e-9);
}

#[test]
fn square_with_a_semicircular_edge() {
    let sq = |b: f64| {
        vec![
            Vertex::new(0.0, 0.0, b),
            Vertex::straight(2.0, 0.0),
            Vertex::straight(2.0, 2.0),
            Vertex::straight(0.0, 2.0),
        ]
    };
    close_rel(
        polygon_area(&sq(1.0), None).expect("valid input").area,
        4.0 + FRAC_PI_2,
        1e-15,
    );
    close_rel(
        polygon_area(&sq(-1.0), None).expect("valid input").area,
        4.0 - FRAC_PI_2,
        1e-15,
    );
    // mpmath: 4.5707963267948966192 for b = tan(pi/8).
    close_rel(
        polygon_area(&sq(0.414_213_562_373_095_03), None)
            .expect("valid input")
            .area,
        4.570_796_326_794_897,
        1e-15,
    );
}

#[test]
fn two_vertex_bulged_rings_are_valid() {
    // Full circle of diameter 2 (AutoCAD DONUT style) and a half disc.
    let full = [Vertex::new(0.0, 0.0, 1.0), Vertex::new(2.0, 0.0, 1.0)];
    let a = polygon_area(&full, None).expect("valid input");
    close_rel(a.area, PI, 1e-15);
    assert_eq!(a.orientation, Orientation::CounterClockwise);
    let half = [Vertex::new(0.0, 0.0, 1.0), Vertex::straight(2.0, 0.0)];
    close_rel(
        polygon_area(&half, None).expect("valid input").area,
        FRAC_PI_2,
        1e-15,
    );
    // Two major arcs (b = -2) on opposite sides: the circles cross only at the shared vertices.
    // mpmath: 210.4919871553266411.
    let lens = [Vertex::new(0.0, 0.0, -2.0), Vertex::new(10.0, 0.0, -2.0)];
    let a = polygon_area(&lens, None).expect("valid input");
    close_rel(a.area, 210.491_987_155_326_64, 1e-14);
    assert_eq!(a.orientation, Orientation::Clockwise);
}

#[test]
fn circular_segment_area_matches_high_precision_oracle() {
    // A chord of 10 closed by a bulged edge: the area is the circular segment alone.
    // (bulge, signed segment area) from the oracle table, for bulges that make a valid ring.
    let cases = [
        (1e-7, 3.333_333_333_333_34e-6),
        (1e-5, 3.333_333_333_4e-4),
        (1e-3, 3.333_333_999_999_904_8e-2),
        (0.1, 3.339_990_507_792_980_2),
        (1.0, 39.269_908_169_872_415),
        (10.0, 1_999.621_675_696_549_6),
        (1e3, 19_634_993.321_530_673),
        (1e6, 19_634_954_084_975.478),
        (1e8, 1.963_495_408_493_621e17),
        (-1e-7, -3.333_333_333_333_34e-6),
        (-1.0, -39.269_908_169_872_415),
    ];
    for (b, area) in cases {
        let r = [Vertex::new(0.0, 0.0, b), Vertex::straight(10.0, 0.0)];
        let a = polygon_area(&r, None).expect("valid input");
        close_rel(a.signed_area, area, 1e-13);
        let expected = if b > 0.0 {
            Orientation::CounterClockwise
        } else {
            Orientation::Clockwise
        };
        assert_eq!(a.orientation, expected, "bulge {b}");
    }
}

#[test]
fn sliver_rings_are_degenerate_and_keep_their_computed_area() {
    // Sagitta 5e-9 and 5e-15 are below eps = 1e-8: the ring lies within 2 eps of one line (§8.6).
    // The area is returned as computed (D8), checked against mpmath.
    for (b, area) in [
        (1e-9, 3.333_333_333_333_333_3e-8),
        (1e-15, 3.333_333_333_333_333_3e-14),
    ] {
        let r = [Vertex::new(0.0, 0.0, b), Vertex::straight(10.0, 0.0)];
        let a = polygon_area(&r, None).expect("valid input");
        close_rel(a.area, area, 1e-13);
        assert_eq!(a.orientation, Orientation::Undefined);
        assert_eq!(a.notes, vec![Note::DegenerateRing]);
    }
}

#[test]
fn huge_bulge_on_a_short_chord_is_ill_conditioned() {
    // b = 1e12 on a chord of 10: radius 2.5e12, so eps (5e3) exceeds the chord but not the arc.
    let r = [Vertex::new(0.0, 0.0, 1e12), Vertex::straight(10.0, 0.0)];
    assert_eq!(
        polygon_area(&r, None),
        Err(GeometryError::IllConditionedBulge { segment: 0 })
    );
}

#[test]
fn bulged_edge_that_stays_clear_is_valid() {
    // Rectangle 10 x 2 whose top edge bulges inward with sagitta 1.5. mpmath: 9.8222475667841816667.
    let r = [
        Vertex::straight(0.0, 0.0),
        Vertex::straight(10.0, 0.0),
        Vertex::new(10.0, 2.0, -0.3),
        Vertex::straight(0.0, 2.0),
    ];
    close_rel(
        polygon_area(&r, None).expect("valid input").area,
        9.822_247_566_784_181,
        1e-15,
    );
    // Two inward arcs in a 10 x 10 square, 0.5 apart at their closest. mpmath: 26.389444881161482303.
    let s = [
        Vertex::new(0.0, 0.0, -0.95),
        Vertex::straight(10.0, 0.0),
        Vertex::new(10.0, 10.0, -0.95),
        Vertex::straight(0.0, 10.0),
    ];
    close_rel(
        polygon_area(&s, None).expect("valid input").area,
        26.389_444_881_161_48,
        1e-14,
    );
}

#[test]
fn centroid_of_a_curved_ring_is_not_supported() {
    let r = [Vertex::new(0.0, 0.0, 1.0), Vertex::new(2.0, 0.0, 1.0)];
    assert_eq!(
        polygon_centroid(&r, None),
        Err(GeometryError::CurvedCentroidUnsupported)
    );
}

#[test]
fn explicit_tolerance_changes_decisions_not_values() {
    // A 1 mm kink in a 10 m edge: real at the default tolerance, collinear at an explicit 1 cm.
    let r = ring(&[(0.0, 0.0), (5.0, 0.001), (10.0, 0.0)]);
    let a = polygon_area(&r, None).expect("valid input");
    assert_eq!(a.orientation, Orientation::Clockwise); // apex above, returning along the x axis
    let a = polygon_area(&r, Some(0.01)).expect("valid input");
    assert_eq!(a.orientation, Orientation::Undefined);
    assert_eq!(a.notes, vec![Note::DegenerateRing]);
    close_rel(a.area, 0.005, 1e-15);
    assert!(a.tolerance.explicit);
    exact(a.tolerance.linear, 0.01);
}
