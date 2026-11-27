// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 19:40 IST · last edited 2026-11-27 20:24 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! Block and MINSERT expansion (D-28 / ADR 0016 §4; gates A3, A5 and A6 in part).
//!
//! Expected positions are worked by hand from the fixtures' raw INSERT records with the formula of
//! GEOMETRY.md §11 (each test shows the steps). Totals are the fixtures README's values, fixed by
//! construction. None is produced by the code under test.

#![allow(clippy::float_cmp)] // determinants of exact scale factors (2, ±1) are compared exactly

mod common;

use civkit_drawing::{
    AnalysisError, Budget, Drawing, EntityKind, Event, ExpandLimits, Limits, Skip, expand, read_dxf,
};
use civkit_geometry::{Point, Vertex as GVertex, polyline_length};
use common::*;

fn close(actual: f64, expected: f64, abs: f64, what: &str) {
    assert!(
        (actual - expected).abs() <= abs,
        "{what}: expected {expected}, got {actual}"
    );
}

/// What a test needs from one placed entity.
#[derive(Debug, Clone)]
struct Seen {
    layer: String,
    kind: &'static str,
    handle: Option<u64>,
    top: Option<u64>,
    insert: Option<u64>,
    /// The world position of the entity's first point (line start, polyline first vertex, circle centre).
    at: (f64, f64),
    det: f64,
    scale: Option<f64>,
    /// For closed polylines: the placed ring's area and perimeter; for lines: the placed length.
    area: f64,
    length: f64,
}

#[derive(Debug, Clone, PartialEq)]
struct Skipped {
    reason: Skip,
    layer: String,
    handle: Option<u64>,
}

fn run(
    d: &Drawing,
    limits: &ExpandLimits,
) -> (
    Vec<Seen>,
    Vec<Skipped>,
    Result<civkit_drawing::ExpandStats, AnalysisError>,
) {
    let budget = Budget::new(Limits::DEFAULT_BUDGET_BYTES);
    let (mut seen, mut skipped) = (Vec::new(), Vec::new());
    let r = expand(d, &budget, limits, |ev| match ev {
        Event::Placed(p) => {
            let t = p.plan_transform().expect("plan entity");
            let pt = |x: f64, y: f64| t.apply(Point::new(x, y)).expect("finite");
            let (at, area, length) = match &p.entity.kind {
                EntityKind::Line { start, end } => {
                    let (a, b) = (pt(start.x, start.y), pt(end.x, end.y));
                    ((a.x, a.y), 0.0, (b.x - a.x).hypot(b.y - a.y))
                }
                EntityKind::Circle { center, .. } => {
                    let c = pt(center.x, center.y);
                    ((c.x, c.y), 0.0, 0.0)
                }
                EntityKind::Polyline(pl) => {
                    let placed: Vec<GVertex> = pl
                        .vertices
                        .iter()
                        .map(|v| {
                            t.apply_vertex(GVertex::new(v.x, v.y, v.bulge))
                                .expect("placed")
                        })
                        .collect();
                    let first = placed[0].point;
                    let len = polyline_length(&placed, pl.closed, None)
                        .expect("length")
                        .value;
                    let area = if pl.closed {
                        civkit_geometry::polygon_area(&placed, None)
                            .expect("area")
                            .area
                    } else {
                        0.0
                    };
                    ((first.x, first.y), area, len)
                }
                EntityKind::Text(tx) => {
                    let a = pt(tx.point.x, tx.point.y);
                    ((a.x, a.y), 0.0, 0.0)
                }
                _ => ((f64::NAN, f64::NAN), 0.0, 0.0),
            };
            seen.push(Seen {
                layer: d.name(p.layer).to_owned(),
                kind: p.entity.kind.dxf_name(),
                handle: p.entity.handle,
                top: p.top_handle,
                insert: p.insert_handle,
                at,
                det: t.det(),
                scale: t.similarity_scale(),
                area,
                length,
            });
        }
        Event::Skipped(s) => skipped.push(Skipped {
            reason: s.reason,
            layer: d.name(s.layer).to_owned(),
            handle: s.entity.handle,
        }),
    });
    (seen, skipped, r)
}

fn on<'a>(seen: &'a [Seen], layer: &str, kind: &str) -> Vec<&'a Seen> {
    seen.iter()
        .filter(|s| s.layer == layer && s.kind == kind)
        .collect()
}

