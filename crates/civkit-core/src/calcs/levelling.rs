// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-26 00:09 IST · last edited 2026-11-26 20:58 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! Levelling calculations for 0.1 (D-20, ADR 0009).
//!
//! Three calculations, as accepted in `planning/LEVELLING_DESCRIPTOR_PROPOSAL.md` (option B):
//!
//! - `surveying.levelling.height_of_instrument`: HI method;
//! - `surveying.levelling.rise_and_fall`: rise-and-fall method;
//! - `surveying.levelling.closure_adjustment`: closure error and proportional adjustment of a run between two
//!   known elevations.
//!
//! Field books are backsight/foresight pairs only: no worked example verifies intermediate sights, so 0.1 has none
//! (D-20). On such field books the two reductions give identical elevations; they differ in the working shown.
//! The rise-and-fall reference cases are labelled as consistency checks (D-20 condition 2).

use crate::{
    CalcError, Calculation, Context, Descriptor, ErrorCode, Group, InputKind, InputSpec, Inputs,
    ListLength, OutputKind, OutputSpec, Outputs, Range,
};
use civkit_units::Dimension;
use std::sync::LazyLock;

const SETUPS: &str = "setups";
const SETUP: &str = "setup";
const POINTS: &str = "points";
const POINT: &str = "point";

// --- Descriptor building blocks ------------------------------------------------------------------

fn length_input(
    name: &str,
    description: &str,
    domain: Range,
    list_group: Option<&str>,
) -> InputSpec {
    InputSpec {
        name: name.into(),
        description: description.into(),
        dimension: Dimension::LENGTH,
        si_unit: Some("m".into()),
        display_unit: Some("m".into()),
        required: true,
        domain,
        kind: if list_group.is_some() {
            InputKind::List
        } else {
            InputKind::Scalar
        },
        group: list_group.map(Into::into),
        ..Default::default()
    }
}

fn length_output(name: &str, description: &str, list_group: Option<(&str, &str)>) -> OutputSpec {
    OutputSpec {
        name: name.into(),
        description: description.into(),
        dimension: Dimension::LENGTH,
        si_unit: Some("m".into()),
        display_unit: Some("m".into()),
        kind: if list_group.is_some() {
            OutputKind::List
        } else {
            OutputKind::Scalar
        },
        length: list_group.map(|(group, item)| ListLength {
            group: group.into(),
            offset: 0,
            item: item.into(),
        }),
    }
}

fn setups_group() -> Group {
    Group {
        name: SETUPS.into(),
        item: SETUP.into(),
        min_len: 1,
        description: "Instrument setups in order of observation. Setup k reads a backsight on the previous point \
                      (the start benchmark for setup 1) and a foresight on the next point."
            .into(),
    }
}

fn reduction_inputs() -> Vec<InputSpec> {
    vec![
        length_input(
            "start_elevation",
            "Elevation of the starting benchmark (below datum is allowed)",
            Range::UNBOUNDED,
            None,
        ),
        length_input(
            "backsights",
            "Staff reading on the previous point, one per setup",
            Range::at_least(0.0),
            Some(SETUPS),
        ),
        length_input(
            "foresights",
            "Staff reading on the next point, one per setup",
            Range::at_least(0.0),
            Some(SETUPS),
        ),
    ]
}

fn sum_outputs() -> Vec<OutputSpec> {
    vec![
        length_output("sum_backsights", "Sum of the backsights, ΣBS", None),
        length_output("sum_foresights", "Sum of the foresights, ΣFS", None),
        length_output(
            "net_rise",
            "ΣBS − ΣFS: positive when the last point is higher than the start (equals last elevation − start)",
            None,
        ),
    ]
}

const ASSUMPTIONS_REDUCTION: [&str; 4] = [
    "The field book is a chain of setups, each with one backsight and one foresight; every foresight is to a point \
     whose elevation is wanted (a benchmark or a change point). Intermediate sights are not supported.",
    "Staff readings are taken from a staff held upright on the point, and are not negative.",
    "The instrument is in adjustment and the line of sight is level; collimation, curvature and refraction errors \
     are not modelled.",
    "Elevations are relative to the datum of the start benchmark.",
];

const SOURCE: &str = "US Navy, Engineering Aid, NAVEDTRA 14069, chapter 14 (level computations): the field book of \
     Fig. 14-14 (pp. 14-13 to 14-15) and the single-setup example of p. 14-10. Public domain.";

