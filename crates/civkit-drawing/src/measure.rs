// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 20:52 IST · last edited 2026-11-27 23:16 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! Quantities by effective layer (D-28 / ADR 0016 §6, §7, §10).
//!
//! Expansion's events are consumed as a stream; no expanded drawing is built. Each entity's measures
//! are computed once in its own coordinates (cached per block entity) and placed by transform: area
//! × |det|, length × s under a similarity; straight geometry under a non-uniform scale is re-measured,
//! curves are not supported. Whatever is left out is counted on its layer and named in an issue:
//! nothing is silently omitted.

use civkit_geometry::{
    Affine2, Arc, Circle, GeometryError, Note as GNote, Point, Vertex as GVertex, WorkLimit,
    arc_length, circle_area, circle_circumference, polygon_area_limited, polyline_length,
    region_area_limited, ulp,
};

use crate::ReadError;
use crate::budget::{BIndex, BVec, Budget, Reservation};
use crate::expand::{AnalysisError, Event, ExpandLimits, ExpandStats, Placed, Skip, expand};
use crate::model::{
    Boundary, Drawing, Edge, EdgeKind, Entity, EntityKind, Hatch, NameId, Note, Polyline, Where,
};

/// Limits and settings of an analysis (D-28 §4.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnalysisOptions {
    /// Expansion limits (nesting, placements).
    pub expand: ExpandLimits,
    /// Hard limit on vertices measured or transformed.
    pub max_vertex_work: u64,
    /// Soft limit on the edge pairs the §8.7 validity check examines (GEOMETRY.md); past it, areas are
    /// not computed and are reported.
    pub max_validity_pairs: u64,
    /// Handles listed per issue.
    pub max_handles: usize,
    /// Distinct issues kept; past it, one `DRAWING_REPORT_TRUNCATED`.
    pub max_issues: usize,
    /// List every placed, measured entity (the report's `detail = "entity"`).
    pub entity_rows: bool,
    /// Rows kept in each list (text, dimensions, entities); past it, `DRAWING_REPORT_TRUNCATED`. Totals are
    /// never truncated.
    pub max_rows: usize,
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            expand: ExpandLimits::default(),
            max_vertex_work: 100_000_000,
            max_validity_pairs: 100_000_000,
            max_handles: 20,
            max_issues: 1000,
            entity_rows: false,
            max_rows: 100_000,
        }
    }
}

/// How much an issue matters (API_DESIGN §5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    /// For information.
    Info,
    /// The results need care.
    Caution,
}

/// What an issue is about, beyond its code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Detail {
    /// Nothing more.
    None,
    /// The feature not supported in 0.1 (`NOT_SUPPORTED`).
    Feature(&'static str),
    /// The geometry note (`DRAWING_GEOMETRY_NOTE`).
    Note(&'static str),
    /// An entity type the reader did not interpret.
    EntityType(NameId),
    /// A `$INSUNITS` code.
    UnitCode(i16),
    /// A block (`DRAWING_BLOCK_SCALE_MISMATCH`).
    Block(NameId),
}

/// One aggregated warning: a code, a level, the layer and detail it concerns, how many times it
/// occurred, and the first handles (API_DESIGN §5, D-28 §10).
#[derive(Debug)]
pub struct Issue {
    /// The warning code.
    pub code: &'static str,
    /// Its level.
    pub level: Level,
    /// The effective layer, if it concerns one.
    pub layer: Option<NameId>,
    /// More about it.
    pub detail: Detail,
    /// How many times it occurred.
    pub count: u64,
    handles: BVec<u64>,
}

impl Issue {
    /// The handles of the first occurrences that had one (at most [`AnalysisOptions::max_handles`]).
    pub fn handles(&self) -> &[u64] {
        &self.handles
    }
}

/// Entities measured on one layer, by kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Counts {
    /// LINE.
    pub line: u64,
    /// ARC.
    pub arc: u64,
    /// CIRCLE.
    pub circle: u64,
    /// Open polylines.
    pub polyline_open: u64,
    /// Closed polylines.
    pub polyline_closed: u64,
    /// HATCH (with an area).
    pub hatch: u64,
    /// TEXT (listed, not measured).
    pub text: u64,
    /// DIMENSION (listed, not measured).
    pub dimension: u64,
}

/// Totals of one effective layer, in drawing units. Kinds are never summed together (D-28 §6).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayerTotals {
    /// The layer.
    pub layer: NameId,
    /// Frozen in the LAYER table.
    pub frozen: bool,
    /// Off in the LAYER table.
    pub off: bool,
    /// Entities measured, by kind.
    pub counts: Counts,
    /// Total LINE length.
    pub line_length: f64,
    /// Total ARC length.
    pub arc_length: f64,
    /// Total open-polyline length.
    pub polyline_open_length: f64,
    /// Total closed-polyline perimeter.
    pub polyline_closed_perimeter: f64,
    /// Total closed-polyline area.
    pub polyline_closed_area: f64,
    /// Total circle circumference.
    pub circle_circumference: f64,
    /// Total circle area.
    pub circle_area: f64,
    /// Total hatch area.
    pub hatch_area: f64,
    /// Entities (or measures) left out of these totals; each is named in an issue.
    pub omitted: u64,
}

impl LayerTotals {
    /// Whether anything on this layer was left out of its totals.
    #[must_use]
    pub fn incomplete(&self) -> bool {
        self.omitted > 0
    }
}

/// The drawing's units (D-28 §7).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Units {
    /// `$INSUNITS` as stated; `None` when not stated.
    pub insunits: Option<i16>,
    /// The unit's short name, when CivKit converts it.
    pub name: Option<&'static str>,
    /// Metres per drawing unit, when known; never assumed.
    pub to_metres: Option<f64>,
}

impl Units {
    /// A length in metres, when the units are known.
    #[must_use]
    pub fn length_m(&self, drawing_units: f64) -> Option<f64> {
        self.to_metres.map(|f| drawing_units * f)
    }

    /// An area in square metres, when the units are known.
    #[must_use]
    pub fn area_m2(&self, drawing_units: f64) -> Option<f64> {
        self.to_metres.map(|f| drawing_units * f * f)
    }
}

/// `$INSUNITS` codes converted in 0.1, with exact factors (D-28 §7).
const UNITS: [(i16, &str, f64); 10] = [
    (1, "in", 0.0254),
    (2, "ft", 0.3048),
    (3, "mi", 1609.344),
    (4, "mm", 1e-3),
    (5, "cm", 1e-2),
    (6, "m", 1.0),
    (7, "km", 1e3),
    (10, "yd", 0.9144),
    (14, "dm", 0.1),
    (21, "ft_us", 1200.0 / 3937.0),
];

