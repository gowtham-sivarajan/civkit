// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 21:00 IST · last edited 2026-11-27 21:44 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! The unit check, F3's planted problems P1–P4 (D-28 / ADR 0016 §8; gate A2). Every finding is a
//! caution-level warning, never an error: a drawing can be like this on purpose.
//!
//! F3's expected findings are the fixtures README's; its handles were read from the raw file (the
//! 96 000 000 building is `102`, the dimension `107`). Synthetic cases state their values inline.

mod common;

use civkit_drawing::{
    AnalysisOptions, Budget, Detail, Drawing, Issue, Level, Limits, Quantities, measure, read_dxf,
};
use common::*;

const P_CODES: [&str; 4] = [
    "DRAWING_UNITS_CONFLICT",
    "DRAWING_BLOCK_SCALE_MISMATCH",
    "DRAWING_SIZE_OUTLIER",
    "DRAWING_DIMENSION_OVERRIDE",
];

fn measured(d: &Drawing) -> Quantities {
    measure(
        d,
        &Budget::new(Limits::DEFAULT_BUDGET_BYTES),
        &AnalysisOptions::default(),
    )
    .expect("measures")
}

fn findings(q: &Quantities) -> Vec<&Issue> {
    q.issues()
        .iter()
        .filter(|i| P_CODES.contains(&i.code))
        .collect()
}

#[test]
fn f3_gives_exactly_p1_to_p4() {
    let d = read("f3_units_mismatch.dxf");
    let q = measured(&d);
    let f = findings(&q);
    let codes: Vec<&str> = f.iter().map(|i| i.code).collect();
    assert_eq!(codes.len(), 4, "{f:?}");
    for code in P_CODES {
        let i = f
            .iter()
            .find(|i| i.code == code)
            .unwrap_or_else(|| panic!("{code}"));
        assert_eq!((i.level, i.count), (Level::Caution, 1), "{code}");
    }
    // P2 names the block; P3 the 96 000 000 building on BUILDING; P4 the dimension.
    let p2 = f.iter().find(|i| i.code == P_CODES[1]).expect("P2");
    match p2.detail {
        Detail::Block(b) => assert_eq!(d.name(b), "WINDOW_MM"),
        other => panic!("{other:?}"),
    }
    let p3 = f.iter().find(|i| i.code == P_CODES[2]).expect("P3");
    assert_eq!(p3.handles(), &[0x102]);
    assert_eq!(p3.layer.map(|l| d.name(l)), Some("BUILDING"));
    let p4 = f.iter().find(|i| i.code == P_CODES[3]).expect("P4");
    assert_eq!(p4.handles(), &[0x107]);
}

#[test]
fn the_other_fixtures_give_none() {
    for name in [
        "f1_site_plan_m.dxf",
        "f2_site_plan_mm.dxf",
        "f4_legacy_r12.dxf",
        "f5_blocks_inserts.dxf",
        "insert_bricscad_level_1.dxf",
        "Minimal_DXF_AC1009.dxf",
    ] {
        let d = read(name);
        assert!(
            findings(&measured(&d)).is_empty(),
            "{name}: {:?}",
            findings(&measured(&d))
        );
    }
}

// Synthetic drawings -----------------------------------------------------------------------------------

fn header(insunits: Option<i32>, measurement: Option<i32>) -> String {
    let mut s = "0\nSECTION\n2\nHEADER\n".to_owned();
    if let Some(u) = insunits {
        s += &format!("9\n$INSUNITS\n70\n{u}\n");
    }
    if let Some(m) = measurement {
        s += &format!("9\n$MEASUREMENT\n70\n{m}\n");
    }
    s + "0\nENDSEC\n"
}

