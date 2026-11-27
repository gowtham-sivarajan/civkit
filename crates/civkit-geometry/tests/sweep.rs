// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 19:08 IST · last edited 2026-11-27 19:18 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! The §8.7 sweep prefilter and its work limit (docs/GEOMETRY.md §8.7, D-28 / ADR 0016, gate A7).
//!
//! Work is the number of edge pairs the sweep examines: pairs whose x-ranges, each widened by ε,
//! overlap. Counts below are worked by hand from each ring's edges. That the sweep finds exactly
//! what testing every pair finds is shown by the property tests inside the crate (`src/sweep.rs`).

use std::f64::consts::PI;

use civkit_geometry::{
    GeometryError, Vertex, WorkLimit, polygon_area, polygon_area_limited, region_area,
    region_area_limited,
};

fn close_rel(actual: f64, expected: f64, rel: f64) {
    let scale = expected.abs().max(f64::MIN_POSITIVE);
    assert!(
        ((actual - expected) / scale).abs() <= rel,
        "expected {expected:e}, got {actual:e} (relative tolerance {rel:e})"
    );
}

fn square(x0: f64, y0: f64, s: f64) -> Vec<Vertex> {
    vec![
        Vertex::straight(x0, y0),
        Vertex::straight(x0 + s, y0),
        Vertex::straight(x0 + s, y0 + s),
        Vertex::straight(x0, y0 + s),
    ]
}

/// A square's edges: bottom and top span the whole width, right and left are points in x. Pairs
/// whose x-ranges overlap: bottom–right, bottom–top, bottom–left, right–top, top–left (right–left do
/// not). So a square costs 5.
#[test]
fn a_square_costs_five_at_the_boundary() {
    let ring = square(0.0, 0.0, 1.0);
    let mut work = WorkLimit::new(5);
    let a = polygon_area_limited(&ring, None, &mut work).expect("within the limit");
    assert_eq!(a.area, 1.0);
    assert_eq!(work.used(), 5);

    let mut work = WorkLimit::new(4);
    assert_eq!(
        polygon_area_limited(&ring, None, &mut work),
        Err(GeometryError::WorkLimitExceeded { limit: 4 })
    );
}

/// The limit is shared: two calls draw on the same budget.
#[test]
fn work_accumulates_across_calls() {
    let ring = square(0.0, 0.0, 1.0);
    let mut work = WorkLimit::new(10);
    polygon_area_limited(&ring, None, &mut work).expect("first");
    polygon_area_limited(&ring, None, &mut work).expect("second");
    assert_eq!(work.used(), 10);
    assert_eq!(
        polygon_area_limited(&ring, None, &mut work),
        Err(GeometryError::WorkLimitExceeded { limit: 10 })
    );
}

/// Over the limit, a self-intersecting ring is not reported as self-intersecting: the check was not
/// completed. Bow tie (0,0)–(2,2)–(2,0)–(0,2): the same five x-overlaps as a square.
#[test]
fn over_the_limit_is_neither_valid_nor_self_intersecting() {
    let bow_tie = [
        Vertex::straight(0.0, 0.0),
        Vertex::straight(2.0, 2.0),
        Vertex::straight(2.0, 0.0),
        Vertex::straight(0.0, 2.0),
    ];
    assert_eq!(
        polygon_area_limited(&bow_tie, None, &mut WorkLimit::new(4)),
        Err(GeometryError::WorkLimitExceeded { limit: 4 })
    );
    assert_eq!(
        polygon_area_limited(&bow_tie, None, &mut WorkLimit::new(5)),
        Err(GeometryError::SelfIntersection {
            first: 0,
            second: 2
        })
    );
}

/// Two separate crossings: edges 0 and 2 cross at (11, 1), edges 4 and 6 at (1, 1). The sweep meets
/// the left crossing first (it runs in x), but the error names the first pair in (i, j) order, as
/// testing every pair in order would: (0, 2).
#[test]
fn the_first_crossing_in_order_is_reported() {
    let ring = [
        Vertex::straight(10.0, 0.0),
        Vertex::straight(12.0, 2.0),
        Vertex::straight(12.0, 0.0),
        Vertex::straight(10.0, 2.0),
        Vertex::straight(2.0, 2.0),
        Vertex::straight(0.0, 0.0),
        Vertex::straight(0.0, 2.0),
        Vertex::straight(2.0, 0.0),
    ];
    assert_eq!(
        polygon_area(&ring, None),
        Err(GeometryError::SelfIntersection {
            first: 0,
            second: 2
        })
    );
    // Region: rings 1 and 2 cross each other on the left, rings 0 and 2 on the right: (0, 2) first.
    let rings = [
        square(20.0, 0.0, 2.0),
        square(0.0, 0.0, 2.0),
        square(1.0, 1.0, 20.0),
    ];
    let refs: Vec<&[Vertex]> = rings.iter().map(Vec::as_slice).collect();
    assert_eq!(
        region_area(&refs, None),
        Err(GeometryError::RingsIntersect {
            first: 0,
            second: 2
        })
    );
}

