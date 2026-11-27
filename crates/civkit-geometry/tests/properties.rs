// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 18:37 IST · last edited 2026-11-27 22:53 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! Invariants (docs/GEOMETRY.md §12, item 6). Each property states its tolerance explicitly;
//! exact equality is used only where every operation involved is exact.

use std::f64::consts::TAU;

use civkit_geometry::{Vertex, azimuth_from_components, components, polygon_area, polyline_length};
use proptest::prelude::*;

/// A star-shaped ring on integer coordinates: vertices at increasing angles around the origin.
/// Integer coordinates keep every product in the shoelace sum exact.
fn star_ring() -> impl Strategy<Value = Vec<(f64, f64)>> {
    prop::collection::vec((1_u32..1000, 0_u32..3600), 3..12).prop_map(|mut v| {
        v.sort_by_key(|&(_, a)| a);
        v.dedup_by_key(|p| p.1);
        v.into_iter()
            .map(|(r, a)| {
                let t = f64::from(a) / 3600.0 * TAU;
                (
                    (f64::from(r) * t.cos()).round(),
                    (f64::from(r) * t.sin()).round(),
                )
            })
            .collect()
    })
}

/// Exact equality of values (+0 and −0 count as equal).
fn same(a: f64, b: f64) -> bool {
    a.to_bits() == b.to_bits() || (a == 0.0 && b == 0.0)
}

fn ring(points: &[(f64, f64)], dx: f64, dy: f64, k: f64) -> Vec<Vertex> {
    points
        .iter()
        .map(|&(x, y)| Vertex::straight(x * k + dx, y * k + dy))
        .collect()
}

