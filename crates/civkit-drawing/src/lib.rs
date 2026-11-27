// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 10:54 IST · last edited 2026-11-27 22:05 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! CivKit drawing: the CAD-neutral drawing model and CivKit's own DXF reader (D-12 / ADR 0002; D-27 / ADR 0015).
//!
//! [`read_dxf`] reads ASCII DXF (R12–R2018) into a [`Drawing`]: header values (with "not stated" kept apart from any
//! stated value), layers, blocks and the entities 0.1 reads (LINE, CIRCLE, ARC, LWPOLYLINE and old 2D POLYLINE,
//! TEXT, INSERT including MINSERT arrays, HATCH boundaries, DIMENSION). Every other entity is counted by type and
//! layer ([`Drawing::unsupported`]); what could not be fully read is noted ([`Drawing::notes`]).
//!
//! Memory: every heap allocation the reader makes is reserved from a [`Budget`] first, so the heap it requests never
//! passes the budget. That is a bound on reader-requested heap, not on process memory. Analysis of the drawing (Phase
//! 4 task 3) reserves from the same budget.
//!
//! Test drawings: `tests/fixtures/dxf/`.

mod budget;
mod decode;
mod error;
mod expand;
mod measure;
mod model;
mod pairs;
mod reader;
mod report;

pub use budget::{BStr, BVec, Budget};
pub use error::ReadError;
pub use expand::{AnalysisError, Event, ExpandLimits, ExpandStats, Placed, Skip, Skipped, expand};
pub use measure::{
    AnalysisOptions, Counts, Detail, DimensionRow, EntityRow, Issue, LayerTotals, Level,
    Quantities, TextRow, Units, measure,
};
pub use model::{
    Block, Boundary, Dimension, Drawing, Edge, EdgeKind, Entity, EntityKind, Hatch, HatchPath,
    Insert, Layer, NameId, Note, Point2, Point3, Polyline, PolylineSource, Text, Vertex, Where,
};
pub use reader::{Limits, read_dxf, read_dxf_with};
pub use report::{Analysis, REPORT_ID, REPORT_VERSION, analyze, analyze_dxf};
