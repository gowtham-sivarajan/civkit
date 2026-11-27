// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 19:08 IST · last edited 2026-11-27 19:18 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! The §8.7 prefilter: a sweep over edge bounding boxes, each widened by ε, and the work limit
//! (docs/GEOMETRY.md §8.7, D-28 / ADR 0016).
//!
//! Two edges whose widened boxes do not overlap are more than 2ε apart, so they cannot conflict;
//! testing only overlapping pairs therefore finds exactly the conflicts that testing every pair
//! finds. Work is counted as the pairs the sweep examines (x-ranges overlap), which also bounds its
//! time when many boxes overlap in x but not in y.

use crate::bulge::BulgeArc;
use crate::edges::Edge;
use crate::error::GeometryError;
use crate::point::Point;

/// A limit on the edge pairs the §8.7 sweep may examine, shared by the calls it is passed to.
/// Exceeding it is [`GeometryError::WorkLimitExceeded`]: the check was not completed, so the
/// geometry is neither found valid nor found self-intersecting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkLimit {
    limit: u64,
    used: u64,
}

impl WorkLimit {
    /// A limit of `limit` examined pairs.
    #[must_use]
    pub const fn new(limit: u64) -> Self {
        Self { limit, used: 0 }
    }

    /// No limit (`u64::MAX`), as the functions without a limit use.
    #[must_use]
    pub const fn unlimited() -> Self {
        Self::new(u64::MAX)
    }

    /// The limit.
    #[must_use]
    pub const fn limit(&self) -> u64 {
        self.limit
    }

    /// Pairs examined so far.
    #[must_use]
    pub const fn used(&self) -> u64 {
        self.used
    }

    /// Pairs still allowed.
    #[must_use]
    pub const fn remaining(&self) -> u64 {
        self.limit - self.used
    }

    fn take(&mut self) -> Result<(), GeometryError> {
        if self.used >= self.limit {
            return Err(GeometryError::WorkLimitExceeded { limit: self.limit });
        }
        self.used += 1;
        Ok(())
    }
}

/// An edge's axis-aligned bounding box, widened by ε on every side.
#[derive(Debug, Clone, Copy)]
pub(crate) struct EdgeBox {
    min: Point,
    max: Point,
}

impl EdgeBox {
    /// The box of a line's ends, or of an arc's ends and its extremes (§5), widened by `eps`.
    pub(crate) fn of(edge: &Edge, eps: f64) -> Self {
        let (p, q, arc): (Point, Point, Option<&BulgeArc>) = match edge {
            Edge::Line(p, q) => (*p, *q, None),
            Edge::Arc(a) => (a.p1, a.p2, Some(a)),
        };
        let mut min = Point::new(p.x.min(q.x), p.y.min(q.y));
        let mut max = Point::new(p.x.max(q.x), p.y.max(q.y));
        if let Some(a) = arc {
            for e in a.extremes() {
                min = Point::new(min.x.min(e.x), min.y.min(e.y));
                max = Point::new(max.x.max(e.x), max.y.max(e.y));
            }
        }
        Self {
            min: Point::new(min.x - eps, min.y - eps),
            max: Point::new(max.x + eps, max.y + eps),
        }
    }

    fn overlaps_in_y(&self, other: &Self) -> bool {
        self.min.y <= other.max.y && other.min.y <= self.max.y
    }
}

