// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 11:00 IST · last edited 2026-11-25 11:00 IST

//! Errors: the codes of `docs/API_DESIGN.md` §5 and the JSON error envelope.

use serde::{Serialize, Serializer};

/// Every error code a request can produce (`docs/API_DESIGN.md` §5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    /// No calculation has this ID.
    UnknownCalculation,
    /// The calculation exists but not in the requested version.
    UnknownVersion,
    /// A required input is missing.
    MissingInput,
    /// The request names an input the calculation does not have.
    UnknownInput,
    /// `output_units` names an output the calculation does not have.
    UnknownOutput,
    /// The request is not valid JSON or does not have the request shape.
    InvalidJson,
    /// A value is malformed: not a finite number, a bad DMS array, a bad bearing.
    InvalidValue,
    /// A dimensional input was given as a bare number (no silent default unit, D-19).
    MissingUnit,
    /// A unit of the wrong dimension.
    UnitMismatch,
    /// A unit the engine does not know.
    UnknownUnit,
    /// A unit that could mean two things (`ton`, `gal`, `lb`, `k`, bare `cal`).
    AmbiguousUnit,
    /// Mathematically invalid input, e.g. a negative area.
    OutOfDomain,
    /// Input outside a method's stated validity range, where the calculation escalates that to an error.
    OutsideValidityRange,
    /// A bug in CivKit. Always worth reporting.
    Internal,
}

/// One error: a code, a human-readable message and, where it applies, the input it concerns.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CalcError {
    /// Machine-readable code.
    pub code: ErrorCode,
    /// Human-readable explanation.
    pub message: String,
    /// The input concerned, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,
}

impl CalcError {
    /// An error not tied to one input.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            input: None,
        }
    }

    /// An error about one input.
    pub fn for_input(
        code: ErrorCode,
        input: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            input: Some(input.into()),
        }
    }

    /// An internal error (a CivKit bug).
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Internal, message)
    }
}

impl std::fmt::Display for CalcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.input {
            Some(input) => write!(f, "{:?} ({input}): {}", self.code, self.message),
            None => write!(f, "{:?}: {}", self.code, self.message),
        }
    }
}

impl std::error::Error for CalcError {}

/// The JSON shape of a failed request: `{ "error": {…} }` for one error, `{ "errors": [ … ] }` for several.
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorEnvelope(pub Vec<CalcError>);

impl Serialize for ErrorEnvelope {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(1))?;
        match self.0.as_slice() {
            [single] => map.serialize_entry("error", single)?,
            many => map.serialize_entry("errors", many)?,
        }
        map.end()
    }
}
