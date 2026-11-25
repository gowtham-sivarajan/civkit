// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 15:35 IST · last edited 2026-11-25 15:35 IST

//! Checks the unit table against **defining relations**, not against restated numbers: each test expresses
//! one unit through others (1 mi = 5280 ft, 1 lbf = 1 lbm × standard gravity, 1 gal_us = 231 in³, …).
//! A wrong factor in the table breaks a relation even if the same wrong number were copied into a test.

use civkit_units::{Dimension, Unit, table::NAMED};

fn u(text: &str) -> Unit {
    Unit::parse(text).unwrap_or_else(|e| panic!("'{text}' should parse: {e}"))
}

/// Asserts `count` × `a` equals one `b` (same dimension, relative tolerance 1e-12).
fn relation(count: f64, a: &str, b: &str) {
    let (ua, ub) = (u(a), u(b));
    assert_eq!(
        ua.dimension, ub.dimension,
        "{a} and {b} differ in dimension"
    );
    let lhs = count * ua.factor;
    let rel = (lhs - ub.factor).abs() / ub.factor;
    assert!(
        rel < 1e-12,
        "{count} {a} should equal 1 {b}: {lhs} vs {} (rel {rel:e})",
        ub.factor
    );
}

#[test]
fn si_prefixed_names() {
    relation(1000.0, "mm", "m");
    relation(100.0, "cm", "m");
    relation(1000.0, "m", "km");
    relation(1000.0, "N", "kN");
    relation(1000.0, "kN", "MN");
    relation(1000.0, "Pa", "kPa");
    relation(1000.0, "kPa", "MPa");
    relation(1000.0, "MPa", "GPa");
    relation(1.0, "N/mm^2", "MPa");
    relation(100.0, "kPa", "bar");
    relation(1000.0, "g", "kg");
    relation(1000.0, "kg", "t");
    relation(1000.0, "L", "m^3");
    relation(1.0, "m^3/s", "cumec");
    relation(10_000.0, "m^2", "ha");
}

#[test]
fn international_inch_foot_yard_mile() {
    relation(1.0, "in", "in"); // parses
    relation(12.0, "in", "ft");
    relation(3.0, "ft", "yd");
    relation(5280.0, "ft", "mi");
    relation(1.0 / 0.0254, "in", "m"); // 1 in = 25.4 mm exactly (1959 agreement)
    relation(43_560.0, "ft^2", "acre");
}

#[test]
fn us_survey_foot() {
    relation(3937.0 / 1200.0, "ft_us", "m");
    let ratio = u("ft_us").factor / u("ft").factor;
    assert!(
        (ratio - 1.0 - 2e-6).abs() < 1e-8,
        "survey foot is 2 ppm longer, got {ratio}"
    );
}

#[test]
fn gallons() {
    relation(231.0, "in^3", "gal_us");
    relation(4.546_09, "L", "gal_imp");
}

#[test]
fn force_from_mass_and_standard_gravity() {
    let g0 = 9.806_65; // m/s², CGPM 1901
    let lbf = u("lbf").factor;
    assert!((lbf - u("lbm").factor * g0).abs() < 1e-12);
    assert!((u("kgf").factor - u("kg").factor * g0).abs() < 1e-12);
    relation(1000.0, "lbf", "kip");
    relation(2000.0, "lbm", "ton_short");
    relation(2240.0, "lbm", "ton_long");
}

#[test]
fn imperial_pressures() {
    relation(1.0, "lbf/in^2", "psi");
    relation(1000.0, "psi", "ksi");
    relation(1.0, "lbf/ft^2", "psf");
    relation(1000.0, "psf", "ksf");
    relation(144.0, "psf", "psi");
    relation(1.0, "kip/in^2", "ksi");
}

#[test]
fn time_and_angle() {
    relation(60.0, "s", "min");
    relation(60.0, "min", "h");
    relation(24.0, "h", "day");
    let full_turn = 2.0 * std::f64::consts::PI;
    assert!((360.0 * u("deg").factor - full_turn).abs() < 1e-12);
    assert!((400.0 * u("grad").factor - full_turn).abs() < 1e-12);
    assert_eq!(u("rad").factor.to_bits(), 1.0f64.to_bits());
}

#[test]
fn every_named_unit_parses_to_its_table_entry() {
    for (name, factor, dimension) in NAMED {
        let parsed = u(name);
        assert_eq!(parsed.dimension, *dimension, "{name}");
        assert_eq!(parsed.factor.to_bits(), factor.to_bits(), "{name}");
    }
}

#[test]
fn dimensions_of_compounds_used_by_c1_to_c9() {
    let force = Dimension::new(1, 1, -2, 0);
    let cases = [
        ("m^2", Dimension::AREA),
        ("m^3", Dimension::VOLUME),
        ("ft", Dimension::LENGTH),
        ("m^3/s", Dimension::new(3, 0, -1, 0)),
        ("m/s", Dimension::new(1, 0, -1, 0)),
        ("kip", force),
        ("kip/ft", Dimension::new(0, 1, -2, 0)),
        ("kN/m", Dimension::new(0, 1, -2, 0)),
        ("ksi", Dimension::new(-1, 1, -2, 0)),
        ("GPa", Dimension::new(-1, 1, -2, 0)),
        ("in^4", Dimension::new(4, 0, 0, 0)),
        ("mm^4", Dimension::new(4, 0, 0, 0)),
        ("kip*ft^2", Dimension::new(3, 1, -2, 0)),
        ("kN·m", Dimension::new(2, 1, -2, 0)),
        ("rad", Dimension::ANGLE),
    ];
    for (text, expected) in cases {
        assert_eq!(u(text).dimension, expected, "{text}");
    }
}