// --- Pure arithmetic -----------------------------------------------------------------------------

/// Error for a result that is not a finite number: a caller problem (absurdly large readings), never a `NaN`.
fn too_large(input: &str, item: &str, k: usize, what: &str) -> CalcError {
    let mut e = CalcError::for_input(
        ErrorCode::OutOfDomain,
        input,
        format!("{item} {}: the {what} is too large to compute", k + 1),
    );
    e.index = Some(k);
    e
}

fn finite_or(value: f64, error: impl FnOnce() -> CalcError) -> Result<f64, CalcError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(error())
    }
}

/// Height-of-instrument reduction: HI = previous elevation + BS; elevation = HI − FS.
fn hi_reduction(start: f64, bs: &[f64], fs: &[f64]) -> Result<(Vec<f64>, Vec<f64>), CalcError> {
    let mut previous = start;
    let (mut heights, mut elevations) =
        (Vec::with_capacity(bs.len()), Vec::with_capacity(bs.len()));
    for (k, (b, f)) in bs.iter().zip(fs).enumerate() {
        let hi = finite_or(previous + b, || {
            too_large("backsights", SETUP, k, "height of instrument")
        })?;
        let z = finite_or(hi - f, || too_large("foresights", SETUP, k, "elevation"))?;
        heights.push(hi);
        elevations.push(z);
        previous = z;
    }
    Ok((heights, elevations))
}

/// The three per-setup lists of a rise-and-fall reduction: rises, falls and elevations.
type RiseFallLists = (Vec<f64>, Vec<f64>, Vec<f64>);

/// Rise-and-fall reduction: rise = max(BS − FS, 0), fall = max(FS − BS, 0), elevation = previous + rise − fall.
fn rf_reduction(start: f64, bs: &[f64], fs: &[f64]) -> Result<RiseFallLists, CalcError> {
    let mut previous = start;
    let n = bs.len();
    let (mut rises, mut falls, mut elevations) = (
        Vec::with_capacity(n),
        Vec::with_capacity(n),
        Vec::with_capacity(n),
    );
    for (k, (b, f)) in bs.iter().zip(fs).enumerate() {
        let rise = (b - f).max(0.0);
        let fall = (f - b).max(0.0);
        let z = finite_or(previous + rise - fall, || {
            too_large("backsights", SETUP, k, "elevation")
        })?;
        rises.push(rise);
        falls.push(fall);
        elevations.push(z);
        previous = z;
    }
    Ok((rises, falls, elevations))
}

/// The sums every reduction reports: ΣBS, ΣFS and the net rise ΣBS − ΣFS.
fn sums(bs: &[f64], fs: &[f64]) -> Result<(f64, f64, f64), CalcError> {
    let sum_b: f64 = bs.iter().sum();
    let sum_f: f64 = fs.iter().sum();
    let net = sum_b - sum_f;
    if !(sum_b.is_finite() && sum_f.is_finite() && net.is_finite()) {
        return Err(CalcError::for_input(
            ErrorCode::OutOfDomain,
            "backsights",
            "the sum of the readings is too large to compute",
        ));
    }
    Ok((sum_b, sum_f, net))
}

/// The result of closing a run on a known elevation and adjusting it in proportion to the distance run.
#[derive(Debug, Clone, PartialEq)]
struct Closure {
    closure_error: f64,
    total_length: f64,
    corrections: Vec<f64>,
    adjusted: Vec<f64>,
}

/// e = Z_n − Z_known; c_k = −e·S_k/L with S_k the cumulative length; Z′_k = Z_k + c_k.
fn closure_adjustment(
    elevations: &[f64],
    lengths: &[f64],
    closing_elevation: f64,
) -> Result<Closure, CalcError> {
    if elevations.len() != lengths.len() || elevations.is_empty() {
        return Err(CalcError::internal(
            "levelling lists differ in length or are empty",
        ));
    }
    let total: f64 = lengths.iter().sum();
    if !total.is_finite() {
        return Err(CalcError::for_input(
            ErrorCode::OutOfDomain,
            "section_lengths",
            "the total length is too large to compute",
        ));
    }
    if total <= 0.0 {
        return Err(CalcError::internal("total length is not positive"));
    }
    let last = elevations[elevations.len() - 1];
    let error = finite_or(last - closing_elevation, || {
        CalcError::for_input(
            ErrorCode::OutOfDomain,
            "closing_elevation",
            "the closure error is too large to compute",
        )
    })?;
    let mut cumulative = 0.0;
    let (mut corrections, mut adjusted) = (Vec::new(), Vec::new());
    for (k, (z, s)) in elevations.iter().zip(lengths).enumerate() {
        cumulative += s;
        // S_k / L is at most 1, so the product cannot overflow.
        let c = -error * (cumulative / total);
        let a = finite_or(z + c, || {
            too_large("elevations", POINT, k, "adjusted elevation")
        })?;
        corrections.push(c);
        adjusted.push(a);
    }
    Ok(Closure {
        closure_error: error,
        total_length: total,
        corrections,
        adjusted,
    })
}

