// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 15:40 IST · last edited 2026-11-27 15:40 IST

//! ADR 0015 gate X2: for every fixture (and every fuzz-corpus file both readers accept), what CivKit's reader and
//! `undxf` 0.11.0 both support agrees: header values and their absence, layers, blocks, and each supported entity in
//! file order. Numbers must be equal, except angles, which `undxf` converts to radians.
//!
//! `undxf` reads a MINSERT as its first placement only, so a MINSERT is compared on that placement; its rows, columns
//! and spacing are checked against the fixtures' expected values in `fixtures.rs`.

#![allow(clippy::float_cmp)] // the reader must keep numbers bit for bit, so exact comparison is the test

mod common;

use std::collections::BTreeMap;
use std::path::PathBuf;

use civkit_drawing::{Boundary, Budget, Drawing, Edge, Entity, EntityKind, Limits, read_dxf};
use common::*;
use uncad_model::model::{Entity as U, HatchBoundaryPath, HatchEdge, TextOverride};

/// One supported entity, reduced to what both readers keep. Angles in radians.
#[derive(Debug, Clone)]
enum S {
    Line([f64; 6]),
    Circle([f64; 4]),
    Arc([f64; 4], f64, f64),
    Poly { closed: bool, v: Vec<[f64; 3]> },
    Text(String, [f64; 2]),
    Insert(String, [f64; 3], [f64; 3], f64),
    Hatch(Vec<Option<Vec<[f64; 3]>>>),
    Dim(Option<f64>, String),
}

const ANGLE_TOL: f64 = 1e-12;

/// A layout block: `*Model_Space` or any `*Paper_Space…` (R12 writes `$Model_Space`, `$Paper_Space`).
fn is_layout(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n == "*model_space"
        || n == "$model_space"
        || n.starts_with("*paper_space")
        || n.starts_with("$paper_space")
}

fn same(a: &S, b: &S) -> bool {
    let ang = |x: f64, y: f64| (x - y).abs() <= ANGLE_TOL;
    match (a, b) {
        (S::Line(x), S::Line(y)) => x == y,
        (S::Circle(x), S::Circle(y)) => x == y,
        (S::Arc(x, s1, e1), S::Arc(y, s2, e2)) => x == y && ang(*s1, *s2) && ang(*e1, *e2),
        (S::Poly { closed: c1, v: v1 }, S::Poly { closed: c2, v: v2 }) => c1 == c2 && v1 == v2,
        (S::Text(t1, p1), S::Text(t2, p2)) => t1 == t2 && p1 == p2,
        (S::Insert(n1, p1, s1, r1), S::Insert(n2, p2, s2, r2)) => {
            n1 == n2 && p1 == p2 && s1 == s2 && ang(*r1, *r2)
        }
        (S::Hatch(h1), S::Hatch(h2)) => h1 == h2,
        (S::Dim(m1, t1), S::Dim(m2, t2)) => m1 == m2 && t1 == t2,
        _ => false,
    }
}

fn ours(d: &Drawing, e: &Entity) -> Option<S> {
    Some(match &e.kind {
        EntityKind::Line { start, end } => {
            S::Line([start.x, start.y, start.z, end.x, end.y, end.z])
        }
        EntityKind::Circle { center, radius } => S::Circle([center.x, center.y, center.z, *radius]),
        EntityKind::Arc {
            center,
            radius,
            start_deg,
            end_deg,
        } => S::Arc(
            [center.x, center.y, center.z, *radius],
            start_deg.to_radians(),
            end_deg.to_radians(),
        ),
        EntityKind::Polyline(p) => S::Poly {
            closed: p.closed,
            v: p.vertices.iter().map(|v| [v.x, v.y, v.bulge]).collect(),
        },
        EntityKind::Text(t) => S::Text(t.value.to_string(), [t.point.x, t.point.y]),
        EntityKind::Insert(i) => S::Insert(
            d.name(i.block).to_owned(),
            [i.point.x, i.point.y, i.point.z],
            [i.scale.x, i.scale.y, i.scale.z],
            i.rotation_deg.to_radians(),
        ),
        EntityKind::Hatch(h) => S::Hatch(
            h.paths
                .iter()
                .map(|p| match &p.boundary {
                    Boundary::Polyline { vertices, .. } => {
                        Some(vertices.iter().map(|v| [v.x, v.y, v.bulge]).collect())
                    }
                    Boundary::Edges(edges) => Some(
                        edges
                            .iter()
                            .map(|e| match e {
                                Edge::Line { start, .. } => [start.x, start.y, 0.0],
                                Edge::Arc { center, .. } => [center.x, center.y, 1.0],
                            })
                            .collect(),
                    ),
                    Boundary::NotSupported { .. } => None,
                })
                .collect(),
        ),
        EntityKind::Dimension(x) => {
            let text = match x.text.as_deref() {
                None | Some("") | Some("<>") => "measured".to_owned(),
                Some(" ") => "suppressed".to_owned(),
                Some(s) => format!("override {s}"),
            };
            S::Dim(x.measurement, text)
        }
        other => panic!(
            "the cross-check does not know {}: add it here",
            other.dxf_name()
        ),
    })
}