/// Uninterpreted types that carry no length or area (text, attributes, viewports, points): listed,
/// but they never make a total incomplete.
const NOT_QUANTITIES: [&str; 5] = ["ATTDEF", "ATTRIB", "MTEXT", "POINT", "VIEWPORT"];

/// A TEXT placed in the world. Its value is drawing text, read from the drawing at `block` and `index`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextRow {
    /// Effective layer.
    pub layer: NameId,
    /// Handle.
    pub handle: Option<u64>,
    /// World x of its insertion point.
    pub x: f64,
    /// World y of its insertion point.
    pub y: f64,
    /// Its block, as a position in [`Drawing::blocks`] (none at the top level).
    pub block: Option<usize>,
    /// Its position in that block's entities (or in [`Drawing::entities`]).
    pub index: usize,
}

/// A DIMENSION placed in the world, read from the drawing at `block` and `index`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DimensionRow {
    /// Effective layer.
    pub layer: NameId,
    /// Handle.
    pub handle: Option<u64>,
    /// Its block (none at the top level).
    pub block: Option<usize>,
    /// Its position in that block's entities (or in [`Drawing::entities`]).
    pub index: usize,
}

/// One placed, measured entity (`AnalysisOptions::entity_rows`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EntityRow {
    /// Effective layer.
    pub layer: NameId,
    /// `line`, `arc`, `circle`, `polyline_open`, `polyline_closed` or `hatch`.
    pub kind: &'static str,
    /// Handle.
    pub handle: Option<u64>,
    /// Handle of the top-level entity it comes from.
    pub top_handle: Option<u64>,
    /// Handle of the INSERT that placed it directly.
    pub insert_handle: Option<u64>,
    /// Length, circumference or perimeter, placed, in drawing units.
    pub length: Option<f64>,
    /// Area, placed, in drawing units (none when it has none or it was left out).
    pub area: Option<f64>,
}

/// The quantities of a drawing.
#[derive(Debug)]
pub struct Quantities {
    layers: BVec<LayerTotals>,
    issues: BVec<Issue>,
    units: Units,
    expansion: ExpandStats,
    texts: BVec<TextRow>,
    dimensions: BVec<DimensionRow>,
    entities: BVec<EntityRow>,
    entity_rows: bool,
}

impl Quantities {
    /// One row per effective layer that has measures or omissions, in first-seen order.
    pub fn layers(&self) -> &[LayerTotals] {
        &self.layers
    }

    /// Aggregated issues, in first-seen order.
    pub fn issues(&self) -> &[Issue] {
        &self.issues
    }

    /// The drawing's units.
    pub fn units(&self) -> Units {
        self.units
    }

    /// What expansion did.
    pub fn expansion(&self) -> ExpandStats {
        self.expansion
    }

    /// Placed TEXT, in drawing order (at most `max_rows`).
    pub fn texts(&self) -> &[TextRow] {
        &self.texts
    }

    /// Placed DIMENSION, in drawing order (at most `max_rows`).
    pub fn dimensions(&self) -> &[DimensionRow] {
        &self.dimensions
    }

    /// Placed, measured entities, when asked for (at most `max_rows`).
    pub fn entities(&self) -> Option<&[EntityRow]> {
        self.entity_rows.then_some(&*self.entities)
    }
}

// ---------------------------------------------------------------------------------------- internals

/// Compensated (Neumaier) summation.
#[derive(Debug, Clone, Copy, Default)]
struct Sum {
    sum: f64,
    c: f64,
}

impl Sum {
    fn add(&mut self, x: f64) {
        let t = self.sum + x;
        if self.sum.abs() >= x.abs() {
            self.c += (self.sum - t) + x;
        } else {
            self.c += (x - t) + self.sum;
        }
        self.sum = t;
    }

    fn value(&self) -> f64 {
        self.sum + self.c
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Row {
    layer: Option<NameId>,
    counts: Counts,
    sums: [Sum; 8],
    omitted: u64,
}

const LINE: usize = 0;
const ARC: usize = 1;
const OPEN: usize = 2;
const PERIMETER: usize = 3;
const CLOSED_AREA: usize = 4;
const CIRCUMFERENCE: usize = 5;
const CIRCLE_AREA: usize = 6;
const HATCH_AREA: usize = 7;

/// Why a measure is missing.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Fail {
    Invalid,
    WorkLimit,
    NotSupported(&'static str),
    NotRead,
}

/// Notes from measuring, as bits.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Notes(u8);

const NOTE_NAMES: [&str; 4] = [
    "zero-length segment",
    "degenerate ring",
    "orientation undefined",
    "zero sweep",
];

impl Notes {
    fn from(notes: &[GNote]) -> Self {
        let mut bits = 0;
        for n in notes {
            bits |= match n {
                GNote::ZeroLengthSegment { .. } => 1,
                GNote::DegenerateRing => 2,
                GNote::OrientationUndefined => 4,
                GNote::ZeroSweep => 8,
                _ => 0,
            };
        }
        Self(bits)
    }
}

/// An entity's measures in its own coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Local {
    Circle {
        circumference: f64,
        area: f64,
    },
    Arc {
        length: f64,
        notes: Notes,
    },
    Open {
        length: f64,
        curved: bool,
        notes: Notes,
    },
    Closed {
        perimeter: f64,
        area: Result<f64, Fail>,
        curved: bool,
        notes: Notes,
    },
    Hatch {
        area: Result<f64, Fail>,
        curved: bool,
        notes: Notes,
    },
    Failed(Fail),
    /// TEXT and DIMENSION: listed, not measured.
    Listed,
}

struct Work {
    used: u64,
    limit: u64,
}

impl Work {
    fn take(&mut self, n: usize) -> Result<(), AnalysisError> {
        let n = n as u64;
        match self.used.checked_add(n) {
            Some(u) if u <= self.limit => {
                self.used = u;
                Ok(())
            }
            _ => Err(AnalysisError::Limit {
                what: "vertices",
                limit: self.limit,
            }),
        }
    }
}

fn geometry_fail(e: GeometryError) -> Fail {
    match e {
        GeometryError::WorkLimitExceeded { .. } => Fail::WorkLimit,
        _ => Fail::Invalid,
    }
}

fn gvertices(vertices: &[crate::model::Vertex]) -> impl Iterator<Item = GVertex> + '_ {
    vertices.iter().map(|v| GVertex::new(v.x, v.y, v.bulge))
}

