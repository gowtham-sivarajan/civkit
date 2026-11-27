// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 23:36 IST · last edited 2026-11-27 23:49 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! ADR 0016 gates A5 (limits) and A6 (no silent loss), end to end through the analysis.
//!
//! A5: a 10⁹ × 10⁹ MINSERT, 33-deep nesting, a self-inserting block and a 1 000 000-vertex zig-zag ring each give the
//! stated error or warning quickly, never a panic, an overrun or a hang.
//!
//! A6: every unsupported or invalid case of D-28 §4–§7 leaves its layer's totals `incomplete` (counted in `omitted`)
//! and a warning names it (code, detail, handle), while the other entities on that layer are still measured.

use std::time::{Duration, Instant};

use civkit_drawing::{
    AnalysisOptions, Budget, Detail, Drawing, Issue, LayerTotals, Limits, Quantities, analyze_dxf,
    measure, read_dxf,
};

/// Far longer than any of these takes in a debug build; it only catches a hang.
const QUICK: Duration = Duration::from_secs(120);

fn header() -> String {
    "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n9\n$INSUNITS\n70\n6\n0\nENDSEC\n".to_owned()
}

fn dxf(blocks: &str, entities: &str) -> String {
    format!(
        "{}0\nSECTION\n2\nBLOCKS\n{blocks}0\nENDSEC\n0\nSECTION\n2\nENTITIES\n{entities}0\nENDSEC\n0\nEOF\n",
        header()
    )
}

fn block(name: &str, body: &str) -> String {
    format!("0\nBLOCK\n8\n0\n2\n{name}\n70\n0\n10\n0\n20\n0\n30\n0\n{body}0\nENDBLK\n")
}

fn insert(layer: &str, handle: &str, name: &str, extra: &str) -> String {
    format!("0\nINSERT\n5\n{handle}\n8\n{layer}\n2\n{name}\n10\n0\n20\n0\n30\n0\n{extra}")
}

/// A line of length 1 on `layer`: the "other entity" each A6 case keeps measuring beside the failure.
fn line(layer: &str, handle: &str) -> String {
    format!("0\nLINE\n5\n{handle}\n8\n{layer}\n10\n0\n20\n0\n30\n0\n11\n1\n21\n0\n31\n0\n")
}

fn read(text: &str) -> Drawing {
    read_dxf(text.as_bytes(), &Budget::new(Limits::DEFAULT_BUDGET_BYTES)).expect("reads")
}

fn measured(text: &str) -> (Drawing, Quantities) {
    let d = read(text);
    let q = measure(
        &d,
        &Budget::new(Limits::DEFAULT_BUDGET_BYTES),
        &AnalysisOptions::default(),
    )
    .expect("measures");
    (d, q)
}

fn layer<'q>(d: &Drawing, q: &'q Quantities, name: &str) -> &'q LayerTotals {
    q.layers()
        .iter()
        .find(|l| d.name(l.layer) == name)
        .unwrap_or_else(|| panic!("no layer {name}"))
}

/// The A6 assertion: the layer is incomplete by `omitted`, a warning with `code` and `detail` names `handle` on that
/// layer, and the layer's line (length 1) is still measured.
fn assert_named(
    d: &Drawing,
    q: &Quantities,
    layer_name: &str,
    code: &str,
    detail: Detail,
    handle: u64,
    omitted: u64,
) {
    let l = layer(d, q, layer_name);
    assert!(l.incomplete(), "{layer_name}: incomplete");
    assert_eq!(l.omitted, omitted, "{layer_name}: omitted");
    assert_eq!(
        (l.counts.line, l.line_length),
        (1, 1.0),
        "{layer_name}: the other entity is still measured"
    );
    let found: Vec<&Issue> = q
        .issues()
        .iter()
        .filter(|i| {
            i.code == code && i.detail == detail && i.layer.map(|x| d.name(x)) == Some(layer_name)
        })
        .collect();
    assert_eq!(
        found.len(),
        1,
        "{layer_name}: one {code} {detail:?} warning; issues: {:?}",
        q.issues()
    );
    assert!(
        found[0].handles().contains(&handle),
        "{layer_name}: names handle {handle:X}"
    );
}

