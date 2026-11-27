// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 18:57 IST · last edited 2026-11-27 19:02 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! 2D affine transformations and block placement (docs/GEOMETRY.md §11, D-28 / ADR 0016, gate A3).
//!
//! Expected values are worked by hand from the formula in §11 (each test shows the steps) or are
//! closed forms. None is produced by this crate.

use std::f64::consts::PI;

use civkit_geometry::{
    Affine2, GeometryError, InsertPlacement, Orientation, Point, Vertex, polygon_area,
};
use proptest::prelude::*;

fn close(actual: f64, expected: f64, abs: f64) {
    assert!(
        (actual - expected).abs() <= abs,
        "expected {expected:e}, got {actual:e} (absolute tolerance {abs:e})"
    );
}

fn close_rel(actual: f64, expected: f64, rel: f64) {
    let scale = expected.abs().max(f64::MIN_POSITIVE);
    assert!(
        ((actual - expected) / scale).abs() <= rel,
        "expected {expected:e}, got {actual:e} (relative tolerance {rel:e})"
    );
}

/// Exact equality of values (+0 and −0 count as equal), for results every step of which is exact.
fn same(a: f64, b: f64) -> bool {
    a.to_bits() == b.to_bits() || (a == 0.0 && b == 0.0)
}

fn point_eq(p: Point, x: f64, y: f64) {
    assert_eq!(
        (p.x, p.y),
        (x, y),
        "expected ({x}, {y}), got ({}, {})",
        p.x,
        p.y
    );
}

fn at(t: &Affine2, x: f64, y: f64) -> Point {
    t.apply(Point::new(x, y)).expect("finite")
}

#[test]
fn identity_translation_scale() {
    point_eq(at(&Affine2::IDENTITY, 3.5, -2.0), 3.5, -2.0);
    let t = Affine2::translation(10.0, -4.0).expect("finite");
    point_eq(at(&t, 1.0, 1.0), 11.0, -3.0);
    let s = Affine2::scale(2.0, 3.0).expect("finite");
    point_eq(at(&s, 1.0, 1.0), 2.0, 3.0);
    assert_eq!(s.det(), 6.0);
}

/// Multiples of 90° are exact (DXF rotations are stated in degrees; 90° is common, fixture F5).
#[test]
fn quarter_turns_are_exact() {
    let cases = [
        (90.0, (0.0, 1.0)),
        (180.0, (-1.0, 0.0)),
        (270.0, (0.0, -1.0)),
        (-90.0, (0.0, -1.0)),
        (450.0, (0.0, 1.0)),
        (360.0, (1.0, 0.0)),
    ];
    for (deg, (x, y)) in cases {
        let r = Affine2::rotation_deg(deg).expect("finite");
        let p = at(&r, 1.0, 0.0);
        assert!(
            same(p.x, x) && same(p.y, y),
            "{deg}°: expected ({x}, {y}), got ({}, {})",
            p.x,
            p.y
        );
        assert_eq!(r.det(), 1.0, "{deg}°");
    }
}

#[test]
fn general_rotation() {
    // 30°: (1, 0) → (√3/2, 1/2).
    let r = Affine2::rotation_deg(30.0).expect("finite");
    let p = at(&r, 1.0, 0.0);
    close(p.x, 3.0_f64.sqrt() / 2.0, 1e-15);
    close(p.y, 0.5, 1e-15);
    // 135°: (2, 0) → (−√2, √2).
    let p = at(&Affine2::rotation_deg(135.0).expect("finite"), 2.0, 0.0);
    close(p.x, -(2.0_f64.sqrt()), 1e-15);
    close(p.y, 2.0_f64.sqrt(), 1e-15);
}

