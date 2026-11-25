// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 10:58 IST · last edited 2026-11-26 00:53 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! Runs every reference case in `tests/reference/` (repository root) against the engine.
//! Format: `tests/reference/README.md`. A case that fails to parse is a test failure, not a skip.

use civkit_core::{Number, OutputValue, Registry, Request, calcs::demo::WeightedMean};
use serde::Deserialize;
use serde_json::json;
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
    value: ExpectedNumber,
    #[serde(default)]
    unit: Option<String>,
    tolerance: Tolerance,
}

/// An expected output: one number, or a list. A list entry may be `null`, meaning "the source does not print
/// this value, so it is not checked" (CONTRACT_LISTS_PROPOSAL §3.7).
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ExpectedNumber {
    Scalar(f64),
    List(Vec<Option<f64>>),
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

/// Whether `got` is within the tolerance of `exp`, and the allowed deviation (for messages).
fn within(tolerance: &Tolerance, got: f64, exp: f64) -> (bool, f64) {
    let allowed = match *tolerance {
        Tolerance::Relative(r) => r * exp.abs().max(f64::MIN_POSITIVE),
        Tolerance::Absolute(a) => a,
    };
    ((got - exp).abs() <= allowed, allowed)
}

/// Compares one output with its expectation; returns a description of every mismatch.
///
/// A list is compared entry by entry with the output's tolerance; `None` entries are not checked, but at least
/// one entry must be. `item` is the output's item label ("segment"), used to say which entry is wrong.
fn compare_output(
    name: &str,
    exp: &ExpectedValue,
    got: &OutputValue,
    item: Option<&str>,
) -> Vec<String> {
    let mut problems = Vec::new();
    match (&exp.value, &got.value) {
        (ExpectedNumber::Scalar(e), Number::Scalar(g)) => {
            let (ok, allowed) = within(&exp.tolerance, *g, *e);
            if !ok {
                problems.push(format!("'{name}': got {g}, expected {e} ± {allowed}"));
            }
        }
        (ExpectedNumber::List(entries), Number::List(values)) => {
            if entries.iter().all(Option::is_none) {
                problems.push(format!(
                    "'{name}': an expected list needs at least one checked entry (not all null)"
                ));
            }
            if entries.len() != values.len() {
                problems.push(format!(
                    "'{name}': expected {} entries, got {}",
                    entries.len(),
                    values.len()
                ));
            } else {
                let label = item.unwrap_or("entry");
                for (k, (e, g)) in entries.iter().zip(values).enumerate() {
                    let Some(e) = e else { continue };
                    let (ok, allowed) = within(&exp.tolerance, *g, *e);
                    if !ok {
                        problems.push(format!(
                            "'{name}': {label} {}: got {g}, expected {e} ± {allowed}",
                            k + 1
                        ));
                    }
                }
            }
        }
        (ExpectedNumber::List(_), Number::Scalar(_)) => {
            problems.push(format!("'{name}': expected a list, got a scalar"));
        }
        (ExpectedNumber::Scalar(_), Number::List(values)) => {
            problems.push(format!(
                "'{name}': expected a scalar, got a list of {} entries",
                values.len()
            ));
        }
    }
    if exp.unit != got.unit {
        problems.push(format!(
            "'{name}': unit {:?}, expected {:?}",
            got.unit, exp.unit
        ));
    }
    problems
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
                // The item label of a list output ("segment") comes from the descriptor, for messages.
                let item = registry
                    .describe(&response.id, Some(response.version))
                    .and_then(|d| d.outputs.iter().find(|o| &o.name == name))
                    .and_then(|o| o.length.as_ref())
                    .map(|l| l.item.as_str());
                problems.extend(
                    compare_output(name, exp, got, item)
                        .into_iter()
                        .map(|p| format!("{p} ({})", case.case)),
                );
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

// --- Output comparison, including list outputs (D-23, CONTRACT_LISTS_PROPOSAL §3.7) ----------------------

fn expected(json: serde_json::Value) -> ExpectedValue {
    serde_json::from_value(json).expect("expected value parses")
}

fn got(value: Number, unit: Option<&str>) -> OutputValue {
    OutputValue {
        value,
        unit: unit.map(str::to_owned),
    }
}

#[test]
fn scalar_outputs_compare_as_before() {
    let exp = expected(json!({ "value": 2.96, "unit": "m", "tolerance": { "relative": 0.005 } }));
    assert!(compare_output("q", &exp, &got(Number::Scalar(2.961), Some("m")), None).is_empty());
    let p = compare_output("q", &exp, &got(Number::Scalar(3.1), Some("m")), None);
    assert_eq!(p.len(), 1, "{p:?}");
    assert!(p[0].contains("got 3.1") && p[0].contains("2.96"), "{p:?}");
    let p = compare_output("q", &exp, &got(Number::Scalar(2.96), Some("ft")), None);
    assert!(p.iter().any(|m| m.contains("unit")), "{p:?}");
}

#[test]
fn a_list_is_checked_entry_by_entry_with_the_same_tolerance() {
    let exp = expected(
        json!({ "value": [10.0, 20.0, 30.0], "unit": "m", "tolerance": { "absolute": 0.5 } }),
    );
    let ok = got(Number::List(vec![10.2, 19.8, 30.5]), Some("m"));
    assert!(compare_output("v", &exp, &ok, Some("segment")).is_empty());
}

#[test]
fn a_list_mismatch_names_the_item_and_its_number() {
    let exp = expected(
        json!({ "value": [10.0, 20.0, 30.0], "unit": "m", "tolerance": { "absolute": 0.5 } }),
    );
    let bad = got(Number::List(vec![10.0, 21.0, 30.0]), Some("m"));
    let p = compare_output("v", &exp, &bad, Some("segment"));
    assert_eq!(p.len(), 1, "{p:?}");
    assert!(
        p[0].contains("segment 2") && p[0].contains("got 21") && p[0].contains("expected 20"),
        "{p:?}"
    );
}

#[test]
fn null_entries_are_not_checked_but_at_least_one_entry_must_be() {
    let exp = expected(
        json!({ "value": [null, 20.0, null], "unit": "m", "tolerance": { "absolute": 0.5 } }),
    );
    // Entries 1 and 3 are not printed by the source: anything goes there.
    let any = got(Number::List(vec![999.0, 20.0, -5.0]), Some("m"));
    assert!(compare_output("v", &exp, &any, Some("segment")).is_empty());

    let nothing =
        expected(json!({ "value": [null, null], "unit": "m", "tolerance": { "absolute": 0.5 } }));
    let p = compare_output(
        "v",
        &nothing,
        &got(Number::List(vec![1.0, 2.0]), Some("m")),
        None,
    );
    assert!(p.iter().any(|m| m.contains("at least one")), "{p:?}");
    let empty = expected(json!({ "value": [], "unit": "m", "tolerance": { "absolute": 0.5 } }));
    let p = compare_output("v", &empty, &got(Number::List(vec![]), Some("m")), None);
    assert!(p.iter().any(|m| m.contains("at least one")), "{p:?}");
}

#[test]
fn list_length_and_shape_must_agree() {
    let exp =
        expected(json!({ "value": [10.0, 20.0], "unit": "m", "tolerance": { "absolute": 0.5 } }));
    let p = compare_output("v", &exp, &got(Number::List(vec![10.0]), Some("m")), None);
    assert!(
        p.iter().any(|m| m.contains("2 entries") && m.contains('1')),
        "{p:?}"
    );
    let p = compare_output("v", &exp, &got(Number::Scalar(10.0), Some("m")), None);
    assert!(p.iter().any(|m| m.contains("expected a list")), "{p:?}");

    let scalar = expected(json!({ "value": 10.0, "unit": "m", "tolerance": { "absolute": 0.5 } }));
    let p = compare_output(
        "v",
        &scalar,
        &got(Number::List(vec![10.0, 11.0]), Some("m")),
        None,
    );
    assert!(p.iter().any(|m| m.contains("expected a scalar")), "{p:?}");
}

#[test]
fn a_list_output_unit_is_checked_once() {
    let exp =
        expected(json!({ "value": [1.0, 2.0], "unit": "m", "tolerance": { "absolute": 0.5 } }));
    let p = compare_output(
        "v",
        &exp,
        &got(Number::List(vec![1.0, 2.0]), Some("mm")),
        None,
    );
    assert_eq!(p.len(), 1, "{p:?}");
    assert!(p[0].contains("unit"), "{p:?}");
}

/// Phase 2D integrity: a descriptor's `examples` and the case files on disk must be the same set.
///
/// A missing file would send a reader (or a generated doc page) to nothing; an unlisted file would be a case the
/// descriptor does not admit to. Both are documentation lies that no other test catches.
#[test]
fn every_descriptor_example_exists_and_every_case_file_is_listed() {
    let registry = registry();
    let repo_root = reference_dir().join("../..");
    let mut problems = Vec::new();
    for d in registry.list() {
        for example in &d.examples {
            if !repo_root.join(example).is_file() {
                problems.push(format!("{}: example '{example}' does not exist", d.id));
            }
            let expected_folder = format!("tests/reference/{}/", d.id);
            if !example.starts_with(&expected_folder) {
                problems.push(format!(
                    "{}: example '{example}' is not in {expected_folder}",
                    d.id
                ));
            }
        }
        let dir = reference_dir().join(&d.id);
        for file in case_files(&dir) {
            let name = file
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            let listed = format!("tests/reference/{}/{name}", d.id);
            if !d.examples.contains(&listed) {
                problems.push(format!(
                    "{}: case file '{name}' is not listed in the descriptor's examples",
                    d.id
                ));
            }
        }
    }
    // And no case folder belongs to a calculation that is not registered.
    let ids: Vec<&str> = registry.list().iter().map(|d| d.id.as_str()).collect();
    for entry in std::fs::read_dir(reference_dir()).expect("tests/reference exists") {
        let path = entry.expect("readable entry").path();
        if path.is_dir() {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            if !ids.contains(&name) {
                problems.push(format!(
                    "tests/reference/{name}/ belongs to no registered calculation"
                ));
            }
        }
    }
    assert!(
        problems.is_empty(),
        "{} problem(s):\n{}",
        problems.len(),
        problems.join("\n")
    );
}

/// Phase 2D: ACCEPTANCE_CRITERIA §3 "descriptor complete". `check_descriptor` enforces the structure; this checks that
/// the human-facing text exists, because interfaces, forms and generated documentation are built from it.
#[test]
fn every_descriptor_has_its_text() {
    let registry = registry();
    let mut problems = Vec::new();
    for d in registry.list() {
        let blank = |s: &str| s.trim().is_empty();
        if blank(&d.title) || blank(&d.summary) {
            problems.push(format!("{}: title or summary is empty", d.id));
        }
        if d.references.iter().any(|r| blank(r)) || d.references.is_empty() {
            problems.push(format!("{}: references are missing or blank", d.id));
        }
        if d.assumptions.is_empty() || d.assumptions.iter().any(|a| blank(a)) {
            problems.push(format!("{}: assumptions are missing or blank", d.id));
        }
        if d.examples.is_empty() {
            problems.push(format!("{}: no examples", d.id));
        }
        for i in &d.inputs {
            if blank(&i.description) {
                problems.push(format!("{}: input '{}' has no description", d.id, i.name));
            }
        }
        for o in &d.outputs {
            if blank(&o.description) {
                problems.push(format!("{}: output '{}' has no description", d.id, o.name));
            }
        }
        for rule in &d.validity {
            if blank(&rule.reason) {
                problems.push(format!(
                    "{}: validity rule on '{}' has no reason",
                    d.id, rule.input
                ));
            }
        }
    }
    assert!(
        problems.is_empty(),
        "{} problem(s):\n{}",
        problems.len(),
        problems.join("\n")
    );
}

#[test]
fn every_reference_case_records_how_it_was_verified() {
    // ENGINEERING_STANDARDS §2 item 5: the expected result is independently verified, and the case says how.
    let mut problems = Vec::new();
    for file in case_files(&reference_dir()) {
        let text = std::fs::read_to_string(&file).expect("readable file");
        let value: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
        for field in ["provenance", "verification", "title"] {
            let ok = value[field].as_str().is_some_and(|s| s.trim().len() >= 20);
            if !ok {
                problems.push(format!(
                    "{}: '{field}' is missing or too short",
                    file.display()
                ));
            }
        }
        // Cases written by an AI model must say so in the provenance line (ADR 0003).
        if !value["provenance"]
            .as_str()
            .unwrap_or_default()
            .contains("claude")
        {
            problems.push(format!("{}: provenance names no model", file.display()));
        }
    }
    assert!(
        problems.is_empty(),
        "{} problem(s):\n{}",
        problems.len(),
        problems.join("\n")
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
