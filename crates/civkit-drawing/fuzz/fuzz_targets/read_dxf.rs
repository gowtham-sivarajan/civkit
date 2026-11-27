// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 15:40 IST · last edited 2026-11-27 15:40 IST

//! Fuzz target for the DXF reader (ADR 0015 gate X6). Each input is read twice: under the default budget (as the ADR
//! requires) and under a small one, so budget exhaustion is fuzzed too. A panic, or a read whose requested heap passes
//! what it reserved (the X3 check), is a failure.

#![no_main]

use civkit_drawing::{Budget, Limits, ReadError, read_dxf};
use libfuzzer_sys::fuzz_target;
use peak_alloc::PeakAlloc;

#[global_allocator]
static HEAP: PeakAlloc = PeakAlloc;

/// As in tests/memory.rs, plus room for libFuzzer's own bookkeeping during the call.
const SLACK: usize = 64 * 1024;

fuzz_target!(|data: &[u8]| {
    for limit in [Limits::DEFAULT_BUDGET_BYTES, 64 * 1024] {
        let budget = Budget::new(limit);
        let base = HEAP.current_usage();
        HEAP.reset_peak_usage();
        let result = read_dxf(data, &budget);
        let measured = HEAP.peak_usage().saturating_sub(base);
        assert!(
            measured <= budget.peak() + SLACK,
            "heap requested ({measured} bytes) passed the heap reserved ({} bytes) under a budget of {limit}",
            budget.peak()
        );
        if let Err(ReadError::BudgetExhausted { limit: l, .. }) = &result {
            assert_eq!(*l, limit);
        }
        drop(result);
    }
});
