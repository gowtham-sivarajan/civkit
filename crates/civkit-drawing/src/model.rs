// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 15:40 IST · last edited 2026-11-27 21:10 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! The parsed drawing: what the DXF reader keeps (ADR 0015, Q-C). It is the file's content in DXF terms, not results:
//! coordinates in drawing units and in each entity's own coordinate system (OCS, by its extrusion), angles in degrees
//! as the file states them. Block expansion, transformations and lengths and areas are analysis (Phase 4 task 3).

use crate::budget::{BStr, BVec};

/// A name the drawing uses (layer, block, entity type, header string), looked up with [`Drawing::name`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NameId(pub(crate) u32);

/// A point with three coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point3 {
    /// x
    pub x: f64,
    /// y
    pub y: f64,
    /// z
    pub z: f64,
}

/// A point with two coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point2 {
    /// x
    pub x: f64,
    /// y
    pub y: f64,
}

/// One polyline vertex. A non-zero bulge makes the segment to the next vertex an arc of included angle 4·atan(bulge),
/// counter-clockwise when positive.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vertex {
    /// x
    pub x: f64,
    /// y
    pub y: f64,
    /// Bulge of the segment that starts here.
    pub bulge: f64,
}

/// One drawing entity the reader interprets.
#[derive(Debug)]
pub struct Entity {
    /// Its layer.
    pub layer: NameId,
    /// Its handle (group 5), if the file gives a valid one.
    pub handle: Option<u64>,
    /// Whether it is in paper space (group 67 = 1).
    pub paper_space: bool,
    /// The normal of its plane (groups 210/220/230); (0, 0, 1) when not stated.
    pub extrusion: Point3,
    /// What it is.
    pub kind: EntityKind,
}

/// The entity types 0.1 reads (ADR 0015). Every other type is counted, see [`Drawing::unsupported`].
#[derive(Debug)]
#[non_exhaustive]
pub enum EntityKind {
    /// LINE.
    Line {
        /// Start point.
        start: Point3,
        /// End point.
        end: Point3,
    },
    /// CIRCLE.
    Circle {
        /// Centre (OCS).
        center: Point3,
        /// Radius.
        radius: f64,
    },
    /// ARC, counter-clockwise from the start angle to the end angle (OCS).
    Arc {
        /// Centre (OCS).
        center: Point3,
        /// Radius.
        radius: f64,
        /// Start angle in degrees, as stated.
        start_deg: f64,
        /// End angle in degrees, as stated.
        end_deg: f64,
    },
    /// LWPOLYLINE, or an old 2D POLYLINE with its VERTEX entities.
    Polyline(Polyline),
    /// TEXT.
    Text(Text),
    /// INSERT, including a MINSERT array (columns × rows).
    Insert(Insert),
    /// HATCH, with its boundary paths.
    Hatch(Hatch),
    /// DIMENSION.
    Dimension(Dimension),
}

impl EntityKind {
    /// The DXF type name this entity was read from.
    pub fn dxf_name(&self) -> &'static str {
        match self {
            Self::Line { .. } => "LINE",
            Self::Circle { .. } => "CIRCLE",
            Self::Arc { .. } => "ARC",
            Self::Polyline(p) => match p.source {
                PolylineSource::Lightweight => "LWPOLYLINE",
                PolylineSource::Old => "POLYLINE",
            },
            Self::Text(_) => "TEXT",
            Self::Insert(_) => "INSERT",
            Self::Hatch(_) => "HATCH",
            Self::Dimension(_) => "DIMENSION",
        }
    }
}

/// Which DXF entity a polyline came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolylineSource {
    /// LWPOLYLINE.
    Lightweight,
    /// An old 2D POLYLINE with VERTEX entities and SEQEND.
    Old,
}

/// A 2D polyline (OCS).
#[derive(Debug)]
pub struct Polyline {
    /// Vertices in order.
    pub vertices: BVec<Vertex>,
    /// Whether the last vertex joins the first.
    pub closed: bool,
    /// Elevation (z in the OCS).
    pub elevation: f64,
    /// LWPOLYLINE or old POLYLINE.
    pub source: PolylineSource,
}

/// TEXT.
#[derive(Debug)]
pub struct Text {
    /// Insertion point (group 10).
    pub point: Point3,
    /// Text height.
    pub height: f64,
    /// Rotation in degrees, as stated.
    pub rotation_deg: f64,
    /// The text, decoded (ADR 0015 encodings).
    pub value: BStr,
}

