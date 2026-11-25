// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 18:37 IST · last edited 2026-11-25 18:37 IST

//! Tolerances (docs/GEOMETRY.md §5) and compensated summation (§9).

use crate::error::GeometryError;
use crate::point::Point;

/// The distance from |x| to the next larger `f64`; ulp(0) is the smallest positive subnormal (§5).
/// NaN for non-finite input.
#[must_use]
pub fn ulp(x: f64) -> f64 {
    let a = x.abs();
    if !a.is_finite() {
        return f64::NAN;
    }
    f64::from_bits(a.to_bits() + 1) - a
}

/// The tolerance used by one call, returned with its result (§5, D2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tolerance {
    /// Linear tolerance ε, in the input's length unit.
    pub linear: f64,
    /// Area tolerance ε_A = ε × D.
    pub area: f64,
    /// D: diagonal of the bounding box of the geometry, including arc extremes.
    pub size: f64,
    /// M: largest absolute coordinate.
    pub magnitude: f64,
    /// True when the caller passed ε explicitly.
    pub explicit: bool,
}

impl Tolerance {
    /// ε = max(10⁻⁹·D, 1024·ulp(M)), or the explicit value when one is given.
    pub(crate) fn compute(
        size: f64,
        magnitude: f64,
        explicit: Option<f64>,
    ) -> Result<Self, GeometryError> {
        let linear = match explicit {
            Some(e) if e.is_finite() && e >= 0.0 => e,
            Some(_) => return Err(GeometryError::InvalidTolerance),
            None => (1e-9 * size).max(1024.0 * ulp(magnitude)),
        };
        Ok(Self {
            linear,
            area: linear * size,
            size,
            magnitude,
            explicit: explicit.is_some(),
        })
    }

    /// §8.9: D > 0 and 1024·ulp(M) > 10⁻⁶·D. Independent of an explicit ε.
    #[must_use]
    pub fn precision_limited(&self) -> bool {
        self.size > 0.0 && 1024.0 * ulp(self.magnitude) > 1e-6 * self.size
    }
}

/// Axis-aligned bounding box, used for D.
#[derive(Debug, Clone, Copy)]
pub(crate) struct BoundingBox {
    min: Point,
    max: Point,
    empty: bool,
}

impl BoundingBox {
    pub(crate) fn new() -> Self {
        Self {
            min: Point::new(0.0, 0.0),
            max: Point::new(0.0, 0.0),
            empty: true,
        }
    }

    pub(crate) fn add(&mut self, p: Point) {
        if self.empty {
            self.min = p;
            self.max = p;
            self.empty = false;
        } else {
            self.min = Point::new(self.min.x.min(p.x), self.min.y.min(p.y));
            self.max = Point::new(self.max.x.max(p.x), self.max.y.max(p.y));
        }
    }

    /// The diagonal D; an overflow is an error.
    pub(crate) fn diagonal(&self) -> Result<f64, GeometryError> {
        if self.empty {
            return Ok(0.0);
        }
        let d = (self.max.x - self.min.x).hypot(self.max.y - self.min.y);
        if d.is_finite() {
            Ok(d)
        } else {
            Err(GeometryError::Overflow)
        }
    }
}

/// Largest absolute coordinate M.
pub(crate) fn magnitude<I: IntoIterator<Item = Point>>(points: I) -> f64 {
    points
        .into_iter()
        .fold(0.0_f64, |m, p| m.max(p.x.abs()).max(p.y.abs()))
}

/// Compensated (Neumaier) summation.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Sum {
    sum: f64,
    compensation: f64,
}

impl Sum {
    pub(crate) fn add(&mut self, x: f64) {
        let t = self.sum + x;
        if self.sum.abs() >= x.abs() {
            self.compensation += (self.sum - t) + x;
        } else {
            self.compensation += (x - t) + self.sum;
        }
        self.sum = t;
    }

    /// The compensated total; an overflow (or NaN from one) is an error.
    pub(crate) fn value(&self) -> Result<f64, GeometryError> {
        if !self.sum.is_finite() {
            return Err(GeometryError::Overflow);
        }
        let v = self.sum + self.compensation;
        if v.is_finite() {
            Ok(v)
        } else {
            Err(GeometryError::Overflow)
        }
    }
}
