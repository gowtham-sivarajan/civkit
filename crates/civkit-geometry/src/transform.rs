// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 18:59 IST · last edited 2026-11-27 19:02 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! 2D affine transformations and block placement (docs/GEOMETRY.md §11, D-28 / ADR 0016).

use crate::error::GeometryError;
use crate::point::{Point, Vertex};

/// σ₁ − σ₂ ≤ `SIMILARITY`·σ₁ makes a transform a similarity (§11).
const SIMILARITY: f64 = 1e-9;

/// A 2D affine transformation: (x, y) ↦ (a·x + b·y + tx, c·x + d·y + ty) (§11).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Affine2 {
    /// Row 1, column 1 of the linear part.
    pub a: f64,
    /// Row 1, column 2.
    pub b: f64,
    /// Row 2, column 1.
    pub c: f64,
    /// Row 2, column 2.
    pub d: f64,
    /// Translation in x.
    pub tx: f64,
    /// Translation in y.
    pub ty: f64,
}

/// One block placement of an INSERT or one MINSERT cell (§11):
/// p' = P + R(θ)·((c·Δc, r·Δr) + S(sx, sy)·(p − B)).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InsertPlacement {
    /// B, the block's base point.
    pub base: Point,
    /// P, the insertion point.
    pub at: Point,
    /// (sx, sy), the scale factors; negative mirrors.
    pub scale: (f64, f64),
    /// θ in degrees, counter-clockwise, as DXF states it.
    pub rotation_deg: f64,
    /// (c·Δc, r·Δr): the MINSERT cell's offset, unscaled, along the rotated axes; (0, 0) for an INSERT.
    pub offset: (f64, f64),
}

fn require_finite(values: &[f64]) -> Result<(), GeometryError> {
    if values.iter().all(|v| v.is_finite()) {
        Ok(())
    } else {
        Err(GeometryError::NonFinite)
    }
}