/// INSERT: a block placed once, or as an array of `columns × rows` (a MINSERT).
#[derive(Debug)]
pub struct Insert {
    /// The block placed.
    pub block: NameId,
    /// Insertion point (OCS).
    pub point: Point3,
    /// Scale factors (groups 41, 42, 43); 1 when not stated. Negative mirrors.
    pub scale: Point3,
    /// Rotation in degrees, as stated.
    pub rotation_deg: f64,
    /// Columns of the array (group 70); 1 when not stated (a count below 1 is read as 1).
    pub columns: u32,
    /// Rows of the array (group 71); 1 when not stated (a count below 1 is read as 1).
    pub rows: u32,
    /// Distance between columns (group 44).
    pub column_spacing: f64,
    /// Distance between rows (group 45).
    pub row_spacing: f64,
}

/// HATCH.
#[derive(Debug)]
pub struct Hatch {
    /// Pattern name (group 2).
    pub pattern: NameId,
    /// Whether it is a solid fill (group 70 = 1).
    pub solid: bool,
    /// Elevation (z of group 10/20/30).
    pub elevation: f64,
    /// Boundary paths in order.
    pub paths: BVec<HatchPath>,
}

/// One hatch boundary path.
#[derive(Debug)]
pub struct HatchPath {
    /// The path type flags (group 92): 1 external, 2 polyline, 16 outermost, …
    pub flags: i32,
    /// The path's geometry.
    pub boundary: Boundary,
}

/// A hatch boundary.
#[derive(Debug)]
pub enum Boundary {
    /// A polyline path.
    Polyline {
        /// Vertices in order (with bulges).
        vertices: BVec<Vertex>,
        /// Whether it is closed (group 73).
        closed: bool,
    },
    /// A path of line and arc edges.
    Edges(BVec<Edge>),
    /// A path with an edge kind 0.1 does not read; the hatch gets no area from it (and a [`Note`] says so).
    NotSupported {
        /// The first unsupported edge kind in the path.
        edge: EdgeKind,
    },
}

/// One edge of a hatch boundary path.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Edge {
    /// A straight edge.
    Line {
        /// Start.
        start: Point2,
        /// End.
        end: Point2,
    },
    /// A circular arc edge.
    Arc {
        /// Centre.
        center: Point2,
        /// Radius.
        radius: f64,
        /// Start angle in degrees, as stated.
        start_deg: f64,
        /// End angle in degrees, as stated.
        end_deg: f64,
        /// Whether it runs counter-clockwise (group 73).
        counter_clockwise: bool,
    },
}

/// Hatch edge kinds 0.1 does not read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind {
    /// An elliptic arc edge.
    Ellipse,
    /// A spline edge.
    Spline,
}

/// DIMENSION.
#[derive(Debug)]
pub struct Dimension {
    /// The dimension type (group 70, low 3 bits): 0 rotated/horizontal/vertical, 1 aligned, 2 angular, 3 diameter,
    /// 4 radius, 5 angular 3-point, 6 ordinate.
    pub dim_type: u8,
    /// The measurement the file states (group 42).
    pub measurement: Option<f64>,
    /// The text override, exactly as stated (group 1); `None` when not stated. Interpreting `""`, `"<>"` and `" "`
    /// is analysis.
    pub text: Option<BStr>,
    /// First definition point (group 13).
    pub first_point: Point3,
    /// Second definition point (group 14).
    pub second_point: Point3,
    /// The dimension's own block (group 2), if stated.
    pub block: Option<NameId>,
}

/// A layer from the LAYER table.
#[derive(Debug)]
pub struct Layer {
    /// Its name.
    pub name: NameId,
    /// Frozen (flag 1 of group 70).
    pub frozen: bool,
    /// Off (a negative colour, group 62).
    pub off: bool,
}

/// A block definition.
#[derive(Debug)]
pub struct Block {
    /// Its name.
    pub name: NameId,
    /// Base point.
    pub base: Point3,
    /// Its entities, in order.
    pub entities: BVec<Entity>,
}