fn drawing(header: &str, blocks: &str, entities: &str) -> Drawing {
    let text = format!(
        "{header}0\nSECTION\n2\nBLOCKS\n{blocks}0\nENDSEC\n0\nSECTION\n2\nENTITIES\n{entities}0\nENDSEC\n0\nEOF\n"
    );
    read_dxf(text.as_bytes(), &Budget::new(Limits::DEFAULT_BUDGET_BYTES)).expect("reads")
}

fn count(q: &Quantities, code: &str) -> usize {
    q.issues().iter().filter(|i| i.code == code).count()
}

/// P1: an imperial `$INSUNITS` with `$MEASUREMENT` metric, or a metric one with imperial. Matching
/// pairs, and either value absent, are not findings.
#[test]
fn p1_header_conflict() {
    for (u, m, conflict) in [
        (Some(1), Some(1), true),  // inches, metric
        (Some(2), Some(1), true),  // feet, metric
        (Some(21), Some(1), true), // US survey feet, metric
        (Some(4), Some(0), true),  // millimetres, imperial
        (Some(6), Some(0), true),  // metres, imperial
        (Some(6), Some(1), false), // metres, metric
        (Some(2), Some(0), false), // feet, imperial
        (Some(6), None, false),    // not stated
        (None, Some(1), false),    // units not stated (its own warning)
        (Some(0), Some(1), false), // unitless
    ] {
        let q = measured(&drawing(&header(u, m), "", ""));
        assert_eq!(count(&q, P_CODES[0]) == 1, conflict, "{u:?} {m:?}");
    }
}

fn block(name: &str) -> String {
    format!(
        "0\nBLOCK\n8\n0\n2\n{name}\n70\n0\n10\n0\n20\n0\n30\n0\n0\nLINE\n8\n0\n10\n0\n20\n0\n30\n0\n11\n1\n21\n0\n31\n0\n0\nENDBLK\n"
    )
}

fn insert(handle: &str, name: &str, scale: f64) -> String {
    format!(
        "0\nINSERT\n5\n{handle}\n8\nL\n2\n{name}\n10\n0\n20\n0\n30\n0\n41\n{scale}\n42\n{scale}\n"
    )
}

/// P2: one block at uniform scales whose ratio is within 1 % of a ratio of two length units (≥ 10):
/// 1000 (mm/m), 25.4 (mm/in), 10 (mm/cm), 12 (in/ft). Ratios 2, 7 and 500 are not.
#[test]
fn p2_block_scale_mismatch() {
    for (a, b, finding) in [
        (0.001, 1.0, true),
        (1.0, 25.4, true),
        (1.0, 10.0, true),
        (1.0, 12.0, true),
        (1.0, 1000.0 * 1.005, true),
        (1.0, 2.0, false),
        (1.0, 7.0, false),
        (1.0, 500.0, false),
        (1.0, 1.0, false),
    ] {
        let q = measured(&drawing(
            &header(Some(6), Some(1)),
            &block("B"),
            &(insert("10", "B", a) + &insert("11", "B", b)),
        ));
        assert_eq!(count(&q, P_CODES[1]) == 1, finding, "{a} vs {b}");
    }
}

fn square(handle: &str, layer: &str, x: f64, side: f64) -> String {
    let mut s = format!("0\nLWPOLYLINE\n5\n{handle}\n8\n{layer}\n90\n4\n70\n1\n");
    for (px, py) in [(x, 0.0), (x + side, 0.0), (x + side, side), (x, side)] {
        s += &format!("10\n{px}\n20\n{py}\n");
    }
    s
}