/// `a.then(b)` applies `a` first, then `b`.
#[test]
fn composition_order() {
    let t = Affine2::translation(1.0, 2.0).expect("finite");
    let r = Affine2::rotation_deg(90.0).expect("finite");
    // Translate then rotate: (0, 0) → (1, 2) → (−2, 1).
    point_eq(at(&t.then(&r).expect("finite"), 0.0, 0.0), -2.0, 1.0);
    // Rotate then translate: (0, 0) → (0, 0) → (1, 2).
    point_eq(at(&r.then(&t).expect("finite"), 0.0, 0.0), 1.0, 2.0);
}

#[test]
fn mirrors_and_determinants() {
    let m = Affine2::mirror_x();
    point_eq(at(&m, 2.0, 3.0), -2.0, 3.0);
    assert_eq!(m.det(), -1.0);
    assert!(m.is_mirror());

    let s = Affine2::scale(-1.0, 1.0).expect("finite");
    assert!(s.is_mirror());
    // Two mirrors make a rotation.
    let mm = m
        .then(&Affine2::scale(1.0, -1.0).expect("finite"))
        .expect("finite");
    assert!(!mm.is_mirror());
    point_eq(at(&mm, 2.0, 3.0), -2.0, -3.0);
    assert!(!Affine2::rotation_deg(37.0).expect("finite").is_mirror());
}

#[test]
fn similarity_detection() {
    // Uniform scale 2 with a rotation: scale 2 (= √|det|).
    let t = Affine2::scale(2.0, 2.0)
        .and_then(|s| s.then(&Affine2::rotation_deg(37.0)?))
        .expect("finite");
    close_rel(t.similarity_scale().expect("similarity"), 2.0, 1e-15);
    // A mirror is a similarity of scale 1.
    assert_eq!(Affine2::mirror_x().similarity_scale(), Some(1.0));
    // Non-uniform scale.
    assert_eq!(
        Affine2::scale(2.0, 1.0).expect("finite").similarity_scale(),
        None
    );
    // σ₁ − σ₂ ≤ 10⁻⁹·σ₁: a relative difference of 10⁻¹² is uniform, 10⁻⁶ is not.
    assert!(
        Affine2::scale(1.0, 1.0 + 1e-12)
            .expect("finite")
            .similarity_scale()
            .is_some()
    );
    assert!(
        Affine2::scale(1.0, 1.0 + 1e-6)
            .expect("finite")
            .similarity_scale()
            .is_none()
    );
    // Singular values of a shear [[1, 1], [0, 1]]: the golden ratio φ and 1/φ.
    let shear = Affine2::new(1.0, 1.0, 0.0, 1.0, 0.0, 0.0).expect("finite");
    let (s1, s2) = shear.singular_values();
    let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;
    close_rel(s1, phi, 1e-15);
    close_rel(s2, 1.0 / phi, 1e-15);
    assert!(shear.similarity_scale().is_none());
}

/// The insert formula of §11: p' = P + R(θ)·((c·Δc, r·Δr) + S(sx, sy)·(p − B)).
#[test]
fn insert_placement_formula() {
    // B = (1, 1), P = (10, 20), S = (2, 3), θ = 90°, offset (c·Δc, r·Δr) = (2·5, 1·7) = (10, 7).
    // p = (2, 1): p − B = (1, 0); S → (2, 0); + offset → (12, 7); R(90°) → (−7, 12); + P → (3, 32).
    let t = Affine2::insert(&InsertPlacement {
        base: Point::new(1.0, 1.0),
        at: Point::new(10.0, 20.0),
        scale: (2.0, 3.0),
        rotation_deg: 90.0,
        offset: (10.0, 7.0),
    })
    .expect("finite");
    point_eq(at(&t, 2.0, 1.0), 3.0, 32.0);
    // The base point itself lands on P plus the rotated offset: R(90°)·(10, 7) = (−7, 10) → (3, 30).
    point_eq(at(&t, 1.0, 1.0), 3.0, 30.0);
    assert_eq!(t.det(), 6.0);
}