fn sorted_points(v: &[&Seen]) -> Vec<(f64, f64)> {
    let mut p: Vec<(f64, f64)> = v.iter().map(|s| s.at).collect();
    p.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
    p
}

fn points_match(actual: &[(f64, f64)], expected: &[(f64, f64)], what: &str) {
    let mut e = expected.to_vec();
    e.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
    assert_eq!(actual.len(), e.len(), "{what}: count");
    for (a, x) in actual.iter().zip(&e) {
        close(a.0, x.0, 1e-12, what);
        close(a.1, x.1, 1e-12, what);
    }
}

/// F5: 13 bays (6 plain, 2 rotated 90°, 1 at scale (2, 1), 1 mirrored, a MINSERT of 3), each a
/// closed 2.5 × 5 rectangle and a stripe; 2 islands, each a rectangle and 2 trees at scale 0.5.
/// Block contents on layer 0 take the inserting layer.
#[test]
fn f5_expands_to_13_bays_and_4_trees() {
    let d = read("f5_blocks_inserts.dxf");
    let (seen, skipped, r) = run(&d, &ExpandLimits::default());
    let stats = r.expect("expands");
    assert!(skipped.is_empty(), "{skipped:?}");
    assert_eq!(on(&seen, "PARKING", "LWPOLYLINE").len(), 13);
    assert_eq!(on(&seen, "PARKING", "LINE").len(), 13);
    assert_eq!(on(&seen, "LANDSCAPE", "LWPOLYLINE").len(), 2);
    assert_eq!(on(&seen, "LANDSCAPE", "CIRCLE").len(), 4);
    assert!(
        seen.iter().all(|s| s.layer != "0"),
        "nothing stays on layer 0"
    );
    assert_eq!(stats.max_depth, 2);

    // README totals: bay area 175, perimeter 200, stripes 52; island area 20.
    let sum = |v: Vec<&Seen>, f: fn(&Seen) -> f64| v.iter().map(|s| f(s)).sum::<f64>();
    close(
        sum(on(&seen, "PARKING", "LWPOLYLINE"), |s| s.area),
        175.0,
        1e-9,
        "bay area",
    );
    close(
        sum(on(&seen, "PARKING", "LWPOLYLINE"), |s| s.length),
        200.0,
        1e-9,
        "bay perimeter",
    );
    close(
        sum(on(&seen, "PARKING", "LINE"), |s| s.length),
        52.0,
        1e-9,
        "stripes",
    );
    close(
        sum(on(&seen, "LANDSCAPE", "LWPOLYLINE"), |s| s.area),
        20.0,
        1e-9,
        "islands",
    );
    // Trees: radius 1 at scale 0.5.
    for t in on(&seen, "LANDSCAPE", "CIRCLE") {
        assert_eq!(t.scale, Some(0.5));
    }
}

/// Stripe starts, block point (1.25, 0.5) (base (0, 0)):
/// - scale 1 at x = 0, 2.5, …, 12.5: (1.25 + x, 0.5);
/// - 112, 90° at (20, 0): R(90°)(1.25, 0.5) = (−0.5, 1.25) → (19.5, 1.25); 113 at (20, 2.5): (19.5, 3.75);
/// - 114, scale (2, 1) at (0, 8): (2.5, 0.5) → (2.5, 8.5);
/// - 115, scale (−1, 1) at (40, 0): (−1.25, 0.5) → (38.75, 0.5);
/// - 116, MINSERT 3 × 1 at (0, 15), spacing 2.5: (1.25 + 2.5c, 15.5).
///
/// Tree centres: TREE's circle is at its base (100, 100), so it lands on each TREE insertion point
/// (201.25, 1) and (203.75, 1) in ISLAND (base (200, 0)): (1.25, 1) and (3.75, 1), then + (25, 0) and
/// + (25, 5).
#[test]
fn f5_positions_match_the_hand_computed_ones() {
    let d = read("f5_blocks_inserts.dxf");
    let (seen, _, r) = run(&d, &ExpandLimits::default());
    r.expect("expands");
    let stripes = [
        (1.25, 0.5),
        (3.75, 0.5),
        (6.25, 0.5),
        (8.75, 0.5),
        (11.25, 0.5),
        (13.75, 0.5),
        (19.5, 1.25),
        (19.5, 3.75),
        (2.5, 8.5),
        (38.75, 0.5),
        (1.25, 15.5),
        (3.75, 15.5),
        (6.25, 15.5),
    ];
    points_match(
        &sorted_points(&on(&seen, "PARKING", "LINE")),
        &stripes,
        "stripe starts",
    );
    let trees = [(26.25, 1.0), (28.75, 1.0), (26.25, 6.0), (28.75, 6.0)];
    points_match(
        &sorted_points(&on(&seen, "LANDSCAPE", "CIRCLE")),
        &trees,
        "tree centres",
    );

    // The mirrored bay (115) is the only placement with det < 0; the stretched one (114) has det 2.
    let bays = on(&seen, "PARKING", "LWPOLYLINE");
    let mirrored: Vec<_> = bays.iter().filter(|s| s.det < 0.0).collect();
    assert_eq!(mirrored.len(), 1);
    assert_eq!(mirrored[0].top, Some(0x115));
    assert!(bays.iter().any(|s| s.top == Some(0x114) && s.det == 2.0));
    // The MINSERT's three cells share their top-level handle.
    assert_eq!(bays.iter().filter(|s| s.top == Some(0x116)).count(), 3);
}

