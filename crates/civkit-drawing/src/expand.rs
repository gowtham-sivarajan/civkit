// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 19:45 IST · last edited 2026-11-27 22:05 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! Block and MINSERT expansion (D-28 / ADR 0016 §4): placements are streamed to a visitor, never
//! collected into an expanded copy of the drawing. The walk is iterative (an explicit stack, at most
//! `max_depth` blocks deep), every allocation reserves from the [`Budget`] it is given, and its work
//! is limited, so any input finishes quickly or is refused.

use std::fmt;

use civkit_geometry::{Affine2, InsertPlacement, Point};

use crate::ReadError;
use crate::budget::{BVec, Budget};
use crate::model::{Drawing, Entity, EntityKind, Insert, NameId, Point3};

/// Limits on expansion (D-28 §4.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExpandLimits {
    /// Deepest block nesting allowed (ADR 0015: 32).
    pub max_depth: usize,
    /// Work: one unit per entity visited and per MINSERT cell placed.
    pub max_work: u64,
}

impl Default for ExpandLimits {
    fn default() -> Self {
        Self {
            max_depth: 32,
            max_work: 50_000_000,
        }
    }
}

/// Why analysis of a drawing stopped without a result.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AnalysisError {
    /// The DXF file could not be read ([`crate::analyze_dxf`]).
    Read(ReadError),
    /// A reservation from the budget was refused, or the allocator refused memory the budget allowed.
    Memory(ReadError),
    /// A limit was reached: `"nesting"` (block depth) or `"placements"` (work).
    Limit {
        /// Which limit.
        what: &'static str,
        /// Its value.
        limit: u64,
    },
}

impl fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(e) | Self::Memory(e) => write!(f, "{e}"),
            Self::Limit { what, limit } => {
                write!(f, "the drawing passes the {what} limit of {limit}")
            }
        }
    }
}

impl std::error::Error for AnalysisError {}

impl AnalysisError {
    /// The contract error code (API_DESIGN §5): `DRAWING_LIMIT` when a limit was reached (input size,
    /// per-item caps, the heap budget, nesting, placements, vertices), `DRAWING_UNREADABLE` when the file
    /// is not readable DXF.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Read(
                ReadError::InputTooLarge { .. }
                | ReadError::TooManyVertices { .. }
                | ReadError::StringTooLong { .. }
                | ReadError::BudgetExhausted { .. }
                | ReadError::AllocationFailed { .. },
            )
            | Self::Memory(_)
            | Self::Limit { .. } => "DRAWING_LIMIT",
            Self::Read(_) => "DRAWING_UNREADABLE",
        }
    }
}

impl From<ReadError> for AnalysisError {
    fn from(e: ReadError) -> Self {
        Self::Memory(e)
    }
}

/// Why an INSERT was not expanded (D-28 §4.3). Its layer's totals are then incomplete.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Skip {
    /// The block it names is not defined.
    MissingBlock,
    /// Its block is already being expanded above it: the block reaches itself.
    Cycle,
    /// A scale factor of 0 (determinant 0).
    ZeroScale,
    /// An extrusion other than (0, 0, ±1): not a plan entity.
    NonPlanExtrusion,
    /// Its placement transform is not finite (overflow).
    NotFinite,
}

/// One entity placed in the world: its block-to-world transform and its effective layer.
#[derive(Debug, Clone, Copy)]
pub struct Placed<'d> {
    /// The entity, in its block's coordinates (or the world's, at the top level).
    pub entity: &'d Entity,
    /// Block coordinates to world coordinates (the identity at the top level).
    pub transform: Affine2,
    /// Its layer, after layer `0` inside a block took the inserting layer.
    pub layer: NameId,
    /// Block nesting depth (0 at the top level).
    pub depth: usize,
    /// The handle of the top-level entity this placement comes from.
    pub top_handle: Option<u64>,
    /// The handle of the INSERT that placed it directly (none at the top level).
    pub insert_handle: Option<u64>,
    /// The block it belongs to, as a position in [`Drawing::blocks`] (none at the top level).
    pub block: Option<usize>,
    /// Its position in that block's entities (or in [`Drawing::entities`]).
    pub index: usize,
}

impl Placed<'_> {
    /// The transform from the entity's own stated coordinates to the world, in plan: the placement
    /// composed with the entity's OCS (GEOMETRY.md §11). LINE and DIMENSION points are stated in
    /// world (block) coordinates, so their extrusion does not move them. `None` for a non-plan
    /// extrusion.
    #[must_use]
    pub fn plan_transform(&self) -> Option<Affine2> {
        match self.entity.kind {
            EntityKind::Line { .. } | EntityKind::Dimension(_) => Some(self.transform),
            _ => ocs(self.entity.extrusion).and_then(|o| o.then(&self.transform).ok()),
        }
    }
}

