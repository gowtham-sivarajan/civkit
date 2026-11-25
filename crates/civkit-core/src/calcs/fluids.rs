// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 19:15 IST · last edited 2026-11-25 23:47 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! Fluids calculations for 0.1 (D-20, ADR 0009).
//!
//! `fluids.open_channel.manning_trapezoidal` — Manning's equation for a trapezoidal channel with a known
//! depth. D-20 scope: area, wetted perimeter, hydraulic radius, velocity and discharge. Not in 0.1: solving
//! for normal depth, circular pipes, other sections.

use crate::{
    CalcError, Calculation, Context, Descriptor, ErrorCode, InputSpec, Inputs, OutputSpec, Outputs,
    Range,
};
use civkit_units::Dimension;
use std::sync::LazyLock;

const LENGTH: Dimension = Dimension::LENGTH;

fn input(
    name: &str,
    description: &str,
    dimension: Dimension,
    si_unit: Option<&str>,
    domain: Range,
) -> InputSpec {
    InputSpec {
        name: name.into(),
        description: description.into(),
        dimension,
        si_unit: si_unit.map(Into::into),
        display_unit: si_unit.map(Into::into),
        required: true,
        default: None,
        accepts_bearing: false,
        domain,
        ..Default::default() // scalar input: no list, group or choices (D-23)
    }
}

fn output(name: &str, description: &str, dimension: Dimension, si_unit: &str) -> OutputSpec {
    OutputSpec {
        name: name.into(),
        description: description.into(),
        dimension,
        si_unit: Some(si_unit.into()),
        display_unit: Some(si_unit.into()),
        ..Default::default() // scalar output (D-23)
    }
}

/// Manning's equation, trapezoidal channel with known depth (SI form).
#[derive(Debug, Default)]
pub struct ManningTrapezoidal;

static MANNING: LazyLock<Descriptor> = LazyLock::new(|| {
    Descriptor {
    id: "fluids.open_channel.manning_trapezoidal".into(),
    version: 1,
    title: "Manning's equation, trapezoidal channel with known depth".into(),
    summary: "Flow area, wetted perimeter, hydraulic radius, mean velocity and discharge of uniform flow in a \
              trapezoidal channel at a given depth: A = (B + z·y)·y, P = B + 2y·√(1 + z²), R = A/P, \
              V = (1/n)·R^(2/3)·S^(1/2), Q = V·A. A rectangle (z = 0) and a triangle (B = 0) are special cases."
        .into(),
    inputs: vec![
        input(
            "bottom_width",
            "Bottom width B",
            LENGTH,
            Some("m"),
            Range::at_least(0.0),
        ),
        input(
            "side_slope",
            "Side slope z, horizontal per 1 vertical (1V:zH), the same on both sides",
            Dimension::DIMENSIONLESS,
            None,
            Range::at_least(0.0),
        ),
        input(
            "depth",
            "Flow depth y, measured vertically from the bed",
            LENGTH,
            Some("m"),
            Range::greater_than(0.0),
        ),
        input(
            "manning_n",
            "Manning roughness coefficient n for the SI form of the equation",
            Dimension::DIMENSIONLESS,
            None,
            Range::greater_than(0.0),
        ),
        input(
            "bed_slope",
            "Longitudinal bed slope S (energy slope for uniform flow), e.g. 0.002 or 0.2 %",
            Dimension::DIMENSIONLESS,
            None,
            Range::greater_than(0.0),
        ),
    ],
    outputs: vec![
        output("area", "Flow area A", Dimension::AREA, "m^2"),
        output("wetted_perimeter", "Wetted perimeter P", LENGTH, "m"),
        output("hydraulic_radius", "Hydraulic radius R = A/P", LENGTH, "m"),
        output("velocity", "Mean velocity V", Dimension::new(1, 0, -1, 0), "m/s"),
        output("discharge", "Discharge Q = V·A", Dimension::new(3, 0, -1, 0), "m^3/s"),
    ],
    references: vec![
        "FHWA, Introduction to Highway Hydraulics, HDS-4 (1997, metric), chapter 4 (open channel flow), \
         Manning's equation; Examples 4.1 and 4.2 (§4.3). Public domain."
            .into(),
    ],
    assumptions: vec![
        "Steady, uniform flow: the bed slope equals the energy slope.".into(),
        "Prismatic trapezoidal section with equal side slopes; the depth is given, not solved for.".into(),
        "SI form of Manning's equation (k = 1). n is the SI coefficient, entered as a plain number; \
         inputs in other length units are converted to metres first, so the result does not depend on them."
            .into(),
    ],
    validity: Vec::new(),
    examples: vec![
        "tests/reference/fluids.open_channel.manning_trapezoidal/hds4-ex4-1.json".into(),
        "tests/reference/fluids.open_channel.manning_trapezoidal/hds4-ex4-2.json".into(),
    ],
    ..Default::default() // no list groups (D-23)
}
});

impl Calculation for ManningTrapezoidal {
    fn descriptor(&self) -> &Descriptor {
        &MANNING
    }