impl Affine2 {
    /// The identity.
    pub const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        tx: 0.0,
        ty: 0.0,
    };

    /// A transform from its six entries.
    ///
    /// # Errors
    /// [`GeometryError::NonFinite`] if an entry is not finite.
    pub fn new(a: f64, b: f64, c: f64, d: f64, tx: f64, ty: f64) -> Result<Self, GeometryError> {
        require_finite(&[a, b, c, d, tx, ty])?;
        Ok(Self { a, b, c, d, tx, ty })
    }

    /// A translation by (dx, dy).
    ///
    /// # Errors
    /// [`GeometryError::NonFinite`].
    pub fn translation(dx: f64, dy: f64) -> Result<Self, GeometryError> {
        Self::new(1.0, 0.0, 0.0, 1.0, dx, dy)
    }

    /// A scale by (sx, sy) about the origin; a negative factor mirrors.
    ///
    /// # Errors
    /// [`GeometryError::NonFinite`].
    pub fn scale(sx: f64, sy: f64) -> Result<Self, GeometryError> {
        Self::new(sx, 0.0, 0.0, sy, 0.0, 0.0)
    }

    /// A counter-clockwise rotation about the origin by an angle in degrees. Multiples of 90° are
    /// exact.
    ///
    /// # Errors
    /// [`GeometryError::NonFinite`].
    pub fn rotation_deg(degrees: f64) -> Result<Self, GeometryError> {
        require_finite(&[degrees])?;
        let r = degrees.rem_euclid(360.0);
        // rem_euclid is exact, so an exact multiple of 90° is recognised exactly.
        #[allow(clippy::float_cmp)]
        let (sin, cos) = if r == 0.0 {
            (0.0, 1.0)
        } else if r == 90.0 {
            (1.0, 0.0)
        } else if r == 180.0 {
            (0.0, -1.0)
        } else if r == 270.0 {
            (-1.0, 0.0)
        } else {
            r.to_radians().sin_cos()
        };
        Self::new(cos, -sin, sin, cos, 0.0, 0.0)
    }

    /// x ↦ −x: the map of an OCS with extrusion (0, 0, −1) to world coordinates (§11).
    #[must_use]
    pub const fn mirror_x() -> Self {
        Self {
            a: -1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            tx: 0.0,
            ty: 0.0,
        }
    }

    /// The placement transform of an INSERT or one MINSERT cell (§11).
    ///
    /// # Errors
    /// [`GeometryError::NonFinite`] for a non-finite parameter, [`GeometryError::Overflow`] if the
    /// composition overflows.
    pub fn insert(p: &InsertPlacement) -> Result<Self, GeometryError> {
        Self::translation(-p.base.x, -p.base.y)?
            .then(&Self::scale(p.scale.0, p.scale.1)?)?
            .then(&Self::translation(p.offset.0, p.offset.1)?)?
            .then(&Self::rotation_deg(p.rotation_deg)?)?
            .then(&Self::translation(p.at.x, p.at.y)?)
    }

    /// This transform followed by `next`: `self.then(next).apply(p) == next.apply(self.apply(p))`.
    ///
    /// # Errors
    /// [`GeometryError::Overflow`] if an entry of the result is not finite.
    pub fn then(&self, next: &Self) -> Result<Self, GeometryError> {
        let n = next;
        let r = Self {
            a: n.a * self.a + n.b * self.c,
            b: n.a * self.b + n.b * self.d,
            c: n.c * self.a + n.d * self.c,
            d: n.c * self.b + n.d * self.d,
            tx: n.a * self.tx + n.b * self.ty + n.tx,
            ty: n.c * self.tx + n.d * self.ty + n.ty,
        };
        if r.is_finite() {
            Ok(r)
        } else {
            Err(GeometryError::Overflow)
        }
    }

    fn is_finite(&self) -> bool {
        [self.a, self.b, self.c, self.d, self.tx, self.ty]
            .iter()
            .all(|v| v.is_finite())
    }

    /// The image of a point.
    ///
    /// # Errors
    /// [`GeometryError::NonFinite`] for a non-finite point or transform, [`GeometryError::Overflow`]
    /// for a non-finite result (§8.1).
    pub fn apply(&self, p: Point) -> Result<Point, GeometryError> {
        if !(p.is_finite() && self.is_finite()) {
            return Err(GeometryError::NonFinite);
        }
        let q = Point::new(
            self.a * p.x + self.b * p.y + self.tx,
            self.c * p.x + self.d * p.y + self.ty,
        );
        if q.is_finite() {
            Ok(q)
        } else {
            Err(GeometryError::Overflow)
        }
    }

    /// The image of a polyline vertex. A mirror (det < 0) negates the bulge (§11).
    ///
    /// # Errors
    /// [`GeometryError::DegenerateTransform`] when det = 0;
    /// [`GeometryError::CurveUnderNonUniformScale`] for a bulged vertex under a transform that is not
    /// a similarity; otherwise as [`Affine2::apply`].
    pub fn apply_vertex(&self, v: Vertex) -> Result<Vertex, GeometryError> {
        require_finite(&[v.bulge])?;
        let det = self.det();
        if det == 0.0 {
            return Err(GeometryError::DegenerateTransform);
        }
        if v.bulge != 0.0 && self.similarity_scale().is_none() {
            return Err(GeometryError::CurveUnderNonUniformScale);
        }
        let point = self.apply(v.point)?;
        let bulge = if det < 0.0 { -v.bulge } else { v.bulge };
        Ok(Vertex { point, bulge })
    }

    /// The determinant of the linear part, a·d − b·c: the factor by which areas scale.
    #[must_use]
    pub fn det(&self) -> f64 {
        self.a * self.d - self.b * self.c
    }

    /// Whether the transform reverses orientation (det < 0).
    #[must_use]
    pub fn is_mirror(&self) -> bool {
        self.det() < 0.0
    }

    /// The singular values (σ₁, σ₂), σ₁ ≥ σ₂ ≥ 0, of the linear part.
    #[must_use]
    pub fn singular_values(&self) -> (f64, f64) {
        let (q, r) = self.qr();
        (q + r, (q - r).abs())
    }

    /// With E = (a + d)/2, F = (a − d)/2, G = (c + b)/2, H = (c − b)/2: Q = |(E, H)| and
    /// R = |(F, G)|, so that σ₁ = Q + R and σ₂ = |Q − R|.
    fn qr(&self) -> (f64, f64) {
        let e = (self.a + self.d) / 2.0;
        let f = (self.a - self.d) / 2.0;
        let g = (self.c + self.b) / 2.0;
        let h = (self.c - self.b) / 2.0;
        (e.hypot(h), f.hypot(g))
    }

    /// The scale s = √|det| when the transform is a similarity (σ₁ − σ₂ ≤ 10⁻⁹·σ₁) and not
    /// degenerate; `None` otherwise (§11).
    #[must_use]
    pub fn similarity_scale(&self) -> Option<f64> {
        let (q, r) = self.qr();
        // σ₁ − σ₂ = 2·min(Q, R), computed without cancellation.
        let similar = 2.0 * q.min(r) <= SIMILARITY * (q + r);
        let det = self.det();
        (similar && det != 0.0 && det.is_finite()).then(|| det.abs().sqrt())
    }
}
