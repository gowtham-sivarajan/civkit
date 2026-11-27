// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 20:38 IST · last edited 2026-11-27 21:34 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! Quantities by effective layer (D-28 / ADR 0016 §6–§7; gates A1, A6, A5 in part).
//!
//! Expected values are the fixtures README's (fixed by construction and checked with other programs)
//! or closed forms worked by hand in each test. None is produced by the code under test.

#![allow(clippy::float_cmp)] // counts and exact values are compared exactly where every step is exact

mod common;

use std::f64::consts::PI;

use civkit_drawing::{
    AnalysisOptions, Budget, Detail, Drawing, Issue, LayerTotals, Level, Limits, Quantities,
    measure, read_dxf,
};
use common::*;

fn close_rel(actual: f64, expected: f64, rel: f64, what: &str) {
    let scale = expected.abs().max(f64::MIN_POSITIVE);
    assert!(
        ((actual - expected) / scale).abs() <= rel,
        "{what}: expected {expected:e}, got {actual:e}"
    );
}

fn measured(d: &Drawing) -> Quantities {
    measure(
        d,
        &Budget::new(Limits::DEFAULT_BUDGET_BYTES),
        &AnalysisOptions::default(),
    )
    .expect("measures")
}

fn layer<'q>(d: &Drawing, q: &'q Quantities, name: &str) -> &'q LayerTotals {
    q.layers()
        .iter()
        .find(|l| d.name(l.layer) == name)
        .unwrap_or_else(|| panic!("no layer {name}"))
}

fn issues<'q>(q: &'q Quantities, code: &str) -> Vec<&'q Issue> {
    q.issues().iter().filter(|i| i.code == code).collect()
}

fn cautions(q: &Quantities) -> Vec<&Issue> {
    q.issues()
        .iter()
        .filter(|i| i.level == Level::Caution)
        .collect()
}

// Fixtures ------------------------------------------------------------------------------------------

/// F1 (metres) and F4 (same geometry, R12, units not stated), README values; F2 is F1 × 1000 in mm.
fn check_site_plan(d: &Drawing, q: &Quantities, k: f64) {
    let r = 1e-9;
    let b = layer(d, q, "BOUNDARY");
    assert_eq!(b.counts.polyline_closed, 1);
    close_rel(
        b.polyline_closed_perimeter,
        200.0 * k,
        r,
        "boundary perimeter",
    );
    close_rel(b.polyline_closed_area, 2550.0 * k * k, r, "boundary area");
    let road = layer(d, q, "ROAD_CL");
    assert_eq!(road.counts.polyline_open, 1);
    close_rel(
        road.polyline_open_length,
        (100.0 + 10.0 * PI) * k,
        r,
        "road",
    );
    let bld = layer(d, q, "BUILDING");
    assert_eq!(bld.counts.polyline_closed, 2);
    close_rel(bld.polyline_closed_area, 196.0 * k * k, r, "buildings area");
    close_rel(
        bld.polyline_closed_perimeter,
        80.0 * k,
        r,
        "buildings perimeter",
    );
    let w = layer(d, q, "WATER");
    assert_eq!(w.counts.circle, 1);
    close_rel(w.circle_area, 9.0 * PI * k * k, r, "water area");
    close_rel(
        w.circle_circumference,
        6.0 * PI * k,
        r,
        "water circumference",
    );
    let dr = layer(d, q, "DRAIN");
    assert_eq!((dr.counts.arc, dr.counts.line), (1, 1));
    close_rel(dr.arc_length, 5.0 * PI * k, r, "drain arc");
    close_rel(dr.line_length, 25.0 * k, r, "drain line");
    assert_eq!(layer(d, q, "TEXT").counts.text, 1);
    for l in q.layers() {
        assert!(!l.incomplete(), "{} incomplete", d.name(l.layer));
    }
}