// A5 -------------------------------------------------------------------------------------------------------------

#[test]
fn a5_hostile_inputs_end_quickly_with_the_stated_outcome() {
    let options = AnalysisOptions::default();

    // A 10⁹ × 10⁹ MINSERT: DRAWING_LIMIT before any cell is placed.
    let t = Instant::now();
    let huge = dxf(
        &block("B", &line("0", "A1")),
        &insert(
            "L",
            "10",
            "B",
            "70\n1000000000\n71\n1000000000\n44\n1\n45\n1\n",
        ),
    );
    let e = analyze_dxf(huge.as_bytes(), &options).expect_err("limit");
    assert_eq!(e.code(), "DRAWING_LIMIT");
    assert!(t.elapsed() < QUICK);

    // 33-deep nesting: DRAWING_LIMIT.
    let mut blocks = String::new();
    for k in 0..33 {
        let body = if k == 32 {
            line("0", "A1")
        } else {
            insert("0", &format!("{:X}", 0x100 + k), &format!("B{}", k + 1), "")
        };
        blocks += &block(&format!("B{k}"), &body);
    }
    let t = Instant::now();
    let e = analyze_dxf(
        dxf(&blocks, &insert("L", "10", "B0", "")).as_bytes(),
        &options,
    )
    .expect_err("limit");
    assert_eq!(e.code(), "DRAWING_LIMIT");
    assert!(t.elapsed() < QUICK);

    // A self-inserting block: a DRAWING_BLOCK_CYCLE warning, and the analysis completes.
    let t = Instant::now();
    let (d, q) = measured(&dxf(
        &block("SELF", &(line("0", "A1") + &insert("0", "A2", "SELF", ""))),
        &insert("C", "10", "SELF", ""),
    ));
    assert!(
        q.issues()
            .iter()
            .any(|i| i.code == "DRAWING_BLOCK_CYCLE" && i.handles() == [0xA2])
    );
    assert!(layer(&d, &q, "C").incomplete());
    assert!(t.elapsed() < QUICK);

    // A 1 000 000-vertex zig-zag ring: 499 999 long rows joined at alternate ends and closed along x = −1, so every
    // row overlaps every other in x (the §8.7 sweep's worst case). The validity check stops at its work limit: the
    // perimeter is measured, the area is left out and named, and nothing hangs or overruns the budget.
    let rows = 499_999_u32;
    let mut s = format!("0\nLWPOLYLINE\n5\n21\n8\nZ\n90\n{}\n70\n1\n", 2 * rows + 2);
    for r in 0..rows {
        let (a, b) = if r % 2 == 0 { (0, 1000) } else { (1000, 0) };
        s.push_str(&format!("10\n{a}\n20\n{r}\n10\n{b}\n20\n{r}\n"));
    }
    s.push_str(&format!("10\n-1\n20\n{}\n10\n-1\n20\n0\n", rows - 1));
    let t = Instant::now();
    let (d, q) = measured(&dxf("", &s));
    let z = layer(&d, &q, "Z");
    assert_eq!((z.counts.polyline_closed, z.omitted), (1, 1));
    assert!(z.polyline_closed_perimeter > 0.0);
    assert!(q.issues().iter().any(|i| i.code == "NOT_SUPPORTED"
        && i.detail == Detail::Feature("validity check work limit")
        && i.handles() == [0x21]));
    assert!(t.elapsed() < QUICK, "took {:?}", t.elapsed());
}

// A6 -------------------------------------------------------------------------------------------------------------

fn hatch(layer: &str, handle: &str, paths: &[String]) -> String {
    format!(
        "0\nHATCH\n5\n{handle}\n8\n{layer}\n10\n0\n20\n0\n30\n0\n2\nSOLID\n70\n1\n71\n0\n91\n{}\n{}75\n1\n76\n1\n98\n0\n",
        paths.len(),
        paths.concat()
    )
}

