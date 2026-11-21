//! The checks CI runs on every catalogue change.
mod common;

use common::*;
use fit_core::catalogue::{ChartState, Measurement::*};
use fit_core::units::Unit;
use fit_core::validate::validate;

fn good() -> fit_core::catalogue::Product {
    product(
        "p",
        "C",
        vec![chart(
            ChartState::AsSold,
            Unit::In,
            vec![row(
                "32",
                32.0,
                &[(Waist, 33.0), (Thigh, 12.5), (Inseam, 34.0)],
            )],
        )],
    )
}

fn problems(p: fit_core::catalogue::Product) -> Vec<String> {
    validate(&catalogue(vec![brand("b", vec![p])]), &ease())
}

#[test]
fn a_good_product_passes() {
    assert_eq!(problems(good()), Vec::<String>::new());
}

#[test]
fn every_number_needs_a_link_and_a_date() {
    let mut p = good();
    p.charts[0].source = "".into();
    p.charts[0].seen = "last week".into();
    let found = problems(p);
    assert!(found.iter().any(|s| s.contains("link")), "{found:?}");
    assert!(found.iter().any(|s| s.contains("date")), "{found:?}");
}

#[test]
fn numbers_must_fall_in_sensible_ranges() {
    let mut p = good();
    // A thigh of 12.5 cm flat is far too small: someone forgot the unit.
    p.charts[0].unit = Unit::Cm;
    let found = problems(p);
    assert!(
        found
            .iter()
            .any(|s| s.contains("thigh") && s.contains("range")),
        "{found:?}"
    );
}

#[test]
fn no_after_wash_number_may_be_bigger_than_its_as_sold_number() {
    let mut p = good();
    p.sold_washed = false;
    p.charts.push(chart(
        ChartState::AfterWash,
        Unit::In,
        vec![row("32", 32.0, &[(Waist, 34.0), (Thigh, 12.0)])],
    ));
    let found = problems(p);
    assert!(
        found
            .iter()
            .any(|s| s.contains("after-wash") && s.contains("waist")),
        "{found:?}"
    );
}

#[test]
fn every_product_has_at_least_a_waist_and_a_thigh() {
    let p = product(
        "p",
        "C",
        vec![chart(
            ChartState::AsSold,
            Unit::In,
            vec![row("32", 32.0, &[(Waist, 33.0), (Inseam, 34.0)])],
        )],
    );
    let found = problems(p);
    assert!(
        found.iter().any(|s| s.contains("waist and a thigh")),
        "{found:?}"
    );
}

#[test]
fn every_measurement_needs_its_measuring_point() {
    let mut p = good();
    p.charts[0].points.remove(&Thigh);
    let found = problems(p);
    assert!(
        found.iter().any(|s| s.contains("measuring point")),
        "{found:?}"
    );
}

#[test]
fn thigh_and_knee_points_must_be_ones_the_matching_knows() {
    let mut p = good();
    p.charts[0].points.get_mut(&Knee).unwrap().at = "near the knee".into();
    let found = problems(p);
    assert!(
        found
            .iter()
            .any(|s| s.contains("knee") && s.contains("point")),
        "{found:?}"
    );
}

#[test]
fn prices_need_a_currency_with_an_exchange_rate() {
    let mut p = good();
    p.list_price.as_mut().unwrap().currency = "XYZ".into();
    let found = problems(p);
    assert!(found.iter().any(|s| s.contains("XYZ")), "{found:?}");
}

#[test]
fn product_ids_are_unique() {
    let found = validate(&catalogue(vec![brand("b", vec![good(), good()])]), &ease());
    assert!(found.iter().any(|s| s.contains("twice")), "{found:?}");
}

#[test]
fn the_shipped_catalogue_and_ease_table_are_valid() {
    let (catalogue, ease) = fit_core::embedded::load().expect("the built-in data loads");
    let found = validate(&catalogue, &ease);
    assert!(found.is_empty(), "{}", found.join("\n"));
}
