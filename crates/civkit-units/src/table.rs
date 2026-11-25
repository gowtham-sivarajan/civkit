// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 15:33 IST · last edited 2026-11-25 15:33 IST

//! The named units of 0.1 (`docs/API_DESIGN.md` §7) and the names rejected as ambiguous.
//!
//! Factors convert one unit to SI and are exact where the unit is defined exactly:
//! - international yard and pound agreement (1959): `in`, `ft`, `yd`, `mi`, `acre`, `lbm`
//! - standard gravity, 9.806 65 m/s² (CGPM 1901): `kgf`, `lbf`, `kip`
//! - US survey foot = 1200/3937 m
//! - US gallon = 231 in³; imperial gallon = 4.546 09 L
//!
//! Tests in `tests/definitions.rs` check these relations rather than restating the numbers.

use crate::Dimension;
use std::f64::consts::PI;

const L: Dimension = Dimension::LENGTH;
const A: Dimension = Dimension::AREA;
const V: Dimension = Dimension::VOLUME;
const M: Dimension = Dimension::new(0, 1, 0, 0);
const T: Dimension = Dimension::new(0, 0, 1, 0);
const F: Dimension = Dimension::new(1, 1, -2, 0); // force, M·L·T⁻²
const P: Dimension = Dimension::new(-1, 1, -2, 0); // pressure, M·L⁻¹·T⁻²
const ANG: Dimension = Dimension::ANGLE;
const ONE: Dimension = Dimension::DIMENSIONLESS;
const FLOW: Dimension = Dimension::new(3, 0, -1, 0);

const INCH: f64 = 0.0254;
const FOOT: f64 = 0.3048;
const POUND_MASS: f64 = 0.453_592_37;
const STANDARD_GRAVITY: f64 = 9.806_65;
const POUND_FORCE: f64 = POUND_MASS * STANDARD_GRAVITY;

/// `(name, SI factor, dimension)` for every named unit. Names are case-sensitive.
pub const NAMED: &[(&str, f64, Dimension)] = &[
    // Length
    ("mm", 1e-3, L),
    ("cm", 1e-2, L),
    ("m", 1.0, L),
    ("km", 1e3, L),
    ("in", INCH, L),
    ("ft", FOOT, L),
    ("ft_us", 1200.0 / 3937.0, L),
    ("yd", 3.0 * FOOT, L),
    ("mi", 5280.0 * FOOT, L),
    // Area
    ("ha", 1e4, A),
    ("acre", 43_560.0 * FOOT * FOOT, A),
    // Volume
    ("L", 1e-3, V),
    ("gal_us", 231.0 * INCH * INCH * INCH, V),
    ("gal_imp", 4.546_09e-3, V),
    // Mass
    ("g", 1e-3, M),
    ("kg", 1.0, M),
    ("t", 1e3, M),
    ("lbm", POUND_MASS, M),
    ("ton_short", 2000.0 * POUND_MASS, M),
    ("ton_long", 2240.0 * POUND_MASS, M),
    // Force
    ("N", 1.0, F),
    ("kN", 1e3, F),
    ("MN", 1e6, F),
    ("kgf", STANDARD_GRAVITY, F),
    ("lbf", POUND_FORCE, F),
    ("kip", 1000.0 * POUND_FORCE, F),
    // Pressure and stress
    ("Pa", 1.0, P),
    ("kPa", 1e3, P),
    ("MPa", 1e6, P),
    ("GPa", 1e9, P),
    ("bar", 1e5, P),
    ("psi", POUND_FORCE / (INCH * INCH), P),
    ("ksi", 1000.0 * POUND_FORCE / (INCH * INCH), P),
    ("psf", POUND_FORCE / (FOOT * FOOT), P),
    ("ksf", 1000.0 * POUND_FORCE / (FOOT * FOOT), P),
    // Time
    ("s", 1.0, T),
    ("min", 60.0, T),
    ("h", 3600.0, T),
    ("day", 86_400.0, T),
    // Angle (dms and bearing are structured values, handled in `angle`)
    ("rad", 1.0, ANG),
    ("deg", PI / 180.0, ANG),
    ("grad", PI / 200.0, ANG),
    // Dimensionless
    ("%", 0.01, ONE),
    // Alias
    ("cumec", 1.0, FLOW),
];

/// Names rejected as ambiguous, with what to write instead.
pub const AMBIGUOUS: &[(&str, &str)] = &[
    (
        "ton",
        "t (tonne), ton_short (2000 lb) or ton_long (2240 lb)",
    ),
    ("gal", "gal_us or gal_imp"),
    ("lb", "lbm (pound mass) or lbf (pound force)"),
    ("cal", "a specific energy unit (none are supported in 0.1)"),
    ("k", "kip"),
];

/// Looks up a named unit.
pub fn named(name: &str) -> Option<(f64, Dimension)> {
    NAMED
        .iter()
        .find(|(n, _, _)| *n == name)
        .map(|(_, f, d)| (*f, *d))
}
