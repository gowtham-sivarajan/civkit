// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 18:37 IST · last edited 2026-11-25 18:37 IST

//! Bulged segments (docs/GEOMETRY.md §4, §9, §10).
//!
//! Every quantity is computed from the bulge b, never from sin(Δ/2). Geometry tests use the arc's own
//! frame: origin at the chord midpoint m, x along the chord (unit u), y along its left normal n.
//! In that frame the chord runs from (−a, 0) to (a, 0), the centre is (0, k) and the arc point at
//! angle δ from the arc's midpoint is (r·sin δ, −b·a + s·2r·sin²(δ/2)), with s = sign(b). Nothing
//! subtracts two numbers of the size of the radius, so a nearly straight arc of huge radius loses
//! no accuracy.

use crate::point::Point;

/// Arc length of a bulged segment with chord c: s = c(1 + b²)·atan|b|/|b| (§9). Equals c for b = 0.
pub(crate) fn arc_length(c: f64, b: f64) -> f64 {
    let ab = b.abs();
    if ab == 0.0 {
        c
    } else if ab <= 1.0 {
        c * (1.0 + ab * ab) * (ab.atan() / ab)
    } else {
        c * ab.atan() * (ab + 1.0 / ab)
    }
}

/// T(x) = 1/3! − x²/5! + x⁴/7! − …, so that x − sin x = x³·T(x) without cancellation (§10).
fn x_minus_sin_x_over_x3(x: f64) -> f64 {
    let x2 = x * x;
    let mut term: f64 = 1.0 / 6.0;
    let mut sum: f64 = 0.0;
    let mut n = 0.0;
    while n < 40.0 {
        sum += term;
        if term.abs() < f64::EPSILON / 2.0 * sum.abs() {
            break;
        }
        term *= -x2 / ((2.0 * n + 4.0) * (2.0 * n + 5.0));
        n += 1.0;
    }
    sum
}

/// Signed area of the circular segment between a bulged edge and its chord c (§10):
/// sign(b)·(r²/2)·(|Δ| − sin|Δ|), computed without cancellation or overflow for small |Δ|.
pub(crate) fn segment_area(c: f64, b: f64) -> f64 {
    let ab = b.abs();
    if ab == 0.0 || c == 0.0 {
        return 0.0;
    }
    let x = 4.0 * ab.atan();
    let magnitude = if x < 0.5 {
        let q = x / ab;
        let f = 1.0 + ab * ab;
        c * c * f * f / 32.0 * q * q * x * x_minus_sin_x_over_x3(x)
    } else {
        let r = c / 4.0 * (ab + 1.0 / ab);
        r * r / 2.0 * (x - x.sin())
    };
    magnitude.copysign(b)
}

/// A bulged segment in its own frame. Requires a chord > 0 and b ≠ 0, all finite.
#[derive(Debug, Clone, Copy)]
pub(crate) struct BulgeArc {
    pub(crate) p1: Point,
    pub(crate) p2: Point,
    m: Point,
    u: (f64, f64),
    n: (f64, f64),
    a: f64,
    b: f64,
    s: f64,
    r: f64,
    k: f64,
    pub(crate) psi: f64,
}

impl BulgeArc {
    pub(crate) fn new(p1: Point, p2: Point, b: f64) -> Self {
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        let c = dx.hypot(dy);
        let u = (dx / c, dy / c);
        let ab = b.abs();
        let a = c / 2.0;
        Self {
            p1,
            p2,
            m: Point::new(p1.x + dx / 2.0, p1.y + dy / 2.0),
            u,
            n: (-u.1, u.0),
            a,
            b,
            s: 1.0_f64.copysign(b),
            r: a / 2.0 * (ab + 1.0 / ab),
            k: a / 2.0 * (1.0 / b - b),
            psi: 2.0 * ab.atan(),
        }
    }

    pub(crate) fn local_of(&self, p: Point) -> (f64, f64) {
        let dx = p.x - self.m.x;
        let dy = p.y - self.m.y;
        (dx * self.u.0 + dy * self.u.1, dx * self.n.0 + dy * self.n.1)
    }