/// BricsCAD: 6 placements of `arrows_lv1`, each placing `arrows` (6 lines) 9 times: 324 lines. A
/// placement is mirrored when exactly one of the outer and inner inserts is: outer F8, FC, 11C, 120
/// are mirrored (4 of 6), inner D8, D9, E1, E2, EA, EB (6 of 9): 4 × 3 + 2 × 6 = 24 placements, 144
/// lines.
#[test]
fn bricscad_nested_and_mirrored_placements() {
    let d = read("insert_bricscad_level_1.dxf");
    let (seen, skipped, r) = run(&d, &ExpandLimits::default());
    r.expect("expands");
    assert!(skipped.is_empty(), "{skipped:?}");
    let lines = on(&seen, "0", "LINE");
    assert_eq!(lines.len(), 324);
    assert_eq!(lines.iter().filter(|s| s.det < 0.0).count(), 144);

    let find = |top: u64, insert: u64, handle: u64| {
        lines
            .iter()
            .find(|s| s.top == Some(top) && s.insert == Some(insert) && s.handle == Some(handle))
            .map(|s| s.at)
            .unwrap_or_else(|| panic!("{top:X}/{insert:X}/{handle:X}"))
    };
    // Line A9 starts at (0.5, 0) in `arrows`.
    // F8 (scale (−1, 1) at (7.939834716739, 7.861085190823)) / D8 (scale (−1, 1) at (1.389567226468, 0)):
    // inner (−0.5, 0) + (1.389567226468, 0) = (0.889567226468, 0); outer (−0.889567226468, 0) + P.
    let a = find(0xF8, 0xD8, 0xA9);
    close(a.0, 7.050267490271, 1e-12, "F8/D8/A9 x");
    close(a.1, 7.861085190823, 1e-12, "F8/D8/A9 y");
    // 118 (90° at (2.938493831503, 14.926252759746)) / E0 (45° at (0, 1.90822588289)):
    // inner R(45°)(0.5, 0) = (√2/4, √2/4) + (0, 1.90822588289) = (0.353553390593, 2.261779273483);
    // outer R(90°) → (−2.261779273483, 0.353553390593) + P = (0.676714558020, 15.279806150339).
    let b = find(0x118, 0xE0, 0xA9);
    close(b.0, 0.676714558020, 1e-11, "118/E0/A9 x");
    close(b.1, 15.279806150339, 1e-11, "118/E0/A9 y");
    // 120 (scale (1, −1), 90° at (10.888693717036, 15.016661555213)) / D9 (scale (1, −1) at
    // (2.754384116192, 0)): inner (0.5, 0) + (2.754384116192, 0) = (3.254384116192, 0); outer S → same;
    // R(90°) → (0, 3.254384116192) + P = (10.888693717036, 18.271045671405).
    let c = find(0x120, 0xD9, 0xA9);
    close(c.0, 10.888693717036, 1e-12, "120/D9/A9 x");
    close(c.1, 18.271045671405, 1e-12, "120/D9/A9 y");
}

// Synthetic drawings ------------------------------------------------------------------------------

