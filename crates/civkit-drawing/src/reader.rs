// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 15:40 IST · last edited 2026-11-27 15:40 IST

//! CivKit's DXF reader (D-27, ADR 0015): ASCII DXF R12–R2018 into a [`Drawing`], under a [`Budget`].
//!
//! The reader is iterative (no recursion), reads the input in place, and stores nothing without reserving it from the
//! budget first. A structural problem is an error naming its line; content 0.1 does not interpret is counted or noted,
//! never silently dropped.

use crate::ReadError;
use crate::budget::{BMap, BStr, BVec, Budget, Scratch};
use crate::decode::{Encoding, decode, decoded_len};
use crate::model::{
    Block, Boundary, Dimension, Drawing, Edge, EdgeKind, Entity, EntityKind, Hatch, HatchPath,
    Header, Insert, Layer, NameId, Names, Note, Point2, Point3, Polyline, PolylineSource, Text,
    Vertex,
};
use crate::pairs::{Pair, Pairs};

/// Limits on what one read may contain (ADR 0015, Q-D). The heap itself is bounded by the [`Budget`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Limits {
    /// The largest input accepted, in bytes; checked before reading.
    pub max_input_bytes: usize,
    /// The most vertices (or edges) in one polyline or hatch path.
    pub max_vertices: usize,
    /// The longest string kept (text, names), in bytes as written in the file.
    pub max_string_bytes: usize,
    /// The longest entity type name counted, in bytes.
    pub max_type_name_bytes: usize,
}

impl Limits {
    /// The default heap budget for reading a drawing and analysing it: 512 MiB.
    pub const DEFAULT_BUDGET_BYTES: usize = 512 << 20;
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_input_bytes: 256 << 20,
            max_vertices: 1_000_000,
            max_string_bytes: 64 << 10,
            max_type_name_bytes: 256,
        }
    }
}

/// Reads an ASCII DXF file with the default [`Limits`]. The drawing's storage is reserved from `budget`, which later
/// analysis of the same drawing shares.
pub fn read_dxf(input: &[u8], budget: &Budget) -> Result<Drawing, ReadError> {
    read_dxf_with(input, budget, &Limits::default())
}

/// Reads an ASCII DXF file with the given limits.
pub fn read_dxf_with(input: &[u8], budget: &Budget, limits: &Limits) -> Result<Drawing, ReadError> {
    if input.len() > limits.max_input_bytes {
        return Err(ReadError::InputTooLarge {
            size: input.len(),
            limit: limits.max_input_bytes,
        });
    }
    if input.starts_with(b"AutoCAD Binary DXF") {
        return Err(ReadError::BinaryNotSupported);
    }
    let input = input.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(input);
    let mut reader = Reader::new(input, budget, limits)?;
    reader.read()?;
    reader.finish()
}

/// Fields every entity has.
struct Common {
    layer: NameId,
    handle: Option<u64>,
    paper_space: bool,
    extrusion: Point3,
}

impl Common {
    fn entity(self, kind: EntityKind) -> Entity {
        Entity {
            layer: self.layer,
            handle: self.handle,
            paper_space: self.paper_space,
            extrusion: self.extrusion,
            kind,
        }
    }
}

struct Reader<'a, 'l> {
    pairs: Pairs<'a>,
    budget: Budget,
    limits: &'l Limits,
    drawing: Drawing,
    encoding: Encoding,
    scratch: Scratch,
    undecodable: u64,
    layer0: NameId,
}

/// Header variables 0.1 reads.
#[derive(Clone, Copy)]
enum Var {
    AcadVer,
    InsUnits,
    Measurement,
    CodePage,
    Other,
}

impl<'a, 'l> Reader<'a, 'l> {
    fn new(input: &'a [u8], budget: &Budget, limits: &'l Limits) -> Result<Self, ReadError> {
        let drawing = Drawing {
            header: Header::default(),
            layers: BVec::new(budget),
            blocks: BVec::new(budget),
            entities: BVec::new(budget),
            unsupported: BMap::new(budget),
            notes: BVec::new(budget),
            names: Names {
                list: BVec::new(budget),
                index: BMap::new(budget),
            },
        };
        let mut reader = Self {
            pairs: Pairs::new(input),
            budget: budget.clone(),
            limits,
            drawing,
            encoding: Encoding::AsciiOnly,
            scratch: Scratch::new(budget),
            undecodable: 0,
            layer0: NameId(0),
        };
        reader.layer0 = reader.intern_str("0")?;
        Ok(reader)
    }

