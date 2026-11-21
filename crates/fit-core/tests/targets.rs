//! Targets at each measuring point (ADR 0005, ADR 0006).
mod common;

use common::*;
use fit_core::catalogue::{KneePoint, Measurement::*, ThighPoint};
use fit_core::profile::*;
use fit_core::targets::{targets, Points, Target, TargetFrom};
use fit_core::units::QUARTER_INCH;

fn ih_points() -> Points {
    Points {
        thigh: ThighPoint::Crotch,
        knee: KneePoint::Below13,
        approximate: false,
    }
}

fn range_of(t: &Target) -> (f64, f64, f64) {
    match *t {
        Target::Range { low, middle, high } => (low, middle, high),
        Target::AtLeast { .. } => panic!("expected a range"),
    }
}

#[test]
fn without_a_pair_the_seat_target_is_the_body_plus_the_rooms_ease() {
    let t = targets(&profile(Room::Regular), None, &ease(), ih_points(), false);
    let (low, middle, high) = range_of(&t.targets[&Seat].target);
    // Seat 97.8 cm around is 48.9 flat; regular seat ease is +1.2 to +1.8" flat.
    assert!(close(low, 48.9 + 1.2 * IN));
    assert!(close(high, 48.9 + 1.8 * IN));
    assert!(close(middle, 48.9 + 1.5 * IN));
    assert_eq!(t.targets[&Seat].from, TargetFrom::Body);
}

#[test]
fn the_thigh_is_compared_with_the_body_at_the_brands_thigh_point() {
    let points = Points {
        thigh: ThighPoint::OneInchBelow,
        ..ih_points()
    };
    let t = targets(&profile(Room::Snug), None, &ease(), points, false);
    let (low, ..) = range_of(&t.targets[&Thigh].target);
    assert!(close(low, 55.0 / 2.0 + 0.6 * IN));
}

#[test]
fn each_brands_knee_is_compared_with_the_leg_at_that_brands_knee_point() {
    for (point, leg) in [
        (KneePoint::Below12, 41.0),
        (KneePoint::Below13, 40.0),
        (KneePoint::Below14, 39.0),
        (KneePoint::HalfInseam, 39.5),
    ] {
        let points = Points {
            knee: point,
            ..ih_points()
        };
        let t = targets(&profile(Room::Roomy), None, &ease(), points, false);
        let (low, ..) = range_of(&t.targets[&Knee].target);
        assert!(close(low, leg / 2.0 + 2.0 * IN), "{point:?}");
    }
}

#[test]
fn without_a_pair_there_is_no_leg_opening_or_rise_target() {
    let t = targets(&profile(Room::Regular), None, &ease(), ih_points(), false);
    for m in [LegOpening, FrontRise, BackRise] {
        assert!(!t.targets.contains_key(&m), "{m:?}");
    }
}

#[test]
fn the_inseam_is_at_least_the_inseam_you_want() {
    let t = targets(&profile(Room::Regular), None, &ease(), ih_points(), false);
    assert_eq!(t.targets[&Inseam].target, Target::AtLeast { min: 81.0 });
}

fn pair_profile() -> (Profile, PairReadings) {
    let readings = PairReadings {
        waist: Some(42.0),
        thigh_crotch: Some(31.0),
        knee_13in: Some(23.0),
        leg_opening: Some(20.0),
        front_rise: Some(28.0),
        inseam: Some(84.0),
        ..Default::default()
    };
    let mut p = profile(Room::Regular);
    p.reference_pair = Some(ReferencePair {
        id: "pair".into(),
        name: "My 634S".into(),
        source: PairSource::Measured {
            readings: readings.clone(),
        },
        feel: Default::default(),
        notes: String::new(),
    });
    (p, readings)
}

