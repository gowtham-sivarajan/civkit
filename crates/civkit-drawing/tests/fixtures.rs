// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 15:40 IST · last edited 2026-11-27 15:40 IST

//! ADR 0015 gate X1: every fixture reads with its expected entities and header values. Expected values come from
//! tests/fixtures/dxf/README.md (fixed by construction, checked with ezdxf) and from the files themselves.

#![allow(clippy::float_cmp)] // the reader must keep numbers bit for bit, so exact comparison is the test

mod common;

use std::collections::BTreeMap;

use civkit_drawing::{Boundary, EntityKind, PolylineSource};
use common::*;

const TOL: f64 = 1e-9;

fn header(name: &str) -> (Option<String>, Option<i16>, Option<i16>, Option<String>) {
    let d = read(name);
    (
        d.acadver().map(str::to_owned),
        d.insunits(),
        d.measurement(),
        d.codepage().map(str::to_owned),
    )
}

#[test]
fn headers_keep_not_stated_apart_from_stated() {
    let s = |x: &str| Some(x.to_owned());
    assert_eq!(
        header("f1_site_plan_m.dxf"),
        (s("AC1032"), Some(6), Some(1), s("ANSI_1252"))
    );
    assert_eq!(
        header("f2_site_plan_mm.dxf"),
        (s("AC1024"), Some(4), Some(1), s("ANSI_1252"))
    );
    assert_eq!(
        header("f3_units_mismatch.dxf"),
        (s("AC1027"), Some(1), Some(1), s("ANSI_1252"))
    );
    // R12 has no $INSUNITS and this file states no $MEASUREMENT: both "not stated", never a default.
    assert_eq!(
        header("f4_legacy_r12.dxf"),
        (s("AC1009"), None, None, s("ANSI_1252"))
    );
    assert_eq!(
        header("f5_blocks_inserts.dxf"),
        (s("AC1018"), Some(6), Some(1), s("ANSI_1252"))
    );
    assert_eq!(header("Minimal_DXF_AC1009.dxf"), (None, None, None, None));
    assert_eq!(
        header("insert_bricscad_level_1.dxf"),
        (s("AC1027"), Some(6), Some(1), s("ANSI_1252"))
    );
}

#[test]
fn the_minimal_file_reads_empty() {
    let d = read("Minimal_DXF_AC1009.dxf");
    assert!(d.entities().is_empty() && d.blocks().is_empty() && d.layers().is_empty());
    assert!(d.unsupported().is_empty() && d.notes().is_empty());
}

#[test]
fn f1_entities_layers_and_blocks() {
    let d = read("f1_site_plan_m.dxf");
    let expected: BTreeMap<&str, usize> = [
        ("ARC", 1),
        ("CIRCLE", 1),
        ("DIMENSION", 1),
        ("HATCH", 1),
        ("LINE", 1),
        ("LWPOLYLINE", 4),
        ("TEXT", 1),
    ]
    .into();
    assert_eq!(kinds(d.entities()), expected);
    let layers: Vec<&str> = d.layers().iter().map(|l| d.name(l.name)).collect();
    assert_eq!(
        layers,
        [
            "0",
            "Defpoints",
            "BOUNDARY",
            "ROAD_CL",
            "BUILDING",
            "WATER",
            "DRAIN",
            "HATCH",
            "TEXT",
            "DIM"
        ]
    );
    let blocks: Vec<&str> = d.blocks().iter().map(|b| d.name(b.name)).collect();
    assert_eq!(
        blocks,
        ["*Model_Space", "*Paper_Space", "*Paper_Space0", "*D1"]
    );
    // The dimension's block holds 3 LINE (read) and 2 SOLID and 1 MTEXT, which 0.1 does not interpret: counted.
    let d1 = d
        .blocks()
        .iter()
        .find(|b| d.name(b.name) == "*D1")
        .expect("*D1");
    assert_eq!(kinds(&d1.entities), [("LINE", 3)].into());
    let mut unsupported: BTreeMap<String, u64> = BTreeMap::new();
    for (kind, _layer, n) in d.unsupported() {
        *unsupported.entry(kind.to_owned()).or_default() += n;
    }
    assert_eq!(
        unsupported,
        [("MTEXT".to_owned(), 1), ("SOLID".to_owned(), 2)].into()
    );
}

