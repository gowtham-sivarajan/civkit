// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-25 23:56 IST · last edited 2026-11-26 00:00 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! Surveying calculations for 0.1 (D-20, ADR 0009).
//!
//! `surveying.traverse.compass_rule`: a closed traverse from bearings and distances to latitudes, departures,
//! misclosure and compass-rule (Bowditch) adjusted latitudes and departures. The descriptor is the one the
//! maintainer accepted in `planning/TRAVERSE_DESCRIPTOR_PROPOSAL.md`. Not in 0.1: adjusted coordinates, area by
//! coordinates, open traverses, angular balancing, other adjustment methods, the direction of the misclosure.

use crate::{
    CalcError, Calculation, Context, Descriptor, ErrorCode, Group, InputKind, InputSpec, Inputs,
    ListLength, OutputKind, OutputSpec, Outputs, Range,
};
use civkit_geometry::components;
use civkit_units::Dimension;
use std::sync::LazyLock;

/// The group of traverse legs, and the word that numbers them in messages and the trace.
const LEGS: &str = "legs";
const LEG: &str = "leg";

fn list_output(name: &str, description: &str) -> OutputSpec {
    OutputSpec {
        name: name.into(),
        description: description.into(),
        dimension: Dimension::LENGTH,
        si_unit: Some("m".into()),
        display_unit: Some("m".into()),
        kind: OutputKind::List,
        length: Some(ListLength {
            group: LEGS.into(),
            offset: 0,
            item: LEG.into(),
        }),
    }
}

fn length_output(name: &str, description: &str) -> OutputSpec {
    OutputSpec {
        name: name.into(),
        description: description.into(),
        dimension: Dimension::LENGTH,
        si_unit: Some("m".into()),
        display_unit: Some("m".into()),
        ..Default::default()
    }
}

/// Closed traverse with compass-rule (Bowditch) adjustment.
#[derive(Debug, Default)]
pub struct TraverseCompassRule;

static TRAVERSE: LazyLock<Descriptor> = LazyLock::new(|| {
    Descriptor {
    id: "surveying.traverse.compass_rule".into(),
    version: 1,
    title: "Closed traverse: misclosure and compass-rule (Bowditch) adjustment".into(),
    summary: "From the azimuth and horizontal distance of each leg of a closed traverse: the latitude \
              (d·cos α) and departure (d·sin α) of every leg, the misclosure of the loop, the relative \
              misclosure E/P (the precision ratio is 1 : P/E), and the compass-rule corrections, which share \
              the misclosure out in proportion to leg length, and the adjusted latitudes and departures. A \
              correction is the negative of the misclosure shared by length; an adjusted value is the computed \
              value plus its correction."
        .into(),
    inputs: vec![
        InputSpec {
            name: "bearings".into(),
            description: "Azimuth of each leg, clockwise from north, in order of measurement. Quadrant bearings \
                          (unit \"bearing\", e.g. \"N 3°45' E\") or decimal angles (deg, rad, grad). Lists of DMS \
                          values are not supported."
                .into(),
            dimension: Dimension::ANGLE,
            si_unit: Some("rad".into()),
            display_unit: Some("deg".into()),
            required: true,
            accepts_bearing: true,
            kind: InputKind::List,
            group: Some(LEGS.into()),
            ..Default::default()
        },
        InputSpec {
            name: "distances".into(),
            description: "Horizontal distance of each leg, in the same order as the bearings".into(),
            dimension: Dimension::LENGTH,
            si_unit: Some("m".into()),
            display_unit: Some("m".into()),
            required: true,
            domain: Range::greater_than(0.0),
            kind: InputKind::List,
            group: Some(LEGS.into()),
            ..Default::default()
        },
    ],
    groups: vec![Group {
        name: LEGS.into(),
        item: LEG.into(),
        min_len: 3,
        description: "Traverse legs in order of measurement. Leg k runs from station k to station k + 1; the \
                      last leg ends at station 1."
            .into(),
    }],
    outputs: vec![
        list_output("latitudes", "Latitude of each leg, d·cos α (northing positive)"),
        list_output("departures", "Departure of each leg, d·sin α (easting positive)"),
        list_output(
            "latitude_corrections",
            "Compass-rule correction to each latitude, −M_N·d/P",
        ),
        list_output(
            "departure_corrections",
            "Compass-rule correction to each departure, −M_E·d/P",
        ),
        list_output("adjusted_latitudes", "Latitude of each leg plus its correction"),
        list_output("adjusted_departures", "Departure of each leg plus its correction"),
        length_output("perimeter", "Total length of the traverse, P = Σd"),
        length_output(
            "latitude_misclosure",
            "Sum of the latitudes, M_N (signed; zero for a perfect closure)",
        ),
        length_output(
            "departure_misclosure",
            "Sum of the departures, M_E (signed; zero for a perfect closure)",
        ),
        length_output(
            "linear_misclosure",
            "Distance by which the traverse fails to close, E = √(M_N² + M_E²)",
        ),
        OutputSpec {
            name: "relative_misclosure".into(),
            description: "Linear misclosure divided by the perimeter, E/P (0 for a perfect closure). The \
                          precision ratio is 1 : N with N = P/E, which is infinite for a perfect closure and \
                          so is not returned."
                .into(),
            dimension: Dimension::DIMENSIONLESS,
            ..Default::default()
        },
    ],
    references: vec![
        "US Navy, Engineering Aid, NAVEDTRA 14070, chapter 7 (traverse computation and the compass rule), \
         Figs 7-12 to 7-14, pp. 7-10 to 7-11. Public domain."
            .into(),
    ],
    assumptions: vec![
        "The traverse is closed: the last leg ends at the first station. Open (link) traverses are not supported."
            .into(),
        "The bearings are already balanced; angular misclosure is outside this calculation.".into(),
        "Plane grid, no geodesy: distances are horizontal and measured on the grid.".into(),
        "The compass rule shares the misclosure out in proportion to leg length; it does not model the sources \
         of error."
            .into(),
    ],
    validity: Vec::new(),
    examples: vec![
        "tests/reference/surveying.traverse.compass_rule/navedtra-14070-fig7-12.json".into(),
    ],
}
});

