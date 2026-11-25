// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 18:37 IST · last edited 2026-11-25 18:37 IST

//! The self-intersection check (docs/GEOMETRY.md §8.7, D7). Internal: there is no public
//! intersection API.
//!
//! Two non-adjacent edges conflict when they come within ε anywhere. Two adjacent edges (sharing
//! one vertex, or both in a two-vertex ring) conflict when a far endpoint lies within ε of the other
//! edge, or when they cross or reach a local minimum distance ≤ ε more than ε from every shared
//! vertex.
//!
//! The minimum distance is taken over a complete candidate set: endpoints against the other edge,
//! interior points where both edges are normal to the line joining them (critical points), and
//! crossings. Along the second edge, the signed distance to the arc's circle is monotone between
//! critical points, so each piece holds at most one crossing, found by bisection.

use crate::bulge::BulgeArc;
use crate::point::Point;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Edge {
    Line(Point, Point),
    Arc(BulgeArc),
}

impl Edge {
    fn endpoints(&self) -> [Point; 2] {
        match self {
            Self::Line(p, q) => [*p, *q],
            Self::Arc(a) => [a.p1, a.p2],
        }
    }

    fn distance_to(&self, p: Point) -> f64 {
        match self {
            Self::Line(a, b) => point_segment_distance(p, *a, *b),
            Self::Arc(arc) => arc.distance_to(p),
        }
    }

    /// Parameter range: [0, 1] for a line, [−ψ, ψ] for an arc.
    fn range(&self) -> (f64, f64) {
        match self {
            Self::Line(..) => (0.0, 1.0),
            Self::Arc(a) => (-a.psi, a.psi),
        }
    }

    fn at(&self, t: f64) -> Point {
        match self {
            Self::Line(p, q) => {
                if t <= 0.0 {
                    *p
                } else if t >= 1.0 {
                    *q
                } else {
                    Point::new(p.x + t * (q.x - p.x), p.y + t * (q.y - p.y))
                }
            }
            Self::Arc(a) => a.global_at(t),
        }
    }
}

pub(crate) fn point_segment_distance(p: Point, a: Point, b: Point) -> f64 {
    let d = b.minus(a);
    let len2 = d.x * d.x + d.y * d.y;
    if len2 == 0.0 {
        return p.distance(a);
    }
    let t = (((p.x - a.x) * d.x + (p.y - a.y) * d.y) / len2).clamp(0.0, 1.0);
    p.distance(Point::new(a.x + t * d.x, a.y + t * d.y))
}

fn orient(p: Point, q: Point, r: Point) -> f64 {
    (q.x - p.x) * (r.y - p.y) - (q.y - p.y) * (r.x - p.x)
}

fn strictly_opposite(a: f64, b: f64) -> bool {
    (a < 0.0 && b > 0.0) || (a > 0.0 && b < 0.0)
}

fn segment_segment_distance(a: Point, b: Point, c: Point, d: Point) -> f64 {
    if strictly_opposite(orient(a, b, c), orient(a, b, d))
        && strictly_opposite(orient(c, d, a), orient(c, d, b))
    {
        return 0.0;
    }
    point_segment_distance(a, c, d)
        .min(point_segment_distance(b, c, d))
        .min(point_segment_distance(c, a, b))
        .min(point_segment_distance(d, a, b))
}

fn is_shared(p: Point, shared: &[Point]) -> bool {
    shared.iter().any(|s| s.same(p))
}

fn far_from_shared(p: Point, shared: &[Point], eps: f64) -> bool {
    shared.iter().all(|s| p.distance(*s) > eps)
}

/// Whether two ring edges conflict (§8.7).
pub(crate) fn conflict(e1: &Edge, e2: &Edge, shared: &[Point], eps: f64) -> bool {
    let d = match (e1, e2) {
        (Edge::Line(a, b), Edge::Line(c, d)) => {
            if shared.is_empty() {
                segment_segment_distance(*a, *b, *c, *d)
            } else {
                // Two straight edges sharing a vertex can only meet again by folding back, which
                // puts a far endpoint on the other edge.
                endpoint_distances(e1, e2, shared)
            }
        }
        (Edge::Arc(arc), other) | (other @ Edge::Line(..), Edge::Arc(arc)) => {
            curve_to_arc_distance(arc, other, shared, eps)
        }
    };
    d <= eps
}

fn endpoint_distances(e1: &Edge, e2: &Edge, shared: &[Point]) -> f64 {
    let mut best = f64::INFINITY;
    for p in e2.endpoints() {
        if !is_shared(p, shared) {
            best = best.min(e1.distance_to(p));
        }
    }
    for p in e1.endpoints() {
        if !is_shared(p, shared) {
            best = best.min(e2.distance_to(p));
        }
    }
    best
}