/// Measures an entity in its own coordinates.
fn local(
    entity: &Entity,
    budget: &Budget,
    work: &mut Work,
    pairs: &mut WorkLimit,
) -> Result<Local, AnalysisError> {
    Ok(match &entity.kind {
        EntityKind::Circle { center, radius } => {
            work.take(1)?;
            let c = Circle::new(Point::new(center.x, center.y), *radius);
            match (circle_circumference(&c, None), circle_area(&c, None)) {
                (Ok(l), Ok(a)) => Local::Circle {
                    circumference: l.value,
                    area: a.value,
                },
                _ => Local::Failed(Fail::Invalid),
            }
        }
        EntityKind::Arc {
            center,
            radius,
            start_deg,
            end_deg,
        } => {
            work.take(1)?;
            let sweep = (end_deg - start_deg).rem_euclid(360.0);
            if sweep == 0.0 {
                Local::Arc {
                    length: 0.0,
                    notes: Notes(8),
                }
            } else {
                let a = Arc::new(
                    Point::new(center.x, center.y),
                    *radius,
                    start_deg.to_radians(),
                    sweep.to_radians(),
                );
                match arc_length(&a, None) {
                    Ok(m) => Local::Arc {
                        length: m.value,
                        notes: Notes::from(&m.notes),
                    },
                    Err(_) => Local::Failed(Fail::Invalid),
                }
            }
        }
        EntityKind::Polyline(p) => polyline(p, budget, work, pairs)?,
        EntityKind::Hatch(h) => hatch(h, budget, work, pairs)?,
        EntityKind::Text(_) | EntityKind::Dimension(_) => Local::Listed,
        // Lines are measured per placement; inserts expand.
        _ => Local::Listed,
    })
}

/// Reserves, from the shared budget, what a geometry call may allocate outside it (D-28 §4.1): the vertex copies and
/// the geometry crate's own working arrays, `per_vertex` bytes for each of `n` vertices. Gate A4
/// (`tests/analysis_memory.rs`) measures the real use against these constants.
fn reserve_for_geometry(
    budget: &Budget,
    n: usize,
    per_vertex: usize,
) -> Result<Reservation, AnalysisError> {
    let bytes = n
        .checked_mul(per_vertex)
        .ok_or(ReadError::AllocationFailed { bytes: usize::MAX })?;
    Ok(Reservation::new(budget, bytes)?)
}

/// Per vertex, for one polyline: its copy, polygon cleaning, the §8.7 edges, boxes and sweep. Measured by gate A4 at
/// about 264 bytes (a convex 200 000-vertex ring, 2026-11-27); reserved with margin.
pub(crate) const GEOMETRY_BYTES_PER_VERTEX: usize = 320;

/// Per vertex, for one hatch region: its rings, `region_area`'s own copies, each ring's check and the cross-ring
/// sweep, alive together. Measured by gate A4 at about 504 bytes (a 100 000-vertex polyline path), 512 (a
/// 100 000-edge path) and 327 (500 rings of 200 vertices), 2026-11-27; reserved with margin. Under the default
/// 512 MiB budget, a single hatch boundary above about 750 000 vertices is therefore a `DRAWING_LIMIT`.
pub(crate) const REGION_BYTES_PER_VERTEX: usize = 640;

fn polyline(
    p: &Polyline,
    budget: &Budget,
    work: &mut Work,
    pairs: &mut WorkLimit,
) -> Result<Local, AnalysisError> {
    work.take(p.vertices.len())?;
    let _reserved = reserve_for_geometry(budget, p.vertices.len(), GEOMETRY_BYTES_PER_VERTEX)?;
    let v: Vec<GVertex> = gvertices(&p.vertices).collect();
    let n = v.len();
    let curved_segments = if p.closed { n } else { n.saturating_sub(1) };
    let curved = v.iter().take(curved_segments).any(|x| x.bulge != 0.0);
    let length = match polyline_length(&v, p.closed, None) {
        Ok(m) => m,
        Err(_) => return Ok(Local::Failed(Fail::Invalid)),
    };
    if !p.closed {
        return Ok(Local::Open {
            length: length.value,
            curved,
            notes: Notes::from(&length.notes),
        });
    }
    let (area, notes) = match polygon_area_limited(&v, None, pairs) {
        Ok(a) => (Ok(a.area), Notes::from(&a.notes)),
        Err(e) => (Err(geometry_fail(e)), Notes::from(&length.notes)),
    };
    Ok(Local::Closed {
        perimeter: length.value,
        area,
        curved,
        notes,
    })
}

/// A hatch boundary path as a ring of bulged vertices.
fn path_ring(boundary: &Boundary) -> Result<Vec<GVertex>, Fail> {
    match boundary {
        Boundary::Polyline { vertices, .. } => Ok(gvertices(vertices).collect()),
        Boundary::NotSupported { edge } => Err(Fail::NotSupported(match edge {
            EdgeKind::Ellipse => "hatch ellipse edge",
            EdgeKind::Spline => "hatch spline edge",
        })),
        Boundary::Edges(edges) => {
            // Each edge as (start, end, vertices to add with their bulges).
            let mut ring = Vec::new();
            let mut ends = Vec::new();
            for e in edges.iter() {
                match *e {
                    Edge::Line { start, end } => {
                        ring.push(GVertex::straight(start.x, start.y));
                        ends.push(((start.x, start.y), (end.x, end.y)));
                    }
                    Edge::Arc {
                        center,
                        radius,
                        start_deg,
                        end_deg,
                        counter_clockwise,
                    } => {
                        // The sweep's size is (e − s) mod 360 either way (360 when the ends differ
                        // by whole turns, a full circle).
                        let mut sweep = (end_deg - start_deg).rem_euclid(360.0);
                        if sweep == 0.0 && end_deg - start_deg != 0.0 {
                            sweep = 360.0;
                        }
                        // A clockwise edge (73 = 0) states complementary angles: it runs clockwise
                        // from −s to −e (D-28 §6, as ezdxf's `ArcEdge.load_tags` reads it).
                        let (from, to, turn) = if counter_clockwise {
                            (start_deg, end_deg, 1.0)
                        } else {
                            (-start_deg, -end_deg, -1.0)
                        };
                        let at = |deg: f64| {
                            let (s, c) = deg.to_radians().sin_cos();
                            (center.x + radius * c, center.y + radius * s)
                        };
                        // Two halves, so a full circle (360°) is representable: bulge ±tan(Δ/8)
                        // each, negative when clockwise.
                        let b = turn * (sweep.to_radians() / 8.0).tan();
                        let (p0, pm, p1) = (at(from), at(from + turn * sweep / 2.0), at(to));
                        ring.push(GVertex::new(p0.0, p0.1, b));
                        ring.push(GVertex::new(pm.0, pm.1, b));
                        ends.push((p0, p1));
                    }
                }
            }
            // Consecutive edges must join within ε of the path (GEOMETRY §5 rule over its points).
            let pts = ends.iter().flat_map(|&(a, b)| [a, b]);
            let (mut lo, mut hi, mut m) = ((f64::MAX, f64::MAX), (f64::MIN, f64::MIN), 0.0_f64);
            for (x, y) in pts {
                lo = (lo.0.min(x), lo.1.min(y));
                hi = (hi.0.max(x), hi.1.max(y));
                m = m.max(x.abs()).max(y.abs());
            }
            let d = (hi.0 - lo.0).hypot(hi.1 - lo.1);
            let eps = (1e-9 * d).max(1024.0 * ulp(m));
            for k in 0..ends.len() {
                let (_, end) = ends[k];
                let (next, _) = ends[(k + 1) % ends.len()];
                if (end.0 - next.0).hypot(end.1 - next.1) > eps {
                    return Err(Fail::NotSupported("hatch boundary not joined"));
                }
            }
            Ok(ring)
        }
    }
}

