// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 18:37 IST · last edited 2026-11-25 18:37 IST

//! Lengths of polylines, arcs and circles (docs/GEOMETRY.md §9, §8.8).

use std::f64::consts::{FRAC_PI_2, PI, TAU};

use crate::angle::wrap;
use crate::bulge::{BulgeArc, arc_length as bulge_arc_length};
use crate::error::{GeometryError, Note};
use crate::point::{Point, Vertex};
use crate::tolerance::{BoundingBox, Sum, Tolerance, magnitude};

/// A measured value with the tolerance used and the degeneracies handled.
#[derive(Debug, Clone, PartialEq)]
pub struct Measure {
    /// The value, in the input's length unit (or its square for areas).
    pub value: f64,
    /// The tolerance used.
    pub tolerance: Tolerance,
    /// Degeneracies handled (§8).
    pub notes: Vec<Note>,
}

fn with_precision_note(mut notes: Vec<Note>, t: &Tolerance) -> Vec<Note> {
    if t.precision_limited() {
        notes.push(Note::PrecisionLimited);
    }
    notes
}

fn finite_result(value: f64) -> Result<f64, GeometryError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(GeometryError::Overflow)
    }
}

/// Length of a polyline with straight and bulged segments (§9). A closed polyline includes the
/// closing segment with the last vertex's bulge; an open one does not use that bulge (it must
/// still be finite).
///
/// # Errors
/// [`GeometryError::NonFinite`], [`GeometryError::Overflow`], [`GeometryError::InvalidTolerance`],
/// [`GeometryError::TooFewVertices`] (fewer than 2), [`GeometryError::IllConditionedBulge`].
pub fn polyline_length(
    vertices: &[Vertex],
    closed: bool,
    tolerance: Option<f64>,
) -> Result<Measure, GeometryError> {
    if !vertices
        .iter()
        .all(|v| v.point.is_finite() && v.bulge.is_finite())
    {
        return Err(GeometryError::NonFinite);
    }
    let n = vertices.len();
    if n < 2 {
        return Err(GeometryError::TooFewVertices {
            found: n,
            required: 2,
        });
    }
    let origin = vertices[0].point;
    let points: Vec<Point> = vertices.iter().map(|v| v.point.minus(origin)).collect();
    if !points.iter().all(|p| p.is_finite()) {
        return Err(GeometryError::Overflow);
    }
    let segments = if closed { n } else { n - 1 };
    let ends = |i: usize| (points[i], points[(i + 1) % n], vertices[i].bulge);

    let mut bbox = BoundingBox::new();
    points.iter().for_each(|p| bbox.add(*p));
    for i in 0..segments {
        let (p, q, b) = ends(i);
        if b != 0.0 && p.distance(q) > 0.0 {
            BulgeArc::new(p, q, b).extremes().for_each(|e| bbox.add(e));
        }
    }
    let t = Tolerance::compute(
        bbox.diagonal()?,
        magnitude(vertices.iter().map(|v| v.point)),
        tolerance,
    )?;

    let mut sum = Sum::default();
    let mut notes = Vec::new();
    for i in 0..segments {
        let (p, q, b) = ends(i);
        let c = p.distance(q);
        let s = finite_result(bulge_arc_length(c, b))?;
        if c <= t.linear && s > t.linear {
            return Err(GeometryError::IllConditionedBulge { segment: i });
        }
        if s <= t.linear && !(closed && i == n - 1) {
            notes.push(Note::ZeroLengthSegment { segment: i });
        }
        sum.add(s);
    }
    Ok(Measure {
        value: sum.value()?,
        notes: with_precision_note(notes, &t),
        tolerance: t,
    })
}

/// A circular arc: centre, radius r ≥ 0, start direction θ₀ and signed sweep Δ (§4).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Arc {
    /// Centre.
    pub centre: Point,
    /// Radius.
    pub radius: f64,
    /// Start direction θ₀, counter-clockwise from +x, in radians.
    pub start: f64,
    /// Sweep Δ in radians: positive counter-clockwise, |Δ| < 2π.
    pub sweep: f64,
}

impl Arc {
    /// An arc from centre, radius, start direction and sweep.
    #[must_use]
    pub const fn new(centre: Point, radius: f64, start: f64, sweep: f64) -> Self {
        Self {
            centre,
            radius,
            start,
            sweep,
        }
    }
}

/// A circle: centre and radius.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle {
    /// Centre.
    pub centre: Point,
    /// Radius.
    pub radius: f64,
}

