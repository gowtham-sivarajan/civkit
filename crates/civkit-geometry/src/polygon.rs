// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 18:37 IST · last edited 2026-11-27 20:24 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! Polygon area, orientation and centroid (docs/GEOMETRY.md §8, §10).

use crate::bulge::{BulgeArc, arc_length, segment_area};
use crate::edges::{Edge, conflict};
use crate::error::{GeometryError, Note};
use crate::point::{Point, Vertex};
use crate::sweep::{EdgeBox, WorkLimit, overlapping_pairs};
use crate::tolerance::{BoundingBox, Sum, Tolerance, magnitude, ulp};

/// Orientation of a ring (§10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    /// Signed area > ε_A: East → North, or right → up.
    CounterClockwise,
    /// Signed area < −ε_A.
    Clockwise,
    /// Degenerate ring (§8.6) or |signed area| ≤ ε_A.
    Undefined,
}

/// Area of a ring (§10).
#[derive(Debug, Clone, PartialEq)]
pub struct PolygonArea {
    /// |A|, as computed (never rounded to 0, D8).
    pub area: f64,
    /// A: positive for a counter-clockwise ring.
    pub signed_area: f64,
    /// Orientation from A against ε_A.
    pub orientation: Orientation,
    /// The tolerance used.
    pub tolerance: Tolerance,
    /// Degeneracies handled (§8).
    pub notes: Vec<Note>,
}

/// Centroid of a straight-edged ring (§10).
#[derive(Debug, Clone, PartialEq)]
pub struct Centroid {
    /// The centroid.
    pub point: Point,
    /// The signed area from the same sum.
    pub signed_area: f64,
    /// The tolerance used.
    pub tolerance: Tolerance,
    /// Degeneracies handled (§8).
    pub notes: Vec<Note>,
}

struct Prepared {
    tolerance: Tolerance,
    origin: Point,
    points: Vec<Point>,
    signed_area: f64,
    orientation: Orientation,
    degenerate: bool,
    notes: Vec<Note>,
}

/// Chord and arc length of the segment from `p` to `q` with bulge `b`.
fn segment_lengths(p: Point, q: Point, b: f64) -> (f64, f64) {
    let c = p.distance(q);
    (c, arc_length(c, b))
}

fn is_zero_length(p: Point, q: Point, b: f64, eps: f64) -> bool {
    let (c, s) = segment_lengths(p, q, b);
    c <= eps && s <= eps
}

/// Cleaning (§10): repeatedly drop the start vertex of the first zero-length segment, in ring
/// order, until none remains. Done in one linear pass with the same result **(D-28)**: every
/// segment before the scan position is known not to be zero-length; dropping a vertex changes only
/// its predecessor's segment, so the scan steps back one vertex (or, when the first vertex is
/// dropped, restarts at the new first vertex, the only one before which nothing is known). Each
/// step either advances or drops a vertex, so there are at most 2n steps.
fn clean(ring: Vec<(usize, Point, f64)>, eps: f64) -> Vec<(usize, Point, f64)> {
    let n = ring.len();
    if n == 0 {
        return ring;
    }
    let mut next: Vec<usize> = (0..n).map(|i| (i + 1) % n).collect();
    let mut prev: Vec<usize> = (0..n).map(|i| (i + n - 1) % n).collect();
    let mut head = 0;
    let mut alive = n;
    let mut at = head;
    loop {
        let after = next[at];
        if is_zero_length(ring[at].1, ring[after].1, ring[at].2, eps) {
            if alive == 1 {
                return Vec::new();
            }
            let before = prev[at];
            next[before] = after;
            prev[after] = before;
            alive -= 1;
            if at == head {
                head = after;
                at = head;
            } else {
                at = before;
            }
        } else if after == head {
            break;
        } else {
            at = after;
        }
    }
    let mut out = Vec::with_capacity(alive);
    let mut k = head;
    for _ in 0..alive {
        out.push(ring[k]);
        k = next[k];
    }
    out
}

