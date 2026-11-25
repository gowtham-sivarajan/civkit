// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 22:03 IST · last edited 2026-11-25 23:47 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! List and choice inputs (D-23, ADR 0011; planning/CONTRACT_LISTS_PROPOSAL.md §3).
//!
//! `math.demo.spacing` is a test-only calculation over a group of points: gaps between neighbours (offset −1),
//! weighted positions (offset 0) and running totals from 0, closed by the total (offset +1). A `fault` makes it break one
//! contract rule at a time, to show that the registry catches each.

use civkit_core::{
    CalcError, Calculation, Context, Descriptor, ErrorCode, Group, InputKind, InputSpec,
    InputValue, Inputs, ListLength, MAX_LIST_LEN, OutputKind, OutputSpec, Outputs, Range, Registry,
    Request, TraceItem, TraceStep, check_descriptor,
};
use civkit_units::Dimension;
use serde_json::json;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Fault {
    None,
    ShortOutput,
    NoChoiceStep,
    TwoChoiceSteps,
    ChoiceAfterNumeric,
    WrongChoiceValue,
    UndeclaredChoiceStep,
    UnknownItemLabel,
    ItemIndexTooLarge,
    ItemTextMismatch,
    NonFiniteElement,
    ScalarForList,
}

struct Spacing {
    descriptor: Descriptor,
    fault: Fault,
}

fn list_input(name: &str, dimension: Dimension, si: Option<&str>, domain: Range) -> InputSpec {
    InputSpec {
        name: name.into(),
        description: format!("{name}, one per point"),
        dimension,
        si_unit: si.map(Into::into),
        display_unit: si.map(Into::into),
        required: true,
        domain,
        kind: InputKind::List,
        group: Some("points".into()),
        ..InputSpec::default()
    }
}

fn list_output(name: &str, offset: i8, item: &str) -> OutputSpec {
    OutputSpec {
        name: name.into(),
        description: format!("{name}; entry k pairs with point k (offset {offset})"),
        dimension: Dimension::LENGTH,
        si_unit: Some("m".into()),
        display_unit: Some("m".into()),
        kind: OutputKind::List,
        length: Some(ListLength {
            group: "points".into(),
            offset,
            item: item.into(),
        }),
    }
}

fn spacing_descriptor() -> Descriptor {
    Descriptor {
        id: "math.demo.spacing".into(),
        version: 1,
        title: "Spacing of points along a line (test only)".into(),
        summary: "Gaps, weighted positions and cumulative marks.".into(),
        inputs: vec![
            list_input("positions", Dimension::LENGTH, Some("m"), Range::UNBOUNDED),
            list_input(
                "weights",
                Dimension::DIMENSIONLESS,
                None,
                Range::at_least(0.0),
            ),
            InputSpec {
                name: "method".into(),
                description: "How the gaps are summarised".into(),
                required: true,
                kind: InputKind::Choice,
                choices: vec!["sum".into(), "mean".into()],
                ..InputSpec::default()
            },
        ],
        groups: vec![Group {
            name: "points".into(),
            item: "point".into(),
            min_len: 2,
            description: "Points in order along the line".into(),
        }],
        outputs: vec![
            list_output("gaps", -1, "gap"),
            list_output("half_gaps", -1, "gap"),
            list_output("weighted", 0, "point"),
            list_output("marks", 1, "mark"),
            OutputSpec {
                name: "summary".into(),
                description: "Sum or mean of the gaps".into(),
                dimension: Dimension::LENGTH,
                si_unit: Some("m".into()),
                display_unit: Some("m".into()),
                ..OutputSpec::default()
            },
        ],
        references: vec!["Test only.".into()],
        ..Descriptor::default()
    }
}

