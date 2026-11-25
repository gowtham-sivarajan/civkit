// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-26 00:51 IST · last edited 2026-11-26 00:53 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! The JSON surface of the list and choice calculations (Phase 2D regression).
//!
//! Python, MCP, the C ABI and the desktop app all go through `Registry::call_json`: JSON text in, JSON text out.
//! Phase 3's parity tests will compare these exact shapes across interfaces, so they are pinned here first.

use civkit_core::Registry;
use serde_json::{Value, json};

fn call(request: &Value) -> Value {
    let text = Registry::with_builtins().call_json(&request.to_string());
    serde_json::from_str(&text).expect("call_json always returns JSON")
}

fn traverse_request() -> Value {
    json!({
        "id": "surveying.traverse.compass_rule",
        "version": 1,
        "inputs": {
            "bearings": { "value": [0, 90, 180, 270], "unit": "deg" },
            "distances": { "value": [100, 50, 99, 50], "unit": "m" }
        }
    })
}

fn earthwork_request(method: &str) -> Value {
    json!({
        "id": "quantities.earthwork.average_end_area",
        "inputs": {
            "method": { "value": method },
            "chainages": { "value": [0, 50, 75], "unit": "m" },
            "cut_areas": { "value": [0, 0, 10], "unit": "m^2" },
            "fill_areas": { "value": [20, 10, 0], "unit": "m^2" }
        }
    })
}

#[test]
fn a_list_calculation_returns_arrays_with_one_unit_and_structured_trace_items() {
    let v = call(&traverse_request());
    for field in [
        "id",
        "version",
        "civkit_version",
        "outputs",
        "trace",
        "warnings",
        "references",
        "assumptions",
    ] {
        assert!(v.get(field).is_some(), "missing '{field}' in {v}");
    }
    assert!(v.get("error").is_none() && v.get("errors").is_none(), "{v}");
    // A list output is a JSON array of numbers with one unit; a scalar is a bare number.
    let lat = &v["outputs"]["latitudes"];
    assert_eq!(lat["unit"], "m");
    let values = lat["value"].as_array().expect("array");
    assert_eq!(values.len(), 4);
    assert!(values.iter().all(Value::is_number));
    assert!(v["outputs"]["perimeter"]["value"].is_number());
    // A dimensionless output carries no unit key at all.
    assert!(
        v["outputs"]["relative_misclosure"].get("unit").is_none(),
        "{}",
        v["outputs"]["relative_misclosure"]
    );
    // Per-item trace steps carry a structured item; summary steps do not.
    let first = &v["trace"][0];
    assert_eq!(first["item"], json!({ "label": "leg", "index": 0 }));
    assert!(first["step"].as_str().expect("text").starts_with("leg 1: "));
    let summary = v["trace"]
        .as_array()
        .expect("trace")
        .iter()
        .find(|s| s["step"] == "P = Σd")
        .expect("summary step");
    assert!(summary.get("item").is_none() && summary.get("choice").is_none());
}

#[test]
fn a_choice_calculation_records_the_method_as_the_first_trace_step_in_json() {
    for method in ["average_end_area", "pyramid_at_zero_end"] {
        let v = call(&earthwork_request(method));
        assert_eq!(
            v["trace"][0],
            json!({ "step": "method", "choice": method }),
            "{v}"
        );
        assert!(
            v["trace"][0].get("value").is_none(),
            "a choice step has no value"
        );
        assert!(
            v["trace"][1]["item"]["label"] == "segment",
            "{}",
            v["trace"][1]
        );
        assert_eq!(
            v["version"], 1,
            "an omitted version resolves to the latest and is echoed"
        );
    }
    // The two methods give different numbers for the same data: the choice is not decorative.
    let pyramid = call(&earthwork_request("pyramid_at_zero_end"));
    let average = call(&earthwork_request("average_end_area"));
    assert_ne!(
        pyramid["outputs"]["total_cut"]["value"],
        average["outputs"]["total_cut"]["value"]
    );
}

#[test]
fn an_element_error_carries_the_index_in_json_and_scalar_errors_do_not() {
    let mut request = traverse_request();
    request["inputs"]["distances"]["value"] = json!([100, 0, 99, 50]);
    let v = call(&request);
    assert_eq!(v["error"]["code"], "OUT_OF_DOMAIN", "{v}");
    assert_eq!(v["error"]["input"], "distances");
    assert_eq!(v["error"]["index"], 1);
    assert!(
        v["error"]["message"]
            .as_str()
            .expect("message")
            .contains("leg 2")
    );

    // A scalar-input failure has no `index` key at all (API_DESIGN §5: the field is omitted when not used).
    let v = call(&json!({
        "id": "fluids.open_channel.manning_trapezoidal",
        "inputs": { "bottom_width": { "value": -1, "unit": "m" } }
    }));
    assert!(v.get("error").is_some() || v.get("errors").is_some(), "{v}");
    let errors = v
        .get("errors")
        .cloned()
        .unwrap_or_else(|| json!([v["error"].clone()]));
    for e in errors.as_array().expect("errors") {
        assert!(
            e.get("index").is_none(),
            "scalar errors must not carry an index: {e}"
        );
    }
}

