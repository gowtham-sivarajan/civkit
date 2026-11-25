// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 18:37 IST · last edited 2026-11-25 18:37 IST

//! Angle conventions, distance and direction (docs/GEOMETRY.md §4, §7).

use std::f64::consts::{FRAC_PI_2, TAU};

use crate::error::GeometryError;
use crate::point::Point;
use crate::tolerance::{Tolerance, magnitude};

/// §4 normalisation of a finite angle into [0, 2π): `rem_euclid`, then TAU → 0, then −0 → +0.
pub(crate) fn wrap(a: f64) -> f64 {
    let r = a.rem_euclid(TAU);
    let r = if r >= TAU { 0.0 } else { r };
    r + 0.0
}

fn finite(values: &[f64]) -> Result<(), GeometryError> {
    if values.iter().all(|v| v.is_finite()) {
        Ok(())
    } else {
        Err(GeometryError::NonFinite)
    }
}

/// Normalise an angle in radians into [0, 2π) (§4). No value is snapped by a tolerance.
///
/// # Errors
/// [`GeometryError::NonFinite`] for NaN or ±∞.
pub fn normalize_angle(a: f64) -> Result<f64, GeometryError> {
    finite(&[a])?;
    Ok(wrap(a))
}

/// Azimuth α (clockwise from North) from direction angle θ (counter-clockwise from East):
/// α = normalise(π/2 − θ) (§4).
///
/// # Errors
/// [`GeometryError::NonFinite`] for NaN or ±∞.
pub fn azimuth_from_direction(theta: f64) -> Result<f64, GeometryError> {
    finite(&[theta])?;
    Ok(wrap(FRAC_PI_2 - theta))
}

/// Direction angle θ from azimuth α: θ = normalise(π/2 − α) (§4).
///
/// # Errors
/// [`GeometryError::NonFinite`] for NaN or ±∞.
pub fn direction_from_azimuth(alpha: f64) -> Result<f64, GeometryError> {
    finite(&[alpha])?;
    Ok(wrap(FRAC_PI_2 - alpha))
}

/// Azimuth from components: α = atan2(Δx, Δy), East component first (§4, §7).
///
/// # Errors
/// [`GeometryError::NonFinite`]; [`GeometryError::UndefinedDirection`] when both components are
/// exactly zero. Whether a computed difference counts as zero is the caller's decision (§7).
pub fn azimuth_from_components(dx: f64, dy: f64) -> Result<f64, GeometryError> {
    finite(&[dx, dy])?;
    if dx == 0.0 && dy == 0.0 {
        return Err(GeometryError::UndefinedDirection);
    }
    Ok(wrap(dx.atan2(dy)))
}

/// Components from azimuth and distance: (Δx, Δy) = (d·sin α, d·cos α), i.e. (departure, latitude)
/// (§4). A component of −0 is returned as +0.
///
/// # Errors
/// [`GeometryError::NonFinite`]; [`GeometryError::NegativeDistance`].
pub fn components(azimuth: f64, distance: f64) -> Result<(f64, f64), GeometryError> {
    finite(&[azimuth, distance])?;
    if distance < 0.0 {
        return Err(GeometryError::NegativeDistance);
    }
    let (s, c) = azimuth.sin_cos();
    Ok((distance * s + 0.0, distance * c + 0.0))
}

/// Distance between two points, d = hypot(Δx, Δy) (§7).
///
/// # Errors
/// [`GeometryError::NonFinite`]; [`GeometryError::Overflow`].
pub fn distance(a: Point, b: Point) -> Result<f64, GeometryError> {
    finite(&[a.x, a.y, b.x, b.y])?;
    let d = a.distance(b);
    if d.is_finite() {
        Ok(d)
    } else {
        Err(GeometryError::Overflow)
    }
}

/// Direction from one point to another (§7).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Direction {
    /// Azimuth α in [0, 2π), clockwise from North.
    pub azimuth: f64,
    /// Direction angle θ in [0, 2π), counter-clockwise from East.
    pub theta: f64,
    /// Distance d.
    pub distance: f64,
    /// The tolerance ε used for the coincident-point test.
    pub tolerance: f64,
}

/// Direction and distance from `from` to `to` (§7). The default tolerance is the two-point rule
/// ε = max(10⁻⁹·d, 1024·ulp(M)); an explicit ε may be passed.
///
/// # Errors
/// [`GeometryError::NonFinite`]; [`GeometryError::Overflow`]; [`GeometryError::InvalidTolerance`];
/// [`GeometryError::CoincidentPoints`] when d ≤ ε.
pub fn direction(
    from: Point,
    to: Point,
    tolerance: Option<f64>,
) -> Result<Direction, GeometryError> {
    let d = distance(from, to)?;
    let t = Tolerance::compute(d, magnitude([from, to]), tolerance)?;
    if d <= t.linear {
        return Err(GeometryError::CoincidentPoints);
    }
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    Ok(Direction {
        azimuth: wrap(dx.atan2(dy)),
        theta: wrap(dy.atan2(dx)),
        distance: d,
        tolerance: t.linear,
    })
}
