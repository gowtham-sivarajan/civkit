// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 11:03 IST · last edited 2026-11-25 11:03 IST

//! The JSON request and response shapes (`docs/API_DESIGN.md` §3, §5, §6).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A request to run one calculation.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    /// Calculation ID.
    pub id: String,
    /// Calculation version. Omitted means the latest; the response always states the version used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,
    /// Inputs by name.
    #[serde(default)]
    pub inputs: BTreeMap<String, InputValue>,
    /// Units wanted for particular outputs; outputs not listed are returned in SI.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub output_units: BTreeMap<String, String>,
}

/// One input value as it arrives in JSON.
///
/// `value` is kept as raw JSON because some inputs are structured (a DMS angle is `[d, m, s]`, a bearing is
/// a string). Phase 1 accepts finite numbers only.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InputValue {
    /// The value.
    pub value: serde_json::Value,
    /// The unit; omitted only for dimensionless inputs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// Marks a negative DMS angle (`docs/API_DESIGN.md` §7).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub negative: Option<bool>,
}

impl InputValue {
    /// A dimensionless number.
    pub fn number(value: f64) -> Self {
        Self {
            value: serde_json::json!(value),
            unit: None,
            negative: None,
        }
    }
}

/// The result of a successful calculation.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Response {
    /// Calculation ID.
    pub id: String,
    /// The version actually used.
    pub version: u32,
    /// The CivKit version that produced this result (`docs/API_DESIGN.md` §11).
    pub civkit_version: String,
    /// Outputs by name.
    pub outputs: BTreeMap<String, OutputValue>,
    /// The steps that produced the outputs, in order, in SI.
    pub trace: Vec<TraceStep>,
    /// Warnings; empty when there are none.
    pub warnings: Vec<Warning>,
    /// Citations for the method.
    pub references: Vec<String>,
    /// Assumptions of the method.
    pub assumptions: Vec<String>,
}

/// One output value.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OutputValue {
    /// Full `f64` precision; interfaces round for display only.
    pub value: f64,
    /// Unit; omitted for dimensionless outputs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
}

/// One step of the trace: what was computed and the value it gave.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TraceStep {
    /// Symbolic expression, e.g. `R = A / P`.
    pub step: String,
    /// Value of the step, in SI.
    pub value: f64,
    /// SI unit; omitted when dimensionless.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// Source for this step (book page, code clause), if it differs from the calculation's references.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
}

/// How serious a warning is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    /// For information.
    Info,
    /// The result may be less reliable; check it.
    Caution,
}

/// Warning codes (`docs/API_DESIGN.md` §5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WarningCode {
    /// An input is outside the method's stated validity range.
    OutsideValidityRange,
    /// The result is sensitive to rounding of the inputs.
    RoundingSensitive,
}

/// A warning: the result is returned, but something deserves attention.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Warning {
    /// Machine-readable code.
    pub code: WarningCode,
    /// Human-readable explanation.
    pub message: String,
    /// Severity.
    pub level: Level,
}
