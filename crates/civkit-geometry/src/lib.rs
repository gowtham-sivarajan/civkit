// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 10:54 IST · last edited 2026-11-27 19:01 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! CivKit geometry: the 2D primitives used by calculations and drawing analysis.
//!
//! Every convention is defined in `docs/GEOMETRY.md` (accepted as D-22, ADR 0010), and each item
//! here cites its section. In short:
//! - one right-handed frame, (x, y) = (Easting, Northing) (§3);
//! - angles in radians: direction θ counter-clockwise from +x, azimuth α clockwise from +y, both in
//!   [0, 2π) (§4); bulge b = tan(Δ/4), positive counter-clockwise;
//! - a per-call tolerance ε = max(10⁻⁹·D, 1024·ulp(M)), returned with every result (§5);
//! - every degenerate case has one defined behaviour, reported or refused, never silent (§8);
//! - unit-agnostic: lengths come back in the input's unit (§2).
//!
//! - 2D affine transformations and block placement, and regions by even-odd nesting (§11, D-28).
//!
//! Not in this crate in 0.1: 3D, public intersections, splines, geodesy.

mod angle;
mod bulge;
mod edges;
mod error;
mod length;
mod point;
mod polygon;
mod region;
mod tolerance;
mod transform;

pub use angle::{
    Direction, azimuth_from_components, azimuth_from_direction, components, direction,
    direction_from_azimuth, distance, normalize_angle,
};
pub use error::{GeometryError, Note};
pub use length::{
    Arc, Circle, Measure, arc_length, circle_area, circle_circumference, polyline_length,
};
pub use point::{Point, Vertex};
pub use polygon::{Centroid, Orientation, PolygonArea, polygon_area, polygon_centroid};
pub use region::{RegionArea, RegionRing, region_area};
pub use tolerance::{Tolerance, ulp};
pub use transform::{Affine2, InsertPlacement};
