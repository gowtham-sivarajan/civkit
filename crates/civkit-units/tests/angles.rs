// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 15:35 IST · last edited 2026-11-25 15:56 IST

//! DMS and bearing conversions against values computed by hand.

use civkit_units::{bearing_to_azimuth, dms_to_radians};

fn deg(rad: f64) -> f64 {
    rad.to_degrees()
}

fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-9, "{a} vs {b}");
}

#[test]
fn dms_examples() {
    close(
        deg(dms_to_radians(&[12.0, 34.0, 56.7], false).expect("ok")),
        12.0 + 34.0 / 60.0 + 56.7 / 3600.0,
    );
    close(
        deg(dms_to_radians(&[0.0, 30.0, 0.0], true).expect("ok")),
        -0.5,
    );
    close(
        deg(dms_to_radians(&[359.0, 59.0, 59.999], false).expect("ok")),
        359.0 + 59.0 / 60.0 + 59.999 / 3600.0,
    );
    close(dms_to_radians(&[0.0, 0.0, 0.0], true).expect("ok"), 0.0);
}

#[test]
fn bearings_in_all_four_quadrants() {
    // Bearings of traverse C3 (NAVEDTRA 14070, Fig 7-12), converted by hand to azimuths.
    close(deg(bearing_to_azimuth("N 3°45' E").expect("ok")), 3.75);
    close(
        deg(bearing_to_azimuth("S 86°15' E").expect("ok")),
        180.0 - 86.25,
    );
    close(
        deg(bearing_to_azimuth("S 10°49' E").expect("ok")),
        180.0 - (10.0 + 49.0 / 60.0),
    );
    close(
        deg(bearing_to_azimuth("S 32°40' W").expect("ok")),
        180.0 + 32.0 + 40.0 / 60.0,
    );
    close(
        deg(bearing_to_azimuth("N 43°29' W").expect("ok")),
        360.0 - (43.0 + 29.0 / 60.0),
    );
    close(
        deg(bearing_to_azimuth("S 57°31' W").expect("ok")),
        180.0 + 57.0 + 31.0 / 60.0,
    );
}

#[test]
fn bearing_spellings_are_equivalent() {
    let reference = bearing_to_azimuth("N 3°45'00\" E").expect("ok");
    for spelling in [
        "N3°45'00\"E",
        "N 3d45m00s E",
        "N3d45m0sE",
        "N 3°45′00″ E",
        "N 3°45' E",
        "N 3d45m E",
    ] {
        close(bearing_to_azimuth(spelling).expect(spelling), reference);
    }
}

#[test]
fn cardinal_boundaries_every_valid_spelling() {
    // Each cardinal direction has exactly two valid spellings; all must agree.
    let cases = [
        ("N 0° E", 0.0),
        ("N 0° W", 0.0), // 360° wraps to 0
        ("N 90° E", 90.0),
        ("S 90° E", 90.0),
        ("S 0° E", 180.0),
        ("S 0° W", 180.0),
        ("S 90° W", 270.0),
        ("N 90° W", 270.0),
        ("N 90°00'00\" E", 90.0),
    ];
    for (text, expected) in cases {
        close(deg(bearing_to_azimuth(text).expect(text)), expected);
    }
}

#[test]
fn limits_just_inside_are_accepted() {
    let just_under_90 = 89.0 + 59.0 / 60.0 + 59.999 / 3600.0;
    close(
        deg(bearing_to_azimuth("N 89°59'59.999\" E").expect("ok")),
        just_under_90,
    );
    close(
        deg(bearing_to_azimuth("S 0°59'59.999\" W").expect("ok")),
        180.0 + 59.0 / 60.0 + 59.999 / 3600.0,
    );
    close(
        deg(bearing_to_azimuth("N 0°00'00.001\" W").expect("ok")),
        360.0 - 0.001 / 3600.0,
    );
}

#[test]
fn dms_negative_zero_is_zero() {
    let z = dms_to_radians(&[0.0, 0.0, 0.0], true).expect("ok");
    assert!(z == 0.0 && z.is_sign_positive());
}
