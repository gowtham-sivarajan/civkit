// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 11:06 IST · last edited 2026-11-25 11:06 IST

//! **Template calculation.** Copy this file to start a new calculation (see `CONTRIBUTING.md` §2).
//!
//! `math.demo.weighted_mean` is not an engineering calculation and is never registered by
//! [`Registry::with_builtins`](crate::Registry::with_builtins). It exists to show the shape every calculation
//! follows and to exercise the registry end to end (the reference-case harness registers it explicitly).
//!
//! A real calculation differs in three ways:
//! 1. Its references cite a published, openly licensed or public-domain source (D-18), with edition and page.
//! 2. Its inputs have physical dimensions (units arrive in Phase 2).
//! 3. It has at least one reference case from that source in `tests/reference/<id>/`.

use crate::{
    Breach, CalcError, Calculation, Context, Descriptor, InputSpec, Inputs, OutputSpec, Outputs,
    Range, ValidityRule,
};
use civkit_units::Dimension;
use std::sync::LazyLock;

/// Weighted mean of two numbers: `m = w·a + (1 − w)·b`.
#[derive(Debug, Default)]
pub struct WeightedMean;

// 1. The descriptor: everything interfaces need. Built once, on first use.
static DESCRIPTOR: LazyLock<Descriptor> = LazyLock::new(|| {
    Descriptor {
    id: "math.demo.weighted_mean".into(),
    version: 1,
    title: "Weighted mean of two numbers (template)".into(),
    summary: "Demonstration and template calculation: m = w·a + (1 − w)·b. Not an engineering calculation; \
              not registered by default."
        .into(),
    inputs: vec![
        InputSpec {
            name: "a".into(),
            description: "First value".into(),
            dimension: Dimension::DIMENSIONLESS,
            display_unit: None,
            required: true,
            default: None,
            domain: Range::UNBOUNDED,
        },
        InputSpec {
            name: "b".into(),
            description: "Second value".into(),
            dimension: Dimension::DIMENSIONLESS,
            display_unit: None,
            required: true,
            default: None,
            domain: Range::UNBOUNDED,
        },
        InputSpec {
            name: "w".into(),
            description: "Weight given to a; b gets 1 − w".into(),
            dimension: Dimension::DIMENSIONLESS,
            display_unit: None,
            required: false,
            default: Some(0.5),
            // Mathematical domain: outside [0, 1] it is no longer a mean → OUT_OF_DOMAIN error.
            domain: Range::closed(0.0, 1.0),
        },
    ],
    outputs: vec![OutputSpec {
        name: "mean".into(),
        description: "Weighted mean".into(),
        dimension: Dimension::DIMENSIONLESS,
        display_unit: None,
    }],
    references: vec!["Definition of the weighted arithmetic mean (demonstration only).".into()],
    assumptions: vec!["Weights sum to 1.".into()],
    // A stated validity range: inside the domain but worth a warning.
    validity: vec![ValidityRule {
        input: "w".into(),
        range: Range::closed(0.05, 0.95),
        on_breach: Breach::Warning,
        reason: "a weight this extreme usually means one value was meant to be ignored".into(),
    }],
    examples: vec!["tests/reference/math.demo.weighted_mean/demo-01.json".into()],
}
});

impl Calculation for WeightedMean {
    fn descriptor(&self) -> &Descriptor {
        &DESCRIPTOR
    }

    // 2. The computation. Inputs are already validated and in SI; record every step in the trace.
    fn compute(&self, inputs: &Inputs, ctx: &mut Context) -> Result<Outputs, CalcError> {
        let a = inputs.require("a")?;
        let b = inputs.require("b")?;
        let w = inputs.require("w")?; // optional, but always present because it has a default

        let mean = w * a + (1.0 - w) * b;
        ctx.step("m = w·a + (1 − w)·b", mean, None);

        Ok(Outputs::new().with("mean", mean))
    }
}

// 3. Unit tests: edge cases and invalid inputs. Published reference cases go in tests/reference/.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ErrorCode, InputValue, Registry, Request, WarningCode};

    fn registry() -> Registry {
        let mut r = Registry::new();
        r.register(Box::new(WeightedMean))
            .expect("valid descriptor");
        r
    }

    fn request(inputs: &[(&str, f64)]) -> Request {
        Request {
            id: "math.demo.weighted_mean".into(),
            version: None,
            inputs: inputs
                .iter()
                .map(|(k, v)| ((*k).to_owned(), InputValue::number(*v)))
                .collect(),
            output_units: Default::default(),
        }
    }

    #[test]
    fn default_weight_gives_plain_mean() {
        let r = registry()
            .call(&request(&[("a", 2.0), ("b", 4.0)]))
            .expect("ok");
        assert!((r.outputs["mean"].value - 3.0).abs() < 1e-12);
        assert_eq!(r.version, 1);
        assert!(r.warnings.is_empty());
        assert_eq!(r.trace.len(), 1);
    }

    #[test]
    fn weight_at_domain_limits_is_allowed_but_warns() {
        for (w, expected) in [(0.0, 4.0), (1.0, 2.0)] {
            let r = registry()
                .call(&request(&[("a", 2.0), ("b", 4.0), ("w", w)]))
                .expect("ok");
            assert!((r.outputs["mean"].value - expected).abs() < 1e-12);
            assert_eq!(r.warnings.len(), 1);
            assert_eq!(r.warnings[0].code, WarningCode::OutsideValidityRange);
        }
    }

    #[test]
    fn weight_outside_domain_is_an_error() {
        let e = registry()
            .call(&request(&[("a", 2.0), ("b", 4.0), ("w", 1.5)]))
            .expect_err("out of domain");
        assert_eq!(e.0[0].code, ErrorCode::OutOfDomain);
        assert_eq!(e.0[0].input.as_deref(), Some("w"));
    }

    #[test]
    fn missing_required_inputs_are_all_reported() {
        let e = registry().call(&request(&[])).expect_err("missing");
        let codes: Vec<_> = e.0.iter().map(|x| (x.code, x.input.clone())).collect();
        assert_eq!(
            codes,
            vec![
                (ErrorCode::MissingInput, Some("a".into())),
                (ErrorCode::MissingInput, Some("b".into()))
            ]
        );
    }
}
