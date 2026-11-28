// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 10:58 IST · last edited 2026-11-28 14:11 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! # civkit-core
//!
//! The calculation registry and JSON contract at the centre of CivKit (`docs/ARCHITECTURE.md` §3.1,
//! `docs/API_DESIGN.md`).
//!
//! Every calculation describes itself with a [`Descriptor`] and implements [`Calculation`]. The [`Registry`]
//! validates a [`Request`] against the descriptor (inputs, units, domains, validity rules), runs the
//! calculation, and returns a [`Response`] with outputs, trace, warnings, references and assumptions, or an
//! [`ErrorEnvelope`]. Python, MCP, the C ABI and the desktop app all go through [`Registry::call_json`], so
//! every interface gives the same answer.
//!
//! ```
//! use civkit_core::{Registry, calcs::demo::WeightedMean};
//!
//! let mut registry = Registry::new();
//! registry.register(Box::new(WeightedMean)).unwrap();
//! let json = registry.call_json(
//!     r#"{ "id": "math.demo.weighted_mean", "inputs": { "a": { "value": 2 }, "b": { "value": 4 } } }"#,
//! );
//! assert!(json.contains(r#""mean":{"value":3.0}"#));
//! ```
//!
//! This crate has no dependency on AutoCAD, Python, the network or any AI service (the dependency rule).

pub mod calcs;
mod contract;
mod descriptor;
mod error;
mod registry;

pub use contract::{
    InputValue, Level, Number, OutputValue, Request, Response, TraceItem, TraceStep, Warning,
    WarningCode,
};
pub use descriptor::{
    Breach, Descriptor, Group, InputKind, InputSpec, Limit, ListLength, OutputKind, OutputSpec,
    Range, ValidityRule,
};
pub use error::{CalcError, ErrorCode, ErrorEnvelope};
pub use registry::{
    Calculation, Context, DOMAINS, Inputs, MAX_ID_LEN, MAX_LIST_LEN, Outputs, RESERVED_INPUT_NAMES,
    Registry, RegistryError, check_descriptor,
};

/// The CivKit version, reported in every response.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The disclaimer every interface shows (docs/ENGINEERING_STANDARDS.md §7). It is shown by the interfaces (Python
/// documentation and display, MCP instructions and tool descriptions, generated pages), never added to the JSON
/// contract, so responses stay identical across interfaces (D-13).
pub const DISCLAIMER: &str =
    "CivKit results support, and do not replace, the judgement of a qualified engineer.";