/// Parameters on `q` where it is normal to the radius of `arc` (critical points of the distance).
fn critical_parameters(arc: &BulgeArc, q: &Edge) -> Vec<f64> {
    match q {
        Edge::Line(p0, p1) => {
            let a = arc.local_of(*p0);
            let b = arc.local_of(*p1);
            let d = (b.0 - a.0, b.1 - a.1);
            let len2 = d.0 * d.0 + d.1 * d.1;
            if len2 == 0.0 {
                return Vec::new();
            }
            // Foot of the perpendicular from the centre (0, k) onto the line, in the arc's frame.
            let t = (-a.0 * d.0 + (arc.k() - a.1) * d.1) / len2;
            if t > 0.0 && t < 1.0 {
                vec![t]
            } else {
                Vec::new()
            }
        }
        Edge::Arc(other) => {
            // Points of `other` on the line through both centres: radial(δ) = (sin δ, −s cos δ)
            // parallel to the centre difference, in `other`'s frame.
            let c = other.local_of(arc.centre());
            let w = (c.0, c.1 - other.k());
            if w.0 == 0.0 && w.1 == 0.0 {
                return Vec::new();
            }
            let d1 = w.0.atan2(-other.sign() * w.1);
            let d2 = if d1 > 0.0 {
                d1 - std::f64::consts::PI
            } else {
                d1 + std::f64::consts::PI
            };
            [d1, d2]
                .into_iter()
                .filter(|d| d.abs() < other.psi)
                .collect()
        }
    }
}

fn curve_to_arc_distance(arc: &BulgeArc, q: &Edge, shared: &[Point], eps: f64) -> f64 {
    let as_line = Edge::Arc(*arc);
    let mut best = endpoint_distances(&as_line, q, shared);

    let f = |t: f64| arc.signed_distance(arc.local_of(q.at(t)));
    let (t0, t1) = q.range();
    let mut crit = critical_parameters(arc, q);
    crit.sort_by(f64::total_cmp);

    // Interior critical points: the joining segment is radial for the arc and normal to q.
    for &t in &crit {
        let p = q.at(t);
        let l = arc.local_of(p);
        if far_from_shared(p, shared, eps) && arc.in_wedge(l) {
            best = best.min(arc.signed_distance(l).abs());
        }
    }

    // Crossings of the arc's circle, one per monotone piece.
    let mut breaks = Vec::with_capacity(crit.len() + 2);
    breaks.push(t0);
    breaks.extend(crit.iter().copied().filter(|&t| t > t0 && t < t1));
    breaks.push(t1);
    for w in breaks.windows(2) {
        let (mut lo, mut hi) = (w[0], w[1]);
        let mut flo = f(lo);
        if !strictly_opposite(flo, f(hi)) {
            continue;
        }
        for _ in 0..200 {
            let mid = lo + (hi - lo) / 2.0;
            if mid <= lo || mid >= hi {
                break;
            }
            let fm = f(mid);
            if fm == 0.0 {
                lo = mid;
                hi = mid;
                break;
            }
            if (fm < 0.0) == (flo < 0.0) {
                lo = mid;
                flo = fm;
            } else {
                hi = mid;
            }
        }
        let p = q.at(lo + (hi - lo) / 2.0);
        if far_from_shared(p, shared, eps) && arc.in_wedge(arc.local_of(p)) {
            return 0.0;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sampled(e: &Edge, n: usize) -> Vec<Point> {
        let (t0, t1) = e.range();
        (0..=n)
            .map(|i| e.at(t0 + (t1 - t0) * i as f64 / n as f64))
            .collect()
    }

    fn brute(a: &Edge, b: &Edge) -> f64 {
        let sa = sampled(a, 4000);
        let sb = sampled(b, 4000);
        let mut best = f64::INFINITY;
        for p in &sa {
            best = best.min(b.distance_to(*p));
        }
        for p in &sb {
            best = best.min(a.distance_to(*p));
        }
        best
    }

    #[test]
    fn non_adjacent_distances_match_sampling() {
        let arcs = [
            BulgeArc::new(Point::new(0.0, 0.0), Point::new(10.0, 0.0), -0.5),
            BulgeArc::new(Point::new(2.0, 6.0), Point::new(9.0, 3.0), 0.7),
            BulgeArc::new(Point::new(-4.0, 1.0), Point::new(-1.0, 8.0), 1.6),
            BulgeArc::new(Point::new(3.0, -2.0), Point::new(6.0, -2.5), -2.4),
        ];
        let lines = [
            Edge::Line(Point::new(0.0, 5.0), Point::new(10.0, 5.0)),
            Edge::Line(Point::new(5.0, -10.0), Point::new(5.0, 10.0)),
            Edge::Line(Point::new(-6.0, -6.0), Point::new(-5.0, 9.0)),
        ];
        let mut edges: Vec<Edge> = arcs.iter().map(|a| Edge::Arc(*a)).collect();
        edges.extend(lines);
        for (i, e1) in edges.iter().enumerate() {
            for e2 in edges.iter().skip(i + 1) {
                if matches!((e1, e2), (Edge::Line(..), Edge::Line(..))) {
                    continue;
                }
                let exact = match (e1, e2) {
                    (Edge::Arc(a), o) | (o @ Edge::Line(..), Edge::Arc(a)) => {
                        curve_to_arc_distance(a, o, &[], 0.0)
                    }
                    _ => unreachable!(),
                };
                let b = brute(e1, e2);
                // Sampling can only overestimate; crossings give exactly 0.
                assert!(exact <= b + 1e-9, "{e1:?} / {e2:?}: {exact} > {b}");
                // 4000 samples on edges of length ~10: near a crossing the closest sample can be
                // up to half a spacing (~1.3e-3) away.
                assert!(b - exact < 5e-3, "{e1:?} / {e2:?}: {exact} vs {b}");
            }
        }
    }
}