fn hatch(
    h: &Hatch,
    budget: &Budget,
    work: &mut Work,
    pairs: &mut WorkLimit,
) -> Result<Local, AnalysisError> {
    if h.paths.is_empty() {
        return Ok(Local::Failed(Fail::NotRead));
    }
    // Reserve before the rings are built: a ring has one vertex per polyline vertex or line edge, two per arc edge.
    let n: usize = h
        .paths
        .iter()
        .map(|path| match &path.boundary {
            Boundary::Polyline { vertices, .. } => vertices.len(),
            Boundary::Edges(edges) => edges
                .iter()
                .map(|e| if matches!(e, Edge::Arc { .. }) { 2 } else { 1 })
                .sum(),
            Boundary::NotSupported { .. } => 0,
        })
        .sum();
    work.take(n)?;
    let _reserved = reserve_for_geometry(budget, n, REGION_BYTES_PER_VERTEX)?;
    let mut rings = Vec::with_capacity(h.paths.len());
    for path in h.paths.iter() {
        match path_ring(&path.boundary) {
            Ok(r) => rings.push(r),
            Err(f) => return Ok(Local::Failed(f)),
        }
    }
    let curved = rings.iter().flatten().any(|v| v.bulge != 0.0);
    let refs: Vec<&[GVertex]> = rings.iter().map(Vec::as_slice).collect();
    Ok(match region_area_limited(&refs, None, pairs) {
        Ok(r) => {
            let mut bits = Notes::default();
            for ring in &r.rings {
                bits.0 |= Notes::from(&ring.area.notes).0;
            }
            Local::Hatch {
                area: Ok(r.area),
                curved,
                notes: bits,
            }
        }
        Err(e) => Local::Hatch {
            area: Err(match e {
                GeometryError::WorkLimitExceeded { .. } => Fail::WorkLimit,
                _ => Fail::Invalid,
            }),
            curved,
            notes: Notes::default(),
        },
    })
}

/// Whether, placed by `t`, a box of local points is so far from the origin, relative to its size,
/// that precision is limited (GEOMETRY.md §8.9).
fn precision_limited(t: &Affine2, points: &[(f64, f64)]) -> bool {
    let mut lo = (f64::MAX, f64::MAX);
    let mut hi = (f64::MIN, f64::MIN);
    let mut m = 0.0_f64;
    for &(x, y) in points {
        let Ok(p) = t.apply(Point::new(x, y)) else {
            return false;
        };
        lo = (lo.0.min(p.x), lo.1.min(p.y));
        hi = (hi.0.max(p.x), hi.1.max(p.y));
        m = m.max(p.x.abs()).max(p.y.abs());
    }
    let d = (hi.0 - lo.0).hypot(hi.1 - lo.1);
    d > 0.0 && 1024.0 * ulp(m) > 1e-6 * d
}

/// The corners of an entity's local bounding box (arcs and circles by their full circle).
fn local_box(entity: &Entity) -> Vec<(f64, f64)> {
    let mut lo = (f64::MAX, f64::MAX);
    let mut hi = (f64::MIN, f64::MIN);
    let mut add = |x: f64, y: f64| {
        lo = (lo.0.min(x), lo.1.min(y));
        hi = (hi.0.max(x), hi.1.max(y));
    };
    match &entity.kind {
        EntityKind::Line { start, end } => {
            add(start.x, start.y);
            add(end.x, end.y);
        }
        EntityKind::Circle { center, radius } | EntityKind::Arc { center, radius, .. } => {
            add(center.x - radius, center.y - radius);
            add(center.x + radius, center.y + radius);
        }
        EntityKind::Polyline(p) => p.vertices.iter().for_each(|v| add(v.x, v.y)),
        EntityKind::Hatch(h) => {
            for path in h.paths.iter() {
                match &path.boundary {
                    Boundary::Polyline { vertices, .. } => {
                        vertices.iter().for_each(|v| add(v.x, v.y))
                    }
                    Boundary::Edges(edges) => {
                        for e in edges.iter() {
                            match *e {
                                Edge::Line { start, end } => {
                                    add(start.x, start.y);
                                    add(end.x, end.y);
                                }
                                Edge::Arc { center, radius, .. } => {
                                    add(center.x - radius, center.y - radius);
                                    add(center.x + radius, center.y + radius);
                                }
                            }
                        }
                    }
                    Boundary::NotSupported { .. } => {}
                }
            }
        }
        _ => {}
    }
    if lo.0 > hi.0 {
        return Vec::new();
    }
    vec![lo, (hi.0, lo.1), hi, (lo.0, hi.1)]
}

/// Rows, issues and their lookups.
struct Collector<'d> {
    drawing: &'d Drawing,
    options: AnalysisOptions,
    budget: Budget,
    rows: BVec<Row>,
    row_of: BVec<u32>,
    issues: BVec<Issue>,
    issue_index: BIndex,
    dropped: u64,
    /// Smallest and largest uniform scale each block is placed at (P2).
    block_scales: BVec<(f64, f64)>,
    /// Closed areas by (row, kind) for the size check (P3), up to `MAX_AREA_SAMPLES`.
    areas: BVec<(u32, u8, f64, Option<u64>)>,
    areas_capped: bool,
    texts: BVec<TextRow>,
    dimensions: BVec<DimensionRow>,
    entities: BVec<EntityRow>,
    /// Rows left out of the text, dimension and entity lists.
    rows_dropped: [u64; 3],
}

/// Closed areas kept for the size check (P3); past it the check covers the first ones and says so.
const MAX_AREA_SAMPLES: usize = 1_000_000;

