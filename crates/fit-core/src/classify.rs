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
///
/// Entries are sorted by waist once, and each one looks only at the window
/// of waists within half an inch of its own, so the whole catalogue takes
/// milliseconds rather than comparing every size with every other.
pub fn rises(entries: &[RiseEntry]) -> Vec<Option<Rise>> {
    // Entries with both numbers, sorted by waist: (waist, rise, product, index).
    let mut sorted: Vec<(f64, f64, usize, usize)> = entries
        .iter()
        .enumerate()
        .filter_map(|(i, e)| Some((e.waist?, e.front_rise?, e.product, i)))
        .collect();
    sorted.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.3.cmp(&b.3)));

    let mut out = vec![None; entries.len()];
    // Per product, the nearest size seen in the current window, stamped
    // with the entry it was found for, so nothing needs clearing.
    let products = entries.iter().map(|e| e.product + 1).max().unwrap_or(0);
    let mut best: Vec<(usize, f64, usize, f64)> = vec![(usize::MAX, 0.0, 0, 0.0); products];
    let mut seen: Vec<usize> = vec![];
    for (stamp, &(waist, rise, product, index)) in sorted.iter().enumerate() {
        let lo = sorted.partition_point(|e| e.0 < waist - SAME_WAIST - 1e-9);
        let hi = sorted.partition_point(|e| e.0 <= waist + SAME_WAIST + 1e-9);
        // For each product, its size with the nearest waist (the earliest
        // entry on a tie). This entry stands for its own product.
        seen.clear();
        for &(w, r, p, i) in &sorted[lo..hi] {
            let pick = if p == product {
                (-1.0, 0, rise)
            } else {
                ((w - waist).abs(), i, r)
            };
            let cur = &mut best[p];
            if cur.0 != stamp {
                *cur = (stamp, pick.0, pick.1, pick.2);
                seen.push(p);
            } else if (pick.0, pick.1) < (cur.1, cur.2) {
                *cur = (stamp, pick.0, pick.1, pick.2);
            }
        }
        let n = seen.len();
        if n < 3 {
            continue;
        }
        let below = seen.iter().filter(|p| best[**p].3 < rise - 1e-9).count() as f64;
        let equal = seen.iter().filter(|p| (best[**p].3 - rise).abs() <= 1e-9).count() as f64;
        // Average rank among ties, 0-based, as a share of the products.
        let rank = below + (equal - 1.0) / 2.0;
        out[index] = Some(match (3.0 * (rank + 0.5) / n as f64).floor() as usize {
            0 => Rise::Low,
            1 => Rise::Mid,
            _ => Rise::High,
        });
    }
    out
}