    // ---------------------------------------------------------------- names and strings

    /// Interns the scratch buffer's text.
    fn intern_scratch(&mut self) -> Result<NameId, ReadError> {
        let Self {
            scratch,
            drawing,
            budget,
            ..
        } = self;
        let text = scratch.as_str();
        if let Some(&id) = drawing.names.index.get(text) {
            return Ok(NameId(id));
        }
        let id =
            u32::try_from(drawing.names.list.len()).map_err(|_| ReadError::BudgetExhausted {
                requested: 0,
                limit: budget.limit(),
            })?;
        let stored = BStr::build(budget, text.len(), |s| s.push_str(text))?;
        let key = BStr::build(budget, text.len(), |s| s.push_str(text))?;
        drawing.names.list.push(stored)?;
        drawing.names.index.insert_new(key, id)?;
        Ok(NameId(id))
    }

    fn intern_str(&mut self, text: &str) -> Result<NameId, ReadError> {
        self.scratch.prepare(text.len())?.push_str(text);
        self.intern_scratch()
    }

    /// Interns a value's bytes, decoded, refusing values longer than `limit`.
    fn intern_bytes(&mut self, bytes: &[u8], line: u64, limit: usize) -> Result<NameId, ReadError> {
        if bytes.len() > limit {
            return Err(ReadError::StringTooLong { line, limit });
        }
        let (len, replaced) = decoded_len(bytes, self.encoding);
        let encoding = self.encoding;
        let buffer = self.scratch.prepare(len)?;
        decode(bytes, encoding, |c| buffer.push(c));
        if replaced {
            self.undecodable += 1;
        }
        self.intern_scratch()
    }

    /// A name from a value (layer, block, pattern), with the string limit.
    fn name(&mut self, p: &Pair) -> Result<NameId, ReadError> {
        self.intern_bytes(p.value, p.line, self.limits.max_string_bytes)
    }

    /// A string value kept as text (TEXT, dimension override).
    fn text(&mut self, p: &Pair) -> Result<BStr, ReadError> {
        let limit = self.limits.max_string_bytes;
        if p.value.len() > limit {
            return Err(ReadError::StringTooLong {
                line: p.line,
                limit,
            });
        }
        let (len, replaced) = decoded_len(p.value, self.encoding);
        if replaced {
            self.undecodable += 1;
        }
        let encoding = self.encoding;
        BStr::build(&self.budget, len, |s| {
            decode(p.value, encoding, |c| s.push(c));
        })
    }

    // ---------------------------------------------------------------- sections

    fn read(&mut self) -> Result<(), ReadError> {
        loop {
            let p = self.pairs.expect()?;
            if p.code == 999 {
                continue;
            }
            if p.code != 0 {
                return Err(ReadError::Malformed {
                    line: p.line,
                    expected: "0 SECTION or 0 EOF",
                });
            }
            if p.is("EOF") {
                return Ok(());
            }
            if !p.is("SECTION") {
                return Err(ReadError::Malformed {
                    line: p.line,
                    expected: "0 SECTION or 0 EOF",
                });
            }
            let name = self.pairs.expect()?;
            if name.code != 2 {
                return Err(ReadError::Malformed {
                    line: name.line,
                    expected: "2 and a section name",
                });
            }
            match name.trimmed() {
                b"HEADER" => {
                    self.header()?;
                    self.encoding = self.encoding_from_header();
                }
                b"TABLES" => self.tables()?,
                b"BLOCKS" => self.blocks()?,
                b"ENTITIES" => {
                    let mut list =
                        std::mem::replace(&mut self.drawing.entities, BVec::new(&self.budget));
                    let r = self.entities(b"ENDSEC", &mut list);
                    self.drawing.entities = list;
                    r?;
                }
                _ => self.skip_section()?,
            }
        }
    }