// --- surveying.levelling.height_of_instrument ----------------------------------------------------

/// Differential levelling, height-of-instrument method.
#[derive(Debug, Default)]
pub struct HeightOfInstrument;

static HI_DESCRIPTOR: LazyLock<Descriptor> = LazyLock::new(|| {
    let mut outputs = vec![
        length_output(
            "heights_of_instrument",
            "Height of instrument of each setup, HI = previous elevation + BS",
            Some((SETUPS, SETUP)),
        ),
        length_output(
            "elevations",
            "Elevation of the point sighted by each setup's foresight, HI − FS",
            Some((SETUPS, SETUP)),
        ),
    ];
    outputs.extend(sum_outputs());
    Descriptor {
        id: "surveying.levelling.height_of_instrument".into(),
        version: 1,
        title: "Differential levelling: height-of-instrument method".into(),
        summary: "Reduces a levelling field book of backsight and foresight pairs from a starting benchmark: the \
                  height of instrument of each setup (previous elevation + backsight), the elevation of each \
                  point sighted (height of instrument − foresight), the sums of backsights and foresights and \
                  their difference. Closure and adjustment are a separate calculation."
            .into(),
        inputs: reduction_inputs(),
        groups: vec![setups_group()],
        outputs,
        references: vec![SOURCE.into()],
        assumptions: ASSUMPTIONS_REDUCTION
            .iter()
            .copied()
            .chain([super::NO_VALIDITY_RANGE])
            .map(str::to_owned)
            .collect(),
        validity: Vec::new(),
        examples: vec![
            "tests/reference/surveying.levelling.height_of_instrument/navedtra-14069-fig14-14.json".into(),
            "tests/reference/surveying.levelling.height_of_instrument/navedtra-14069-p14-10.json".into(),
        ],
    }
});

impl Calculation for HeightOfInstrument {
    fn descriptor(&self) -> &Descriptor {
        &HI_DESCRIPTOR
    }

    fn compute(&self, inputs: &Inputs, ctx: &mut Context) -> Result<Outputs, CalcError> {
        let start = inputs.require("start_elevation")?;
        let bs = inputs.list("backsights")?;
        let fs = inputs.list("foresights")?;
        let (heights, elevations) = hi_reduction(start, bs, fs)?;
        for k in 0..bs.len() {
            ctx.step_item(
                SETUP,
                k,
                "HI = previous elevation + BS",
                heights[k],
                Some("m"),
            );
            ctx.step_item(SETUP, k, "elevation = HI − FS", elevations[k], Some("m"));
        }
        let (sum_b, sum_f, net) = sums(bs, fs)?;
        ctx.step("ΣBS", sum_b, Some("m"));
        ctx.step("ΣFS", sum_f, Some("m"));
        ctx.step("net rise = ΣBS − ΣFS", net, Some("m"));
        Ok(Outputs::new()
            .with_list("heights_of_instrument", heights)
            .with_list("elevations", elevations)
            .with("sum_backsights", sum_b)
            .with("sum_foresights", sum_f)
            .with("net_rise", net))
    }
}

// --- surveying.levelling.rise_and_fall -----------------------------------------------------------

/// Differential levelling, rise-and-fall method.
#[derive(Debug, Default)]
pub struct RiseAndFall;

