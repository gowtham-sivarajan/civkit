// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 18:37 IST · last edited 2026-11-25 18:37 IST

//! One test per row of docs/GEOMETRY.md §8, plus the self-intersection cases of the review.

use civkit_geometry::{
    GeometryError, Note, Orientation, Vertex, polygon_area, polygon_centroid, polyline_length,
};

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

fn is_self_intersection<T: std::fmt::Debug>(r: Result<T, GeometryError>) -> bool {
    matches!(r, Err(GeometryError::SelfIntersection { .. }))
}

// 8.1 -------------------------------------------------------------------------------------------

#[test]
fn non_finite_input_is_an_error() {
    let bad = [
        ring(&[(0.0, 0.0), (f64::NAN, 0.0), (1.0, 1.0)]),
        vec![
            Vertex::new(0.0, 0.0, f64::INFINITY),
            Vertex::straight(1.0, 0.0),
            Vertex::straight(1.0, 1.0),
        ],
    ];
    for r in &bad {
        assert_eq!(polygon_area(r, None), Err(GeometryError::NonFinite));
        assert_eq!(polygon_centroid(r, None), Err(GeometryError::NonFinite));
        assert_eq!(
            polyline_length(r, false, None),
            Err(GeometryError::NonFinite)
        );
    }
    let ok = ring(&[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0)]);
    assert_eq!(
        polygon_area(&ok, Some(f64::NAN)),
        Err(GeometryError::InvalidTolerance)
    );
    assert_eq!(
        polygon_area(&ok, Some(-1e-9)),
        Err(GeometryError::InvalidTolerance)
    );
}

#[test]
fn overflowing_results_are_an_error() {
    let huge = ring(&[(0.0, 0.0), (1e200, 0.0), (1e200, 1e200), (0.0, 1e200)]);
    assert_eq!(polygon_area(&huge, None), Err(GeometryError::Overflow));
}

// 8.2, 8.4 --------------------------------------------------------------------------------------

#[test]
fn zero_length_segment_in_a_ring_is_reported() {
    let r = ring(&[(0.0, 0.0), (1.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]);
    let a = polygon_area(&r, None).expect("valid input");
    exact(a.area, 1.0);
    assert_eq!(a.orientation, Orientation::CounterClockwise);
    assert_eq!(a.notes, vec![Note::ZeroLengthSegment { segment: 1 }]);
}

#[test]
fn explicit_closure_is_silent() {
    let r = ring(&[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0), (0.0, 0.0)]);
    let a = polygon_area(&r, None).expect("valid input");
    exact(a.area, 1.0);
    assert!(a.notes.is_empty());
}

#[test]
fn ill_conditioned_bulge_in_a_ring() {
    let r = vec![
        Vertex::new(0.0, 0.0, 1e15),
        Vertex::straight(1e-12, 0.0),
        Vertex::straight(5.0, 5.0),
    ];
    assert_eq!(
        polygon_area(&r, None),
        Err(GeometryError::IllConditionedBulge { segment: 0 })
    );
}

// 8.3 -------------------------------------------------------------------------------------------

#[test]
fn collinear_vertex_between_its_neighbours_is_allowed() {
    let r = ring(&[
        (0.0, 0.0),
        (5.0, 0.0),
        (10.0, 0.0),
        (10.0, 10.0),
        (0.0, 10.0),
    ]);
    let a = polygon_area(&r, None).expect("valid input");
    exact(a.area, 100.0);
    assert!(a.notes.is_empty());
}

// 8.5 -------------------------------------------------------------------------------------------

#[test]
fn too_few_vertices_for_a_polygon() {
    assert_eq!(
        polygon_area(&ring(&[(0.0, 0.0), (1.0, 0.0)]), None),
        Err(GeometryError::TooFewVertices {
            found: 2,
            required: 3
        })
    );
    // Three vertices, but two coincide: two remain after cleaning.
    assert_eq!(
        polygon_area(&ring(&[(0.0, 0.0), (1.0, 0.0), (1.0, 0.0)]), None),
        Err(GeometryError::TooFewVertices {
            found: 2,
            required: 3
        })
    );
    assert_eq!(
        polygon_area(&[], None),
        Err(GeometryError::TooFewVertices {
            found: 0,
            required: 3
        })
    );
    // One bulged vertex alone is a zero-length segment: nothing remains after cleaning.
    assert_eq!(
        polygon_area(&[Vertex::new(0.0, 0.0, 1.0)], None),
        Err(GeometryError::TooFewVertices {
            found: 0,
            required: 3
        })
    );
}

// 8.6 -------------------------------------------------------------------------------------------