#[test]
fn where_a_pair_has_a_reading_the_target_is_that_reading_plus_or_minus_a_quarter_inch() {
    let (p, readings) = pair_profile();
    let t = targets(&p, Some(&readings), &ease(), ih_points(), false);
    for (m, v) in [
        (Waist, 42.0),
        (Thigh, 31.0),
        (Knee, 23.0),
        (LegOpening, 20.0),
        (FrontRise, 28.0),
    ] {
        assert_eq!(
            t.targets[&m].target,
            Target::Range {
                low: v - QUARTER_INCH,
                middle: v,
                high: v + QUARTER_INCH
            },
            "{m:?}"
        );
        assert_eq!(t.targets[&m].from, TargetFrom::Pair);
    }
    // The pair has no seat reading, so the seat falls back to the body.
    assert_eq!(t.targets[&Seat].from, TargetFrom::Body);
}

#[test]
fn a_pair_reading_at_another_knee_point_isnt_used() {
    let (p, readings) = pair_profile();
    let points = Points {
        knee: KneePoint::Below12,
        ..ih_points()
    };
    let t = targets(&p, Some(&readings), &ease(), points, false);
    assert_eq!(t.targets[&Knee].from, TargetFrom::Body);
}

#[test]
fn each_too_nudges_the_target_a_quarter_inch_that_way() {
    let (mut p, readings) = pair_profile();
    let pair = p.reference_pair.as_mut().unwrap();
    pair.feel.insert(Thigh, Feel::TooTight);
    pair.feel.insert(LegOpening, Feel::TooLoose);
    pair.feel.insert(FrontRise, Feel::TooShort);
    let t = targets(&p, Some(&readings), &ease(), ih_points(), false);
    assert!(close(
        range_of(&t.targets[&Thigh].target).1,
        31.0 + QUARTER_INCH
    ));
    assert!(close(
        range_of(&t.targets[&LegOpening].target).1,
        20.0 - QUARTER_INCH
    ));
    assert!(close(
        range_of(&t.targets[&FrontRise].target).1,
        28.0 + QUARTER_INCH
    ));
}

#[test]
fn a_pairs_inseam_shifted_by_its_feel_sets_the_least_inseam() {
    let (mut p, readings) = pair_profile();
    p.reference_pair
        .as_mut()
        .unwrap()
        .feel
        .insert(Inseam, Feel::TooLong);
    let t = targets(&p, Some(&readings), &ease(), ih_points(), false);
    assert_eq!(
        t.targets[&Inseam].target,
        Target::AtLeast {
            min: 84.0 - QUARTER_INCH
        }
    );
}

#[test]
fn a_measurement_set_to_doesnt_matter_has_no_target() {
    let mut p = profile(Room::Regular);
    p.desired_fit.doesnt_matter = vec![Seat, Inseam];
    let t = targets(&p, None, &ease(), ih_points(), false);
    assert!(!t.targets.contains_key(&Seat));
    assert!(!t.targets.contains_key(&Inseam));
    assert!(t.targets.contains_key(&Thigh));
}

#[test]
fn the_hard_limit_is_the_body_plus_minimum_ease() {
    let t = targets(&profile(Room::Snug), None, &ease(), ih_points(), false);
    assert!(close(t.limits[&Waist], 81.3 / 2.0 + 0.2 * IN));
    assert!(close(t.limits[&Seat], 48.9 + 0.7 * IN));
    assert!(close(t.limits[&Thigh], 57.2 / 2.0 + 0.2 * IN));
    assert!(close(t.limits[&Knee], 40.0 / 2.0 + 0.3 * IN));
}

#[test]
fn stretch_lowers_every_range_by_2_cm_around_and_the_minimum_to_zero() {
    let t = targets(&profile(Room::Regular), None, &ease(), ih_points(), true);
    let (low, _, high) = range_of(&t.targets[&Seat].target);
    assert!(close(low, 48.9 + 1.2 * IN - 1.0));
    assert!(close(high, 48.9 + 1.8 * IN - 1.0));
    assert!(close(t.limits[&Seat], 48.9));
}

#[test]
fn a_stretch_range_never_drops_below_the_minimum() {
    let t = targets(&profile(Room::Regular), None, &ease(), ih_points(), true);
    let (low, _, high) = range_of(&t.targets[&Waist].target);
    // +0.2..+0.6" flat less 1 cm is -0.49..+0.52 cm: the low end stops at 0.
    assert!(close(low, 81.3 / 2.0));
    assert!(close(high, 81.3 / 2.0 + 0.6 * IN - 1.0));
}