impl Collector<'_> {
    fn row(&mut self, layer: NameId) -> Result<usize, AnalysisError> {
        let slot = layer.0 as usize;
        if let Some(&r) = self.row_of.get(slot)
            && r != u32::MAX
        {
            return Ok(r as usize);
        }
        let id = self.rows.len();
        self.rows.push(Row {
            layer: Some(layer),
            ..Row::default()
        })?;
        if let Some(s) = self.row_of.get_mut(slot) {
            *s = u32::try_from(id).unwrap_or(u32::MAX);
        }
        Ok(id)
    }

    fn omit(&mut self, layer: NameId, n: u64) -> Result<(), AnalysisError> {
        let r = self.row(layer)?;
        self.rows[r].omitted += n;
        Ok(())
    }

    fn issue(
        &mut self,
        code: &'static str,
        level: Level,
        layer: Option<NameId>,
        detail: Detail,
        count: u64,
        handle: Option<u64>,
    ) -> Result<(), AnalysisError> {
        let key = (code, layer, detail);
        let hash = self.issue_index.hash(&key);
        let issues = &self.issues;
        let found = self.issue_index.find(hash, |i| {
            let x = &issues[i as usize];
            (x.code, x.layer, x.detail) == key
        });
        let i = match found {
            Some(i) => i as usize,
            None => {
                if self.issues.len() + 1 >= self.options.max_issues {
                    self.dropped += count;
                    return Ok(());
                }
                let id = self.issues.len();
                self.issues.push(Issue {
                    code,
                    level,
                    layer,
                    detail,
                    count: 0,
                    handles: BVec::new(&self.budget),
                })?;
                let issues = &self.issues;
                self.issue_index.insert(
                    hash,
                    u32::try_from(id).unwrap_or(u32::MAX),
                    |h, old| {
                        use std::hash::BuildHasher;
                        let x = &issues[old as usize];
                        h.hash_one((x.code, x.layer, x.detail))
                    },
                )?;
                id
            }
        };
        let max_handles = self.options.max_handles;
        let x = &mut self.issues[i];
        x.count += count;
        x.level = x.level.max(level);
        if let Some(h) = handle
            && x.handles.len() < max_handles
        {
            x.handles.push(h)?;
        }
        Ok(())
    }

    /// Something left out of `layer`'s totals, named by an issue.
    fn left_out(
        &mut self,
        code: &'static str,
        layer: NameId,
        detail: Detail,
        handle: Option<u64>,
    ) -> Result<(), AnalysisError> {
        self.omit(layer, 1)?;
        self.issue(code, Level::Caution, Some(layer), detail, 1, handle)
    }

    fn fail(
        &mut self,
        fail: Fail,
        layer: NameId,
        handle: Option<u64>,
    ) -> Result<(), AnalysisError> {
        match fail {
            Fail::Invalid => self.left_out("DRAWING_GEOMETRY_INVALID", layer, Detail::None, handle),
            Fail::WorkLimit => self.left_out(
                "NOT_SUPPORTED",
                layer,
                Detail::Feature("validity check work limit"),
                handle,
            ),
            Fail::NotSupported(f) => {
                self.left_out("NOT_SUPPORTED", layer, Detail::Feature(f), handle)
            }
            Fail::NotRead => self.left_out("DRAWING_HATCH_NOT_READ", layer, Detail::None, handle),
        }
    }

    fn notes(
        &mut self,
        notes: Notes,
        layer: NameId,
        handle: Option<u64>,
    ) -> Result<(), AnalysisError> {
        for (bit, name) in NOTE_NAMES.iter().enumerate() {
            if notes.0 & (1 << bit) != 0 {
                self.issue(
                    "DRAWING_GEOMETRY_NOTE",
                    Level::Info,
                    Some(layer),
                    Detail::Note(name),
                    1,
                    handle,
                )?;
            }
        }
        Ok(())
    }

    /// Keeps a closed area for the size check (P3).
    fn sample(
        &mut self,
        row: usize,
        kind: u8,
        area: f64,
        handle: Option<u64>,
    ) -> Result<(), AnalysisError> {
        if self.areas.len() >= MAX_AREA_SAMPLES {
            self.areas_capped = true;
            return Ok(());
        }
        self.areas
            .push((u32::try_from(row).unwrap_or(u32::MAX), kind, area, handle))?;
        Ok(())
    }

    /// Lists a placed, measured entity, when entity rows are asked for.
    fn entity_row(
        &mut self,
        p: &Placed<'_>,
        kind: &'static str,
        length: Option<f64>,
        area: Option<f64>,
    ) -> Result<(), AnalysisError> {
        if !self.options.entity_rows {
            return Ok(());
        }
        if self.entities.len() >= self.options.max_rows {
            self.rows_dropped[2] += 1;
            return Ok(());
        }
        self.entities.push(EntityRow {
            layer: p.layer,
            kind,
            handle: p.entity.handle,
            top_handle: p.top_handle,
            insert_handle: p.insert_handle,
            length,
            area,
        })?;
        Ok(())
    }

    fn is_quantity_type(&self, kind: NameId) -> bool {
        !NOT_QUANTITIES.contains(&self.drawing.name(kind))
    }
}