#[test]
fn f1_totals_by_layer() {
    let d = read("f1_site_plan_m.dxf");
    let q = measured(&d);
    check_site_plan(&d, &q, 1.0);
    // The hatch over building 1: area 96. The dimension is counted.
    let h = layer(&d, &q, "HATCH");
    assert_eq!(h.counts.hatch, 1);
    close_rel(h.hatch_area, 96.0, 1e-9, "hatch");
    assert_eq!(layer(&d, &q, "DIM").counts.dimension, 1);
    // Metres: SI equals drawing units.
    assert_eq!(q.units().insunits, Some(6));
    assert_eq!(q.units().to_metres, Some(1.0));
    // The dimension's own block holds MTEXT and SOLID, never placed: listed, no caution.
    assert!(cautions(&q).is_empty(), "{:?}", cautions(&q));
    let listed: Vec<String> = issues(&q, "NOT_SUPPORTED")
        .iter()
        .map(|i| match i.detail {
            Detail::EntityType(t) => d.name(t).to_owned(),
            other => panic!("{other:?}"),
        })
        .collect();
    assert!(listed.contains(&"MTEXT".to_owned()) && listed.contains(&"SOLID".to_owned()));
}

#[test]
fn f2_in_metres_equals_f1() {
    let (d1, d2) = (read("f1_site_plan_m.dxf"), read("f2_site_plan_mm.dxf"));
    let (q1, q2) = (measured(&d1), measured(&d2));
    check_site_plan(&d2, &q2, 1000.0);
    let u = q2.units();
    assert_eq!((u.insunits, u.to_metres), (Some(4), Some(1e-3)));
    for name in ["BOUNDARY", "BUILDING", "WATER", "DRAIN", "ROAD_CL", "HATCH"] {
        let (a, b) = (layer(&d1, &q1, name), layer(&d2, &q2, name));
        for (x, y) in [
            (
                a.polyline_closed_area,
                u.area_m2(b.polyline_closed_area).expect("units"),
            ),
            (a.hatch_area, u.area_m2(b.hatch_area).expect("units")),
            (a.circle_area, u.area_m2(b.circle_area).expect("units")),
            (
                a.polyline_open_length,
                u.length_m(b.polyline_open_length).expect("units"),
            ),
            (a.arc_length, u.length_m(b.arc_length).expect("units")),
            (a.line_length, u.length_m(b.line_length).expect("units")),
        ] {
            if x == 0.0 {
                assert_eq!(y, 0.0, "{name}");
            } else {
                close_rel(y, x, 1e-9, name);
            }
        }
    }
}

/// F4: the same drawing-unit values as F1; units not stated, never assumed.
#[test]
fn f4_units_not_stated() {
    let d = read("f4_legacy_r12.dxf");
    let q = measured(&d);
    check_site_plan(&d, &q, 1.0);
    assert_eq!(q.units().insunits, None);
    assert_eq!(q.units().to_metres, None);
    assert_eq!(q.units().length_m(1.0), None);
    let w = issues(&q, "DRAWING_UNITS_NOT_STATED");
    assert_eq!(w.len(), 1);
    assert_eq!(w[0].level, Level::Caution);
}

/// F5: README values: 13 bays, area 175, perimeter 200, stripes 52; islands area 20 (perimeter
/// 2 × 14); 4 trees of radius 0.5: canopy π, circumference 4π.
#[test]
fn f5_blocks_mirrored_and_nested() {
    let d = read("f5_blocks_inserts.dxf");
    let q = measured(&d);
    let p = layer(&d, &q, "PARKING");
    assert_eq!((p.counts.polyline_closed, p.counts.line), (13, 13));
    close_rel(p.polyline_closed_area, 175.0, 1e-12, "bays");
    close_rel(p.polyline_closed_perimeter, 200.0, 1e-12, "bay perimeters");
    close_rel(p.line_length, 52.0, 1e-12, "stripes");
    let l = layer(&d, &q, "LANDSCAPE");
    assert_eq!((l.counts.polyline_closed, l.counts.circle), (2, 4));
    close_rel(l.polyline_closed_area, 20.0, 1e-12, "islands");
    close_rel(
        l.polyline_closed_perimeter,
        28.0,
        1e-12,
        "island perimeters",
    );
    close_rel(l.circle_area, PI, 1e-12, "canopy");
    close_rel(l.circle_circumference, 4.0 * PI, 1e-12, "circumference");
    assert!(q.layers().iter().all(|l| !l.incomplete()));
    assert!(
        q.layers().iter().all(|l| d.name(l.layer) != "0"),
        "block contents take the inserting layer"
    );
    assert!(cautions(&q).is_empty(), "{:?}", cautions(&q));
}