/// The steps of §8 in the order of §8 ("Order of evaluation for a polygon").
fn prepare(
    vertices: &[Vertex],
    tolerance: Option<f64>,
    for_centroid: bool,
    work: &mut WorkLimit,
) -> Result<Prepared, GeometryError> {
    // 8.1 non-finite input.
    if !vertices
        .iter()
        .all(|v| v.point.is_finite() && v.bulge.is_finite())
    {
        return Err(GeometryError::NonFinite);
    }
    if for_centroid && vertices.iter().any(|v| v.bulge != 0.0) {
        return Err(GeometryError::CurvedCentroidUnsupported);
    }
    let n = vertices.len();
    if n == 0 {
        return Err(GeometryError::TooFewVertices {
            found: 0,
            required: 3,
        });
    }

    // Translate to the first vertex (§10).
    let origin = vertices[0].point;
    let points: Vec<Point> = vertices.iter().map(|v| v.point.minus(origin)).collect();
    if !points.iter().all(|p| p.is_finite()) {
        return Err(GeometryError::Overflow);
    }
    let bulges: Vec<f64> = vertices.iter().map(|v| v.bulge).collect();
    let next = |i: usize| (i + 1) % n;

    // Tolerance (§5): D from the vertices and arc extremes; M from the original coordinates.
    let mut bbox = BoundingBox::new();
    for (i, p) in points.iter().enumerate() {
        bbox.add(*p);
        let q = points[next(i)];
        if bulges[i] != 0.0 && p.distance(q) > 0.0 {
            BulgeArc::new(*p, q, bulges[i])
                .extremes()
                .for_each(|e| bbox.add(e));
        }
    }
    let t = Tolerance::compute(
        bbox.diagonal()?,
        magnitude(vertices.iter().map(|v| v.point)),
        tolerance,
    )?;
    let eps = t.linear;

    // Signed area (8.1: an overflowing result is an error).
    let mut sum = Sum::default();
    for i in 0..n {
        let (p, q) = (points[i], points[next(i)]);
        sum.add(0.5 * (p.x * q.y - q.x * p.y));
        sum.add(segment_area(p.distance(q), bulges[i]));
    }
    let signed_area = sum.value()?;

    // 8.2a ill-conditioned bulges; 8.2 zero-length segments (the closing duplicate is silent, 8.4).
    let mut notes = Vec::new();
    for i in 0..n {
        let (c, s) = segment_lengths(points[i], points[next(i)], bulges[i]);
        if !s.is_finite() {
            return Err(GeometryError::Overflow);
        }
        if c <= eps && s > eps {
            return Err(GeometryError::IllConditionedBulge { segment: i });
        }
        if s <= eps && i != n - 1 {
            notes.push(Note::ZeroLengthSegment { segment: i });
        }
    }

    // Cleaning (§10): drop the start vertex of each zero-length segment until none remains.
    let ring = clean((0..n).map(|i| (i, points[i], bulges[i])).collect(), eps);

    // 8.5 too few vertices.
    let m = ring.len();
    let required = if ring.iter().any(|v| v.2 != 0.0) {
        2
    } else {
        3
    };
    if m < required {
        return Err(GeometryError::TooFewVertices { found: m, required });
    }

    // 8.6 degenerate ring: straight or flat (sagitta ≤ ε) edges, all vertices within ε of one line.
    let flat = (0..m).all(|i| {
        let (_, p, b) = ring[i];
        b == 0.0 || b.abs() * p.distance(ring[(i + 1) % m].1) / 2.0 <= eps
    });
    let degenerate = flat && {
        let v0 = ring[0].1;
        let far = ring.iter().map(|v| v.1).fold(v0, |f, p| {
            if p.distance(v0) > f.distance(v0) {
                p
            } else {
                f
            }
        });
        let len = far.distance(v0);
        ring.iter().all(|v| {
            let d = v.1.minus(v0);
            let dir = far.minus(v0);
            ((dir.x * d.y - dir.y * d.x) / len).abs() <= eps
        })
    };

    let orientation = if degenerate {
        notes.push(Note::DegenerateRing);
        Orientation::Undefined
    } else {
        // 8.7 self-intersection.
        let representable = ulp(t.magnitude);
        let edges: Vec<Edge> = (0..m)
            .map(|i| {
                let (_, p, b) = ring[i];
                let q = ring[(i + 1) % m].1;
                if b != 0.0 && b.abs() * p.distance(q) / 2.0 > representable {
                    Edge::Arc(BulgeArc::new(p, q, b))
                } else {
                    Edge::Line(p, q)
                }
            })
            .collect();
        // Only pairs whose ε-widened boxes overlap can conflict (the sweep prefilter). The first
        // conflicting pair in (i, j) order is reported, as testing every pair in order would.
        let boxes: Vec<EdgeBox> = edges.iter().map(|e| EdgeBox::of(e, eps)).collect();
        let mut first: Option<(usize, usize)> = None;
        overlapping_pairs(
            &boxes,
            |_, _| false,
            work,
            |i, j| {
                if first.is_some_and(|f| f <= (i, j)) {
                    return;
                }
                let shared: Vec<Point> = if m == 2 {
                    vec![ring[0].1, ring[1].1]
                } else if j == i + 1 {
                    vec![ring[j].1]
                } else if i == 0 && j == m - 1 {
                    vec![ring[0].1]
                } else {
                    Vec::new()
                };
                if conflict(&edges[i], &edges[j], &shared, eps) {
                    first = Some((i, j));
                }
            },
        )?;
        if let Some((i, j)) = first {
            return Err(GeometryError::SelfIntersection {
                first: ring[i].0,
                second: ring[j].0,
            });
        }
        if signed_area > t.area {
            Orientation::CounterClockwise
        } else if signed_area < -t.area {
            Orientation::Clockwise
        } else {
            notes.push(Note::OrientationUndefined);
            Orientation::Undefined
        }
    };

    if t.precision_limited() {
        notes.push(Note::PrecisionLimited);
    }
    Ok(Prepared {
        tolerance: t,
        origin,
        points,
        signed_area,
        orientation,
        degenerate,
        notes,
    })
}

