//! Finding matches across the catalogue: ranking, the hard limit and filters.
mod common;

use common::*;
use fit_core::catalogue::{ChartState, Convention, Measurement::*, Size, Sold, Status};
use fit_core::matching::{find_matches, Filters};
use fit_core::profile::*;
use fit_core::units::Unit;

/// A size in inches with the body-friendly numbers of an Iron Heart 634S 32:
/// waist 33" around, thigh 12.5", knee 9.3", opening 8.3", rises 11/15, inseam 34.
fn size(label: &str, thigh: f64) -> Size {
    row(
        label,
        32.0,
        &[
            (Waist, 33.0),
            (FrontRise, 11.0),
            (BackRise, 15.0),
            (Thigh, thigh),
            (Knee, 9.3),
            (LegOpening, 8.3),
            (Inseam, 34.0),
        ],
    )
}

fn simple(id: &str, cut: &str, thighs: &[(&str, f64)]) -> TestProduct {
    product(
        id,
        cut,
        vec![chart(
            ChartState::AsSold,
            Unit::In,
            thighs.iter().map(|(l, t)| size(l, *t)).collect(),
        )],
    )
}

fn filters() -> Filters {
    Filters {
        currency: "USD".into(),
        ..Filters::default()
    }
}

fn run(products: Vec<TestProduct>, profile: &Profile, f: &Filters) -> fit_core::matching::Results {
    find_matches(&catalogue(vec![brand("b", products)]), &ease(), profile, f)
}

// Body thigh 57.2 cm around (28.6 flat) and regular thigh ease +1.0..+1.6" put
// the thigh target at 31.14..32.66 cm flat, middle 31.9 cm (12.56").

#[test]
fn there_is_one_row_per_cut_with_its_closest_product_and_size() {
    let r = run(
        vec![
            simple("a1", "A", &[("31", 12.0), ("32", 12.55)]),
            simple("a2", "A", &[("32", 13.0)]),
            simple("b1", "B", &[("32", 13.2)]),
        ],
        &profile(Room::Regular),
        &filters(),
    );
    assert_eq!(r.rows.len(), 2);
    assert_eq!(r.rows[0].cut, "A");
    assert_eq!(r.rows[0].best.product_id, "b/a1");
    assert_eq!(r.rows[0].best.size, "32");
    // Opening the row lists the cut's other products, then its other sizes.
    assert_eq!(r.rows[0].other_products.len(), 1);
    assert_eq!(r.rows[0].other_products[0].product_id, "b/a2");
    assert_eq!(r.rows[0].other_sizes[0].size, "31");
    assert_eq!(r.rows[1].cut, "B");
}

#[test]
fn a_matchs_closeness_is_its_worst_measurement() {
    let r = run(
        vec![simple("a", "A", &[("32", 12.56)])],
        &profile(Room::Regular),
        &filters(),
    );
    let m = &r.rows[0].best;
    let worst = m.measurements.iter().filter_map(|x| x.closeness).fold(100.0, f64::min);
    assert!(close(m.closeness, worst));
    assert!(m.average >= m.closeness);
}

#[test]
fn anything_tighter_than_the_body_plus_minimum_ease_is_left_out() {
    // Body thigh flat 28.6 cm + 0.2" minimum = 29.11 cm; 11.4" is 28.96 cm.
    let r = run(
        vec![simple("a", "A", &[("tight", 11.4), ("ok", 12.0)])],
        &profile(Room::Snug),
        &filters(),
    );
    assert_eq!(r.rows[0].best.size, "ok");
    assert!(r.rows[0].other_sizes.is_empty());
}

#[test]
fn the_hard_limit_applies_even_when_a_reference_pair_wants_it_tighter() {
    let mut p = profile(Room::Snug);
    p.reference_pair = Some(ReferencePair {
        id: "x".into(),
        name: "tight pair".into(),
        source: PairSource::Measured {
            readings: PairReadings {
                thigh_crotch: Some(11.4 * IN),
                ..Default::default()
            },
        },
        feel: Default::default(),
        notes: String::new(),
    });
    let r = run(vec![simple("a", "A", &[("tight", 11.4)])], &p, &filters());
    assert!(r.rows.is_empty());
}