impl Calculation for Spacing {
    fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }

    fn compute(&self, inputs: &Inputs, ctx: &mut Context) -> Result<Outputs, CalcError> {
        let x = inputs.list("positions")?;
        let w = inputs.list("weights")?;
        let method = inputs.choice("method")?;
        let f = self.fault;

        match f {
            Fault::NoChoiceStep => {}
            Fault::ChoiceAfterNumeric => ctx.step("before", 1.0, None),
            Fault::WrongChoiceValue => ctx.choice("method", "mean_of_something_else"),
            Fault::UndeclaredChoiceStep => {
                ctx.choice("method", method);
                ctx.choice("colour", "red");
            }
            _ => ctx.choice("method", method),
        }
        if f == Fault::TwoChoiceSteps {
            ctx.choice("method", method);
        }
        if f == Fault::ChoiceAfterNumeric {
            ctx.choice("method", method);
        }

        let gaps: Vec<f64> = x.windows(2).map(|p| p[1] - p[0]).collect();
        for (k, g) in gaps.iter().enumerate() {
            ctx.step_item("gap", k, "g = x(k+1) − x(k)", *g, Some("m"));
        }
        match f {
            Fault::UnknownItemLabel => ctx.step_item("leg", 0, "x", 1.0, Some("m")),
            Fault::ItemIndexTooLarge => ctx.step_item("gap", gaps.len(), "x", 1.0, Some("m")),
            Fault::ItemTextMismatch => ctx.push(TraceStep {
                step: "gap 2: g".into(),
                value: Some(1.0),
                unit: Some("m".into()),
                item: Some(TraceItem {
                    label: "gap".into(),
                    index: 0,
                }),
                ..TraceStep::default()
            }),
            _ => {}
        }
        let summary = match method {
            "sum" => gaps.iter().sum(),
            _ => gaps.iter().sum::<f64>() / gaps.len() as f64,
        };
        ctx.step("summary", summary, Some("m"));

        let mut gaps_out = gaps.clone();
        if f == Fault::ShortOutput {
            gaps_out.pop();
        }
        if f == Fault::NonFiniteElement {
            gaps_out[0] = f64::NAN;
        }
        let weighted: Vec<f64> = x.iter().zip(w).map(|(a, b)| a * b).collect();
        // Running totals starting at 0 (one per point), closed by the total: points + 1 entries.
        let mut marks = vec![0.0];
        marks.extend(gaps.iter().scan(0.0, |acc, g| {
            *acc += g;
            Some(*acc)
        }));
        marks.push(gaps.iter().sum());
        let out = Outputs::new()
            .with_list("half_gaps", gaps.iter().map(|g| g / 2.0).collect())
            .with_list("weighted", weighted)
            .with_list("marks", marks)
            .with("summary", summary);
        Ok(if f == Fault::ScalarForList {
            out.with("gaps", 1.0)
        } else {
            out.with_list("gaps", gaps_out)
        })
    }
}

fn registry(fault: Fault) -> Registry {
    let mut r = Registry::new();
    r.register(Box::new(Spacing {
        descriptor: spacing_descriptor(),
        fault,
    }))
    .expect("valid descriptor");
    r
}

fn request(inputs: serde_json::Value) -> Request {
    serde_json::from_value(json!({ "id": "math.demo.spacing", "inputs": inputs })).expect("request")
}

fn good_inputs() -> serde_json::Value {
    json!({
        "positions": { "value": [0, 2, 5, 9], "unit": "m" },
        "weights": { "value": [1, 1, 0.5, 2] },
        "method": { "value": "sum" }
    })
}

fn list(r: &civkit_core::Response, name: &str) -> Vec<f64> {
    r.outputs[name]
        .value
        .as_list()
        .expect("list output")
        .to_vec()
}

// --- Successful calls -----------------------------------------------------------------------------

#[test]
fn list_outputs_follow_their_index_sets() {
    let r = registry(Fault::None)
        .call(&request(good_inputs()))
        .expect("ok");
    assert_eq!(list(&r, "gaps"), vec![2.0, 3.0, 4.0]); // 4 points - 1
    assert_eq!(list(&r, "half_gaps"), vec![1.0, 1.5, 2.0]); // shares "gap" with gaps
    assert_eq!(list(&r, "weighted"), vec![0.0, 2.0, 2.5, 18.0]); // 4 points + 0
    assert_eq!(list(&r, "marks"), vec![0.0, 2.0, 5.0, 9.0, 9.0]); // 4 points + 1
    assert_eq!(r.outputs["summary"].value.as_scalar(), Some(9.0));
}

