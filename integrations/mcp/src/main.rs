// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 00:24 IST · last edited 2026-11-27 22:53 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! `civkit-mcp`: CivKit's calculations as MCP tools, over stdio only (D-13, ADR 0013).
//!
//! ```text
//! civkit-mcp [--tools per-calc|generic|both] [--drawing-root <dir>]...
//! ```
//!
//! `--drawing-root` (repeatable) enables the read-only `drawing.analyze` tool for DXF files whose real path lies
//! inside one of the directories (D-28, API_DESIGN §10 and §13). Without it, the tool does not exist.
//!
//! Stdout carries only MCP messages; anything else goes to stderr. The server opens no socket and exits when stdin
//! closes. It serves 2025-11-25 clients (`initialize`) and 2026-07-28 clients (per-request version).

mod server;
mod tools;

use std::path::PathBuf;
use std::process::ExitCode;

use rmcp::ServiceExt;
use rmcp::service::ServerInitializeError;

use server::{CivKit, Surface};

const USAGE: &str = "usage: civkit-mcp [--tools per-calc|generic|both] [--drawing-root <dir>]...   (default: both, no drawing tool)";

/// The command line: the tool surface and the drawing roots (canonical directories).
fn parse_args(args: impl Iterator<Item = String>) -> Result<(Surface, Vec<PathBuf>), String> {
    let mut surface = Surface::Both;
    let mut roots = Vec::new();
    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        let (name, value) = match arg.split_once('=') {
            Some((name, value)) if name.starts_with("--") => {
                (name.to_owned(), Some(value.to_owned()))
            }
            _ => (arg.clone(), None),
        };
        let mut value = |name: &str| {
            value
                .clone()
                .or_else(|| args.next())
                .ok_or_else(|| format!("{name} needs a value\n{USAGE}"))
        };
        match name.as_str() {
            "--tools" => {
                let v = value("--tools")?;
                surface = Surface::parse(&v).ok_or_else(|| {
                    format!("--tools must be per-calc, generic or both, not {v:?}\n{USAGE}")
                })?;
            }
            "--drawing-root" => {
                let v = value("--drawing-root")?;
                let root = std::fs::canonicalize(&v)
                    .ok()
                    .filter(|r| r.is_dir())
                    .ok_or_else(|| format!("--drawing-root {v:?} is not a directory\n{USAGE}"))?;
                roots.push(root);
            }
            other => return Err(format!("unknown argument {other:?}\n{USAGE}")),
        }
    }
    Ok((surface, roots))
}

fn main() -> ExitCode {
    let (surface, roots) = match parse_args(std::env::args().skip(1)) {
        Ok(parsed) => parsed,
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
        let service = match CivKit::new(surface, roots)
            .serve(rmcp::transport::stdio())
            .await
        {
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
