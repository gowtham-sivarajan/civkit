// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-26 00:18 IST · last edited 2026-11-26 20:58 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! Quantities calculations for 0.1 (D-20, ADR 0009).
//!
//! `quantities.earthwork.average_end_area`: earthwork volumes between cross-sections by the average-end-area method,
//! with the zero-end pyramid as an **explicitly requested** method. The descriptor is the one the maintainer accepted
//! in `planning/EARTHWORK_DESCRIPTOR_PROPOSAL.md`. The engine never switches methods silently (ADR 0009 condition 3):
//! `method` is a required choice input and is the first step of the trace.
//!
//! Not in 0.1: the general prismoidal formula, shrink and swell, mass-haul ordinates, curvature correction.

use crate::{
    CalcError, Calculation, Context, Descriptor, ErrorCode, Group, InputKind, InputSpec, Inputs,
    ListLength, OutputKind, OutputSpec, Outputs, Range,
};
use civkit_units::Dimension;
use std::sync::LazyLock;

const STATIONS: &str = "stations";
const STATION: &str = "station";
const SEGMENT: &str = "segment";
const AVERAGE_END_AREA: &str = "average_end_area";
const PYRAMID_AT_ZERO_END: &str = "pyramid_at_zero_end";

/// How a segment's volume is computed. Chosen by the caller, never by the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Method {
    AverageEndArea,
    PyramidAtZeroEnd,
}

impl Method {
    fn parse(text: &str) -> Result<Self, CalcError> {
        match text {
            AVERAGE_END_AREA => Ok(Self::AverageEndArea),
            PYRAMID_AT_ZERO_END => Ok(Self::PyramidAtZeroEnd),
            // The registry has already checked the value against the descriptor's choices.
            other => Err(CalcError::internal(format!(
                "unknown method '{other}' reached compute()"
            ))),
        }
    }
}

/// Volume of one segment for one kind (cut or fill) with end areas `a1` and `a2` and length `length`, and whether
/// the pyramid formula was used.
///
/// Average end area is (A₁ + A₂)/2·L. The pyramid formula, A·L/3, is used only when it was requested **and** exactly
/// one end area is zero. Areas are never negative (the registry has checked the domain), so "zero" is `≤ 0`.
fn segment_volume(method: Method, a1: f64, a2: f64, length: f64) -> (f64, bool) {
    match method {
        Method::PyramidAtZeroEnd if (a1 <= 0.0) != (a2 <= 0.0) => ((a1 + a2) * length / 3.0, true),
        _ => ((a1 + a2) / 2.0 * length, false),
    }
}

/// A caller problem, not a bug: absurdly large inputs whose volume is not a finite number.
fn too_large(input: &str, k: usize, what: &str) -> CalcError {
    let mut e = CalcError::for_input(
        ErrorCode::OutOfDomain,
        input,
        format!("{SEGMENT} {}: the {what} is too large to compute", k + 1),
    );
    e.index = Some(k);
    e
}

/// The chainages must strictly increase: the contract has no rule across the elements of one list, so the
/// calculation checks it. Reports the first station that is not beyond the one before.
fn check_chainages(x: &[f64]) -> Result<(), CalcError> {
    for k in 1..x.len() {
        // The registry guarantees finite values, so `<=` is the exact complement of "strictly greater".
        if x[k] <= x[k - 1] {
            let mut e = CalcError::for_input(
                ErrorCode::OutOfDomain,
                "chainages",
                format!(
                    "{STATION} {}: chainage {} m must be greater than {STATION} {} ({} m)",
                    k + 1,
                    x[k],
                    k,
                    x[k - 1]
                ),
            );
            e.index = Some(k);
            return Err(e);
        }
    }
    Ok(())
}

/// Volumes of every segment, which formula each used, and the totals.
#[derive(Debug, Clone, PartialEq)]
struct Earthwork {
    cut_volumes: Vec<f64>,
    fill_volumes: Vec<f64>,
    cut_pyramid: Vec<bool>,
    fill_pyramid: Vec<bool>,
    total_cut: f64,
    total_fill: f64,
    net_fill: f64,
}