/// Measures a drawing: expands it (a stream of placements), measures each placed entity, and totals
/// the measures by effective layer and kind (D-28 §6). Allocations reserve from `budget`.
///
/// # Errors
/// [`AnalysisError::Limit`] for nesting, placements or vertex work; [`AnalysisError::Memory`] when a
/// reservation is refused. Soft limits and unsupported or invalid geometry are reported in the result,
/// never errors.
pub fn measure(
    drawing: &Drawing,
    budget: &Budget,
    options: &AnalysisOptions,
) -> Result<Quantities, AnalysisError> {
    let mut c = Collector {
        drawing,
        options: *options,
        budget: budget.clone(),
        rows: BVec::new(budget),
        row_of: BVec::filled(budget, drawing.names.len(), u32::MAX)?,
        issues: BVec::new(budget),
        issue_index: BIndex::new(budget),
        dropped: 0,
        block_scales: BVec::filled(budget, drawing.blocks.len(), (f64::INFINITY, 0.0))?,
        areas: BVec::new(budget),
        areas_capped: false,
        texts: BVec::new(budget),
        dimensions: BVec::new(budget),
        entities: BVec::new(budget),
        rows_dropped: [0; 3],
    };
    let mut work = Work {
        used: 0,
        limit: options.max_vertex_work,
    };
    let mut pairs = WorkLimit::new(options.max_validity_pairs);

    // Per-block-entity cache of local measures (D-28 §4.2), one slot per block entity.
    let mut offsets: BVec<usize> = BVec::filled(budget, drawing.blocks.len() + 1, 0)?;
    for (i, b) in drawing.blocks.iter().enumerate() {
        offsets[i + 1] = offsets[i] + b.entities.len();
    }
    let total = offsets.last().copied().unwrap_or(0);
    let mut cache: BVec<Option<Local>> = BVec::filled(budget, total, None)?;
    let mut placed_blocks: BVec<bool> = BVec::filled(budget, drawing.blocks.len(), false)?;
    // Block name → position, for uninterpreted content.
    let mut block_of: BVec<u32> = BVec::filled(budget, drawing.names.len(), u32::MAX)?;
    for (i, b) in drawing.blocks.iter().enumerate() {
        if let Some(s) = block_of.get_mut(b.name.0 as usize) {
            *s = u32::try_from(i).unwrap_or(u32::MAX);
        }
    }

    let mut failure: Option<AnalysisError> = None;
    let stats = expand(drawing, budget, &options.expand, |event| {
        if failure.is_some() {
            return;
        }
        let r = on_event(
            &mut c,
            &mut work,
            &mut pairs,
            &mut cache,
            &offsets,
            &mut placed_blocks,
            event,
        );
        if let Err(e) = r {
            failure = Some(e);
        }
    })?;
    if let Some(e) = failure {
        return Err(e);
    }

    // Uninterpreted content: model space counts against its layer; placed blocks were counted per
    // placement; unplaced blocks and paper space are listed only.
    for &((kind, layer, place), n) in drawing.unsupported.iter() {
        let quantity = c.is_quantity_type(kind);
        let placed = match place {
            Where::Block(b) => block_of
                .get(b.0 as usize)
                .and_then(|&i| placed_blocks.get(i as usize).copied())
                .unwrap_or(false),
            _ => false,
        };
        match place {
            Where::ModelSpace if quantity => {
                c.omit(layer, n)?;
                c.issue(
                    "NOT_SUPPORTED",
                    Level::Caution,
                    Some(layer),
                    Detail::EntityType(kind),
                    n,
                    None,
                )?;
            }
            Where::Block(_) if quantity && placed => {}
            _ => c.issue(
                "NOT_SUPPORTED",
                Level::Info,
                Some(layer),
                Detail::EntityType(kind),
                n,
                None,
            )?,
        }
    }
    for note in drawing.notes.iter() {
        if let Note::TextNotDecoded { count, .. } = note {
            c.issue(
                "DRAWING_TEXT_NOT_DECODED",
                Level::Caution,
                None,
                Detail::None,
                *count,
                None,
            )?;
        }
    }

    // Units (D-28 §7).
    let insunits = drawing.insunits();
    let known = insunits.and_then(|u| UNITS.iter().find(|x| x.0 == u));
    match (insunits, known) {
        (None, _) => c.issue(
            "DRAWING_UNITS_NOT_STATED",
            Level::Caution,
            None,
            Detail::None,
            1,
            None,
        )?,
        (Some(0), _) => {
            c.issue(
                "DRAWING_UNITS_NOT_STATED",
                Level::Caution,
                None,
                Detail::UnitCode(0),
                1,
                None,
            )?;
        }
        (Some(u), None) => {
            c.issue(
                "NOT_SUPPORTED",
                Level::Caution,
                None,
                Detail::UnitCode(u),
                1,
                None,
            )?;
        }
        (Some(_), Some(_)) => {}
    }
    let units = Units {
        insunits,
        name: known.map(|k| k.1),
        to_metres: known.map(|k| k.2),
    };
    unit_check(&mut c, insunits)?;
    for (dropped, list) in
        c.rows_dropped
            .into_iter()
            .zip(["text rows", "dimension rows", "entity rows"])
    {
        if dropped > 0 {
            c.issue(
                "DRAWING_REPORT_TRUNCATED",
                Level::Caution,
                None,
                Detail::Feature(list),
                dropped,
                None,
            )?;
        }
    }
    if c.dropped > 0 {
        let dropped = c.dropped;
        c.options.max_issues = usize::MAX;
        c.issue(
            "DRAWING_REPORT_TRUNCATED",
            Level::Caution,
            None,
            Detail::None,
            dropped,
            None,
        )?;
    }

    let mut layers = BVec::new(budget);
    for row in c.rows.iter() {
        let Some(layer) = row.layer else { continue };
        let entry = drawing.layers.iter().find(|l| l.name == layer);
        layers.push(LayerTotals {
            layer,
            frozen: entry.is_some_and(|l| l.frozen),
            off: entry.is_some_and(|l| l.off),
            counts: row.counts,
            line_length: row.sums[LINE].value(),
            arc_length: row.sums[ARC].value(),
            polyline_open_length: row.sums[OPEN].value(),
            polyline_closed_perimeter: row.sums[PERIMETER].value(),
            polyline_closed_area: row.sums[CLOSED_AREA].value(),
            circle_circumference: row.sums[CIRCUMFERENCE].value(),
            circle_area: row.sums[CIRCLE_AREA].value(),
            hatch_area: row.sums[HATCH_AREA].value(),
            omitted: row.omitted,
        })?;
    }
    Ok(Quantities {
        layers,
        issues: c.issues,
        units,
        expansion: stats,
        texts: c.texts,
        dimensions: c.dimensions,
        entities: c.entities,
        entity_rows: options.entity_rows,
    })
}

#[allow(clippy::too_many_arguments)]
fn on_event(
    c: &mut Collector<'_>,
    work: &mut Work,
    pairs: &mut WorkLimit,
    cache: &mut BVec<Option<Local>>,
    offsets: &[usize],
    placed_blocks: &mut BVec<bool>,
    event: Event<'_>,
) -> Result<(), AnalysisError> {
    match event {
        Event::Placed(p) => place(c, work, pairs, cache, offsets, &p),
        Event::Skipped(s) => {
            let (code, detail) = match s.reason {
                Skip::MissingBlock => ("DRAWING_BLOCK_MISSING", Detail::None),
                Skip::Cycle => ("DRAWING_BLOCK_CYCLE", Detail::None),
                Skip::ZeroScale => ("DRAWING_ZERO_SCALE", Detail::None),
                Skip::NonPlanExtrusion => ("NOT_SUPPORTED", Detail::Feature("non-plan extrusion")),
                Skip::NotFinite => ("NOT_SUPPORTED", Detail::Feature("placement out of range")),
            };
            c.left_out(code, s.layer, detail, s.entity.handle)
        }
        Event::Entered {
            block,
            layer,
            transform,
            ..
        } => {
            if let Some(flag) = placed_blocks.get_mut(block) {
                *flag = true;
            }
            if let (Some(s), Some(range)) =
                (transform.similarity_scale(), c.block_scales.get_mut(block))
            {
                *range = (range.0.min(s), range.1.max(s));
            }
            let drawing = c.drawing;
            let name = drawing.blocks[block].name;
            for &((kind, l, place), n) in drawing.unsupported.iter() {
                if place != Where::Block(name) || !c.is_quantity_type(kind) {
                    continue;
                }
                let effective = if drawing.name(l) == "0" { layer } else { l };
                c.omit(effective, n)?;
                c.issue(
                    "NOT_SUPPORTED",
                    Level::Caution,
                    Some(effective),
                    Detail::EntityType(kind),
                    n,
                    None,
                )?;
            }
            Ok(())
        }
    }
}

