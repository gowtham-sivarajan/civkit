// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 18:59 IST · last edited 2026-11-27 19:02 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! Regions: rings combined by even-odd nesting (docs/GEOMETRY.md §11, D-28 / ADR 0016).
//!
//! Each ring is checked on its own (§8), then every pair of rings must keep more than ε apart
//! (the exact edge relations of §8.7, internal as D7 requires). A ring's nesting depth is the
//! number of other rings that contain one of its vertices; it counts with sign (−1)^depth.

use crate::bulge::{BulgeArc, arc_length};
use crate::edges::{Edge, conflict};
use crate::error::{GeometryError, Note};
use crate::point::{Point, Vertex};
use crate::polygon::{PolygonArea, polygon_area};
use crate::tolerance::{BoundingBox, Sum, Tolerance, magnitude, ulp};

/// One ring of a region.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionRing {
    /// How many other rings contain it: even adds its area, odd subtracts it.
    pub depth: usize,
    /// The ring's own area, computed with the region's ε.
    pub area: PolygonArea,
}

/// The area of a region (§11).
#[derive(Debug, Clone, PartialEq)]
pub struct RegionArea {
    /// Σ (−1)^depth · |Aᵢ|.
    pub area: f64,
    /// The rings, in input order.
    pub rings: Vec<RegionRing>,
    /// The one tolerance used for every ring and every check, from the whole region (§5).
    pub tolerance: Tolerance,
    /// Region-level notes (§8.9). Each ring's own notes are in its [`PolygonArea`].
    pub notes: Vec<Note>,
}

/// A ring translated to the region's origin.
struct Ring {
    points: Vec<Point>,
    bulges: Vec<f64>,
}

impl Ring {
    fn segment(&self, i: usize) -> (Point, Point, f64) {
        let n = self.points.len();
        (self.points[i], self.points[(i + 1) % n], self.bulges[i])
    }

    /// The ring's edges for the crossing check; zero-length segments (§8.2) are skipped, and a
    /// bulge whose sagitta is at most ulp(M) is its chord (as §8.7).
    fn edges(&self, eps: f64, representable: f64) -> Vec<Edge> {
        (0..self.points.len())
            .filter_map(|i| {
                let (p, q, b) = self.segment(i);
                let c = p.distance(q);
                if arc_length(c, b) <= eps {
                    None
                } else if b != 0.0 && b.abs() * c / 2.0 > representable {
                    Some(Edge::Arc(BulgeArc::new(p, q, b)))
                } else {
                    Some(Edge::Line(p, q))
                }
            })
            .collect()
    }

    /// Candidate test points on the ring's boundary: each vertex and each edge's midpoint (the arc's
    /// midpoint for a bulged edge). Every point of a ring that crosses no other ring has the same
    /// containment, so any of them may be tested.
    fn probes(&self, representable: f64) -> impl Iterator<Item = Point> + '_ {
        (0..self.points.len()).flat_map(move |i| {
            let (p, q, b) = self.segment(i);
            let mid = if b != 0.0 && b.abs() * p.distance(q) / 2.0 > representable {
                BulgeArc::new(p, q, b).global_at(0.0)
            } else {
                Point::new(p.x + (q.x - p.x) / 2.0, p.y + (q.y - p.y) / 2.0)
            };
            [p, mid]
        })
    }

    /// Whether a point lies within ε of the chord of one of this ring's bulged edges. There the
    /// chord-and-segment parity of [`Ring::contains`] is ill-defined (the chord bounds both).
    fn near_a_chord(&self, p: Point, eps: f64, representable: f64) -> bool {
        (0..self.points.len()).any(|i| {
            let (a, b, bulge) = self.segment(i);
            let c = a.distance(b);
            if bulge == 0.0 || c == 0.0 || bulge.abs() * c / 2.0 <= representable {
                return false;
            }
            let local = BulgeArc::new(a, b, bulge).local_of(p);
            local.1.abs() <= eps && local.0.abs() <= c / 2.0 + eps
        })
    }

    /// Whether a point more than ε from the ring, and not near one of its bulged chords, lies
    /// inside it. The ray parity of the arcs equals
    /// the parity of the chord polygon, flipped once for each circular segment (between a chord and
    /// its arc) that contains the point.
    fn contains(&self, p: Point, representable: f64) -> bool {
        let mut inside = false;
        for i in 0..self.points.len() {
            let (a, b, bulge) = self.segment(i);
            if (a.y > p.y) != (b.y > p.y) {
                let x = a.x + (p.y - a.y) * (b.x - a.x) / (b.y - a.y);
                if p.x < x {
                    inside = !inside;
                }
            }
            let c = a.distance(b);
            if bulge != 0.0 && c > 0.0 && bulge.abs() * c / 2.0 > representable {
                let arc = BulgeArc::new(a, b, bulge);
                let local = arc.local_of(p);
                // The segment is the disc on the side of the chord holding the arc's midpoint,
                // whose local y is −b·c/2.
                if arc.signed_distance(local) < 0.0 && local.1 * bulge < 0.0 {
                    inside = !inside;
                }
            }
        }
        inside
    }
}