    pub(crate) fn global_of(&self, (x, y): (f64, f64)) -> Point {
        Point::new(
            self.m.x + x * self.u.0 + y * self.n.0,
            self.m.y + x * self.u.1 + y * self.n.1,
        )
    }

    /// Arc point at angle δ ∈ [−ψ, ψ] from the arc's midpoint, in local coordinates. δ = −ψ is p1.
    pub(crate) fn point_at(&self, delta: f64) -> (f64, f64) {
        let h = (delta / 2.0).sin();
        (
            self.r * delta.sin(),
            -self.b * self.a + self.s * 2.0 * self.r * h * h,
        )
    }

    /// The arc point at δ, in global coordinates; the stored endpoints are returned exactly.
    pub(crate) fn global_at(&self, delta: f64) -> Point {
        if delta <= -self.psi {
            self.p1
        } else if delta >= self.psi {
            self.p2
        } else {
            self.global_of(self.point_at(delta))
        }
    }

    /// Signed distance of a local point from the full circle (positive outside), computed as
    /// (|P − C|² − r²)/(|P − C| + r) with |P − C|² − r² = x² + y² − 2ky − a² (since r² = a² + k²).
    pub(crate) fn signed_distance(&self, (x, y): (f64, f64)) -> f64 {
        let num = x * x + y * y - 2.0 * self.k * y - self.a * self.a;
        num / (x.hypot(y - self.k) + self.r)
    }

    /// Whether the ray from the centre through a local point meets the arc (its radial projection
    /// lies on the arc). Cross products relative to the centre, expanded so no radius-sized
    /// numbers are subtracted.
    pub(crate) fn in_wedge(&self, (x, y): (f64, f64)) -> bool {
        let c1 = self.s * (self.k * (self.a + x) - self.a * y);
        let c2 = self.s * (self.k * (self.a - x) - self.a * y);
        if self.b.abs() <= 1.0 {
            c1 >= 0.0 && c2 >= 0.0
        } else {
            c1 >= 0.0 || c2 >= 0.0
        }
    }

    /// Exact distance from a point to the arc.
    pub(crate) fn distance_to(&self, p: Point) -> f64 {
        let l = self.local_of(p);
        if self.in_wedge(l) {
            self.signed_distance(l).abs()
        } else {
            p.distance(self.p1).min(p.distance(self.p2))
        }
    }

    /// The circle centre in global coordinates (only used where a radius-sized error is harmless).
    pub(crate) fn centre(&self) -> Point {
        self.global_of((0.0, self.k))
    }

    pub(crate) fn k(&self) -> f64 {
        self.k
    }

    pub(crate) fn sign(&self) -> f64 {
        self.s
    }