/// An INSERT that was not expanded.
#[derive(Debug, Clone, Copy)]
pub struct Skipped<'d> {
    /// Why.
    pub reason: Skip,
    /// The INSERT.
    pub entity: &'d Entity,
    /// Its effective layer.
    pub layer: NameId,
    /// Its block nesting depth.
    pub depth: usize,
}

/// What expansion reports to its visitor, in drawing order (depth first).
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum Event<'d> {
    /// An entity placed (every kind but INSERT, which expands instead).
    Placed(Placed<'d>),
    /// An INSERT not expanded.
    Skipped(Skipped<'d>),
    /// A block placed once (one INSERT, or one MINSERT cell), before its entities: content the reader did not
    /// interpret counts here, once per placement.
    Entered {
        /// The block, as a position in [`Drawing::blocks`].
        block: usize,
        /// The layer that layer `0` takes inside it.
        layer: NameId,
        /// Its nesting depth (1 for a block placed at the top level).
        depth: usize,
        /// Block coordinates to world coordinates for this placement.
        transform: Affine2,
    },
}

/// What an expansion did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ExpandStats {
    /// Work used (entities visited plus cells placed).
    pub work: u64,
    /// Block placements (INSERT cells) expanded.
    pub placements: u64,
    /// Deepest block nesting reached.
    pub max_depth: usize,
}

/// The map from an entity's OCS to plan coordinates: (0, 0, 1) is the identity and (0, 0, −1)
/// negates x, each within 10⁻⁹; any other extrusion is not a plan entity (GEOMETRY.md §11).
fn ocs(n: Point3) -> Option<Affine2> {
    const T: f64 = 1e-9;
    if n.x.abs() > T || n.y.abs() > T {
        None
    } else if (n.z - 1.0).abs() <= T {
        Some(Affine2::IDENTITY)
    } else if (n.z + 1.0).abs() <= T {
        Some(Affine2::mirror_x())
    } else {
        None
    }
}

enum Frame<'d> {
    /// Entities of the drawing (top level) or of one block placement.
    Entities {
        list: &'d [Entity],
        next: usize,
        transform: Affine2,
        /// The layer that layer `0` takes here (none at the top level).
        inherit: Option<NameId>,
        /// The block being placed (for cycles).
        block: Option<usize>,
        depth: usize,
        top: Option<u64>,
        insert: Option<u64>,
    },
    /// The cells of one INSERT (1 × 1, or a MINSERT array).
    Cells {
        entity: &'d Entity,
        insert: &'d Insert,
        block: usize,
        outer: Affine2,
        ocs: Affine2,
        layer: NameId,
        next: u64,
        total: u64,
        depth: usize,
        top: Option<u64>,
    },
}

enum Step<'d> {
    Pop,
    Entity {
        entity: &'d Entity,
        index: usize,
        block: Option<usize>,
        transform: Affine2,
        inherit: Option<NameId>,
        depth: usize,
        top: Option<u64>,
        insert: Option<u64>,
    },
    Cell {
        index: u64,
    },
}

struct Work {
    used: u64,
    limit: u64,
}

impl Work {
    fn take(&mut self, n: u64) -> Result<(), AnalysisError> {
        match self.used.checked_add(n) {
            Some(u) if u <= self.limit => {
                self.used = u;
                Ok(())
            }
            _ => Err(AnalysisError::Limit {
                what: "placements",
                limit: self.limit,
            }),
        }
    }
}

