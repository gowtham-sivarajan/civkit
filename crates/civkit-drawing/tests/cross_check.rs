// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 15:40 IST · last edited 2026-11-27 23:16 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! ADR 0015 gate X2: what CivKit's reader and `undxf` 0.11.0 both support agrees, for every input both accept:
//! - header: `$ACADVER`, `$INSUNITS`, `$MEASUREMENT`, `$DWGCODEPAGE`, each including its absence;
//! - layers: names, off, frozen;
//! - every supported entity, top level and in every block, in file order: layer, handle, extrusion (where `undxf` keeps
//!   one) and all its geometry: hatch boundaries edge by edge (endpoints, centres, radii, angles, direction);
//! - entities CivKit does not interpret: counts by (type, layer), compared in both directions.
//!
//! Numbers must be equal, except angles, which `undxf` converts to radians. Inputs: the fixtures, the synthetic
//! drawings below (always run), and the fuzz corpus when present (`CIVKIT_REQUIRE_CORPUS=1` makes it mandatory; the
//! fuzz workflow sets it).
//!
//! `undxf` reads a MINSERT as its first placement only, so a MINSERT is compared on that placement; its rows, columns
//! and spacing are checked against the fixtures' expected values in `fixtures.rs`.

#![allow(clippy::float_cmp)] // the reader must keep numbers bit for bit, so exact comparison is the test

mod common;

use std::collections::BTreeMap;
use std::path::PathBuf;

use civkit_drawing::{
    Boundary, Budget, Drawing, Edge, Entity, EntityKind, Limits, Point3, ReadError, read_dxf,
};
use common::*;
use uncad_model::model::{Entity as U, HatchBoundaryPath, HatchEdge, Point3D, Ref, TextOverride};

const ANGLE_TOL: f64 = 1e-12;

/// A layout block: `*Model_Space` or any `*Paper_Space…` (R12 writes `$Model_Space`, `$Paper_Space`).
fn is_layout(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n == "*model_space"
        || n == "$model_space"
        || n.starts_with("*paper_space")
        || n.starts_with("$paper_space")
}

/// A number, or an angle in radians (compared with a tolerance for the degree conversion).
#[derive(Debug, Clone, Copy)]
enum N {
    V(f64),
    Angle(f64),
}

