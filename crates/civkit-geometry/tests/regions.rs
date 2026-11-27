// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 18:57 IST · last edited 2026-11-27 19:02 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! Regions: rings combined by even-odd nesting (docs/GEOMETRY.md §11, D-28 / ADR 0016, gate A7).
//!
//! Expected values are closed forms (squares, half-discs, discs) worked by hand in each test. None
//! is produced by this crate.

use std::f64::consts::PI;

use civkit_geometry::{GeometryError, Note, Vertex, region_area};
use proptest::prelude::*;

fn close_rel(actual: f64, expected: f64, rel: f64) {
    let scale = expected.abs().max(f64::MIN_POSITIVE);
    assert!(
        ((actual - expected) / scale).abs() <= rel,
        "expected {expected:e}, got {actual:e} (relative tolerance {rel:e})"
    );
}

/// An axis-aligned square from (x0, y0) with side s; counter-clockwise unless `cw`.
fn square(x0: f64, y0: f64, s: f64, cw: bool) -> Vec<Vertex> {
    let mut v = vec![
        Vertex::straight(x0, y0),
        Vertex::straight(x0 + s, y0),
        Vertex::straight(x0 + s, y0 + s),
        Vertex::straight(x0, y0 + s),
    ];
    if cw {
        v.reverse();
    }
    v
}

/// A circle of radius r about (cx, cy) as two semicircular bulges (b = ±1).
fn circle(cx: f64, cy: f64, r: f64, cw: bool) -> Vec<Vertex> {
    let b = if cw { -1.0 } else { 1.0 };
    vec![Vertex::new(cx + r, cy, b), Vertex::new(cx - r, cy, b)]
}

fn area(rings: &[Vec<Vertex>]) -> Result<f64, GeometryError> {
    let refs: Vec<&[Vertex]> = rings.iter().map(Vec::as_slice).collect();
    region_area(&refs, None).map(|r| r.area)
}

#[test]
fn one_ring_is_its_area() {
    for cw in [false, true] {
        assert_eq!(area(&[square(0.0, 0.0, 10.0, cw)]), Ok(100.0), "cw = {cw}");
    }
}

/// Outer 10 × 10 with a 2 × 2 hole: 100 − 4 = 96, whatever the rings' orientations.
#[test]
fn a_hole_is_subtracted_whatever_the_orientation() {
    for outer_cw in [false, true] {
        for hole_cw in [false, true] {
            let r = area(&[
                square(0.0, 0.0, 10.0, outer_cw),
                square(2.0, 2.0, 2.0, hole_cw),
            ]);
            assert_eq!(r, Ok(96.0), "outer cw {outer_cw}, hole cw {hole_cw}");
        }
    }
}

/// Outer 10 × 10, hole 8 × 8 at (1, 1), island 4 × 4 at (3, 3): 100 − 64 + 16 = 52, in any input
/// order and with orientations varied.
#[test]
fn nested_island_counts_again() {
    let outer = square(0.0, 0.0, 10.0, false);
    let hole = square(1.0, 1.0, 8.0, false);
    let island = square(3.0, 3.0, 4.0, true);
    let orders = [
        [outer.clone(), hole.clone(), island.clone()],
        [island.clone(), outer.clone(), hole.clone()],
        [hole.clone(), island.clone(), outer.clone()],
    ];
    for rings in &orders {
        assert_eq!(area(rings), Ok(52.0));
    }
    let refs: Vec<&[Vertex]> = orders[1].iter().map(Vec::as_slice).collect();
    let r = region_area(&refs, None).expect("valid");
    let depths: Vec<usize> = r.rings.iter().map(|ring| ring.depth).collect();
    assert_eq!(depths, vec![2, 0, 1], "depth of each ring, in input order");
    assert_eq!(r.rings[1].area.area, 100.0);
}

#[test]
fn disjoint_rings_add() {
    let r = area(&[square(0.0, 0.0, 3.0, false), square(10.0, 0.0, 2.0, true)]);
    assert_eq!(r, Ok(13.0));
}

/// Curved rings: a disc of radius 5 with a 2 × 2 square hole (25π − 4) and with a disc hole of
/// radius 1 (25π − π = 24π).
#[test]
fn curved_rings() {
    close_rel(
        area(&[circle(0.0, 0.0, 5.0, false), square(-1.0, -1.0, 2.0, true)]).expect("valid"),
        25.0 * PI - 4.0,
        1e-14,
    );
    close_rel(
        area(&[circle(0.0, 0.0, 5.0, true), circle(0.0, 0.0, 1.0, false)]).expect("valid"),
        24.0 * PI,
        1e-14,
    );
}

/// A hole inside an outward bulge lies outside the outer ring's chord polygon: containment must
/// follow the arc, not the chord. Outer: 10 × 10 square whose bottom edge bulges down as a
/// semicircle (b = 1 from (0,0) to (10,0), centre (5, 0), radius 5): 100 + 25π/2. Hole: 2 × 2 at
/// (4, −3), every corner within 3.2 of (5, 0). Region: 100 + 25π/2 − 4.
#[test]
fn containment_follows_an_outward_bulge() {
    let mut outer = square(0.0, 0.0, 10.0, false);
    outer[0].bulge = 1.0;
    let hole = square(4.0, -3.0, 2.0, false);
    close_rel(
        area(&[outer, hole]).expect("valid"),
        100.0 + 25.0 * PI / 2.0 - 4.0,
        1e-14,
    );
}