#[test]
fn choice_step_comes_first_and_items_are_structured() {
    let r = registry(Fault::None)
        .call(&request(good_inputs()))
        .expect("ok");
    assert_eq!(r.trace[0].step, "method");
    assert_eq!(r.trace[0].choice.as_deref(), Some("sum"));
    assert_eq!(r.trace[0].value, None);
    let gap2 = &r.trace[2];
    assert_eq!(gap2.step, "gap 2: g = x(k+1) − x(k)");
    assert_eq!(
        gap2.item,
        Some(TraceItem {
            label: "gap".into(),
            index: 1
        })
    );
    // JSON shape of the two step forms.
    let json = serde_json::to_value(&r.trace[..3]).expect("json");
    assert_eq!(json[0], json!({ "step": "method", "choice": "sum" }));
    assert_eq!(
        json[2],
        json!({ "step": "gap 2: g = x(k+1) − x(k)", "value": 3.0, "unit": "m", "item": { "label": "gap", "index": 1 } })
    );
}

#[test]
fn output_units_apply_to_every_element() {
    let mut req = request(good_inputs());
    req.output_units.insert("gaps".into(), "mm".into());
    let r = registry(Fault::None).call(&req).expect("ok");
    assert_eq!(list(&r, "gaps"), vec![2000.0, 3000.0, 4000.0]);
    assert_eq!(r.outputs["gaps"].unit.as_deref(), Some("mm"));
    let json = serde_json::to_value(&r.outputs["gaps"]).expect("json");
    assert_eq!(
        json,
        json!({ "value": [2000.0, 3000.0, 4000.0], "unit": "mm" })
    );
}

#[test]
fn list_units_convert_once_for_the_whole_list() {
    let mut inputs = good_inputs();
    inputs["positions"] = json!({ "value": [0, 2, 5, 9], "unit": "ft" });
    let r = registry(Fault::None).call(&request(inputs)).expect("ok");
    let g = list(&r, "gaps");
    assert!((g[0] - 0.6096).abs() < 1e-12);
}

// --- Request validation ---------------------------------------------------------------------------

fn errors(inputs: serde_json::Value) -> Vec<CalcError> {
    registry(Fault::None)
        .call(&request(inputs))
        .expect_err("should fail")
        .0
}

#[test]
fn element_errors_carry_index_and_item_number() {
    let mut inputs = good_inputs();
    inputs["weights"] = json!({ "value": [1, -1, 0.5, -2] });
    let e = errors(inputs);
    assert_eq!(e.len(), 2, "{e:?}");
    assert_eq!(e[0].code, ErrorCode::OutOfDomain);
    assert_eq!(e[0].input.as_deref(), Some("weights"));
    assert_eq!(e[0].index, Some(1));
    assert!(e[0].message.starts_with("point 2: "), "{}", e[0].message);
    assert_eq!(e[1].index, Some(3));
    let json = serde_json::to_value(&e[0]).expect("json");
    assert_eq!(json["index"], json!(1));
}

#[test]
fn element_type_and_shape_errors() {
    let mut inputs = good_inputs();
    inputs["positions"] = json!({ "value": [0, "two", 5, 9], "unit": "m" });
    let e = errors(inputs);
    assert_eq!((e[0].code, e[0].index), (ErrorCode::InvalidValue, Some(1)));

    let mut inputs = good_inputs();
    inputs["positions"] = json!({ "value": 5, "unit": "m" });
    let e = errors(inputs);
    assert_eq!((e[0].code, e[0].index), (ErrorCode::InvalidValue, None));
    assert!(e[0].message.contains("list"), "{}", e[0].message);

    let mut inputs = good_inputs();
    inputs["positions"] = json!({ "value": [[1, 2, 3], [1, 2, 4]], "unit": "dms" });
    assert_eq!(errors(inputs)[0].code, ErrorCode::InvalidValue);

    let mut inputs = good_inputs();
    inputs["positions"] = json!({ "value": [0, 2, 5, 9] });
    assert_eq!(errors(inputs)[0].code, ErrorCode::MissingUnit);
}