/// Something the reader did not fail on but could not fully read.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Note {
    /// Strings whose bytes could not be decoded (an unsupported or unstated code page, or invalid UTF-8): undecodable
    /// bytes became U+FFFD. Never guessed.
    TextNotDecoded {
        /// The code page the header states, if any.
        codepage: Option<NameId>,
        /// How many strings were affected.
        count: u64,
    },
    /// A hatch boundary path with an edge kind 0.1 does not read.
    HatchEdgeNotSupported {
        /// The hatch's handle.
        handle: Option<u64>,
        /// The edge kind.
        edge: EdgeKind,
    },
    /// A hatch whose boundary data did not have the expected structure; it is kept without boundary paths.
    HatchNotRead {
        /// The hatch's handle.
        handle: Option<u64>,
    },
}

/// Where an uninterpreted entity was: model space, paper space, or a block definition (whose contents count only
/// where the block is placed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Where {
    /// The ENTITIES section, model space.
    ModelSpace,
    /// The ENTITIES section, paper space (group 67 = 1).
    PaperSpace,
    /// A block definition.
    Block(NameId),
}

/// What the header states. `None` means the file does not state it, never a default (ADR 0015, R3).
#[derive(Debug, Default)]
pub(crate) struct Header {
    pub(crate) acadver: Option<NameId>,
    pub(crate) insunits: Option<i16>,
    pub(crate) measurement: Option<i16>,
    pub(crate) codepage: Option<NameId>,
}

/// A DXF drawing as read.
#[derive(Debug)]
pub struct Drawing {
    pub(crate) header: Header,
    pub(crate) layers: BVec<Layer>,
    pub(crate) blocks: BVec<Block>,
    pub(crate) entities: BVec<Entity>,
    /// Counts of uninterpreted entities by (type, layer, where), in first-seen order.
    pub(crate) unsupported: BVec<((NameId, NameId, Where), u64)>,
    pub(crate) notes: BVec<Note>,
    /// Every name, once, in first-seen order; a [`NameId`] is a position here.
    pub(crate) names: BVec<BStr>,
}

impl Drawing {
    /// A name by its id.
    pub fn name(&self, id: NameId) -> &str {
        self.names.get(id.0 as usize).map_or("", |s| s)
    }

    /// `$ACADVER`, if stated (e.g. `AC1032`).
    pub fn acadver(&self) -> Option<&str> {
        self.header.acadver.map(|n| self.name(n))
    }

    /// `$INSUNITS`, if stated (e.g. 6 = metres). `None` means the file does not state it.
    pub fn insunits(&self) -> Option<i16> {
        self.header.insunits
    }

    /// `$MEASUREMENT`, if stated (0 imperial, 1 metric). `None` means the file does not state it.
    pub fn measurement(&self) -> Option<i16> {
        self.header.measurement
    }

    /// `$DWGCODEPAGE`, if stated (e.g. `ANSI_1252`).
    pub fn codepage(&self) -> Option<&str> {
        self.header.codepage.map(|n| self.name(n))
    }

    /// The LAYER table, in file order.
    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }

    /// Block definitions, in file order.
    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    /// The ENTITIES section's entities that 0.1 reads, in file order.
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    /// Entities 0.1 does not interpret, wherever they are (ENTITIES and blocks), counted by type and layer and sorted.
    /// Nothing is silently dropped.
    pub fn unsupported(&self) -> Vec<(&str, &str, u64)> {
        let mut out: Vec<(&str, &str, u64)> = self
            .unsupported
            .iter()
            .map(|((kind, layer, _), n)| (self.name(*kind), self.name(*layer), *n))
            .collect();
        out.sort_unstable();
        // The same type and layer in several places is one entry.
        let mut merged: Vec<(&str, &str, u64)> = Vec::with_capacity(out.len());
        for (k, l, n) in out {
            match merged.last_mut() {
                Some(last) if last.0 == k && last.1 == l => last.2 += n,
                _ => merged.push((k, l, n)),
            }
        }
        merged
    }

    /// Entities 0.1 does not interpret, by type, layer and where they are, in first-seen order (D-28: content of a
    /// block counts where the block is placed; paper space is listed, not measured).
    pub fn unsupported_where(&self) -> &[((NameId, NameId, Where), u64)] {
        &self.unsupported
    }

    /// What the reader could not fully read but did not fail on.
    pub fn notes(&self) -> &[Note] {
        &self.notes
    }
}
