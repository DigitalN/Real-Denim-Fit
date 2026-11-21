//! Units and the flat/around distinction.
//!
//! Inside the core every width (waist, seat, thigh, knee, leg opening) is held
//! **flat** in centimetres: half the way around. Lengths (rises, inseam) are
//! plain centimetres. Brands publish in inches or cm, flat or around, and
//! everything is converted on the way in.

use serde::{Deserialize, Serialize};

/// Centimetres in one inch.
pub const CM_PER_INCH: f64 = 2.54;
/// A quarter inch, in cm: the size of a nudge and the half-width of a
/// reference-pair target.
pub const QUARTER_INCH: f64 = CM_PER_INCH / 4.0;
/// An eighth of an inch, in cm: the step closeness loses 10% over beyond a
/// target's edge.
pub const EIGHTH_INCH: f64 = CM_PER_INCH / 8.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Unit {
    Cm,
    In,
}

impl Unit {
    /// Converts a value in this unit to centimetres.
    pub fn to_cm(self, value: f64) -> f64 {
        match self {
            Unit::Cm => value,
            Unit::In => value * CM_PER_INCH,
        }
    }
}

/// How a number was measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Way {
    /// Straight across a garment laid flat: half the way around.
    Flat,
    /// All the way around.
    Around,
    /// A length along the garment, such as a rise or the inseam.
    Length,
}

impl Way {
    /// Converts a value measured this way into the core's form: flat for
    /// widths, unchanged for lengths.
    pub fn to_core(self, value: f64) -> f64 {
        match self {
            Way::Around => value / 2.0,
            Way::Flat | Way::Length => value,
        }
    }
}

/// A value with its unit, as written in the data files.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Quantity {
    pub value: f64,
    pub unit: Unit,
}

impl Quantity {
    pub fn cm(self) -> f64 {
        self.unit.to_cm(self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_waist_published_around_in_inches_is_held_flat_in_cm() {
        // Naked & Famous publish a 34" circumference waist: 17" flat, 43.18 cm.
        let core = Way::Around.to_core(Unit::In.to_cm(34.0));
        assert!((core - 43.18).abs() < 1e-9);
    }

    #[test]
    fn flat_and_length_values_keep_their_size() {
        assert_eq!(Way::Flat.to_core(Unit::Cm.to_cm(22.0)), 22.0);
        assert!((Way::Length.to_core(Unit::In.to_cm(12.0)) - 30.48).abs() < 1e-9);
    }
}