/// A block definition: `0 BLOCK`, name, base (0, 0), its entities, `0 ENDBLK`.
fn block(name: &str, body: &str) -> String {
    format!("0\nBLOCK\n8\n0\n2\n{name}\n70\n0\n10\n0\n20\n0\n30\n0\n{body}0\nENDBLK\n8\n0\n")
}

fn line(layer: &str, handle: &str, x: f64) -> String {
    format!(
        "0\nLINE\n5\n{handle}\n8\n{layer}\n10\n{x}\n20\n0\n30\n0\n11\n{}\n21\n0\n31\n0\n",
        x + 1.0
    )
}

/// An INSERT; `extra` adds group pairs (scale, counts, spacing, extrusion).
fn insert(layer: &str, handle: &str, name: &str, x: f64, extra: &str) -> String {
    format!("0\nINSERT\n5\n{handle}\n8\n{layer}\n2\n{name}\n10\n{x}\n20\n0\n30\n0\n{extra}")
}

fn drawing(blocks: &str, entities: &str) -> Drawing {
    let text = format!(
        "0\nSECTION\n2\nBLOCKS\n{blocks}0\nENDSEC\n0\nSECTION\n2\nENTITIES\n{entities}0\nENDSEC\n0\nEOF\n"
    );
    read_dxf(text.as_bytes(), &Budget::new(Limits::DEFAULT_BUDGET_BYTES)).expect("reads")
}

/// Work is one unit per entity visited and per MINSERT cell. A MINSERT of 3 cells of a 2-line
/// block: 1 (the INSERT) + 3 (cells) + 3 × 2 (lines) = 10.
#[test]
fn work_limit_at_its_boundary() {
    let d = drawing(
        &block("B", &(line("0", "A1", 0.0) + &line("0", "A2", 5.0))),
        &insert("L", "10", "B", 0.0, "70\n3\n71\n1\n44\n10\n45\n0\n"),
    );
    let (seen, _, r) = run(
        &d,
        &ExpandLimits {
            max_work: 10,
            ..ExpandLimits::default()
        },
    );
    assert_eq!(r.expect("within").work, 10);
    assert_eq!(seen.len(), 6);
    let (_, _, r) = run(
        &d,
        &ExpandLimits {
            max_work: 9,
            ..ExpandLimits::default()
        },
    );
    assert_eq!(
        r.map(|s| s.work),
        Err(AnalysisError::Limit {
            what: "placements",
            limit: 9
        })
    );
}

/// A MINSERT of 10⁹ × 10⁹ is refused before any cell is placed.
#[test]
fn a_huge_minsert_is_refused_at_once() {
    let d = drawing(
        &block("B", &line("0", "A1", 0.0)),
        &insert(
            "L",
            "10",
            "B",
            0.0,
            "70\n1000000000\n71\n1000000000\n44\n1\n45\n1\n",
        ),
    );
    let (seen, _, r) = run(&d, &ExpandLimits::default());
    assert_eq!(
        r.map(|s| s.work),
        Err(AnalysisError::Limit {
            what: "placements",
            limit: ExpandLimits::default().max_work
        })
    );
    assert!(seen.is_empty());
}

/// Blocks B0 … B32, each inserting the next, the last holding a line. Inserting B0 from the drawing
/// nests 33 deep; inserting B1 nests 32, which is allowed.
#[test]
fn nesting_deeper_than_32_is_refused() {
    let mut blocks = String::new();
    for k in 0..33 {
        let body = if k == 32 {
            line("0", "A1", 0.0)
        } else {
            insert(
                "0",
                &format!("{:X}", 0x100 + k),
                &format!("B{}", k + 1),
                0.0,
                "",
            )
        };
        blocks += &block(&format!("B{k}"), &body);
    }
    let deep = drawing(&blocks, &insert("L", "10", "B0", 0.0, ""));
    let (_, _, r) = run(&deep, &ExpandLimits::default());
    assert_eq!(
        r.map(|s| s.max_depth),
        Err(AnalysisError::Limit {
            what: "nesting",
            limit: 32
        })
    );
    let ok = drawing(&blocks, &insert("L", "10", "B1", 0.0, ""));
    let (seen, _, r) = run(&ok, &ExpandLimits::default());
    assert_eq!(r.expect("32 deep").max_depth, 32);
    assert_eq!(seen.len(), 1);
}