fn place(
    c: &mut Collector<'_>,
    work: &mut Work,
    pairs: &mut WorkLimit,
    cache: &mut BVec<Option<Local>>,
    offsets: &[usize],
    p: &Placed<'_>,
) -> Result<(), AnalysisError> {
    let layer = p.layer;
    let handle = p.entity.handle;
    let r = c.row(layer)?;

    // Lines: world (block) coordinates, measured per placement.
    if let EntityKind::Line { start, end } = &p.entity.kind {
        work.take(2)?;
        let t = p.transform;
        let (Ok(a), Ok(b)) = (
            t.apply(Point::new(start.x, start.y)),
            t.apply(Point::new(end.x, end.y)),
        ) else {
            return c.fail(Fail::Invalid, layer, handle);
        };
        let length = (b.x - a.x).hypot(b.y - a.y);
        c.rows[r].counts.line += 1;
        c.rows[r].sums[LINE].add(length);
        c.entity_row(p, "line", Some(length), None)?;
        if precision_limited(&t, &[(start.x, start.y), (end.x, end.y)]) {
            c.issue(
                "ROUNDING_SENSITIVE",
                Level::Caution,
                Some(layer),
                Detail::None,
                1,
                handle,
            )?;
        }
        return Ok(());
    }
    match &p.entity.kind {
        EntityKind::Text(text) => {
            c.rows[r].counts.text += 1;
            if c.texts.len() >= c.options.max_rows {
                c.rows_dropped[0] += 1;
                return Ok(());
            }
            let t = p.plan_transform().unwrap_or(p.transform);
            let at = t
                .apply(Point::new(text.point.x, text.point.y))
                .unwrap_or(Point::new(f64::NAN, f64::NAN));
            c.texts.push(TextRow {
                layer,
                handle,
                x: at.x,
                y: at.y,
                block: p.block,
                index: p.index,
            })?;
            return Ok(());
        }
        EntityKind::Dimension(dim) => {
            c.rows[r].counts.dimension += 1;
            if c.dimensions.len() >= c.options.max_rows {
                c.rows_dropped[1] += 1;
            } else {
                c.dimensions.push(DimensionRow {
                    layer,
                    handle,
                    block: p.block,
                    index: p.index,
                })?;
            }
            return dimension_override(c, dim, layer, handle);
        }
        _ => {}
    }
    let Some(t) = p.plan_transform() else {
        return c.left_out(
            "NOT_SUPPORTED",
            layer,
            Detail::Feature("non-plan extrusion"),
            handle,
        );
    };

    // Local measures, cached per block entity.
    let slot = p.block.and_then(|b| offsets.get(b).map(|o| o + p.index));
    let local_measures = match slot.and_then(|s| cache.get(s).copied().flatten()) {
        Some(l) => {
            work.take(1)?;
            l
        }
        None => {
            let budget = c.budget.clone();
            let l = local(p.entity, &budget, work, pairs)?;
            if let Some(s) = slot
                && let Some(entry) = cache.get_mut(s)
            {
                *entry = Some(l);
            }
            l
        }
    };

    let similarity = t.similarity_scale();
    let det = t.det().abs();
    let curve_unsupported = Fail::NotSupported("curve under non-uniform scale");
    let measured = match local_measures {
        Local::Listed => return Ok(()),
        Local::Failed(f) => return c.fail(f, layer, handle),
        Local::Circle {
            circumference,
            area,
        } => {
            let Some(s) = similarity else {
                return c.fail(curve_unsupported, layer, handle);
            };
            c.rows[r].counts.circle += 1;
            c.rows[r].sums[CIRCUMFERENCE].add(circumference * s);
            c.rows[r].sums[CIRCLE_AREA].add(area * det);
            c.sample(r, 0, area * det, handle)?;
            c.entity_row(p, "circle", Some(circumference * s), Some(area * det))?;
            Notes::default()
        }
        Local::Arc { length, notes } => {
            let Some(s) = similarity else {
                return c.fail(curve_unsupported, layer, handle);
            };
            c.rows[r].counts.arc += 1;
            c.rows[r].sums[ARC].add(length * s);
            c.entity_row(p, "arc", Some(length * s), None)?;
            notes
        }
        Local::Open {
            length,
            curved,
            notes,
        } => {
            let length = match similarity {
                Some(s) => length * s,
                None if curved => return c.fail(curve_unsupported, layer, handle),
                None => match placed_length(p.entity, &t, false, work, &c.budget)? {
                    Some(l) => l,
                    None => return c.fail(Fail::Invalid, layer, handle),
                },
            };
            c.rows[r].counts.polyline_open += 1;
            c.rows[r].sums[OPEN].add(length);
            c.entity_row(p, "polyline_open", Some(length), None)?;
            notes
        }
        Local::Closed {
            perimeter,
            area,
            curved,
            notes,
        } => {
            let perimeter = match similarity {
                Some(s) => perimeter * s,
                None if curved => return c.fail(curve_unsupported, layer, handle),
                None => match placed_length(p.entity, &t, true, work, &c.budget)? {
                    Some(l) => l,
                    None => return c.fail(Fail::Invalid, layer, handle),
                },
            };
            c.rows[r].counts.polyline_closed += 1;
            c.rows[r].sums[PERIMETER].add(perimeter);
            match area {
                Ok(a) => {
                    c.rows[r].sums[CLOSED_AREA].add(a * det);
                    c.sample(r, 1, a * det, handle)?;
                }
                Err(f) => c.fail(f, layer, handle)?,
            }
            c.entity_row(
                p,
                "polyline_closed",
                Some(perimeter),
                area.ok().map(|a| a * det),
            )?;
            notes
        }
        Local::Hatch {
            area,
            curved,
            notes,
        } => {
            if similarity.is_none() && curved {
                return c.fail(curve_unsupported, layer, handle);
            }
            match area {
                Ok(a) => {
                    c.rows[r].counts.hatch += 1;
                    c.rows[r].sums[HATCH_AREA].add(a * det);
                    c.sample(r, 2, a * det, handle)?;
                    c.entity_row(p, "hatch", None, Some(a * det))?;
                }
                Err(f) => return c.fail(f, layer, handle),
            }
            notes
        }
    };
    c.notes(measured, layer, handle)?;
    if precision_limited(&t, &local_box(p.entity)) {
        c.issue(
            "ROUNDING_SENSITIVE",
            Level::Caution,
            Some(layer),
            Detail::None,
            1,
            handle,
        )?;
    }
    Ok(())
}