// Synthetic drawings ----------------------------------------------------------------------------------

fn block(name: &str, body: &str) -> String {
    format!("0\nBLOCK\n8\n0\n2\n{name}\n70\n0\n10\n0\n20\n0\n30\n0\n{body}0\nENDBLK\n8\n0\n")
}

fn insert(layer: &str, handle: &str, name: &str, at: (f64, f64), extra: &str) -> String {
    format!(
        "0\nINSERT\n5\n{handle}\n8\n{layer}\n2\n{name}\n10\n{}\n20\n{}\n30\n0\n{extra}",
        at.0, at.1
    )
}

fn line(layer: &str, handle: &str, a: (f64, f64), b: (f64, f64)) -> String {
    format!(
        "0\nLINE\n5\n{handle}\n8\n{layer}\n10\n{}\n20\n{}\n30\n0\n11\n{}\n21\n{}\n31\n0\n",
        a.0, a.1, b.0, b.1
    )
}

fn circle(layer: &str, handle: &str, c: (f64, f64), r: f64, extra: &str) -> String {
    format!(
        "0\nCIRCLE\n5\n{handle}\n8\n{layer}\n10\n{}\n20\n{}\n30\n0\n40\n{r}\n{extra}",
        c.0, c.1
    )
}

fn arc(layer: &str, handle: &str, c: (f64, f64), r: f64, s: f64, e: f64) -> String {
    format!(
        "0\nARC\n5\n{handle}\n8\n{layer}\n10\n{}\n20\n{}\n30\n0\n40\n{r}\n50\n{s}\n51\n{e}\n",
        c.0, c.1
    )
}

/// An LWPOLYLINE from (x, y, bulge) vertices.
fn lwpolyline(layer: &str, handle: &str, closed: bool, v: &[(f64, f64, f64)]) -> String {
    let mut s = format!(
        "0\nLWPOLYLINE\n5\n{handle}\n8\n{layer}\n90\n{}\n70\n{}\n",
        v.len(),
        i32::from(closed)
    );
    for (x, y, b) in v {
        s += &format!("10\n{x}\n20\n{y}\n");
        if *b != 0.0 {
            s += &format!("42\n{b}\n");
        }
    }
    s
}

fn drawing(header: &str, blocks: &str, entities: &str) -> Drawing {
    let text = format!(
        "{header}0\nSECTION\n2\nBLOCKS\n{blocks}0\nENDSEC\n0\nSECTION\n2\nENTITIES\n{entities}0\nENDSEC\n0\nEOF\n"
    );
    read_dxf(text.as_bytes(), &Budget::new(Limits::DEFAULT_BUDGET_BYTES)).expect("reads")
}

fn metres() -> String {
    "0\nSECTION\n2\nHEADER\n9\n$INSUNITS\n70\n6\n0\nENDSEC\n".to_owned()
}

/// A 2 × 2 square whose bottom edge bulges outward as a semicircle: area 4 + π/2, perimeter 6 + π.
const SEMI: [(f64, f64, f64); 4] = [
    (0.0, 0.0, 1.0),
    (2.0, 0.0, 0.0),
    (2.0, 2.0, 0.0),
    (0.0, 2.0, 0.0),
];