/// Expands the drawing's model space, calling `visit` for every placed entity and every INSERT not
/// expanded. Allocations reserve from `budget` (the parsed drawing's, D-28 §4.1) and are released
/// before it returns.
///
/// # Errors
/// [`AnalysisError::Limit`] when nesting passes `max_depth` or work passes `max_work` (a MINSERT
/// whose cells alone would pass it is refused before any is placed); [`AnalysisError::Memory`] when
/// a reservation is refused. No partial result is implied: the caller discards what it saw.
pub fn expand<'d>(
    drawing: &'d Drawing,
    budget: &Budget,
    limits: &ExpandLimits,
    mut visit: impl FnMut(Event<'d>),
) -> Result<ExpandStats, AnalysisError> {
    // Block by name: one slot per interned name.
    let mut table: BVec<u32> = BVec::filled(budget, drawing.names.len(), u32::MAX)?;
    for (i, b) in drawing.blocks.iter().enumerate() {
        if let (Some(slot), Ok(i)) = (table.get_mut(b.name.0 as usize), u32::try_from(i)) {
            *slot = i;
        }
    }
    let block_of = |name: NameId| {
        table
            .get(name.0 as usize)
            .copied()
            .filter(|&i| i != u32::MAX)
            .map(|i| i as usize)
    };
    let is_zero_layer = |layer: NameId| drawing.name(layer) == "0";

    let mut work = Work {
        used: 0,
        limit: limits.max_work,
    };
    let mut stats = ExpandStats::default();
    let mut stack: BVec<Frame<'d>> = BVec::new(budget);
    stack.push(Frame::Entities {
        list: &drawing.entities,
        next: 0,
        transform: Affine2::IDENTITY,
        inherit: None,
        block: None,
        depth: 0,
        top: None,
        insert: None,
    })?;

    loop {
        let step = match stack.last_mut() {
            None => break,
            Some(Frame::Entities {
                list,
                next,
                transform,
                inherit,
                block,
                depth,
                top,
                insert,
            }) => match list.get(*next) {
                None => Step::Pop,
                Some(entity) => {
                    *next += 1;
                    Step::Entity {
                        entity,
                        index: *next - 1,
                        block: *block,
                        transform: *transform,
                        inherit: *inherit,
                        depth: *depth,
                        top: *top,
                        insert: *insert,
                    }
                }
            },
            Some(Frame::Cells { next, total, .. }) => {
                if *next == *total {
                    Step::Pop
                } else {
                    *next += 1;
                    Step::Cell { index: *next - 1 }
                }
            }
        };

        match step {
            Step::Pop => {
                stack.pop();
            }
            Step::Entity {
                entity,
                index,
                block: in_block,
                transform,
                inherit,
                depth,
                top,
                insert,
            } => {
                work.take(1)?;
                if depth == 0 && entity.paper_space {
                    continue;
                }
                let layer = match inherit {
                    Some(l) if is_zero_layer(entity.layer) => l,
                    _ => entity.layer,
                };
                let top = if depth == 0 { entity.handle } else { top };
                let EntityKind::Insert(ins) = &entity.kind else {
                    visit(Event::Placed(Placed {
                        entity,
                        transform,
                        layer,
                        depth,
                        top_handle: top,
                        insert_handle: insert,
                        block: in_block,
                        index,
                    }));
                    continue;
                };
                let skip = |reason| {
                    Event::Skipped(Skipped {
                        reason,
                        entity,
                        layer,
                        depth,
                    })
                };
                let Some(block) = block_of(ins.block) else {
                    visit(skip(Skip::MissingBlock));
                    continue;
                };
                let cycle = stack
                    .iter()
                    .any(|f| matches!(f, Frame::Entities { block: Some(b), .. } if *b == block));
                if cycle {
                    visit(skip(Skip::Cycle));
                    continue;
                }
                let Some(insert_ocs) = ocs(entity.extrusion) else {
                    visit(skip(Skip::NonPlanExtrusion));
                    continue;
                };
                if ins.scale.x == 0.0 || ins.scale.y == 0.0 {
                    visit(skip(Skip::ZeroScale));
                    continue;
                }
                if depth + 1 > limits.max_depth {
                    return Err(AnalysisError::Limit {
                        what: "nesting",
                        limit: limits.max_depth as u64,
                    });
                }
                let total = u64::from(ins.columns) * u64::from(ins.rows);
                if total > work.limit - work.used {
                    return Err(AnalysisError::Limit {
                        what: "placements",
                        limit: work.limit,
                    });
                }
                stack.push(Frame::Cells {
                    entity,
                    insert: ins,
                    block,
                    outer: transform,
                    ocs: insert_ocs,
                    layer,
                    next: 0,
                    total,
                    depth,
                    top,
                })?;
            }
            Step::Cell { index } => {
                work.take(1)?;
                let Some(&Frame::Cells {
                    entity,
                    insert: ins,
                    block,
                    outer,
                    ocs: insert_ocs,
                    layer,
                    depth,
                    top,
                    ..
                }) = stack.last()
                else {
                    break;
                };
                let columns = u64::from(ins.columns);
                let (c, r) = (index % columns, index / columns);
                let b = &drawing.blocks[block];
                let placement = Affine2::insert(&InsertPlacement {
                    base: Point::new(b.base.x, b.base.y),
                    at: Point::new(ins.point.x, ins.point.y),
                    scale: (ins.scale.x, ins.scale.y),
                    rotation_deg: ins.rotation_deg,
                    offset: (c as f64 * ins.column_spacing, r as f64 * ins.row_spacing),
                })
                .and_then(|t| t.then(&insert_ocs))
                .and_then(|t| t.then(&outer));
                let skip = |reason| {
                    Event::Skipped(Skipped {
                        reason,
                        entity,
                        layer,
                        depth,
                    })
                };
                let t = match placement {
                    Ok(t) if t.det() != 0.0 => t,
                    Ok(_) => {
                        visit(skip(Skip::ZeroScale));
                        continue;
                    }
                    Err(_) => {
                        visit(skip(Skip::NotFinite));
                        continue;
                    }
                };
                stats.placements += 1;
                stats.max_depth = stats.max_depth.max(depth + 1);
                visit(Event::Entered {
                    block,
                    layer,
                    depth: depth + 1,
                    transform: t,
                });
                stack.push(Frame::Entities {
                    list: &b.entities,
                    next: 0,
                    transform: t,
                    inherit: Some(layer),
                    block: Some(block),
                    depth: depth + 1,
                    top,
                    insert: entity.handle,
                })?;
            }
        }
    }
    stats.work = work.used;
    Ok(stats)
}