/// One supported entity reduced to what both readers keep: its type, layer, handle, and a flat list of fields.
#[derive(Debug)]
struct S {
    kind: &'static str,
    layer: String,
    handle: Option<u64>,
    fields: Vec<(&'static str, N)>,
    text: Option<String>,
}

fn same(a: &S, b: &S) -> bool {
    a.kind == b.kind
        && a.layer == b.layer
        && a.handle == b.handle
        && a.text == b.text
        && a.fields.len() == b.fields.len()
        && a.fields.iter().zip(&b.fields).all(|((na, x), (nb, y))| {
            na == nb
                && match (x, y) {
                    (N::V(p), N::V(q)) => p == q,
                    (N::Angle(p), N::Angle(q)) => (p - q).abs() <= ANGLE_TOL,
                    _ => false,
                }
        })
}

fn p3(out: &mut Vec<(&'static str, N)>, name: &'static str, p: Point3) {
    out.extend([(name, N::V(p.x)), (name, N::V(p.y)), (name, N::V(p.z))]);
}

fn u3(out: &mut Vec<(&'static str, N)>, name: &'static str, p: &Point3D) {
    out.extend([(name, N::V(p.x)), (name, N::V(p.y)), (name, N::V(p.z))]);
}

fn ours(d: &Drawing, e: &Entity) -> S {
    let mut f = Vec::new();
    let mut text = None;
    let kind = match &e.kind {
        EntityKind::Line { start, end } => {
            p3(&mut f, "start", *start);
            p3(&mut f, "end", *end);
            "LINE"
        }
        EntityKind::Circle { center, radius } => {
            p3(&mut f, "center", *center);
            f.push(("radius", N::V(*radius)));
            p3(&mut f, "extrusion", e.extrusion);
            "CIRCLE"
        }
        EntityKind::Arc {
            center,
            radius,
            start_deg,
            end_deg,
        } => {
            p3(&mut f, "center", *center);
            f.push(("radius", N::V(*radius)));
            f.push(("start", N::Angle(start_deg.to_radians())));
            f.push(("end", N::Angle(end_deg.to_radians())));
            p3(&mut f, "extrusion", e.extrusion);
            "ARC"
        }
        EntityKind::Polyline(p) => {
            f.push(("closed", N::V(f64::from(u8::from(p.closed)))));
            for v in p.vertices.iter() {
                f.extend([("x", N::V(v.x)), ("y", N::V(v.y)), ("bulge", N::V(v.bulge))]);
            }
            f.push(("elevation", N::V(p.elevation)));
            p3(&mut f, "extrusion", e.extrusion);
            "POLYLINE"
        }
        EntityKind::Text(t) => {
            f.extend([("x", N::V(t.point.x)), ("y", N::V(t.point.y))]);
            f.push(("height", N::V(t.height)));
            f.push(("rotation", N::Angle(t.rotation_deg.to_radians())));
            p3(&mut f, "extrusion", e.extrusion);
            text = Some(t.value.to_string());
            "TEXT"
        }
        EntityKind::Insert(i) => {
            text = Some(d.name(i.block).to_owned());
            p3(&mut f, "point", i.point);
            p3(&mut f, "scale", i.scale);
            f.push(("rotation", N::Angle(i.rotation_deg.to_radians())));
            p3(&mut f, "extrusion", e.extrusion);
            "INSERT"
        }
        EntityKind::Hatch(h) => {
            f.push(("solid", N::V(f64::from(u8::from(h.solid)))));
            f.push(("elevation", N::V(h.elevation)));
            p3(&mut f, "extrusion", e.extrusion);
            for path in h.paths.iter() {
                match &path.boundary {
                    Boundary::Polyline { vertices, .. } => {
                        f.push(("polyline path", N::V(vertices.len() as f64)));
                        for v in vertices.iter() {
                            f.extend([
                                ("x", N::V(v.x)),
                                ("y", N::V(v.y)),
                                ("bulge", N::V(v.bulge)),
                            ]);
                        }
                    }
                    Boundary::Edges(edges) => {
                        f.push(("edge path", N::V(edges.len() as f64)));
                        for edge in edges.iter() {
                            match edge {
                                Edge::Line { start, end } => f.extend([
                                    ("line", N::V(start.x)),
                                    ("line", N::V(start.y)),
                                    ("line", N::V(end.x)),
                                    ("line", N::V(end.y)),
                                ]),
                                Edge::Arc {
                                    center,
                                    radius,
                                    start_deg,
                                    end_deg,
                                    counter_clockwise,
                                } => f.extend([
                                    ("arc", N::V(center.x)),
                                    ("arc", N::V(center.y)),
                                    ("arc", N::V(*radius)),
                                    ("arc", N::Angle(start_deg.to_radians())),
                                    ("arc", N::Angle(end_deg.to_radians())),
                                    ("arc", N::V(f64::from(u8::from(*counter_clockwise)))),
                                ]),
                            }
                        }
                    }
                    Boundary::NotSupported { .. } => f.push(("unsupported path", N::V(0.0))),
                }
            }
            "HATCH"
        }
        EntityKind::Dimension(x) => {
            f.push(("type", N::V(f64::from(x.dim_type))));
            f.push(("measurement", N::V(x.measurement.unwrap_or(f64::NAN))));
            p3(&mut f, "first", x.first_point);
            p3(&mut f, "second", x.second_point);
            let shown = match x.text.as_deref() {
                None | Some("") | Some("<>") => "measured".to_owned(),
                Some(" ") => "suppressed".to_owned(),
                Some(s) => format!("override {s}"),
            };
            text = Some(format!(
                "{shown} | block {}",
                x.block.map_or("", |b| d.name(b))
            ));
            "DIMENSION"
        }
        other => panic!(
            "the cross-check does not know {}: add it here",
            other.dxf_name()
        ),
    };
    S {
        kind,
        layer: d.name(e.layer).to_owned(),
        handle: e.handle,
        fields: f,
        text,
    }
}

/// The name a reference states, resolved or not (`Ref::name` gives `""` for an unresolved one, such as a layer missing
/// from the LAYER table or a dimension's block that is not defined); `None` when the file states none.
fn written(r: &Ref<String>) -> Option<&str> {
    match r {
        Ref::Resolved(s) | Ref::Unresolved(s) => Some(s),
        Ref::Absent => None,
    }
}

fn layer_of(c: &uncad_model::model::EntityCommon) -> String {
    // No group 8: DXF's default layer "0"; undxf leaves the reference absent.
    written(&c.layer).unwrap_or("0").to_owned()
}

fn handle_of(c: &uncad_model::model::EntityCommon) -> Option<u64> {
    written(&c.source_handle).and_then(|h| u64::from_str_radix(h, 16).ok())
}

/// `undxf`'s entity reduced the same way, or `None` for a type CivKit does not interpret.
fn theirs(e: &U) -> Option<S> {
    let mut f = Vec::new();
    let mut text = None;
    let (kind, common) = match e {
        U::Line(l) => {
            u3(&mut f, "start", &l.start_point);
            u3(&mut f, "end", &l.end_point);
            ("LINE", &l.common)
        }
        U::Circle(c) => {
            u3(&mut f, "center", &c.center);
            f.push(("radius", N::V(c.radius)));
            u3(&mut f, "extrusion", &c.extrusion);
            ("CIRCLE", &c.common)
        }
        U::Arc(a) => {
            u3(&mut f, "center", &a.center);
            f.push(("radius", N::V(a.radius)));
            f.push(("start", N::Angle(a.start_angle)));
            f.push(("end", N::Angle(a.end_angle)));
            u3(&mut f, "extrusion", &a.extrusion);
            ("ARC", &a.common)
        }
        U::LwPolyline(p) | U::Polyline2D(p) => {
            f.push(("closed", N::V(f64::from(u8::from(p.closed)))));
            for v in &p.vertices {
                f.extend([
                    ("x", N::V(v.point.x)),
                    ("y", N::V(v.point.y)),
                    ("bulge", N::V(v.bulge)),
                ]);
            }
            f.push(("elevation", N::V(p.elevation)));
            u3(&mut f, "extrusion", &p.extrusion);
            ("POLYLINE", &p.common)
        }
        U::Text(t) => {
            f.extend([("x", N::V(t.start_point.x)), ("y", N::V(t.start_point.y))]);
            f.push(("height", N::V(t.text_height)));
            f.push(("rotation", N::Angle(t.rotation)));
            u3(&mut f, "extrusion", &t.extrusion);
            text = Some(t.text.clone());
            ("TEXT", &t.common)
        }
        U::Insert(i) => {
            text = Some(written(&i.block_name).unwrap_or("").to_owned());
            u3(&mut f, "point", &i.insertion_point);
            u3(&mut f, "scale", &i.scale);
            f.push(("rotation", N::Angle(i.rotation)));
            u3(&mut f, "extrusion", &i.extrusion);
            ("INSERT", &i.common)
        }
        U::Hatch(h) => {
            f.push(("solid", N::V(f64::from(u8::from(h.solid_fill)))));
            f.push(("elevation", N::V(h.elevation)));
            u3(&mut f, "extrusion", &h.extrusion);
            for path in &h.boundary_paths {
                match path {
                    HatchBoundaryPath::Polyline(v) => {
                        f.push(("polyline path", N::V(v.len() as f64)));
                        for v in v {
                            f.extend([
                                ("x", N::V(v.point.x)),
                                ("y", N::V(v.point.y)),
                                ("bulge", N::V(v.bulge)),
                            ]);
                        }
                    }
                    HatchBoundaryPath::Edges(edges) => {
                        if edges
                            .iter()
                            .any(|e| !matches!(e, HatchEdge::Line { .. } | HatchEdge::Arc { .. }))
                        {
                            f.push(("unsupported path", N::V(0.0)));
                            continue;
                        }
                        f.push(("edge path", N::V(edges.len() as f64)));
                        for edge in edges {
                            match edge {
                                HatchEdge::Line { start, end } => f.extend([
                                    ("line", N::V(start.x)),
                                    ("line", N::V(start.y)),
                                    ("line", N::V(end.x)),
                                    ("line", N::V(end.y)),
                                ]),
                                HatchEdge::Arc {
                                    center,
                                    radius,
                                    start_angle,
                                    end_angle,
                                    is_ccw,
                                } => f.extend([
                                    ("arc", N::V(center.x)),
                                    ("arc", N::V(center.y)),
                                    ("arc", N::V(*radius)),
                                    ("arc", N::Angle(*start_angle)),
                                    ("arc", N::Angle(*end_angle)),
                                    ("arc", N::V(f64::from(u8::from(*is_ccw)))),
                                ]),
                                _ => unreachable!("filtered above"),
                            }
                        }
                    }
                }
            }
            ("HATCH", &h.common)
        }
        U::Dimension(x) => {
            let kind_code = x.kind.as_ref().map_or(f64::NAN, |k| {
                serde_json::to_value(k)
                    .ok()
                    .and_then(|v| v.as_str().map(dim_code))
                    .unwrap_or(f64::NAN)
            });
            f.push(("type", N::V(kind_code)));
            f.push(("measurement", N::V(x.measurement.unwrap_or(f64::NAN))));
            let zero = Point3D {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            };
            u3(
                &mut f,
                "first",
                x.points.extension1.as_ref().unwrap_or(&zero),
            );
            u3(
                &mut f,
                "second",
                x.points.extension2.as_ref().unwrap_or(&zero),
            );
            let shown = match &x.text_override {
                TextOverride::Measured => "measured".to_owned(),
                TextOverride::Suppressed => "suppressed".to_owned(),
                TextOverride::Literal(s) => format!("override {s}"),
            };
            text = Some(format!(
                "{shown} | block {}",
                written(&x.block_name).unwrap_or("")
            ));
            ("DIMENSION", &x.common)
        }
        _ => return None,
    };
    Some(S {
        kind,
        layer: layer_of(common),
        handle: handle_of(common),
        fields: f,
        text,
    })
}

/// `undxf`'s dimension kind (as it serialises), as DXF group 70's low bits.
fn dim_code(name: &str) -> f64 {
    match name.to_ascii_uppercase().replace('_', "").as_str() {
        "ROTATED" => 0.0,
        "ALIGNED" => 1.0,
        "ANGULAR2LINE" => 2.0,
        "DIAMETER" => 3.0,
        "RADIUS" => 4.0,
        "ANGULAR3POINT" => 5.0,
        "ORDINATE" => 6.0,
        _ => f64::NAN,
    }
}

/// NaN stands for "not stated" on both sides; make it compare equal to itself.
fn normalise(mut s: S) -> S {
    for (_, n) in &mut s.fields {
        if let N::V(v) = n
            && v.is_nan()
        {
            *v = -0.123_456_789; // a sentinel no input uses
        }
    }
    s
}

fn compare_lists(what: &str, d: &Drawing, a: &[&Entity], b: &[U]) {
    let a: Vec<S> = a.iter().map(|e| normalise(ours(d, e))).collect();
    // undxf also lists an INSERT's attributes at the top level; those are not supported entities, so they drop out here.
    let b: Vec<S> = b.iter().filter_map(theirs).map(normalise).collect();
    assert_eq!(
        a.len(),
        b.len(),
        "{what}: {} supported entities vs {} in undxf\nours {a:#?}\nundxf {b:#?}",
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

/// `undxf`'s name for a type CivKit counts, in CivKit's spelling.
fn civkit_name(e: &U) -> String {
    match e {
        U::Unknown { type_name, .. } => type_name.clone(),
        U::Polyline3D(_) => "POLYLINE (3D)".to_owned(),
        U::PolylineMesh(_) => "POLYLINE (mesh)".to_owned(),
        U::PolylinePFace(_) => "POLYLINE (polyface)".to_owned(),
        other => serde_json::to_value(other).expect("serialises")["type"]
            .as_str()
            .unwrap_or("?")
            .to_owned(),
    }
}

fn check(what: &str, bytes: &[u8], d: &Drawing) {
    let (db, header) =
        undxf::read_bytes_with_header(bytes).unwrap_or_else(|e| panic!("{what}: undxf: {e}"));
    // Header values, and their absence.
    let first = |name: &str| {
        header
            .groups(name)
            .and_then(|g| g.first())
            .map(|g| g.value.trim().to_owned())
    };
    assert_eq!(
        d.acadver().map(str::to_owned),
        first("ACADVER"),
        "{what}: $ACADVER"
    );
    assert_eq!(
        d.codepage().map(str::to_owned),
        first("DWGCODEPAGE"),
        "{what}: $DWGCODEPAGE"
    );
    assert_eq!(
        d.insunits().map(i32::from),
        db.header.insunits.map(i32::from),
        "{what}: $INSUNITS"
    );
    let measurement = first("MEASUREMENT").map(|v| v.parse::<i32>().ok());
    assert_eq!(
        d.measurement().map(|m| Some(i32::from(m))),
        measurement,
        "{what}: $MEASUREMENT"
    );
    // Layers, with their state.
    let mut ours_layers: Vec<(String, bool, bool)> = d
        .layers()
        .iter()
        .map(|l| (d.name(l.name).to_owned(), l.off, l.frozen))
        .collect();
    ours_layers.sort();
    let theirs_layers: Vec<(String, bool, bool)> = db
        .tables
        .layers
        .values()
        .map(|l| (l.name.clone(), l.off, l.frozen))
        .collect();
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
    // Uninterpreted entities by (type, layer), in both directions. undxf lists layout-block entities both at the top
    // level and under their block record, so those records are left out here.
    let mut ours_counts: BTreeMap<(String, String), u64> = BTreeMap::new();
    for (kind, layer, n) in d.unsupported() {
        *ours_counts
            .entry((kind.to_owned(), layer.to_owned()))
            .or_default() += n;
    }
    let mut theirs_counts: BTreeMap<(String, String), u64> = BTreeMap::new();
    let non_layout = db
        .tables
        .block_records
        .values()
        .filter(|r| !is_layout(&r.name))
        .flat_map(|r| r.entities.iter());
    for e in db.entities.iter().chain(non_layout) {
        if theirs(e).is_none() {
            *theirs_counts
                .entry((civkit_name(e), layer_of(e.common())))
                .or_default() += 1;
        }
    }
    assert_eq!(
        ours_counts, theirs_counts,
        "{what}: uninterpreted entities by (type, layer)"
    );
}

#[test]
fn every_fixture_agrees_with_undxf() {
    for name in ALL_FIXTURES {
        let bytes = fixture_bytes(name);
        check(name, &bytes, &read(name));
    }
}

/// The curated well-formed corpus (ADR 0015 X2, strict): hand-authored DXF files that exercise what the fixtures do
/// not. A disagreement on a valid DXF found in the fuzz corpus is promoted here.
fn well_formed() -> Vec<(String, Vec<u8>)> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/well_formed");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("tests/well_formed")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "dxf"))
        .collect();
    files.sort();
    assert!(files.len() >= 3, "the curated corpus is missing files");
    files
        .into_iter()
        .map(|p| {
            (
                p.file_name().expect("name").to_string_lossy().into_owned(),
                std::fs::read(&p).expect("read"),
            )
        })
        .collect()
}

#[test]
fn every_well_formed_case_agrees_with_undxf() {
    for (name, bytes) in well_formed() {
        let d = read_dxf(&bytes, &Budget::new(Limits::DEFAULT_BUDGET_BYTES))
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        // Each case must exercise something, or the comparison proves little.
        assert!(!d.layers().is_empty() && d.entities().len() >= 4, "{name}");
        check(&name, &bytes, &d);
    }
}

/// FNV-1a, 64-bit: a stable name for an input in the report (libFuzzer also names corpus files by their SHA-1).
fn fnv64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// The fuzz corpus (ADR 0015 X2, amended: report and triage). Every file both readers accept is compared; each
/// disagreement is recorded with the input's hash and the first differing field (printed, and appended to the CI job
/// summary), and the test passes. It fails only on what CivKit must never do: a panic (this test would abort), a read
/// slower than 10 s (a hang), or a budget or allocation failure on an input of at most 1 MiB. `CIVKIT_REQUIRE_CORPUS`
/// makes a missing or empty corpus a failure (the fuzz workflow sets it).
#[test]
fn fuzz_corpus_disagreements_are_reported() {
    use std::io::Write;
    let required = std::env::var_os("CIVKIT_REQUIRE_CORPUS").is_some();
    let corpus = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fuzz/corpus/read_dxf");
    let mut entries: Vec<PathBuf> = match std::fs::read_dir(&corpus) {
        Ok(dir) => dir.flatten().map(|e| e.path()).collect(),
        Err(_) => Vec::new(),
    };
    entries.sort();
    assert!(
        !required || !entries.is_empty(),
        "CIVKIT_REQUIRE_CORPUS is set but {} is missing or empty",
        corpus.display()
    );
    let (mut compared, mut disagreements) = (0usize, Vec::new());
    // Comparison failures are recorded, not shown as panics.
    let quiet = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    for path in &entries {
        let bytes = std::fs::read(path).expect("corpus file");
        let name = path
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        let start = std::time::Instant::now();
        let ours = read_dxf(&bytes, &Budget::new(Limits::DEFAULT_BUDGET_BYTES));
        let took = start.elapsed();
        assert!(
            took.as_secs() < 10,
            "{name}: reading took {took:?} (a hang)"
        );
        let d = match ours {
            Ok(d) => d,
            Err(e @ (ReadError::BudgetExhausted { .. } | ReadError::AllocationFailed { .. })) => {
                std::panic::set_hook(quiet);
                panic!(
                    "{name} ({} bytes): {e}: a budget or allocation failure on a small input",
                    bytes.len()
                );
            }
            Err(_) => continue, // CivKit refuses it: covered by its own refusal and safety tests
        };
        let accepted =
            std::panic::catch_unwind(|| undxf::read_bytes(&bytes).is_ok()).unwrap_or(false);
        if !accepted {
            continue;
        }
        compared += 1;
        if let Err(e) =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| check(&name, &bytes, &d)))
        {
            let message = e
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| e.downcast_ref::<&str>().map(|s| (*s).to_owned()))
                .unwrap_or_default();
            let first: String = message
                .lines()
                .take(3)
                .collect::<Vec<_>>()
                .join(" / ")
                .chars()
                .take(400)
                .collect();
            disagreements.push(format!(
                "| `{name}` | `{:016x}` | {} | {} |",
                fnv64(&bytes),
                bytes.len(),
                first.replace('|', r"\|")
            ));
        }
    }
    std::panic::set_hook(quiet);
    let mut report = format!(
        "### X2 fuzz-corpus cross-check (report and triage)\n\n{} corpus files; {compared} accepted by both \
         readers; {} disagreements.\n",
        entries.len(),
        disagreements.len()
    );
    if !disagreements.is_empty() {
        report.push_str("\n| Input | FNV-1a 64 | Bytes | First difference |\n|---|---|---|---|\n");
        for line in disagreements.iter().take(100) {
            report.push_str(line);
            report.push('\n');
        }
        if disagreements.len() > 100 {
            report.push_str(&format!("\n… and {} more.\n", disagreements.len() - 100));
        }
        report.push_str(
            "\nTriage each one (ADR 0015, amendment): a valid DXF goes into tests/well_formed/; an ambiguity \
             CivKit should refuse gets a refusal and a test; a tolerance difference on malformed input is logged \
             in the maintainer's triage log.\n",
        );
    }
    println!("{report}");
    if let Some(summary) = std::env::var_os("GITHUB_STEP_SUMMARY")
        && let Ok(mut f) = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(summary)
    {
        let _ = f.write_all(report.as_bytes());
    }
    assert!(
        !required || compared > 0,
        "CIVKIT_REQUIRE_CORPUS is set but no corpus file was readable by both readers"
    );
}
