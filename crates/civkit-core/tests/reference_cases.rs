// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 10:58 IST · last edited 2026-11-25 10:58 IST

//! Runs every reference case in `tests/reference/` (repository root) against the engine.
//! Format: `tests/reference/README.md`. A case that fails to parse is a test failure, not a skip.

use civkit_core::{Registry, Request, calcs::demo::WeightedMean};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    #[allow(dead_code)]
    provenance: String,
    case: String,
    #[allow(dead_code)]
    title: String,
    source: Source,
    request: Request,
    expected: Expected,
    #[allow(dead_code)]
    verification: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    kind: SourceKind,
    citation: String,
    #[serde(default)]
    register_id: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    location: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    licence: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    url: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum SourceKind {
    /// A published, openly licensed or public-domain worked example (D-18).
    Published,
    /// Format demonstration only; allowed only for `math.demo.*`.
    Demo,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Expected {
    #[serde(default)]
    outputs: BTreeMap<String, ExpectedValue>,
    #[serde(default)]
    warnings: Vec<String>,
    #[serde(default)]
    errors: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedValue {
    value: f64,
    #[serde(default)]
    unit: Option<String>,
    tolerance: Tolerance,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
enum Tolerance {
    Relative(f64),
    Absolute(f64),
}

fn reference_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/reference")
}

fn case_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir).expect("tests/reference exists") {
        let path = entry.expect("readable entry").path();
        if path.is_dir() {
            files.extend(case_files(&path));
        } else if path.extension().is_some_and(|e| e == "json") {
            files.push(path);
        }
    }
    files.sort();
    files
}

fn registry() -> Registry {
    let mut r = Registry::with_builtins();
    r.register(Box::new(WeightedMean)).expect("demo registers");
    r
}

/// Checks one case; returns a description of every mismatch.
fn check(path: &Path, registry: &Registry) -> Vec<String> {
    let text = std::fs::read_to_string(path).expect("readable file");
    let case: Case = match serde_json::from_str(&text) {
        Ok(c) => c,
        Err(e) => return vec![format!("does not parse as a reference case: {e}")],
    };
    let mut problems = Vec::new();

    // The folder name must be the calculation ID, so cases are easy to find.
    let folder = path
        .parent()
        .and_then(Path::file_name)
        .and_then(|f| f.to_str())
        .unwrap_or_default();
    if folder != case.request.id {
        problems.push(format!(
            "folder '{folder}' does not match calculation '{}'",
            case.request.id
        ));
    }
    match case.source.kind {
        SourceKind::Demo if !case.request.id.starts_with("math.demo.") => {
            problems.push("'demo' sources are only allowed for math.demo.* calculations".into());
        }
        SourceKind::Published if case.source.register_id.is_none() => {
            problems.push(
                "published sources need 'register_id' (S-number in REFERENCE_SOURCES.md)".into(),
            );
        }
        _ => {}
    }
    if case.source.citation.trim().is_empty() {
        problems.push("empty citation".into());
    }

    match registry.call(&case.request) {
        Ok(response) => {
            if !case.expected.errors.is_empty() {
                problems.push(format!(
                    "expected errors {:?} but the call succeeded",
                    case.expected.errors
                ));
            }
            for (name, exp) in &case.expected.outputs {
                let Some(got) = response.outputs.get(name) else {
                    problems.push(format!("output '{name}' missing"));
                    continue;
                };
                let (ok, allowed) = match exp.tolerance {
                    Tolerance::Relative(r) => {
                        let allowed = r * exp.value.abs().max(f64::MIN_POSITIVE);
                        ((got.value - exp.value).abs() <= allowed, allowed)
                    }
                    Tolerance::Absolute(a) => ((got.value - exp.value).abs() <= a, a),
                };
                if !ok {
                    problems.push(format!(
                        "'{name}': got {}, expected {} ± {allowed} ({})",
                        got.value, exp.value, case.case
                    ));
                }
                if exp.unit != got.unit {
                    problems.push(format!(
                        "'{name}': unit {:?}, expected {:?}",
                        got.unit, exp.unit
                    ));
                }
            }
            let got_warnings: Vec<String> = response
                .warnings
                .iter()
                .map(|w| {
                    serde_json::to_value(w.code)
                        .ok()
                        .and_then(|v| v.as_str().map(str::to_owned))
                        .unwrap_or_default()
                })
                .collect();
            if got_warnings != case.expected.warnings {
                problems.push(format!(
                    "warnings {got_warnings:?}, expected {:?}",
                    case.expected.warnings
                ));
            }
        }
        Err(envelope) => {
            let got: Vec<String> = envelope
                .0
                .iter()
                .map(|e| {
                    serde_json::to_value(e.code)
                        .ok()
                        .and_then(|v| v.as_str().map(str::to_owned))
                        .unwrap_or_default()
                })
                .collect();
            if got != case.expected.errors {
                problems.push(format!(
                    "errors {got:?}, expected {:?}",
                    case.expected.errors
                ));
            }
        }
    }
    problems
}

#[test]
fn every_reference_case_passes() {
    let registry = registry();
    let files = case_files(&reference_dir());
    assert!(
        !files.is_empty(),
        "no reference cases found in {}",
        reference_dir().display()
    );

    let mut failures = Vec::new();
    for file in &files {
        for problem in check(file, &registry) {
            failures.push(format!("{}: {problem}", file.display()));
        }
    }
    assert!(
        failures.is_empty(),
        "{} reference-case problem(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn every_registered_calculation_has_a_reference_case() {
    // ENGINEERING_STANDARDS: no calculation without at least one reference case.
    let registry = registry();
    for d in registry.list() {
        let dir = reference_dir().join(&d.id);
        let has_case = dir.is_dir() && !case_files(&dir).is_empty();
        assert!(
            has_case,
            "{} has no reference case in tests/reference/{}/",
            d.id, d.id
        );
    }
}