#[test]
fn collinear_ring_is_degenerate_not_self_intersecting() {
    let r = ring(&[(0.0, 0.0), (5.0, 0.0), (10.0, 0.0)]);
    let a = polygon_area(&r, None).expect("valid input");
    exact(a.area, 0.0);
    assert_eq!(a.orientation, Orientation::Undefined);
    assert_eq!(a.notes, vec![Note::DegenerateRing]);
    assert_eq!(
        polygon_centroid(&r, None),
        Err(GeometryError::DegenerateRing)
    );

    // Nearly collinear: the computed area is kept, not rounded to 0 (D8).
    let r = ring(&[(0.0, 0.0), (10.0, 1e-12), (20.0, 0.0)]);
    let a = polygon_area(&r, None).expect("valid input");
    assert_eq!(a.orientation, Orientation::Undefined);
    assert!(a.area > 0.0 && (a.area - 1e-11).abs() <= 1e-24);
}

// 8.7 -------------------------------------------------------------------------------------------

#[test]
fn figure_eight_is_refused() {
    // Net shoelace is exactly 0 here; it must not be mistaken for a degenerate ring.
    let r = ring(&[(0.0, 0.0), (2.0, 2.0), (2.0, 0.0), (0.0, 2.0)]);
    assert!(is_self_intersection(polygon_area(&r, None)));
    assert!(is_self_intersection(polygon_centroid(&r, None)));
    // Length is unaffected.
    assert!(polyline_length(&r, true, None).is_ok());
}

#[test]
fn pinch_keyhole_and_spike_are_refused() {
    let pinch = ring(&[
        (0.0, 0.0),
        (1.0, 1.0),
        (2.0, 0.0),
        (2.0, 2.0),
        (1.0, 1.0),
        (0.0, 2.0),
    ]);
    let keyhole = ring(&[
        (0.0, 0.0),
        (10.0, 0.0),
        (10.0, 10.0),
        (0.0, 10.0),
        (0.0, 5.0),
        (4.0, 5.0),
        (4.0, 6.0),
        (6.0, 6.0),
        (6.0, 4.0),
        (4.0, 4.0),
        (4.0, 5.0),
        (0.0, 5.0),
    ]);
    let spike = ring(&[(0.0, 0.0), (10.0, 0.0), (5.0, 0.0), (5.0, 5.0)]);
    let back_and_forth = ring(&[
        (0.0, 0.0),
        (10.0, 0.0),
        (10.0, 10.0),
        (12.0, 10.0),
        (10.0, 10.0),
        (0.0, 10.0),
    ]);
    for r in [pinch, keyhole, spike, back_and_forth] {
        assert!(is_self_intersection(polygon_area(&r, None)), "{r:?}");
    }
}

#[test]
fn near_miss_just_outside_and_inside_eps() {
    // Vertex (5, g) approaches the non-adjacent bottom edge. D = hypot(10, 10), eps = 1e-9 * D.
    let eps = 1e-9 * 200.0_f64.sqrt();
    let notch = |g: f64| ring(&[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (5.0, g), (0.0, 10.0)]);
    assert!(polygon_area(&notch(2.0 * eps), None).is_ok());
    assert!(is_self_intersection(polygon_area(&notch(0.5 * eps), None)));
    assert!(is_self_intersection(polygon_area(&notch(0.0), None)));
}

#[test]
fn bulged_edge_crossing_another_edge_is_refused() {
    // Review finding M6: a chord-only check would return signed area -1.37 here.
    let r = vec![
        Vertex::straight(0.0, 0.0),
        Vertex::straight(10.0, 0.0),
        Vertex::new(10.0, 2.0, -0.6),
        Vertex::straight(0.0, 2.0),
    ];
    assert!(is_self_intersection(polygon_area(&r, None)));
}

#[test]
fn arcs_touching_or_crossing_each_other_are_refused() {
    let square = |b: f64| {
        vec![
            Vertex::new(0.0, 0.0, b),
            Vertex::straight(10.0, 0.0),
            Vertex::new(10.0, 10.0, b),
            Vertex::straight(0.0, 10.0),
        ]
    };
    assert!(polygon_area(&square(-0.95), None).is_ok()); // 0.5 apart
    assert!(is_self_intersection(polygon_area(&square(-1.0), None))); // tangent at (5, 5)
    assert!(is_self_intersection(polygon_area(&square(-1.1), None))); // crossing
}

#[test]
fn arc_touching_a_non_adjacent_vertex_is_refused() {
    // The bottom edge bulges up (b < 0, inward) with sagitta exactly 5 and touches the apex (5, 5).
    let r = vec![
        Vertex::new(0.0, 0.0, -1.0),
        Vertex::straight(10.0, 0.0),
        Vertex::straight(10.0, 10.0),
        Vertex::straight(5.0, 5.0),
        Vertex::straight(0.0, 10.0),
    ];
    assert!(is_self_intersection(polygon_area(&r, None)));
}

// 8.9 -------------------------------------------------------------------------------------------

#[test]
fn small_object_far_from_the_origin_is_precision_limited() {
    let tri = ring(&[(4e6, 4e6), (4e6 + 0.1, 4e6), (4e6, 4e6 + 0.1)]);
    let a = polygon_area(&tri, None).expect("valid input");
    assert_eq!(a.notes, vec![Note::PrecisionLimited]);
    // The offsets 0.1 are themselves only representable to ulp(4e6) at this position.
    assert!((a.area - 0.005).abs() <= 1e-9);
}
