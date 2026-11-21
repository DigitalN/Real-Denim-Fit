//! Loading the catalogue files and the checks CI runs on them
//! (docs/catalogue-format.md).
mod common;

use std::path::Path;

use common::*;
use fit_core::after_wash::{after_wash, Basis};
use fit_core::catalogue::{Catalogue, ChartState, Measurement::*, Sold};
use fit_core::units::Unit;
use fit_core::validate::validate;

fn fixtures() -> Catalogue {
    Catalogue::from_dir(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/catalogue")).unwrap()
}

#[test]
fn a_catalogue_folder_loads_with_charts_inheriting_from_the_brand() {
    let c = fixtures();
    assert_eq!(c.brands.len(), 1);
    let b = &c.brands[0];
    assert_eq!(b.cuts[0].brand_label.as_deref(), Some("Straight Cut"));
    assert_eq!(b.cuts[0].line, Some(fit_core::catalogue::Line::Men));
    let (brand, cut, p) = c.product("test-brand/634-21oz-indigo").unwrap();
    assert_eq!(p.finish, Some(fit_core::catalogue::Finish::Rinsed));
    // The UHR product has no finish key yet: read as not stated.
    assert_eq!(c.product("test-brand/634-uhr").unwrap().2.finish, None);
    assert_eq!(cut.name, "634");
    let w = after_wash(brand, p).unwrap();
    assert_eq!(w.basis, Basis::SoldWashed);
    // Waist inherits "around" from brand.json: 31.7" around is 40.26 cm flat.
    assert!(close(w.sizes[0].after[&Waist], 31.7 * 2.54 / 2.0));
    assert!(
        !w.approximate,
        "no seat is published, so the unknown seat point doesn't matter"
    );
}

#[test]
fn a_chart_can_override_conventions_measuring_points_and_tolerance() {
    let c = fixtures();
    let (brand, _, p) = c.product("test-brand/634-uhr").unwrap();
    let chart = &p.charts[0];
    let e = chart.effective(brand);
    assert_eq!(e.tolerance.unwrap().value, 1.0);
    let w = after_wash(brand, p).unwrap();
    assert_eq!(w.basis, Basis::FromShrinkage);
    assert!(w.approximate, "the chart's knee point is unknown");
    // Waist 42.4 cm flat, less 1" around (1.27 cm flat).
    assert!(close(w.sizes[0].after[&Waist], 42.4 - 1.27));
}

#[test]
fn the_fixture_catalogue_is_valid() {
    let found = validate(&fixtures(), &ease());
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[test]
fn a_cut_file_whose_brand_doesnt_match_its_folder_is_refused() {
    let files = vec![
        (
            "exchange-rates.json".to_string(),
            r#"{"source":"https://x.example","date":"2026-11-20","base":"EUR","rates":{}}"#.to_string(),
        ),
        (
            "a/634.json".to_string(),
            r#"{"id":"634","brand":"b","name":"634","brand_label":null,"products":[]}"#.to_string(),
        ),
    ];
    assert!(Catalogue::from_files(&files)
        .unwrap_err()
        .contains("doesn't match its folder"));
}

#[test]
fn a_misspelt_measurement_is_refused() {
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/catalogue/test-brand/634.json"),
    )
    .unwrap()
    .replace("\"leg_opening\": 8.7", "\"leg_openning\": 8.7");
    let files = vec![
        (
            "exchange-rates.json".to_string(),
            r#"{"source":"https://x.example","date":"2026-11-20","base":"EUR","rates":{}}"#.to_string(),
        ),
        (
            "test-brand/brand.json".to_string(),
            std::fs::read_to_string(
                Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/catalogue/test-brand/brand.json"),
            )
            .unwrap(),
        ),
        ("test-brand/634.json".to_string(), text),
    ];
    assert!(Catalogue::from_files(&files).is_err());
}

fn good() -> TestProduct {
    product(
        "p",
        "C",
        vec![chart(
            ChartState::AsSold,
            Unit::In,
            vec![row("32", 32.0, &[(Waist, 33.0), (Thigh, 12.5), (Inseam, 34.0)])],
        )],
    )
}

fn problems(p: TestProduct) -> Vec<String> {
    validate(&catalogue(vec![brand("b", vec![p])]), &ease())
}

#[test]
fn a_good_product_passes() {
    assert_eq!(problems(good()), Vec::<String>::new());
}

#[test]
fn every_number_needs_a_link_and_a_date() {
    let mut p = good();
    p.charts[0].source.url = "".into();
    p.charts[0].source.seen = "last week".into();
    let found = problems(p);
    assert!(found.iter().any(|s| s.contains("link")), "{found:?}");
    assert!(found.iter().any(|s| s.contains("date")), "{found:?}");
}

#[test]
fn numbers_must_fall_in_sensible_ranges() {
    let mut p = good();
    // A thigh of 12.5 cm flat is far too small: someone picked the wrong unit.
    p.charts[0].unit = Unit::Cm;
    let found = problems(p);
    assert!(
        found.iter().any(|s| s.contains("thigh") && s.contains("range")),
        "{found:?}"
    );
}

#[test]
fn no_after_wash_number_may_be_bigger_than_its_as_sold_number() {
    let mut p = good();
    p.sold = Some(Sold::Raw);
    p.charts.push(chart(
        ChartState::AfterWash,
        Unit::In,
        vec![row("32", 32.0, &[(Waist, 34.0), (Thigh, 12.0)])],
    ));
    let found = problems(p);
    assert!(
        found.iter().any(|s| s.contains("after-wash") && s.contains("waist")),
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
    assert!(found.iter().any(|s| s.contains("waist and a thigh")), "{found:?}");
}

#[test]
fn a_size_may_lack_a_waist_when_the_product_has_one_elsewhere() {
    let p = product(
        "p",
        "C",
        vec![chart(
            ChartState::AsSold,
            Unit::In,
            vec![
                row("28", 28.0, &[(Thigh, 11.5)]),
                row("32", 32.0, &[(Waist, 33.0), (Thigh, 12.5)]),
            ],
        )],
    );
    assert_eq!(problems(p), Vec::<String>::new());
}

#[test]
fn a_product_without_a_chart_should_have_been_left_out() {
    let found = problems(product("p", "C", vec![]));
    assert!(found.iter().any(|s| s.contains("no chart")), "{found:?}");
}

#[test]
fn every_width_needs_a_convention_and_thigh_knee_seat_a_measuring_point() {
    let mut b = brand("b", vec![good()]);
    b.conventions.remove(&Thigh);
    b.measuring_points.thigh = None;
    let found = validate(&catalogue(vec![b]), &ease());
    assert!(found.iter().any(|s| s.contains("convention")), "{found:?}");
    assert!(found.iter().any(|s| s.contains("measuring point")), "{found:?}");
}

#[test]
fn measuring_points_must_be_ones_the_matching_knows() {
    let mut b = brand("b", vec![good()]);
    b.measuring_points.knee = Some("near the knee".into());
    let found = validate(&catalogue(vec![b]), &ease());
    assert!(
        found.iter().any(|s| s.contains("knee") && s.contains("point")),
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
fn product_ids_are_unique_within_a_brand() {
    let found = validate(&catalogue(vec![brand("b", vec![good(), good()])]), &ease());
    assert!(found.iter().any(|s| s.contains("twice")), "{found:?}");
}

#[test]
fn ids_are_lowercase_kebab_case() {
    let mut p = good();
    p.id = "Weird Guy".into();
    assert!(problems(p).iter().any(|s| s.contains("kebab")));
}

#[test]
fn brands_sold_in_lengths_need_a_length_on_every_size() {
    let mut b = brand("b", vec![good()]);
    b.sizing = fit_core::catalogue::Sizing::WaistAndLength;
    let found = validate(&catalogue(vec![b]), &ease());
    assert!(found.iter().any(|s| s.contains("length_tag")), "{found:?}");
}

#[test]
fn the_shipped_catalogue_and_ease_table_are_valid() {
    let (catalogue, ease) = fit_core::embedded::load().expect("the built-in data loads");
    let found = validate(&catalogue, &ease);
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[test]
fn a_line_must_be_men_women_unisex_or_null() {
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/catalogue/test-brand/634.json"),
    )
    .unwrap()
    .replace("\"line\": \"men\"", "\"line\": \"mens\"");
    let files = vec![
        (
            "exchange-rates.json".to_string(),
            r#"{"source":"https://x.example","date":"2026-11-20","base":"EUR","rates":{}}"#.to_string(),
        ),
        (
            "test-brand/brand.json".to_string(),
            std::fs::read_to_string(
                Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/catalogue/test-brand/brand.json"),
            )
            .unwrap(),
        ),
        ("test-brand/634.json".to_string(), text),
    ];
    assert!(Catalogue::from_files(&files)
        .unwrap_err()
        .contains("unknown variant `mens`"));
}

#[test]
fn a_finish_must_be_raw_rinsed_washed_or_null() {
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/catalogue/test-brand/634.json"),
    )
    .unwrap()
    .replace("\"finish\": \"rinsed\"", "\"finish\": \"faded\"");
    let files = vec![
        (
            "exchange-rates.json".to_string(),
            r#"{"source":"https://x.example","date":"2026-11-20","base":"EUR","rates":{}}"#.to_string(),
        ),
        (
            "test-brand/brand.json".to_string(),
            std::fs::read_to_string(
                Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/catalogue/test-brand/brand.json"),
            )
            .unwrap(),
        ),
        ("test-brand/634.json".to_string(), text),
    ];
    assert!(Catalogue::from_files(&files)
        .unwrap_err()
        .contains("unknown variant `faded`"));
}

#[test]
fn a_raw_finish_goes_with_sold_raw_and_rinsed_or_washed_with_one_washed() {
    use fit_core::catalogue::Finish;
    let with = |finish: Option<Finish>, sold: Option<Sold>| {
        let mut p = good();
        p.finish = finish;
        p.sold = sold;
        problems(p)
    };
    assert!(with(Some(Finish::Raw), Some(Sold::Raw)).is_empty());
    assert!(with(Some(Finish::Rinsed), Some(Sold::OneWashed)).is_empty());
    assert!(with(Some(Finish::Washed), Some(Sold::OneWashed)).is_empty());
    assert!(with(None, Some(Sold::Raw)).is_empty());
    for (finish, sold) in [
        (Finish::Raw, Sold::OneWashed),
        (Finish::Rinsed, Sold::Raw),
        (Finish::Washed, Sold::Raw),
    ] {
        let found = with(Some(finish), Some(sold));
        assert!(
            found.iter().any(|s| s.contains("finish")),
            "{finish:?} with {sold:?}: {found:?}"
        );
    }
}