#[test]
fn shrinkage_unknown_is_ranked_after_the_rest() {
    let mut unknown = simple("u", "U", &[("32", 12.56)]);
    unknown.sold = Some(Sold::Raw);
    let r = run(
        vec![unknown, simple("k", "K", &[("32", 13.0)])],
        &profile(Room::Regular),
        &filters(),
    );
    assert_eq!(r.rows[0].cut, "K");
    assert_eq!(r.rows[1].cut, "U");
    assert_eq!(r.rows[1].best.basis, fit_core::after_wash::Basis::ShrinkageUnknown);
}

#[test]
fn equally_close_matches_go_to_the_higher_average_then_more_measurements() {
    // Both have the same worst measurement (the thigh, too big); B's other
    // measurements are further off, so A ranks first.
    let a = simple("a", "A", &[("32", 13.2)]);
    let mut b = simple("b", "B", &[("32", 13.2)]);
    b.charts[0].sizes[0].measurements.insert(Knee, 9.9);
    let r = run(vec![b, a], &profile(Room::Regular), &filters());
    assert!(close(r.rows[0].best.closeness, r.rows[1].best.closeness));
    assert_eq!(r.rows[0].cut, "A");

    // Same worst and average, but C also publishes a seat, exactly as far
    // past its target's edge as the thigh is.
    let thigh_over = 13.2 * IN - (57.2 / 2.0 + 1.6 * IN);
    let c_size = {
        let mut s = size("32", 13.2);
        s.measurements.insert(Seat, (97.8 / 2.0 + 1.8 * IN + thigh_over) / IN);
        s
    };
    let c = product("c", "C", vec![chart(ChartState::AsSold, Unit::In, vec![c_size])]);
    let d = simple("d", "D", &[("32", 13.2)]);
    let mut p = profile(Room::Regular);
    p.desired_fit.doesnt_matter = vec![Knee, Waist, Inseam];
    let r = run(vec![d, c], &p, &filters());
    let (first, second) = (&r.rows[0].best, &r.rows[1].best);
    assert_eq!(first.product_id, "b/c");
    assert!(first.compared > second.compared);
}

#[test]
fn a_measurement_the_brand_doesnt_publish_is_shown_as_not_published() {
    let r = run(
        vec![simple("a", "A", &[("32", 12.56)])],
        &profile(Room::Regular),
        &filters(),
    );
    let seat = r.rows[0]
        .best
        .measurements
        .iter()
        .find(|m| m.measurement == Seat)
        .unwrap();
    assert!(seat.value.is_none());
    assert!(seat.target.is_some());
    assert!(seat.closeness.is_none());
}

#[test]
fn matches_from_a_brand_that_doesnt_say_where_it_measures_are_approximate() {
    let mut b = brand("vague", vec![simple("a", "A", &[("32", 12.56)])]);
    b.measuring_points.thigh = Some("unknown".into());
    let r = find_matches(&catalogue(vec![b]), &ease(), &profile(Room::Regular), &filters());
    assert!(r.rows[0].best.approximate);
    let r = run(
        vec![simple("a", "A", &[("32", 12.56)])],
        &profile(Room::Regular),
        &filters(),
    );
    assert!(!r.rows[0].best.approximate);
}

#[test]
fn without_a_reference_pair_the_results_say_they_came_from_the_body() {
    let r = run(
        vec![simple("a", "A", &[("32", 12.56)])],
        &profile(Room::Regular),
        &filters(),
    );
    assert!(r.from_body_only);
}

// --- Brands sold in lengths --------------------------------------------------

fn nudie_size(length: f64, inseam: f64) -> Size {
    let mut s = size(&format!("W32 L{length}"), 12.56);
    s.length_tag = Some(length);
    s.measurements.insert(Inseam, inseam);
    s
}

