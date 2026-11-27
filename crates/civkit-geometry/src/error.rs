// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 18:37 IST · last edited 2026-11-27 19:18 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! Errors and degeneracy notes (docs/GEOMETRY.md §8). Rust types only in 0.1 (D-22, D3).

use std::fmt;

/// Why a geometry operation has no result. Each variant names its section of docs/GEOMETRY.md.
///
/// Through the calculation contract, [`GeometryError::is_invalid_value`] errors map to `INVALID_VALUE`
/// and all others to `OUT_OF_DOMAIN` (GEOMETRY.md §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum GeometryError {
    /// A coordinate, radius, bulge, angle or distance is NaN or infinite (§8.1).
    NonFinite,
    /// A result is too large for `f64` (§8.1).
    Overflow,
    /// An explicit tolerance is NaN, infinite or negative (§5).
    InvalidTolerance,
    /// A distance passed to [`crate::components`] is negative (§7).
    NegativeDistance,
    /// The two points are equal within the tolerance, so the direction is undefined (§7).
    CoincidentPoints,
    /// Both components are exactly zero, so the direction is undefined (§7).
    UndefinedDirection,
    /// Too few vertices remain (§8.5). `found` counts the cleaned ring for polygons.
    TooFewVertices {
        /// Vertices present.
        found: usize,
        /// Vertices required.
        required: usize,
    },
    /// A bulged segment whose chord is within tolerance but whose arc is not (§8.2a).
    /// `segment` is the index of its start vertex.
    IllConditionedBulge {
        /// Index of the segment's start vertex.
        segment: usize,
    },
    /// The centroid of a degenerate ring is undefined (§8.6).
    DegenerateRing,
    /// The ring intersects or touches itself (§8.7). The two edges are named by their start
    /// vertices in the input.
    SelfIntersection {
        /// Start vertex of the first edge.
        first: usize,
        /// Start vertex of the second edge.
        second: usize,
    },
    /// The centroid needs a defined orientation (§10).
    UndefinedOrientation,
    /// A negative radius (§8.8).
    NegativeRadius,
    /// A radius within tolerance of zero with a non-zero sweep, or a circle of zero radius (§8.8).
    RadiusTooSmall,
    /// An arc sweep of 2π or more; use a circle instead (§8.8).
    SweepOutOfRange,
    /// The centroid of a ring with bulged edges is not supported in 0.1 (§10, D5).
    CurvedCentroidUnsupported,
    /// A transform with determinant 0 cannot map a polyline vertex (§11).
    DegenerateTransform,
    /// A curved segment under a scale that is not uniform would become elliptical; not supported
    /// in 0.1 (§11).
    CurveUnderNonUniformScale,
    /// A region needs at least one ring (§11).
    NoRings,
    /// Two rings of a region cross or come within the tolerance of each other (§11). They are named
    /// by their positions in the input.
    RingsIntersect {
        /// The first ring.
        first: usize,
        /// The second ring.
        second: usize,
    },
    /// The §8.7 check needs more edge-pair examinations than the caller's [`crate::WorkLimit`]
    /// allows. The check was not completed: the geometry is neither valid nor self-intersecting.
    WorkLimitExceeded {
        /// The limit that was reached.
        limit: u64,
    },
}

impl GeometryError {
    /// True for errors caused by a malformed value rather than by geometry (contract code
    /// `INVALID_VALUE`); false for undefined results (`OUT_OF_DOMAIN`).
    #[must_use]
    pub fn is_invalid_value(&self) -> bool {
        matches!(self, Self::NonFinite | Self::InvalidTolerance)
    }

    /// The section of docs/GEOMETRY.md that defines this error.
    #[must_use]
    pub fn section(&self) -> &'static str {
        match self {
            Self::NonFinite | Self::Overflow => "§8.1",
            Self::InvalidTolerance => "§5",
            Self::NegativeDistance | Self::CoincidentPoints | Self::UndefinedDirection => "§7",
            Self::TooFewVertices { .. } => "§8.5",
            Self::IllConditionedBulge { .. } => "§8.2a",
            Self::DegenerateRing => "§8.6",
            Self::SelfIntersection { .. } | Self::WorkLimitExceeded { .. } => "§8.7",
            Self::UndefinedOrientation | Self::CurvedCentroidUnsupported => "§10",
            Self::NegativeRadius | Self::RadiusTooSmall | Self::SweepOutOfRange => "§8.8",
            Self::DegenerateTransform
            | Self::CurveUnderNonUniformScale
            | Self::NoRings
            | Self::RingsIntersect { .. } => "§11",
        }
    }
}

impl fmt::Display for GeometryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::NonFinite => "a value is not a finite number".to_owned(),
            Self::Overflow => "the result is too large to represent".to_owned(),
            Self::InvalidTolerance => {
                "the tolerance must be a finite number of at least 0".to_owned()
            }
            Self::NegativeDistance => "the distance must not be negative".to_owned(),
            Self::CoincidentPoints => {
                "the points coincide, so the direction is undefined".to_owned()
            }
            Self::UndefinedDirection => {
                "both components are zero, so the direction is undefined".to_owned()
            }
            Self::TooFewVertices { found, required } => {
                format!("{found} vertices remain, at least {required} are needed")
            }
            Self::IllConditionedBulge { segment } => format!(
                "segment {segment} has a chord within tolerance but an arc that is not; its radius is undefined"
            ),
            Self::DegenerateRing => "the ring has no area: it lies along one line".to_owned(),
            Self::SelfIntersection { first, second } => {
                format!("the ring intersects or touches itself (edges {first} and {second})")
            }
            Self::UndefinedOrientation => "the ring's orientation is undefined".to_owned(),
            Self::NegativeRadius => "the radius must not be negative".to_owned(),
            Self::RadiusTooSmall => "the radius is zero within tolerance".to_owned(),
            Self::SweepOutOfRange => "an arc sweep must be less than 2π; use a circle".to_owned(),
            Self::CurvedCentroidUnsupported => {
                "the centroid of a ring with curved edges is not supported in 0.1".to_owned()
            }
            Self::DegenerateTransform => {
                "the transform has determinant 0, so it cannot map a vertex".to_owned()
            }
            Self::CurveUnderNonUniformScale => {
                "a curved segment under a non-uniform scale is not supported in 0.1".to_owned()
            }
            Self::NoRings => "a region needs at least one ring".to_owned(),
            Self::RingsIntersect { first, second } => {
                format!("rings {first} and {second} of the region cross or touch")
            }
            Self::WorkLimitExceeded { limit } => format!(
                "the intersection check needs more than {limit} edge-pair examinations, so it was not completed"
            ),
        };
        write!(f, "{text} ({})", self.section())
    }
}

impl std::error::Error for GeometryError {}

/// A degeneracy that was handled and is reported with the result (docs/GEOMETRY.md §8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Note {
    /// The segment starting at this vertex is zero-length (§8.2). The explicit closing duplicate
    /// of a ring is not reported (§8.4).
    ZeroLengthSegment {
        /// Index of the segment's start vertex.
        segment: usize,
    },
    /// The arc has zero sweep, so its length is 0 (§8.8).
    ZeroSweep,
    /// The ring lies along one line; its area is returned as computed (§8.6).
    DegenerateRing,
    /// The ring is valid but its signed area is within the area tolerance, so its orientation is
    /// undefined (§10).
    OrientationUndefined,
    /// The coordinates are so large, relative to the object, that precision is limited (§8.9).
    PrecisionLimited,
}
