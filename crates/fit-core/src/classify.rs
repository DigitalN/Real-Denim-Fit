//! Leg shape and rise, worked out from each size's own numbers (ADR 0008).

use std::collections::BTreeMap;

use crate::catalogue::Measurement;
use crate::profile::{LegShape, Rise};
use crate::units::CM_PER_INCH;

/// Knee minus leg opening, flat: tapered is 3 cm or more, straight 1–3 cm,
/// wide under 1 cm.
pub fn leg_shape(numbers: &BTreeMap<Measurement, f64>) -> Option<LegShape> {
    let taper = numbers.get(&Measurement::Knee)? - numbers.get(&Measurement::LegOpening)?;
    Some(if taper >= 3.0 - 1e-9 {
        LegShape::Tapered
    } else if taper >= 1.0 - 1e-9 {
        LegShape::Straight
    } else {
        LegShape::Wide
    })
}

/// One size, for working out rises across the whole catalogue.
#[derive(Debug, Clone, Copy)]
pub struct RiseEntry {
    /// Which product the size belongs to: each product counts once.
    pub product: usize,
    /// After-wash waist, flat cm.
    pub waist: Option<f64>,
    /// After-wash front rise, cm.
    pub front_rise: Option<f64>,
}

/// How far apart two waists can be and still count as "the same waist":
/// half an inch flat, one inch around.
const SAME_WAIST: f64 = CM_PER_INCH / 2.0;

/// The rise of every entry: low, mid or high by which third its front rise
/// falls in among every product at the same waist. `None` without a front
/// rise, or with fewer than three products to compare.
pub fn rises(entries: &[RiseEntry]) -> Vec<Option<Rise>> {
    entries
        .iter()
        .map(|me| {
            let (waist, rise) = (me.waist?, me.front_rise?);
            // For each product, its size with the nearest waist.
            let mut nearest: BTreeMap<usize, (f64, f64)> = BTreeMap::new();
            for e in entries {
                let (Some(w), Some(r)) = (e.waist, e.front_rise) else {
                    continue;
                };
                let gap = (w - waist).abs();
                if gap > SAME_WAIST + 1e-9 {
                    continue;
                }
                let pick = if e.product == me.product {
                    (-1.0, rise)
                } else {
                    (gap, r)
                };
                nearest
                    .entry(e.product)
                    .and_modify(|cur| {
                        if pick.0 < cur.0 {
                            *cur = pick
                        }
                    })
                    .or_insert(pick);
            }
            let n = nearest.len();
            if n < 3 {
                return None;
            }
            let below = nearest.values().filter(|(_, r)| *r < rise - 1e-9).count() as f64;
            let equal = nearest
                .values()
                .filter(|(_, r)| (*r - rise).abs() <= 1e-9)
                .count() as f64;
            // Average rank among ties, 0-based, as a share of the products.
            let rank = below + (equal - 1.0) / 2.0;
            let third = (3.0 * (rank + 0.5) / n as f64).floor() as usize;
            Some(match third {
                0 => Rise::Low,
                1 => Rise::Mid,
                _ => Rise::High,
            })
        })
        .collect()
}