static RF_DESCRIPTOR: LazyLock<Descriptor> = LazyLock::new(|| {
    let mut outputs = vec![
        length_output(
            "rises",
            "Rise of each setup, max(BS − FS, 0)",
            Some((SETUPS, SETUP)),
        ),
        length_output(
            "falls",
            "Fall of each setup, max(FS − BS, 0)",
            Some((SETUPS, SETUP)),
        ),
        length_output(
            "elevations",
            "Elevation of the point sighted by each setup's foresight, previous elevation + rise − fall",
            Some((SETUPS, SETUP)),
        ),
    ];
    outputs.extend(sum_outputs());
    Descriptor {
        id: "surveying.levelling.rise_and_fall".into(),
        version: 1,
        title: "Differential levelling: rise-and-fall method".into(),
        summary: "Reduces a levelling field book of backsight and foresight pairs from a starting benchmark by \
                  the rise-and-fall method: the rise or fall of each setup (backsight − foresight), the elevation \
                  of each point sighted, the sums of backsights and foresights and their difference. For a field \
                  book of backsight and foresight pairs it gives the same elevations as the height-of-instrument \
                  method; the working differs. Closure and adjustment are a separate calculation."
            .into(),
        inputs: reduction_inputs(),
        groups: vec![setups_group()],
        outputs,
        references: vec![
            SOURCE.into(),
            "The method is the standard textbook procedure. The cited source reduces its field books by the \
             height-of-instrument method, so the reference cases check numerical consistency with those results; \
             they are not independent validation of this method by a source (D-20 condition 2)."
                .into(),
        ],
        assumptions: ASSUMPTIONS_REDUCTION
            .iter()
            .copied()
            .chain([super::NO_VALIDITY_RANGE])
            .map(str::to_owned)
            .collect(),
        validity: Vec::new(),
        examples: vec![
            "tests/reference/surveying.levelling.rise_and_fall/navedtra-14069-fig14-14.json".into(),
            "tests/reference/surveying.levelling.rise_and_fall/navedtra-14069-p14-10.json".into(),
        ],
    }
});

impl Calculation for RiseAndFall {
    fn descriptor(&self) -> &Descriptor {
        &RF_DESCRIPTOR
    }

    fn compute(&self, inputs: &Inputs, ctx: &mut Context) -> Result<Outputs, CalcError> {
        let start = inputs.require("start_elevation")?;
        let bs = inputs.list("backsights")?;
        let fs = inputs.list("foresights")?;
        let (rises, falls, elevations) = rf_reduction(start, bs, fs)?;
        for k in 0..bs.len() {
            ctx.step_item(SETUP, k, "rise = max(BS − FS, 0)", rises[k], Some("m"));
            ctx.step_item(SETUP, k, "fall = max(FS − BS, 0)", falls[k], Some("m"));
            ctx.step_item(
                SETUP,
                k,
                "elevation = previous elevation + rise − fall",
                elevations[k],
                Some("m"),
            );
        }
        let (sum_b, sum_f, net) = sums(bs, fs)?;
        ctx.step("ΣBS", sum_b, Some("m"));
        ctx.step("ΣFS", sum_f, Some("m"));
        ctx.step("net rise = ΣBS − ΣFS", net, Some("m"));
        Ok(Outputs::new()
            .with_list("rises", rises)
            .with_list("falls", falls)
            .with_list("elevations", elevations)
            .with("sum_backsights", sum_b)
            .with("sum_foresights", sum_f)
            .with("net_rise", net))
    }
}

// --- surveying.levelling.closure_adjustment ------------------------------------------------------

/// Closure error of a levelling run between two known elevations, and its proportional adjustment.
#[derive(Debug, Default)]
pub struct ClosureAdjustment;

