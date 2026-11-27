// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 22:50 IST · last edited 2026-11-27 22:53 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! The Rust reference for the drawing parity tests (gate A8, `tests/parity`): reads one DXF file and writes
//! its report (JSON) or schedule (CSV) to stdout, exactly as `Analysis::to_json` / `to_csv` give them.
//!
//! ```text
//! analyze_dxf <path> [--format json|csv] [--detail layer|entity]
//! ```
//!
//! On an analysis error it writes the contract error envelope to stdout and exits with status 1.

use std::io::Write;
use std::process::ExitCode;

use civkit_drawing::{AnalysisOptions, analyze_dxf};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(path) = args.first() else {
        eprintln!("usage: analyze_dxf <path> [--format json|csv] [--detail layer|entity]");
        return ExitCode::from(2);
    };
    let flag = |name: &str| {
        args.windows(2)
            .find(|w| w[0] == name)
            .map(|w| w[1].as_str())
    };
    let options = AnalysisOptions {
        entity_rows: flag("--detail") == Some("entity"),
        ..AnalysisOptions::default()
    };
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("analyze_dxf: {path}: {e}");
            return ExitCode::from(2);
        }
    };
    let text = analyze_dxf(&bytes, &options).and_then(|a| match flag("--format") {
        Some("csv") => a.to_csv(),
        _ => a.to_json(),
    });
    let (text, status) = match text {
        Ok(t) => (t, ExitCode::SUCCESS),
        Err(e) => (
            format!(
                "{{\"error\":{{\"code\":\"{}\",\"message\":\"{}\"}}}}",
                e.code(),
                e.to_string().replace('\\', "\\\\").replace('"', "\\\"")
            ),
            ExitCode::from(1),
        ),
    };
    let mut out = std::io::stdout().lock();
    if out
        .write_all(text.as_bytes())
        .and_then(|()| out.flush())
        .is_err()
    {
        return ExitCode::FAILURE;
    }
    status
}
