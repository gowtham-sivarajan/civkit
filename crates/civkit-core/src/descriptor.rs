// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 10:58 IST · last edited 2026-11-25 10:58 IST

//! Calculation descriptors: how a calculation describes itself (`docs/API_DESIGN.md` §2).
//! Every interface (Python, MCP, FFI, desktop forms, documentation) is generated from these.

use civkit_units::Dimension;
use serde::Serialize;

/// Everything an interface needs to know about one version of one calculation.
#[derive(Debug, Clone, PartialEq, Serialize)]
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
}

/// One input.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct InputSpec {
    /// Name used in requests, `snake_case`.
    pub name: String,
    /// What the input is, for documentation and AI clients.
    pub description: String,
    /// Physical dimension. Dimensional inputs must always carry a unit (D-19).
    pub dimension: Dimension,
    /// Unit shown in documentation and forms. Never applied silently to a bare number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_unit: Option<String>,
    /// Whether the request must contain it.
    pub required: bool,
    /// Value used when an optional input is omitted, in SI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<f64>,
    /// Mathematical domain, in SI. A value outside it is an `OUT_OF_DOMAIN` error.
    #[serde(skip_serializing_if = "Range::is_unbounded")]
    pub domain: Range,
}

/// One output.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OutputSpec {
    /// Name used in responses, `snake_case`.
    pub name: String,
    /// What the output is.
    pub description: String,
    /// Physical dimension.
    pub dimension: Dimension,
    /// SI unit the output is reported in by default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_unit: Option<String>,
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