static ADJ_DESCRIPTOR: LazyLock<Descriptor> = LazyLock::new(|| {
    Descriptor {
    id: "surveying.levelling.closure_adjustment".into(),
    version: 1,
    title: "Levelling closure error and proportional adjustment".into(),
    summary: "Closes a levelling run on the known elevation of its last point: the closure error e = computed − \
              known (positive when the run came out too high), the total length of the run, and a correction to \
              each computed elevation proportional to the distance run, c = −e·(distance so far)/(total length). \
              The adjusted last elevation equals the known one. A closed loop is the same calculation with the \
              start elevation as the closing elevation."
        .into(),
    inputs: vec![
        length_input(
            "elevations",
            "Computed elevation of each point, in order of observation; the last is the point whose elevation is known",
            Range::UNBOUNDED,
            Some(POINTS),
        ),
        length_input(
            "section_lengths",
            "Length of the route from the previous point (the start benchmark for point 1) to each point, e.g. the \
             backsight distance plus the foresight distance of the setup",
            Range::greater_than(0.0),
            Some(POINTS),
        ),
        length_input(
            "closing_elevation",
            "Known elevation of the last point",
            Range::UNBOUNDED,
            None,
        ),
    ],
    groups: vec![Group {
        name: POINTS.into(),
        item: POINT.into(),
        min_len: 1,
        description: "Points whose elevations were computed from the start benchmark, in order of observation. \
                      The last point is the one whose elevation is also known."
            .into(),
    }],
    outputs: vec![
        length_output(
            "closure_error",
            "Closure error e = computed − known elevation of the last point (signed)",
            None,
        ),
        length_output("total_length", "Total length of the run, L = Σ section lengths", None),
        length_output(
            "corrections",
            "Correction to each elevation, −e·(cumulative length)/L",
            Some((POINTS, POINT)),
        ),
        length_output(
            "adjusted_elevations",
            "Computed elevation of each point plus its correction",
            Some((POINTS, POINT)),
        ),
    ],
    references: vec![SOURCE.into()],
    assumptions: vec![
        "Errors accumulate in proportion to the distance run, so the closure error is shared out by cumulative length."
            .into(),
        "The closing elevation is the known elevation of the last point; the start benchmark is not adjusted.".into(),
        "No permissible-closure test is applied: the allowable misclosure depends on the class of work and is not \
         part of this calculation."
            .into(),
        super::NO_VALIDITY_RANGE.into(),
    ],
    validity: Vec::new(),
    examples: vec![
        "tests/reference/surveying.levelling.closure_adjustment/navedtra-14069-fig14-14.json".into(),
    ],
}
});

impl Calculation for ClosureAdjustment {
    fn descriptor(&self) -> &Descriptor {
        &ADJ_DESCRIPTOR
    }

