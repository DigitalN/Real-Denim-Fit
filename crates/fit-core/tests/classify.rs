//! Leg shape and rise come from the numbers (ADR 0008).
mod common;

use std::collections::BTreeMap;

use common::IN;
use fit_core::catalogue::Measurement::{self, *};
use fit_core::classify::{leg_shape, rises, RiseEntry};
use fit_core::profile::{LegShape, Rise};

fn flat(knee: f64, opening: f64) -> BTreeMap<Measurement, f64> {
    BTreeMap::from([(Knee, knee), (LegOpening, opening)])
}

#[test]
fn leg_shape_comes_from_knee_minus_leg_opening_flat() {
    assert_eq!(leg_shape(&flat(23.0, 20.0)), Some(LegShape::Tapered));
    assert_eq!(leg_shape(&flat(23.0, 19.0)), Some(LegShape::Tapered));
    assert_eq!(leg_shape(&flat(23.0, 21.0)), Some(LegShape::Straight));
    assert_eq!(leg_shape(&flat(23.0, 22.0)), Some(LegShape::Straight));
    assert_eq!(leg_shape(&flat(23.0, 22.5)), Some(LegShape::Wide));
    assert_eq!(leg_shape(&flat(23.0, 24.0)), Some(LegShape::Wide));
}

#[test]
fn the_weird_guy_is_tapered_and_the_634s_and_cs105x_are_straight() {
    // Size 32 knee and leg opening, inches flat, from docs/research.
    assert_eq!(leg_shape(&flat(8.75 * IN, 7.5 * IN)), Some(LegShape::Tapered));
    assert_eq!(leg_shape(&flat(9.3 * IN, 8.7 * IN)), Some(LegShape::Straight));
    assert_eq!(leg_shape(&flat(9.5 * IN, 8.5 * IN)), Some(LegShape::Straight));
}

#[test]
fn without_a_knee_or_leg_opening_the_leg_shape_is_unknown() {
    assert_eq!(leg_shape(&BTreeMap::from([(LegOpening, 20.0)])), None);
}

fn entry(product: usize, waist: f64, front_rise: Option<f64>) -> RiseEntry {
    RiseEntry {
        product,
        waist: Some(waist),
        front_rise,
    }
}

#[test]
fn rise_is_the_third_a_front_rise_falls_in_among_products_at_the_same_waist() {
    let entries = vec![
        entry(0, 41.0, Some(27.0)),
        entry(1, 41.3, Some(29.0)),
        entry(2, 40.8, Some(31.0)),
        // A much bigger waist with a tiny rise doesn't count against the rest.
        entry(3, 52.0, Some(20.0)),
    ];
    let r = rises(&entries);
    assert_eq!(&r[..3], &[Some(Rise::Low), Some(Rise::Mid), Some(Rise::High)]);
}

#[test]
fn each_product_counts_once_at_a_waist() {
    // Product 0 is sold in three lengths with the same rise; it mustn't fill
    // the bottom two thirds by itself.
    let entries = vec![
        entry(0, 41.0, Some(27.0)),
        entry(0, 41.0, Some(27.0)),
        entry(0, 41.0, Some(27.0)),
        entry(1, 41.0, Some(29.0)),
        entry(2, 41.0, Some(31.0)),
    ];
    let r = rises(&entries);
    assert_eq!(r[0], Some(Rise::Low));
    assert_eq!(r[3], Some(Rise::Mid));
    assert_eq!(r[4], Some(Rise::High));
}

#[test]
fn a_size_without_a_front_rise_or_too_few_neighbours_has_no_rise() {
    let r = rises(&[
        entry(0, 41.0, None),
        entry(1, 41.0, Some(28.0)),
        entry(2, 41.0, Some(29.0)),
    ]);
    assert_eq!(r[0], None);
    assert_eq!(r[1], None, "only two products to compare at this waist");
}

/// The original pairwise classification, kept as the reference the faster
/// one must agree with exactly.
fn rises_pairwise(entries: &[RiseEntry]) -> Vec<Option<Rise>> {
    const SAME_WAIST: f64 = 2.54 / 2.0;
    entries
        .iter()
        .map(|me| {
            let (waist, rise) = (me.waist?, me.front_rise?);
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
            let equal = nearest.values().filter(|(_, r)| (*r - rise).abs() <= 1e-9).count() as f64;
            let rank = below + (equal - 1.0) / 2.0;
            Some(match (3.0 * (rank + 0.5) / n as f64).floor() as usize {
                0 => Rise::Low,
                1 => Rise::Mid,
                _ => Rise::High,
            })
        })
        .collect()
}

#[test]
fn rises_match_the_pairwise_reference_on_a_large_catalogue() {
    // 300 products of 8–14 sizes, with waists on a 0.25 cm grid so there
    // are plenty of exact ties and equal gaps, and some missing numbers.
    let mut seed: u64 = 20261121;
    let mut next = |n: u64| {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (seed >> 33) % n
    };
    let mut entries = vec![];
    for product in 0..300 {
        let start = 36.0 + next(16) as f64 * 0.25;
        let rise = 25.0 + next(20) as f64 * 0.25;
        for s in 0..(8 + next(7)) {
            let waist = start + s as f64 * 1.25 + next(3) as f64 * 0.25;
            entries.push(RiseEntry {
                product,
                waist: if next(40) == 0 { None } else { Some(waist) },
                front_rise: if next(30) == 0 {
                    None
                } else {
                    Some(rise + s as f64 * 0.3)
                },
            });
        }
    }
    assert!(entries.len() > 3000);
    assert_eq!(rises(&entries), rises_pairwise(&entries));
}

#[test]
fn the_whole_catalogue_classifies_quickly() {
    // Shaped like the real catalogue but bigger: 1,000 products of 13 sizes
    // a size apart (1.27 cm flat), each product's waists offset a little.
    // Comparing every size with every other took 9 s in a debug build for
    // the real 12,500 sizes; this must take well under a second unoptimised.
    let entries: Vec<RiseEntry> = (0..13_000)
        .map(|i| RiseEntry {
            product: i / 13,
            waist: Some(36.0 + (i % 13) as f64 * 1.27 + (i / 13 % 25) as f64 * 0.1),
            front_rise: Some(25.0 + (i / 13 % 17) as f64 * 0.3 + (i % 13) as f64 * 0.2),
        })
        .collect();
    let start = std::time::Instant::now();
    let r = rises(&entries);
    let took = start.elapsed();
    assert!(r.iter().filter(|x| x.is_some()).count() > 12_000);
    // A loose wall-clock limit: CI runs this unoptimised on shared runners,
    // beside the pairwise reference test, and took 1.2 s there once. 5 s
    // still catches a return to comparing every pair (12 s on this data, 9 s
    // on the real catalogue); rises_match_the_pairwise_reference_on_a_large_catalogue
    // proves the results are right.
    assert!(took < std::time::Duration::from_secs(5), "took {took:?}");
}