fn theirs(e: &U) -> Option<S> {
    Some(match e {
        U::Line(l) => S::Line([
            l.start_point.x,
            l.start_point.y,
            l.start_point.z,
            l.end_point.x,
            l.end_point.y,
            l.end_point.z,
        ]),
        U::Circle(c) => S::Circle([c.center.x, c.center.y, c.center.z, c.radius]),
        U::Arc(a) => S::Arc(
            [a.center.x, a.center.y, a.center.z, a.radius],
            a.start_angle,
            a.end_angle,
        ),
        U::LwPolyline(p) | U::Polyline2D(p) => S::Poly {
            closed: p.closed,
            v: p.vertices
                .iter()
                .map(|v| [v.point.x, v.point.y, v.bulge])
                .collect(),
        },
        U::Text(t) => S::Text(t.text.clone(), [t.start_point.x, t.start_point.y]),
        U::Insert(i) => S::Insert(
            i.block_name.name().to_owned(),
            [
                i.insertion_point.x,
                i.insertion_point.y,
                i.insertion_point.z,
            ],
            [i.scale.x, i.scale.y, i.scale.z],
            i.rotation,
        ),
        U::Hatch(h) => S::Hatch(
            h.boundary_paths
                .iter()
                .map(|p| match p {
                    HatchBoundaryPath::Polyline(v) => {
                        Some(v.iter().map(|v| [v.point.x, v.point.y, v.bulge]).collect())
                    }
                    HatchBoundaryPath::Edges(edges) => edges
                        .iter()
                        .map(|e| match e {
                            HatchEdge::Line { start, .. } => Some([start.x, start.y, 0.0]),
                            HatchEdge::Arc { center, .. } => Some([center.x, center.y, 1.0]),
                            _ => None,
                        })
                        .collect(),
                })
                .collect(),
        ),
        U::Dimension(x) => {
            let text = match &x.text_override {
                TextOverride::Measured => "measured".to_owned(),
                TextOverride::Suppressed => "suppressed".to_owned(),
                TextOverride::Literal(s) => format!("override {s}"),
            };
            S::Dim(x.measurement, text)
        }
        _ => return None,
    })
}

fn compare_lists(what: &str, d: &Drawing, a: &[&Entity], b: &[U]) {
    let a: Vec<S> = a.iter().filter_map(|e| ours(d, e)).collect();
    // undxf also lists an INSERT's attributes at the top level; those are not supported entities, so they drop out here.
    let b: Vec<S> = b.iter().filter_map(theirs).collect();
    assert_eq!(
        a.len(),
        b.len(),
        "{what}: {} supported entities vs {} in undxf\nours {a:?}\nundxf {b:?}",
        a.len(),
        b.len()
    );
    for (k, (x, y)) in a.iter().zip(&b).enumerate() {
        assert!(
            same(x, y),
            "{what}, entity {k}:\n  ours  {x:?}\n  undxf {y:?}"
        );
    }
}

