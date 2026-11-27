// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 15:40 IST · last edited 2026-11-27 23:21 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! Fuzz target for the DXF reader and the drawing analysis (ADR 0015 gate X6; ADR 0016 gate A9). Each input runs
//! twice:
//!
//! - under the default budget, the whole pipeline: `read_dxf`, then, when the read succeeds, `analyze` with the same
//!   budget and default options, then `to_json` and `to_csv`, each output dropped before the next (so fuzz inputs
//!   reach the report writers with drawing text in them). The heap requested across the pipeline must not pass the
//!   heap reserved by more than 16 KiB, the slack of X3 and A4.
//! - under a small budget, the reader alone, so budget exhaustion is fuzzed too (its check unchanged).
//!
//! Read errors and analysis errors (`DRAWING_UNREADABLE`, `DRAWING_LIMIT`) are valid outcomes. A panic, or heap
//! requested beyond the reservation, is a failure.

#![no_main]

use civkit_drawing::{AnalysisOptions, Budget, Limits, ReadError, analyze, read_dxf};
use libfuzzer_sys::fuzz_target;
use peak_alloc::PeakAlloc;

#[global_allocator]
static HEAP: PeakAlloc = PeakAlloc;

/// The default-budget pipeline: the slack of X3 (tests/memory.rs) and A4 (tests/analysis_memory.rs).
const PIPELINE_SLACK: usize = 16 * 1024;

/// The small-budget reader pass, as before: room for libFuzzer's own bookkeeping during the call.
const READER_SLACK: usize = 64 * 1024;

const SMALL_BUDGET: usize = 64 * 1024;

fuzz_target!(|data: &[u8]| {
    // The default budget: read, analyse, write the report and the schedule.
    let budget = Budget::new(Limits::DEFAULT_BUDGET_BYTES);
    let base = HEAP.current_usage();
    HEAP.reset_peak_usage();
    if let Ok(drawing) = read_dxf(data, &budget) {
        match analyze(drawing, &budget, &AnalysisOptions::default()) {
            Ok(analysis) => {
                // Writing can only run out of budget, which is a documented limit.
                if let Ok(json) = analysis.to_json() {
                    drop(json);
                }
                if let Ok(csv) = analysis.to_csv() {
                    drop(csv);
                }
            }
            Err(e) => {
                let code = e.code();
                assert!(
                    code == "DRAWING_LIMIT" || code == "DRAWING_UNREADABLE",
                    "{code}"
                );
            }
        }
    }
    let measured = HEAP.peak_usage().saturating_sub(base);
    assert!(
        measured <= budget.peak() + PIPELINE_SLACK,
        "read + analyse + report + schedule requested {measured} bytes but reserved {} under the default budget",
        budget.peak()
    );

    // A small budget: the reader alone, ending in success or the budget error.
    let budget = Budget::new(SMALL_BUDGET);
    let base = HEAP.current_usage();
    HEAP.reset_peak_usage();
    let result = read_dxf(data, &budget);
    let measured = HEAP.peak_usage().saturating_sub(base);
    assert!(
        measured <= budget.peak() + READER_SLACK,
        "heap requested ({measured} bytes) passed the heap reserved ({} bytes) under a budget of {SMALL_BUDGET}",
        budget.peak()
    );
    if let Err(ReadError::BudgetExhausted { limit, .. }) = &result {
        assert_eq!(*limit, SMALL_BUDGET);
    }
    drop(result);
});