/// The result of applying the compass rule to a set of latitudes and departures. Lengths in the unit of the input.
#[derive(Debug, Clone, PartialEq)]
struct Adjustment {
    perimeter: f64,
    latitude_misclosure: f64,
    departure_misclosure: f64,
    linear_misclosure: f64,
    relative_misclosure: f64,
    latitude_corrections: Vec<f64>,
    departure_corrections: Vec<f64>,
    adjusted_latitudes: Vec<f64>,
    adjusted_departures: Vec<f64>,
}

/// The compass (Bowditch) rule: corrections −M·d/P shared out by leg length, added to each latitude and departure.
///
/// Pure arithmetic, so that it can be tested against the book's own printed latitudes and departures.
/// The three slices have the same length and `distances` are positive (the registry has checked both).
fn compass_rule(
    latitudes: &[f64],
    departures: &[f64],
    distances: &[f64],
) -> Result<Adjustment, CalcError> {
    if latitudes.len() != distances.len() || departures.len() != distances.len() {
        return Err(CalcError::internal("traverse lists differ in length"));
    }
    let perimeter: f64 = distances.iter().sum();
    if !perimeter.is_finite() {
        return Err(CalcError::for_input(
            ErrorCode::OutOfDomain,
            "distances",
            "the perimeter is too large to compute (the distances add up to more than a finite number)",
        ));
    }
    if perimeter <= 0.0 {
        return Err(CalcError::internal("traverse perimeter is not positive"));
    }
    let m_n: f64 = latitudes.iter().sum();
    let m_e: f64 = departures.iter().sum();
    let linear = m_n.hypot(m_e);
    // d / P first: it is at most 1, so the product can never overflow.
    let share = |d: f64| d / perimeter;
    let c_n: Vec<f64> = distances.iter().map(|d| -m_n * share(*d)).collect();
    let c_e: Vec<f64> = distances.iter().map(|d| -m_e * share(*d)).collect();
    let adj_n = latitudes.iter().zip(&c_n).map(|(a, c)| a + c).collect();
    let adj_e = departures.iter().zip(&c_e).map(|(a, c)| a + c).collect();
    Ok(Adjustment {
        perimeter,
        latitude_misclosure: m_n,
        departure_misclosure: m_e,
        linear_misclosure: linear,
        relative_misclosure: linear / perimeter,
        latitude_corrections: c_n,
        departure_corrections: c_e,
        adjusted_latitudes: adj_n,
        adjusted_departures: adj_e,
    })
}

impl Calculation for TraverseCompassRule {
    fn descriptor(&self) -> &Descriptor {
        &TRAVERSE
    }

