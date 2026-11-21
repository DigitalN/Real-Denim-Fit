//! The catalogue as it's written in `catalogue/brands/*.json`.
//!
//! Every number sits in a **chart**, and the chart carries what each of its
//! numbers needs: the source link, the date it was seen, the unit, the brand's
//! tolerance and whether it was read from an image. Each measurement in the
//! chart has its measuring point and whether it's flat, around or a length.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::units::{Quantity, Unit, Way};

/// The eight garment measurements, in the glossary's words.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Measurement {
    Waist,
    FrontRise,
    BackRise,
    Seat,
    Thigh,
    Knee,
    LegOpening,
    Inseam,
}

impl Measurement {
    pub const ALL: [Measurement; 8] = [
        Measurement::Waist,
        Measurement::FrontRise,
        Measurement::BackRise,
        Measurement::Seat,
        Measurement::Thigh,
        Measurement::Knee,
        Measurement::LegOpening,
        Measurement::Inseam,
    ];

    /// Widths shrink with the fabric's width; lengths with its length.
    pub fn is_length(self) -> bool {
        matches!(
            self,
            Measurement::FrontRise | Measurement::BackRise | Measurement::Inseam
        )
    }

    pub fn label(self) -> &'static str {
        match self {
            Measurement::Waist => "waist",
            Measurement::FrontRise => "front rise",
            Measurement::BackRise => "back rise",
            Measurement::Seat => "seat",
            Measurement::Thigh => "thigh",
            Measurement::Knee => "knee",
            Measurement::LegOpening => "leg opening",
            Measurement::Inseam => "inseam",
        }
    }
}

/// Where a brand takes its thigh measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ThighPoint {
    #[serde(rename = "crotch")]
    Crotch,
    #[serde(rename = "1in-below-crotch")]
    OneInchBelow,
}

/// Where a brand takes its knee measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum KneePoint {
    #[serde(rename = "12in-below-crotch")]
    Below12,
    #[serde(rename = "13in-below-crotch")]
    Below13,
    #[serde(rename = "14in-below-crotch")]
    Below14,
    #[serde(rename = "half-inseam")]
    HalfInseam,
}

/// The measuring point of one measurement in a chart.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub way: Way,
    /// For the thigh and knee this is one of the codes the matching uses
    /// (`crotch`, `1in-below-crotch`, `12in-below-crotch`, `13in-below-crotch`,
    /// `14in-below-crotch`, `half-inseam`) or `not-stated`. For the others
    /// it's the brand's own description.
    pub at: String,
}

impl Point {
    pub fn thigh_point(&self) -> Option<ThighPoint> {
        serde_json::from_value(serde_json::Value::String(self.at.clone())).ok()
    }
    pub fn knee_point(&self) -> Option<KneePoint> {
        serde_json::from_value(serde_json::Value::String(self.at.clone())).ok()
    }
}

/// How far a brand says its numbers may be off.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Tolerance {
    Stated(Quantity),
    /// Always the text `not stated`.
    NotStated(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChartState {
    AsSold,
    AfterWash,
}

/// One size's row in a chart. Numbers are in the chart's unit and way.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SizeRow {
    /// The size as the brand names it, such as `32` or `W32 L34`.
    pub size: String,
    /// The waist number on the tag.
    pub tag_waist: f64,
    /// The length on the tag, for brands sold in lengths.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag_length: Option<f64>,
    #[serde(flatten)]
    pub numbers: BTreeMap<Measurement, f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chart {
    pub state: ChartState,
    pub source: String,
    /// The date the chart was seen, as YYYY-MM-DD.
    pub seen: String,
    pub unit: Unit,
    pub tolerance: Tolerance,
    pub from_image: bool,
    pub points: BTreeMap<Measurement, Point>,
    pub sizes: Vec<SizeRow>,
}

/// What a brand says about how much a product shrinks in its first wash.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Shrinkage {
    /// Applies to the waist, seat, thigh, knee and leg opening.
    pub width_percent: f64,
    /// Applies to the rises and inseam.
    pub length_percent: f64,
    /// The brand's own words, such as "1–3%".
    pub stated: String,
    pub source: String,
    pub seen: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListPrice {
    pub amount: f64,
    pub currency: String,
    pub source: String,
    pub seen: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub cut: String,
    /// The brand's own name for the fit, such as "Slim Tapered". Never used
    /// in matching (ADR 0008); only in the ease-table calibration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brand_fit_label: Option<String>,
    pub name: String,
    pub fabric: String,
    pub fabric_weight_oz: Option<f64>,
    pub composition: String,
    pub elastane_percent: f64,
    /// `null` when the brand doesn't say.
    pub sanforized: Option<bool>,
    /// Sold already washed, so the chart is how it will be after a wash.
    pub sold_washed: bool,
    /// `null` when the brand doesn't give a figure.
    pub shrinkage: Option<Shrinkage>,
    pub list_price: Option<ListPrice>,
    pub url: String,
    /// False once the product has gone from the brand's site.
    pub on_brands_site: bool,
    pub charts: Vec<Chart>,
}

impl Product {
    pub fn is_stretch(&self) -> bool {
        self.elastane_percent > 0.0
    }
    pub fn chart(&self, state: ChartState) -> Option<&Chart> {
        self.charts.iter().find(|c| c.state == state)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Brand {
    pub id: String,
    pub name: String,
    pub site: String,
    pub measuring_guide: Option<String>,
    /// False for a brand that doesn't say where it measures; its matches are
    /// marked approximate.
    pub says_where_it_measures: bool,
    /// Sold in a waist × length grid, such as Nudie's W32 L30/L32/L34.
    pub sold_in_lengths: bool,
    /// Notes for the maintainer, never shown in matching.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    pub products: Vec<Product>,
}

/// Fixed rates from the European Central Bank, euros as the base.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExchangeRates {
    pub source: String,
    /// The ECB reference date of the rates.
    pub date: String,
    pub base: String,
    /// Units of each currency per euro.
    pub rates: BTreeMap<String, f64>,
}

impl ExchangeRates {
    /// Converts an amount between currencies, or `None` when either rate is
    /// missing.
    pub fn convert(&self, amount: f64, from: &str, to: &str) -> Option<f64> {
        let rate = |c: &str| {
            if c == self.base {
                Some(1.0)
            } else {
                self.rates.get(c).copied()
            }
        };
        Some(amount / rate(from)? * rate(to)?)
    }

    pub fn currencies(&self) -> Vec<String> {
        let mut all: Vec<String> = self.rates.keys().cloned().collect();
        all.push(self.base.clone());
        all.sort();
        all
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Catalogue {
    pub brands: Vec<Brand>,
    pub exchange_rates: ExchangeRates,
}

impl Catalogue {
    pub fn product(&self, id: &str) -> Option<(&Brand, &Product)> {
        self.brands
            .iter()
            .flat_map(|b| b.products.iter().map(move |p| (b, p)))
            .find(|(_, p)| p.id == id)
    }
}

/// Quantity in a chart's unit, for convenience in tests and tools.
pub fn quantity(value: f64, unit: Unit) -> Quantity {
    Quantity { value, unit }
}
