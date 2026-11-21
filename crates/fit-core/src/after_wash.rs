//! Works out each product's after-wash measurements (ADR 0004).

use std::collections::BTreeMap;

use serde::Serialize;

use crate::catalogue::{Brand, Chart, ChartState, KneePoint, Measurement, Product, Shrinkage, Sold, ThighPoint};

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
    pub waist_tag: f64,
    pub length_tag: Option<f64>,
    pub after: BTreeMap<Measurement, f64>,
    pub as_sold: Option<BTreeMap<Measurement, f64>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Washed {
    pub basis: Basis,
    pub sizes: Vec<WashedSize>,
    /// `None` when the brand doesn't say where it takes the thigh.
    pub thigh_point: Option<ThighPoint>,
    /// `None` when the brand doesn't say where it takes the knee.
    pub knee_point: Option<KneePoint>,
    /// A measuring point this chart uses is "unknown": matches are marked
    /// approximate.
    pub approximate: bool,
}

struct CoreRow {
    label: String,
    waist_tag: f64,
    length_tag: Option<f64>,
    numbers: BTreeMap<Measurement, f64>,
}

/// A chart's sizes converted to cm, widths flat.
fn core_rows(brand: &Brand, chart: &Chart) -> Vec<CoreRow> {
    chart
        .sizes
        .iter()
        .map(|s| CoreRow {
            label: s.label.clone(),
            waist_tag: s.waist_tag,
            length_tag: s.length_tag,
            numbers: s
                .measurements
                .iter()
                .filter_map(|(m, v)| Some((*m, chart.way(brand, *m)?.to_core(chart.unit.to_cm(*v)))))
                .collect(),
        })
        .collect()
}

/// Shrinks one size's numbers by the brand's stated shrinkage. An amount is
/// turned into a share of that size's waist (around) and inseam, and the
/// other widths and lengths shrink in proportion.
fn shrink(
    numbers: &BTreeMap<Measurement, f64>,
    shrinkage: &Shrinkage,
    fallback: (f64, f64),
) -> BTreeMap<Measurement, f64> {
    let (width, length) = match shrinkage {
        Shrinkage::Percent { width, length, .. } => (width / 100.0, length / 100.0),
        Shrinkage::Amount {
            unit, waist, inseam, ..
        } => {
            let share = |amount: Option<f64>, of: Option<f64>, fb: f64| match (amount, of) {
                (Some(a), Some(v)) if v > 0.0 => unit.to_cm(a) / v,
                (Some(_), None) => fb,
                _ => 0.0,
            };
            (
                share(*waist, numbers.get(&Measurement::Waist).map(|w| w * 2.0), fallback.0),
                share(*inseam, numbers.get(&Measurement::Inseam).copied(), fallback.1),
            )
        }
    };
    numbers
        .iter()
        .map(|(m, v)| (*m, v * (1.0 - if m.is_length() { length } else { width })))
        .collect()
}

/// For an amount, the shares of the size nearest tag 32, used for sizes
/// missing their waist or inseam.
fn fallback_shares(rows: &[CoreRow], shrinkage: &Shrinkage) -> (f64, f64) {
    let Shrinkage::Amount {
        unit, waist, inseam, ..
    } = shrinkage
    else {
        return (0.0, 0.0);
    };
    let nearest = |m: Measurement| {
        rows.iter()
            .filter(|r| r.numbers.contains_key(&m))
            .min_by(|a, b| (a.waist_tag - 32.0).abs().total_cmp(&(b.waist_tag - 32.0).abs()))
            .map(|r| r.numbers[&m])
    };
    let w = match (waist, nearest(Measurement::Waist)) {
        (Some(a), Some(v)) => unit.to_cm(*a) / (v * 2.0),
        _ => 0.0,
    };
    let l = match (inseam, nearest(Measurement::Inseam)) {
        (Some(a), Some(v)) => unit.to_cm(*a) / v,
        _ => 0.0,
    };
    (w, l)
}

/// The product's sizes as they'll be after the first wash, or `None` for a
/// product with no measurements, which is left out.
pub fn after_wash(brand: &Brand, product: &Product) -> Option<Washed> {
    let as_sold = product.chart(ChartState::AsSold);
    let washed_chart = product.chart(ChartState::AfterWash);
    let main = washed_chart.or(as_sold)?;
    if main.sizes.iter().all(|s| s.measurements.is_empty()) {
        return None;
    }

    let basis = if washed_chart.is_some() {
        Basis::BrandChart
    } else if product.sold == Some(Sold::OneWashed) {
        Basis::SoldWashed
    } else if product.shrinkage.is_some() {
        Basis::FromShrinkage
    } else {
        Basis::ShrinkageUnknown
    };

    let sold_rows = as_sold.map(|c| core_rows(brand, c)).unwrap_or_default();
    let main_rows = core_rows(brand, main);
    let fallback = product
        .shrinkage
        .as_ref()
        .map(|s| fallback_shares(&main_rows, s))
        .unwrap_or((0.0, 0.0));

    let sizes = main_rows
        .into_iter()
        .filter(|r| !r.numbers.is_empty())
        .map(|r| {
            let after = match (basis, &product.shrinkage) {
                (Basis::FromShrinkage, Some(s)) => shrink(&r.numbers, s, fallback),
                _ => r.numbers.clone(),
            };
            let as_sold = match basis {
                Basis::BrandChart => sold_rows.iter().find(|s| s.label == r.label).map(|s| s.numbers.clone()),
                _ => Some(r.numbers),
            };
            WashedSize {
                size: r.label,
                waist_tag: r.waist_tag,
                length_tag: r.length_tag,
                after,
                as_sold,
            }
        })
        .collect::<Vec<_>>();

    let effective = main.effective(brand);
    let points = &effective.measuring_points;
    let published = |m: Measurement| main.sizes.iter().any(|s| s.measurements.contains_key(&m));
    let approximate = [Measurement::Thigh, Measurement::Knee, Measurement::Seat]
        .into_iter()
        .any(|m| published(m) && points.get(m).is_none_or(|p| p == "unknown"));

    Some(Washed {
        basis,
        sizes,
        thigh_point: points.thigh_point(),
        knee_point: points.knee_point(),
        approximate,
    })
}
