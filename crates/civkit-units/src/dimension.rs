// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 10:57 IST · last edited 2026-11-25 15:33 IST

//! Physical dimensions as integer exponents of the base dimensions, with angle as its own dimension (D-5).

use serde::{Deserialize, Serialize};

/// The physical dimension of a quantity, as integer exponents of the base dimensions.
///
/// Angle is its own dimension (D-5), so an angle can never be passed where a plain ratio is expected.
/// A dimensionless quantity has every exponent zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Dimension {
    /// Length (L), base unit metre.
    pub length: i8,
    /// Mass (M), base unit kilogram.
    pub mass: i8,
    /// Time (T), base unit second.
    pub time: i8,
    /// Thermodynamic temperature (Θ), base unit kelvin. Affine temperatures are out of scope for 0.1.
    pub temperature: i8,
    /// Amount of substance (N), base unit mole.
    pub amount: i8,
    /// Electric current (I), base unit ampere.
    pub current: i8,
    /// Luminous intensity (J), base unit candela.
    pub luminous_intensity: i8,
    /// Plane angle, base unit radian. Kept separate from dimensionless on purpose.
    pub angle: i8,
}

impl Dimension {
    /// A plain ratio (every exponent zero).
    pub const DIMENSIONLESS: Self = Self::new(0, 0, 0, 0);
    /// Length, L.
    pub const LENGTH: Self = Self::new(1, 0, 0, 0);
    /// Area, L².
    pub const AREA: Self = Self::new(2, 0, 0, 0);
    /// Volume, L³.
    pub const VOLUME: Self = Self::new(3, 0, 0, 0);
    /// Plane angle.
    pub const ANGLE: Self = Self {
        angle: 1,
        ..Self::DIMENSIONLESS
    };

    /// A dimension from the exponents of length, mass, time and temperature (the others zero).
    pub const fn new(length: i8, mass: i8, time: i8, temperature: i8) -> Self {
        Self {
            length,
            mass,
            time,
            temperature,
            amount: 0,
            current: 0,
            luminous_intensity: 0,
            angle: 0,
        }
    }

    /// `true` when every exponent is zero.
    pub fn is_dimensionless(&self) -> bool {
        *self == Self::DIMENSIONLESS
    }

    fn exponents(&self) -> [i8; 8] {
        [
            self.length,
            self.mass,
            self.time,
            self.temperature,
            self.amount,
            self.current,
            self.luminous_intensity,
            self.angle,
        ]
    }

    fn from_exponents(e: [i8; 8]) -> Self {
        Self {
            length: e[0],
            mass: e[1],
            time: e[2],
            temperature: e[3],
            amount: e[4],
            current: e[5],
            luminous_intensity: e[6],
            angle: e[7],
        }
    }

    /// The dimension of a product, or `None` if an exponent would overflow.
    pub fn checked_mul(self, other: Self) -> Option<Self> {
        let (a, b) = (self.exponents(), other.exponents());
        let mut out = [0i8; 8];
        for i in 0..8 {
            out[i] = a[i].checked_add(b[i])?;
        }
        Some(Self::from_exponents(out))
    }

    /// The dimension of a quotient, or `None` if an exponent would overflow.
    pub fn checked_div(self, other: Self) -> Option<Self> {
        self.checked_mul(other.checked_powi(-1)?)
    }

    /// The dimension raised to an integer power, or `None` if an exponent would overflow.
    pub fn checked_powi(self, power: i8) -> Option<Self> {
        let a = self.exponents();
        let mut out = [0i8; 8];
        for i in 0..8 {
            out[i] = a[i].checked_mul(power)?;
        }
        Some(Self::from_exponents(out))
    }
}

impl std::fmt::Display for Dimension {
    /// Writes the dimension in the form used by error messages, e.g. `L^2`, `L*T^-1`, `1`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let parts = [
            ("L", self.length),
            ("M", self.mass),
            ("T", self.time),
            ("Θ", self.temperature),
            ("N", self.amount),
            ("I", self.current),
            ("J", self.luminous_intensity),
            ("angle", self.angle),
        ];
        let mut first = true;
        for (symbol, exponent) in parts {
            if exponent == 0 {
                continue;
            }
            if !first {
                f.write_str("*")?;
            }
            first = false;
            if exponent == 1 {
                f.write_str(symbol)?;
            } else {
                write!(f, "{symbol}^{exponent}")?;
            }
        }
        if first {
            f.write_str("1")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimensionless_is_default_and_displays_as_one() {
        assert_eq!(Dimension::default(), Dimension::DIMENSIONLESS);
        assert!(Dimension::DIMENSIONLESS.is_dimensionless());
        assert_eq!(Dimension::DIMENSIONLESS.to_string(), "1");
    }

    #[test]
    fn angle_is_not_dimensionless() {
        assert!(!Dimension::ANGLE.is_dimensionless());
        assert_eq!(Dimension::ANGLE.to_string(), "angle");
    }

    #[test]
    fn display_of_compound_dimensions() {
        assert_eq!(Dimension::AREA.to_string(), "L^2");
        assert_eq!(Dimension::new(1, 0, -1, 0).to_string(), "L*T^-1");
        assert_eq!(Dimension::new(-1, 1, -2, 0).to_string(), "L^-1*M*T^-2");
    }

    #[test]
    fn missing_exponents_deserialize_as_zero() {
        let d: Dimension = serde_json::from_str(r#"{ "length": 2 }"#).expect("valid JSON");
        assert_eq!(d, Dimension::AREA);
    }
}
