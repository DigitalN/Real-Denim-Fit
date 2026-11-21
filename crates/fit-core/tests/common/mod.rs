//! Small builders so each test can say only what it's about.
#![allow(dead_code)]

use std::collections::BTreeMap;

use fit_core::catalogue::*;
use fit_core::profile::*;
use fit_core::units::{Quantity, Unit, Way};

pub const SEEN: &str = "2026-11-20";

pub fn point(way: Way, at: &str) -> Point {
    Point {
        way,
        at: at.to_string(),
    }
}

/// The points Iron Heart uses: waist around, thigh at the crotch, knee 13"
/// below, everything else flat or a length.
pub fn iron_heart_points() -> BTreeMap<Measurement, Point> {
    BTreeMap::from([
        (
            Measurement::Waist,
            point(Way::Around, "top of the waistband"),
        ),
        (
            Measurement::FrontRise,
            point(Way::Length, "crotch seam to top of waistband"),
        ),
        (
            Measurement::BackRise,
            point(Way::Length, "crotch seam to top of waistband"),
        ),
        (
            Measurement::Seat,
            point(Way::Flat, "widest part of the seat"),
        ),
        (Measurement::Thigh, point(Way::Flat, "crotch")),
        (Measurement::Knee, point(Way::Flat, "13in-below-crotch")),
        (
            Measurement::LegOpening,
            point(Way::Flat, "bottom of the leg"),
        ),
        (
            Measurement::Inseam,
            point(Way::Length, "crotch seam to hem"),
        ),
    ])
}

pub fn row(size: &str, tag_waist: f64, numbers: &[(Measurement, f64)]) -> SizeRow {
    SizeRow {
        size: size.to_string(),
        tag_waist,
        tag_length: None,
        numbers: numbers.iter().copied().collect(),
    }
}

pub fn chart(state: ChartState, unit: Unit, sizes: Vec<SizeRow>) -> Chart {
    Chart {
        state,
        source: "https://example.com/chart".into(),
        seen: SEEN.into(),
        unit,
        tolerance: Tolerance::Stated(Quantity {
            value: 0.5,
            unit: Unit::In,
        }),
        from_image: false,
        points: iron_heart_points(),
        sizes,
    }
}

pub fn product(id: &str, cut: &str, charts: Vec<Chart>) -> Product {
    Product {
        id: id.into(),
        cut: cut.into(),
        brand_fit_label: None,
        name: format!("{cut} test"),
        fabric: "14oz selvedge".into(),
        fabric_weight_oz: Some(14.0),
        composition: "100% cotton".into(),
        elastane_percent: 0.0,
        sanforized: Some(true),
        sold_washed: true,
        shrinkage: None,
        list_price: Some(ListPrice {
            amount: 200.0,
            currency: "USD".into(),
            source: "https://example.com/p".into(),
            seen: SEEN.into(),
        }),
        url: "https://example.com/p".into(),
        on_brands_site: true,
        charts,
    }
}

pub fn brand(id: &str, products: Vec<Product>) -> Brand {
    Brand {
        id: id.into(),
        name: id.into(),
        site: "https://example.com".into(),
        measuring_guide: None,
        says_where_it_measures: true,
        sold_in_lengths: false,
        notes: vec![],
        products,
    }
}

pub fn rates() -> ExchangeRates {
    ExchangeRates {
        source: "https://www.ecb.europa.eu/stats/eurofxref/eurofxref-daily.xml".into(),
        date: SEEN.into(),
        base: "EUR".into(),
        rates: BTreeMap::from([("USD".into(), 1.25), ("GBP".into(), 0.8)]),
    }
}

pub fn catalogue(brands: Vec<Brand>) -> Catalogue {
    Catalogue {
        brands,
        exchange_rates: rates(),
    }
}

/// A size-32 body close to Levi's 32, in cm around.
pub fn body() -> Body {
    Body {
        waist: 81.3,
        seat: 97.8,
        thigh_top: 57.2,
        thigh_1in: 55.0,
        leg_12in: 41.0,
        leg_13in: 40.0,
        leg_14in: 39.0,
        leg_half_inseam: 39.5,
        inseam_wanted: 81.0,
    }
}

pub fn profile(room: Room) -> Profile {
    Profile {
        body: body(),
        desired_fit: DesiredFit {
            room,
            leg_shape: None,
            rise: None,
            doesnt_matter: vec![],
        },
        reference_pair: None,
    }
}

pub fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

/// A test-owned ease table with the plan's starting numbers, so tests don't
/// move when the real table is calibrated.
pub fn ease() -> fit_core::ease::EaseTable {
    let r = |from: f64, to: f64| {
        format!(r#"{{"from":{from},"to":{to},"quality":"best estimate","sources":["t"]}}"#)
    };
    let m = |v: f64| format!(r#"{{"value":{v},"quality":"well sourced","sources":["t"]}}"#);
    let point = |s: (f64, f64), g: (f64, f64), o: (f64, f64), min: f64| {
        format!(
            r#"{{"snug":{},"regular":{},"roomy":{},"minimum":{}}}"#,
            r(s.0, s.1),
            r(g.0, g.1),
            r(o.0, o.1),
            m(min)
        )
    };
    let json = format!(
        r#"{{"about":"test","unit":"in","way":"flat","points":{{
            "waist":{},"seat":{},"thigh":{},"knee":{}}},
            "stretch":{{"about":"t","range_shift":{{"value":-1.0,"unit":"cm"}},"minimum":{{"value":0.0,"unit":"cm"}},"quality":"best estimate","sources":["t"]}},
            "sources":{{"t":"test"}}}}"#,
        point((0.2, 0.6), (0.2, 0.6), (0.2, 0.6), 0.2),
        point((0.8, 1.2), (1.2, 1.8), (1.8, 2.8), 0.7),
        point((0.6, 1.0), (1.0, 1.6), (1.6, 2.4), 0.2),
        point((0.8, 1.4), (1.2, 2.0), (2.0, 3.0), 0.3),
    );
    fit_core::ease::EaseTable::from_json(&json).unwrap()
}

pub const IN: f64 = 2.54;
