// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 23:10 IST · last edited 2026-11-27 23:16 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! ADR 0016 gate A4: the heap requested by read + analyse + report + schedule stays within the one shared budget
//! (plus the X3 slack), measured with a counting allocator. Like X3 (`tests/memory.rs`), one test function, alone in
//! this binary: the allocator counts every thread, so parallel tests would mix.
//!
//! For each input: the bytes are built first (caller-owned, outside the budget); then one default budget, the
//! measurement starts, and `read_dxf`, `analyze`, `to_json` and `to_csv` run with it, each output dropped before the
//! next. The requested peak must not pass the reserved peak by more than the slack.

mod common;

use civkit_drawing::{AnalysisOptions, Budget, Limits, analyze, read_dxf};
use common::*;
use peak_alloc::PeakAlloc;

#[global_allocator]
static HEAP: PeakAlloc = PeakAlloc;

/// The same slack as X3: the standard library's own small allocations, never storage that grows with the input.
const SLACK: usize = 16 * 1024;

fn entities(body: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n9\n$INSUNITS\n70\n6\n0\nENDSEC\n\
         0\nSECTION\n2\nENTITIES\n{body}0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn with_blocks(blocks: &str, body: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n9\n$INSUNITS\n70\n6\n0\nENDSEC\n\
         0\nSECTION\n2\nBLOCKS\n{blocks}0\nENDSEC\n0\nSECTION\n2\nENTITIES\n{body}0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

/// A convex ring of `n` vertices (a valid polygon, so the whole validity check runs).
fn ring(n: usize) -> String {
    let mut s = String::new();
    for k in 0..n {
        let t = k as f64 / n as f64 * std::f64::consts::TAU;
        s.push_str(&format!(
            "10\n{}\n20\n{}\n",
            1000.0 * t.cos(),
            1000.0 * t.sin()
        ));
    }
    s
}

fn analysis_heavy() -> Vec<(&'static str, Vec<u8>, AnalysisOptions)> {
    let default = AnalysisOptions::default();
    let rows = AnalysisOptions {
        entity_rows: true,
        ..AnalysisOptions::default()
    };
    let mut out = Vec::new();

    // The geometry reservation (256 bytes per vertex): one large closed polyline, measured with its validity check.
    let n = 200_000;
    out.push((
        "closed LWPOLYLINE, 200 000 vertices",
        entities(&format!(
            "0\nLWPOLYLINE\n5\nA1\n8\nL\n90\n{n}\n70\n1\n{}",
            ring(n)
        )),
        default,
    ));

    // A large hatch: one polyline path, and one edge path of many line edges (rings built from edges).
    let n = 100_000;
    let mut edges = String::new();
    for k in 0..n {
        let (a, b) = (k as f64, (k + 1) as f64);
        let t = |x: f64| x / n as f64 * std::f64::consts::TAU;
        edges.push_str(&format!(
            "72\n1\n10\n{}\n20\n{}\n11\n{}\n21\n{}\n",
            1000.0 * t(a).cos(),
            1000.0 * t(a).sin(),
            1000.0 * t(b).cos(),
            1000.0 * t(b).sin()
        ));
    }
    out.push((
        "HATCH with one 100 000-vertex polyline path",
        entities(&format!(
            "0\nHATCH\n5\nB1\n8\nH\n10\n0\n20\n0\n30\n0\n2\nSOLID\n70\n1\n71\n0\n91\n1\n\
             92\n2\n72\n0\n73\n1\n93\n{n}\n{}97\n0\n75\n1\n76\n1\n98\n0\n",
            ring(n)
        )),
        default,
    ));
    out.push((
        "HATCH with one 100 000-edge path",
        entities(&format!(
            "0\nHATCH\n5\nB2\n8\nH\n10\n0\n20\n0\n30\n0\n2\nSOLID\n70\n1\n71\n0\n91\n1\n\
             92\n1\n93\n{n}\n{edges}97\n0\n75\n1\n76\n1\n98\n0\n"
        )),
        default,
    ));
    // Many rings in one hatch: the cross-ring sweep over all their edges.
    let mut paths = String::new();
    for k in 0..500 {
        let (x, y) = (f64::from(k % 25) * 10.0, f64::from(k / 25) * 10.0);
        paths.push_str("92\n2\n72\n0\n73\n1\n93\n200\n");
        for j in 0..200 {
            let t = f64::from(j) / 200.0 * std::f64::consts::TAU;
            paths.push_str(&format!(
                "10\n{}\n20\n{}\n",
                x + 4.0 * t.cos(),
                y + 4.0 * t.sin()
            ));
        }
        paths.push_str("97\n0\n");
    }
    out.push((
        "HATCH with 500 disjoint 200-vertex rings",
        entities(&format!(
            "0\nHATCH\n5\nB3\n8\nH\n10\n0\n20\n0\n30\n0\n2\nSOLID\n70\n1\n71\n0\n91\n500\n{paths}75\n1\n76\n1\n98\n0\n"
        )),
        default,
    ));

    // Expansion: a 200-line block placed by a 50 × 50 MINSERT (500 000 placed lines), with entity rows (100 000,
    // then truncated) and the per-block cache.
    let mut block = String::from("0\nBLOCK\n8\n0\n2\nB\n70\n0\n10\n0\n20\n0\n30\n0\n");
    for k in 0..200 {
        block.push_str(&format!(
            "0\nLINE\n8\n0\n10\n{k}\n20\n0\n30\n0\n11\n{k}\n21\n1\n31\n0\n"
        ));
    }
    block.push_str("0\nENDBLK\n");
    out.push((
        "MINSERT 50 × 50 of a 200-line block, entity rows",
        with_blocks(
            &block,
            "0\nINSERT\n5\n10\n8\nP\n2\nB\n10\n0\n20\n0\n30\n0\n70\n50\n71\n50\n44\n300\n45\n2\n",
        ),
        rows,
    ));

    // The report: 20 000 TEXT of 1 KiB each, listed in the JSON.
    let mut texts = String::new();
    for k in 0..20_000 {
        texts.push_str(&format!(
            "0\nTEXT\n5\n{:X}\n8\nT\n10\n{k}\n20\n0\n30\n0\n40\n1\n1\n{}\n",
            0x100 + k,
            "é".repeat(512)
        ));
    }
    out.push(("20 000 TEXT of 1 KiB", entities(&texts), default));

    // The schedule: layer names near the 64 KiB string limit, one line each.
    let mut long = String::new();
    for k in 0..40 {
        let name = format!("{k:02}{}", "N".repeat(60 * 1024));
        long.push_str(&format!(
            "0\nLINE\n8\n{name}\n10\n0\n20\n0\n30\n0\n11\n1\n21\n0\n31\n0\n"
        ));
    }
    out.push(("40 layers with 60 KiB names", entities(&long), default));

    // Issues: many distinct warnings, each with handles (missing blocks on many layers).
    let mut missing = String::new();
    for k in 0..5_000 {
        missing.push_str(&format!(
            "0\nINSERT\n5\n{:X}\n8\nM{}\n2\nNOPE\n10\n0\n20\n0\n30\n0\n",
            0x1000 + k,
            k % 500
        ));
    }
    out.push((
        "5 000 missing-block inserts on 500 layers",
        entities(&missing),
        default,
    ));
    out
}

/// Read, analyse, write the report, write the schedule; returns the requested and reserved peaks.
fn measure(bytes: &[u8], options: &AnalysisOptions) -> (usize, usize) {
    let budget = Budget::new(Limits::DEFAULT_BUDGET_BYTES);
    let base = HEAP.current_usage();
    HEAP.reset_peak_usage();
    let drawing = read_dxf(bytes, &budget).expect("reads");
    let analysis = analyze(drawing, &budget, options).expect("analyses");
    let json = analysis.to_json().expect("json");
    drop(json);
    let csv = analysis.to_csv().expect("csv");
    drop(csv);
    drop(analysis);
    let measured = HEAP.peak_usage().saturating_sub(base);
    (measured, budget.peak())
}

#[test]
fn analysis_stays_within_the_shared_budget() {
    let mut inputs: Vec<(String, Vec<u8>, AnalysisOptions)> = Vec::new();
    for name in ALL_FIXTURES {
        for entity_rows in [false, true] {
            inputs.push((
                format!("{name} (entity rows: {entity_rows})"),
                fixture_bytes(name),
                AnalysisOptions {
                    entity_rows,
                    ..AnalysisOptions::default()
                },
            ));
        }
    }
    inputs.extend(
        analysis_heavy()
            .into_iter()
            .map(|(n, b, o)| (n.to_owned(), b, o)),
    );

    for (name, bytes, options) in &inputs {
        let (measured, reserved) = measure(bytes, options);
        assert!(
            measured <= reserved + SLACK,
            "A4 {name}: requested {measured} bytes but reserved only {reserved} (+{SLACK} slack): an allocation escaped \
             the shared budget"
        );
        println!("A4 {name}: requested {measured}, reserved {reserved}");
    }
}
