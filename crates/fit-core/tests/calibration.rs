//! The ease-table calibration against Levi's body chart (docs/research/ease.md).
mod common;

use common::*;
use fit_core::calibration::{calibrate, report, slim_in_regular_share, BodyChart, Landing};
use fit_core::catalogue::{ChartState, Measurement::*};
use fit_core::units::Unit;

fn levis() -> BodyChart {
    BodyChart::from_json(fit_core::embedded::LEVIS_BODY_CHART).unwrap()
}

#[test]
fn levis_size_32_is_the_body_the_research_names() {
    let b = levis();
    let s = b.sizes.iter().find(|s| s.size == 32.0).unwrap();
    assert_eq!((s.waist, s.seat, s.thigh), (81.3, 97.8, 57.2));
}

#[test]
fn each_cut_lands_in_the_room_most_of_its_sizes_fall_in() {
    // Levi's 32 thigh is 57.2 cm around, 28.6 flat. Snug thigh ease is
    // +0.6..+1.0" flat, so a 12" (30.48 cm) thigh is +1.88 cm: snug.
    // Regular is +1.0..+1.6", so 12.75" (32.39 cm) is +3.79 cm: regular.
    let mut slim = product(
        "s",
        "Slim One",
        vec![chart(
            ChartState::AsSold,
            Unit::In,
            vec![row("32", 32.0, &[(Waist, 33.0), (Thigh, 12.0)])],
        )],
    );
    slim.fabric.weight_oz = Some(14.0);
    let roomy = product(
        "r",
        "Wide One",
        vec![chart(
            ChartState::AsSold,
            Unit::In,
            vec![row("32", 32.0, &[(Waist, 33.0), (Thigh, 12.75)])],
        )],
    );
    let mut b = brand("b", vec![slim, roomy]);
    b.cuts[0].brand_label = Some("Slim Tapered".into());
    let results = calibrate(&catalogue(vec![b]), &ease(), &levis());
    let slim = results.iter().find(|r| r.cut == "Slim One").unwrap();
    assert_eq!(slim.lands, Landing::Snug);
    assert!(slim.called_slim());
    assert_eq!(
        results.iter().find(|r| r.cut == "Wide One").unwrap().lands,
        Landing::Regular
    );
    assert_eq!(slim_in_regular_share(&results), Some(0.0));
}

/// The plan's rule: if the cuts brands call slim mostly land in regular,
/// the snug and regular ranges must move up (by PR, with the user's OK).
#[test]
fn the_cuts_brands_call_slim_dont_mostly_land_in_regular() {
    let (catalogue, ease) = fit_core::embedded::load().unwrap();
    let results = calibrate(&catalogue, &ease, &levis());
    let Some(share) = slim_in_regular_share(&results) else {
        eprintln!("No cut in the built-in catalogue is called slim; nothing to calibrate.");
        return;
    };
    assert!(
        share <= 0.5,
        "{:.0}% of the cuts brands call slim land in regular or roomier, so the ease table's snug and \
         regular ranges should move up.\n\n{}",
        share * 100.0,
        report(&results)
    );
}
