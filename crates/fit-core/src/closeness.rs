//! How close one measurement comes to its target (ADR 0009).

use crate::targets::Target;
use crate::units::EIGHTH_INCH;

/// 100% at the middle of the target, 80% at its edges, then 10% less for
/// every eighth of an inch beyond an edge, down to 0%.
pub fn closeness(value: f64, target: &Target) -> f64 {
    let beyond = |by: f64| (80.0 - 10.0 * by / EIGHTH_INCH).max(0.0);
    match *target {
        Target::AtLeast { min } => {
            if value >= min - 1e-9 {
                100.0
            } else {
                beyond(min - value)
            }
        }
        Target::Range { low, middle, high } => {
            if value < low - 1e-9 {
                beyond(low - value)
            } else if value > high + 1e-9 {
                beyond(value - high)
            } else {
                let half = if value >= middle {
                    high - middle
                } else {
                    middle - low
                };
                if half <= 0.0 {
                    100.0
                } else {
                    100.0 - 20.0 * (value - middle).abs() / half
                }
            }
        }
    }
}