/// Skipped placements: a block that inserts itself (cycle), an undefined block, a zero scale and a
/// non-plan extrusion are not expanded and are each reported once, with the inserting layer.
#[test]
fn cycles_missing_blocks_zero_scale_and_non_plan_inserts_are_reported() {
    let d = drawing(
        &(block(
            "SELF",
            &(line("0", "A1", 0.0) + &insert("0", "A2", "SELF", 0.0, "")),
        ) + &block("B", &line("0", "A3", 0.0))),
        &(insert("C", "10", "SELF", 0.0, "")
            + &insert("M", "11", "NOPE", 0.0, "")
            + &insert("Z", "12", "B", 0.0, "41\n0\n")
            + &insert("X", "13", "B", 0.0, "210\n1\n220\n0\n230\n0\n")),
    );
    let (seen, skipped, r) = run(&d, &ExpandLimits::default());
    r.expect("expands");
    assert_eq!(
        skipped,
        vec![
            Skipped {
                reason: Skip::Cycle,
                layer: "C".into(),
                handle: Some(0xA2)
            },
            Skipped {
                reason: Skip::MissingBlock,
                layer: "M".into(),
                handle: Some(0x11)
            },
            Skipped {
                reason: Skip::ZeroScale,
                layer: "Z".into(),
                handle: Some(0x12)
            },
            Skipped {
                reason: Skip::NonPlanExtrusion,
                layer: "X".into(),
                handle: Some(0x13)
            },
        ]
    );
    // SELF's own line is placed once, on the inserting layer.
    assert_eq!(seen.len(), 1);
    assert_eq!((seen[0].layer.as_str(), seen[0].handle), ("C", Some(0xA1)));
}

/// An insert with extrusion (0, 0, −1): its insertion point and the block are in that OCS, whose x
/// is world −x. Block line from (0, 0) placed at OCS (5, 0) starts at OCS (5, 0), world (−5, 0); it
/// ends at OCS (6, 0), world (−6, 0).
#[test]
fn an_insert_with_negative_extrusion_is_mirrored() {
    let d = drawing(
        &block("B", &line("0", "A1", 0.0)),
        &insert("L", "10", "B", 5.0, "210\n0\n220\n0\n230\n-1\n"),
    );
    let (seen, skipped, r) = run(&d, &ExpandLimits::default());
    r.expect("expands");
    assert!(skipped.is_empty());
    assert_eq!(seen[0].at, (-5.0, 0.0));
    assert!(seen[0].det < 0.0);
    close(seen[0].length, 1.0, 0.0, "length");
}

/// Layer 0 inside a block takes the inserting layer, through nesting; any other layer is kept.
#[test]
fn layer_zero_inherits_through_nesting() {
    let d = drawing(
        &(block("OUTER", &insert("0", "A1", "INNER", 0.0, ""))
            + &block("INNER", &(line("0", "A2", 0.0) + &line("KEEP", "A3", 2.0)))),
        &insert("TOP", "10", "OUTER", 0.0, ""),
    );
    let (seen, _, r) = run(&d, &ExpandLimits::default());
    r.expect("expands");
    let layers: Vec<(&str, Option<u64>)> =
        seen.iter().map(|s| (s.layer.as_str(), s.handle)).collect();
    assert_eq!(layers, vec![("TOP", Some(0xA2)), ("KEEP", Some(0xA3))]);
}

/// Expansion reserves from the budget it is given: with none, it fails cleanly.
#[test]
fn expansion_reserves_from_the_budget() {
    let d = read("f5_blocks_inserts.dxf");
    let r = expand(&d, &Budget::new(0), &ExpandLimits::default(), |_| {});
    assert!(matches!(r, Err(AnalysisError::Memory(_))), "{r:?}");
    let budget = Budget::new(Limits::DEFAULT_BUDGET_BYTES);
    expand(&d, &budget, &ExpandLimits::default(), |_| {}).expect("expands");
    assert_eq!(budget.used(), 0, "everything released");
    assert!(budget.peak() > 0, "something was reserved");
}

/// Paper-space entities are not expanded or placed (they are counted by the analysis, not measured).
#[test]
fn paper_space_is_not_placed() {
    let d = drawing(
        &block("B", &line("0", "A1", 0.0)),
        &(insert("L", "10", "B", 0.0, "67\n1\n") + &line("L", "11", 0.0)),
    );
    let (seen, _, r) = run(&d, &ExpandLimits::default());
    r.expect("expands");
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].handle, Some(0x11));
}
