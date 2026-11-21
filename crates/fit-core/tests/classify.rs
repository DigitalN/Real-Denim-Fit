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
