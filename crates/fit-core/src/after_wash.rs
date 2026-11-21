//! Works out each product's after-wash measurements (ADR 0004).

use std::collections::BTreeMap;

use serde::Serialize;

use crate::catalogue::{Chart, ChartState, KneePoint, Measurement, Product, ThighPoint};

/// Where a product's after-wash numbers came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Basis {
    /// The brand publishes an after-wash chart.
    BrandChart,
    /// Sold already washed, so the chart is used as it is.
    SoldWashed,
    /// Estimated from the brand's shrinkage.
    FromShrinkage,
    /// Will shrink by an amount the brand doesn't give. Ranked after the rest.
    ShrinkageUnknown,
}

/// One size in the core's form: cm, widths flat.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WashedSize {
    pub size: String,
    pub tag_waist: f64,
    pub tag_length: Option<f64>,
    pub after: BTreeMap<Measurement, f64>,
    pub as_sold: Option<BTreeMap<Measurement, f64>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Washed {
    pub basis: Basis,
    pub sizes: Vec<WashedSize>,
    /// `None` when the chart doesn't say where the thigh is taken.
    pub thigh_point: Option<ThighPoint>,
    /// `None` when the chart doesn't say where the knee is taken.
    pub knee_point: Option<KneePoint>,
}

/// A chart's rows converted to cm, widths flat.
fn core_rows(chart: &Chart) -> Vec<(String, f64, Option<f64>, BTreeMap<Measurement, f64>)> {
    chart
        .sizes
        .iter()
        .map(|row| {
            let numbers = row
                .numbers
                .iter()
                .filter_map(|(m, v)| {
                    let way = chart.points.get(m)?.way;
                    Some((*m, way.to_core(chart.unit.to_cm(*v))))
                })
                .collect();
            (row.size.clone(), row.tag_waist, row.tag_length, numbers)
        })
        .collect()
}

/// The product's sizes as they'll be after the first wash, or `None` for a
/// product with no measurements, which is left out.
pub fn after_wash(product: &Product) -> Option<Washed> {
    let as_sold = product.chart(ChartState::AsSold);
    let washed_chart = product.chart(ChartState::AfterWash);
    let main = washed_chart.or(as_sold)?;
    if main.sizes.iter().all(|r| r.numbers.is_empty()) {
        return None;
    }

    let basis = if washed_chart.is_some() {
        Basis::BrandChart
    } else if product.sold_washed {
        Basis::SoldWashed
    } else if product.shrinkage.is_some() {
        Basis::FromShrinkage
    } else {
        Basis::ShrinkageUnknown
    };

    let sold_rows = as_sold.map(core_rows).unwrap_or_default();
    let sold_for = |size: &str| {
        sold_rows
            .iter()
            .find(|(s, ..)| s == size)
            .map(|(.., n)| n.clone())
    };

    let sizes = core_rows(main)
        .into_iter()
        .filter(|(.., n)| !n.is_empty())
        .map(|(size, tag_waist, tag_length, numbers)| {
            let after = match (basis, &product.shrinkage) {
                (Basis::FromShrinkage, Some(s)) => numbers
                    .iter()
                    .map(|(m, v)| {
                        let pct = if m.is_length() {
                            s.length_percent
                        } else {
                            s.width_percent
                        };
                        (*m, v * (1.0 - pct / 100.0))
                    })
                    .collect(),
                _ => numbers.clone(),
            };
            let as_sold = match basis {
                Basis::BrandChart => sold_for(&size),
                _ => Some(numbers),
            };
            WashedSize {
                size,
                tag_waist,
                tag_length,
                after,
                as_sold,
            }
        })
        .collect();

    Some(Washed {
        basis,
        sizes,
        thigh_point: main
            .points
            .get(&Measurement::Thigh)
            .and_then(|p| p.thigh_point()),
        knee_point: main
            .points
            .get(&Measurement::Knee)
            .and_then(|p| p.knee_point()),
    })
}
