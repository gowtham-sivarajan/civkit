// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 15:40 IST · last edited 2026-11-27 15:40 IST

//! ADR 0015 gates X3 (budget accounting) and X4 (budget exhaustion), measured with a counting allocator.
//!
//! X3: for every fixture and adversarial input, the heap actually requested during a read is at most the heap the
//! reader reserved from its budget, plus a small fixed slack. So no allocation escapes the reservation.
//! X4: under budgets from a few bytes upward, every read ends in success or the budget error, and the heap requested
//! never passes the budget plus the slack.
//!
//! One test function, run alone in this binary: the allocator counts every thread, so parallel tests would mix.

#![allow(clippy::float_cmp)] // the reader must keep numbers bit for bit, so exact comparison is the test

mod common;

use civkit_drawing::{Budget, Limits, ReadError, read_dxf};
use common::*;
use peak_alloc::PeakAlloc;

#[global_allocator]
static HEAP: PeakAlloc = PeakAlloc;

/// Allocations outside the reservation that are allowed: the standard library's own small allocations (none are
/// expected; this guards against counting noise, not against unreserved storage, which grows with the input).
const SLACK: usize = 16 * 1024;

/// Reads `bytes` under `budget`; returns the result, the peak heap requested during the read and the peak reserved.
fn measure(bytes: &[u8], limit: usize) -> (Result<(), ReadError>, usize, usize) {
    let budget = Budget::new(limit);
    let base = HEAP.current_usage();
    HEAP.reset_peak_usage();
    let r = read_dxf(bytes, &budget).map(drop);
    let measured = HEAP.peak_usage().saturating_sub(base);
    (r, measured, budget.peak())
}

fn wrap(body: &str) -> Vec<u8> {
    format!("0\nSECTION\n2\nENTITIES\n{body}0\nENDSEC\n0\nEOF\n").into_bytes()
}

fn adversarial() -> Vec<(&'static str, Vec<u8>)> {
    let mut out = Vec::new();
    let mut s = String::new();
    for i in 0..50_000 {
        s.push_str(&format!(
            "0\nLINE\n8\n0\n10\n{i}.5\n20\n1.25\n11\n{i}.75\n21\n2.5\n"
        ));
    }
    out.push(("50 000 tiny LINEs", wrap(&s)));

    let mut s = String::new();
    for i in 0..20_000 {
        s.push_str(&format!("0\nTYPE{i}\n8\nLAYER{i}\n1\nx\n"));
    }
    out.push((
        "20 000 unrecognised entities, all types and layers distinct",
        wrap(&s),
    ));

    let mut s = String::from("0\nLWPOLYLINE\n8\n0\n90\n200000\n70\n1\n");
    for i in 0..200_000 {
        s.push_str(&format!("10\n{i}.5\n20\n{}.25\n42\n0.5\n", i % 7));
    }
    out.push(("one LWPOLYLINE, 200 000 vertices", wrap(&s)));

    let mut s = String::new();
    for i in 0..200 {
        s.push_str(&format!("0\nTEXT\n8\nT{i}\n1\n{}\n", "x".repeat(16 * 1024)));
    }
    out.push(("200 TEXT of 16 KiB", wrap(&s)));

    let mut s = String::from("0\nSECTION\n2\nTABLES\n0\nTABLE\n2\nLAYER\n");
    for i in 0..30_000 {
        s.push_str(&format!("0\nLAYER\n2\nLAYER_{i:06}\n70\n0\n62\n7\n"));
    }
    s.push_str("0\nENDTAB\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n");
    for i in 0..10_000 {
        s.push_str(&format!("0\nBLOCK\n2\nB{i}\n10\n0\n20\n0\n0\nLINE\n8\n0\n10\n0\n20\n0\n11\n1\n21\n1\n0\nENDBLK\n"));
    }
    s.push_str("0\nENDSEC\n0\nEOF\n");
    out.push(("30 000 layers and 10 000 blocks", s.into_bytes()));

    let mut s = String::from("0\nHATCH\n8\nH\n2\nSOLID\n70\n1\n91\n2000\n");
    for _ in 0..2000 {
        s.push_str("92\n2\n72\n1\n73\n1\n93\n50\n");
        for k in 0..50 {
            s.push_str(&format!("10\n{k}\n20\n{}\n42\n0.1\n", k * 2));
        }
        s.push_str("97\n0\n");
    }
    s.push_str("75\n0\n76\n1\n98\n0\n");
    out.push(("a HATCH of 2000 paths", wrap(&s)));

    let mut b = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1015\n9\n$DWGCODEPAGE\n3\nANSI_1252\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n".to_vec();
    for _ in 0..2000 {
        b.extend_from_slice(b"0\nTEXT\n8\nL\n1\n");
        b.extend(std::iter::repeat_n(0xE9u8, 500)); // é in ANSI_1252: two bytes in UTF-8
        b.extend_from_slice(b"\n");
    }
    b.extend_from_slice(b"0\nENDSEC\n0\nEOF\n");
    out.push(("2000 ANSI_1252 texts that grow in UTF-8", b));
    out
}