#[test]
fn group_lengths_must_match() {
    let mut inputs = good_inputs();
    inputs["weights"] = json!({ "value": [1, 1, 1] });
    let e = errors(inputs);
    assert_eq!(e.len(), 1);
    assert_eq!(e[0].code, ErrorCode::InvalidValue);
    assert_eq!(e[0].input.as_deref(), Some("weights"));
    assert!(
        e[0].message.contains("'points'")
            && e[0].message.contains('4')
            && e[0].message.contains('3'),
        "{}",
        e[0].message
    );
}

#[test]
fn minimum_and_maximum_lengths() {
    let mut inputs = good_inputs();
    inputs["positions"] = json!({ "value": [0], "unit": "m" });
    inputs["weights"] = json!({ "value": [1] });
    let e = errors(inputs);
    assert_eq!(e[0].code, ErrorCode::InvalidValue);
    assert!(e[0].message.contains('2'), "{}", e[0].message);

    // A +1 output exists, so the group's maximum is MAX_LIST_LEN - 1.
    let at_max = MAX_LIST_LEN - 1;
    let ok = json!({
        "positions": { "value": vec![1.0; at_max], "unit": "m" },
        "weights": { "value": vec![1.0; at_max] },
        "method": { "value": "sum" }
    });
    let r = registry(Fault::None)
        .call(&request(ok))
        .expect("at the limit");
    assert_eq!(list(&r, "marks").len(), MAX_LIST_LEN);
    let over = json!({
        "positions": { "value": vec![1.0; at_max + 1], "unit": "m" },
        "weights": { "value": vec![1.0; at_max + 1] },
        "method": { "value": "sum" }
    });
    let e = errors(over);
    assert_eq!(e[0].code, ErrorCode::InvalidValue);
    assert!(
        e[0].message.contains(&at_max.to_string()),
        "{}",
        e[0].message
    );
}

#[test]
fn choices_are_checked_and_required() {
    let mut inputs = good_inputs();
    inputs["method"] = json!({ "value": "median" });
    let e = errors(inputs);
    assert_eq!(
        (e[0].code, e[0].input.as_deref()),
        (ErrorCode::InvalidValue, Some("method"))
    );
    assert!(
        e[0].message.contains("sum") && e[0].message.contains("mean"),
        "{}",
        e[0].message
    );

    let mut inputs = good_inputs();
    inputs["method"] = json!({ "value": 1 });
    assert_eq!(errors(inputs)[0].code, ErrorCode::InvalidValue);

    let mut inputs = good_inputs();
    inputs["method"] = json!({ "value": "sum", "unit": "m" });
    assert_eq!(errors(inputs)[0].code, ErrorCode::InvalidValue);

    let mut inputs = good_inputs();
    inputs.as_object_mut().expect("object").remove("method");
    assert_eq!(errors(inputs)[0].code, ErrorCode::MissingInput);
}

// --- Violations by the calculation are INTERNAL ---------------------------------------------------

#[test]
fn every_contract_violation_by_compute_is_internal() {
    for fault in [
        Fault::ShortOutput,
        Fault::NoChoiceStep,
        Fault::TwoChoiceSteps,
        Fault::ChoiceAfterNumeric,
        Fault::WrongChoiceValue,
        Fault::UndeclaredChoiceStep,
        Fault::UnknownItemLabel,
        Fault::ItemIndexTooLarge,
        Fault::ItemTextMismatch,
        Fault::NonFiniteElement,
        Fault::ScalarForList,
    ] {
        let e = registry(fault)
            .call(&request(good_inputs()))
            .expect_err("fault must be caught")
            .0;
        assert_eq!(e[0].code, ErrorCode::Internal, "{fault:?}: {e:?}");
    }
}

// --- Descriptor checks ----------------------------------------------------------------------------

fn rejected(change: impl FnOnce(&mut Descriptor)) -> String {
    let mut d = spacing_descriptor();
    change(&mut d);
    check_descriptor(&d).expect_err("should be rejected").0
}

#[test]
fn well_formed_list_descriptor_is_accepted() {
    check_descriptor(&spacing_descriptor()).expect("valid");
}