fn earthwork(
    method: Method,
    chainages: &[f64],
    cut: &[f64],
    fill: &[f64],
) -> Result<Earthwork, CalcError> {
    let n = chainages.len();
    if n < 2 || cut.len() != n || fill.len() != n {
        return Err(CalcError::internal(
            "earthwork lists differ in length or are too short",
        ));
    }
    let mut out = Earthwork {
        cut_volumes: Vec::with_capacity(n - 1),
        fill_volumes: Vec::with_capacity(n - 1),
        cut_pyramid: Vec::with_capacity(n - 1),
        fill_pyramid: Vec::with_capacity(n - 1),
        total_cut: 0.0,
        total_fill: 0.0,
        net_fill: 0.0,
    };
    for k in 0..n - 1 {
        let length = chainages[k + 1] - chainages[k];
        let (cv, cp) = segment_volume(method, cut[k], cut[k + 1], length);
        let (fv, fp) = segment_volume(method, fill[k], fill[k + 1], length);
        if !cv.is_finite() {
            return Err(too_large("cut_areas", k, "cut volume"));
        }
        if !fv.is_finite() {
            return Err(too_large("fill_areas", k, "fill volume"));
        }
        out.cut_volumes.push(cv);
        out.fill_volumes.push(fv);
        out.cut_pyramid.push(cp);
        out.fill_pyramid.push(fp);
    }
    out.total_cut = out.cut_volumes.iter().sum();
    out.total_fill = out.fill_volumes.iter().sum();
    out.net_fill = out.total_fill - out.total_cut;
    if !(out.total_cut.is_finite() && out.total_fill.is_finite() && out.net_fill.is_finite()) {
        return Err(CalcError::for_input(
            ErrorCode::OutOfDomain,
            "cut_areas",
            "the total volume is too large to compute",
        ));
    }
    Ok(out)
}

fn area_input(name: &str, description: &str) -> InputSpec {
    InputSpec {
        name: name.into(),
        description: description.into(),
        dimension: Dimension::AREA,
        si_unit: Some("m^2".into()),
        display_unit: Some("m^2".into()),
        required: true,
        domain: Range::at_least(0.0),
        kind: InputKind::List,
        group: Some(STATIONS.into()),
        ..Default::default()
    }
}

fn volume_list(name: &str, description: &str) -> OutputSpec {
    OutputSpec {
        name: name.into(),
        description: description.into(),
        dimension: Dimension::VOLUME,
        si_unit: Some("m^3".into()),
        display_unit: Some("m^3".into()),
        kind: OutputKind::List,
        length: Some(ListLength {
            group: STATIONS.into(),
            offset: -1,
            item: SEGMENT.into(),
        }),
    }
}

fn volume_scalar(name: &str, description: &str) -> OutputSpec {
    OutputSpec {
        name: name.into(),
        description: description.into(),
        dimension: Dimension::VOLUME,
        si_unit: Some("m^3".into()),
        display_unit: Some("m^3".into()),
        ..Default::default()
    }
}

/// Earthwork volumes by average end area, with the zero-end pyramid as an explicitly requested method.
#[derive(Debug, Default)]
pub struct AverageEndArea;

