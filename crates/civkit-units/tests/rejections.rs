// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 15:35 IST · last edited 2026-11-25 15:59 IST

//! Everything the parser must refuse, and how (`docs/API_DESIGN.md` §7, §7.1).

use civkit_units::{Dimension, Unit, UnitError, bearing_to_azimuth, dms_to_radians};

fn err(text: &str) -> UnitError {
    match Unit::parse(text) {
        Ok(u) => panic!("'{text}' should be rejected, parsed as {u:?}"),
        Err(e) => e,
    }
}

#[test]
fn ambiguous_names_are_rejected_not_guessed() {
    for name in ["ton", "gal", "lb", "cal", "k"] {
        assert!(matches!(err(name), UnitError::Ambiguous { .. }), "{name}");
        // also inside compounds
        assert!(
            matches!(err(&format!("{name}/m")), UnitError::Ambiguous { .. }),
            "{name}/m"
        );
    }
    // the suggested replacements must exist
    for name in [
        "t",
        "ton_short",
        "ton_long",
        "gal_us",
        "gal_imp",
        "lbm",
        "lbf",
        "kip",
    ] {
        assert!(
            Unit::parse(name).is_ok(),
            "{name} is offered as a replacement, so it must parse"
        );
    }
}

#[test]
fn no_general_prefixes_and_case_sensitive() {
    for text in [
        "mN", "Mm", "um", "µm", "nm", "kft", "KN", "kn", "MPA", "pa", "M", "l", "Kg", "sec", "hr",
        "degC",
    ] {
        assert!(matches!(err(text), UnitError::Unknown { .. }), "{text}");
    }
    match err("KN") {
        UnitError::Unknown { hint: Some(h), .. } => {
            assert!(h.contains("kN"), "hint should suggest kN: {h}")
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn grammar_violations() {
    let bad = [
        "", "m/", "/s", "m*", "*m", "m**2", "m^", "m^0", "m^-0", "m^10", "m^1.5", "m^+2", "m^2^3",
        "m s", " m", "m ", "kN / m", "m/s/s", "W/m^2*K", "kN/m*s", "(kN*m)", "m²", "m³/s", "2*m",
        "1000mm", "%*m", "m/%", "m^a",
    ];
    for text in bad {
        assert!(Unit::parse(text).is_err(), "'{text}' should be rejected");
    }
    assert!(matches!(err(""), UnitError::Empty));
    match err("m²") {
        UnitError::Unknown { hint: Some(h), .. } => assert!(h.contains('^')),
        other => panic!("{other:?}"),
    }
}

#[test]
fn dimension_mismatch_is_reported() {
    let e = Unit::parse("kN")
        .and_then(|u| u.expect(Dimension::LENGTH))
        .expect_err("mismatch");
    assert!(matches!(e, UnitError::Mismatch { .. }));
    assert!(e.to_string().contains("L*M*T^-2"), "{e}");
}

#[test]
fn angle_and_dimensionless_are_different() {
    assert_ne!(
        Unit::parse("rad").map(|u| u.dimension),
        Unit::parse("%").map(|u| u.dimension)
    );
    assert!(
        Unit::parse("deg")
            .and_then(|u| u.expect(Dimension::DIMENSIONLESS))
            .is_err()
    );
}

#[test]
fn bad_dms_arrays() {
    let cases: [(&[f64], bool); 11] = [
        (&[12.0, 34.0], false),
        (&[12.0, 34.0, 56.0, 1.0], false),
        (&[-12.0, 34.0, 56.0], false),
        (&[12.0, -1.0, 0.0], false),
        (&[-0.0, 30.0, 0.0], false),
        (&[12.5, 0.0, 0.0], false),
        (&[12.0, 60.0, 0.0], false),
        (&[12.0, 0.0, 60.0], false),
        (&[f64::NAN, 0.0, 0.0], false),
        (&[12.0, 30.5, 0.0], false), // fractional minutes: [12, 30, 30] is the canonical spelling
        (&[12.0, 59.0, f64::INFINITY], false),
    ];
    for (dms, negative) in cases {
        assert!(
            matches!(
                dms_to_radians(dms, negative),
                Err(UnitError::InvalidValue(_))
            ),
            "{dms:?}"
        );
    }
}

#[test]
fn bad_bearings() {
    let bad = [
        "",
        "N",
        "N E",
        "3°45' E",
        "N 3°45'",
        "E 3°45' N",
        "n 3°45' e",
        "N 91° E",
        "N 90°0'1\" E",
        "N 3°60' E",
        "N 3°45'60\" E",
        "N 3.5° E",
        "N 3°45.5' E",
        "N 45'3° E",
        "N 3°10\" E",
        "N 3 E",
        "N °45' E",
        "N 3°45'x E",
        "N -3° E",
        "N 90°0'0.001\" E",
        "N 90°01' E",
        "N 95° E",
        "N 3°45'59.9999\"\" E",
        "s 3° e",
        "N 3° e",
        "Ñ 3° E",
        "NE 3° E",
        "N 3° EW",
        "N 4 5° E",
        "N 3°4 5' E",
        "N 3°45'1 2.5\" E",
        "N 3°45'12 .5\" E",
    ];
    for text in bad {
        assert!(
            matches!(bearing_to_azimuth(text), Err(UnitError::InvalidValue(_))),
            "'{text}' should be rejected"
        );
    }
}
