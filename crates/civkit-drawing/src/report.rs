// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 22:04 IST · last edited 2026-11-27 22:05 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! The drawing report (JSON) and the quantity schedule (CSV) (D-28 / ADR 0016 §9, API_DESIGN §13).
//!
//! Strings that come from the drawing (layer, block and type names, text values, override text) appear
//! only in fields documented as drawing text, escaped, never in messages. Numbers are written in full
//! precision (shortest round-trip text); nothing is rounded. Both outputs are built in a buffer
//! reserved from the analysis's budget.

use crate::budget::{BVec, Budget};
use crate::expand::AnalysisError;
use crate::measure::{AnalysisOptions, Detail, Issue, LayerTotals, Level, Quantities, measure};
use crate::model::{Drawing, Entity, EntityKind, Where};
use crate::reader::{Limits, read_dxf};

/// The report's identity (API_DESIGN §4, §13).
pub const REPORT_ID: &str = "drawing.analyze";
/// The report's version (SemVer, API_DESIGN §4 and §11).
pub const REPORT_VERSION: &str = "1.0.0";
const CIVKIT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// A drawing and its quantities, ready to be written as a report or a schedule.
#[derive(Debug)]
pub struct Analysis {
    drawing: Drawing,
    quantities: Quantities,
    budget: Budget,
}

/// Analyses a drawing already read with `budget` (the shared budget, ADR 0015 Q-D).
///
/// # Errors
/// As [`measure`].
pub fn analyze(
    drawing: Drawing,
    budget: &Budget,
    options: &AnalysisOptions,
) -> Result<Analysis, AnalysisError> {
    let quantities = measure(&drawing, budget, options)?;
    Ok(Analysis {
        drawing,
        quantities,
        budget: budget.clone(),
    })
}

/// Reads and analyses a DXF file, with the default limits and heap budget.
///
/// # Errors
/// [`AnalysisError::Read`] when the file cannot be read (its [`AnalysisError::code`] tells a limit
/// from an unreadable file), otherwise as [`measure`].
pub fn analyze_dxf(input: &[u8], options: &AnalysisOptions) -> Result<Analysis, AnalysisError> {
    let budget = Budget::new(Limits::DEFAULT_BUDGET_BYTES);
    let drawing = read_dxf(input, &budget).map_err(AnalysisError::Read)?;
    analyze(drawing, &budget, options)
}

/// Output text, reserved from the budget as it grows.
struct Out(BVec<u8>);

impl Out {
    fn s(&mut self, text: &str) -> Result<(), AnalysisError> {
        Ok(self.0.extend_from_slice(text.as_bytes())?)
    }

    fn finish(self) -> String {
        // Only `&str` pieces were written, so the bytes are UTF-8.
        String::from_utf8(self.0.into_vec()).unwrap_or_default()
    }
}

/// A number in full precision; JSON has no NaN or infinity, so those (never produced) are `null`.
fn number(x: f64) -> String {
    if x.is_finite() {
        format!("{x}")
    } else {
        "null".to_owned()
    }
}

fn handle(h: Option<u64>) -> Option<String> {
    h.map(|h| format!("{h:X}"))
}

/// A JSON string literal.
fn json_string(out: &mut Out, text: &str) -> Result<(), AnalysisError> {
    out.s("\"")?;
    let mut start = 0;
    for (i, ch) in text.char_indices() {
        let escaped = match ch {
            '"' => Some("\\\"".to_owned()),
            '\\' => Some("\\\\".to_owned()),
            c if (c as u32) < 0x20 => Some(format!("\\u{:04x}", c as u32)),
            _ => None,
        };
        if let Some(e) = escaped {
            out.s(&text[start..i])?;
            out.s(&e)?;
            start = i + ch.len_utf8();
        }
    }
    out.s(&text[start..])?;
    out.s("\"")
}

fn json_opt_string(out: &mut Out, text: Option<&str>) -> Result<(), AnalysisError> {
    match text {
        Some(t) => json_string(out, t),
        None => out.s("null"),
    }
}

fn level(l: Level) -> &'static str {
    match l {
        Level::Info => "info",
        Level::Caution => "caution",
    }
}

