// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 22:00 IST · last edited 2026-11-27 22:05 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! The drawing report and the schedule (D-28 / ADR 0016 §9, API_DESIGN §13; gate A8 in part).
//!
//! Expected values are the fixtures README's; the JSON is parsed with serde_json (a test-only
//! dependency) and the CSV by splitting RFC 4180 rows here, so neither check uses the writers under
//! test.

#![allow(clippy::float_cmp)] // exact values from exact geometry (200, 2550) are compared exactly

mod common;

use civkit_drawing::{AnalysisError, AnalysisOptions, REPORT_ID, REPORT_VERSION, analyze_dxf};
use common::*;
use serde_json::Value;

fn json(name: &str) -> Value {
    let a = analyze_dxf(&fixture_bytes(name), &AnalysisOptions::default()).expect("analyses");
    serde_json::from_str(&a.to_json().expect("json")).expect("valid JSON")
}

fn json_of(text: &str, options: &AnalysisOptions) -> Value {
    let a = analyze_dxf(text.as_bytes(), options).expect("analyses");
    serde_json::from_str(&a.to_json().expect("json")).expect("valid JSON")
}

fn layer<'v>(report: &'v Value, name: &str) -> &'v Value {
    report["layers"]
        .as_array()
        .expect("layers")
        .iter()
        .find(|l| l["name"] == name)
        .unwrap_or_else(|| panic!("no layer {name}"))
}

fn codes(report: &Value) -> Vec<String> {
    report["warnings"]
        .as_array()
        .expect("warnings")
        .iter()
        .map(|w| w["code"].as_str().expect("code").to_owned())
        .collect()
}

/// Splits RFC 4180 CSV (quoted fields, doubled quotes, CRLF or LF rows) into rows of fields.
fn parse_csv(text: &str) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match (quoted, c) {
            (true, '"') if chars.peek() == Some(&'"') => {
                field.push('"');
                chars.next();
            }
            (true, '"') => quoted = false,
            (true, c) => field.push(c),
            (false, '"') => quoted = true,
            (false, ',') => row.push(std::mem::take(&mut field)),
            (false, '\r') => {}
            (false, '\n') => {
                row.push(std::mem::take(&mut field));
                rows.push(std::mem::take(&mut row));
            }
            (false, c) => field.push(c),
        }
    }
    assert!(
        field.is_empty() && row.is_empty(),
        "the last row ends with a line break"
    );
    rows
}

const HEADER: [&str; 12] = [
    "layer",
    "kind",
    "count",
    "length",
    "area",
    "unit",
    "length_m",
    "area_m2",
    "incomplete",
    "omitted",
    "report_version",
    "civkit_version",
];

// The JSON report ------------------------------------------------------------------------------------