fn poly_path(pts: &[(f64, f64, f64)]) -> String {
    let mut s = format!("92\n2\n72\n1\n73\n1\n93\n{}\n", pts.len());
    for (x, y, b) in pts {
        s += &format!("10\n{x}\n20\n{y}\n42\n{b}\n");
    }
    s + "97\n0\n"
}

fn edge_path(edges: &str, count: usize) -> String {
    format!("92\n1\n93\n{count}\n{edges}97\n0\n")
}

fn line_edge(a: (f64, f64), b: (f64, f64)) -> String {
    format!(
        "72\n1\n10\n{}\n20\n{}\n11\n{}\n21\n{}\n",
        a.0, a.1, b.0, b.1
    )
}

const SQUARE: [(f64, f64, f64); 4] = [
    (0.0, 0.0, 0.0),
    (10.0, 0.0, 0.0),
    (10.0, 10.0, 0.0),
    (0.0, 10.0, 0.0),
];

#[test]
fn a6_unsupported_hatch_boundaries_are_named() {
    let square_edges = |third: String| {
        line_edge((0.0, 0.0), (10.0, 0.0))
            + &line_edge((10.0, 0.0), (10.0, 10.0))
            + &third
            + &line_edge((0.0, 10.0), (0.0, 0.0))
    };
    // Edge type 3, an elliptic arc: centre, major axis end, ratio, angles, direction.
    let ellipse = "72\n3\n10\n5\n20\n10\n11\n5\n21\n0\n40\n0.5\n50\n0\n51\n180\n73\n1\n".to_owned();
    // Edge type 4, a spline: degree, rational, periodic, 4 knots… here 8 knots and 4 control points.
    let spline = "72\n4\n94\n3\n73\n0\n74\n0\n95\n8\n96\n4\n40\n0\n40\n0\n40\n0\n40\n0\n40\n1\n40\n1\n40\n1\n40\n1\n\
                  10\n10\n20\n10\n10\n7\n20\n12\n10\n3\n20\n12\n10\n0\n20\n10\n"
        .to_owned();
    // A hatch whose boundary data has the wrong structure: 93 where a path's 92 must come.
    let unread = "0\nHATCH\n5\nB3\n8\nR\n10\n0\n20\n0\n30\n0\n2\nSOLID\n70\n1\n71\n0\n91\n1\n93\n4\n75\n1\n76\n1\n98\n0\n";
    // An edge path with a gap: the last edge stops at (0, 1), not back at (0, 0).
    let gap = line_edge((0.0, 0.0), (10.0, 0.0))
        + &line_edge((10.0, 0.0), (10.0, 10.0))
        + &line_edge((10.0, 10.0), (0.0, 10.0))
        + &line_edge((0.0, 10.0), (0.0, 1.0));
    // Two crossing rings in one hatch.
    let crossing = [
        poly_path(&SQUARE),
        poly_path(&[
            (5.0, 5.0, 0.0),
            (15.0, 5.0, 0.0),
            (15.0, 15.0, 0.0),
            (5.0, 15.0, 0.0),
        ]),
    ];

    let text = dxf(
        "",
        &(hatch("E", "B1", &[edge_path(&square_edges(ellipse), 4)])
            + &line("E", "C1")
            + &hatch("S", "B2", &[edge_path(&square_edges(spline), 4)])
            + &line("S", "C2")
            + unread
            + &line("R", "C3")
            + &hatch("G", "B4", &[edge_path(&gap, 4)])
            + &line("G", "C4")
            + &hatch("X", "B5", &crossing)
            + &line("X", "C5")),
    );
    let (d, q) = measured(&text);
    assert_named(
        &d,
        &q,
        "E",
        "NOT_SUPPORTED",
        Detail::Feature("hatch ellipse edge"),
        0xB1,
        1,
    );
    assert_named(
        &d,
        &q,
        "S",
        "NOT_SUPPORTED",
        Detail::Feature("hatch spline edge"),
        0xB2,
        1,
    );
    assert_named(&d, &q, "R", "DRAWING_HATCH_NOT_READ", Detail::None, 0xB3, 1);
    assert_named(
        &d,
        &q,
        "G",
        "NOT_SUPPORTED",
        Detail::Feature("hatch boundary not joined"),
        0xB4,
        1,
    );
    assert_named(
        &d,
        &q,
        "X",
        "DRAWING_GEOMETRY_INVALID",
        Detail::None,
        0xB5,
        1,
    );
}