fn f1_or_f4_geometry(name: &str, scale: f64, source: PolylineSource) {
    let d = read(name);
    let e = d.entities();
    let boundary = polylines(&on_layer(&d, e, "BOUNDARY"));
    assert_eq!(boundary.len(), 1);
    assert!(boundary[0].closed && boundary[0].source == source);
    assert_eq!(boundary[0].vertices.len(), 5);
    assert!(close(
        straight_area(&boundary[0].vertices),
        2550.0 * scale * scale,
        TOL
    ));
    assert!(close(
        length(&boundary[0].vertices, true),
        200.0 * scale,
        TOL
    ));

    let road = polylines(&on_layer(&d, e, "ROAD_CL"));
    assert_eq!(road.len(), 1);
    assert!(!road[0].closed);
    assert!(close(
        length(&road[0].vertices, false),
        (100.0 + 10.0 * std::f64::consts::PI) * scale,
        TOL
    ));

    let buildings = polylines(&on_layer(&d, e, "BUILDING"));
    let mut areas: Vec<f64> = buildings
        .iter()
        .map(|p| straight_area(&p.vertices) / (scale * scale))
        .collect();
    areas.sort_by(f64::total_cmp);
    assert!(
        close(areas[0], 96.0, TOL) && close(areas[1], 100.0, TOL),
        "{areas:?}"
    );

    let water: Vec<_> = on_layer(&d, e, "WATER");
    match &water[0].kind {
        EntityKind::Circle { center, radius } => {
            assert_eq!(
                (center.x, center.y, *radius),
                (50.0 * scale, 20.0 * scale, 3.0 * scale)
            );
        }
        other => panic!("WATER: {other:?}"),
    }
    let drain = on_layer(&d, e, "DRAIN");
    let arc = drain.iter().find_map(|x| match &x.kind {
        EntityKind::Arc {
            center,
            radius,
            start_deg,
            end_deg,
        } => Some((center.x, center.y, *radius, *start_deg, *end_deg)),
        _ => None,
    });
    assert_eq!(
        arc,
        Some((10.0 * scale, 30.0 * scale, 5.0 * scale, 0.0, 180.0))
    );
    let line = drain.iter().find_map(|x| match &x.kind {
        EntityKind::Line { start, end } => Some((start.x, start.y, end.x, end.y)),
        _ => None,
    });
    assert_eq!(
        line,
        Some((5.0 * scale, 30.0 * scale, 5.0 * scale, 5.0 * scale))
    );
    let text = on_layer(&d, e, "TEXT");
    match &text[0].kind {
        EntityKind::Text(t) => assert_eq!(&*t.value, "PLOT 17"),
        other => panic!("TEXT: {other:?}"),
    }
}

#[test]
fn f1_geometry_matches_the_readme() {
    f1_or_f4_geometry("f1_site_plan_m.dxf", 1.0, PolylineSource::Lightweight);
}

#[test]
fn f2_is_f1_times_a_thousand() {
    f1_or_f4_geometry("f2_site_plan_mm.dxf", 1000.0, PolylineSource::Lightweight);
}

#[test]
fn f4_old_polylines_have_the_same_geometry() {
    f1_or_f4_geometry("f4_legacy_r12.dxf", 1.0, PolylineSource::Old);
    let d = read("f4_legacy_r12.dxf");
    let expected: BTreeMap<&str, usize> = [
        ("ARC", 1),
        ("CIRCLE", 1),
        ("LINE", 1),
        ("POLYLINE", 4),
        ("TEXT", 1),
    ]
    .into();
    assert_eq!(kinds(d.entities()), expected);
    assert!(
        d.unsupported().is_empty(),
        "VERTEX and SEQEND belong to their POLYLINE"
    );
}

#[test]
fn f1_hatch_keeps_its_boundary() {
    let d = read("f1_site_plan_m.dxf");
    let hatch = d
        .entities()
        .iter()
        .find_map(|e| match &e.kind {
            EntityKind::Hatch(h) => Some(h),
            _ => None,
        })
        .expect("HATCH");
    assert_eq!(d.name(hatch.pattern), "ANSI31");
    assert!(!hatch.solid);
    assert_eq!(hatch.paths.len(), 1);
    match &hatch.paths[0].boundary {
        Boundary::Polyline { vertices, closed } => {
            assert!(*closed);
            assert_eq!(vertices.len(), 4);
            assert!(close(straight_area(vertices), 96.0, TOL));
        }
        other => panic!("{other:?}"),
    }
}

fn first_dimension(d: &civkit_drawing::Drawing) -> &civkit_drawing::Dimension {
    d.entities()
        .iter()
        .find_map(|e| match &e.kind {
            EntityKind::Dimension(x) => Some(x),
            _ => None,
        })
        .expect("DIMENSION")
}

#[test]
fn dimensions_keep_measurement_and_text_override() {
    let f1 = read("f1_site_plan_m.dxf");
    let a = first_dimension(&f1);
    assert_eq!(
        (a.dim_type, a.measurement, a.text.as_deref()),
        (0, Some(60.0), None)
    );
    assert_eq!(
        (
            a.first_point.x,
            a.first_point.y,
            a.second_point.x,
            a.second_point.y
        ),
        (0.0, 0.0, 60.0, 0.0)
    );
    assert_eq!(a.block.map(|b| f1.name(b)), Some("*D1"));
    let f3 = read("f3_units_mismatch.dxf");
    let b = first_dimension(&f3);
    assert_eq!((b.measurement, b.text.as_deref()), (Some(12.0), Some("15")));
}