/// Bulged rings under similarity transforms, nested and mirrored. Placements of SEMI:
/// - scale 3, rotated 30°: area 9(4 + π/2), perimeter 3(6 + π);
/// - scale (−3, 3) (mirrored): the same;
/// - OUTER (SEMI at scale 2) placed at scale −0.5 in x, 0.5 in y: net scale 1, mirrored: 4 + π/2, 6 + π.
///
/// Total on layer R: area 19(4 + π/2), perimeter 7(6 + π). An ARC of radius 2 over 90° (π) in a block
/// at scale 2 and mirrored: 2π each.
#[test]
fn bulges_and_arcs_under_nested_and_mirrored_placements() {
    let d = drawing(
        &metres(),
        &(block("SEMI", &lwpolyline("0", "A1", true, &SEMI))
            + &block(
                "OUTER",
                &insert("0", "A2", "SEMI", (0.0, 0.0), "41\n2\n42\n2\n"),
            )
            + &block("QUARTER", &arc("0", "A3", (0.0, 0.0), 2.0, 0.0, 90.0))),
        &(insert("R", "10", "SEMI", (10.0, 0.0), "41\n3\n42\n3\n50\n30\n")
            + &insert("R", "11", "SEMI", (20.0, 0.0), "41\n-3\n42\n3\n")
            + &insert("R", "12", "OUTER", (30.0, 0.0), "41\n-0.5\n42\n0.5\n")
            + &insert("A", "13", "QUARTER", (0.0, 50.0), "41\n2\n42\n2\n")
            + &insert("A", "14", "QUARTER", (0.0, 60.0), "41\n-2\n42\n2\n")),
    );
    let q = measured(&d);
    let r = layer(&d, &q, "R");
    assert_eq!(r.counts.polyline_closed, 3);
    close_rel(
        r.polyline_closed_area,
        19.0 * (4.0 + PI / 2.0),
        1e-12,
        "area",
    );
    close_rel(
        r.polyline_closed_perimeter,
        7.0 * (6.0 + PI),
        1e-12,
        "perimeter",
    );
    let a = layer(&d, &q, "A");
    assert_eq!(a.counts.arc, 2);
    close_rel(a.arc_length, 4.0 * PI, 1e-12, "arcs");
    assert!(!r.incomplete() && !a.incomplete());
}

/// Under the non-uniform scale (2, 1): a 1 × 1 square has area 2 and perimeter 6; the line from (0, 0)
/// to (1, 1) has length √5; a circle and a bulged ring are not supported: counted as omitted, named.
#[test]
fn non_uniform_scale_keeps_straight_geometry_and_reports_curves() {
    let square = [
        (0.0, 0.0, 0.0),
        (1.0, 0.0, 0.0),
        (1.0, 1.0, 0.0),
        (0.0, 1.0, 0.0),
    ];
    let d = drawing(
        &metres(),
        &block(
            "B",
            &(lwpolyline("0", "A1", true, &square)
                + &line("0", "A2", (0.0, 0.0), (1.0, 1.0))
                + &circle("0", "A3", (5.0, 5.0), 1.0, "")
                + &lwpolyline("0", "A4", true, &SEMI)),
        ),
        &insert("S", "10", "B", (0.0, 0.0), "41\n2\n42\n1\n"),
    );
    let q = measured(&d);
    let s = layer(&d, &q, "S");
    assert_eq!(s.counts.polyline_closed, 1);
    close_rel(s.polyline_closed_area, 2.0, 1e-15, "area");
    close_rel(s.polyline_closed_perimeter, 6.0, 1e-15, "perimeter");
    close_rel(s.line_length, 5.0_f64.sqrt(), 1e-15, "line");
    assert_eq!(s.counts.circle, 0);
    assert_eq!(s.omitted, 2);
    assert!(s.incomplete());
    let ns = issues(&q, "NOT_SUPPORTED");
    assert_eq!(ns.len(), 1);
    assert_eq!(
        ns[0].detail,
        Detail::Feature("curve under non-uniform scale")
    );
    assert_eq!(ns[0].count, 2);
    assert_eq!(ns[0].handles(), &[0xA3, 0xA4]);
    assert_eq!(ns[0].level, Level::Caution);
}

