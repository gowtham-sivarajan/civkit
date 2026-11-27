// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 10:50 IST · last edited 2026-11-27 10:50 IST

//! The engine's own answer, as text: reads one JSON request per line on stdin and writes `Registry::call_json`'s
//! output, one line each, on stdout. The cross-interface parity check (`tests/parity`) compares Python and MCP with
//! this, reading all three with a correctly rounding JSON parser.
//!
//! ```text
//! cargo run --release -p civkit-core --example call_json < requests.jsonl
//! ```

use std::io::{BufRead, Write};

use civkit_core::Registry;

fn main() -> std::io::Result<()> {
    let registry = Registry::with_builtins();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for line in std::io::stdin().lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        writeln!(out, "{}", registry.call_json(&line))?;
        out.flush()?;
    }
    Ok(())
}
