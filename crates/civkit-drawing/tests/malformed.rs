// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 15:40 IST · last edited 2026-11-27 15:40 IST

//! ADR 0015 gate X5: malformed input gives a clean error naming the problem and its line; limits are enforced;
//! encodings and unsupported content are reported, not guessed or dropped.

#![allow(clippy::float_cmp)] // the reader must keep numbers bit for bit, so exact comparison is the test

mod common;

use civkit_drawing::{
    Boundary, Budget, EntityKind, Limits, Note, ReadError, read_dxf, read_dxf_with,
};
use common::*;
use proptest::prelude::*;

fn read_text(text: &str) -> Result<civkit_drawing::Drawing, ReadError> {
    read_dxf(text.as_bytes(), &Budget::new(Limits::DEFAULT_BUDGET_BYTES))
}

fn entities(body: &str) -> String {
    format!("0\nSECTION\n2\nENTITIES\n{body}0\nENDSEC\n0\nEOF\n")
}

const LINE: &str = "0\nLINE\n8\nL\n10\n1\n20\n2\n30\n0\n11\n3\n21\n4\n31\n0\n";

#[test]
fn a_well_formed_minimal_drawing_reads() {
    let d = read_text(&entities(LINE)).expect("reads");
    match &d.entities()[0].kind {
        EntityKind::Line { start, end } => {
            assert_eq!((start.x, start.y, end.x, end.y), (1.0, 2.0, 3.0, 4.0))
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(d.name(d.entities()[0].layer), "L");
}

#[test]
fn crlf_leading_spaces_comments_and_unknown_sections_are_accepted() {
    let text = format!(
        "999\ncomment\n  0\r\nSECTION\r\n  2\r\nTHUMBNAILIMAGE\r\n 90\r\n3\r\n310\r\nABCDEF\r\n  0\r\nENDSEC\r\n{}",
        entities(LINE).replace('\n', "\r\n")
    );
    let d = read_text(&text).expect("reads");
    assert_eq!(d.entities().len(), 1);
}

#[test]
fn binary_dxf_is_not_supported_in_0_1() {
    let mut bytes = b"AutoCAD Binary DXF\r\n\x1a\x00".to_vec();
    bytes.extend_from_slice(&[0; 32]);
    assert!(matches!(
        read_dxf(&bytes, &Budget::new(1 << 20)),
        Err(ReadError::BinaryNotSupported)
    ));
}

#[test]
fn a_missing_eof_or_truncation_is_an_error() {
    assert!(matches!(
        read_text(""),
        Err(ReadError::UnexpectedEnd { .. })
    ));
    assert!(matches!(
        read_text("0\nSECTION\n2\nENTITIES\n"),
        Err(ReadError::UnexpectedEnd { .. })
    ));
    assert!(matches!(
        read_text(&format!("0\nSECTION\n2\nENTITIES\n{LINE}0\nENDSEC\n")),
        Err(ReadError::UnexpectedEnd { .. })
    ));
    // A code without its value line.
    assert!(matches!(
        read_text("0\nSECTION\n2"),
        Err(ReadError::UnexpectedEnd { .. })
    ));
}

#[test]
fn a_group_code_that_is_not_an_integer_names_its_line() {
    let r = read_text("0\nSECTION\nx2\nENTITIES\n0\nENDSEC\n0\nEOF\n");
    assert!(
        matches!(r, Err(ReadError::BadGroupCode { line: 3 })),
        "{r:?}"
    );
}

#[test]
fn bad_numbers_name_their_line_and_code() {
    let r = read_text(&entities("0\nLINE\n8\nL\n10\nabc\n"));
    assert!(
        matches!(r, Err(ReadError::InvalidNumber { line: 10, code: 10 })),
        "{r:?}"
    );
    for bad in ["NaN", "inf", "-infinity", "1e309"] {
        let r = read_text(&entities(&format!(
            "0\nCIRCLE\n8\nL\n10\n0\n20\n0\n40\n{bad}\n"
        )));
        assert!(
            matches!(r, Err(ReadError::NonFinite { line: 14, code: 40 })),
            "{bad}: {r:?}"
        );
    }
    let r = read_text(&entities("0\nLWPOLYLINE\n8\nL\n70\n99999999999\n"));
    assert!(
        matches!(r, Err(ReadError::InvalidNumber { line: 10, code: 70 })),
        "{r:?}"
    );
}

#[test]
fn structure_errors_are_clean() {
    assert!(matches!(
        read_text("0\nENDSEC\n0\nEOF\n"),
        Err(ReadError::Malformed { line: 2, .. })
    ));
    let unterminated_block =
        "0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nB\n0\nLINE\n8\n0\n0\nENDSEC\n0\nEOF\n";
    assert!(matches!(
        read_text(unterminated_block),
        Err(ReadError::Malformed { .. })
    ));
}

#[test]
fn the_input_size_limit_is_checked_first() {
    let limits = Limits {
        max_input_bytes: 10,
        ..Limits::default()
    };
    let r = read_dxf_with(entities(LINE).as_bytes(), &Budget::new(1 << 20), &limits);
    assert!(
        matches!(r, Err(ReadError::InputTooLarge { limit: 10, .. })),
        "{r:?}"
    );
}

#[test]
fn per_item_caps() {
    let limits = Limits {
        max_vertices: 3,
        ..Limits::default()
    };
    let poly = entities(
        "0\nLWPOLYLINE\n8\nL\n90\n4\n10\n0\n20\n0\n10\n1\n20\n0\n10\n1\n20\n1\n10\n0\n20\n1\n",
    );
    let r = read_dxf_with(poly.as_bytes(), &Budget::new(1 << 20), &limits);
    assert!(
        matches!(r, Err(ReadError::TooManyVertices { limit: 3, .. })),
        "{r:?}"
    );

    let limits = Limits {
        max_string_bytes: 4,
        ..Limits::default()
    };
    let text = entities("0\nTEXT\n8\nL\n1\nPLOT 17\n");
    let r = read_dxf_with(text.as_bytes(), &Budget::new(1 << 20), &limits);
    assert!(
        matches!(r, Err(ReadError::StringTooLong { limit: 4, .. })),
        "{r:?}"
    );

    let long_type = format!("0\n{}\n8\nL\n", "X".repeat(300));
    let r = read_text(&entities(&long_type));
    assert!(
        matches!(r, Err(ReadError::StringTooLong { limit: 256, .. })),
        "{r:?}"
    );
}

#[test]
fn a_huge_stated_count_does_not_allocate_ahead() {
    // 93 states 2 000 000 000 vertices; the cap refuses it before any allocation for them.
    let hatch =
        entities("0\nHATCH\n8\nH\n2\nSOLID\n70\n1\n91\n1\n92\n2\n72\n0\n73\n1\n93\n2000000000\n");
    let budget = Budget::new(1 << 20);
    let r = read_dxf(hatch.as_bytes(), &budget);
    assert!(matches!(r, Err(ReadError::TooManyVertices { .. })), "{r:?}");
    assert!(budget.peak() < 64 * 1024, "peak {}", budget.peak());
}

#[test]
fn the_budget_is_enforced() {
    let r = read_dxf(&fixture_bytes("f1_site_plan_m.dxf"), &Budget::new(1024));
    assert!(
        matches!(r, Err(ReadError::BudgetExhausted { limit: 1024, .. })),
        "{r:?}"
    );
}

#[test]
fn unsupported_entities_are_counted_by_type_and_layer() {
    let body = "0\nMTEXT\n8\nNOTES\n1\nhello\n0\nMTEXT\n8\nNOTES\n0\nSPLINE\n8\nCURVES\n0\nPOLYLINE\n8\nP3D\n66\n1\n70\n8\n0\nVERTEX\n8\nP3D\n10\n0\n20\n0\n30\n5\n0\nSEQEND\n8\nP3D\n";
    let d = read_text(&entities(body)).expect("reads");
    assert!(d.entities().is_empty());
    let got: Vec<(String, String, u64)> = d
        .unsupported()
        .into_iter()
        .map(|(k, l, n)| (k.to_owned(), l.to_owned(), n))
        .collect();
    assert_eq!(
        got,
        [
            ("MTEXT".to_owned(), "NOTES".to_owned(), 2),
            ("POLYLINE (3D)".to_owned(), "P3D".to_owned(), 1),
            ("SPLINE".to_owned(), "CURVES".to_owned(), 1),
        ]
    );
}

#[test]
fn insert_attributes_are_consumed_with_their_insert() {
    let body =
        "0\nINSERT\n8\nL\n66\n1\n2\nB\n10\n1\n20\n2\n0\nATTRIB\n8\nL\n1\nvalue\n0\nSEQEND\n8\nL\n";
    let d = read_text(&entities(body)).expect("reads");
    assert_eq!(kinds(d.entities()), [("INSERT", 1)].into());
    let got: Vec<(String, u64)> = d
        .unsupported()
        .into_iter()
        .map(|(k, _, n)| (k.to_owned(), n))
        .collect();
    assert_eq!(
        got,
        [("ATTRIB".to_owned(), 1)],
        "attributes are not interpreted in 0.1, so they are counted"
    );
}

#[test]
fn hatch_edges_of_unsupported_kinds_are_reported() {
    // One path of edges: a line, then an ellipse (not supported in 0.1).
    let body = "0\nHATCH\n5\n1F\n8\nH\n2\nSOLID\n70\n1\n91\n1\n92\n1\n93\n2\n\
                72\n1\n10\n0\n20\n0\n11\n1\n21\n0\n\
                72\n3\n10\n0\n20\n0\n11\n2\n21\n0\n40\n0.5\n50\n0\n51\n360\n73\n1\n\
                97\n0\n75\n0\n76\n1\n98\n0\n";
    let d = read_text(&entities(body)).expect("reads");
    let EntityKind::Hatch(h) = &d.entities()[0].kind else {
        panic!("hatch")
    };
    assert!(
        matches!(h.paths[0].boundary, Boundary::NotSupported { .. }),
        "{:?}",
        h.paths[0].boundary
    );
    assert!(
        d.notes().iter().any(|n| matches!(
            n,
            Note::HatchEdgeNotSupported {
                handle: Some(0x1F),
                ..
            }
        )),
        "{:?}",
        d.notes()
    );
}

#[test]
fn text_encodings() {
    // R2007 and later: UTF-8.
    let utf8 = format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n{}",
        entities("0\nTEXT\n8\nL\n1\ncafé\n")
    );
    let d = read_text(&utf8).expect("reads");
    let EntityKind::Text(t) = &d.entities()[0].kind else {
        panic!()
    };
    assert_eq!(&*t.value, "café");

    // Before R2007 with ANSI_1252: the byte 0xE9 is é; \U+XXXX escapes are decoded in every version.
    let mut ansi = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1015\n9\n$DWGCODEPAGE\n3\nANSI_1252\n0\nENDSEC\n".to_vec();
    ansi.extend_from_slice(
        b"0\nSECTION\n2\nENTITIES\n0\nTEXT\n8\nL\n1\ncaf\xE9 \\U+00B0\n0\nENDSEC\n0\nEOF\n",
    );
    let d = read_dxf(&ansi, &Budget::new(1 << 20)).expect("reads");
    let EntityKind::Text(t) = &d.entities()[0].kind else {
        panic!()
    };
    assert_eq!(&*t.value, "café °");
    assert!(d.notes().is_empty());

    // Another code page: geometry is read, undecodable text is replaced and reported, never guessed.
    let mut other =
        b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1015\n9\n$DWGCODEPAGE\n3\nANSI_932\n0\nENDSEC\n"
            .to_vec();
    other.extend_from_slice(
        b"0\nSECTION\n2\nENTITIES\n0\nTEXT\n8\nL\n1\n\x93\xFA\n0\nENDSEC\n0\nEOF\n",
    );
    let d = read_dxf(&other, &Budget::new(1 << 20)).expect("reads");
    let EntityKind::Text(t) = &d.entities()[0].kind else {
        panic!()
    };
    assert_eq!(&*t.value, "\u{FFFD}\u{FFFD}");
    assert!(
        d.notes()
            .iter()
            .any(|n| matches!(n, Note::TextNotDecoded { count: 1, .. })),
        "{:?}",
        d.notes()
    );
}

fn mutate(bytes: &[u8], kind: u8, a: usize, b: u8) -> Vec<u8> {
    let mut v = bytes.to_vec();
    if v.is_empty() {
        return v;
    }
    let i = a % v.len();
    match kind % 4 {
        0 => v.truncate(i),
        1 => v[i] = b,
        2 => {
            v.remove(i);
        }
        _ => {
            let end = (i + 64).min(v.len());
            let s = v[i..end].to_vec();
            for _ in 0..(b % 8) {
                v.extend_from_slice(&s);
            }
        }
    }
    v
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 512, ..ProptestConfig::default() })]

    /// Arbitrary bytes never panic: they read or give an error.
    #[test]
    fn arbitrary_bytes_never_panic(bytes in proptest::collection::vec(any::<u8>(), 0..2048)) {
        let _ = read_dxf(&bytes, &Budget::new(1 << 20));
    }

    /// Corrupted fixtures never panic.
    #[test]
    fn corrupted_fixtures_never_panic(f in 0usize..7, kind in any::<u8>(), a in any::<usize>(), b in any::<u8>()) {
        let bytes = mutate(&fixture_bytes(ALL_FIXTURES[f]), kind, a, b);
        let _ = read_dxf(&bytes, &Budget::new(Limits::DEFAULT_BUDGET_BYTES));
    }
}