    fn compute(&self, inputs: &Inputs, ctx: &mut Context) -> Result<Outputs, CalcError> {
        let b = inputs.require("bottom_width")?;
        let z = inputs.require("side_slope")?;
        let y = inputs.require("depth")?;
        let n = inputs.require("manning_n")?;
        let s = inputs.require("bed_slope")?;
        if b == 0.0 && z == 0.0 {
            return Err(CalcError::for_input(
                ErrorCode::OutOfDomain,
                "bottom_width",
                "a bottom width of 0 with vertical sides (side_slope 0) is not a channel",
            ));
        }

        let area = (b + z * y) * y;
        ctx.step("A = (B + z·y)·y", area, Some("m^2"));
        let perimeter = b + 2.0 * y * (1.0 + z * z).sqrt();
        ctx.step("P = B + 2·y·√(1 + z²)", perimeter, Some("m"));
        let radius = area / perimeter;
        ctx.step("R = A / P", radius, Some("m"));
        let velocity = radius.powf(2.0 / 3.0) * s.sqrt() / n;
        ctx.step("V = (1/n)·R^(2/3)·S^(1/2)", velocity, Some("m/s"));
        let discharge = velocity * area;
        ctx.step("Q = V·A", discharge, Some("m^3/s"));

        Ok(Outputs::new()
            .with("area", area)
            .with("wetted_perimeter", perimeter)
            .with("hydraulic_radius", radius)
            .with("velocity", velocity)
            .with("discharge", discharge))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{InputValue, Registry, Request};

    fn call(inputs: &[(&str, f64, Option<&str>)]) -> Result<crate::Response, crate::ErrorEnvelope> {
        let mut r = Registry::new();
        r.register(Box::new(ManningTrapezoidal))
            .expect("valid descriptor");
        r.call(&Request {
            id: "fluids.open_channel.manning_trapezoidal".into(),
            version: Some(1),
            inputs: inputs
                .iter()
                .map(|(k, v, u)| {
                    let value = InputValue {
                        unit: u.map(str::to_owned),
                        ..InputValue::number(*v)
                    };
                    ((*k).to_owned(), value)
                })
                .collect(),
            output_units: Default::default(),
        })
    }

    fn channel(b: f64, z: f64, y: f64) -> Vec<(&'static str, f64, Option<&'static str>)> {
        vec![
            ("bottom_width", b, Some("m")),
            ("side_slope", z, None),
            ("depth", y, Some("m")),
            ("manning_n", 0.013, None),
            ("bed_slope", 0.002, None),
        ]
    }

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() <= 1e-12 * b.abs(), "{a} vs {b}");
    }

    /// A scalar output's value (this calculation has no list outputs).
    fn out(r: &crate::Response, name: &str) -> f64 {
        r.outputs[name].value.as_scalar().expect("scalar output")
    }

    #[test]
    fn rectangle_and_triangle_are_special_cases() {
        // Rectangle 2 m x 1 m: A = 2, P = 4, R = 0.5.
        let r = call(&channel(2.0, 0.0, 1.0)).expect("rectangle");
        close(out(&r, "area"), 2.0);
        close(out(&r, "wetted_perimeter"), 4.0);
        close(out(&r, "hydraulic_radius"), 0.5);
        // Triangle, z = 1, y = 1: A = 1, P = 2·√2.
        let r = call(&channel(0.0, 1.0, 1.0)).expect("triangle");
        close(out(&r, "area"), 1.0);
        close(out(&r, "wetted_perimeter"), 2.0 * 2.0_f64.sqrt());
        // V and Q follow from R by the SI formula.
        let v = 0.5_f64.powf(2.0 / 3.0) * 0.002_f64.sqrt() / 0.013;
        let r = call(&channel(2.0, 0.0, 1.0)).expect("rectangle");
        close(out(&r, "velocity"), v);
        close(out(&r, "discharge"), 2.0 * v);
        assert_eq!(r.trace.len(), 5);
    }

    #[test]
    fn no_channel_is_out_of_domain() {
        let e = call(&channel(0.0, 0.0, 1.0)).expect_err("no channel");
        assert_eq!(e.0[0].code, ErrorCode::OutOfDomain);
        assert_eq!(e.0[0].input.as_deref(), Some("bottom_width"));
    }

    #[test]
    fn invalid_inputs_are_reported_together() {
        let mut inputs = channel(-1.0, 2.0, 0.0);
        inputs[3].1 = 0.0; // n
        let e = call(&inputs).expect_err("invalid");
        let names: Vec<_> = e.0.iter().map(|x| (x.code, x.input.clone())).collect();
        assert_eq!(
            names,
            vec![
                (ErrorCode::OutOfDomain, Some("bottom_width".into())),
                (ErrorCode::OutOfDomain, Some("depth".into())),
                (ErrorCode::OutOfDomain, Some("manning_n".into())),
            ]
        );
    }

    #[test]
    fn units_of_the_inputs_do_not_change_the_result() {
        // HDS-4 Example 4.2 entered in feet, slope as a percentage.
        let f = 0.3048;
        let feet = call(&[
            ("bottom_width", 1.5 / f, Some("ft")),
            ("side_slope", 2.0, None),
            ("depth", 0.6 / f, Some("ft")),
            ("manning_n", 0.013, None),
            ("bed_slope", 0.2, Some("%")),
        ])
        .expect("feet");
        let metres = call(&[
            ("bottom_width", 1.5, Some("m")),
            ("side_slope", 2.0, None),
            ("depth", 0.6, Some("m")),
            ("manning_n", 0.013, None),
            ("bed_slope", 0.002, None),
        ])
        .expect("metres");
        for name in ["area", "wetted_perimeter", "velocity", "discharge"] {
            close(out(&feet, name), out(&metres, name));
        }
    }

    #[test]
    fn a_depth_without_a_unit_is_refused() {
        let mut inputs = channel(2.0, 0.0, 1.0);
        inputs[2].2 = None;
        let e = call(&inputs).expect_err("bare number");
        assert_eq!(e.0[0].code, ErrorCode::MissingUnit);
    }
}