/// A warning's message: fixed text per code, never drawing text (SECURITY.md).
fn message(i: &Issue) -> &'static str {
    match (i.code, i.detail, i.level) {
        ("NOT_SUPPORTED", Detail::UnitCode(_), _) => {
            "This drawing unit is not converted in 0.1; quantities are in drawing units only."
        }
        ("NOT_SUPPORTED", Detail::EntityType(_), Level::Caution) => {
            "Entities of this type are not interpreted in 0.1, so they are left out of this layer's totals (counted in omitted)."
        }
        ("NOT_SUPPORTED", Detail::EntityType(_), Level::Info) => {
            "Entities of this type are not interpreted in 0.1; here they carry no quantity (text-like, in paper space, or in a block that is never placed), so no total is affected."
        }
        ("NOT_SUPPORTED", _, Level::Info) => {
            "The size check covered only the first areas; the totals are complete."
        }
        ("NOT_SUPPORTED", _, _) => {
            "Not supported in 0.1, so left out of this layer's totals (counted in omitted)."
        }
        ("DRAWING_GEOMETRY_INVALID", _, _) => {
            "The geometry is invalid (for example a ring that crosses itself), so its area is left out of this layer's totals."
        }
        ("DRAWING_GEOMETRY_NOTE", _, _) => {
            "A degenerate detail of the geometry was handled as documented (see note)."
        }
        ("ROUNDING_SENSITIVE", _, _) => {
            "The geometry is far from the origin relative to its size, so precision is limited."
        }
        ("DRAWING_BLOCK_MISSING", _, _) => {
            "An INSERT names a block that is not defined; it is left out of this layer's totals."
        }
        ("DRAWING_BLOCK_CYCLE", _, _) => {
            "A block inserts itself, directly or through others; that INSERT is left out of this layer's totals."
        }
        ("DRAWING_ZERO_SCALE", _, _) => {
            "An INSERT has a scale factor of 0; it is left out of this layer's totals."
        }
        ("DRAWING_HATCH_NOT_READ", _, _) => {
            "A hatch boundary could not be read; its area is left out of this layer's totals."
        }
        ("DRAWING_TEXT_NOT_DECODED", _, _) => {
            "Some text could not be decoded; undecodable bytes are shown as U+FFFD, never guessed."
        }
        ("DRAWING_UNITS_NOT_STATED", Detail::UnitCode(0), _) => {
            "The drawing declares itself unitless; quantities are in drawing units only, never assumed to be metres."
        }
        ("DRAWING_UNITS_NOT_STATED", _, _) => {
            "The drawing does not state its units; quantities are in drawing units only, never assumed to be metres."
        }
        ("DRAWING_UNITS_CONFLICT", _, _) => {
            "$INSUNITS and $MEASUREMENT disagree: one is imperial, the other metric."
        }
        ("DRAWING_BLOCK_SCALE_MISMATCH", _, _) => {
            "One block is placed at scales a unit-conversion ratio apart (for example 1000); check the units of its placements."
        }
        ("DRAWING_SIZE_OUTLIER", _, _) => {
            "An area is at least 1000 times the median of its layer and kind (or at most 1/1000 of it); it may be drawn in other units."
        }
        ("DRAWING_DIMENSION_OVERRIDE", _, Level::Caution) => {
            "A dimension's text overrides its measurement with a different number (DIMLFAC is not read in 0.1, so a dimension scale could explain it)."
        }
        ("DRAWING_DIMENSION_OVERRIDE", _, _) => {
            "A dimension's text overrides its measurement with text that is not a number."
        }
        ("DRAWING_REPORT_TRUNCATED", _, _) => {
            "Only the first rows of this list are given; the totals are complete."
        }
        _ => "See the warning code.",
    }
}

const DIMENSION_TYPES: [&str; 7] = [
    "rotated",
    "aligned",
    "angular",
    "diameter",
    "radius",
    "angular_3_point",
    "ordinate",
];