#[test]
fn f1_report() {
    let r = json("f1_site_plan_m.dxf");
    assert_eq!(r["report"], REPORT_ID);
    assert_eq!(REPORT_ID, "drawing.analyze");
    assert_eq!(r["version"], REPORT_VERSION);
    assert_eq!(REPORT_VERSION, "1.0.0");
    assert_eq!(r["civkit_version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(r["plan_quantities"], true);
    assert_eq!(r["drawing"]["acadver"], "AC1032");
    let u = &r["drawing"]["units"];
    assert_eq!(
        (
            u["insunits"].as_i64(),
            u["name"].as_str(),
            u["stated"].as_bool()
        ),
        (Some(6), Some("m"), Some(true))
    );
    assert_eq!(u["to_metres"].as_f64(), Some(1.0));

    let b = layer(&r, "BOUNDARY");
    assert_eq!(b["counts"]["polyline_closed"], 1);
    assert_eq!(
        b["drawing_units"]["polyline_closed_perimeter"].as_f64(),
        Some(200.0)
    );
    assert_eq!(
        b["drawing_units"]["polyline_closed_area"].as_f64(),
        Some(2550.0)
    );
    assert_eq!(b["si"]["polyline_closed_area_m2"].as_f64(), Some(2550.0));
    assert_eq!(
        (b["incomplete"].as_bool(), b["omitted"].as_u64()),
        (Some(false), Some(0))
    );
    let road = layer(&r, "ROAD_CL")["drawing_units"]["polyline_open_length"]
        .as_f64()
        .expect("road");
    assert!((road - (100.0 + 10.0 * std::f64::consts::PI)).abs() <= 1e-9 * road);

    // Layers are sorted by name.
    let names: Vec<&str> = r["layers"]
        .as_array()
        .expect("layers")
        .iter()
        .map(|l| l["name"].as_str().expect("name"))
        .collect();
    let mut sorted = names.clone();
    sorted.sort_unstable();
    assert_eq!(names, sorted);

    // Text and dimensions are listed as drawing text, with world positions and hex handles.
    let text = r["text"].as_array().expect("text");
    assert_eq!(text.len(), 1);
    assert_eq!(
        (text[0]["layer"].as_str(), text[0]["value"].as_str()),
        (Some("TEXT"), Some("PLOT 17"))
    );
    assert!(
        text[0]["handle"]
            .as_str()
            .is_some_and(|h| u64::from_str_radix(h, 16).is_ok())
    );
    let dims = r["dimensions"].as_array().expect("dimensions");
    assert_eq!(dims.len(), 1);
    assert_eq!(dims[0]["measurement"].as_f64(), Some(60.0));
    assert_eq!(dims[0]["type"], "rotated");

    // The dimension's own block holds MTEXT and SOLID: listed where they are.
    let ni = r["not_interpreted"].as_array().expect("not_interpreted");
    assert!(ni.iter().any(|e| e["type"] == "SOLID"
        && e["where"] == "block"
        && e["block"] == "*D1"
        && e["count"] == 2));
    // No entity rows unless asked for.
    assert!(r.get("entities").is_none());
}

#[test]
fn f4_units_not_stated() {
    let r = json("f4_legacy_r12.dxf");
    let u = &r["drawing"]["units"];
    assert_eq!(
        (
            u["insunits"].is_null(),
            u["to_metres"].is_null(),
            u["stated"].as_bool()
        ),
        (true, true, Some(false))
    );
    assert!(
        layer(&r, "BOUNDARY")["si"].is_null(),
        "no SI values without units"
    );
    assert!(codes(&r).contains(&"DRAWING_UNITS_NOT_STATED".to_owned()));
    let w = r["warnings"]
        .as_array()
        .expect("warnings")
        .iter()
        .find(|w| w["code"] == "DRAWING_UNITS_NOT_STATED")
        .expect("w");
    assert_eq!(w["level"], "caution");
    assert!(w["message"].as_str().is_some_and(|m| !m.is_empty()));
}

/// F3's findings, with hex handles; messages carry no drawing text.
#[test]
fn f3_warnings() {
    let r = json("f3_units_mismatch.dxf");
    let ws = r["warnings"].as_array().expect("warnings");
    let find = |code: &str| {
        ws.iter()
            .find(|w| w["code"] == code)
            .unwrap_or_else(|| panic!("{code}"))
    };
    assert_eq!(
        find("DRAWING_SIZE_OUTLIER")["handles"],
        serde_json::json!(["102"])
    );
    assert_eq!(find("DRAWING_SIZE_OUTLIER")["layer"], "BUILDING");
    assert_eq!(
        find("DRAWING_DIMENSION_OVERRIDE")["handles"],
        serde_json::json!(["107"])
    );
    assert_eq!(find("DRAWING_BLOCK_SCALE_MISMATCH")["block"], "WINDOW_MM");
    assert_eq!(find("DRAWING_UNITS_CONFLICT")["unit_code"], 1);
    for w in ws {
        let m = w["message"].as_str().expect("message");
        let words: Vec<&str> = m
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .collect();
        for name in ["BUILDING", "WINDOW_MM", "DIM", "WINDOW"] {
            assert!(!words.contains(&name), "drawing text in a message: {m}");
        }
    }
}

/// R2018 (UTF-8 text, ADR 0015), metres.
fn header() -> String {
    "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n9\n$INSUNITS\n70\n6\n0\nENDSEC\n".to_owned()
}

fn entities(body: &str) -> String {
    format!(
        "{}0\nSECTION\n2\nENTITIES\n{body}0\nENDSEC\n0\nEOF\n",
        header()
    )
}

fn line(layer: &str, handle: &str, x: f64) -> String {
    format!(
        "0\nLINE\n5\n{handle}\n8\n{layer}\n10\n{x}\n20\n0\n30\n0\n11\n{}\n21\n0\n31\n0\n",
        x + 2.0
    )
}

fn text(layer: &str, handle: &str, value: &str) -> String {
    format!("0\nTEXT\n5\n{handle}\n8\n{layer}\n10\n1\n20\n2\n30\n0\n40\n1\n1\n{value}\n")
}

/// Drawing text is escaped, never interpreted: quotes, backslashes and non-ASCII round-trip.
#[test]
fn drawing_text_round_trips_through_json() {
    let value = r#"say "hi" \ back\slash, ünïcödé ≥ 1"#;
    let layer_name = r#"L"q\uote"#;
    let r = json_of(
        &entities(&(text(layer_name, "A1", value) + &line(layer_name, "A2", 0.0))),
        &AnalysisOptions::default(),
    );
    assert_eq!(r["text"][0]["value"], value);
    assert_eq!(r["text"][0]["layer"], layer_name);
    assert_eq!(layer(&r, layer_name)["counts"]["line"], 1);
}

/// Entity rows on request: one per placed, measured entity, with the chain of handles; bounded, and a
/// truncation is a warning while the totals stay complete.
#[test]
fn entity_rows_and_their_bound() {
    let options = AnalysisOptions {
        entity_rows: true,
        ..AnalysisOptions::default()
    };
    let a = analyze_dxf(&fixture_bytes("f5_blocks_inserts.dxf"), &options).expect("analyses");
    let r: Value = serde_json::from_str(&a.to_json().expect("json")).expect("valid");
    let rows = r["entities"].as_array().expect("entities");
    // 13 bays (rectangle + stripe), 2 islands, 4 trees.
    assert_eq!(rows.len(), 13 * 2 + 2 + 4);
    let mirrored = rows
        .iter()
        .find(|e| e["top_handle"] == "115" && e["kind"] == "polyline_closed")
        .expect("bay 115");
    assert_eq!(mirrored["area"].as_f64(), Some(12.5));
    assert_eq!(mirrored["length"].as_f64(), Some(15.0));
    let tree = rows.iter().find(|e| e["kind"] == "circle").expect("tree");
    assert!(tree["insert_handle"] == "10A" || tree["insert_handle"] == "10B");

    let small = AnalysisOptions {
        entity_rows: true,
        max_rows: 5,
        ..AnalysisOptions::default()
    };
    let r = json_of(
        std::str::from_utf8(&fixture_bytes("f5_blocks_inserts.dxf")).expect("utf8"),
        &small,
    );
    assert_eq!(r["entities"].as_array().expect("entities").len(), 5);
    let t = r["warnings"]
        .as_array()
        .expect("w")
        .iter()
        .find(|w| w["code"] == "DRAWING_REPORT_TRUNCATED")
        .expect("truncated");
    assert_eq!(t["feature"], "entity rows");
    assert_eq!(t["count"], 27, "32 rows, 5 kept");
    assert_eq!(
        layer(&r, "PARKING")["drawing_units"]["polyline_closed_area"].as_f64(),
        Some(175.0)
    );
}

/// Text rows have the same bound.
#[test]
fn text_rows_are_bounded() {
    let body: String = (0..8)
        .map(|k| text("T", &format!("{:X}", 0x20 + k), "x"))
        .collect();
    let r = json_of(
        &entities(&body),
        &AnalysisOptions {
            max_rows: 3,
            ..AnalysisOptions::default()
        },
    );
    assert_eq!(r["text"].as_array().expect("text").len(), 3);
    assert_eq!(layer(&r, "T")["counts"]["text"], 8);
    assert!(
        r["warnings"]
            .as_array()
            .expect("w")
            .iter()
            .any(|w| w["code"] == "DRAWING_REPORT_TRUNCATED"
                && w["feature"] == "text rows"
                && w["count"] == 5)
    );
}

#[test]
fn the_report_is_deterministic() {
    let bytes = fixture_bytes("insert_bricscad_level_1.dxf");
    let once = analyze_dxf(&bytes, &AnalysisOptions::default())
        .expect("a")
        .to_json()
        .expect("j");
    let twice = analyze_dxf(&bytes, &AnalysisOptions::default())
        .expect("a")
        .to_json()
        .expect("j");
    assert_eq!(once, twice);
}

// The CSV schedule -------------------------------------------------------------------------------------

#[test]
fn f1_schedule() {
    let a = analyze_dxf(
        &fixture_bytes("f1_site_plan_m.dxf"),
        &AnalysisOptions::default(),
    )
    .expect("analyses");
    let csv = a.to_csv().expect("csv");
    assert!(!csv.starts_with('\u{feff}'), "no byte-order mark");
    let rows = parse_csv(&csv);
    assert_eq!(rows[0], HEADER);
    let v = env!("CARGO_PKG_VERSION");
    let row = |l: &str, k: &str| {
        rows.iter()
            .find(|r| r[0] == l && r[1] == k)
            .unwrap_or_else(|| panic!("{l} {k}"))
    };
    assert_eq!(
        row("BOUNDARY", "polyline_closed"),
        &[
            "BOUNDARY",
            "polyline_closed",
            "1",
            "200",
            "2550",
            "m",
            "200",
            "2550",
            "false",
            "0",
            "1.0.0",
            v
        ]
    );
    assert_eq!(row("WATER", "circle")[2], "1");
    let area: f64 = row("WATER", "circle")[4].parse().expect("number");
    assert!((area - 9.0 * std::f64::consts::PI).abs() <= 1e-12 * area);
    // A line has a length, no area.
    assert_eq!(row("DRAIN", "line")[3..5], ["25".to_owned(), String::new()]);
    // Text is counted, not measured.
    assert_eq!(
        row("TEXT", "text")[2..5],
        ["1".to_owned(), String::new(), String::new()]
    );
    // Every row has every column and the versions.
    assert!(rows.iter().all(|r| r.len() == 12));
    assert!(rows[1..].iter().all(|r| r[10] == "1.0.0" && r[11] == v));
}

#[test]
fn f4_schedule_has_no_si_columns_filled() {
    let a = analyze_dxf(
        &fixture_bytes("f4_legacy_r12.dxf"),
        &AnalysisOptions::default(),
    )
    .expect("analyses");
    let rows = parse_csv(&a.to_csv().expect("csv"));
    assert!(
        rows[1..]
            .iter()
            .all(|r| r[5].is_empty() && r[6].is_empty() && r[7].is_empty())
    );
}

/// Layer names come from the drawing: a leading `=`, `+`, `-` or `@` is neutralised with `'` (so are a
/// tab and CR, which DXF value lines cannot carry at the start);
/// commas, quotes and line breaks are quoted (RFC 4180). A layer with omissions but no measures still
/// has a row.
#[test]
fn schedule_cells_are_safe() {
    let names = ["=SUM(A1)", "+1", "-2", "@x", "a,b", r#"q"q"#];
    let body: String = names
        .iter()
        .enumerate()
        .map(|(k, n)| line(n, &format!("{:X}", 0x40 + k), 0.0))
        .collect();
    let body = body + "0\nINSERT\n5\n50\n8\nMISSING\n2\nNOPE\n10\n0\n20\n0\n30\n0\n";
    let a = analyze_dxf(entities(&body).as_bytes(), &AnalysisOptions::default()).expect("analyses");
    let rows = parse_csv(&a.to_csv().expect("csv"));
    let layers: Vec<&str> = rows[1..].iter().map(|r| r[0].as_str()).collect();
    for expected in ["'=SUM(A1)", "'+1", "'-2", "'@x", "a,b", r#"q"q"#] {
        assert!(layers.contains(&expected), "{expected:?} in {layers:?}");
    }
    let missing = rows
        .iter()
        .find(|r| r[0] == "MISSING")
        .expect("row for a layer with omissions only");
    assert_eq!(
        (
            missing[1].as_str(),
            missing[2].as_str(),
            missing[8].as_str(),
            missing[9].as_str()
        ),
        ("", "0", "true", "1")
    );
}

// Errors ----------------------------------------------------------------------------------------------

#[test]
fn errors_have_contract_codes() {
    let bad = analyze_dxf(
        b"0\nSECTION\n2\nENTITIES\n0\nLINE\n10\nnot a number\n",
        &AnalysisOptions::default(),
    );
    assert_eq!(
        bad.as_ref().err().map(AnalysisError::code),
        Some("DRAWING_UNREADABLE")
    );
    let deep = AnalysisOptions {
        expand: civkit_drawing::ExpandLimits {
            max_work: 1,
            ..civkit_drawing::ExpandLimits::default()
        },
        ..AnalysisOptions::default()
    };
    let limited = analyze_dxf(&fixture_bytes("f5_blocks_inserts.dxf"), &deep);
    assert_eq!(
        limited.as_ref().err().map(AnalysisError::code),
        Some("DRAWING_LIMIT")
    );
    let mut huge = b"0\nSECTION\n2\nENTITIES\n".to_vec();
    huge.resize(civkit_drawing::Limits::default().max_input_bytes + 1, b' ');
    assert_eq!(
        analyze_dxf(&huge, &AnalysisOptions::default())
            .err()
            .map(|e| e.code()),
        Some("DRAWING_LIMIT")
    );
}