static DESCRIPTOR: LazyLock<Descriptor> = LazyLock::new(|| {
    Descriptor {
    id: "quantities.earthwork.average_end_area".into(),
    version: 1,
    title: "Earthwork volumes: average end area, with the zero-end pyramid on request".into(),
    summary: "Cut and fill volumes between consecutive cross-sections from their end areas. With \
              `average_end_area` every segment is (A₁ + A₂)/2·L. With `pyramid_at_zero_end` a segment whose cut (or \
              fill) area is zero at exactly one end is a pyramid, A·L/3; all other segments use the average. \
              The method is a required input and is the first step of the trace: the engine never chooses it. \
              Net fill is total fill minus total cut (positive: fill must be brought in)."
        .into(),
    inputs: vec![
        InputSpec {
            name: "method".into(),
            description: "How segment volumes are computed: `average_end_area` (every segment) or \
                          `pyramid_at_zero_end` (average end area, except A·L/3 where exactly one end area of a \
                          kind is zero). Required; there is no default."
                .into(),
            required: true,
            kind: InputKind::Choice,
            choices: vec![AVERAGE_END_AREA.into(), PYRAMID_AT_ZERO_END.into()],
            ..Default::default()
        },
        InputSpec {
            name: "chainages".into(),
            description: "Chainage of each cross-section, strictly increasing (station 0+50 is 50 m)".into(),
            dimension: Dimension::LENGTH,
            si_unit: Some("m".into()),
            display_unit: Some("m".into()),
            required: true,
            kind: InputKind::List,
            group: Some(STATIONS.into()),
            ..Default::default()
        },
        area_input(
            "cut_areas",
            "Cut cross-section area at each station; 0 where there is no cut",
        ),
        area_input(
            "fill_areas",
            "Fill cross-section area at each station; 0 where there is no fill",
        ),
    ],
    groups: vec![Group {
        name: STATIONS.into(),
        item: STATION.into(),
        min_len: 2,
        description: "Cross-section stations in order of increasing chainage. Segment k lies between station k \
                      and station k + 1."
            .into(),
    }],
    outputs: vec![
        volume_list("cut_volumes", "Cut volume of each segment between consecutive stations"),
        volume_list("fill_volumes", "Fill volume of each segment between consecutive stations"),
        volume_scalar("total_cut", "Sum of the cut volumes"),
        volume_scalar("total_fill", "Sum of the fill volumes"),
        volume_scalar(
            "net_fill",
            "Total fill minus total cut: positive when fill must be brought in, negative when cut is in surplus",
        ),
    ],
    references: vec![
        "Fundamentals of Transportation, \"Earthwork\", Example 1 and the Solution page (Wikibooks; mirrored on \
         Engineering LibreTexts §7.3): average end area and the zero-end pyramid. CC BY-SA."
            .into(),
    ],
    assumptions: vec![
        "End areas are cross-sectional areas perpendicular to the centreline at the stations, and the section \
         changes linearly between stations."
            .into(),
        "Average end area is an approximation. It overstates a segment that tapers to a zero end (a pyramid has \
         A·L/3, not A·L/2), which is why the pyramid method is offered, and never applied automatically."
            .into(),
        "No shrink or swell, no curvature correction, and no prismoidal correction (not in 0.1).".into(),
        "Cut and fill are not netted within a station.".into(),
        super::NO_VALIDITY_RANGE.into(),
    ],
    validity: Vec::new(),
    examples: vec![
        "tests/reference/quantities.earthwork.average_end_area/wikibooks-ex1.json".into(),
        "tests/reference/quantities.earthwork.average_end_area/wikibooks-solution.json".into(),
    ],
}
});

impl Calculation for AverageEndArea {
    fn descriptor(&self) -> &Descriptor {
        &DESCRIPTOR
    }

