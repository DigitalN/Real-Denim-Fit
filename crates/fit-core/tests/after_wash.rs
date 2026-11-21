//! Every product is judged after its first wash (ADR 0004).
mod common;

use common::*;
use fit_core::after_wash::{after_wash, Basis};
use fit_core::catalogue::{ChartState, Convention, Measurement::*, Sold};
use fit_core::units::Unit;

fn ih_32() -> Vec<fit_core::catalogue::Size> {
    vec![row(
        "32",
        32.0,
        &[
            (Waist, 31.7),
            (FrontRise, 11.0),
            (Thigh, 12.5),
            (Knee, 9.3),
            (LegOpening, 8.7),
            (Inseam, 35.5),
        ],
    )]
}

#[test]
fn a_brands_after_wash_chart_is_used_when_it_has_one() {
    // Naked & Famous Hedorah, size 32: waist 35 pre-wash, 32 post-wash.
    let mut p = product(
        "hedorah",
        "Weird Guy",
        vec![
            chart(
                ChartState::AsSold,
                Unit::In,
                vec![row("32", 32.0, &[(Waist, 35.0), (Thigh, 12.5)])],
            ),
            chart(
                ChartState::AfterWash,
                Unit::In,
                vec![row("32", 32.0, &[(Waist, 32.0), (Thigh, 12.0)])],
            ),
        ],
    );
    p.sold = Some(Sold::Raw);
    let w = after_wash(&brand("b", vec![]), &p).unwrap();
    assert_eq!(w.basis, Basis::BrandChart);
    // 32" around is 16" flat: 40.64 cm.
    assert!(close(w.sizes[0].after[&Waist], 40.64));
    assert!(close(w.sizes[0].as_sold.as_ref().unwrap()[&Waist], 44.45));
}

#[test]
fn a_product_sold_washed_keeps_its_chart() {
    let p = product("634s", "634S", vec![chart(ChartState::AsSold, Unit::In, ih_32())]);
    let w = after_wash(&brand("b", vec![]), &p).unwrap();
    assert_eq!(w.basis, Basis::SoldWashed);
    assert!(close(w.sizes[0].after[&Thigh], 12.5 * 2.54));
    assert!(close(w.sizes[0].after[&Waist], 31.7 * 2.54 / 2.0));
}

#[test]
fn otherwise_the_chart_shrinks_by_the_brands_stated_shrinkage() {
    let mut p = product("raw", "634S", vec![chart(ChartState::AsSold, Unit::In, ih_32())]);
    p.sold = Some(Sold::Raw);
    p.shrinkage = Some(percent_shrinkage(2.0, 4.0));
    let w = after_wash(&brand("b", vec![]), &p).unwrap();
    assert_eq!(w.basis, Basis::FromShrinkage);
    assert!(close(w.sizes[0].after[&Thigh], 12.5 * 2.54 * 0.98));
    assert!(close(w.sizes[0].after[&Inseam], 35.5 * 2.54 * 0.96));
    assert!(close(w.sizes[0].after[&FrontRise], 11.0 * 2.54 * 0.96));
}

#[test]
fn unknown_shrinkage_on_a_product_that_will_shrink_is_marked() {
    let mut p = product("momo", "0705", vec![chart(ChartState::AsSold, Unit::In, ih_32())]);
    p.sold = Some(Sold::Raw);
    let w = after_wash(&brand("b", vec![]), &p).unwrap();
    assert_eq!(w.basis, Basis::ShrinkageUnknown);
    // The numbers stay as sold.
    assert!(close(w.sizes[0].after[&Thigh], 12.5 * 2.54));
}

#[test]
fn a_product_with_no_measurements_is_left_out() {
    let p = product("none", "634S", vec![]);
    assert!(after_wash(&brand("b", vec![]), &p).is_none());
    let empty = product("none", "634S", vec![chart(ChartState::AsSold, Unit::In, vec![])]);
    assert!(after_wash(&brand("b", vec![]), &empty).is_none());
}

#[test]
fn centimetre_charts_are_read_in_centimetres() {
    let mut c = chart(
        ChartState::AsSold,
        Unit::Cm,
        vec![row("32", 32.0, &[(Waist, 87.0), (Thigh, 34.5)])],
    );
    c.conventions = Some([(Waist, Convention::Around)].into());
    let w = after_wash(&brand("b", vec![]), &product("momo", "0705", vec![c])).unwrap();
    assert!(close(w.sizes[0].after[&Waist], 43.5));
    assert!(close(w.sizes[0].after[&Thigh], 34.5));
}

#[test]
fn a_shrinkage_amount_shrinks_widths_with_the_waist_and_lengths_with_the_inseam() {
    // Iron Heart UHR: about 1" off the waist and up to 2" off the inseam.
    let mut p = product("uhr", "634", vec![chart(ChartState::AsSold, Unit::In, ih_32())]);
    p.sold = Some(Sold::Raw);
    p.shrinkage = Some(fit_core::catalogue::Shrinkage::Amount {
        unit: Unit::In,
        waist: Some(1.0),
        inseam: Some(2.0),
        note: None,
        source: source(),
    });
    let w = after_wash(&brand("b", vec![]), &p).unwrap();
    assert_eq!(w.basis, Basis::FromShrinkage);
    let s = &w.sizes[0].after;
    assert!(close(s[&Waist], (31.7 - 1.0) * 2.54 / 2.0));
    assert!(close(s[&Inseam], (35.5 - 2.0) * 2.54));
    assert!(close(s[&Thigh], 12.5 * 2.54 * (1.0 - 1.0 / 31.7)));
    assert!(close(s[&FrontRise], 11.0 * 2.54 * (1.0 - 2.0 / 35.5)));
}

#[test]
fn an_unknown_measuring_point_makes_matches_approximate() {
    let p = product("p", "C", vec![chart(ChartState::AsSold, Unit::In, ih_32())]);
    let mut b = brand("b", vec![]);
    assert!(!after_wash(&b, &p).unwrap().approximate);
    b.measuring_points.knee = Some("unknown".into());
    assert!(after_wash(&b, &p).unwrap().approximate);
    b.measuring_points.knee = Some("13in-below-crotch".into());
    b.measuring_points.thigh = Some("unknown".into());
    assert!(after_wash(&b, &p).unwrap().approximate);
}

#[test]
fn an_unknown_seat_point_never_makes_matches_approximate() {
    // Matching compares the seat with the body's seat whatever the brand's
    // point, so only the thigh and knee points matter.
    let mut sizes = ih_32();
    sizes[0].measurements.insert(Seat, 20.0);
    let p = product("p", "C", vec![chart(ChartState::AsSold, Unit::In, sizes)]);
    let mut b = brand("b", vec![]);
    b.measuring_points.seat = Some("unknown".into());
    assert!(!after_wash(&b, &p).unwrap().approximate);
}
