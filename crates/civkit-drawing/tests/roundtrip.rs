// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 15:40 IST · last edited 2026-11-27 15:40 IST

//! Property (DXF_READER_PROPOSAL §6): writing a random small drawing in the 0.1 subset and reading it back gives the
//! same entities, bit for bit. Numbers are written in Rust's shortest round-trip form, so this also shows that the
//! reader parses numbers correctly rounded (ADR 0015, R6).

#![allow(clippy::float_cmp)] // the reader must keep numbers bit for bit, so exact comparison is the test

use civkit_drawing::{Budget, EntityKind, Limits, read_dxf};
use proptest::prelude::*;

#[derive(Debug, Clone)]
enum Item {
    Line([f64; 4]),
    Circle([f64; 3]),
    Arc([f64; 3], f64, f64),
    Poly(Vec<(f64, f64, f64)>, bool),
}

fn coord() -> impl Strategy<Value = f64> {
    prop_oneof![
        -1e6..1e6f64,
        any::<f64>().prop_filter("finite", |x| x.is_finite()),
        Just(0.1),
        Just(1.0 / 3.0),
        Just(-0.0),
    ]
}

fn item() -> impl Strategy<Value = Item> {
    prop_oneof![
        prop::array::uniform4(coord()).prop_map(Item::Line),
        (coord(), coord(), 0.001..1e4f64).prop_map(|(x, y, r)| Item::Circle([x, y, r])),
        (
            coord(),
            coord(),
            0.001..1e4f64,
            0.0..360.0f64,
            0.0..360.0f64
        )
            .prop_map(|(x, y, r, s, e)| Item::Arc([x, y, r], s, e)),
        (
            prop::collection::vec((coord(), coord(), -2.0..2.0f64), 2..40),
            any::<bool>()
        )
            .prop_map(|(v, c)| Item::Poly(v, c)),
    ]
}

fn write(items: &[Item]) -> String {
    let mut s = String::from("0\nSECTION\n2\nENTITIES\n");
    for it in items {
        match it {
            Item::Line([a, b, c, d]) => s.push_str(&format!(
                "0\nLINE\n8\n0\n10\n{a}\n20\n{b}\n11\n{c}\n21\n{d}\n"
            )),
            Item::Circle([x, y, r]) => {
                s.push_str(&format!("0\nCIRCLE\n8\n0\n10\n{x}\n20\n{y}\n40\n{r}\n"))
            }
            Item::Arc([x, y, r], a, b) => s.push_str(&format!(
                "0\nARC\n8\n0\n10\n{x}\n20\n{y}\n40\n{r}\n50\n{a}\n51\n{b}\n"
            )),
            Item::Poly(v, closed) => {
                s.push_str(&format!(
                    "0\nLWPOLYLINE\n8\n0\n90\n{}\n70\n{}\n",
                    v.len(),
                    u8::from(*closed)
                ));
                for (x, y, b) in v {
                    s.push_str(&format!("10\n{x}\n20\n{y}\n42\n{b}\n"));
                }
            }
        }
    }
    s.push_str("0\nENDSEC\n0\nEOF\n");
    s
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    #[test]
    fn written_drawings_read_back_exactly(items in prop::collection::vec(item(), 0..30)) {
        let d = read_dxf(write(&items).as_bytes(), &Budget::new(Limits::DEFAULT_BUDGET_BYTES)).expect("reads");
        prop_assert_eq!(d.entities().len(), items.len());
        for (it, e) in items.iter().zip(d.entities()) {
            let bits = |x: f64| x.to_bits();
            match (it, &e.kind) {
                (Item::Line([a, b, c, dd]), EntityKind::Line { start, end }) => {
                    prop_assert_eq!([bits(*a), bits(*b), bits(*c), bits(*dd)], [bits(start.x), bits(start.y), bits(end.x), bits(end.y)]);
                }
                (Item::Circle([x, y, r]), EntityKind::Circle { center, radius }) => {
                    prop_assert_eq!([bits(*x), bits(*y), bits(*r)], [bits(center.x), bits(center.y), bits(*radius)]);
                }
                (Item::Arc([x, y, r], a, b), EntityKind::Arc { center, radius, start_deg, end_deg }) => {
                    prop_assert_eq!(
                        [bits(*x), bits(*y), bits(*r), bits(*a), bits(*b)],
                        [bits(center.x), bits(center.y), bits(*radius), bits(*start_deg), bits(*end_deg)]
                    );
                }
                (Item::Poly(v, closed), EntityKind::Polyline(p)) => {
                    prop_assert_eq!(p.closed, *closed);
                    let got: Vec<[u64; 3]> = p.vertices.iter().map(|w| [bits(w.x), bits(w.y), bits(w.bulge)]).collect();
                    let want: Vec<[u64; 3]> = v.iter().map(|(x, y, b)| [bits(*x), bits(*y), bits(*b)]).collect();
                    prop_assert_eq!(got, want);
                }
                (want, got) => prop_assert!(false, "{want:?} read as {got:?}"),
            }
        }
    }
}