    fn compute(&self, inputs: &Inputs, ctx: &mut Context) -> Result<Outputs, CalcError> {
        let azimuths = inputs.list("bearings")?;
        let distances = inputs.list("distances")?;

        // 1. Latitude and departure of every leg (GEOMETRY.md §4: `components` returns (Δx, Δy) = (Dep, Lat)).
        let mut latitudes = Vec::with_capacity(distances.len());
        let mut departures = Vec::with_capacity(distances.len());
        for (k, (alpha, d)) in azimuths.iter().zip(distances).enumerate() {
            let (dep, lat) = components(*alpha, *d)
                .map_err(|e| CalcError::internal(format!("{LEG} {}: {e}", k + 1)))?;
            ctx.step_item(LEG, k, "Lat = d·cos α", lat, Some("m"));
            ctx.step_item(LEG, k, "Dep = d·sin α", dep, Some("m"));
            latitudes.push(lat);
            departures.push(dep);
        }

        // 2. Misclosure and the compass rule.
        let a = compass_rule(&latitudes, &departures, distances)?;
        ctx.step("P = Σd", a.perimeter, Some("m"));
        ctx.step("M_N = ΣLat", a.latitude_misclosure, Some("m"));
        ctx.step("M_E = ΣDep", a.departure_misclosure, Some("m"));
        ctx.step("E = √(M_N² + M_E²)", a.linear_misclosure, Some("m"));
        ctx.step("E/P", a.relative_misclosure, None);
        for k in 0..distances.len() {
            ctx.step_item(
                LEG,
                k,
                "c_N = −M_N·d/P",
                a.latitude_corrections[k],
                Some("m"),
            );
            ctx.step_item(
                LEG,
                k,
                "c_E = −M_E·d/P",
                a.departure_corrections[k],
                Some("m"),
            );
        }
        for k in 0..distances.len() {
            ctx.step_item(
                LEG,
                k,
                "Lat′ = Lat + c_N",
                a.adjusted_latitudes[k],
                Some("m"),
            );
            ctx.step_item(
                LEG,
                k,
                "Dep′ = Dep + c_E",
                a.adjusted_departures[k],
                Some("m"),
            );
        }

        Ok(Outputs::new()
            .with_list("latitudes", latitudes)
            .with_list("departures", departures)
            .with_list("latitude_corrections", a.latitude_corrections)
            .with_list("departure_corrections", a.departure_corrections)
            .with_list("adjusted_latitudes", a.adjusted_latitudes)
            .with_list("adjusted_departures", a.adjusted_departures)
            .with("perimeter", a.perimeter)
            .with("latitude_misclosure", a.latitude_misclosure)
            .with("departure_misclosure", a.departure_misclosure)
            .with("linear_misclosure", a.linear_misclosure)
            .with("relative_misclosure", a.relative_misclosure))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The printed numbers of reference case C3 (NAVEDTRA 14070, Figs 7-12 to 7-14), in feet. Used here as
    // *inputs to the adjustment only*: the contract takes bearings, not latitudes, so the exact arithmetic of the
    // compass rule is checked against the book's own table, free of any trigonometry.
    const DIST: [f64; 6] = [584.21, 720.26, 212.00, 292.31, 278.53, 527.54];
    const LAT: [f64; 6] = [582.95, -47.10, -208.23, -246.07, 202.09, -283.31];
    const DEP: [f64; 6] = [38.20, 718.72, 39.77, -157.77, -191.67, -445.01];
    const C_LAT: [f64; 6] = [-0.07, -0.09, -0.03, -0.04, -0.03, -0.07];
    const C_DEP: [f64; 6] = [-0.50, -0.62, -0.18, -0.25, -0.24, -0.45];
    const ADJ_LAT: [f64; 6] = [582.88, -47.19, -208.26, -246.11, 202.06, -283.38];
    const ADJ_DEP: [f64; 6] = [37.70, 718.10, 39.59, -158.02, -191.91, -445.46];

    /// The book prints to 0.01 ft; its corrections are rounded before adding, so the worst difference
    /// (0.00515 ft, re-derived outside the repository) stays below this.
    const BOOK_ROUNDING: f64 = 0.006;

    fn assert_close(got: &[f64], book: &[f64], tol: f64, what: &str) {
        for (k, (g, b)) in got.iter().zip(book).enumerate() {
            assert!(
                (g - b).abs() <= tol,
                "{what}, leg {}: got {g}, book {b}",
                k + 1
            );
        }
    }

    #[test]
    fn the_compass_rule_reproduces_the_books_table_from_the_books_own_latitudes_and_departures() {
        let a = compass_rule(&LAT, &DEP, &DIST).expect("book data");
        assert_close(
            &a.latitude_corrections,
            &C_LAT,
            BOOK_ROUNDING,
            "latitude correction",
        );
        assert_close(
            &a.departure_corrections,
            &C_DEP,
            BOOK_ROUNDING,
            "departure correction",
        );
        assert_close(
            &a.adjusted_latitudes,
            &ADJ_LAT,
            BOOK_ROUNDING,
            "adjusted latitude",
        );
        assert_close(
            &a.adjusted_departures,
            &ADJ_DEP,
            BOOK_ROUNDING,
            "adjusted departure",
        );
    }

    #[test]
    fn the_books_sums_and_perimeter() {
        let a = compass_rule(&LAT, &DEP, &DIST).expect("book data");
        assert!((a.perimeter - 2614.85).abs() < 1e-9);
        // Σ of the printed latitudes and departures is exactly 0.33 and 2.24 (the book's Σ row).
        assert!((a.latitude_misclosure - 0.33).abs() < 1e-9);
        assert!((a.departure_misclosure - 2.24).abs() < 1e-9);
        // E = √(0.33² + 2.24²); the book prints 2.26 for the same sums.
        assert!((a.linear_misclosure - 0.33_f64.hypot(2.24)).abs() < 1e-12);
        assert!((a.linear_misclosure - 2.26).abs() < 0.005);
        assert!((a.relative_misclosure - a.linear_misclosure / 2614.85).abs() < 1e-15);
    }

    #[test]
    fn adjusted_values_sum_to_zero_exactly_enough() {
        let a = compass_rule(&LAT, &DEP, &DIST).expect("book data");
        assert!(a.adjusted_latitudes.iter().sum::<f64>().abs() < 1e-12);
        assert!(a.adjusted_departures.iter().sum::<f64>().abs() < 1e-12);
        // And the corrections sum to minus the misclosure.
        assert!((a.latitude_corrections.iter().sum::<f64>() + a.latitude_misclosure).abs() < 1e-12);
        assert!(
            (a.departure_corrections.iter().sum::<f64>() + a.departure_misclosure).abs() < 1e-12
        );
    }

    #[test]
    fn a_closed_traverse_gets_no_correction_and_a_zero_ratio() {
        let a =
            compass_rule(&[3.0, 4.0, -7.0], &[4.0, -4.0, 0.0], &[5.0, 5.66, 7.0]).expect("closed");
        assert_eq!(a.linear_misclosure, 0.0);
        assert_eq!(a.relative_misclosure, 0.0);
        assert!(a.latitude_corrections.iter().all(|c| *c == 0.0));
        assert_eq!(a.adjusted_latitudes, [3.0, 4.0, -7.0]);
    }

    #[test]
    fn corrections_are_proportional_to_length_and_opposite_in_sign() {
        // Two equal legs and one double leg, misclosure (+3, −6): corrections −3·d/P and +6·d/P.
        let a =
            compass_rule(&[1.0, 1.0, 1.0], &[-2.0, -2.0, -2.0], &[10.0, 10.0, 20.0]).expect("legs");
        assert_eq!(a.perimeter, 40.0);
        assert_eq!(a.latitude_misclosure, 3.0);
        assert_eq!(a.departure_misclosure, -6.0);
        assert_eq!(a.latitude_corrections, [-0.75, -0.75, -1.5]);
        assert_eq!(a.departure_corrections, [1.5, 1.5, 3.0]);
        assert_eq!(a.adjusted_latitudes, [0.25, 0.25, -0.5]);
        assert_eq!(a.adjusted_departures, [-0.5, -0.5, 1.0]);
    }

    #[test]
    fn an_overflowing_perimeter_is_an_out_of_domain_error() {
        let e = compass_rule(&[0.0; 3], &[0.0; 3], &[1.0e308; 3]).expect_err("overflow");
        assert_eq!(e.code, ErrorCode::OutOfDomain);
        assert_eq!(e.input.as_deref(), Some("distances"));
    }

    #[test]
    fn a_huge_but_finite_traverse_does_not_overflow_in_the_corrections() {
        // d·M would be 1e400 without sharing d/P first.
        let a =
            compass_rule(&[1.0e200, 1.0e200, -1.0e200], &[0.0; 3], &[1.0e200; 3]).expect("huge");
        assert!(a.latitude_corrections.iter().all(|c| c.is_finite()));
        assert!(a.adjusted_latitudes.iter().all(|c| c.is_finite()));
    }
}
