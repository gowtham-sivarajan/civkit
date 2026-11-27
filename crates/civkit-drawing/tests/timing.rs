// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 21:24 IST · last edited 2026-11-27 21:34 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! Timing evidence for the analysis work limits (D-28 §4.4, ADR 0016 "Settled during implementation").
//! Not run by default: `cargo test --release -p civkit-drawing --test timing -- --ignored --nocapture`.
//! Each case is the most expensive work per unit the analysis does for one counter; it prints the time
//! per unit, from which a limit's worst-case time follows (limit × time per unit).

use std::time::Instant;

use civkit_drawing::{AnalysisOptions, Budget, Limits, measure, read_dxf};

fn time_case(name: &str, text: &str, units: f64) {
    let budget = Budget::new(Limits::DEFAULT_BUDGET_BYTES);
    let d = read_dxf(text.as_bytes(), &budget).expect("reads");
    let t = Instant::now();
    let q = measure(&d, &budget, &AnalysisOptions::default()).expect("measures");
    let s = t.elapsed().as_secs_f64();
    println!(
        "{name}: {units:.3e} units in {s:.3} s = {:.1} ns per unit (layers {})",
        s / units * 1e9,
        q.layers().len()
    );
}

fn ring(handle: &str, n: usize, closed: bool) -> String {
    let mut s = format!(
        "0\nLWPOLYLINE\n5\n{handle}\n8\n0\n90\n{n}\n70\n{}\n",
        i32::from(closed)
    );
    for k in 0..n {
        let t = k as f64 / n as f64 * std::f64::consts::TAU;
        s += &format!("10\n{}\n20\n{}\n", 1000.0 * t.cos(), 1000.0 * t.sin());
    }
    s
}

fn dxf(blocks: &str, entities: &str) -> String {
    format!(
        "0\nSECTION\n2\nBLOCKS\n{blocks}0\nENDSEC\n0\nSECTION\n2\nENTITIES\n{entities}0\nENDSEC\n0\nEOF\n"
    )
}

#[test]
#[ignore = "timing evidence, run by hand in release"]
fn vertex_work_per_unit() {
    // Local measure of a closed ring: length, area and validity (the sweep), n units.
    let n = 1_000_000;
    time_case(
        "closed ring, local",
        &dxf("", &ring("A1", n, true)),
        n as f64,
    );

    // Re-measure under a non-uniform scale: n units per placement. A 100 000-vertex straight ring placed
    // 200 times at scale (2, 1): 2 × 10⁷ units (plus one local measure, 10⁵).
    let n = 100_000;
    let block = format!(
        "0\nBLOCK\n8\n0\n2\nB\n70\n0\n10\n0\n20\n0\n30\n0\n{}0\nENDBLK\n",
        ring("A1", n, true)
    );
    let inserts: String = (0..200)
        .map(|k| {
            format!(
                "0\nINSERT\n5\n{:X}\n8\nL\n2\nB\n10\n{}\n20\n0\n30\n0\n41\n2\n42\n1\n",
                0x100 + k,
                5000 * k
            )
        })
        .collect();
    time_case(
        "non-uniform re-measure",
        &dxf(&block, &inserts),
        (n * 200 + n) as f64,
    );

    // Lines: 2 units each. 2 × 10⁶ lines.
    let lines: String = (0..2_000_000)
        .map(|k| {
            format!(
                "0\nLINE\n8\nL\n10\n{k}\n20\n0\n30\n0\n11\n{}\n21\n1\n31\n0\n",
                k + 1
            )
        })
        .collect();
    time_case("lines", &dxf("", &lines), 4_000_000.0);
}

#[test]
#[ignore = "timing evidence, run by hand in release"]
fn placement_work_per_unit() {
    // A MINSERT of a one-line block: per cell, 1 unit for the cell and 1 for the line. 10⁷ cells.
    let block = "0\nBLOCK\n8\n0\n2\nB\n70\n0\n10\n0\n20\n0\n30\n0\n0\nLINE\n8\n0\n10\n0\n20\n0\n30\n0\n11\n1\n21\n0\n31\n0\n0\nENDBLK\n";
    let minsert =
        "0\nINSERT\n5\n10\n8\nL\n2\nB\n10\n0\n20\n0\n30\n0\n70\n10000\n71\n1000\n44\n2\n45\n2\n";
    time_case("MINSERT cells", &dxf(block, minsert), 2e7 + 1.0);
}

#[test]
#[ignore = "timing evidence, run by hand in release"]
fn validity_pairs_per_unit() {
    // A meander of 20 000 long rows: about 2 × 10⁸ examined pairs, stopped at the default limit (10⁸).
    let rows = 20_000;
    let mut s = format!("0\nLWPOLYLINE\n5\nA1\n8\n0\n90\n{}\n70\n1\n", 2 * rows + 2);
    for r in 0..rows {
        let (a, b) = if r % 2 == 0 {
            (0.0, 1000.0)
        } else {
            (1000.0, 0.0)
        };
        s += &format!("10\n{a}\n20\n{r}\n10\n{b}\n20\n{r}\n");
    }
    s += &format!("10\n-1\n20\n{}\n10\n-1\n20\n0\n", rows - 1);
    time_case("validity pairs (to the limit)", &dxf("", &s), 1e8);
}

#[test]
#[ignore = "timing evidence, run by hand in release"]
fn validity_exact_tests_per_unit() {
    // The worst case for the validity limit: every pair's boxes overlap, so every examined pair is also
    // tested exactly. A meander of parallel diagonals 0.01 apart (all boxes about [0, 1000] × [0, 1200]),
    // none touching: 20 000 rows, about 2 × 10⁸ pairs, stopped at the default limit (10⁸).
    let rows = 20_000;
    let d = 0.01;
    let mut s = format!("0\nLWPOLYLINE\n5\nA1\n8\n0\n90\n{}\n70\n1\n", 2 * rows + 2);
    for r in 0..rows {
        let y = f64::from(r) * d;
        let (a, b) = if r % 2 == 0 {
            ((0.0, y), (1000.0, 1000.0 + y))
        } else {
            ((1000.0, 1000.0 + y), (0.0, y))
        };
        s += &format!("10\n{}\n20\n{}\n10\n{}\n20\n{}\n", a.0, a.1, b.0, b.1);
    }
    s += &format!("10\n-1\n20\n{}\n10\n-1\n20\n0\n", f64::from(rows - 1) * d);
    time_case("validity exact tests (to the limit)", &dxf("", &s), 1e8);
}