impl Circle {
    /// A circle from centre and radius.
    #[must_use]
    pub const fn new(centre: Point, radius: f64) -> Self {
        Self { centre, radius }
    }
}

fn on_circle(c: Point, r: f64, theta: f64) -> Point {
    Point::new(c.x + r * theta.cos(), c.y + r * theta.sin())
}

/// Length of an arc, r·|Δ| (§9). Domain rules in §8.8.
///
/// # Errors
/// [`GeometryError::NonFinite`], [`GeometryError::NegativeRadius`], [`GeometryError::SweepOutOfRange`],
/// [`GeometryError::InvalidTolerance`], [`GeometryError::RadiusTooSmall`], [`GeometryError::Overflow`].
pub fn arc_length(arc: &Arc, tolerance: Option<f64>) -> Result<Measure, GeometryError> {
    let Arc {
        centre,
        radius: r,
        start,
        sweep,
    } = *arc;
    if !(centre.is_finite() && r.is_finite() && start.is_finite() && sweep.is_finite()) {
        return Err(GeometryError::NonFinite);
    }
    if r < 0.0 {
        return Err(GeometryError::NegativeRadius);
    }
    if sweep.abs() >= TAU {
        return Err(GeometryError::SweepOutOfRange);
    }
    // Bounding box: endpoints and the axis extremes inside the sweep (§5).
    let mut points = vec![
        centre,
        on_circle(centre, r, start),
        on_circle(centre, r, start + sweep),
    ];
    for phi in [0.0, FRAC_PI_2, PI, 3.0 * FRAC_PI_2] {
        let rel = if sweep >= 0.0 {
            wrap(phi - start)
        } else {
            wrap(start - phi)
        };
        if rel <= sweep.abs() {
            points.push(on_circle(centre, r, phi));
        }
    }
    let mut bbox = BoundingBox::new();
    points[1..].iter().for_each(|p| bbox.add(*p));
    let t = Tolerance::compute(
        bbox.diagonal()?,
        magnitude(points.iter().copied()),
        tolerance,
    )?;
    if sweep == 0.0 {
        return Ok(Measure {
            value: 0.0,
            notes: with_precision_note(vec![Note::ZeroSweep], &t),
            tolerance: t,
        });
    }
    if r <= t.linear {
        return Err(GeometryError::RadiusTooSmall);
    }
    Ok(Measure {
        value: finite_result(r * sweep.abs())?,
        notes: with_precision_note(Vec::new(), &t),
        tolerance: t,
    })
}

fn circle_tolerance(circle: &Circle, tolerance: Option<f64>) -> Result<Tolerance, GeometryError> {
    let Circle { centre, radius: r } = *circle;
    if !(centre.is_finite() && r.is_finite()) {
        return Err(GeometryError::NonFinite);
    }
    if r < 0.0 {
        return Err(GeometryError::NegativeRadius);
    }
    let corners = [
        Point::new(centre.x - r, centre.y - r),
        Point::new(centre.x + r, centre.y + r),
    ];
    let mut bbox = BoundingBox::new();
    corners.iter().for_each(|p| bbox.add(*p));
    let t = Tolerance::compute(
        bbox.diagonal()?,
        magnitude([centre, corners[0], corners[1]]),
        tolerance,
    )?;
    if r <= t.linear {
        return Err(GeometryError::RadiusTooSmall);
    }
    Ok(t)
}

/// Circumference of a circle, 2πr (§9).
///
/// # Errors
/// [`GeometryError::NonFinite`], [`GeometryError::NegativeRadius`], [`GeometryError::InvalidTolerance`],
/// [`GeometryError::RadiusTooSmall`], [`GeometryError::Overflow`].
pub fn circle_circumference(
    circle: &Circle,
    tolerance: Option<f64>,
) -> Result<Measure, GeometryError> {
    let t = circle_tolerance(circle, tolerance)?;
    Ok(Measure {
        value: finite_result(TAU * circle.radius)?,
        notes: with_precision_note(Vec::new(), &t),
        tolerance: t,
    })
}

/// Area of a circle, πr² (§10).
///
/// # Errors
/// As [`circle_circumference`].
pub fn circle_area(circle: &Circle, tolerance: Option<f64>) -> Result<Measure, GeometryError> {
    let t = circle_tolerance(circle, tolerance)?;
    let r = circle.radius;
    Ok(Measure {
        value: finite_result(PI * r * r)?,
        notes: with_precision_note(Vec::new(), &t),
        tolerance: t,
    })
}
