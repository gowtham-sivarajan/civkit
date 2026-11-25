// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 19:15 IST · last edited 2026-11-25 23:47 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! Structural calculations for 0.1 (D-20, ADR 0009).
//!
//! `structural.beam.simply_supported` — a prismatic, simply supported beam carrying a full-span uniformly
//! distributed load and one point load. D-20 scope: support reactions, deflection at a requested point and the
//! end slope θ_A (verified by C6–C8). Maximum shear and maximum moment are **not** in this version: they wait for
//! an openly licensed worked example that verifies them numerically (D-20, condition 1).

use crate::{
    CalcError, Calculation, Context, Descriptor, ErrorCode, InputSpec, Inputs, OutputSpec, Outputs,
    Range,
};
use civkit_units::Dimension;
use std::sync::LazyLock;

const LENGTH: Dimension = Dimension::LENGTH;
const FORCE: Dimension = Dimension::new(1, 1, -2, 0);
const FORCE_PER_LENGTH: Dimension = Dimension::new(0, 1, -2, 0);
const PRESSURE: Dimension = Dimension::new(-1, 1, -2, 0);
const SECOND_MOMENT: Dimension = Dimension::new(4, 0, 0, 0);

struct In<'a> {
    name: &'a str,
    description: &'a str,
    dimension: Dimension,
    si: &'a str,
    display: &'a str,
    required: bool,
    default: Option<f64>,
    domain: Range,
}

fn input(i: In<'_>) -> InputSpec {
    InputSpec {
        name: i.name.into(),
        description: i.description.into(),
        dimension: i.dimension,
        si_unit: Some(i.si.into()),
        display_unit: Some(i.display.into()),
        required: i.required,
        default: i.default,
        accepts_bearing: false,
        domain: i.domain,
        ..Default::default() // scalar input: no list, group or choices (D-23)
    }
}

fn output(
    name: &str,
    description: &str,
    dimension: Dimension,
    si: &str,
    display: &str,
) -> OutputSpec {
    OutputSpec {
        name: name.into(),
        description: description.into(),
        dimension,
        si_unit: Some(si.into()),
        display_unit: Some(display.into()),
        ..Default::default() // scalar output (D-23)
    }
}

/// Simply supported beam: UDL over the full span plus one point load.
#[derive(Debug, Default)]
pub struct SimplySupportedBeam;