/// Calls `test(i, j)`, i < j, for every pair of boxes that overlap, in an unspecified order. Every
/// pair whose x-ranges overlap is examined and charged to `work` first, except pairs `skip` names
/// (which are neither charged nor tested).
///
/// # Errors
/// [`GeometryError::WorkLimitExceeded`] when `work` runs out.
pub(crate) fn overlapping_pairs(
    boxes: &[EdgeBox],
    skip: impl Fn(usize, usize) -> bool,
    work: &mut WorkLimit,
    mut test: impl FnMut(usize, usize),
) -> Result<(), GeometryError> {
    let mut order: Vec<usize> = (0..boxes.len()).collect();
    order.sort_by(|&a, &b| boxes[a].min.x.total_cmp(&boxes[b].min.x).then(a.cmp(&b)));
    let mut active: Vec<usize> = Vec::new();
    for &k in &order {
        let bk = boxes[k];
        active.retain(|&a| boxes[a].max.x >= bk.min.x);
        for &a in &active {
            let (i, j) = if a < k { (a, k) } else { (k, a) };
            if skip(i, j) {
                continue;
            }
            work.take()?;
            if boxes[a].overlaps_in_y(&bk) {
                test(i, j);
            }
        }
        active.push(k);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use proptest::prelude::*;

    use super::*;
    use crate::edges::conflict;

    const EPS: f64 = 1e-8;

    /// A random ring: small integer coordinates nudged by 0, ±ε/2 or ±5ε (near touches either side
    /// of ε), with bulges from a fixed set of both signs, minor and major arcs.
    fn ring() -> impl Strategy<Value = Vec<(f64, f64, f64)>> {
        let nudge = prop::sample::select(vec![0.0, 0.5 * EPS, -0.5 * EPS, 5.0 * EPS, -5.0 * EPS]);
        let bulge = prop::sample::select(vec![0.0, 0.0, 0.3, -0.3, 1.0, -1.0, 2.5, -2.5]);
        prop::collection::vec((0_i32..6, 0_i32..6, nudge.clone(), nudge, bulge), 2..10).prop_map(
            |v| {
                v.into_iter()
                    .map(|(x, y, dx, dy, b)| (f64::from(x) + dx, f64::from(y) + dy, b))
                    .collect()
            },
        )
    }

    /// Edges as §8.7 builds them (a zero-length chord is a line), and the shared vertices of each
    /// pair by the ring's adjacency.
    fn edges(v: &[(f64, f64, f64)]) -> Vec<Edge> {
        let n = v.len();
        (0..n)
            .map(|i| {
                let p = Point::new(v[i].0, v[i].1);
                let q = Point::new(v[(i + 1) % n].0, v[(i + 1) % n].1);
                if v[i].2 != 0.0 && p.distance(q) > 0.0 {
                    Edge::Arc(BulgeArc::new(p, q, v[i].2))
                } else {
                    Edge::Line(p, q)
                }
            })
            .collect()
    }

    fn shared(v: &[(f64, f64, f64)], i: usize, j: usize) -> Vec<Point> {
        let m = v.len();
        let pt = |k: usize| Point::new(v[k].0, v[k].1);
        if m == 2 {
            vec![pt(0), pt(1)]
        } else if j == i + 1 {
            vec![pt(j)]
        } else if i == 0 && j == m - 1 {
            vec![pt(0)]
        } else {
            Vec::new()
        }
    }

    proptest! {
        /// The sweep finds exactly the conflicting pairs that testing every pair finds, and is
        /// charged exactly the pairs whose widened x-ranges overlap.
        #[test]
        fn sweep_finds_exactly_what_all_pairs_find(v in ring()) {
            let e = edges(&v);
            let all: BTreeSet<(usize, usize)> = (0..e.len())
                .flat_map(|i| (i + 1..e.len()).map(move |j| (i, j)))
                .filter(|&(i, j)| conflict(&e[i], &e[j], &shared(&v, i, j), EPS))
                .collect();
            let boxes: Vec<EdgeBox> = e.iter().map(|x| EdgeBox::of(x, EPS)).collect();
            let mut found = BTreeSet::new();
            let mut work = WorkLimit::unlimited();
            overlapping_pairs(&boxes, |_, _| false, &mut work, |i, j| {
                if conflict(&e[i], &e[j], &shared(&v, i, j), EPS) {
                    found.insert((i, j));
                }
            })
            .expect("unlimited");
            prop_assert_eq!(&found, &all);

            let x_overlaps = (0..boxes.len())
                .flat_map(|i| (i + 1..boxes.len()).map(move |j| (i, j)))
                .filter(|&(i, j)| boxes[i].min.x <= boxes[j].max.x && boxes[j].min.x <= boxes[i].max.x)
                .count() as u64;
            prop_assert_eq!(work.used(), x_overlaps);
        }

        /// A pair whose widened boxes do not overlap is more than ε apart (the prefilter's premise),
        /// checked against the exact relation for every pair of edges from two random rings.
        #[test]
        fn disjoint_boxes_never_conflict(a in ring(), b in ring()) {
            let (ea, eb) = (edges(&a), edges(&b));
            for x in &ea {
                for y in &eb {
                    let (bx, by) = (EdgeBox::of(x, EPS), EdgeBox::of(y, EPS));
                    let overlap = bx.min.x <= by.max.x && by.min.x <= bx.max.x && bx.overlaps_in_y(&by);
                    if !overlap {
                        prop_assert!(!conflict(x, y, &[], EPS));
                    }
                }
            }
        }
    }

    #[test]
    fn skipped_pairs_are_neither_charged_nor_tested() {
        let e = [
            Edge::Line(Point::new(0.0, 0.0), Point::new(1.0, 0.0)),
            Edge::Line(Point::new(0.0, 0.0), Point::new(1.0, 0.0)),
        ];
        let boxes: Vec<EdgeBox> = e.iter().map(|x| EdgeBox::of(x, EPS)).collect();
        let mut work = WorkLimit::new(0);
        let mut tested = 0;
        overlapping_pairs(&boxes, |_, _| true, &mut work, |_, _| tested += 1)
            .expect("nothing charged");
        assert_eq!((work.used(), tested), (0, 0));
    }
}
