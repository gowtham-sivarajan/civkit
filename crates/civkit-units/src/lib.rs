// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 10:57 IST · last edited 2026-11-25 15:34 IST

//! # civkit-units
//!
//! Physical dimensions, the named units of CivKit 0.1, the unit-string parser and structured angles
//! (`docs/API_DESIGN.md` §7 and §7.1, decisions D-5 and D-19).
//!
//! - Every quantity is held in SI with its [`Dimension`]; angle is a dimension of its own.
//! - Only the named units in [`table::NAMED`] are recognised; there is no general prefix parsing.
//! - Ambiguous names (`ton`, `gal`, `lb`, `cal`, `k`) are rejected, never guessed.
//! - There is no default unit: a caller must always say which unit a number is in.
//!
//! ```
//! use civkit_units::{Dimension, Unit};
//!
//! let line_load = Unit::parse("kip/ft").unwrap();
//! assert_eq!(line_load.dimension, Dimension::new(0, 1, -2, 0)); // force per length
//! let si = line_load.to_si(2.0); // 2 kip/ft in N/m
//! assert!((si - 29_187.8).abs() < 0.1);
//! ```

mod angle;
mod dimension;
mod error;
pub mod table;
mod unit;

pub use angle::{bearing_to_azimuth, dms_to_radians};
pub use dimension::Dimension;
pub use error::UnitError;
pub use unit::Unit;
