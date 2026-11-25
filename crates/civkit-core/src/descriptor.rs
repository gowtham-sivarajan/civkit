// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 10:58 IST · last edited 2026-11-25 22:03 IST

//! Calculation descriptors: how a calculation describes itself (`docs/API_DESIGN.md` §2).
//! Every interface (Python, MCP, FFI, desktop forms, documentation) is generated from these.

use civkit_units::Dimension;
use serde::Serialize;

/// Everything an interface needs to know about one version of one calculation.
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct Descriptor {
    /// Namespaced ID, `domain.topic.method`, e.g. `fluids.open_channel.manning`.
    pub id: String,
    /// Starts at 1. Any change to a result for the same input bumps it (`docs/API_DESIGN.md` §4).
    pub version: u32,
    /// Short human-readable name.
    pub title: String,
    /// One-paragraph description.
    pub summary: String,
    /// Inputs, in the order interfaces should present them.
    pub inputs: Vec<InputSpec>,
    /// Outputs, in presentation order.
    pub outputs: Vec<OutputSpec>,
    /// Citations: book, edition, page/section, or code clause.
    pub references: Vec<String>,
    /// Plain-language assumptions.
    pub assumptions: Vec<String>,
    /// Stated validity limits of the method, beyond the mathematical domain of each input.
    pub validity: Vec<ValidityRule>,
    /// Paths of reference-case files in `tests/reference/`.
    pub examples: Vec<String>,
    /// Named index sets shared by list inputs (D-23). Omitted from JSON when empty.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<Group>,
}

/// A group of list inputs that share one index: entry k of every list in the group belongs together (D-23).
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct Group {
    /// Name, `snake_case`, referenced by list inputs and list outputs.
    pub name: String,
    /// Singular word used to number entries from 1 in messages and the trace ("leg 3").
    pub item: String,
    /// Smallest allowed length, at least 1.
    pub min_len: usize,
    /// What one entry is and how the entries are ordered.
    pub description: String,
}

/// The shape of an input (D-23). Omitted from JSON when `scalar`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InputKind {
    /// One value: a number, a DMS array or a bearing.
    #[default]
    Scalar,
    /// A JSON array with one unit for the whole list; belongs to a group. Required in 0.1.
    List,
    /// One string from `choices`. Required, with no default, in 0.1.
    Choice,
}

impl InputKind {
    /// `true` for `Scalar`.
    pub fn is_scalar(&self) -> bool {
        *self == Self::Scalar
    }
}

/// The shape of an output (D-23). Omitted from JSON when `scalar`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputKind {
    /// One value.
    #[default]
    Scalar,
    /// A list whose length follows a group (see [`ListLength`]).
    List,
}

impl OutputKind {
    /// `true` for `Scalar`.
    pub fn is_scalar(&self) -> bool {
        *self == Self::Scalar
    }
}

/// How long a list output is, and what its entries are called (D-23).
///
/// Its index set is (`group`, `offset`): the length is the group's length + `offset`. `item` names that index
/// set; outputs that share a group and offset may share an item label.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct ListLength {
    /// The group the length follows.
    pub group: String,
    /// −1 (one entry between consecutive group items), 0 (one per item) or +1 (one per item plus one).
    pub offset: i8,
    /// Singular word for one entry ("segment").
    pub item: String,
}

/// One input.
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct InputSpec {
    /// Name used in requests, `snake_case`.
    pub name: String,
    /// What the input is, for documentation and AI clients.
    pub description: String,
    /// Physical dimension. Dimensional inputs must always carry a unit (D-19).
    pub dimension: Dimension,
    /// **Internal calculation unit**: the canonical coherent SI unit without prefix (e.g. `m`, `Pa`, `m^3/s`,
    /// `rad`) in which `compute` receives the value and in which `domain` and validity ranges are stated.
    /// `None` only for dimensionless inputs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub si_unit: Option<String>,
    /// **Default presentation unit** for documentation and forms (may differ from `si_unit`, e.g. `kN`).
    /// Never applied silently to a bare number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_unit: Option<String>,
    /// Whether the request must contain it.
    pub required: bool,
    /// Value used when an optional input is omitted, in SI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<f64>,
    /// Whether this angle input also accepts a quadrant bearing (`"unit": "bearing"`), converted to an azimuth.
    /// Only for surveying inputs; requires the angle dimension (`docs/API_DESIGN.md` §7).
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub accepts_bearing: bool,
    /// Mathematical domain, in SI. A value outside it is an `OUT_OF_DOMAIN` error. For a list, it applies to
    /// every element.
    #[serde(skip_serializing_if = "Range::is_unbounded")]
    pub domain: Range,
    /// Scalar, list or choice (D-23).
    #[serde(skip_serializing_if = "InputKind::is_scalar")]
    pub kind: InputKind,
    /// For a list input: the group it belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// For a choice input: the allowed values, lower-case `snake_case`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub choices: Vec<String>,
}