    fn compute(&self, inputs: &Inputs, ctx: &mut Context) -> Result<Outputs, CalcError> {
        let elevations = inputs.list("elevations")?;
        let lengths = inputs.list("section_lengths")?;
        let closing = inputs.require("closing_elevation")?;
        let c = closure_adjustment(elevations, lengths, closing)?;
        ctx.step("L = Σs", c.total_length, Some("m"));
        ctx.step("e = Z_n − Z_known", c.closure_error, Some("m"));
        for k in 0..elevations.len() {
            ctx.step_item(POINT, k, "correction = −e·S/L", c.corrections[k], Some("m"));
            ctx.step_item(
                POINT,
                k,
                "adjusted elevation = Z + correction",
                c.adjusted[k],
                Some("m"),
            );
        }
        Ok(Outputs::new()
            .with("closure_error", c.closure_error)
            .with("total_length", c.total_length)
            .with_list("corrections", c.corrections)
            .with_list("adjusted_elevations", c.adjusted))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Reference case C4 (NAVEDTRA 14069, Fig. 14-14), in feet; the arithmetic is unit-agnostic.
    const START: f64 = 133.163;
    const BS: [f64; 5] = [6.659, 4.968, 4.508, 1.412, 7.073];
    const FS: [f64; 5] = [4.971, 7.452, 5.857, 1.261, 1.785];
    const KNOWN: f64 = 136.442;
    const SECTIONS: [f64; 5] = [440.0, 500.0, 620.0, 200.0, 380.0];

    // The book's printed values.
    const BOOK_HI: [f64; 5] = [139.822, 139.819, 136.875, 132.430, 138.242];
    const BOOK_Z: [f64; 5] = [134.851, 132.367, 131.018, 131.169, 136.457];

    // Exact rational re-derivation (outside the repository, 2026-11-26), rounded to five places.
    const EXACT_RISES: [f64; 5] = [1.688, 0.0, 0.0, 0.151, 5.288];
    const EXACT_FALLS: [f64; 5] = [0.0, 2.484, 1.349, 0.0, 0.0];
    const EXACT_CORR: [f64; 5] = [-0.00308, -0.00659, -0.01093, -0.01234, -0.01500];
    const EXACT_ADJ: [f64; 5] = [134.84792, 132.36041, 131.00707, 131.15666, 136.44200];

    fn assert_close(got: &[f64], expected: &[f64], tol: f64, what: &str) {
        assert_eq!(got.len(), expected.len(), "{what}");
        for (k, (g, e)) in got.iter().zip(expected).enumerate() {
            assert!((g - e).abs() <= tol, "{what}[{k}]: got {g}, expected {e}");
        }
    }

    #[test]
    fn hi_reproduces_the_books_field_book_to_its_last_printed_digit() {
        let (hi, z) = hi_reduction(START, &BS, &FS).expect("c4");
        assert_close(&hi, &BOOK_HI, 1e-9, "HI");
        assert_close(&z, &BOOK_Z, 1e-9, "elevation");
        let (sb, sf, net) = sums(&BS, &FS).expect("sums");
        assert!(
            (sb - 24.620).abs() < 1e-9 && (sf - 21.326).abs() < 1e-9 && (net - 3.294).abs() < 1e-9
        );
        assert!(
            (z[4] - START - net).abs() < 1e-9,
            "last − start = ΣBS − ΣFS"
        );
    }

    #[test]
    fn rise_and_fall_gives_the_same_elevations_on_a_backsight_foresight_field_book() {
        let (rises, falls, z) = rf_reduction(START, &BS, &FS).expect("c4");
        assert_close(&z, &BOOK_Z, 1e-9, "elevation");
        assert_close(&rises, &EXACT_RISES, 1e-9, "rises");
        assert_close(&falls, &EXACT_FALLS, 1e-9, "falls");
    }

    #[test]
    fn the_single_setup_example_c9_uses_the_backsight_the_book_computed_with() {
        // The text prints 11.65 but computes 365.01 + 11.56 = 376.57.
        let (hi, z) = hi_reduction(365.01, &[11.56], &[1.42]).expect("c9");
        assert!((hi[0] - 376.57).abs() < 1e-9 && (z[0] - 375.15).abs() < 1e-9);
        let (_, wrong) = hi_reduction(365.01, &[11.65], &[1.42]).expect("printed value");
        assert!(
            (wrong[0] - 375.15).abs() > 0.05,
            "the printed 11.65 would not give the book's answer"
        );
    }

    #[test]
    fn closure_and_adjustment_match_the_book_and_the_exact_rational_values() {
        let c = closure_adjustment(&BOOK_Z, &SECTIONS, KNOWN).expect("c4");
        assert!(
            (c.closure_error - 0.015).abs() < 1e-9,
            "the book: +0.015 ft"
        );
        assert!((c.total_length - 2140.0).abs() < 1e-9, "the book: 2,140 ft");
        assert!(
            (c.adjusted[0] - 134.848).abs() < 5e-4,
            "the book prints 134.848 for TBM 16"
        );
        assert_close(&c.corrections, &EXACT_CORR, 6e-6, "correction");
        assert_close(&c.adjusted, &EXACT_ADJ, 6e-6, "adjusted elevation");
        assert!(
            (c.adjusted[4] - KNOWN).abs() < 1e-12,
            "the last point lands on the known elevation"
        );
    }

    #[test]
    fn a_perfect_closure_changes_nothing() {
        let c = closure_adjustment(&[1.0, 2.0], &[3.0, 4.0], 2.0).expect("closed");
        assert_eq!(c.closure_error, 0.0);
        assert!(c.corrections.iter().all(|x| *x == 0.0));
        assert_eq!(c.adjusted, [1.0, 2.0]);
    }

    #[test]
    fn a_single_point_takes_the_whole_correction() {
        let c = closure_adjustment(&[10.03], &[250.0], 10.0).expect("one point");
        assert!((c.corrections[0] + 0.03).abs() < 1e-12);
        assert!((c.adjusted[0] - 10.0).abs() < 1e-12);
    }

    #[test]
    fn overflowing_values_are_out_of_domain_errors() {
        let e = closure_adjustment(&[1.0, 2.0], &[1.0e308, 1.0e308], 2.0).expect_err("length");
        assert_eq!(
            (e.code, e.input.as_deref()),
            (ErrorCode::OutOfDomain, Some("section_lengths"))
        );
        let e = closure_adjustment(&[1.0e308], &[1.0], -1.0e308).expect_err("error");
        assert_eq!(
            (e.code, e.input.as_deref()),
            (ErrorCode::OutOfDomain, Some("closing_elevation"))
        );
        let e = hi_reduction(1.0e308, &[1.0e308], &[0.0]).expect_err("hi");
        assert_eq!((e.code, e.index), (ErrorCode::OutOfDomain, Some(0)));
        let e = sums(&[1.0e308, 1.0e308], &[0.0, 0.0]).expect_err("sum");
        assert_eq!(e.code, ErrorCode::OutOfDomain);
    }
}