    fn skip_section(&mut self) -> Result<(), ReadError> {
        loop {
            let p = self.pairs.expect()?;
            if p.code == 0 && p.is("ENDSEC") {
                return Ok(());
            }
        }
    }

    fn header(&mut self) -> Result<(), ReadError> {
        let mut var = Var::Other;
        loop {
            let p = self.pairs.expect()?;
            match p.code {
                0 if p.is("ENDSEC") => return Ok(()),
                0 => {
                    return Err(ReadError::Malformed {
                        line: p.line,
                        expected: "0 ENDSEC",
                    });
                }
                9 => {
                    var = match p.trimmed() {
                        b"$ACADVER" => Var::AcadVer,
                        b"$INSUNITS" => Var::InsUnits,
                        b"$MEASUREMENT" => Var::Measurement,
                        b"$DWGCODEPAGE" => Var::CodePage,
                        _ => Var::Other,
                    }
                }
                code => match (var, code) {
                    (Var::AcadVer, 1) => {
                        let v = self.intern_bytes(
                            p.trimmed(),
                            p.line,
                            self.limits.max_type_name_bytes,
                        )?;
                        self.drawing.header.acadver = Some(v);
                    }
                    (Var::InsUnits, 70) => self.drawing.header.insunits = Some(p.int()?),
                    (Var::Measurement, 70) => self.drawing.header.measurement = Some(p.int()?),
                    (Var::CodePage, 3) => {
                        let v = self.intern_bytes(
                            p.trimmed(),
                            p.line,
                            self.limits.max_type_name_bytes,
                        )?;
                        self.drawing.header.codepage = Some(v);
                    }
                    _ => {}
                },
            }
        }
    }

    fn encoding_from_header(&self) -> Encoding {
        let version = self
            .drawing
            .acadver()
            .and_then(|v| v.strip_prefix("AC"))
            .and_then(|n| n.parse::<u32>().ok());
        if version.is_some_and(|v| v >= 1021) {
            return Encoding::Utf8;
        }
        match self.drawing.codepage() {
            Some(cp) if cp.eq_ignore_ascii_case("ANSI_1252") => Encoding::Ansi1252,
            _ => Encoding::AsciiOnly,
        }
    }

    fn tables(&mut self) -> Result<(), ReadError> {
        loop {
            let p = self.pairs.expect()?;
            if p.code != 0 {
                return Err(ReadError::Malformed {
                    line: p.line,
                    expected: "0 TABLE or 0 ENDSEC",
                });
            }
            if p.is("ENDSEC") {
                return Ok(());
            }
            if !p.is("TABLE") {
                return Err(ReadError::Malformed {
                    line: p.line,
                    expected: "0 TABLE or 0 ENDSEC",
                });
            }
            let name = self.pairs.expect()?;
            if name.code != 2 {
                return Err(ReadError::Malformed {
                    line: name.line,
                    expected: "2 and a table name",
                });
            }
            let layers = name.is("LAYER");
            self.pairs.skip_fields()?;
            loop {
                let p = self.pairs.expect()?;
                if p.code != 0 {
                    return Err(ReadError::Malformed {
                        line: p.line,
                        expected: "0 and a table record",
                    });
                }
                if p.is("ENDTAB") {
                    self.pairs.skip_fields()?;
                    break;
                }
                if p.is("ENDSEC") {
                    return Err(ReadError::Malformed {
                        line: p.line,
                        expected: "0 ENDTAB",
                    });
                }
                if layers && p.is("LAYER") {
                    self.layer_record()?;
                } else {
                    self.pairs.skip_fields()?;
                }
            }
        }
    }

    fn layer_record(&mut self) -> Result<(), ReadError> {
        let (mut name, mut flags, mut color) = (None, 0i32, 0i32);
        while let Some(p) = self.pairs.field()? {
            match p.code {
                2 => name = Some(self.name(&p)?),
                70 => flags = p.int()?,
                62 => color = p.int()?,
                _ => {}
            }
        }
        if let Some(name) = name {
            self.drawing.layers.push(Layer {
                name,
                frozen: flags & 1 != 0,
                off: color < 0,
            })?;
        }
        Ok(())
    }

