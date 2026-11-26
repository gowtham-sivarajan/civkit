// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 00:02 IST · last edited 2026-11-27 00:02 IST

//! `civkit._civkit`: the thin Rust layer of the Python package (D-26, ADR 0014 point 4).
//!
//! Exactly four functions, all over the JSON contract. Input parsing, results, errors and typed functions are plain
//! Python in `python/civkit/`, so every calculation goes through `Registry::call_json` and parity with it holds by
//! construction.

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
    Ok(())
}
