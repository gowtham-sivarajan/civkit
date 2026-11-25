// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 15:34 IST · last edited 2026-11-25 15:34 IST

//! Unit strings: the grammar and the parser (`docs/API_DESIGN.md` §7, implementation notes in §7.1).
//!
//! ```text
//! unit        = "%" | numerator [ "/" term ]
//! numerator   = term { ("*" | "·") term }
//! term        = name [ "^" exponent ]
//! exponent    = [ "-" ] digit          (1–9; zero is rejected)
//! name        = a named unit from the table, case-sensitive
//! ```
//!
//! No whitespace, parentheses, numeric factors, prefixes or superscript characters.
//! At most one `/`, followed by a single term, so `W/m^2*K` (ambiguous) is rejected: write `W*m^-2*K^-1`.

use crate::table::{AMBIGUOUS, NAMED, named};
use crate::{Dimension, UnitError};

/// A parsed unit: its factor to SI and its dimension. Affine units (°C, °F) are not representable on purpose.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Unit {
    /// Multiply a value in this unit by `factor` to get SI.
    pub factor: f64,
    /// The unit's dimension.
    pub dimension: Dimension,
}

impl Unit {
    /// Parses a unit string.
    pub fn parse(text: &str) -> Result<Self, UnitError> {
        if text.is_empty() {
            return Err(UnitError::Empty);
        }
        if text.chars().any(char::is_whitespace) {
            return Err(UnitError::Syntax(format!(
                "unit '{text}' contains whitespace; write it without spaces"
            )));
        }
        if text == "%" {
            return Ok(Self {
                factor: 0.01,
                dimension: Dimension::DIMENSIONLESS,
            });
        }
        let (numerator, denominator) = match text.split_once('/') {
            None => (text, None),
            Some((n, d)) => (n, Some(d)),
        };
        if let Some(d) = denominator
            && (d.contains('/') || d.contains('*') || d.contains('·'))
        {
            return Err(UnitError::Syntax(format!(
                "unit '{text}': only one term may follow '/'; for more, use negative powers (e.g. W*m^-2*K^-1)"
            )));
        }

        let mut unit = Self {
            factor: 1.0,
            dimension: Dimension::DIMENSIONLESS,
        };
        for term in numerator.split(['*', '·']) {
            unit = unit.times(parse_term(term, text)?, text)?;
        }
        if let Some(d) = denominator {
            let t = parse_term(d, text)?;
            unit = unit.times(
                Self {
                    factor: 1.0 / t.factor,
                    dimension: inverse(t.dimension, text)?,
                },
                text,
            )?;
        }
        Ok(unit)
    }

    /// Converts a value in this unit to SI.
    pub fn to_si(&self, value: f64) -> f64 {
        value * self.factor
    }

    /// Converts an SI value to this unit.
    pub fn from_si(&self, value_si: f64) -> f64 {
        value_si / self.factor
    }

    /// Checks the unit has the required dimension.
    pub fn expect(self, dimension: Dimension) -> Result<Self, UnitError> {
        if self.dimension == dimension {
            Ok(self)
        } else {
            Err(UnitError::Mismatch {
                expected: dimension,
                found: self.dimension,
            })
        }
    }

    fn times(self, other: Self, text: &str) -> Result<Self, UnitError> {
        let dimension = self
            .dimension
            .checked_mul(other.dimension)
            .ok_or_else(|| overflow(text))?;
        Ok(Self {
            factor: self.factor * other.factor,
            dimension,
        })
    }
}

fn overflow(text: &str) -> UnitError {
    UnitError::Syntax(format!("unit '{text}': exponents are too large"))
}

fn inverse(d: Dimension, text: &str) -> Result<Dimension, UnitError> {
    d.checked_powi(-1).ok_or_else(|| overflow(text))
}

/// Parses `name` or `name^exponent`.
fn parse_term(term: &str, text: &str) -> Result<Unit, UnitError> {
    if term.is_empty() {
        return Err(UnitError::Syntax(format!(
            "unit '{text}' has an empty term (check '*', '/' and '^')"
        )));
    }
    let (name, exponent) = match term.split_once('^') {
        None => (term, 1i8),
        Some((n, e)) => (n, parse_exponent(e, text)?),
    };
    if name == "%" {
        return Err(UnitError::Syntax(format!(
            "unit '{text}': '%' cannot be combined with other units"
        )));
    }
    let (factor, dimension) = lookup(name)?;
    let dimension = dimension
        .checked_powi(exponent)
        .ok_or_else(|| overflow(text))?;
    Ok(Unit {
        factor: factor.powi(i32::from(exponent)),
        dimension,
    })
}

fn parse_exponent(e: &str, text: &str) -> Result<i8, UnitError> {
    let digits = e.strip_prefix('-').unwrap_or(e);
    let valid = digits.len() == 1 && digits.chars().all(|c| matches!(c, '1'..='9'));
    if !valid {
        return Err(UnitError::Syntax(format!(
            "unit '{text}': exponent '{e}' must be a whole number from -9 to 9, not zero"
        )));
    }
    let magnitude = i8::try_from(digits.as_bytes()[0] - b'0').map_err(|_| overflow(text))?;
    Ok(if e.starts_with('-') {
        -magnitude
    } else {
        magnitude
    })
}

fn lookup(name: &str) -> Result<(f64, Dimension), UnitError> {
    if let Some(found) = named(name) {
        return Ok(found);
    }
    if let Some((_, instead)) = AMBIGUOUS.iter().find(|(n, _)| *n == name) {
        return Err(UnitError::Ambiguous {
            name: name.to_owned(),
            instead: (*instead).to_owned(),
        });
    }
    Err(UnitError::Unknown {
        name: name.to_owned(),
        hint: hint(name),
    })
}

/// A suggestion for an unknown name: a case-insensitive match, or a note about superscripts and prefixes.
fn hint(name: &str) -> Option<String> {
    if name.chars().any(|c| "²³¹⁰⁴⁵⁶⁷⁸⁹⁻".contains(c)) {
        return Some("write powers with '^', e.g. m^2".into());
    }
    if name.chars().any(|c| matches!(c, '(' | ')')) {
        return Some("parentheses are not supported; use negative powers, e.g. W*m^-2*K^-1".into());
    }
    if name
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_digit() || c == '.')
    {
        return Some("numeric factors are not supported in unit strings".into());
    }
    NAMED
        .iter()
        .find(|(n, _, _)| n.eq_ignore_ascii_case(name))
        .map(|(n, _, _)| format!("did you mean '{n}'? (unit names are case-sensitive)"))
        .or_else(|| Some("see the named units in docs/API_DESIGN.md §7".into()))
}
