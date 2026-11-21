//! Small builders so each test can say only what it's about.
#![allow(dead_code)]

use std::collections::BTreeMap;

use fit_core::catalogue::*;
use fit_core::profile::*;
use fit_core::units::{Quantity, Unit};

pub const SEEN: &str = "2026-11-20";

pub fn source() -> Source {
    Source {
        url: "https://example.com/chart".into(),
        seen: SEEN.into(),
    }
}

pub fn row(label: &str, waist_tag: f64, numbers: &[(Measurement, f64)]) -> Size {
    Size {
        label: label.to_string(),
        waist_tag,
        length_tag: None,
        measurements: numbers.iter().copied().collect(),
    }
}

/// A chart that inherits everything from its brand.
pub fn chart(state: ChartState, unit: Unit, sizes: Vec<Size>) -> Chart {
    Chart {
        state,
        estimated_by_brand: false,
        source: source(),
        unit,
        from_image: false,
        conventions: None,
        measuring_points: None,
        tolerance: None,
        sizes,
    }
}

/// A product with the cut it belongs to. Derefs to the product so tests
/// can change its fields directly.
#[derive(Clone)]
pub struct TestProduct {
    pub cut: String,
    pub product: Product,
}

impl std::ops::Deref for TestProduct {
    type Target = Product;
    fn deref(&self) -> &Product {
        &self.product
    }
}
impl std::ops::DerefMut for TestProduct {
    fn deref_mut(&mut self) -> &mut Product {
        &mut self.product
    }
}

/// A rigid, sanforized, one-washed product in USD.
pub fn product(id: &str, cut: &str, charts: Vec<Chart>) -> TestProduct {
    TestProduct {
        cut: cut.into(),
        product: Product {
            id: id.into(),
            name: format!("{cut} test"),
            url: "https://example.com/p".into(),
            status: Status::Current,
            status_seen: SEEN.into(),
            fabric: Fabric {
                weight_oz: Some(14.0),
                composition: Some(vec![Fibre {
                    fibre: "cotton".into(),
                    percent: 100.0,
                }]),
                sanforized: Some(true),
            },
            sold: Some(Sold::OneWashed),
            list_price: Some(ListPrice {
                amount: 200.0,
                currency: "USD".into(),
                url: "https://example.com/p".into(),
                seen: SEEN.into(),
            }),
            shrinkage: None,
            charts,
        },
    }
}

pub fn stretch(elastane: f64) -> Option<Vec<Fibre>> {
    Some(vec![
        Fibre {
            fibre: "cotton".into(),
            percent: 100.0 - elastane,
        },
        Fibre {
            fibre: "elastane".into(),
            percent: elastane,
        },
    ])
}

pub fn percent_shrinkage(width: f64, length: f64) -> Shrinkage {
    Shrinkage::Percent {
        length,
        width,
        note: None,
        source: source(),
    }
}

/// A brand measuring the Iron Heart way: waist around, other widths flat,
/// thigh at the crotch, knee 13" below, seat at the fullest point.
pub fn brand(id: &str, products: Vec<TestProduct>) -> Brand {
    let mut cuts: Vec<Cut> = vec![];
    for tp in products {
        let cut_id = tp.cut.to_lowercase().replace(' ', "-");
        match cuts.iter_mut().find(|c| c.id == cut_id) {
            Some(c) => c.products.push(tp.product),
            None => cuts.push(Cut {
                id: cut_id,
                brand: id.into(),
                name: tp.cut.clone(),
                brand_label: None,
                products: vec![tp.product],
            }),
        }
    }
    Brand {
        id: id.into(),
        name: id.into(),
        site: "https://example.com".into(),
        currency: "USD".into(),
        sizing: Sizing::Waist,
        tolerance: Some(Quantity {
            value: 0.5,
            unit: Unit::In,
        }),
        conventions: BTreeMap::from([
            (Measurement::Waist, Convention::Around),
            (Measurement::Seat, Convention::Flat),
            (Measurement::Thigh, Convention::Flat),
            (Measurement::Knee, Convention::Flat),
            (Measurement::LegOpening, Convention::Flat),
        ]),
        measuring_points: MeasuringPoints {
            thigh: Some("at-crotch".into()),
            knee: Some("13in-below-crotch".into()),
            seat: Some("fullest".into()),
        },
        method_source: source(),
        cuts,
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
    let r = |from: f64, to: f64| format!(r#"{{"from":{from},"to":{to},"quality":"best estimate","sources":["t"]}}"#);
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
