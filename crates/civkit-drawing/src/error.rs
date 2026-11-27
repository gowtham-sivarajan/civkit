// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 15:40 IST · last edited 2026-11-27 15:40 IST

//! Why a DXF file could not be read. Every error is clean: no panic, nothing half-built returned. Line numbers are
//! 1-based lines of the file. Errors carry only numbers and static text, so making one allocates nothing.

use std::fmt;

/// Why a DXF file could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ReadError {
    /// The input is larger than [`crate::Limits::max_input_bytes`]; nothing was read.
    InputTooLarge {
        /// The input's size, in bytes.
        size: usize,
        /// The limit, in bytes.
        limit: usize,
    },
    /// A binary DXF file: not supported in 0.1 (ADR 0015).
    BinaryNotSupported,
    /// The file ends before `0 EOF`, or a group code has no value line.
    UnexpectedEnd {
        /// The last line read.
        line: u64,
    },
    /// A group-code line that is not an integer.
    BadGroupCode {
        /// Its line.
        line: u64,
    },
    /// The file's structure is not what DXF requires here.
    Malformed {
        /// The line of the value that does not fit.
        line: u64,
        /// What was expected there.
        expected: &'static str,
    },
    /// A value that should be a number and is not one, or is out of range for its group code.
    InvalidNumber {
        /// The value's line.
        line: u64,
        /// Its group code.
        code: i32,
    },
    /// A number that is NaN or infinite (including one too large for `f64`).
    NonFinite {
        /// The value's line.
        line: u64,
        /// Its group code.
        code: i32,
    },
    /// More vertices or edges in one polyline or hatch path than [`crate::Limits::max_vertices`].
    TooManyVertices {
        /// The line where the limit was passed (or where the count was stated).
        line: u64,
        /// The limit.
        limit: usize,
    },
    /// A string longer than [`crate::Limits::max_string_bytes`] (or, for an entity type name,
    /// [`crate::Limits::max_type_name_bytes`]).
    StringTooLong {
        /// Its line.
        line: u64,
        /// The limit, in bytes.
        limit: usize,
    },
    /// The heap budget would be exceeded ([`crate::Budget`]).
    BudgetExhausted {
        /// The bytes that could not be reserved.
        requested: usize,
        /// The budget, in bytes.
        limit: usize,
    },
    /// The allocator refused memory within the budget.
    AllocationFailed {
        /// The bytes asked for.
        bytes: usize,
    },
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputTooLarge { size, limit } => {
                write!(f, "the file is {size} bytes; the limit is {limit} bytes")
            }
            Self::BinaryNotSupported => f.write_str("binary DXF is not supported in this version"),
            Self::UnexpectedEnd { line } => {
                write!(
                    f,
                    "the file ends at line {line} before its end marker (0 EOF)"
                )
            }
            Self::BadGroupCode { line } => {
                write!(f, "line {line}: a group code must be an integer")
            }
            Self::Malformed { line, expected } => write!(f, "line {line}: expected {expected}"),
            Self::InvalidNumber { line, code } => {
                write!(
                    f,
                    "line {line}: the value of group {code} is not a valid number"
                )
            }
            Self::NonFinite { line, code } => {
                write!(f, "line {line}: the value of group {code} is not finite")
            }
            Self::TooManyVertices { line, limit } => {
                write!(
                    f,
                    "line {line}: more than {limit} vertices or edges in one polyline or hatch path"
                )
            }
            Self::StringTooLong { line, limit } => {
                write!(f, "line {line}: a string longer than {limit} bytes")
            }
            Self::BudgetExhausted { requested, limit } => write!(
                f,
                "the drawing needs more memory than its budget of {limit} bytes (could not reserve {requested} more)"
            ),
            Self::AllocationFailed { bytes } => {
                write!(f, "the system could not provide {bytes} bytes of memory")
            }
        }
    }
}

impl std::error::Error for ReadError {}