/// One output.
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct OutputSpec {
    /// Name used in responses, `snake_case`.
    pub name: String,
    /// What the output is.
    pub description: String,
    /// Physical dimension.
    pub dimension: Dimension,
    /// **Internal calculation unit**: canonical coherent SI without prefix. `compute` returns the value in it and
    /// the trace uses it. `None` only for dimensionless outputs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub si_unit: Option<String>,
    /// **Default presentation unit**: the unit the response uses when the request does not name one in
    /// `output_units` (may differ from `si_unit`, e.g. `mm` for a deflection). `None` means present in `si_unit`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_unit: Option<String>,
    /// Scalar or list (D-23).
    #[serde(skip_serializing_if = "OutputKind::is_scalar")]
    pub kind: OutputKind,
    /// For a list output: its length rule and item label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<ListLength>,
}

/// One end of a range.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Limit {
    /// The limit value, in SI.
    pub value: f64,
    /// Whether the limit value itself is allowed.
    pub inclusive: bool,
}

/// An interval on the real line; either end may be open-ended.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize)]
pub struct Range {
    /// Lower limit, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<Limit>,
    /// Upper limit, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<Limit>,
}

impl Range {
    /// No limits.
    pub const UNBOUNDED: Self = Self {
        min: None,
        max: None,
    };

    /// `[min, max]`, both ends allowed.
    pub const fn closed(min: f64, max: f64) -> Self {
        Self {
            min: Some(Limit {
                value: min,
                inclusive: true,
            }),
            max: Some(Limit {
                value: max,
                inclusive: true,
            }),
        }
    }

    /// `value > min`.
    pub const fn greater_than(min: f64) -> Self {
        Self {
            min: Some(Limit {
                value: min,
                inclusive: false,
            }),
            max: None,
        }
    }

    /// `value >= min`.
    pub const fn at_least(min: f64) -> Self {
        Self {
            min: Some(Limit {
                value: min,
                inclusive: true,
            }),
            max: None,
        }
    }

    /// `true` when neither end is limited.
    pub fn is_unbounded(&self) -> bool {
        self.min.is_none() && self.max.is_none()
    }

    /// Whether `value` lies in the range.
    pub fn contains(&self, value: f64) -> bool {
        let above_min = match self.min {
            None => true,
            Some(Limit {
                value: m,
                inclusive: true,
            }) => value >= m,
            Some(Limit {
                value: m,
                inclusive: false,
            }) => value > m,
        };
        let below_max = match self.max {
            None => true,
            Some(Limit {
                value: m,
                inclusive: true,
            }) => value <= m,
            Some(Limit {
                value: m,
                inclusive: false,
            }) => value < m,
        };
        above_min && below_max
    }
}

impl std::fmt::Display for Range {
    /// Interval notation, e.g. `[0, 1]`, `(0, ∞)`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.min {
            None => f.write_str("(-∞")?,
            Some(Limit { value, inclusive }) => {
                write!(f, "{}{value}", if inclusive { '[' } else { '(' })?
            }
        }
        f.write_str(", ")?;
        match self.max {
            None => f.write_str("∞)"),
            Some(Limit { value, inclusive }) => {
                write!(f, "{value}{}", if inclusive { ']' } else { ')' })
            }
        }
    }
}

/// What happens when an input is outside a method's stated validity range (D-19).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Breach {
    /// Default: the result is returned with an `OUTSIDE_VALIDITY_RANGE` warning (level `caution`).
    Warning,
    /// Escalated: the request fails with `OUTSIDE_VALIDITY_RANGE`. Only when a result outside the range is
    /// meaningless rather than merely less accurate; the reason must be stated.
    Error,
}

/// A stated validity limit of the method for one input.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ValidityRule {
    /// The input the rule applies to.
    pub input: String,
    /// The range within which the method is valid, in SI.
    pub range: Range,
    /// Warning (default) or error.
    pub on_breach: Breach,
    /// Why the limit exists; required when `on_breach` is `error`.
    pub reason: String,
}