/// P3: among at least 3 closed areas of one kind on one layer, an area at least 10³ times the median
/// (or at most 10⁻³ of it). Areas 4, 9, 4000 (ratio 4000 / 9 ≈ 444): none; 4, 9, 9500 (≈ 1056): a
/// finding; two areas only: never.
#[test]
fn p3_size_outlier() {
    let case = |sides: &[f64]| {
        let body: String = sides
            .iter()
            .enumerate()
            .map(|(k, s)| square(&format!("{:X}", 0x20 + k), "L", 1000.0 * k as f64, *s))
            .collect();
        measured(&drawing(&header(Some(6), Some(1)), "", &body))
    };
    assert_eq!(count(&case(&[2.0, 3.0, 4000.0_f64.sqrt()]), P_CODES[2]), 0);
    let q = case(&[2.0, 3.0, 9500.0_f64.sqrt()]);
    let f: Vec<&Issue> = q.issues().iter().filter(|i| i.code == P_CODES[2]).collect();
    assert_eq!(f.len(), 1);
    assert_eq!(f[0].handles(), &[0x22]);
    assert_eq!(count(&case(&[2.0, 2000.0]), P_CODES[2]), 0);
}

fn dimension(handle: &str, measurement: f64, text: Option<&str>) -> String {
    let mut s = format!(
        "0\nDIMENSION\n5\n{handle}\n8\nD\n70\n32\n42\n{measurement}\n13\n0\n23\n0\n33\n0\n14\n{measurement}\n24\n0\n34\n0\n"
    );
    if let Some(t) = text {
        s += &format!("1\n{t}\n");
    }
    s
}

/// P4: a numeric override that differs from the stated measurement (formatting removed) is a caution;
/// `<>` and no override are not findings; a non-numeric override is info.
#[test]
fn p4_dimension_override() {
    let level = |m: f64, t: Option<&str>| {
        let q = measured(&drawing(
            &header(Some(6), Some(1)),
            "",
            &dimension("30", m, t),
        ));
        q.issues()
            .iter()
            .find(|i| i.code == P_CODES[3])
            .map(|i| i.level)
    };
    assert_eq!(level(12.0, Some("15")), Some(Level::Caution));
    assert_eq!(level(12.0, Some("{\\fArial|b0;15}")), Some(Level::Caution));
    assert_eq!(level(12.0, Some("12")), None);
    assert_eq!(level(12.0, Some("12.000")), None);
    assert_eq!(level(12.0, Some("<>")), None);
    assert_eq!(level(12.0, Some("approx. <> m")), None);
    assert_eq!(level(12.0, None), None);
    assert_eq!(level(12.0, Some("")), None);
    assert_eq!(level(12.0, Some("TYP")), Some(Level::Info));
    // `\X` splits dimension text: what follows goes below the dimension line, so only the part before
    // it is the value (Autodesk MTEXT codes). 15 over "mm" is a wrong value; 12 over "TYP" is not.
    assert_eq!(level(12.0, Some(r"15\Xmm")), Some(Level::Caution));
    assert_eq!(level(12.0, Some(r"12\XTYP")), None);
    // The lower text often has its own formatting; none of it is part of the value.
    assert_eq!(level(12.0, Some(r"12\X{\H0.7x;TYP}")), None);
    // The `<>` placeholder counts only before the first unescaped `\X`: 15 over the measured value is a
    // wrong value; the measured value over 15 is not; with the `\X` escaped (`\\X`), nothing splits.
    assert_eq!(level(12.0, Some(r"15\X<>")), Some(Level::Caution));
    assert_eq!(level(12.0, Some(r"<>\X15")), None);
    assert_eq!(level(12.0, Some(r"15\\X<>")), None);
    assert_eq!(level(12.0, Some(r"12\\X15")), Some(Level::Info));
    // Codes with a payload end at `;` (height, alignment, colour, font), wherever they are.
    assert_eq!(level(12.0, Some(r"{\H2.5;15}")), Some(Level::Caution));
    assert_eq!(level(12.0, Some(r"\A1;15")), Some(Level::Caution));
    assert_eq!(level(12.0, Some(r"\C1;12")), None);
    // A paragraph break before the value is removed; an escaped backslash is text.
    assert_eq!(level(12.0, Some(r"\P15")), Some(Level::Caution));
    assert_eq!(level(12.0, Some("12\\\\")), Some(Level::Info));
}