/// The area of rings combined by even-odd nesting (§11), e.g. a hatch with islands.
///
/// # Errors
/// [`GeometryError::NoRings`]; [`GeometryError::NonFinite`]; [`GeometryError::InvalidTolerance`];
/// [`GeometryError::Overflow`]; any error of [`polygon_area`] for the first ring (in input order)
/// that is invalid on its own; [`GeometryError::RingsIntersect`] for the first pair of rings that
/// cross or come within ε.
pub fn region_area(
    rings: &[&[Vertex]],
    tolerance: Option<f64>,
) -> Result<RegionArea, GeometryError> {
    if rings.is_empty() {
        return Err(GeometryError::NoRings);
    }
    if !rings
        .iter()
        .flat_map(|r| r.iter())
        .all(|v| v.point.is_finite() && v.bulge.is_finite())
    {
        return Err(GeometryError::NonFinite);
    }

    // One origin for the whole region (§10), then one tolerance (§5).
    let origin = rings
        .iter()
        .find_map(|r| r.first())
        .map_or(Point::new(0.0, 0.0), |v| v.point);
    let mut translated = Vec::with_capacity(rings.len());
    for r in rings {
        let points: Vec<Point> = r.iter().map(|v| v.point.minus(origin)).collect();
        if !points.iter().all(|p| p.is_finite()) {
            return Err(GeometryError::Overflow);
        }
        translated.push(Ring {
            points,
            bulges: r.iter().map(|v| v.bulge).collect(),
        });
    }
    let mut bbox = BoundingBox::new();
    for ring in &translated {
        for i in 0..ring.points.len() {
            let (p, q, b) = ring.segment(i);
            bbox.add(p);
            if b != 0.0 && p.distance(q) > 0.0 {
                BulgeArc::new(p, q, b).extremes().for_each(|e| bbox.add(e));
            }
        }
    }
    let t = Tolerance::compute(
        bbox.diagonal()?,
        magnitude(rings.iter().flat_map(|r| r.iter().map(|v| v.point))),
        tolerance,
    )?;
    let eps = t.linear;
    let representable = ulp(t.magnitude);

    // Each ring on its own, with the region's ε.
    let areas = rings
        .iter()
        .map(|r| polygon_area(r, Some(eps)))
        .collect::<Result<Vec<_>, _>>()?;

    // No two rings may cross or touch.
    let edges: Vec<Vec<Edge>> = translated
        .iter()
        .map(|r| r.edges(eps, representable))
        .collect();
    for i in 0..edges.len() {
        for j in i + 1..edges.len() {
            let meet = edges[i]
                .iter()
                .any(|e1| edges[j].iter().any(|e2| conflict(e1, e2, &[], eps)));
            if meet {
                return Err(GeometryError::RingsIntersect {
                    first: i,
                    second: j,
                });
            }
        }
    }

    // Depths, from a point of each ring (more than ε from every other ring), chosen clear of the
    // other rings' bulged chords; the first vertex if none is (a coincidence of measure zero).
    let mut sum = Sum::default();
    let mut out = Vec::with_capacity(rings.len());
    for (j, area) in areas.into_iter().enumerate() {
        let probe = translated[j]
            .probes(representable)
            .find(|&p| {
                translated
                    .iter()
                    .enumerate()
                    .all(|(i, r)| i == j || !r.near_a_chord(p, eps, representable))
            })
            .unwrap_or(translated[j].points[0]);
        let depth = translated
            .iter()
            .enumerate()
            .filter(|&(i, r)| i != j && r.contains(probe, representable))
            .count();
        sum.add(if depth % 2 == 0 {
            area.area
        } else {
            -area.area
        });
        out.push(RegionRing { depth, area });
    }

    let mut notes = Vec::new();
    if t.precision_limited() {
        notes.push(Note::PrecisionLimited);
    }
    Ok(RegionArea {
        area: sum.value()?,
        rings: out,
        tolerance: t,
        notes,
    })
}