/// A self-intersecting closed polyline keeps its perimeter (4 + 4√2) but has no area: invalid,
/// omitted, named. A zero-sweep ARC has length 0 with a note. A circle with extrusion (1, 0, 0) is not
/// a plan entity; one with (0, 0, −1) is (its area is unaffected).
#[test]
fn invalid_and_non_plan_geometry_is_reported() {
    let bow_tie = [
        (0.0, 0.0, 0.0),
        (2.0, 2.0, 0.0),
        (2.0, 0.0, 0.0),
        (0.0, 2.0, 0.0),
    ];
    let d = drawing(
        &metres(),
        "",
        &(lwpolyline("X", "A1", true, &bow_tie)
            + &arc("Z", "A2", (0.0, 0.0), 1.0, 45.0, 405.0)
            + &circle("N", "A3", (0.0, 0.0), 1.0, "210\n1\n220\n0\n230\n0\n")
            + &circle("M", "A4", (3.0, 0.0), 2.0, "210\n0\n220\n0\n230\n-1\n")),
    );
    let q = measured(&d);
    let x = layer(&d, &q, "X");
    assert_eq!(x.counts.polyline_closed, 1);
    close_rel(
        x.polyline_closed_perimeter,
        4.0 + 4.0 * 2.0_f64.sqrt(),
        1e-15,
        "bow tie",
    );
    assert_eq!(x.polyline_closed_area, 0.0);
    assert_eq!(x.omitted, 1);
    let inv = issues(&q, "DRAWING_GEOMETRY_INVALID");
    assert_eq!(inv.len(), 1);
    assert_eq!(inv[0].handles(), &[0xA1]);

    let z = layer(&d, &q, "Z");
    assert_eq!((z.counts.arc, z.arc_length, z.omitted), (1, 0.0, 0));
    let notes = issues(&q, "DRAWING_GEOMETRY_NOTE");
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].detail, Detail::Note("zero sweep"));
    assert_eq!(notes[0].level, Level::Info);

    let n = layer(&d, &q, "N");
    assert_eq!((n.counts.circle, n.omitted), (0, 1));
    assert!(
        issues(&q, "NOT_SUPPORTED")
            .iter()
            .any(|i| i.detail == Detail::Feature("non-plan extrusion") && i.handles() == [0xA3])
    );
    let m = layer(&d, &q, "M");
    close_rel(m.circle_area, 4.0 * PI, 1e-15, "mirrored-OCS circle");
}