static BEAM: LazyLock<Descriptor> = LazyLock::new(|| {
    Descriptor {
    id: "structural.beam.simply_supported".into(),
    version: 1,
    title: "Simply supported beam: reactions, deflection and end slope".into(),
    summary: "A prismatic beam on a pin and a roller, span L, carrying a uniformly distributed load w over the \
              full span and a point load P at distance a from the left support A. Returns both support reactions, \
              the deflection at a requested distance x from A, and the slope at A. Loads act downward; deflection \
              and slope are measured upward and counter-clockwise, so a downward load gives negative values, as in \
              the cited sources. Maximum shear and moment are not part of this version (D-20)."
        .into(),
    inputs: vec![
        input(In {
            name: "span",
            description: "Span L between the supports",
            dimension: LENGTH,
            si: "m",
            display: "m",
            required: true,
            default: None,
            domain: Range::greater_than(0.0),
        }),
        input(In {
            name: "udl",
            description: "Uniformly distributed load w over the full span, acting downward",
            dimension: FORCE_PER_LENGTH,
            si: "N/m",
            display: "kN/m",
            required: false,
            default: Some(0.0),
            domain: Range::at_least(0.0),
        }),
        input(In {
            name: "point_load",
            description: "Point load P, acting downward",
            dimension: FORCE,
            si: "N",
            display: "kN",
            required: false,
            default: Some(0.0),
            domain: Range::at_least(0.0),
        }),
        input(In {
            name: "point_load_position",
            description: "Distance a of the point load from support A, 0 ≤ a ≤ L; required when point_load > 0",
            dimension: LENGTH,
            si: "m",
            display: "m",
            required: false,
            default: None,
            domain: Range::at_least(0.0),
        }),
        input(In {
            name: "elastic_modulus",
            description: "Modulus of elasticity E",
            dimension: PRESSURE,
            si: "Pa",
            display: "GPa",
            required: true,
            default: None,
            domain: Range::greater_than(0.0),
        }),
        input(In {
            name: "second_moment",
            description: "Second moment of area I about the bending axis",
            dimension: SECOND_MOMENT,
            si: "m^4",
            display: "mm^4",
            required: true,
            default: None,
            domain: Range::greater_than(0.0),
        }),
        input(In {
            name: "position",
            description: "Distance x from support A at which the deflection is wanted, 0 ≤ x ≤ L",
            dimension: LENGTH,
            si: "m",
            display: "m",
            required: true,
            default: None,
            domain: Range::at_least(0.0),
        }),
    ],
    outputs: vec![
        output("reaction_a", "Vertical reaction at A (left), upward positive", FORCE, "N", "kN"),
        output("reaction_b", "Vertical reaction at B (right), upward positive", FORCE, "N", "kN"),
        output(
            "deflection",
            "Deflection at x, upward positive (downward deflection is negative)",
            LENGTH,
            "m",
            "mm",
        ),
        output(
            "end_slope_a",
            "Slope θ_A of the deflected shape at A, counter-clockwise positive",
            Dimension::ANGLE,
            "rad",
            "rad",
        ),
    ],
    references: vec![
        "F. Udoeyo, Structural Analysis (Temple University, 2020), §1.7, Example 7.2: reactions, slope at A and \
         midspan deflection of a beam under a UDL and a midspan point load. CC BY-NC-ND 4.0."
            .into(),
        "J. Lord et al., Strength of Materials (Virginia Tech), Examples 11.1 and 11.5: deflection under a UDL and, \
         by superposition, an off-centre point load. CC BY-NC-SA 4.0."
            .into(),
    ],
    assumptions: vec![
        "Euler–Bernoulli beam theory: small deflections, plane sections remain plane, shear deformation ignored."
            .into(),
        "Linear elastic, prismatic beam: E and I are constant along the span; self-weight is included only if it \
         is part of w."
            .into(),
        "Simple supports (pin at A, roller at B) at the beam ends; loads act vertically downward.".into(),
        "The UDL and the point load are combined by superposition.".into(),
    ],
    validity: Vec::new(),
    examples: vec![
        "tests/reference/structural.beam.simply_supported/udoeyo-ex7-2.json".into(),
        "tests/reference/structural.beam.simply_supported/lord-ex11-1.json".into(),
        "tests/reference/structural.beam.simply_supported/lord-ex11-5.json".into(),
    ],
    ..Default::default() // no list groups (D-23)
}
});

impl Calculation for SimplySupportedBeam {
    fn descriptor(&self) -> &Descriptor {
        &BEAM
    }