    fn compute(&self, inputs: &Inputs, ctx: &mut Context) -> Result<Outputs, CalcError> {
        let method_name = inputs.choice("method")?;
        let method = Method::parse(method_name)?;
        // Recorded first, before any numeric step (D-23): the user can always see which method was requested.
        ctx.choice("method", method_name);

        let chainages = inputs.list("chainages")?;
        let cut = inputs.list("cut_areas")?;
        let fill = inputs.list("fill_areas")?;
        check_chainages(chainages)?;

        let e = earthwork(method, chainages, cut, fill)?;
        for k in 0..chainages.len() - 1 {
            let cut_formula = if e.cut_pyramid[k] {
                "cut volume = A·L/3 (zero end, pyramid)"
            } else {
                "cut volume = (A₁ + A₂)/2·L"
            };
            let fill_formula = if e.fill_pyramid[k] {
                "fill volume = A·L/3 (zero end, pyramid)"
            } else {
                "fill volume = (A₁ + A₂)/2·L"
            };
            ctx.step_item(SEGMENT, k, cut_formula, e.cut_volumes[k], Some("m^3"));
            ctx.step_item(SEGMENT, k, fill_formula, e.fill_volumes[k], Some("m^3"));
        }
        ctx.step("ΣV_cut", e.total_cut, Some("m^3"));
        ctx.step("ΣV_fill", e.total_fill, Some("m^3"));
        ctx.step("net fill = ΣV_fill − ΣV_cut", e.net_fill, Some("m^3"));

        Ok(Outputs::new()
            .with_list("cut_volumes", e.cut_volumes)
            .with_list("fill_volumes", e.fill_volumes)
            .with("total_cut", e.total_cut)
            .with("total_fill", e.total_fill)
            .with("net_fill", e.net_fill))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Reference case C2: stations 0, 50, 75, 100, 115, 150 m.
    const X: [f64; 6] = [0.0, 50.0, 75.0, 100.0, 115.0, 150.0];
    const CUT: [f64; 6] = [0.0, 0.0, 0.0, 10.0, 15.0, 30.0];
    const FILL: [f64; 6] = [60.0, 50.0, 25.0, 5.0, 0.0, 0.0];

    fn close(got: f64, expected: f64, what: &str) {
        assert!(
            (got - expected).abs() <= 1e-9,
            "{what}: got {got}, expected {expected}"
        );
    }

    #[test]
    fn the_pyramid_formula_applies_only_when_requested_and_only_with_one_zero_end() {
        let pyr = Method::PyramidAtZeroEnd;
        let avg = Method::AverageEndArea;
        assert_eq!(segment_volume(pyr, 0.0, 30.0, 50.0), (500.0, true), "A·L/3");
        assert_eq!(
            segment_volume(pyr, 30.0, 0.0, 50.0),
            (500.0, true),
            "either end"
        );
        assert_eq!(
            segment_volume(avg, 0.0, 30.0, 50.0),
            (750.0, false),
            "average end area never switches"
        );
        assert_eq!(
            segment_volume(pyr, 10.0, 30.0, 50.0),
            (1000.0, false),
            "no zero end"
        );
        assert_eq!(
            segment_volume(pyr, 0.0, 0.0, 50.0),
            (0.0, false),
            "two zero ends"
        );
        assert_eq!(segment_volume(avg, 0.0, 0.0, 50.0), (0.0, false));
    }

    #[test]
    fn c2_matches_the_source_under_the_pyramid_method() {
        let e = earthwork(Method::PyramidAtZeroEnd, &X, &CUT, &FILL).expect("c2");
        close(e.total_fill, 4087.5, "fill (book 4 087.5)");
        close(e.total_cut, 1058.0 + 1.0 / 3.0, "cut (book 1 058.3)");
        close(e.net_fill, 3029.0 + 1.0 / 6.0, "net (book 3 029.2)");
        assert_eq!(e.cut_pyramid, [false, false, true, false, false]);
        assert_eq!(e.fill_pyramid, [false, false, false, true, false]);
        close(e.cut_volumes[2], 250.0 / 3.0, "pyramid 10·25/3 = 83.3");
        close(e.fill_volumes[3], 25.0, "pyramid 5·15/3 = 25");
    }

    #[test]
    fn the_same_data_under_average_end_area_is_visibly_different() {
        let e = earthwork(Method::AverageEndArea, &X, &CUT, &FILL).expect("c2");
        close(e.total_fill, 4100.0, "12.5 m³ more fill");
        close(e.total_cut, 1100.0, "41.7 m³ more cut");
        assert!(e.cut_pyramid.iter().chain(&e.fill_pyramid).all(|p| !p));
    }

    #[test]
    fn chainages_must_strictly_increase_and_the_first_offender_is_reported() {
        assert!(check_chainages(&[0.0, 1.0, 2.0]).is_ok());
        let e = check_chainages(&[0.0, 5.0, 5.0, 4.0]).expect_err("equal");
        assert_eq!((e.code, e.index), (ErrorCode::OutOfDomain, Some(2)));
        let e = check_chainages(&[3.0, 2.0]).expect_err("decreasing");
        assert_eq!(e.index, Some(1));
    }

    #[test]
    fn an_overflowing_volume_is_an_out_of_domain_error_naming_the_kind() {
        let e = earthwork(
            Method::AverageEndArea,
            &[0.0, 1.0e10],
            &[1.0e300, 1.0e300],
            &[0.0, 0.0],
        )
        .expect_err("cut");
        assert_eq!(
            (e.code, e.input.as_deref(), e.index),
            (ErrorCode::OutOfDomain, Some("cut_areas"), Some(0))
        );
        let e = earthwork(
            Method::AverageEndArea,
            &[0.0, 1.0e10],
            &[0.0, 0.0],
            &[1.0e300, 1.0e300],
        )
        .expect_err("fill");
        assert_eq!(e.input.as_deref(), Some("fill_areas"));
    }
}