/// HATCH: a polyline path with a hole (10 × 10 minus 2 × 2 = 96); edge paths of lines and arcs.
///
/// Arc edges: DXF states a clockwise arc (group 73 = 0) by complementary angles, so raw start s and
/// end e mean the arc runs clockwise from angle −s to angle −e (ezdxf `ArcEdge.load_tags`: stored as
/// the counter-clockwise arc from 360 − e to 360 − s, traversed backwards). It becomes a negative
/// bulge (D-28 §6).
/// - E: square path whose top edge runs from (10, 10) to (0, 10) about (5, 10), raw 0 → 180,
///   counter-clockwise: through (5, 15), outward: 100 + 25π/2.
/// - C: the same square, its top edge clockwise about (5, 15), radius 5√2, raw 45 → 135: from −45, the
///   point (10, 10), to −135, the point (0, 10), through (5, 15 − 5√2), inward. The segment cut off is
///   r²/2 (θ − sin θ) = 25 (π/2 − 1), so the area is 125 − 25π/2.
/// - Q: (0, 0) → (0, 10), then clockwise about (0, 0), radius 10, raw 270 → 0: from −270 ≡ 90, the
///   point (0, 10), to −0, the point (10, 0), through (5√2, 5√2); then back to (0, 0). A quarter disc:
///   25π.
///
/// Read any other way (counter-clockwise from s to e, or clockwise from s to e), C's arc would end at
/// (10, 20) and (0, 20) and Q's at (0, −10): the paths would not join.
#[test]
fn hatch_areas() {
    let poly_path = |pts: &[(f64, f64)]| {
        let mut s = format!("92\n2\n72\n0\n73\n1\n93\n{}\n", pts.len());
        for (x, y) in pts {
            s += &format!("10\n{x}\n20\n{y}\n");
        }
        s + "97\n0\n"
    };
    let hatch = |layer: &str, handle: &str, paths: &[String]| {
        format!(
            "0\nHATCH\n5\n{handle}\n8\n{layer}\n10\n0\n20\n0\n30\n0\n210\n0\n220\n0\n230\n1\n2\nSOLID\n70\n1\n71\n0\n91\n{}\n{}75\n1\n76\n1\n98\n0\n",
            paths.len(),
            paths.concat()
        )
    };
    let pairs = |p: &[String]| p.iter().map(|s| format!("{s}\n")).collect::<String>();
    let line = |a: (f64, f64), b: (f64, f64)| {
        pairs(
            &[
                "72",
                "1",
                "10",
                &a.0.to_string(),
                "20",
                &a.1.to_string(),
                "11",
                &b.0.to_string(),
                "21",
                &b.1.to_string(),
            ]
            .map(str::to_owned),
        )
    };
    let arc = |c: (f64, f64), r: f64, s: f64, e: f64, ccw: i32| {
        pairs(
            &[
                "72",
                "2",
                "10",
                &c.0.to_string(),
                "20",
                &c.1.to_string(),
                "40",
                &r.to_string(),
                "50",
                &s.to_string(),
                "51",
                &e.to_string(),
                "73",
                &ccw.to_string(),
            ]
            .map(str::to_owned),
        )
    };
    let edge_path =
        |edges: &[String]| format!("92\n1\n93\n{}\n{}97\n0\n", edges.len(), edges.concat());
    let square_with_top = |top: String| {
        edge_path(&[
            line((0.0, 0.0), (10.0, 0.0)),
            line((10.0, 0.0), (10.0, 10.0)),
            top,
            line((0.0, 10.0), (0.0, 0.0)),
        ])
    };
    let d = drawing(
        &metres(),
        "",
        &(hatch(
            "H",
            "B1",
            &[
                poly_path(&[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]),
                poly_path(&[(2.0, 2.0), (4.0, 2.0), (4.0, 4.0), (2.0, 4.0)]),
            ],
        ) + &hatch(
            "E",
            "B2",
            &[square_with_top(arc((5.0, 10.0), 5.0, 0.0, 180.0, 1))],
        ) + &hatch(
            "C",
            "B3",
            &[square_with_top(arc(
                (5.0, 15.0),
                50.0_f64.sqrt(),
                45.0,
                135.0,
                0,
            ))],
        ) + &hatch(
            "Q",
            "B4",
            &[edge_path(&[
                line((0.0, 0.0), (0.0, 10.0)),
                arc((0.0, 0.0), 10.0, 270.0, 0.0, 0),
                line((10.0, 0.0), (0.0, 0.0)),
            ])],
        )),
    );
    let q = measured(&d);
    let h = layer(&d, &q, "H");
    assert_eq!(h.counts.hatch, 1);
    close_rel(h.hatch_area, 96.0, 1e-15, "holed hatch");
    close_rel(
        layer(&d, &q, "E").hatch_area,
        100.0 + 25.0 * PI / 2.0,
        1e-14,
        "counter-clockwise",
    );
    close_rel(
        layer(&d, &q, "C").hatch_area,
        125.0 - 25.0 * PI / 2.0,
        1e-14,
        "clockwise, inward",
    );
    close_rel(
        layer(&d, &q, "Q").hatch_area,
        25.0 * PI,
        1e-14,
        "clockwise quarter disc",
    );
    assert!(
        q.layers().iter().all(|l| !l.incomplete()),
        "{:?}",
        q.issues()
    );
    assert!(issues(&q, "NOT_SUPPORTED").is_empty());
}