#[test]
fn several_list_errors_are_reported_together_with_their_indices() {
    let mut request = traverse_request();
    request["inputs"]["distances"]["value"] = json!([-1, 50, 0, 50]);
    let v = call(&request);
    let errors = v["errors"]
        .as_array()
        .unwrap_or_else(|| panic!("expected several errors: {v}"));
    let indices: Vec<_> = errors.iter().map(|e| e["index"].as_u64()).collect();
    assert_eq!(indices, [Some(0), Some(2)], "{v}");
}

#[test]
fn an_unknown_method_lists_the_allowed_values_in_json() {
    let v = call(&earthwork_request("prismoidal"));
    assert_eq!(v["error"]["code"], "INVALID_VALUE", "{v}");
    assert_eq!(v["error"]["input"], "method");
    let message = v["error"]["message"].as_str().expect("message");
    assert!(
        message.contains("average_end_area") && message.contains("pyramid_at_zero_end"),
        "{message}"
    );
}

#[test]
fn the_same_request_gives_byte_identical_json() {
    // "A calculation ID + version always returns the same result for the same input" (API_DESIGN §1).
    let registry = Registry::with_builtins();
    for request in [traverse_request(), earthwork_request("pyramid_at_zero_end")] {
        let text = request.to_string();
        assert_eq!(registry.call_json(&text), registry.call_json(&text));
    }
}

#[test]
fn descriptors_publish_what_interfaces_need_to_build_forms_and_tools() {
    // Phase 3 generates Python functions, MCP tool schemas and forms from the descriptor JSON.
    let registry = Registry::with_builtins();
    let find = |id: &str| -> Value {
        let d = registry
            .describe(id, None)
            .unwrap_or_else(|| panic!("{id} registered"));
        serde_json::to_value(d).expect("descriptor serialises")
    };

    let earthwork = find("quantities.earthwork.average_end_area");
    let method = &earthwork["inputs"][0];
    assert_eq!(method["name"], "method");
    assert_eq!(method["kind"], "choice");
    assert_eq!(
        method["choices"],
        json!(["average_end_area", "pyramid_at_zero_end"])
    );
    assert_eq!(method["required"], true);
    assert_eq!(earthwork["inputs"][1]["kind"], "list");
    assert_eq!(earthwork["inputs"][1]["group"], "stations");
    assert_eq!(
        earthwork["groups"][0],
        json!({
            "name": "stations",
            "item": "station",
            "min_len": 2,
            "description": earthwork["groups"][0]["description"]
        })
    );
    assert_eq!(earthwork["outputs"][0]["kind"], "list");
    assert_eq!(
        earthwork["outputs"][0]["length"],
        json!({ "group": "stations", "offset": -1, "item": "segment" })
    );

    // A scalar calculation publishes none of the list and choice fields.
    let manning = find("fluids.open_channel.manning_trapezoidal");
    assert!(manning.get("groups").is_none());
    assert!(manning["inputs"][0].get("kind").is_none());
}

#[test]
fn every_registered_calculation_describes_itself_as_valid_json_with_the_contract_fields() {
    let registry = Registry::with_builtins();
    let all = registry.list();
    assert!(all.len() >= 7, "the seven 0.1 calculations are registered");
    for d in all {
        let v = serde_json::to_value(d).expect("descriptor serialises");
        for field in [
            "id",
            "version",
            "title",
            "summary",
            "inputs",
            "outputs",
            "references",
            "assumptions",
            "validity",
            "examples",
        ] {
            assert!(
                v.get(field).is_some(),
                "{}: descriptor lacks '{field}'",
                d.id
            );
        }
        // Dimensional inputs and outputs name their internal unit; the registry checks the rest at registration.
        for spec in v["inputs"]
            .as_array()
            .expect("inputs")
            .iter()
            .chain(v["outputs"].as_array().expect("outputs"))
        {
            assert!(
                spec["name"].is_string() && spec["description"].is_string(),
                "{}: {spec}",
                d.id
            );
        }
    }
}
