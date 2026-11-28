// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 00:02 IST · last edited 2026-11-28 14:11 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! `civkit._civkit`: the thin Rust layer of the Python package (D-26, ADR 0014 point 4).
//!
//! Six functions: four over the calculation contract (ADR 0014), and two over the drawing report (D-28, ADR 0016
//! Q-A). Input parsing, results, errors and typed functions are plain Python in `python/civkit/`, so every
//! calculation goes through `Registry::call_json` and every drawing through `civkit_drawing::analyze_dxf`, and parity
//! with them holds by construction.

use std::sync::LazyLock;

use civkit_core::Registry;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

/// The built-in calculations, built once per process.
static REGISTRY: LazyLock<Registry> = LazyLock::new(Registry::with_builtins);

/// Runs one JSON request; returns the response or the error envelope as JSON, exactly as `Registry::call_json`.
#[pyfunction]
fn call_json(request: &str) -> String {
    REGISTRY.call_json(request)
}

/// Every registered descriptor, as a JSON array.
#[pyfunction]
fn list_json() -> PyResult<String> {
    serde_json::to_string(&REGISTRY.list()).map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

/// One descriptor as JSON (the latest version when `version` is `None`), or `None` if there is no such calculation.
#[pyfunction]
#[pyo3(signature = (id, version = None))]
fn describe_json(id: &str, version: Option<u32>) -> PyResult<Option<String>> {
    REGISTRY
        .describe(id, version)
        .map(serde_json::to_string)
        .transpose()
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

/// An analysis error as the contract's error envelope (API_DESIGN §5, §13).
fn drawing_error(e: &civkit_drawing::AnalysisError) -> String {
    serde_json::json!({ "error": { "code": e.code(), "message": e.to_string() } }).to_string()
}

/// Analyses DXF bytes at `detail` (`"layer"` or `"entity"`): `(true, report JSON)`, exactly `Analysis::to_json`, or
/// `(false, error envelope JSON)`. The engine side validates `detail`, so the Python layer validates nothing (ADR 0014
/// point 4).
#[pyfunction]
fn drawing_json(data: &[u8], detail: &str) -> (bool, String) {
    let entity_rows = match detail {
        "layer" => false,
        "entity" => true,
        _ => {
            let envelope = serde_json::json!({ "error": {
                "code": "INVALID_VALUE",
                "input": "detail",
                "message": "detail must be \"layer\" or \"entity\"",
            } });
            return (false, envelope.to_string());
        }
    };
    let options = civkit_drawing::AnalysisOptions {
        entity_rows,
        ..civkit_drawing::AnalysisOptions::default()
    };
    match civkit_drawing::analyze_dxf(data, &options).and_then(|a| a.to_json()) {
        Ok(text) => (true, text),
        Err(e) => (false, drawing_error(&e)),
    }
}

/// The quantity schedule of DXF bytes: `(true, CSV text)`, exactly `Analysis::to_csv`, or `(false, error envelope)`.
#[pyfunction]
fn drawing_csv(data: &[u8]) -> (bool, String) {
    let options = civkit_drawing::AnalysisOptions::default();
    match civkit_drawing::analyze_dxf(data, &options).and_then(|a| a.to_csv()) {
        Ok(text) => (true, text),
        Err(e) => (false, drawing_error(&e)),
    }
}

/// The CivKit engine version.
#[pyfunction]
fn version() -> &'static str {
    civkit_core::VERSION
}

/// The extension module `civkit._civkit`.
#[pymodule]
#[pyo3(name = "_civkit")]
fn civkit_ext(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(call_json, m)?)?;
    m.add_function(wrap_pyfunction!(list_json, m)?)?;
    m.add_function(wrap_pyfunction!(describe_json, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add("DISCLAIMER", civkit_core::DISCLAIMER)?;
    m.add_function(wrap_pyfunction!(drawing_json, m)?)?;
    m.add_function(wrap_pyfunction!(drawing_csv, m)?)?;
    Ok(())
}