/// The eight totals of a layer, with their JSON names and whether each is an area.
fn totals(l: &LayerTotals) -> [(&'static str, f64, bool); 8] {
    [
        ("line_length", l.line_length, false),
        ("arc_length", l.arc_length, false),
        ("polyline_open_length", l.polyline_open_length, false),
        (
            "polyline_closed_perimeter",
            l.polyline_closed_perimeter,
            false,
        ),
        ("polyline_closed_area", l.polyline_closed_area, true),
        ("circle_circumference", l.circle_circumference, false),
        ("circle_area", l.circle_area, true),
        ("hatch_area", l.hatch_area, true),
    ]
}

/// A schedule row per kind: name, count, length, area.
fn kinds(l: &LayerTotals) -> [(&'static str, u64, Option<f64>, Option<f64>); 8] {
    let c = l.counts;
    [
        ("line", c.line, Some(l.line_length), None),
        ("arc", c.arc, Some(l.arc_length), None),
        (
            "circle",
            c.circle,
            Some(l.circle_circumference),
            Some(l.circle_area),
        ),
        (
            "polyline_open",
            c.polyline_open,
            Some(l.polyline_open_length),
            None,
        ),
        (
            "polyline_closed",
            c.polyline_closed,
            Some(l.polyline_closed_perimeter),
            Some(l.polyline_closed_area),
        ),
        ("hatch", c.hatch, None, Some(l.hatch_area)),
        ("text", c.text, None, None),
        ("dimension", c.dimension, None, None),
    ]
}

impl Analysis {
    /// The drawing analysed.
    pub fn drawing(&self) -> &Drawing {
        &self.drawing
    }

    /// Its quantities.
    pub fn quantities(&self) -> &Quantities {
        &self.quantities
    }

    fn out(&self) -> Out {
        Out(BVec::new(&self.budget))
    }

    fn entity_at(&self, block: Option<usize>, index: usize) -> Option<&Entity> {
        match block {
            Some(b) => self.drawing.blocks().get(b)?.entities.get(index),
            None => self.drawing.entities().get(index),
        }
    }

    /// Layers in name order (the list of references reserved from the budget too).
    fn sorted_layers(&self) -> Result<BVec<&LayerTotals>, AnalysisError> {
        let mut layers = BVec::new(&self.budget);
        for l in self.quantities.layers() {
            layers.push(l)?;
        }
        layers.sort_by(|a, b| self.drawing.name(a.layer).cmp(self.drawing.name(b.layer)));
        Ok(layers)
    }

    /// The report as JSON (API_DESIGN §13).
    ///
    /// # Errors
    /// [`AnalysisError::Memory`] when the budget refuses the output.
    pub fn to_json(&self) -> Result<String, AnalysisError> {
        let d = &self.drawing;
        let q = &self.quantities;
        let u = q.units();
        let mut o = self.out();
        o.s("{\"report\":")?;
        json_string(&mut o, REPORT_ID)?;
        o.s(",\"version\":")?;
        json_string(&mut o, REPORT_VERSION)?;
        o.s(",\"civkit_version\":")?;
        json_string(&mut o, CIVKIT_VERSION)?;
        o.s(",\"drawing\":{\"acadver\":")?;
        json_opt_string(&mut o, d.acadver())?;
        o.s(",\"codepage\":")?;
        json_opt_string(&mut o, d.codepage())?;
        o.s(",\"units\":{\"insunits\":")?;
        o.s(&u.insunits.map_or("null".to_owned(), |c| c.to_string()))?;
        o.s(",\"name\":")?;
        json_opt_string(&mut o, u.name)?;
        o.s(",\"to_metres\":")?;
        o.s(&u.to_metres.map_or("null".to_owned(), number))?;
        o.s(&format!(",\"stated\":{}}}}}", u.to_metres.is_some()))?;
        o.s(",\"plan_quantities\":true,\"layers\":[")?;
        for (k, l) in self.sorted_layers()?.iter().copied().enumerate() {
            o.s(if k == 0 { "{\"name\":" } else { ",{\"name\":" })?;
            json_string(&mut o, d.name(l.layer))?;
            let c = l.counts;
            o.s(&format!(
                ",\"frozen\":{},\"off\":{},\"incomplete\":{},\"omitted\":{},\"counts\":{{\"line\":{},\"arc\":{},\"circle\":{},\"polyline_open\":{},\"polyline_closed\":{},\"hatch\":{},\"text\":{},\"dimension\":{}}}",
                l.frozen,
                l.off,
                l.incomplete(),
                l.omitted,
                c.line,
                c.arc,
                c.circle,
                c.polyline_open,
                c.polyline_closed,
                c.hatch,
                c.text,
                c.dimension
            ))?;
            o.s(",\"drawing_units\":{")?;
            for (i, (name, value, _)) in totals(l).into_iter().enumerate() {
                let sep = if i == 0 { "" } else { "," };
                o.s(&format!("{sep}\"{name}\":{}", number(value)))?;
            }
            o.s("},\"si\":")?;
            if u.to_metres.is_some() {
                o.s("{")?;
                for (i, (name, value, area)) in totals(l).into_iter().enumerate() {
                    let sep = if i == 0 { "" } else { "," };
                    let (suffix, si) = if area {
                        ("_m2", u.area_m2(value))
                    } else {
                        ("_m", u.length_m(value))
                    };
                    o.s(&format!(
                        "{sep}\"{name}{suffix}\":{}",
                        si.map_or("null".to_owned(), number)
                    ))?;
                }
                o.s("}}")?;
            } else {
                o.s("null}")?;
            }
        }
        o.s("],\"text\":[")?;
        for (k, row) in q.texts().iter().enumerate() {
            o.s(if k == 0 {
                "{\"layer\":"
            } else {
                ",{\"layer\":"
            })?;
            json_string(&mut o, d.name(row.layer))?;
            o.s(",\"handle\":")?;
            json_opt_string(&mut o, handle(row.handle).as_deref())?;
            o.s(&format!(
                ",\"x\":{},\"y\":{},\"value\":",
                number(row.x),
                number(row.y)
            ))?;
            let value = match self.entity_at(row.block, row.index).map(|e| &e.kind) {
                Some(EntityKind::Text(t)) => Some(&*t.value),
                _ => None,
            };
            json_opt_string(&mut o, value)?;
            o.s("}")?;
        }
        o.s("],\"dimensions\":[")?;
        for (k, row) in q.dimensions().iter().enumerate() {
            o.s(if k == 0 {
                "{\"layer\":"
            } else {
                ",{\"layer\":"
            })?;
            json_string(&mut o, d.name(row.layer))?;
            o.s(",\"handle\":")?;
            json_opt_string(&mut o, handle(row.handle).as_deref())?;
            let dim = match self.entity_at(row.block, row.index).map(|e| &e.kind) {
                Some(EntityKind::Dimension(dim)) => Some(dim),
                _ => None,
            };
            o.s(",\"type\":")?;
            json_opt_string(
                &mut o,
                dim.and_then(|x| DIMENSION_TYPES.get(usize::from(x.dim_type)).copied()),
            )?;
            o.s(",\"measurement\":")?;
            o.s(&dim
                .and_then(|x| x.measurement)
                .map_or("null".to_owned(), number))?;
            o.s(",\"override\":")?;
            json_opt_string(&mut o, dim.and_then(|x| x.text.as_deref()))?;
            o.s("}")?;
        }
        o.s("],\"not_interpreted\":[")?;
        for (k, &((kind, layer, place), count)) in d.unsupported_where().iter().enumerate() {
            o.s(if k == 0 { "{\"type\":" } else { ",{\"type\":" })?;
            json_string(&mut o, d.name(kind))?;
            o.s(",\"layer\":")?;
            json_string(&mut o, d.name(layer))?;
            let (where_, block) = match place {
                Where::ModelSpace => ("model_space", None),
                Where::PaperSpace => ("paper_space", None),
                Where::Block(b) => ("block", Some(d.name(b))),
            };
            o.s(&format!(",\"where\":\"{where_}\",\"block\":"))?;
            json_opt_string(&mut o, block)?;
            o.s(&format!(",\"count\":{count}}}"))?;
        }
        o.s("],\"warnings\":[")?;
        for (k, i) in q.issues().iter().enumerate() {
            o.s(if k == 0 { "{\"code\":" } else { ",{\"code\":" })?;
            json_string(&mut o, i.code)?;
            o.s(&format!(",\"level\":\"{}\",\"message\":", level(i.level)))?;
            json_string(&mut o, message(i))?;
            o.s(&format!(",\"count\":{}", i.count))?;
            if let Some(l) = i.layer {
                o.s(",\"layer\":")?;
                json_string(&mut o, d.name(l))?;
            }
            match i.detail {
                Detail::None => {}
                Detail::Feature(f) => {
                    o.s(",\"feature\":")?;
                    json_string(&mut o, f)?;
                }
                Detail::Note(n) => {
                    o.s(",\"note\":")?;
                    json_string(&mut o, n)?;
                }
                Detail::EntityType(t) => {
                    o.s(",\"entity_type\":")?;
                    json_string(&mut o, d.name(t))?;
                }
                Detail::UnitCode(c) => o.s(&format!(",\"unit_code\":{c}"))?,
                Detail::Block(b) => {
                    o.s(",\"block\":")?;
                    json_string(&mut o, d.name(b))?;
                }
            }
            o.s(",\"handles\":[")?;
            for (j, h) in i.handles().iter().enumerate() {
                o.s(&format!("{}\"{h:X}\"", if j == 0 { "" } else { "," }))?;
            }
            o.s("]}")?;
        }
        o.s("]")?;
        if let Some(rows) = q.entities() {
            o.s(",\"entities\":[")?;
            for (k, e) in rows.iter().enumerate() {
                o.s(if k == 0 {
                    "{\"layer\":"
                } else {
                    ",{\"layer\":"
                })?;
                json_string(&mut o, d.name(e.layer))?;
                o.s(&format!(",\"kind\":\"{}\",\"handle\":", e.kind))?;
                json_opt_string(&mut o, handle(e.handle).as_deref())?;
                o.s(",\"top_handle\":")?;
                json_opt_string(&mut o, handle(e.top_handle).as_deref())?;
                o.s(",\"insert_handle\":")?;
                json_opt_string(&mut o, handle(e.insert_handle).as_deref())?;
                o.s(&format!(
                    ",\"length\":{},\"area\":{}}}",
                    e.length.map_or("null".to_owned(), number),
                    e.area.map_or("null".to_owned(), number)
                ))?;
            }
            o.s("]")?;
        }
        o.s("}")?;
        Ok(o.finish())
    }

    /// The quantity schedule as CSV (RFC 4180, CRLF, UTF-8 without BOM): one row per layer and kind
    /// measured, or one row for a layer with omissions only. Every row carries the report and CivKit
    /// versions (API_DESIGN §3).
    ///
    /// # Errors
    /// [`AnalysisError::Memory`] when the budget refuses the output.
    pub fn to_csv(&self) -> Result<String, AnalysisError> {
        let d = &self.drawing;
        let u = self.quantities.units();
        let unit = u.name.unwrap_or("");
        let mut o = self.out();
        o.s("layer,kind,count,length,area,unit,length_m,area_m2,incomplete,omitted,report_version,civkit_version\r\n")?;
        let opt = |x: Option<f64>| x.map_or(String::new(), number);
        for &l in self.sorted_layers()?.iter() {
            let layer = csv_cell(d.name(l.layer));
            let mut rows: Vec<(&str, u64, Option<f64>, Option<f64>)> =
                kinds(l).into_iter().filter(|k| k.1 > 0).collect();
            if rows.is_empty() {
                rows.push(("", 0, None, None));
            }
            for (kind, count, length, area) in rows {
                o.s(&format!(
                    "{layer},{kind},{count},{},{},{unit},{},{},{},{},{REPORT_VERSION},{CIVKIT_VERSION}\r\n",
                    opt(length),
                    opt(area),
                    opt(length.and_then(|x| u.length_m(x))),
                    opt(area.and_then(|x| u.area_m2(x))),
                    l.incomplete(),
                    l.omitted
                ))?;
            }
        }
        Ok(o.finish())
    }
}

/// A CSV cell from drawing text: a leading `=`, `+`, `-`, `@`, tab or carriage return is prefixed with
/// `'` (spreadsheets would run it as a formula); a cell with a comma, quote or line break is quoted,
/// quotes doubled (RFC 4180).
fn csv_cell(text: &str) -> String {
    let mut cell = String::with_capacity(text.len() + 3);
    if text.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        cell.push('\'');
    }
    cell.push_str(text);
    if cell.contains([',', '"', '\r', '\n']) {
        format!("\"{}\"", cell.replace('"', "\"\""))
    } else {
        cell
    }
}
