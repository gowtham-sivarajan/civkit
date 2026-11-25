// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 18:37 IST · last edited 2026-11-25 18:37 IST

//! Points and polyline vertices (docs/GEOMETRY.md §3, §4).

/// A point in the plane: x = Easting (right), y = Northing (up) (§3).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    /// Easting, or x in a drawing.
    pub x: f64,
    /// Northing, or y in a drawing.
    pub y: f64,
}

impl Point {
    /// A point from (x, y) = (E, N).
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub(crate) fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    pub(crate) fn minus(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }

    pub(crate) fn distance(self, other: Self) -> f64 {
        (self.x - other.x).hypot(self.y - other.y)
    }

    /// Exact identity of the stored coordinates (used to recognise a shared vertex, never to
    /// compare computed geometry).
    pub(crate) fn same(self, other: Self) -> bool {
        self.x.to_bits() == other.x.to_bits() && self.y.to_bits() == other.y.to_bits()
    }
}

/// A polyline or ring vertex. `bulge` describes the segment from this vertex to the next:
/// b = tan(Δ/4), 0 for a straight segment, positive for a counter-clockwise arc (§4).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    /// The vertex position.
    pub point: Point,
    /// Bulge of the segment that starts here.
    pub bulge: f64,
}

impl Vertex {
    /// A vertex whose following segment has the given bulge.
    #[must_use]
    pub const fn new(x: f64, y: f64, bulge: f64) -> Self {
        Self {
            point: Point::new(x, y),
            bulge,
        }
    }

    /// A vertex whose following segment is straight.
    #[must_use]
    pub const fn straight(x: f64, y: f64) -> Self {
        Self::new(x, y, 0.0)
    }
}