#[test]
fn budget_accounting_and_exhaustion() {
    let mut inputs: Vec<(String, Vec<u8>)> = ALL_FIXTURES
        .iter()
        .map(|n| (n.to_string(), fixture_bytes(n)))
        .collect();
    inputs.extend(adversarial().into_iter().map(|(n, b)| (n.to_owned(), b)));

    // X3: accounting, under the default budget.
    for (name, bytes) in &inputs {
        let (r, measured, reserved) = measure(bytes, Limits::DEFAULT_BUDGET_BYTES);
        assert!(r.is_ok(), "{name}: {r:?}");
        assert!(
            measured <= reserved + SLACK,
            "X3 {name}: requested {measured} bytes but reserved only {reserved} (+{SLACK} slack): an allocation escaped"
        );
    }

    // X3 on garbage: corrupted fixtures (the fuzz target's check, run here without libFuzzer). Whether a variant reads
    // or fails, the heap it requested must be covered by what it reserved.
    let mut seed: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    for name in ALL_FIXTURES {
        let original = fixture_bytes(name);
        for k in 0..60 {
            let mut b = original.clone();
            let i = (next() as usize) % b.len().max(1);
            match k % 4 {
                0 => b.truncate(i),
                1 => {
                    for _ in 0..8 {
                        let j = (next() as usize) % b.len();
                        b[j] = next() as u8;
                    }
                }
                2 => {
                    let end = (i + 300).min(b.len());
                    let slice = b[i..end].to_vec();
                    for _ in 0..40 {
                        b.extend_from_slice(&slice);
                    }
                }
                _ => {
                    b.splice(i..i, b"0\nLWPOLYLINE\n8\nX\n".iter().copied());
                }
            }
            let (_, measured, reserved) = measure(&b, Limits::DEFAULT_BUDGET_BYTES);
            assert!(
                measured <= reserved + SLACK,
                "X3 {name} corruption {k}: requested {measured} bytes but reserved only {reserved}"
            );
        }
    }

    // X4: exhaustion, under budgets from a few bytes upward to the one each input needs.
    for (name, bytes) in &inputs {
        let (_, _, needed) = measure(bytes, Limits::DEFAULT_BUDGET_BYTES);
        let mut limit = 1usize;
        loop {
            let (r, measured, _) = measure(bytes, limit);
            match r {
                // Reservation is deterministic: success means the limit covered everything the read reserves.
                Ok(()) => assert!(
                    limit >= needed,
                    "X4 {name}: succeeded under {limit} though it reserves {needed}"
                ),
                Err(ReadError::BudgetExhausted { limit: l, .. }) => assert_eq!(l, limit),
                Err(other) => panic!("X4 {name} under {limit}: {other}"),
            }
            assert!(
                measured <= limit + SLACK,
                "X4 {name}: requested {measured} bytes under a budget of {limit}"
            );
            if limit > needed {
                break;
            }
            limit = limit * 3 + 7; // grows quickly, and does not stay on powers of two
        }
    }
}
