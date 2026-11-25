// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 15:34 IST · last edited 2026-11-25 15:58 IST

//! Structured angle values: degrees–minutes–seconds and quadrant bearings (`docs/API_DESIGN.md` §7).
//! Both convert to radians; a bearing converts to an azimuth (clockwise from north, 0 ≤ azimuth < 2π).

use crate::UnitError;
use std::f64::consts::PI;

/// Converts a DMS angle `[degrees, minutes, seconds]` to radians.
///
/// Canonical form (`docs/API_DESIGN.md` §7.1), so every angle has exactly one spelling:
/// - degrees: a whole number ≥ 0 (no upper limit; angles are not normalised, domains apply);
/// - minutes: a whole number, 0 ≤ m ≤ 59;
/// - seconds: a real number, 0 ≤ s < 60;
/// - the sign is carried only by `negative`, never inside the array. `negative` with a zero angle gives zero.
pub fn dms_to_radians(dms: &[f64], negative: bool) -> Result<f64, UnitError> {
    let invalid = |why: &str| Err(UnitError::InvalidValue(format!("DMS angle {dms:?}: {why}")));
    let [d, m, s] = dms else {
        return invalid("must be an array of exactly three numbers [degrees, minutes, seconds]");
    };
    if !(d.is_finite() && m.is_finite() && s.is_finite()) {
        return invalid("values must be finite numbers");
    }
    if d.is_sign_negative() || m.is_sign_negative() || s.is_sign_negative() {
        return invalid("signs are not allowed inside the array; use \"negative\": true");
    }
    if d.fract() != 0.0 || m.fract() != 0.0 {
        return invalid("degrees and minutes must be whole numbers (canonical DMS)");
    }
    if *m >= 60.0 || *s >= 60.0 {
        return invalid("minutes and seconds must be less than 60");
    }
    let degrees = d + m / 60.0 + s / 3600.0;
    let radians = degrees.to_radians();
    Ok(if negative && radians != 0.0 {
        -radians
    } else {
        radians
    })
}

/// Converts a quadrant bearing such as `N 3°45'00" E` or `S 86d15m E` to an azimuth in radians.
///
/// Grammar and limits: `docs/API_DESIGN.md` §7.1. In short: `N` or `S`, an angle, then `E` or `W`
/// (upper case only; lower case is rejected, not normalised). Degrees 0–90 and minutes 0–59 are whole
/// numbers; seconds are 0 ≤ s < 60; exactly 90° must have zero minutes and seconds. Spaces between parts
/// are allowed.
pub fn bearing_to_azimuth(text: &str) -> Result<f64, UnitError> {
    let invalid = |why: &str| {
        Err(UnitError::InvalidValue(format!(
            "bearing '{text}': {why} (expected e.g. N 3°45'00\" E or N 3d45m00s E)"
        )))
    };
    // Spaces may separate tokens but never split a number ("N 4 5° E" is not 45°).
    let chars: Vec<char> = text.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        if c.is_whitespace() {
            let before = chars[..i].iter().rev().find(|c| !c.is_whitespace());
            let after = chars[i..].iter().find(|c| !c.is_whitespace());
            let numeric = |c: Option<&char>| c.is_some_and(|c| c.is_ascii_digit() || *c == '.');
            if numeric(before) && numeric(after) {
                return invalid("spaces may not split a number");
            }
        }
    }
    let compact: String = chars.iter().filter(|c| !c.is_whitespace()).collect();
    let mut chars = compact.chars();
    let (Some(ns), Some(ew)) = (chars.next(), chars.next_back()) else {
        return invalid("too short");
    };
    if !matches!(ns, 'N' | 'S') || !matches!(ew, 'E' | 'W') {
        return invalid("must start with N or S and end with E or W");
    }
    let body: String = chars.collect();

    // Split the angle into (number, unit-symbol) pairs.
    let mut parts: Vec<(String, char)> = Vec::new();
    let mut number = String::new();
    for c in body.chars() {
        if c.is_ascii_digit() || c == '.' {
            number.push(c);
        } else {
            let symbol = match c {
                '°' | 'd' => 'd',
                '\'' | '′' | 'm' => 'm',
                '"' | '″' | 's' => 's',
                _ => return invalid(&format!("unexpected character '{c}'")),
            };
            if number.is_empty() {
                return invalid("a symbol without a number");
            }
            parts.push((std::mem::take(&mut number), symbol));
        }
    }
    if !number.is_empty() {
        return invalid("a number without a unit symbol");
    }
    let symbols: String = parts.iter().map(|(_, s)| *s).collect();
    if !matches!(symbols.as_str(), "d" | "dm" | "dms") {
        return invalid("parts must be degrees, then minutes, then seconds");
    }
    let mut values = [0.0f64; 3];
    for (i, (n, _)) in parts.iter().enumerate() {
        if i < 2 && n.contains('.') {
            return invalid("degrees and minutes must be whole numbers");
        }
        match n.parse::<f64>() {
            Ok(v) if v.is_finite() => values[i] = v,
            _ => return invalid(&format!("'{n}' is not a number")),
        }
    }
    let [d, m, s] = values;
    if m >= 60.0 || s >= 60.0 {
        return invalid("minutes and seconds must be less than 60");
    }
    let angle = d + m / 60.0 + s / 3600.0;
    if angle > 90.0 {
        return invalid("a quadrant bearing cannot exceed 90°");
    }
    let azimuth_deg = match (ns, ew) {
        ('N', 'E') => angle,
        ('S', 'E') => 180.0 - angle,
        ('S', 'W') => 180.0 + angle,
        _ => 360.0 - angle, // N…W
    };
    let azimuth = azimuth_deg.to_radians();
    Ok(if azimuth >= 2.0 * PI {
        azimuth - 2.0 * PI
    } else {
        azimuth
    })
}