/// Region: each ring's own check (5 each), then pairs of edges from different rings. Outer 0..10
/// with hole 2..4: the outer bottom and top edges overlap all four hole edges in x (8); the outer
/// left (x = 0) and right (x = 10) edges overlap none. 5 + 5 + 8 = 18.
#[test]
fn a_region_costs_its_rings_and_their_cross_pairs() {
    let rings = [square(0.0, 0.0, 10.0), square(2.0, 2.0, 2.0)];
    let refs: Vec<&[Vertex]> = rings.iter().map(Vec::as_slice).collect();
    let mut work = WorkLimit::new(18);
    let r = region_area_limited(&refs, None, &mut work).expect("within the limit");
    assert_eq!(r.area, 96.0);
    assert_eq!(work.used(), 18);
    assert_eq!(
        region_area_limited(&refs, None, &mut WorkLimit::new(17)),
        Err(GeometryError::WorkLimitExceeded { limit: 17 })
    );

    // Two squares apart in x: no cross pairs, 5 + 5 = 10.
    let apart = [square(0.0, 0.0, 1.0), square(5.0, 0.0, 1.0)];
    let refs: Vec<&[Vertex]> = apart.iter().map(Vec::as_slice).collect();
    let mut work = WorkLimit::new(10);
    region_area_limited(&refs, None, &mut work).expect("within the limit");
    assert_eq!(work.used(), 10);
}

/// Boxes are widened by ε: rings closer than ε but whose bare boxes do not overlap are still
/// compared. Square 0..10 and a square above it at 10 + δ. D = |(10, 20 + δ)| ≈ 22.4, ε ≈ 2.2 × 10⁻⁸.
#[test]
fn near_touching_rings_are_compared_through_widened_boxes() {
    let touching = [square(0.0, 0.0, 10.0), square(0.0, 10.0 + 1e-9, 10.0)];
    let refs: Vec<&[Vertex]> = touching.iter().map(Vec::as_slice).collect();
    assert_eq!(
        region_area(&refs, None),
        Err(GeometryError::RingsIntersect {
            first: 0,
            second: 1
        })
    );
    let apart = [square(0.0, 0.0, 10.0), square(0.0, 10.0 + 1e-6, 10.0)];
    let refs: Vec<&[Vertex]> = apart.iter().map(Vec::as_slice).collect();
    close_rel(region_area(&refs, None).expect("apart").area, 200.0, 1e-15);
}

/// An arc's box includes its extremes, not only its chord's ends. Square 0..10 whose bottom edge
/// bulges down to (5, −5) (b = 1); a 2 × 2 square below it whose top edge is δ under (5, −5). The
/// chord's box ends at y = 0, so only the extreme brings the two boxes together.
/// D = |(10, 17 + δ)| ≈ 19.7, ε ≈ 2.0 × 10⁻⁸.
#[test]
fn arc_extremes_are_in_the_box() {
    let mut bulged = square(0.0, 0.0, 10.0);
    bulged[0].bulge = 1.0;
    for (delta, intersect) in [(1e-9, true), (1e-6, false)] {
        let below = square(4.0, -7.0 - delta, 2.0);
        let rings = [bulged.clone(), below];
        let refs: Vec<&[Vertex]> = rings.iter().map(Vec::as_slice).collect();
        let r = region_area(&refs, None);
        if intersect {
            assert_eq!(
                r,
                Err(GeometryError::RingsIntersect {
                    first: 0,
                    second: 1
                })
            );
        } else {
            close_rel(r.expect("apart").area, 100.0 + 25.0 * PI / 2.0 + 4.0, 1e-14);
        }
    }
}

/// A meander of long horizontal edges, all overlapping in x: about n²/2 examined pairs. With a
/// limit it stops at the limit quickly; it never returns "valid" or "self-intersecting".
#[test]
fn a_hostile_meander_stops_at_the_limit() {
    let rows = 20_000_u32;
    let width = 1000.0;
    let mut ring = Vec::new();
    for r in 0..rows {
        let y = f64::from(r);
        if r % 2 == 0 {
            ring.push(Vertex::straight(0.0, y));
            ring.push(Vertex::straight(width, y));
        } else {
            ring.push(Vertex::straight(width, y));
            ring.push(Vertex::straight(0.0, y));
        }
    }
    // Close the meander along x = −1, clear of the joins at x = 0.
    let top = f64::from(rows - 1);
    ring.push(Vertex::straight(-1.0, top));
    ring.push(Vertex::straight(-1.0, 0.0));
    assert_eq!(
        polygon_area_limited(&ring, None, &mut WorkLimit::new(1_000_000)),
        Err(GeometryError::WorkLimitExceeded { limit: 1_000_000 })
    );
}

/// The sweep keeps ordinary rings cheap: a regular 50 000-gon examines a few pairs per edge, not
/// n²/2 ≈ 1.25 × 10⁹. Area n/2·R²·sin(2π/n).
#[test]
fn a_large_ordinary_ring_is_cheap() {
    let n = 50_000_u32;
    let r = 100.0;
    let ring: Vec<Vertex> = (0..n)
        .map(|i| {
            let t = f64::from(i) / f64::from(n) * 2.0 * PI;
            Vertex::straight(r * t.cos(), r * t.sin())
        })
        .collect();
    let mut work = WorkLimit::unlimited();
    let a = polygon_area_limited(&ring, None, &mut work).expect("valid");
    let nf = f64::from(n);
    close_rel(a.area, nf / 2.0 * r * r * (2.0 * PI / nf).sin(), 1e-12);
    assert!(
        work.used() < 10 * u64::from(n),
        "examined {} pairs",
        work.used()
    );
    // The unlimited entry point gives the same result.
    assert_eq!(polygon_area(&ring, None).expect("valid").area, a.area);
}