/// Never silently omitted: an uninterpreted entity type in model space or in a placed block makes
/// the (effective) layer incomplete; a missing block too. One in an unplaced block, or in paper
/// space, is listed but leaves totals complete. Text-like types never affect totals.
#[test]
fn uninterpreted_and_unexpanded_content_is_visible() {
    let d = drawing(
        &metres(),
        &(block(
            "PLACED",
            &("0\nSPLINE\n5\nC1\n8\n0\n".to_owned() + &line("0", "C2", (0.0, 0.0), (1.0, 0.0))),
        ) + &block("UNPLACED", "0\nSPLINE\n5\nC3\n8\nU\n")),
        &(insert("P", "10", "PLACED", (0.0, 0.0), "")
            + &insert("P", "11", "PLACED", (5.0, 0.0), "")
            + "0\nELLIPSE\n5\nC4\n8\nE\n"
            + "0\nMTEXT\n5\nC5\n8\nT\n"
            + "0\nVIEWPORT\n5\nC6\n8\nV\n67\n1\n"
            + &insert("G", "12", "NOPE", (0.0, 0.0), "")),
    );
    let q = measured(&d);
    let p = layer(&d, &q, "P");
    assert_eq!(
        (p.counts.line, p.omitted),
        (2, 2),
        "the SPLINE once per placement"
    );
    assert!(p.incomplete());
    let e = layer(&d, &q, "E");
    assert!(e.incomplete());
    let g = layer(&d, &q, "G");
    assert_eq!(g.omitted, 1);
    let missing = issues(&q, "DRAWING_BLOCK_MISSING");
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].handles(), &[0x12]);
    // U (unplaced block), T (text) and V (paper space) have no incomplete rows.
    for name in ["U", "T", "V"] {
        assert!(
            q.layers()
                .iter()
                .all(|l| d.name(l.layer) != name || !l.incomplete()),
            "{name}"
        );
    }
    // Everything uninterpreted is still listed.
    let types: Vec<(String, Level)> = issues(&q, "NOT_SUPPORTED")
        .iter()
        .filter_map(|i| match i.detail {
            Detail::EntityType(t) => Some((d.name(t).to_owned(), i.level)),
            _ => None,
        })
        .collect();
    for t in ["SPLINE", "ELLIPSE", "MTEXT", "VIEWPORT"] {
        assert!(types.iter().any(|(n, _)| n == t), "{t} listed");
    }
    assert!(types.iter().any(|(n, l)| n == "MTEXT" && *l == Level::Info));
}

/// Warnings are aggregated per code, layer and detail, with at most 20 handles and the full count.
#[test]
fn issues_are_aggregated_and_bounded() {
    let many: String = (0..25)
        .map(|k| insert("G", &format!("{:X}", 0x100 + k), "NOPE", (0.0, 0.0), ""))
        .collect();
    let d = drawing(&metres(), "", &many);
    let q = measured(&d);
    let m = issues(&q, "DRAWING_BLOCK_MISSING");
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].count, 25);
    assert_eq!(m[0].handles().len(), 20);
    assert_eq!(layer(&d, &q, "G").omitted, 25);
}

/// Drawing units (D-28 §7): conversion factors, and units unitless or not converted in 0.1.
#[test]
fn drawing_units() {
    let with = |code: i32| {
        drawing(
            &format!("0\nSECTION\n2\nHEADER\n9\n$INSUNITS\n70\n{code}\n0\nENDSEC\n"),
            "",
            &line("L", "A1", (0.0, 0.0), (10.0, 0.0)),
        )
    };
    // Exact factors: inch 0.0254 m; US survey foot 1200/3937 m; yard 0.9144 m; decimetre 0.1 m.
    for (code, factor) in [
        (1, 0.0254),
        (2, 0.3048),
        (3, 1609.344),
        (4, 1e-3),
        (5, 1e-2),
        (6, 1.0),
        (7, 1e3),
        (10, 0.9144),
        (14, 0.1),
        (21, 1200.0 / 3937.0),
    ] {
        let q = measured(&with(code));
        assert_eq!(q.units().to_metres, Some(factor), "code {code}");
        close_rel(
            q.units().length_m(10.0).expect("known"),
            10.0 * factor,
            1e-15,
            "length",
        );
        close_rel(
            q.units().area_m2(10.0).expect("known"),
            10.0 * factor * factor,
            1e-15,
            "area",
        );
        assert!(issues(&q, "DRAWING_UNITS_NOT_STATED").is_empty());
    }
    // 0 = unitless: drawing units only, with the same caution as "not stated".
    let q = measured(&with(0));
    assert_eq!((q.units().insunits, q.units().to_metres), (Some(0), None));
    assert_eq!(issues(&q, "DRAWING_UNITS_NOT_STATED").len(), 1);
    // 13 (micrometres) is stated but not converted in 0.1.
    let q = measured(&with(13));
    assert_eq!(q.units().to_metres, None);
    assert!(
        issues(&q, "NOT_SUPPORTED")
            .iter()
            .any(|i| i.detail == Detail::UnitCode(13))
    );
}