    fn blocks(&mut self) -> Result<(), ReadError> {
        loop {
            let p = self.pairs.expect()?;
            if p.code == 999 {
                continue;
            }
            if p.code != 0 {
                return Err(ReadError::Malformed {
                    line: p.line,
                    expected: "0 BLOCK or 0 ENDSEC",
                });
            }
            if p.is("ENDSEC") {
                return Ok(());
            }
            if !p.is("BLOCK") {
                return Err(ReadError::Malformed {
                    line: p.line,
                    expected: "0 BLOCK or 0 ENDSEC",
                });
            }
            let (mut name, mut base) = (None, Point3::default());
            while let Some(f) = self.pairs.field()? {
                match f.code {
                    2 if name.is_none() => name = Some(self.name(&f)?),
                    10 => base.x = f.f64()?,
                    20 => base.y = f.f64()?,
                    30 => base.z = f.f64()?,
                    _ => {}
                }
            }
            let name = match name {
                Some(n) => n,
                None => self.intern_str("")?,
            };
            let mut entities = BVec::new(&self.budget);
            self.entities(b"ENDBLK", &mut entities)?;
            self.drawing.blocks.push(Block {
                name,
                base,
                entities,
            })?;
        }
    }

    /// Reads entities until `0 <stop>` (`ENDSEC` or `ENDBLK`), which it consumes.
    fn entities(&mut self, stop: &[u8], list: &mut BVec<Entity>) -> Result<(), ReadError> {
        let expected = if stop == b"ENDBLK" {
            "0 ENDBLK"
        } else {
            "0 ENDSEC"
        };
        loop {
            let p = self.pairs.expect()?;
            if p.code == 999 {
                continue;
            }
            if p.code != 0 {
                return Err(ReadError::Malformed {
                    line: p.line,
                    expected: "0 and an entity type",
                });
            }
            let kw = p.trimmed();
            if kw == stop {
                self.pairs.skip_fields()?;
                return Ok(());
            }
            if matches!(kw, b"ENDSEC" | b"ENDBLK" | b"EOF" | b"BLOCK" | b"SECTION") {
                return Err(ReadError::Malformed {
                    line: p.line,
                    expected,
                });
            }
            self.entity(kw, p.line, list)?;
        }
    }

    // ---------------------------------------------------------------- entities