#[test]
fn malformed_list_descriptors_are_rejected() {
    let msgs = [
        rejected(|d| d.inputs[0].group = Some("nowhere".into())),
        rejected(|d| d.inputs[0].group = None),
        rejected(|d| d.inputs[0].required = false),
        rejected(|d| d.groups[0].min_len = 0),
        rejected(|d| d.groups[0].item = " ".into()),
        rejected(|d| {
            d.groups.push(Group {
                name: "unused".into(),
                item: "thing".into(),
                min_len: 1,
                description: "no list uses it".into(),
            });
        }),
        rejected(|d| d.groups.push(d.groups[0].clone())),
        rejected(|d| d.inputs[2].choices.clear()),
        rejected(|d| d.inputs[2].choices.push("sum".into())),
        rejected(|d| d.inputs[2].choices.push("Not Snake".into())),
        rejected(|d| d.inputs[2].required = false),
        rejected(|d| d.inputs[2].dimension = Dimension::LENGTH),
        rejected(|d| d.inputs[1].choices = vec!["x".into()]),
        rejected(|d| d.outputs[0].length = None),
        rejected(|d| d.outputs[0].length.as_mut().expect("len").group = "nowhere".into()),
        rejected(|d| d.outputs[0].length.as_mut().expect("len").offset = 2),
        rejected(|d| d.outputs[0].length.as_mut().expect("len").item = String::new()),
        rejected(|d| d.outputs[0].description = String::new()),
        // "gap" already names (points, -1); it cannot also name (points, 0).
        rejected(|d| d.outputs[2].length.as_mut().expect("len").item = "gap".into()),
        // The group's own label "point" names (points, 0); an offset +1 output cannot reuse it.
        rejected(|d| d.outputs[3].length.as_mut().expect("len").item = "point".into()),
        // min_len 1 with offset -1 would allow an empty output.
        rejected(|d| d.groups[0].min_len = 1),
        rejected(|d| {
            d.outputs[4].length = Some(ListLength {
                group: "points".into(),
                offset: 0,
                item: "point".into(),
            })
        }),
    ];
    for m in msgs {
        assert!(!m.is_empty());
    }
}

// --- Compatibility: scalar calculations keep their JSON shape -------------------------------------

#[test]
fn scalar_calculations_keep_their_exact_json_shape() {
    let registry = Registry::with_builtins();
    let new_keys = [
        "\"kind\"",
        "\"group\"",
        "\"groups\"",
        "\"choices\"",
        "\"length\"",
        "\"item\"",
        "\"choice\"",
        "\"index\"",
    ];
    // Descriptors are checked field by field: a substring search would also match the `length` exponent that
    // every dimension serialises, or the word "kind" inside a description.
    for d in registry.list() {
        let v = serde_json::to_value(d).expect("descriptor json");
        assert!(v.get("groups").is_none(), "{}: descriptor has groups", d.id);
        for (list, keys) in [
            ("inputs", ["kind", "group", "choices"].as_slice()),
            ("outputs", ["kind", "length"].as_slice()),
        ] {
            for spec in v[list].as_array().expect("spec list") {
                for key in keys {
                    assert!(
                        spec.get(key).is_none(),
                        "{}: {list} entry {} has {key}",
                        d.id,
                        spec["name"]
                    );
                }
            }
        }
    }
    let response = registry.call_json(
        r#"{"id":"fluids.open_channel.manning_trapezoidal","inputs":{"bottom_width":{"value":2,"unit":"m"},"side_slope":{"value":1},"depth":{"value":1,"unit":"m"},"manning_n":{"value":0.013},"bed_slope":{"value":0.002}}}"#,
    );
    let error = registry.call_json(
        r#"{"id":"fluids.open_channel.manning_trapezoidal","inputs":{"bottom_width":{"value":-2,"unit":"m"}}}"#,
    );
    for json in [&response, &error] {
        for key in new_keys {
            assert!(!json.contains(key), "{key} in {json}");
        }
    }
    // A scalar output is still a bare number.
    let v: serde_json::Value = serde_json::from_str(&response).expect("json");
    assert!(v["outputs"]["area"]["value"].is_number());
    assert!(v["trace"][0]["value"].is_number());
}

#[test]
fn input_value_still_accepts_scalars() {
    let v: InputValue =
        serde_json::from_value(json!({ "value": 2.5, "unit": "m" })).expect("scalar");
    assert_eq!(v.unit.as_deref(), Some("m"));
}