/// The length of a straight-edged polyline placed by a non-uniform transform.
fn placed_length(
    entity: &Entity,
    t: &Affine2,
    closed: bool,
    work: &mut Work,
    budget: &Budget,
) -> Result<Option<f64>, AnalysisError> {
    let EntityKind::Polyline(p) = &entity.kind else {
        return Ok(None);
    };
    work.take(p.vertices.len())?;
    let _reserved = reserve_for_geometry(budget, p.vertices.len(), GEOMETRY_BYTES_PER_VERTEX)?;
    let placed: Result<Vec<GVertex>, GeometryError> = gvertices(&p.vertices)
        .map(|v| t.apply_vertex(GVertex::straight(v.point.x, v.point.y)))
        .collect();
    Ok(placed
        .ok()
        .and_then(|v| polyline_length(&v, closed, None).ok())
        .map(|m| m.value))
}

// --------------------------------------------------------------------------- the unit check (D-28 §8)

/// `$INSUNITS` codes that are imperial and metric (P1).
const IMPERIAL: [i16; 5] = [1, 2, 3, 10, 21];
const METRIC: [i16; 5] = [4, 5, 6, 7, 14];

/// P1–P3, after measuring. (P4 is checked as each dimension is placed.)
fn unit_check(c: &mut Collector<'_>, insunits: Option<i16>) -> Result<(), AnalysisError> {
    // P1: the header contradicts itself.
    if let (Some(u), Some(m)) = (insunits, c.drawing.measurement())
        && ((IMPERIAL.contains(&u) && m == 1) || (METRIC.contains(&u) && m == 0))
    {
        c.issue(
            "DRAWING_UNITS_CONFLICT",
            Level::Caution,
            None,
            Detail::UnitCode(u),
            1,
            None,
        )?;
    }

    // P2: one block placed at scales a ratio of two length units apart (≥ 10, within 1 %).
    let drawing = c.drawing;
    for b in 0..c.block_scales.len() {
        let (lo, hi) = c.block_scales[b];
        if !(lo > 0.0 && hi.is_finite()) {
            continue;
        }
        let ratio = hi / lo;
        let near_unit_ratio = UNITS.iter().any(|x| {
            UNITS.iter().any(|y| {
                let r = x.2 / y.2;
                r >= 10.0 && (ratio / r - 1.0).abs() <= 0.01
            })
        });
        if near_unit_ratio {
            let name = drawing.blocks[b].name;
            c.issue(
                "DRAWING_BLOCK_SCALE_MISMATCH",
                Level::Caution,
                None,
                Detail::Block(name),
                1,
                None,
            )?;
        }
    }

    // P3: an area 10³ times the median of its layer and kind, or less (at least 3 areas).
    c.areas
        .sort_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)).then(a.2.total_cmp(&b.2)));
    let mut start = 0;
    while start < c.areas.len() {
        let key = (c.areas[start].0, c.areas[start].1);
        let mut end = start;
        while end < c.areas.len() && (c.areas[end].0, c.areas[end].1) == key {
            end += 1;
        }
        let n = end - start;
        if n >= 3 {
            let mid = start + n / 2;
            let median = if n % 2 == 1 {
                c.areas[mid].2
            } else {
                (c.areas[mid - 1].2 + c.areas[mid].2) / 2.0
            };
            for k in start..end {
                let (row, _, area, handle) = c.areas[k];
                if median > 0.0 && (area >= 1e3 * median || area <= 1e-3 * median) {
                    let layer = c.rows[row as usize].layer;
                    c.issue(
                        "DRAWING_SIZE_OUTLIER",
                        Level::Caution,
                        layer,
                        Detail::None,
                        1,
                        handle,
                    )?;
                }
            }
        }
        start = end;
    }
    if c.areas_capped {
        c.issue(
            "NOT_SUPPORTED",
            Level::Info,
            None,
            Detail::Feature("size check sample limit"),
            1,
            None,
        )?;
    }
    Ok(())
}

/// What a dimension's text override says.
#[derive(Debug, PartialEq)]
enum Override {
    /// No override, or the measured value shown (`<>`).
    Measured,
    /// A number.
    Number(f64),
    /// Something else.
    Other,
}

/// Reads an override as MTEXT (Autodesk's formatting codes):
/// - `\X` splits dimension text; what follows it goes below the dimension line, so only the part
///   before it is the value;
/// - `\\`, `\{` and `\}` are a literal backslash and braces; other braces group formatting and go;
/// - the one-letter codes `\P` (paragraph), `\L`, `\l`, `\O`, `\o`, `\K`, `\k` (underline, overline,
///   strike-through on and off) go; `\~` is a space;
/// - every other code (`\A`, `\C`, `\F`, `\f`, `\H`, `\Q`, `\S`, `\T`, `\W`, …) has a payload ending at
///   `;`, and the code and its payload go.
fn read_override(text: &str) -> Override {
    // The value is the text before the first unescaped `\X` (a `\\` is an escaped backslash, so the
    // `X` after it is text); the `<>` placeholder counts only there.
    let mut end = text.len();
    let mut scan = text.char_indices();
    while let Some((i, ch)) = scan.next() {
        if ch == '\\' {
            match scan.next() {
                Some((_, 'X')) | None => {
                    end = i;
                    break;
                }
                Some(_) => {}
            }
        }
    }
    let value = &text[..end];
    if value.trim().is_empty() || value.contains("<>") {
        return Override::Measured;
    }
    let mut plain = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        match ch {
            '{' | '}' => {}
            '\\' => match chars.next() {
                Some('X') | None => break,
                Some(c @ ('\\' | '{' | '}')) => plain.push(c),
                Some('~') => plain.push(' '),
                Some('P' | 'p' | 'L' | 'l' | 'O' | 'o' | 'K' | 'k') => {}
                Some(_) => {
                    for c in chars.by_ref() {
                        if c == ';' {
                            break;
                        }
                    }
                }
            },
            _ => plain.push(ch),
        }
    }
    match plain.trim().parse::<f64>() {
        Ok(v) if v.is_finite() => Override::Number(v),
        _ => Override::Other,
    }
}

/// P4: a dimension whose override text is a number other than its stated measurement.
fn dimension_override(
    c: &mut Collector<'_>,
    dim: &crate::model::Dimension,
    layer: NameId,
    handle: Option<u64>,
) -> Result<(), AnalysisError> {
    let Some(text) = dim.text.as_deref() else {
        return Ok(());
    };
    let _reserved = Reservation::new(&c.budget, text.len())?;
    let level = match (read_override(text), dim.measurement) {
        (Override::Measured, _) => return Ok(()),
        (Override::Number(v), Some(m)) if (v - m).abs() <= 1e-9 * m.abs() => return Ok(()),
        (Override::Number(_), Some(_)) => Level::Caution,
        _ => Level::Info,
    };
    c.issue(
        "DRAWING_DIMENSION_OVERRIDE",
        level,
        Some(layer),
        Detail::None,
        1,
        handle,
    )
}