#[test]
fn a_brand_sold_in_lengths_gets_the_shortest_length_at_least_your_inseam() {
    // Inseam wanted: 81 cm (31.9").
    let p = product(
        "lean-dean",
        "Lean Dean",
        vec![chart(
            ChartState::AsSold,
            Unit::In,
            vec![nudie_size(30.0, 30.0), nudie_size(32.0, 32.0), nudie_size(34.0, 34.0)],
        )],
    );
    let mut b = brand("nudie", vec![p]);
    b.sizing = fit_core::catalogue::Sizing::WaistAndLength;
    let r = find_matches(&catalogue(vec![b]), &ease(), &profile(Room::Regular), &filters());
    assert_eq!(r.rows[0].best.size, "W32 L32");
    assert!(r.rows[0].other_sizes.iter().all(|m| !m.size.starts_with("W32")));
}

// --- Filters -----------------------------------------------------------------

fn two_brands() -> fit_core::catalogue::Catalogue {
    let mut gbp = simple("ih", "634S", &[("32", 12.56)]);
    gbp.list_price.as_mut().unwrap().amount = 300.0;
    gbp.list_price.as_mut().unwrap().currency = "GBP".into();
    gbp.fabric.weight_oz = Some(21.0);
    let mut stretch = simple("nf", "Weird Guy", &[("32", 12.56)]);
    stretch.fabric.composition = common::stretch(2.0);
    stretch.fabric.sanforized = Some(false);
    stretch.fabric.weight_oz = Some(11.0);
    let mut gone = simple("nf-old", "Super Guy", &[("32", 12.56)]);
    gone.status = Status::Discontinued;
    catalogue(vec![
        brand("iron-heart", vec![gbp]),
        brand("naked-and-famous", vec![stretch, gone]),
    ])
}

fn cuts(f: &Filters) -> Vec<String> {
    let r = find_matches(&two_brands(), &ease(), &profile(Room::Regular), f);
    let mut c: Vec<String> = r.rows.iter().map(|r| r.cut.clone()).collect();
    c.sort();
    c
}

#[test]
fn current_products_only_is_on_by_default() {
    assert!(Filters::default().current_only);
    assert_eq!(cuts(&filters()), ["634S", "Weird Guy"]);
    let all = Filters {
        current_only: false,
        ..filters()
    };
    assert_eq!(cuts(&all), ["634S", "Super Guy", "Weird Guy"]);
    let r = find_matches(&two_brands(), &ease(), &profile(Room::Regular), &all);
    let gone = r.rows.iter().find(|r| r.cut == "Super Guy").unwrap();
    assert!(!gone.best.on_brands_site);
}

#[test]
fn the_brand_filter_keeps_only_the_chosen_brands() {
    let f = Filters {
        brands: Some(vec!["iron-heart".into()]),
        ..filters()
    };
    assert_eq!(cuts(&f), ["634S"]);
}

#[test]
fn the_price_filter_uses_the_list_price_converted_to_your_currency() {
    // £300 at 0.8 GBP and 1.25 USD per euro is about $468.75.
    let r = find_matches(&two_brands(), &ease(), &profile(Room::Regular), &filters());
    let ih = r.rows.iter().find(|r| r.cut == "634S").unwrap();
    assert!(close(ih.best.price.unwrap(), 468.75));
    let f = Filters {
        price_max: Some(400.0),
        ..filters()
    };
    assert_eq!(cuts(&f), ["Weird Guy"]);
    let f = Filters {
        price_min: Some(400.0),
        ..filters()
    };
    assert_eq!(cuts(&f), ["634S"]);
}

#[test]
fn try_on_brands_only_keeps_the_brands_you_marked() {
    let f = Filters {
        try_on_only: true,
        try_on_brands: vec!["naked-and-famous".into()],
        ..filters()
    };
    assert_eq!(cuts(&f), ["Weird Guy"]);
}

#[test]
fn the_fabric_weight_filter_keeps_weights_in_range() {
    let f = Filters {
        weight_min: Some(15.0),
        ..filters()
    };
    assert_eq!(cuts(&f), ["634S"]);
    let f = Filters {
        weight_max: Some(15.0),
        ..filters()
    };
    assert_eq!(cuts(&f), ["Weird Guy"]);
}