/// The factors agree with `civkit-units`, where the unit exists there.
#[test]
fn unit_factors_agree_with_civkit_units() {
    for (code, unit) in [
        (1, "in"),
        (2, "ft"),
        (3, "mi"),
        (4, "mm"),
        (5, "cm"),
        (6, "m"),
        (7, "km"),
        (10, "yd"),
        (21, "ft_us"),
    ] {
        let d = drawing(
            &format!("0\nSECTION\n2\nHEADER\n9\n$INSUNITS\n70\n{code}\n0\nENDSEC\n"),
            "",
            "",
        );
        let ours = measured(&d).units().to_metres.expect("known");
        let theirs = civkit_units::Unit::parse(unit).expect(unit).to_si(1.0);
        close_rel(ours, theirs, 1e-15, unit);
    }
}

/// Measurement reserves from the budget and releases it; the validity work limit is soft (the area
/// is omitted and named), the vertex work limit hard.
#[test]
fn budget_and_work_limits() {
    let d = read("f5_blocks_inserts.dxf");
    let budget = Budget::new(Limits::DEFAULT_BUDGET_BYTES);
    measure(&d, &budget, &AnalysisOptions::default()).expect("measures");
    assert!(budget.peak() > 0);
    // The result holds its rows and issues; dropped, everything is released.
    assert!(measure(&d, &Budget::new(0), &AnalysisOptions::default()).is_err());

    let tight = AnalysisOptions {
        max_validity_pairs: 0,
        ..AnalysisOptions::default()
    };
    let q = measure(&d, &Budget::new(Limits::DEFAULT_BUDGET_BYTES), &tight).expect("soft");
    let p = layer(&d, &q, "PARKING");
    assert_eq!(p.polyline_closed_area, 0.0);
    assert_eq!(p.omitted, 13);
    assert!(
        issues(&q, "NOT_SUPPORTED")
            .iter()
            .any(|i| i.detail == Detail::Feature("validity check work limit"))
    );

    let hard = AnalysisOptions {
        max_vertex_work: 10,
        ..AnalysisOptions::default()
    };
    assert!(matches!(
        measure(&d, &Budget::new(Limits::DEFAULT_BUDGET_BYTES), &hard),
        Err(civkit_drawing::AnalysisError::Limit {
            what: "vertices",
            limit: 10
        })
    ));
}

/// Validity is decided once per block entity, not per placement (D-28 §4.2). A rectangle costs 5
/// edge-pair examinations (as a square, GEOMETRY.md §8.7). With 10 allowed, PARKING_BAY's rectangle
/// and ISLAND's are each checked once, and all 13 bays and both islands get their areas.
#[test]
fn validity_is_decided_once_per_block_entity() {
    let d = read("f5_blocks_inserts.dxf");
    let options = AnalysisOptions {
        max_validity_pairs: 10,
        ..AnalysisOptions::default()
    };
    let q = measure(&d, &Budget::new(Limits::DEFAULT_BUDGET_BYTES), &options).expect("measures");
    close_rel(
        layer(&d, &q, "PARKING").polyline_closed_area,
        175.0,
        1e-12,
        "bays",
    );
    close_rel(
        layer(&d, &q, "LANDSCAPE").polyline_closed_area,
        20.0,
        1e-12,
        "islands",
    );
    assert!(q.layers().iter().all(|l| !l.incomplete()));
}