/// A ring inside an inward bulge's bite lies inside the chord polygon but outside the ring: it is
/// a separate region (depth 0) and adds. Outer: 10 × 10 square whose top edge, from (10,10) to
/// (0,10), bulges inward as a semicircle (b = −1: clockwise from (10,10) to (0,10) passes (5, 5);
/// centre (5, 10), radius 5): 100 − 25π/2. Island 1 × 1 at (4.5, 8.5): inside the bite. Region:
/// 100 − 25π/2 + 1.
#[test]
fn containment_follows_an_inward_bulge() {
    let mut outer = square(0.0, 0.0, 10.0, false);
    outer[2].bulge = -1.0;
    let island = square(4.5, 8.5, 1.0, false);
    let refs = [outer.as_slice(), island.as_slice()];
    let r = region_area(&refs, None).expect("valid");
    close_rel(r.area, 100.0 - 25.0 * PI / 2.0 + 1.0, 1e-14);
    assert_eq!(r.rings[1].depth, 0);
}

#[test]
fn crossing_or_touching_rings_are_refused() {
    // Two overlapping squares.
    assert_eq!(
        area(&[square(0.0, 0.0, 4.0, false), square(2.0, 2.0, 4.0, false)]),
        Err(GeometryError::RingsIntersect {
            first: 0,
            second: 1
        })
    );
    // A hole touching the outer ring's edge from inside.
    assert_eq!(
        area(&[square(0.0, 0.0, 10.0, false), square(0.0, 3.0, 2.0, false)]),
        Err(GeometryError::RingsIntersect {
            first: 0,
            second: 1
        })
    );
    // Two squares sharing one corner.
    assert_eq!(
        area(&[square(0.0, 0.0, 1.0, false), square(1.0, 1.0, 1.0, false)]),
        Err(GeometryError::RingsIntersect {
            first: 0,
            second: 1
        })
    );
    // A hole crossing a curved outer edge (the disc of radius 5 and a square reaching past it).
    assert_eq!(
        area(&[circle(0.0, 0.0, 5.0, false), square(3.0, -1.0, 4.0, false)]),
        Err(GeometryError::RingsIntersect {
            first: 0,
            second: 1
        })
    );
}

#[test]
fn a_ring_that_is_invalid_on_its_own_is_refused() {
    let bow_tie = vec![
        Vertex::straight(0.0, 0.0),
        Vertex::straight(2.0, 2.0),
        Vertex::straight(2.0, 0.0),
        Vertex::straight(0.0, 2.0),
    ];
    assert!(matches!(
        area(&[square(-5.0, -5.0, 20.0, false), bow_tie]),
        Err(GeometryError::SelfIntersection { .. })
    ));
    assert_eq!(area(&[]), Err(GeometryError::NoRings));
    let mut bad = square(0.0, 0.0, 1.0, false);
    bad[1].point.x = f64::NAN;
    assert_eq!(area(&[bad]), Err(GeometryError::NonFinite));
}

/// One tolerance for the whole region, from all rings; far from the origin, precision is limited.
#[test]
fn one_tolerance_and_precision_note() {
    let refs_rings = [square(0.0, 0.0, 10.0, false), square(2.0, 2.0, 2.0, false)];
    let refs: Vec<&[Vertex]> = refs_rings.iter().map(Vec::as_slice).collect();
    let r = region_area(&refs, None).expect("valid");
    // D is the diagonal of the whole region: 10√2; ε = 10⁻⁹·D.
    close_rel(r.tolerance.size, 10.0 * 2.0_f64.sqrt(), 1e-15);
    close_rel(r.tolerance.linear, 1e-8 * 2.0_f64.sqrt(), 1e-15);
    assert!(r.notes.is_empty());

    // M = 10⁹: 1024·ulp(M) ≈ 1.2 × 10⁻⁴ > 10⁻⁶·D (D = √2), yet far below the side, so the ring is valid.
    let far = [square(1e9, 0.0, 1.0, false)];
    let refs: Vec<&[Vertex]> = far.iter().map(Vec::as_slice).collect();
    let r = region_area(&refs, None).expect("valid");
    assert!(r.notes.contains(&Note::PrecisionLimited));
}

proptest! {
    /// Concentric squares, each strictly inside the previous: the region alternates +, −, +, …
    /// Orientations and input order are random. Half-integer coordinates keep every area exact.
    #[test]
    fn concentric_squares_alternate(
        n in 1_usize..6,
        cw in prop::collection::vec(any::<bool>(), 6),
        rotate in 0_usize..6,
    ) {
        let mut rings: Vec<Vec<Vertex>> = (0..n)
            .map(|k| {
                let inset = 2.0 * k as f64;
                square(inset, inset, 30.0 - 2.0 * inset, cw[k])
            })
            .collect();
        let expected: f64 = (0..n)
            .map(|k| {
                let side = 30.0 - 4.0 * k as f64;
                if k % 2 == 0 { side * side } else { -side * side }
            })
            .sum();
        rings.rotate_left(rotate % n);
        prop_assert_eq!(area(&rings), Ok(expected));
    }
}
