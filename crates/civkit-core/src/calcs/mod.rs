// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 10:56 IST · last edited 2026-11-26 20:58 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! Built-in calculations. Domain modules (geotech, structural, surveying, fluids, quantities, math) live here
//! until they grow large enough for their own crates (`docs/ARCHITECTURE.md` §2).

use crate::Calculation;

/// D-24 (ADR 0012): what a calculation states when no reliable basis for a validity range is established.
/// Mathematical domain checks still apply; a range is never invented to fill the field.
pub(crate) const NO_VALIDITY_RANGE: &str = "Validity range: none is stated, because no reliable source for one has \n     been established. The mathematical domain of every input is still enforced.";

pub mod demo;
pub mod fluids;
pub mod levelling;
pub mod quantities;
pub mod structural;
pub mod surveying;

/// Every built-in calculation. The 0.1 set (D-20) is added in Phase 2C, one at a time.
pub fn builtins() -> Vec<Box<dyn Calculation>> {
    // `demo` is deliberately not listed: it is a template and test vehicle, not an engineering calculation.
    vec![
        Box::new(fluids::ManningTrapezoidal),
        Box::new(structural::SimplySupportedBeam),
        Box::new(surveying::TraverseCompassRule),
        Box::new(levelling::HeightOfInstrument),
        Box::new(levelling::RiseAndFall),
        Box::new(levelling::ClosureAdjustment),
        Box::new(quantities::AverageEndArea),
    ]
}