#[test]
fn sanforized_only_and_no_stretch_are_off_by_default_and_filter_when_on() {
    assert!(!Filters::default().sanforized_only);
    assert!(!Filters::default().no_stretch);
    assert_eq!(
        cuts(&Filters {
            sanforized_only: true,
            ..filters()
        }),
        ["634S"]
    );
    assert_eq!(
        cuts(&Filters {
            no_stretch: true,
            ..filters()
        }),
        ["634S"]
    );
}

#[test]
fn only_your_leg_shape_and_rise_are_shown_with_a_switch_for_the_rest() {
    // Tapered: knee 9.3" with a 7.5" opening. Straight: 8.3" opening.
    let mut tapered = simple("t", "Tapered", &[("32", 12.56)]);
    tapered.charts[0].sizes[0].measurements.insert(LegOpening, 7.5);
    let straight = simple("s", "Straight", &[("32", 12.56)]);
    let mut p = profile(Room::Regular);
    p.desired_fit.leg_shape = Some(LegShape::Tapered);
    let r = run(vec![tapered.clone(), straight.clone()], &p, &filters());
    assert_eq!(r.rows.len(), 1);
    assert_eq!(r.rows[0].cut, "Tapered");
    assert_eq!(r.hidden_by_shape_or_rise, 1);
    let f = Filters {
        show_other_shapes_and_rises: true,
        ..filters()
    };
    let r = run(vec![tapered, straight], &p, &f);
    assert_eq!(r.rows.len(), 2);
    assert!(
        !r.rows
            .iter()
            .find(|r| r.cut == "Straight")
            .unwrap()
            .best
            .fits_shape_and_rise
    );
}

#[test]
fn rise_filters_by_the_front_rise_third_at_the_same_waist() {
    let with_rise = |id: &str, rise: f64| {
        let mut p = simple(id, id, &[("32", 12.56)]);
        p.charts[0].sizes[0].measurements.insert(FrontRise, rise);
        p
    };
    let mut p = profile(Room::Regular);
    p.desired_fit.rise = Some(Rise::High);
    let r = run(
        vec![with_rise("low", 10.0), with_rise("mid", 11.0), with_rise("high", 12.0)],
        &p,
        &filters(),
    );
    assert_eq!(r.rows.len(), 1);
    assert_eq!(r.rows[0].cut, "high");
    assert_eq!(r.rows[0].best.rise, Some(Rise::High));
}

// --- Reference pairs ---------------------------------------------------------

#[test]
fn a_pair_picked_from_the_catalogue_uses_that_sizes_after_wash_measurements() {
    let mut raw = simple("raw", "Raw", &[("32", 12.0)]);
    raw.sold = Some(Sold::Raw);
    raw.shrinkage = Some(percent_shrinkage(5.0, 5.0));
    let other = simple("other", "Other", &[("31", 11.4), ("32", 12.0)]);
    let mut p = profile(Room::Roomy);
    p.reference_pair = Some(ReferencePair {
        id: "x".into(),
        name: "Raw 32".into(),
        source: PairSource::FromCatalogue {
            product_id: "b/raw".into(),
            size: "32".into(),
        },
        feel: Default::default(),
        notes: String::new(),
    });
    let r = run(vec![raw, other], &p, &filters());
    assert!(!r.from_body_only);
    let other = r.rows.iter().find(|r| r.cut == "Other").unwrap();
    let thigh = other.best.measurements.iter().find(|m| m.measurement == Thigh).unwrap();
    // 12" less 5% is 11.4" flat.
    assert!(close(thigh.target.unwrap().target.ideal(), 12.0 * 0.95 * IN));
    assert_eq!(thigh.target.unwrap().from, fit_core::targets::TargetFrom::Pair);
}

#[test]
fn a_cm_chart_with_a_waist_around_compares_flat() {
    let mut c = chart(
        ChartState::AsSold,
        Unit::Cm,
        vec![row("32", 32.0, &[(Waist, 83.0), (Thigh, 31.9), (Inseam, 82.0)])],
    );
    c.conventions = Some([(Waist, Convention::Around)].into());
    let r = run(vec![product("m", "M", vec![c])], &profile(Room::Regular), &filters());
    let waist = r.rows[0]
        .best
        .measurements
        .iter()
        .find(|m| m.measurement == Waist)
        .unwrap();
    assert!(close(waist.value.unwrap(), 41.5));
}