    /// The arc's axis-aligned extreme points strictly inside its sweep (for the bounding box, §5).
    pub(crate) fn extremes(&self) -> impl Iterator<Item = Point> + '_ {
        [(1.0, 0.0), (0.0, 1.0), (-1.0, 0.0), (0.0, -1.0)]
            .into_iter()
            .filter_map(move |(ex, ey)| {
                let el = (ex * self.u.0 + ey * self.u.1, ex * self.n.0 + ey * self.n.1);
                let delta = el.0.atan2(-self.s * el.1);
                (delta.abs() < self.psi).then(|| self.global_of(self.point_at(delta)))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Dense sampling of the arc, independent of the closed forms under test.
    fn sampled(arc: &BulgeArc, n: usize) -> Vec<Point> {
        (0..=n)
            .map(|i| {
                let t = i as f64 / n as f64;
                arc.global_of(arc.point_at(-arc.psi + 2.0 * arc.psi * t))
            })
            .collect()
    }

    fn brute_distance(samples: &[Point], p: Point) -> f64 {
        samples
            .iter()
            .fold(f64::INFINITY, |m, q| m.min(p.distance(*q)))
    }

    #[test]
    fn endpoints_and_midpoint_are_on_the_arc() {
        for b in [1e-6, 0.3, -0.3, 1.0, -1.0, 2.5, -7.0] {
            let arc = BulgeArc::new(Point::new(1.0, 2.0), Point::new(7.0, -1.0), b);
            let e1 = arc.global_of(arc.point_at(-arc.psi));
            let e2 = arc.global_of(arc.point_at(arc.psi));
            assert!(
                e1.distance(arc.p1) < 1e-12 && e2.distance(arc.p2) < 1e-12,
                "b = {b}"
            );
            // Sagitta |b|·c/2 on the right of travel for b > 0.
            let mid = arc.local_of(arc.global_of(arc.point_at(0.0)));
            let c = arc.p1.distance(arc.p2);
            assert!((mid.1 + b * c / 2.0).abs() < 1e-12, "b = {b}");
        }
    }

    #[test]
    fn point_distance_matches_sampling() {
        let pts = [
            Point::new(4.0, 0.5),
            Point::new(4.0, 6.0),
            Point::new(-3.0, 2.0),
            Point::new(10.0, -8.0),
            Point::new(4.0, -4.0),
            Point::new(1.0, 2.0),
        ];
        for b in [0.05, 0.4, -0.4, 1.0, -1.0, 1.8, -3.0] {
            let arc = BulgeArc::new(Point::new(1.0, 2.0), Point::new(7.0, -1.0), b);
            let samples = sampled(&arc, 200_000);
            for p in pts {
                let exact = arc.distance_to(p);
                let brute = brute_distance(&samples, p);
                // Sampling overestimates by at most half the sample spacing squared over r (tiny).
                assert!(
                    (exact - brute).abs() < 1e-6,
                    "b = {b}, p = {p:?}: {exact} vs {brute}"
                );
            }
        }
    }

    #[test]
    fn nearly_straight_arc_of_huge_radius_keeps_its_accuracy() {
        // b = 1e-9 on a chord of 10: radius 2.5e9, sagitta 5e-9.
        let arc = BulgeArc::new(Point::new(0.0, 0.0), Point::new(10.0, 0.0), 1e-9);
        // The sagitta point (5, -5e-9) is on the arc; (5, 0) is 5e-9 from it.
        assert!(arc.distance_to(Point::new(5.0, -5e-9)) < 1e-20);
        assert!((arc.distance_to(Point::new(5.0, 0.0)) - 5e-9).abs() < 1e-20);
        assert!((arc.distance_to(Point::new(5.0, 1.0)) - (1.0 + 5e-9)).abs() < 1e-15);
    }

    #[test]
    fn extremes_match_sampling() {
        for b in [0.2, -0.2, 1.0, 2.0, -2.0, 30.0] {
            let arc = BulgeArc::new(Point::new(0.0, 0.0), Point::new(3.0, 4.0), b);
            let samples = sampled(&arc, 200_000);
            let mut lo = (arc.p1.x.min(arc.p2.x), arc.p1.y.min(arc.p2.y));
            let mut hi = (arc.p1.x.max(arc.p2.x), arc.p1.y.max(arc.p2.y));
            for e in arc.extremes() {
                lo = (lo.0.min(e.x), lo.1.min(e.y));
                hi = (hi.0.max(e.x), hi.1.max(e.y));
            }
            let s_lo = samples.iter().fold((f64::INFINITY, f64::INFINITY), |m, p| {
                (m.0.min(p.x), m.1.min(p.y))
            });
            let s_hi = samples
                .iter()
                .fold((f64::NEG_INFINITY, f64::NEG_INFINITY), |m, p| {
                    (m.0.max(p.x), m.1.max(p.y))
                });
            for (x, y) in [
                (lo.0, s_lo.0),
                (lo.1, s_lo.1),
                (hi.0, s_hi.0),
                (hi.1, s_hi.1),
            ] {
                assert!((x - y).abs() < 1e-6, "b = {b}: {x} vs {y}");
            }
        }
    }

    #[test]
    fn segment_area_branches_agree_at_the_switch() {
        // x = 0.5 is reached at b = tan(0.125); both branches must agree there.
        let b = 0.125_f64.tan();
        let c = 10.0;
        let ab = b;
        let x = 4.0 * ab.atan();
        let series = {
            let q = x / ab;
            let f = 1.0 + ab * ab;
            c * c * f * f / 32.0 * q * q * x * x_minus_sin_x_over_x3(x)
        };
        let direct = {
            let r = c / 4.0 * (ab + 1.0 / ab);
            r * r / 2.0 * (x - x.sin())
        };
        assert!(((series - direct) / direct).abs() < 1e-14);
    }
}