#[test]
fn a6_inserts_that_are_not_expanded_are_named() {
    let text = dxf(
        &(block("SELF", &insert("0", "A1", "SELF", ""))
            + &block("B", &line("0", "A2"))
            + &block("FAR", &line("0", "A3")).replace(
                "10\n0\n20\n0\n30\n0\n0\nLINE",
                "10\n1e300\n20\n0\n30\n0\n0\nLINE",
            )),
        &(insert("C", "10", "SELF", "")
            + &line("C", "C1")
            + &insert("Z", "11", "B", "41\n0\n")
            + &line("Z", "C2")
            + &insert("N", "12", "B", "210\n1\n220\n0\n230\n0\n")
            + &line("N", "C3")
            + &insert("O", "13", "FAR", "41\n1e300\n42\n1e300\n")
            + &line("O", "C4")
            + &insert("M", "14", "NOPE", "")
            + &line("M", "C5")),
    );
    let (d, q) = measured(&text);
    // SELF inserted at the top places SELF once; inside it, its own INSERT A1 is the cycle (on layer 0 → C).
    assert_named(&d, &q, "C", "DRAWING_BLOCK_CYCLE", Detail::None, 0xA1, 1);
    assert_named(&d, &q, "Z", "DRAWING_ZERO_SCALE", Detail::None, 0x11, 1);
    assert_named(
        &d,
        &q,
        "N",
        "NOT_SUPPORTED",
        Detail::Feature("non-plan extrusion"),
        0x12,
        1,
    );
    assert_named(
        &d,
        &q,
        "O",
        "NOT_SUPPORTED",
        Detail::Feature("placement out of range"),
        0x13,
        1,
    );
    assert_named(&d, &q, "M", "DRAWING_BLOCK_MISSING", Detail::None, 0x14, 1);
}

#[test]
fn a6_curves_under_a_non_uniform_scale_are_named() {
    let curved_open = "0\nLWPOLYLINE\n5\nA1\n8\n0\n90\n3\n70\n0\n10\n0\n20\n0\n42\n0.5\n10\n1\n20\n0\n10\n2\n20\n0\n";
    let arc = "0\nARC\n5\nA2\n8\n0\n10\n0\n20\n0\n30\n0\n40\n1\n50\n0\n51\n90\n";
    let circle = "0\nCIRCLE\n5\nA3\n8\n0\n10\n0\n20\n0\n30\n0\n40\n1\n";
    let curved_hatch = hatch(
        "0",
        "A4",
        &[poly_path(&[
            (0.0, 0.0, 1.0),
            (2.0, 0.0, 0.0),
            (2.0, 2.0, 0.0),
            (0.0, 2.0, 0.0),
        ])],
    );
    let text = dxf(
        &block(
            "CURVES",
            &format!("{curved_open}{arc}{circle}{curved_hatch}"),
        ),
        &(insert("U", "10", "CURVES", "41\n2\n42\n1\n") + &line("U", "C1")),
    );
    let (d, q) = measured(&text);
    let u = layer(&d, &q, "U");
    assert_eq!(u.omitted, 4);
    assert_eq!((u.counts.line, u.line_length), (1, 1.0));
    let w: Vec<&Issue> = q
        .issues()
        .iter()
        .filter(|i| {
            i.code == "NOT_SUPPORTED"
                && i.detail == Detail::Feature("curve under non-uniform scale")
        })
        .collect();
    assert_eq!(w.len(), 1);
    assert_eq!(w[0].count, 4);
    assert_eq!(w[0].handles(), &[0xA1, 0xA2, 0xA3, 0xA4]);
    // Under the same scale, the straight line inside a block would still be measured (shown in quantities.rs).
}