proptest! {
    #[test]
    fn integer_translation_changes_nothing(pts in star_ring(), dx in -1_000_000_i32..1_000_000, dy in -1_000_000_i32..1_000_000) {
        let a = polygon_area(&ring(&pts, 0.0, 0.0, 1.0), None);
        let b = polygon_area(&ring(&pts, f64::from(dx), f64::from(dy), 1.0), None);
        match (a, b) {
            (Ok(a), Ok(b)) => {
                prop_assert_eq!(a.signed_area.to_bits(), b.signed_area.to_bits());
                prop_assert_eq!(a.orientation, b.orientation);
            }
            (Err(a), Err(b)) => prop_assert_eq!(a, b),
            (a, b) => prop_assert!(false, "different outcomes: {:?} vs {:?}", a, b),
        }
    }

    #[test]
    fn reversal_negates_the_signed_area(pts in star_ring()) {
        let mut rev = pts.clone();
        rev.reverse();
        if let (Ok(a), Ok(b)) = (polygon_area(&ring(&pts, 0.0, 0.0, 1.0), None), polygon_area(&ring(&rev, 0.0, 0.0, 1.0), None)) {
            prop_assert!(same(a.signed_area, -b.signed_area));
            prop_assert!(same(a.area, b.area));
        }
    }

    #[test]
    fn power_of_two_scaling_is_exact_and_keeps_decisions(pts in star_ring(), e in -20_i32..20) {
        let k = 2.0_f64.powi(e);
        let a = polygon_area(&ring(&pts, 0.0, 0.0, 1.0), None);
        let b = polygon_area(&ring(&pts, 0.0, 0.0, k), None);
        match (a, b) {
            (Ok(a), Ok(b)) => {
                prop_assert!(same(b.signed_area, a.signed_area * k * k));
                prop_assert_eq!(a.orientation, b.orientation);
                prop_assert_eq!(a.notes, b.notes);
            }
            (Err(a), Err(b)) => prop_assert_eq!(a, b),
            (a, b) => prop_assert!(false, "different outcomes: {:?} vs {:?}", a, b),
        }
        let la = polyline_length(&ring(&pts, 0.0, 0.0, 1.0), true, None).expect("valid input").value;
        let lb = polyline_length(&ring(&pts, 0.0, 0.0, k), true, None).expect("valid input").value;
        prop_assert!((lb - la * k).abs() <= 1e-15 * la * k);
    }

    #[test]
    fn rotation_keeps_area_and_length(pts in star_ring(), phi in 0.0_f64..TAU) {
        let (s, c) = phi.sin_cos();
        let rot: Vec<(f64, f64)> = pts.iter().map(|&(x, y)| (x * c - y * s, x * s + y * c)).collect();
        if let (Ok(a), Ok(b)) = (polygon_area(&ring(&pts, 0.0, 0.0, 1.0), None), polygon_area(&ring(&rot, 0.0, 0.0, 1.0), None)) {
            // Rotating rounds each coordinate by about ε·M, which moves the area by up to about
            // perimeter × ε·M ≤ 8·ε·M·D, however small the area: a thin ring is ill-conditioned (a triangle of
            // area 3 with coordinates near 500 moves by 3 × 10⁻¹¹). The relative term covers the rest.
            let m = pts.iter().fold(0.0_f64, |m, p| m.max(p.0.abs()).max(p.1.abs()));
            let (xs, ys): (Vec<f64>, Vec<f64>) = pts.iter().copied().unzip();
            let span = |v: &[f64]| v.iter().fold(f64::MIN, |a, &b| a.max(b)) - v.iter().fold(f64::MAX, |a, &b| a.min(b));
            let d = span(&xs).hypot(span(&ys));
            let bound = 1e-12 * a.area.max(1.0) + 8.0 * f64::EPSILON * m * d;
            prop_assert!((a.signed_area - b.signed_area).abs() <= bound);
        }
        let la = polyline_length(&ring(&pts, 0.0, 0.0, 1.0), true, None).expect("valid input").value;
        let lb = polyline_length(&ring(&rot, 0.0, 0.0, 1.0), true, None).expect("valid input").value;
        prop_assert!((la - lb).abs() <= 1e-12 * la);
    }

    #[test]
    fn azimuth_components_round_trip(alpha in 0.0_f64..TAU, d in 1e-3_f64..1e7) {
        let (dx, dy) = components(alpha, d).expect("valid input");
        let back = azimuth_from_components(dx, dy).expect("valid input");
        let diff = (back - alpha).abs();
        prop_assert!(diff.min(TAU - diff) <= 4.0 * f64::EPSILON * TAU, "alpha {} back {}", alpha, back);
        prop_assert!((0.0..TAU).contains(&back));
    }

    #[test]
    fn reversed_bulged_ring_negates_area(b0 in -3.0_f64..3.0, b2 in -0.9_f64..0.9) {
        // Reversing a bulged ring moves each bulge to the other end of its segment and negates it.
        // Segments of `fwd`: (0,0)->(10,0) with b0, (10,0)->(10,10), (10,10)->(0,10) with b2, (0,10)->(0,0).
        let fwd = [Vertex::new(0.0, 0.0, b0), Vertex::straight(10.0, 0.0), Vertex::new(10.0, 10.0, b2), Vertex::straight(0.0, 10.0)];
        // Reversed ring (0,10),(10,10),(10,0),(0,0): (0,10)->(10,10) carries -b2 and (10,0)->(0,0) carries -b0.
        let rev = [Vertex::new(0.0, 10.0, -b2), Vertex::straight(10.0, 10.0), Vertex::new(10.0, 0.0, -b0), Vertex::straight(0.0, 0.0)];
        match (polygon_area(&fwd, None), polygon_area(&rev, None)) {
            (Ok(a), Ok(r)) => prop_assert!((a.signed_area + r.signed_area).abs() <= 1e-12 * a.area.max(1.0)),
            (Err(a), Err(r)) => prop_assert_eq!(std::mem::discriminant(&a), std::mem::discriminant(&r)),
            (a, r) => prop_assert!(false, "different outcomes: {:?} vs {:?}", a, r),
        }
    }
}