/// A MINSERT row of bays (fixture F5's shape): spacing is along the rotated axes and not scaled.
#[test]
fn minsert_columns_are_spaced_along_rotated_axes() {
    for column in 0..3_i32 {
        let t = Affine2::insert(&InsertPlacement {
            base: Point::new(0.0, 0.0),
            at: Point::new(100.0, 50.0),
            scale: (2.0, 2.0),
            rotation_deg: 90.0,
            offset: (f64::from(column) * 2.5, 0.0),
        })
        .expect("finite");
        // Block origin: R(90°)·(2.5c, 0) = (0, 2.5c), so (100, 50 + 2.5c): the spacing is not doubled.
        point_eq(at(&t, 0.0, 0.0), 100.0, 50.0 + 2.5 * f64::from(column));
        // Block point (1, 0): S → (2, 0); + (2.5c, 0); R(90°) → (0, 2 + 2.5c); + P.
        point_eq(at(&t, 1.0, 0.0), 100.0, 52.0 + 2.5 * f64::from(column));
    }
}

/// Nested inserts compose, the outermost applied last.
#[test]
fn nested_inserts_compose() {
    // Inner: a TREE placed at (3, 0) at scale 0.5 inside ISLAND.
    let inner = Affine2::insert(&InsertPlacement {
        base: Point::new(0.0, 0.0),
        at: Point::new(3.0, 0.0),
        scale: (0.5, 0.5),
        rotation_deg: 0.0,
        offset: (0.0, 0.0),
    })
    .expect("finite");
    // Outer: ISLAND placed at (10, 10), rotated 90°, scale 2.
    let outer = Affine2::insert(&InsertPlacement {
        base: Point::new(0.0, 0.0),
        at: Point::new(10.0, 10.0),
        scale: (2.0, 2.0),
        rotation_deg: 90.0,
        offset: (0.0, 0.0),
    })
    .expect("finite");
    let both = inner.then(&outer).expect("finite");
    // Tree point (1, 0): inner → (3.5, 0); outer: S → (7, 0); R(90°) → (0, 7); + (10, 10) → (10, 17).
    point_eq(at(&both, 1.0, 0.0), 10.0, 17.0);
    // Scale of the composition: 0.5 × 2 = 1.
    assert_eq!(both.similarity_scale(), Some(1.0));
}

/// A mirror negates bulges, so the mirrored shape is the mirror image (§11).
#[test]
fn mirrored_bulges_keep_the_shape() {
    // Square 2 × 2 whose bottom edge bulges outward as a semicircle (b = 1 from (0,0) to (2,0)):
    // area 4 + π/2, counter-clockwise.
    let ring = [
        Vertex::new(0.0, 0.0, 1.0),
        Vertex::straight(2.0, 0.0),
        Vertex::straight(2.0, 2.0),
        Vertex::straight(0.0, 2.0),
    ];
    let before = polygon_area(&ring, None).expect("valid");
    close_rel(before.signed_area, 4.0 + PI / 2.0, 1e-14);

    let m = Affine2::mirror_x();
    let mirrored: Vec<Vertex> = ring
        .iter()
        .map(|v| m.apply_vertex(*v).expect("valid"))
        .collect();
    assert_eq!(mirrored[0], Vertex::new(-0.0, 0.0, -1.0));
    let after = polygon_area(&mirrored, None).expect("valid");
    // Same area (had the bulge kept its sign, the edge would bulge inward: 4 − π/2), orientation reversed.
    close_rel(after.area, 4.0 + PI / 2.0, 1e-14);
    close_rel(after.signed_area, -(4.0 + PI / 2.0), 1e-14);
    assert_eq!(after.orientation, Orientation::Clockwise);

    // A rotation keeps the bulge.
    let r = Affine2::rotation_deg(90.0).expect("finite");
    assert_eq!(r.apply_vertex(ring[0]).expect("valid").bulge, 1.0);
}