#[test]
fn f3_planted_unit_problems_are_readable() {
    let d = read("f3_units_mismatch.dxf");
    let inserts: Vec<f64> = d
        .entities()
        .iter()
        .filter_map(|e| match &e.kind {
            EntityKind::Insert(i) => Some(i.scale.x),
            _ => None,
        })
        .collect();
    assert_eq!(inserts, [0.001, 1.0]);
    let mut areas: Vec<f64> = polylines(&on_layer(&d, d.entities(), "BUILDING"))
        .iter()
        .map(|p| straight_area(&p.vertices))
        .collect();
    areas.sort_by(f64::total_cmp);
    assert!(
        close(areas[0], 96.0, TOL)
            && close(areas[1], 100.0, TOL)
            && close(areas[2], 96_000_000.0, TOL),
        "{areas:?}"
    );
}

#[test]
fn f5_inserts_and_the_minsert_array() {
    let d = read("f5_blocks_inserts.dxf");
    assert_eq!(kinds(d.entities()), [("INSERT", 13)].into());
    let parking: Vec<_> = on_layer(&d, d.entities(), "PARKING")
        .into_iter()
        .filter_map(|e| match &e.kind {
            EntityKind::Insert(i) => Some(i),
            _ => None,
        })
        .collect();
    assert_eq!(parking.len(), 11);
    assert!(parking.iter().all(|i| d.name(i.block) == "PARKING_BAY"));
    // 13 bays once expanded: each insert places columns × rows copies (task 3 expands them).
    let bays: u64 = parking
        .iter()
        .map(|i| u64::from(i.columns) * u64::from(i.rows))
        .sum();
    assert_eq!(bays, 13);
    let array = parking.iter().find(|i| i.columns > 1).expect("the MINSERT");
    assert_eq!(
        (
            array.columns,
            array.rows,
            array.column_spacing,
            array.row_spacing
        ),
        (3, 1, 2.5, 0.0)
    );
    assert_eq!((array.point.x, array.point.y), (0.0, 15.0));
    assert_eq!(parking.iter().filter(|i| i.rotation_deg == 90.0).count(), 2);
    assert!(parking.iter().any(|i| (i.scale.x, i.scale.y) == (2.0, 1.0)));
    assert!(
        parking
            .iter()
            .any(|i| (i.scale.x, i.scale.y) == (-1.0, 1.0))
    );
    let landscape = on_layer(&d, d.entities(), "LANDSCAPE");
    assert_eq!(landscape.len(), 2);

    let block = |n: &str| {
        d.blocks()
            .iter()
            .find(|b| d.name(b.name) == n)
            .unwrap_or_else(|| panic!("{n}"))
    };
    let bay = block("PARKING_BAY");
    assert_eq!(
        kinds(&bay.entities),
        [("LINE", 1), ("LWPOLYLINE", 1)].into()
    );
    let island = block("ISLAND");
    assert_eq!(
        kinds(&island.entities),
        [("INSERT", 2), ("LWPOLYLINE", 1)].into()
    );
    for e in island.entities.iter() {
        if let EntityKind::Insert(i) = &e.kind {
            assert_eq!((d.name(i.block), i.scale.x, i.scale.y), ("TREE", 0.5, 0.5));
        }
    }
    assert_eq!(kinds(&block("TREE").entities), [("CIRCLE", 1)].into());
}

#[test]
fn bricscad_nested_and_mirrored_inserts() {
    let d = read("insert_bricscad_level_1.dxf");
    let expected: BTreeMap<&str, usize> = [("INSERT", 6), ("LWPOLYLINE", 6), ("TEXT", 12)].into();
    assert_eq!(kinds(d.entities()), expected);
    let top: Vec<(f64, f64, f64)> = d
        .entities()
        .iter()
        .filter_map(|e| match &e.kind {
            EntityKind::Insert(i) => Some((i.scale.x, i.scale.y, i.rotation_deg)),
            _ => None,
        })
        .collect();
    assert_eq!(
        top,
        [
            (1.0, 1.0, 0.0),
            (-1.0, 1.0, 0.0),
            (1.0, -1.0, 0.0),
            (1.0, 1.0, 90.0),
            (-1.0, 1.0, 90.0),
            (1.0, -1.0, 90.0)
        ]
    );
    let level1 = d
        .blocks()
        .iter()
        .find(|b| d.name(b.name) == "arrows_lv1")
        .expect("arrows_lv1");
    assert_eq!(
        kinds(&level1.entities),
        [("INSERT", 9), ("TEXT", 18)].into()
    );
    let arrows = d
        .blocks()
        .iter()
        .find(|b| d.name(b.name) == "arrows")
        .expect("arrows");
    assert_eq!(kinds(&arrows.entities), [("LINE", 6)].into());
}

#[test]
fn reading_is_deterministic() {
    for name in ALL_FIXTURES {
        assert_eq!(
            format!("{:?}", read(name)),
            format!("{:?}", read(name)),
            "{name}"
        );
    }
}
