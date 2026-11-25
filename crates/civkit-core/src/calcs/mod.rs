// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 11:06 IST · last edited 2026-11-25 11:06 IST

//! Built-in calculations. Domain modules (geotech, structural, surveying, fluids, quantities, math) live here
//! until they grow large enough for their own crates (`docs/ARCHITECTURE.md` §2).

use crate::Calculation;

pub mod demo;

/// Every built-in calculation. The 0.1 set is added in Phase 2.
pub fn builtins() -> Vec<Box<dyn Calculation>> {
    // `demo` is deliberately not listed: it is a template and test vehicle, not an engineering calculation.
    Vec::new()
}