    fn compute(&self, inputs: &Inputs, ctx: &mut Context) -> Result<Outputs, CalcError> {
        let l = inputs.require("span")?;
        let w = inputs.require("udl")?;
        let p = inputs.require("point_load")?;
        let e = inputs.require("elastic_modulus")?;
        let i = inputs.require("second_moment")?;
        let x = inputs.require("position")?;

        let mut errors = Vec::new();
        if x > l {
            errors.push(CalcError::for_input(
                ErrorCode::OutOfDomain,
                "position",
                format!("position {x} m is beyond the span of {l} m"),
            ));
        }
        // The position of the point load matters only when there is a point load.
        let a = match inputs.get("point_load_position") {
            Some(a) if a > l => {
                errors.push(CalcError::for_input(
                    ErrorCode::OutOfDomain,
                    "point_load_position",
                    format!("point_load_position {a} m is beyond the span of {l} m"),
                ));
                a
            }
            Some(a) => a,
            None if p > 0.0 => {
                errors.push(CalcError::for_input(
                    ErrorCode::MissingInput,
                    "point_load_position",
                    "point_load_position is required when point_load is greater than 0",
                ));
                0.0
            }
            None => 0.0,
        };
        if let Some(first) = errors.into_iter().next() {
            // compute() returns one error; the registry has already reported every per-input problem.
            return Err(first);
        }
        let b = l - a;
        let ei = e * i;

        let reaction_a = w * l / 2.0 + p * b / l;
        ctx.step("R_A = w·L/2 + P·(L − a)/L", reaction_a, Some("N"));
        let reaction_b = w * l / 2.0 + p * a / l;
        ctx.step("R_B = w·L/2 + P·a/L", reaction_b, Some("N"));
        ctx.step("EI = E·I", ei, Some("N*m^2"));

        let y_udl = -w * x * (l.powi(3) - 2.0 * l * x * x + x.powi(3)) / (24.0 * ei);
        ctx.step("y_w = −w·x·(L³ − 2L·x² + x³) / (24·EI)", y_udl, Some("m"));
        let y_point = if x <= a {
            let y = -p * b * x * (l * l - b * b - x * x) / (6.0 * l * ei);
            ctx.step(
                "y_P = −P·b·x·(L² − b² − x²) / (6·L·EI), x ≤ a, b = L − a",
                y,
                Some("m"),
            );
            y
        } else {
            let y = -p * a * (l - x) * (l * l - a * a - (l - x).powi(2)) / (6.0 * l * ei);
            ctx.step(
                "y_P = −P·a·(L − x)·(L² − a² − (L − x)²) / (6·L·EI), x > a",
                y,
                Some("m"),
            );
            y
        };
        let deflection = y_udl + y_point;
        ctx.step("y = y_w + y_P", deflection, Some("m"));

        let slope = -w * l.powi(3) / (24.0 * ei) - p * b * (l * l - b * b) / (6.0 * l * ei);
        ctx.step(
            "θ_A = −w·L³/(24·EI) − P·b·(L² − b²)/(6·L·EI)",
            slope,
            Some("rad"),
        );

        Ok(Outputs::new()
            .with("reaction_a", reaction_a)
            .with("reaction_b", reaction_b)
            .with("deflection", deflection)
            .with("end_slope_a", slope))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{InputValue, Registry, Request};

    fn call(inputs: &[(&str, f64, &str)]) -> Result<crate::Response, crate::ErrorEnvelope> {
        let mut r = Registry::new();
        r.register(Box::new(SimplySupportedBeam))
            .expect("valid descriptor");
        r.call(&Request {
            id: "structural.beam.simply_supported".into(),
            version: Some(1),
            inputs: inputs
                .iter()
                .map(|(k, v, u)| {
                    let value = InputValue {
                        unit: Some((*u).to_owned()),
                        ..InputValue::number(*v)
                    };
                    ((*k).to_owned(), value)
                })
                .collect(),
            output_units: [
                ("reaction_a", "N"),
                ("reaction_b", "N"),
                ("deflection", "m"),
                ("end_slope_a", "rad"),
            ]
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect(),
        })
    }

    /// L = 4 m, E·I = 1e6 N·m² (E = 1 Pa would be unrealistic; use 1e10 Pa and 1e-4 m^4).
    fn beam(extra: &[(&'static str, f64, &'static str)]) -> Vec<(&'static str, f64, &'static str)> {
        let mut v = vec![
            ("span", 4.0, "m"),
            ("elastic_modulus", 1e10, "Pa"),
            ("second_moment", 1e-4, "m^4"),
        ];
        v.extend_from_slice(extra);
        v
    }

    fn value(r: &crate::Response, name: &str) -> f64 {
        r.outputs[name].value.as_scalar().expect("scalar output")
    }

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() <= 1e-12 * b.abs().max(1e-30), "{a} vs {b}");
    }

    #[test]
    fn udl_closed_forms() {
        // Midspan deflection 5wL⁴/(384EI), end slope wL³/(24EI), reactions wL/2.
        let (w, l, ei): (f64, f64, f64) = (1000.0, 4.0, 1e6);
        let r = call(&beam(&[("udl", w, "N/m"), ("position", 2.0, "m")])).expect("udl");
        close(value(&r, "reaction_a"), w * l / 2.0);
        close(value(&r, "reaction_b"), w * l / 2.0);
        close(value(&r, "deflection"), -5.0 * w * l.powi(4) / (384.0 * ei));
        close(value(&r, "end_slope_a"), -w * l.powi(3) / (24.0 * ei));
    }

    #[test]
    fn midspan_point_load_closed_forms() {
        // PL³/(48EI) at midspan, PL²/(16EI) at the ends.
        let (p, l, ei): (f64, f64, f64) = (1000.0, 4.0, 1e6);
        let r = call(&beam(&[
            ("point_load", p, "N"),
            ("point_load_position", 2.0, "m"),
            ("position", 2.0, "m"),
        ]))
        .expect("point load");
        close(value(&r, "deflection"), -p * l.powi(3) / (48.0 * ei));
        close(value(&r, "end_slope_a"), -p * l * l / (16.0 * ei));
    }

    #[test]
    fn deflection_is_zero_at_the_supports_and_continuous_under_the_load() {
        let loads = [
            ("udl", 500.0, "N/m"),
            ("point_load", 800.0, "N"),
            ("point_load_position", 1.0, "m"),
        ];
        for x in [0.0, 4.0] {
            let mut i = beam(&loads);
            i.push(("position", x, "m"));
            let r = call(&i).expect("support");
            assert!(value(&r, "deflection").abs() <= 1e-18, "x = {x}");
        }
        // Both branches agree at x = a.
        let mut i = beam(&loads);
        i.push(("position", 1.0, "m"));
        let at = value(&call(&i).expect("at a"), "deflection");
        let mut j = beam(&loads);
        j.push(("position", 1.0 + 1e-12, "m"));
        let past = value(&call(&j).expect("past a"), "deflection");
        assert!((at - past).abs() <= 1e-12 * at.abs());
    }

    #[test]
    fn mirror_symmetry() {
        // A load at a seen from B is a load at L − a: R_A and R_B swap, deflections mirror.
        let left = call(&beam(&[
            ("point_load", 1000.0, "N"),
            ("point_load_position", 1.0, "m"),
            ("position", 1.5, "m"),
        ]))
        .expect("left");
        let right = call(&beam(&[
            ("point_load", 1000.0, "N"),
            ("point_load_position", 3.0, "m"),
            ("position", 2.5, "m"),
        ]))
        .expect("right");
        close(value(&left, "reaction_a"), value(&right, "reaction_b"));
        close(value(&left, "deflection"), value(&right, "deflection"));
    }

    #[test]
    fn load_on_a_support_goes_straight_into_it() {
        let r = call(&beam(&[
            ("point_load", 1000.0, "N"),
            ("point_load_position", 0.0, "m"),
            ("position", 2.0, "m"),
        ]))
        .expect("at support");
        close(value(&r, "reaction_a"), 1000.0);
        assert!(value(&r, "reaction_b").abs() <= 1e-12);
        assert!(value(&r, "deflection").abs() <= 1e-18);
    }

    #[test]
    fn point_load_needs_its_position() {
        let e = call(&beam(&[
            ("point_load", 1000.0, "N"),
            ("position", 2.0, "m"),
        ]))
        .expect_err("missing");
        assert_eq!(e.0[0].code, ErrorCode::MissingInput);
        assert_eq!(e.0[0].input.as_deref(), Some("point_load_position"));
        // Without a point load the position is not needed.
        assert!(call(&beam(&[("udl", 1.0, "N/m"), ("position", 2.0, "m")])).is_ok());
    }

    #[test]
    fn positions_beyond_the_span_are_out_of_domain() {
        let e = call(&beam(&[("udl", 1.0, "N/m"), ("position", 4.5, "m")])).expect_err("x > L");
        assert_eq!(e.0[0].code, ErrorCode::OutOfDomain);
        assert_eq!(e.0[0].input.as_deref(), Some("position"));
        let e = call(&beam(&[
            ("point_load", 1.0, "N"),
            ("point_load_position", 5.0, "m"),
            ("position", 2.0, "m"),
        ]))
        .expect_err("a > L");
        assert_eq!(e.0[0].input.as_deref(), Some("point_load_position"));
    }

    #[test]
    fn negative_loads_and_stiffness_are_out_of_domain() {
        let e = call(&beam(&[("udl", -1.0, "N/m"), ("position", 2.0, "m")])).expect_err("w < 0");
        assert_eq!(e.0[0].code, ErrorCode::OutOfDomain);
        let mut i = beam(&[("position", 2.0, "m")]);
        i[1].1 = 0.0; // E
        let e = call(&i).expect_err("E = 0");
        assert_eq!(e.0[0].input.as_deref(), Some("elastic_modulus"));
    }

    #[test]
    fn trace_is_in_internal_units() {
        let r = call(&beam(&[("udl", 1.0, "kN/m"), ("position", 2.0, "m")])).expect("ok");
        assert_eq!(r.trace.len(), 7);
    }
}
