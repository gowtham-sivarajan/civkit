// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 00:24 IST · last edited 2026-11-27 00:24 IST

//! `civkit-mcp`: CivKit's calculations as MCP tools, over stdio only (D-13, ADR 0013).
//!
//! ```text
//! civkit-mcp [--tools per-calc|generic|both]
//! ```
//!
//! Stdout carries only MCP messages; anything else goes to stderr. The server opens no socket and exits when stdin
//! closes. It serves 2025-11-25 clients (`initialize`) and 2026-07-28 clients (per-request version).

mod server;
mod tools;

use std::process::ExitCode;

use rmcp::ServiceExt;
use rmcp::service::ServerInitializeError;

use server::{CivKit, Surface};

const USAGE: &str = "usage: civkit-mcp [--tools per-calc|generic|both]   (default: both)";

fn parse_args(args: impl Iterator<Item = String>) -> Result<Surface, String> {
    let mut surface = Surface::Both;
    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        let value = match arg.as_str() {
            "--tools" => args
                .next()
                .ok_or_else(|| format!("--tools needs a value\n{USAGE}"))?,
            other => match other.strip_prefix("--tools=") {
                Some(v) => v.to_owned(),
                None => return Err(format!("unknown argument {other:?}\n{USAGE}")),
            },
        };
        surface = Surface::parse(&value).ok_or_else(|| {
            format!("--tools must be per-calc, generic or both, not {value:?}\n{USAGE}")
        })?;
    }
    Ok(surface)
}

fn main() -> ExitCode {
    let surface = match parse_args(std::env::args().skip(1)) {
        Ok(surface) => surface,
        Err(message) => {
            eprintln!("civkit-mcp: {message}");
            return ExitCode::from(2);
        }
    };
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(e) => {
            eprintln!("civkit-mcp: cannot start: {e}");
            return ExitCode::FAILURE;
        }
    };
    let outcome = runtime.block_on(async {
        let service = match CivKit::new(surface).serve(rmcp::transport::stdio()).await {
            Ok(service) => service,
            // Stdin closed before a client chose a lifecycle (nothing sent, or only `server/discover` probes):
            // a normal end, not a failure.
            Err(ServerInitializeError::ConnectionClosed(_)) => return Ok(()),
            Err(e) => return Err(e.to_string()),
        };
        service.waiting().await.map_err(|e| e.to_string())?;
        Ok::<(), String>(())
    });
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("civkit-mcp: {message}");
            ExitCode::FAILURE
        }
    }
}