    fn common(&mut self, c: &mut Common, p: &Pair) -> Result<bool, ReadError> {
        match p.code {
            8 => c.layer = self.name(p)?,
            5 => c.handle = p.handle(),
            67 => c.paper_space = p.int::<i32>()? == 1,
            210 => c.extrusion.x = p.f64()?,
            220 => c.extrusion.y = p.f64()?,
            230 => c.extrusion.z = p.f64()?,
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn new_common(&self) -> Common {
        Common {
            layer: self.layer0,
            handle: None,
            paper_space: false,
            extrusion: Point3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        }
    }

    fn count_unsupported(&mut self, kind: NameId, layer: NameId) -> Result<(), ReadError> {
        if let Some(n) = self.drawing.unsupported.get_mut(&(kind, layer)) {
            *n += 1;
            Ok(())
        } else {
            self.drawing.unsupported.insert_new((kind, layer), 1)
        }
    }

    fn vertex_cap(&self, count: usize, line: u64) -> Result<(), ReadError> {
        if count >= self.limits.max_vertices {
            return Err(ReadError::TooManyVertices {
                line,
                limit: self.limits.max_vertices,
            });
        }
        Ok(())
    }

    fn entity(&mut self, kw: &[u8], line: u64, list: &mut BVec<Entity>) -> Result<(), ReadError> {
        let mut c = self.new_common();
        let kind = match kw {
            b"LINE" => {
                let (mut s, mut e) = (Point3::default(), Point3::default());
                while let Some(p) = self.pairs.field()? {
                    if self.common(&mut c, &p)? {
                        continue;
                    }
                    match p.code {
                        10 => s.x = p.f64()?,
                        20 => s.y = p.f64()?,
                        30 => s.z = p.f64()?,
                        11 => e.x = p.f64()?,
                        21 => e.y = p.f64()?,
                        31 => e.z = p.f64()?,
                        _ => {}
                    }
                }
                EntityKind::Line { start: s, end: e }
            }
            b"CIRCLE" | b"ARC" => {
                let (mut center, mut radius, mut start_deg, mut end_deg) =
                    (Point3::default(), 0.0, 0.0, 0.0);
                while let Some(p) = self.pairs.field()? {
                    if self.common(&mut c, &p)? {
                        continue;
                    }
                    match p.code {
                        10 => center.x = p.f64()?,
                        20 => center.y = p.f64()?,
                        30 => center.z = p.f64()?,
                        40 => radius = p.f64()?,
                        50 => start_deg = p.f64()?,
                        51 => end_deg = p.f64()?,
                        _ => {}
                    }
                }
                if kw == b"CIRCLE" {
                    EntityKind::Circle { center, radius }
                } else {
                    EntityKind::Arc {
                        center,
                        radius,
                        start_deg,
                        end_deg,
                    }
                }
            }
            b"LWPOLYLINE" => {
                let (mut vertices, mut flags, mut elevation) = (BVec::new(&self.budget), 0i32, 0.0);
                while let Some(p) = self.pairs.field()? {
                    if self.common(&mut c, &p)? {
                        continue;
                    }
                    match p.code {
                        10 => {
                            self.vertex_cap(vertices.len(), p.line)?;
                            vertices.push(Vertex {
                                x: p.f64()?,
                                y: 0.0,
                                bulge: 0.0,
                            })?;
                        }
                        20 => {
                            let y = p.f64()?;
                            if let Some(v) = vertices.last_mut() {
                                v.y = y;
                            }
                        }
                        42 => {
                            let b = p.f64()?;
                            if let Some(v) = vertices.last_mut() {
                                v.bulge = b;
                            }
                        }
                        70 => flags = p.int()?,
                        38 => elevation = p.f64()?,
                        _ => {}
                    }
                }
                EntityKind::Polyline(Polyline {
                    vertices,
                    closed: flags & 1 != 0,
                    elevation,
                    source: PolylineSource::Lightweight,
                })
            }
            b"POLYLINE" => match self.old_polyline(&mut c)? {
                Some(kind) => kind,
                None => return Ok(()),
            },
            b"TEXT" => {
                let (mut point, mut height, mut rotation_deg, mut value) =
                    (Point3::default(), 0.0, 0.0, None);
                while let Some(p) = self.pairs.field()? {
                    if self.common(&mut c, &p)? {
                        continue;
                    }
                    match p.code {
                        10 => point.x = p.f64()?,
                        20 => point.y = p.f64()?,
                        30 => point.z = p.f64()?,
                        40 => height = p.f64()?,
                        50 => rotation_deg = p.f64()?,
                        1 => value = Some(self.text(&p)?),
                        _ => {}
                    }
                }
                let value = match value {
                    Some(v) => v,
                    None => BStr::build(&self.budget, 0, |_| {})?,
                };
                EntityKind::Text(Text {
                    point,
                    height,
                    rotation_deg,
                    value,
                })
            }
            b"INSERT" => return self.insert(c, list),
            b"HATCH" => self.hatch(&mut c)?,
            b"DIMENSION" => {
                let mut d = Dimension {
                    dim_type: 0,
                    measurement: None,
                    text: None,
                    first_point: Point3::default(),
                    second_point: Point3::default(),
                    block: None,
                };
                while let Some(p) = self.pairs.field()? {
                    if self.common(&mut c, &p)? {
                        continue;
                    }
                    match p.code {
                        2 => d.block = Some(self.name(&p)?),
                        70 => d.dim_type = (p.int::<i32>()? & 7) as u8,
                        42 => d.measurement = Some(p.f64()?),
                        1 => d.text = Some(self.text(&p)?),
                        13 => d.first_point.x = p.f64()?,
                        23 => d.first_point.y = p.f64()?,
                        33 => d.first_point.z = p.f64()?,
                        14 => d.second_point.x = p.f64()?,
                        24 => d.second_point.y = p.f64()?,
                        34 => d.second_point.z = p.f64()?,
                        _ => {}
                    }
                }
                EntityKind::Dimension(d)
            }
            other => {
                let kind = self.intern_bytes(other, line, self.limits.max_type_name_bytes)?;
                while let Some(p) = self.pairs.field()? {
                    self.common(&mut c, &p)?;
                }
                return self.count_unsupported(kind, c.layer);
            }
        };
        list.push(c.entity(kind))
    }

    /// An old POLYLINE with its VERTEX entities and SEQEND. 3D polylines, meshes, polyface meshes and curve- or
    /// spline-fitted polylines are counted as unsupported (their vertices consumed, not kept).
    fn old_polyline(&mut self, c: &mut Common) -> Result<Option<EntityKind>, ReadError> {
        let (mut flags, mut elevation) = (0i32, 0.0);
        while let Some(p) = self.pairs.field()? {
            if self.common(c, &p)? {
                continue;
            }
            match p.code {
                70 => flags = p.int()?,
                30 => elevation = p.f64()?,
                _ => {}
            }
        }
        let unsupported = if flags & 8 != 0 {
            Some("POLYLINE (3D)")
        } else if flags & 16 != 0 {
            Some("POLYLINE (mesh)")
        } else if flags & 64 != 0 {
            Some("POLYLINE (polyface)")
        } else if flags & 6 != 0 {
            Some("POLYLINE (fitted)")
        } else {
            None
        };
        let mut vertices = BVec::new(&self.budget);
        loop {
            match self.pairs.peek()? {
                Some(p) if p.code == 0 && p.is("VERTEX") => {
                    self.pairs.next()?;
                    let (mut v, mut vflags) = (Vertex::default(), 0i32);
                    while let Some(f) = self.pairs.field()? {
                        match f.code {
                            10 => v.x = f.f64()?,
                            20 => v.y = f.f64()?,
                            42 => v.bulge = f.f64()?,
                            70 => vflags = f.int()?,
                            _ => {}
                        }
                    }
                    // Flag 16 marks a spline frame control point, not a vertex of the shape.
                    if unsupported.is_none() && vflags & 16 == 0 {
                        self.vertex_cap(vertices.len(), p.line)?;
                        vertices.push(v)?;
                    }
                }
                Some(p) if p.code == 0 && p.is("SEQEND") => {
                    self.pairs.next()?;
                    self.pairs.skip_fields()?;
                    break;
                }
                _ => break,
            }
        }
        if let Some(name) = unsupported {
            let kind = self.intern_str(name)?;
            self.count_unsupported(kind, c.layer)?;
            return Ok(None);
        }
        Ok(Some(EntityKind::Polyline(Polyline {
            vertices,
            closed: flags & 1 != 0,
            elevation,
            source: PolylineSource::Old,
        })))
    }

    /// INSERT (and MINSERT arrays), then its ATTRIB entities and SEQEND when group 66 says they follow. Attributes
    /// are not interpreted in 0.1: they are counted.
    fn insert(&mut self, mut c: Common, list: &mut BVec<Entity>) -> Result<(), ReadError> {
        let mut i = Insert {
            block: self.layer0,
            point: Point3::default(),
            scale: Point3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            rotation_deg: 0.0,
            columns: 1,
            rows: 1,
            column_spacing: 0.0,
            row_spacing: 0.0,
        };
        let (mut block, mut attributes) = (None, false);
        while let Some(p) = self.pairs.field()? {
            if self.common(&mut c, &p)? {
                continue;
            }
            match p.code {
                2 => block = Some(self.name(&p)?),
                10 => i.point.x = p.f64()?,
                20 => i.point.y = p.f64()?,
                30 => i.point.z = p.f64()?,
                41 => i.scale.x = p.f64()?,
                42 => i.scale.y = p.f64()?,
                43 => i.scale.z = p.f64()?,
                50 => i.rotation_deg = p.f64()?,
                70 => i.columns = p.int::<i32>()?.max(1).unsigned_abs(),
                71 => i.rows = p.int::<i32>()?.max(1).unsigned_abs(),
                44 => i.column_spacing = p.f64()?,
                45 => i.row_spacing = p.f64()?,
                66 => attributes = p.int::<i32>()? == 1,
                _ => {}
            }
        }
        i.block = match block {
            Some(b) => b,
            None => self.intern_str("")?,
        };
        list.push(c.entity(EntityKind::Insert(i)))?;
        if attributes {
            let attrib = self.intern_str("ATTRIB")?;
            loop {
                match self.pairs.peek()? {
                    Some(p) if p.code == 0 && p.is("ATTRIB") => {
                        self.pairs.next()?;
                        let mut a = self.new_common();
                        while let Some(f) = self.pairs.field()? {
                            self.common(&mut a, &f)?;
                        }
                        self.count_unsupported(attrib, a.layer)?;
                    }
                    Some(p) if p.code == 0 && p.is("SEQEND") => {
                        self.pairs.next()?;
                        self.pairs.skip_fields()?;
                        break;
                    }
                    _ => break,
                }
            }
        }
        Ok(())
    }

    // ---------------------------------------------------------------- HATCH

    fn hatch(&mut self, c: &mut Common) -> Result<EntityKind, ReadError> {
        let (mut pattern, mut solid, mut elevation) = (None, false, 0.0);
        let mut paths = BVec::new(&self.budget);
        let mut stated_paths = None;
        while let Some(p) = self.pairs.field()? {
            if self.common(c, &p)? {
                continue;
            }
            match p.code {
                2 => pattern = Some(self.name(&p)?),
                70 => solid = p.int::<i32>()? == 1,
                30 => elevation = p.f64()?,
                91 => {
                    stated_paths = Some(p.int::<i64>()?);
                    break;
                }
                _ => {}
            }
        }
        if let Some(n) = stated_paths
            && !self.hatch_paths(n, &mut paths, c.handle)?
        {
            self.drawing
                .notes
                .push(Note::HatchNotRead { handle: c.handle })?;
            paths = BVec::new(&self.budget);
        }
        self.pairs.skip_fields()?; // pattern definition, seed points, gradient: not needed for the boundary
        let pattern = match pattern {
            Some(p) => p,
            None => self.intern_str("")?,
        };
        Ok(EntityKind::Hatch(Hatch {
            pattern,
            solid,
            elevation,
            paths,
        }))
    }

    /// The next field if it has `code`; `None` (and the field consumed) otherwise.
    fn want(&mut self, code: i32) -> Result<Option<Pair<'a>>, ReadError> {
        Ok(self.pairs.field()?.filter(|p| p.code == code))
    }

    /// Reads `n` boundary paths. `Ok(false)` when the structure is not as DXF describes it (the caller notes it and
    /// skips the rest of the hatch).
    fn hatch_paths(
        &mut self,
        n: i64,
        paths: &mut BVec<HatchPath>,
        handle: Option<u64>,
    ) -> Result<bool, ReadError> {
        macro_rules! want {
            ($code:expr) => {
                match self.want($code)? {
                    Some(p) => p,
                    None => return Ok(false),
                }
            };
        }
        for _ in 0..n.max(0) {
            let flags: i32 = want!(92).int()?;
            let boundary = if flags & 2 != 0 {
                let has_bulge = want!(72).int::<i32>()? != 0;
                let closed = want!(73).int::<i32>()? != 0;
                let count = want!(93);
                let n_vertices = self.stated_count(&count)?;
                let mut vertices = BVec::new(&self.budget);
                for _ in 0..n_vertices {
                    let x = want!(10).f64()?;
                    let y = want!(20).f64()?;
                    let mut bulge = 0.0;
                    if has_bulge && self.pairs.peek()?.is_some_and(|p| p.code == 42) {
                        bulge = want!(42).f64()?;
                    }
                    vertices.push(Vertex { x, y, bulge })?;
                }
                Boundary::Polyline { vertices, closed }
            } else {
                let count = want!(93);
                let n_edges = self.stated_count(&count)?;
                let mut edges = BVec::new(&self.budget);
                let mut not_supported = None;
                for _ in 0..n_edges {
                    match want!(72).int::<i32>()? {
                        1 => {
                            let (x0, y0, x1, y1) = (
                                want!(10).f64()?,
                                want!(20).f64()?,
                                want!(11).f64()?,
                                want!(21).f64()?,
                            );
                            edges.push(Edge::Line {
                                start: Point2 { x: x0, y: y0 },
                                end: Point2 { x: x1, y: y1 },
                            })?;
                        }
                        2 => {
                            let (x, y, r) = (want!(10).f64()?, want!(20).f64()?, want!(40).f64()?);
                            let (s, e) = (want!(50).f64()?, want!(51).f64()?);
                            let ccw = want!(73).int::<i32>()? != 0;
                            edges.push(Edge::Arc {
                                center: Point2 { x, y },
                                radius: r,
                                start_deg: s,
                                end_deg: e,
                                counter_clockwise: ccw,
                            })?;
                        }
                        3 => {
                            for code in [10, 20, 11, 21, 40, 50, 51, 73] {
                                want!(code);
                            }
                            not_supported.get_or_insert(EdgeKind::Ellipse);
                        }
                        4 => {
                            if !self.skip_spline_edge()? {
                                return Ok(false);
                            }
                            not_supported.get_or_insert(EdgeKind::Spline);
                        }
                        _ => return Ok(false),
                    }
                }
                match not_supported {
                    Some(edge) => {
                        self.drawing
                            .notes
                            .push(Note::HatchEdgeNotSupported { handle, edge })?;
                        Boundary::NotSupported { edge }
                    }
                    None => Boundary::Edges(edges),
                }
            };
            // Source boundary objects: a count, then that many handles.
            if self.pairs.peek()?.is_some_and(|p| p.code == 97) {
                want!(97);
                while self.pairs.peek()?.is_some_and(|p| p.code == 330) {
                    self.pairs.next()?;
                }
            }
            paths.push(HatchPath { flags, boundary })?;
        }
        Ok(true)
    }

    /// A stated vertex or edge count (group 93), refused above the limit before anything is allocated for it.
    fn stated_count(&self, p: &Pair) -> Result<usize, ReadError> {
        let n: i64 = p.int()?;
        let n = usize::try_from(n).map_err(|_| ReadError::InvalidNumber {
            line: p.line,
            code: p.code,
        })?;
        if n > self.limits.max_vertices {
            return Err(ReadError::TooManyVertices {
                line: p.line,
                limit: self.limits.max_vertices,
            });
        }
        Ok(n)
    }

    /// Consumes a spline edge: degree, flags, knots, control points (with weights if rational), fit points and
    /// optional tangents. `Ok(false)` when its structure does not match.
    fn skip_spline_edge(&mut self) -> Result<bool, ReadError> {
        macro_rules! want {
            ($code:expr) => {
                match self.want($code)? {
                    Some(p) => p,
                    None => return Ok(false),
                }
            };
        }
        want!(94);
        let rational = want!(73).int::<i32>()? != 0;
        want!(74);
        let knots: i64 = want!(95).int()?;
        let controls: i64 = want!(96).int()?;
        for _ in 0..knots.max(0) {
            want!(40);
        }
        for _ in 0..controls.max(0) {
            want!(10);
            want!(20);
            if rational {
                want!(42);
            }
        }
        if self.pairs.peek()?.is_some_and(|p| p.code == 97) {
            let fits: i64 = want!(97).int()?;
            for _ in 0..fits.max(0) {
                want!(11);
                want!(21);
            }
            if self.pairs.peek()?.is_some_and(|p| p.code == 12) {
                for code in [12, 22, 13, 23] {
                    want!(code);
                }
            }
        }
        Ok(true)
    }

    fn finish(mut self) -> Result<Drawing, ReadError> {
        if self.undecodable > 0 {
            let note = Note::TextNotDecoded {
                codepage: self.drawing.header.codepage,
                count: self.undecodable,
            };
            self.drawing.notes.push(note)?;
        }
        Ok(self.drawing)
    }
}
