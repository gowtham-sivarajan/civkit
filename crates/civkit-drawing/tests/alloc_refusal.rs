// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 17:40 IST · last edited 2026-11-27 17:40 IST

//! ADR 0015, Q-D: "a refused allocation ends reading with a clean error." Here the allocator itself refuses: a capped
//! global allocator fails every allocation past an allowance. Under allowances from 0 bytes upward, every read must end
//! in success or `AllocationFailed`, with the budget set high so only the allocator refuses. An allocation the reader
//! makes without a fallible path would abort this test process instead, so a pass shows there is none.
//!
//! One test function, run alone in this binary: the cap is global, so parallel tests would interfere.

mod common;

use std::alloc::System;

use cap::Cap;
use civkit_drawing::{Budget, Limits, ReadError, read_dxf};
use common::*;

#[global_allocator]
static HEAP: Cap<System> = Cap::new(System, usize::MAX);

fn inputs() -> Vec<(String, Vec<u8>)> {
    let mut out: Vec<(String, Vec<u8>)> = ALL_FIXTURES
        .iter()
        .map(|n| (n.to_string(), fixture_bytes(n)))
        .collect();
    let mut s = String::from("0\nSECTION\n2\nENTITIES\n");
    for i in 0..300 {
        s.push_str(&format!(
            "0\nTYPE{i}\n8\nL{i}\n0\nLINE\n8\nL{i}\n10\n{i}\n20\n1\n11\n2\n21\n3\n"
        ));
        s.push_str(&format!("0\nTEXT\n8\nT\n1\n{}\n", "x".repeat(i % 50)));
    }
    s.push_str("0\nHATCH\n8\nH\n2\nSOLID\n70\n1\n91\n2\n92\n2\n72\n1\n73\n1\n93\n3\n10\n0\n20\n0\n42\n0.5\n10\n1\n20\n0\n10\n1\n20\n1\n97\n0\n");
    s.push_str("92\n1\n93\n2\n72\n1\n10\n0\n20\n0\n11\n1\n21\n0\n72\n2\n10\n0\n20\n0\n40\n1\n50\n0\n51\n90\n73\n1\n97\n0\n");
    s.push_str("0\nENDSEC\n0\nEOF\n");
    out.push((
        "names, counts, texts and a hatch".to_owned(),
        s.into_bytes(),
    ));
    out
}

#[test]
fn refused_allocations_end_in_a_clean_error() {
    for (name, bytes) in inputs() {
        let budget = Budget::new(Limits::DEFAULT_BUDGET_BYTES); // allocated before the allowance is set
        let (mut allowance, mut refusals) = (0usize, 0u32);
        loop {
            let base = HEAP.allocated();
            HEAP.set_limit(base + allowance)
                .expect("the limit is above what is allocated");
            let result = read_dxf(&bytes, &budget);
            HEAP.set_limit(usize::MAX).expect("lifting the limit");
            match result {
                Ok(drawing) => {
                    drop(drawing);
                    break;
                }
                Err(ReadError::AllocationFailed { .. }) => refusals += 1,
                Err(other) => panic!("{name} under an allowance of {allowance} bytes: {other}"),
            }
            allowance = allowance * 2 + 1;
        }
        assert!(
            refusals > 0,
            "{name}: the allocator never refused, so nothing was tested"
        );
    }
}