/// Area and orientation of a closed ring of vertices (implicitly closed; an explicit closing
/// duplicate is allowed, §8.4). See docs/GEOMETRY.md §10 and the order of evaluation in §8.
///
/// # Errors
/// [`GeometryError::NonFinite`], [`GeometryError::Overflow`], [`GeometryError::InvalidTolerance`],
/// [`GeometryError::IllConditionedBulge`], [`GeometryError::TooFewVertices`],
/// [`GeometryError::SelfIntersection`].
pub fn polygon_area(
    vertices: &[Vertex],
    tolerance: Option<f64>,
) -> Result<PolygonArea, GeometryError> {
    polygon_area_limited(vertices, tolerance, &mut WorkLimit::unlimited())
}

/// As [`polygon_area`], with the §8.7 check's edge-pair examinations drawn from `work`, so that a
/// caller can bound the time any ring takes (D-28).
///
/// # Errors
/// As [`polygon_area`], plus [`GeometryError::WorkLimitExceeded`].
pub fn polygon_area_limited(
    vertices: &[Vertex],
    tolerance: Option<f64>,
    work: &mut WorkLimit,
) -> Result<PolygonArea, GeometryError> {
    let p = prepare(vertices, tolerance, false, work)?;
    Ok(PolygonArea {
        area: p.signed_area.abs(),
        signed_area: p.signed_area,
        orientation: p.orientation,
        tolerance: p.tolerance,
        notes: p.notes,
    })
}

/// Centroid of a straight-edged ring (§10; curved rings are not supported in 0.1, D5).
///
/// # Errors
/// As [`polygon_area`], plus [`GeometryError::CurvedCentroidUnsupported`],
/// [`GeometryError::DegenerateRing`] and [`GeometryError::UndefinedOrientation`].
pub fn polygon_centroid(
    vertices: &[Vertex],
    tolerance: Option<f64>,
) -> Result<Centroid, GeometryError> {
    let p = prepare(vertices, tolerance, true, &mut WorkLimit::unlimited())?;
    if p.degenerate {
        return Err(GeometryError::DegenerateRing);
    }
    if p.orientation == Orientation::Undefined {
        return Err(GeometryError::UndefinedOrientation);
    }
    let n = p.points.len();
    let (mut a2, mut sx, mut sy) = (Sum::default(), Sum::default(), Sum::default());
    for i in 0..n {
        let (u, v) = (p.points[i], p.points[(i + 1) % n]);
        let cross = u.x * v.y - v.x * u.y;
        a2.add(cross);
        sx.add((u.x + v.x) * cross);
        sy.add((u.y + v.y) * cross);
    }
    // A is the signed area of this same sum (never |A|): 1/(6A) = 1/(3·2A).
    let a2 = a2.value()?;
    let point = Point::new(
        sx.value()? / (3.0 * a2) + p.origin.x,
        sy.value()? / (3.0 * a2) + p.origin.y,
    );
    if !point.is_finite() {
        return Err(GeometryError::Overflow);
    }
    Ok(Centroid {
        point,
        signed_area: a2 / 2.0,
        tolerance: p.tolerance,
        notes: p.notes,
    })
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    /// The cleaning loop as first written (§10): drop the start vertex of the first zero-length
    /// segment, search again from the start, until none remains. Quadratic; kept as the reference.
    fn clean_reference(mut ring: Vec<(usize, Point, f64)>, eps: f64) -> Vec<(usize, Point, f64)> {
        loop {
            let m = ring.len();
            let zero = (0..m).find(|&i| {
                let (_, p, b) = ring[i];
                is_zero_length(p, ring[(i + 1) % m].1, b, eps)
            });
            match zero {
                Some(i) => {
                    ring.remove(i);
                }
                None => break,
            }
        }
        ring
    }

    proptest! {
        /// The linear pass gives exactly the reference loop's ring: same vertices, same order. Rings are
        /// drawn from few points with many repeats and near repeats (within and beyond ε), with bulges.
        #[test]
        fn linear_cleaning_equals_the_reference(
            v in prop::collection::vec(
                (
                    0_usize..4,
                    prop::sample::select(vec![0.0, 0.5e-3, 2e-3]),
                    prop::sample::select(vec![0.0, 0.0, 0.5, -1.0, 1e6]),
                ),
                0..40,
            )
        ) {
            let eps = 1e-3;
            let base = [(0.0, 0.0), (5.0, 0.0), (5.0, 5.0), (0.0, 5.0)];
            let ring: Vec<(usize, Point, f64)> = v
                .iter()
                .enumerate()
                .map(|(i, &(k, d, b))| (i, Point::new(base[k].0 + d, base[k].1), b))
                .collect();
            prop_assert_eq!(clean(ring.clone(), eps), clean_reference(ring, eps));
        }
    }
}
