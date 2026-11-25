// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 15:33 IST · last edited 2026-11-25 15:33 IST

//! Unit errors. `civkit-core` maps each to an error code of `docs/API_DESIGN.md` §5.

use crate::Dimension;

/// Why a unit string or a structured angle could not be used.
#[derive(Debug, Clone, PartialEq)]
pub enum UnitError {
    /// The unit string is empty.
    Empty,
    /// A name that is not a named unit. `hint` suggests a correction when one is obvious.
    Unknown {
        /// The unrecognised name.
        name: String,
        /// A suggestion, if any.
        hint: Option<String>,
    },
    /// A name that could mean two things.
    Ambiguous {
        /// The ambiguous name.
        name: String,
        /// What to write instead.
        instead: String,
    },
    /// A unit string that breaks the grammar (operators, exponents, characters).
    Syntax(String),
    /// A unit of the wrong dimension for the quantity.
    Mismatch {
        /// The dimension required.
        expected: Dimension,
        /// The dimension of the unit given.
        found: Dimension,
    },
    /// A malformed structured value: a DMS array or a bearing string.
    InvalidValue(String),
}

impl std::fmt::Display for UnitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => f.write_str("empty unit"),
            Self::Unknown {
                name,
                hint: Some(h),
            } => write!(f, "unknown unit '{name}'; {h}"),
            Self::Unknown { name, hint: None } => write!(f, "unknown unit '{name}'"),
            Self::Ambiguous { name, instead } => write!(f, "'{name}' is ambiguous; use {instead}"),
            Self::Syntax(msg) | Self::InvalidValue(msg) => f.write_str(msg),
            Self::Mismatch { expected, found } => {
                write!(f, "unit has dimension {found}, but {expected} is required")
            }
        }
    }
}

impl std::error::Error for UnitError {}