/// Areas scale by |det| under any affine map; lengths by s under a similarity (§11).
#[test]
fn areas_scale_by_the_determinant() {
    let square = [
        Vertex::straight(0.0, 0.0),
        Vertex::straight(1.0, 0.0),
        Vertex::straight(1.0, 1.0),
        Vertex::straight(0.0, 1.0),
    ];
    let t = Affine2::scale(2.0, 3.0)
        .and_then(|s| s.then(&Affine2::rotation_deg(30.0)?))
        .expect("finite");
    let moved: Vec<Vertex> = square
        .iter()
        .map(|v| {
            t.apply_vertex(*v)
                .expect("straight edges under non-uniform scale")
        })
        .collect();
    close_rel(polygon_area(&moved, None).expect("valid").area, 6.0, 1e-14);
}

#[test]
fn curves_under_non_uniform_scale_are_not_supported() {
    let t = Affine2::scale(2.0, 1.0).expect("finite");
    assert_eq!(
        t.apply_vertex(Vertex::new(0.0, 0.0, 0.5)),
        Err(GeometryError::CurveUnderNonUniformScale)
    );
    // A straight vertex is fine.
    assert_eq!(
        t.apply_vertex(Vertex::straight(1.0, 1.0)),
        Ok(Vertex::straight(2.0, 1.0))
    );
}

#[test]
fn degenerate_and_non_finite() {
    let flat = Affine2::scale(0.0, 1.0).expect("finite");
    assert_eq!(flat.det(), 0.0);
    assert_eq!(
        flat.apply_vertex(Vertex::straight(1.0, 1.0)),
        Err(GeometryError::DegenerateTransform)
    );
    assert_eq!(Affine2::scale(f64::NAN, 1.0), Err(GeometryError::NonFinite));
    assert_eq!(
        Affine2::rotation_deg(f64::INFINITY),
        Err(GeometryError::NonFinite)
    );
    assert_eq!(
        Affine2::IDENTITY.apply(Point::new(f64::NAN, 0.0)),
        Err(GeometryError::NonFinite)
    );
    let huge = Affine2::scale(1e300, 1e300).expect("finite");
    assert_eq!(
        huge.apply(Point::new(1e300, 0.0)),
        Err(GeometryError::Overflow)
    );
    assert_eq!(huge.then(&huge), Err(GeometryError::Overflow));
}

proptest! {
    /// `a.then(b)` equals applying `a`, then `b`. Integer-valued entries and points keep every
    /// product exact, so the equality is exact.
    #[test]
    fn then_equals_sequential_application(
        e in prop::collection::vec(-20_i32..20, 12),
        x in -50_i32..50,
        y in -50_i32..50,
    ) {
        let f = |i: usize| f64::from(e[i]);
        let a = Affine2::new(f(0), f(1), f(2), f(3), f(4), f(5)).expect("finite");
        let b = Affine2::new(f(6), f(7), f(8), f(9), f(10), f(11)).expect("finite");
        let p = Point::new(f64::from(x), f64::from(y));
        let sequential = b.apply(a.apply(p).expect("finite")).expect("finite");
        let composed = a.then(&b).expect("finite").apply(p).expect("finite");
        prop_assert_eq!((composed.x, composed.y), (sequential.x, sequential.y));
        // det is multiplicative.
        prop_assert!(same(a.then(&b).expect("finite").det(), a.det() * b.det()));
    }

    /// A rotation and a uniform scale form a similarity of that scale; mirroring keeps it one.
    #[test]
    fn rotation_and_uniform_scale_is_a_similarity(
        deg in -720.0_f64..720.0,
        k in 0.001_f64..1000.0,
        mirror in any::<bool>(),
    ) {
        let mut t = Affine2::scale(k, k).and_then(|s| s.then(&Affine2::rotation_deg(deg)?)).expect("finite");
        if mirror {
            t = t.then(&Affine2::mirror_x()).expect("finite");
        }
        let s = t.similarity_scale();
        prop_assert!(s.is_some());
        prop_assert!((s.unwrap_or(0.0) - k).abs() <= 1e-12 * k);
        prop_assert_eq!(t.is_mirror(), mirror);
    }
}