fn check(what: &str, bytes: &[u8], d: &Drawing) {
    let (db, header): (uncad_model::CadDatabase, undxf::Header) =
        undxf::read_bytes_with_header(bytes).unwrap_or_else(|e| panic!("{what}: undxf: {e}"));
    // Header values, and their absence.
    assert_eq!(
        d.insunits().map(i32::from),
        db.header.insunits.map(i32::from),
        "{what}: $INSUNITS"
    );
    let their_measurement = header
        .groups("MEASUREMENT")
        .and_then(|g| g.first())
        .map(|g| g.value.trim().parse::<i32>().ok());
    assert_eq!(
        d.measurement().map(|m| Some(i32::from(m))),
        their_measurement,
        "{what}: $MEASUREMENT"
    );
    let their_version = header
        .groups("ACADVER")
        .and_then(|g| g.first())
        .map(|g| g.value.trim().to_owned());
    assert_eq!(
        d.acadver().map(str::to_owned),
        their_version,
        "{what}: $ACADVER"
    );
    // Layers.
    let mut ours_layers: Vec<&str> = d.layers().iter().map(|l| d.name(l.name)).collect();
    ours_layers.sort_unstable();
    let theirs_layers: Vec<&str> = db.tables.layers.keys().map(String::as_str).collect();
    assert_eq!(ours_layers, theirs_layers, "{what}: layers");
    // Top level: undxf's list is what the layout blocks (*Model_Space, *Paper_Space*) own, so compare it with the
    // ENTITIES section plus whatever CivKit's layout blocks hold. Then every other block both readers have.
    let mut top: Vec<&Entity> = d.entities().iter().collect();
    for b in d.blocks().iter().filter(|b| is_layout(d.name(b.name))) {
        top.extend(b.entities.iter());
    }
    compare_lists(&format!("{what} ENTITIES"), d, &top, &db.entities);
    for b in d.blocks().iter().filter(|b| !is_layout(d.name(b.name))) {
        let name = d.name(b.name);
        let Some(record) = db.tables.block_records.get(name) else {
            panic!("{what}: undxf has no block {name}")
        };
        let mine: Vec<&Entity> = b.entities.iter().collect();
        compare_lists(&format!("{what} block {name}"), d, &mine, &record.entities);
    }
    // Types CivKit counts as unsupported: the same number of each in undxf, top level and non-layout blocks together
    // (undxf lists layout-block entities both at the top level and under their block record).
    let mut ours_unsupported: BTreeMap<String, u64> = BTreeMap::new();
    for (kind, _, n) in d.unsupported() {
        *ours_unsupported.entry(kind.to_owned()).or_default() += n;
    }
    let mut theirs_all: BTreeMap<String, u64> = BTreeMap::new();
    let non_layout = db
        .tables
        .block_records
        .values()
        .filter(|r| !is_layout(&r.name))
        .flat_map(|r| r.entities.iter());
    for e in db.entities.iter().chain(non_layout) {
        let name = match e {
            U::Unknown { type_name, .. } => type_name.clone(),
            other => serde_json::to_value(other).expect("serialises")["type"]
                .as_str()
                .unwrap_or("?")
                .to_owned(),
        };
        *theirs_all.entry(name).or_default() += 1;
    }
    for (kind, n) in &ours_unsupported {
        assert_eq!(
            theirs_all.get(kind).copied().unwrap_or(0),
            *n,
            "{what}: count of {kind}"
        );
    }
}

#[test]
fn every_fixture_agrees_with_undxf() {
    for name in ALL_FIXTURES {
        let bytes = fixture_bytes(name);
        check(name, &bytes, &read(name));
    }
}

#[test]
fn every_corpus_file_both_readers_accept_agrees() {
    let corpus = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fuzz/corpus/read_dxf");
    let Ok(dir) = std::fs::read_dir(&corpus) else {
        return;
    };
    let mut checked = 0;
    for entry in dir.flatten() {
        let bytes = std::fs::read(entry.path()).expect("corpus file");
        let Ok(d) = read_dxf(&bytes, &Budget::new(Limits::DEFAULT_BUDGET_BYTES)) else {
            continue;
        };
        if undxf::read_bytes(&bytes).is_err() {
            continue;
        }
        check(&entry.path().display().to_string(), &bytes, &d);
        checked += 1;
    }
    assert!(
        checked > 0
            || corpus
                .read_dir()
                .map(|mut d| d.next().is_none())
                .unwrap_or(true)
    );
}
