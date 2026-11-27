// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 15:40 IST · last edited 2026-11-27 15:40 IST

//! Helpers shared by the reader's tests. The geometry here (shoelace area, bulge arc length) is test-only: it checks
//! that the reader kept the right numbers, against values the fixtures README fixes by construction.

#![allow(dead_code, clippy::float_cmp)] // shared helpers; exact comparison is intended

use std::collections::BTreeMap;
use std::path::PathBuf;

use civkit_drawing::{Budget, Drawing, Entity, EntityKind, Limits, Vertex, read_dxf};

pub fn fixture_path(name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/dxf");
    let direct = root.join(name);
    if direct.exists() {
        direct
    } else {
        root.join("third_party/ezdxf").join(name)
    }
}

pub fn fixture_bytes(name: &str) -> Vec<u8> {
    std::fs::read(fixture_path(name)).unwrap_or_else(|e| panic!("{name}: {e}"))
}

pub const ALL_FIXTURES: [&str; 7] = [
    "f1_site_plan_m.dxf",
    "f2_site_plan_mm.dxf",
    "f3_units_mismatch.dxf",
    "f4_legacy_r12.dxf",
    "f5_blocks_inserts.dxf",
    "Minimal_DXF_AC1009.dxf",
    "insert_bricscad_level_1.dxf",
];

pub fn read(name: &str) -> Drawing {
    let budget = Budget::new(Limits::DEFAULT_BUDGET_BYTES);
    read_dxf(&fixture_bytes(name), &budget).unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// Entity type names, as the DXF file names them, counted.
pub fn kinds(entities: &[Entity]) -> BTreeMap<&'static str, usize> {
    let mut out = BTreeMap::new();
    for e in entities {
        *out.entry(e.kind.dxf_name()).or_default() += 1;
    }
    out
}

pub fn on_layer<'a>(d: &'a Drawing, entities: &'a [Entity], layer: &str) -> Vec<&'a Entity> {
    entities
        .iter()
        .filter(|e| d.name(e.layer) == layer)
        .collect()
}

pub fn polylines<'a>(entities: &[&'a Entity]) -> Vec<&'a civkit_drawing::Polyline> {
    entities
        .iter()
        .filter_map(|e| match &e.kind {
            EntityKind::Polyline(p) => Some(p),
            _ => None,
        })
        .collect()
}

/// Area of a closed polygon with straight edges (all bulges zero).
pub fn straight_area(v: &[Vertex]) -> f64 {
    assert!(v.iter().all(|x| x.bulge == 0.0), "straight edges only");
    let n = v.len();
    (0..n)
        .map(|i| v[i].x * v[(i + 1) % n].y - v[(i + 1) % n].x * v[i].y)
        .sum::<f64>()
        .abs()
        / 2.0
}

/// Length of a polyline whose segments may bulge: a bulge b makes the segment an arc of included angle 4·atan(b).
pub fn length(v: &[Vertex], closed: bool) -> f64 {
    let n = v.len();
    let segments = if closed { n } else { n - 1 };
    (0..segments)
        .map(|i| {
            let (a, b) = (&v[i], &v[(i + 1) % n]);
            let chord = ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt();
            if a.bulge == 0.0 {
                chord
            } else {
                let theta = 4.0 * a.bulge.atan();
                chord * (theta / 2.0) / (theta / 2.0).sin()
            }
        })
        .sum()
}

pub fn close(a: f64, b: f64, rel: f64) -> bool {
    (a - b).abs() <= rel * b.abs().max(1.0)
}
