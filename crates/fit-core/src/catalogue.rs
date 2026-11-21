//! The catalogue, exactly as `docs/catalogue-format.md` describes it:
//! `catalogue/<brand>/brand.json` plus one file per cut, and
//! `catalogue/exchange-rates.json`.
//!
//! Numbers are kept as the brand publishes them, in its unit and
//! convention. Charts inherit their measuring points, conventions and
//! tolerance from the brand unless they override them.

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

    /// Rises and the inseam are lengths; the rest are widths with a
    /// convention (flat or around).
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

/// Flat or all the way around, for a width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Convention {
    Flat,
    Around,
}

impl Convention {
    pub fn way(self) -> Way {
        match self {
            Convention::Flat => Way::Flat,
            Convention::Around => Way::Around,
        }
    }
}

/// Where a brand takes its thigh measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ThighPoint {
    #[serde(rename = "at-crotch")]
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

/// The measuring points a brand or chart states, as written: `thigh`,
/// `knee` and `seat`, each a point code or `"unknown"`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeasuringPoints {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thigh: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub knee: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seat: Option<String>,
}

pub const THIGH_POINTS: [&str; 3] = ["at-crotch", "1in-below-crotch", "unknown"];
pub const KNEE_POINTS: [&str; 5] = [
    "12in-below-crotch",
    "13in-below-crotch",
    "14in-below-crotch",
    "half-inseam",
    "unknown",
];
pub const SEAT_POINTS: [&str; 3] = ["fullest", "16cm-below-waistband", "unknown"];

impl MeasuringPoints {
    pub fn thigh_point(&self) -> Option<ThighPoint> {
        serde_json::from_value(serde_json::Value::String(self.thigh.clone()?)).ok()
    }
    pub fn knee_point(&self) -> Option<KneePoint> {
        serde_json::from_value(serde_json::Value::String(self.knee.clone()?)).ok()
    }
    /// The point for a measurement, as written.
    pub fn get(&self, m: Measurement) -> Option<&str> {
        match m {
            Measurement::Thigh => self.thigh.as_deref(),
            Measurement::Knee => self.knee.as_deref(),
            Measurement::Seat => self.seat.as_deref(),
            _ => None,
        }
    }
}

/// A link and the date it was seen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Source {
    pub url: String,
    pub seen: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Sizing {
    Waist,
    /// Sold in a waist × length grid, such as Nudie's W32 L30/L32/L34.
    WaistAndLength,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Brand {
    pub id: String,
    pub name: String,
    pub site: String,
    pub currency: String,
    pub sizing: Sizing,
    /// `None` when the brand doesn't state one.
    pub tolerance: Option<Quantity>,
    pub conventions: BTreeMap<Measurement, Convention>,
    pub measuring_points: MeasuringPoints,
    pub method_source: Source,
    /// The cut files in the brand's folder.
    #[serde(default, skip_deserializing)]
    pub cuts: Vec<Cut>,
}

impl Brand {
    pub fn sold_in_lengths(&self) -> bool {
        self.sizing == Sizing::WaistAndLength
    }
    pub fn products(&self) -> impl Iterator<Item = (&Cut, &Product)> {
        self.cuts.iter().flat_map(|c| c.products.iter().map(move |p| (c, p)))
    }
}

/// Whether a brand files a cut as men's, women's or unisex. Shown and
/// filtered on; matching ignores it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Line {
    Men,
    Women,
    Unisex,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cut {
    pub id: String,
    pub brand: String,
    pub name: String,
    /// The brand's own name for the fit. Shown, never used in matching
    /// (ADR 0008); the ease-table calibration reads it.
    pub brand_label: Option<String>,
    /// `None` when the brand doesn't say.
    #[serde(default)]
    pub line: Option<Line>,
    pub products: Vec<Product>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    Current,
    /// Gone from the brand's site; kept and marked so.
    Discontinued,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Sold {
    /// Unwashed.
    Raw,
    /// Already washed by the brand.
    OneWashed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fibre {
    pub fibre: String,
    pub percent: f64,
}

/// Fibres that make a fabric stretch. Brands write elastane as
/// polyurethane, spandex or Lycra too.
const STRETCH_FIBRES: [&str; 5] = ["elastane", "polyurethane", "spandex", "lycra", "pu"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fabric {
    pub weight_oz: Option<f64>,
    pub composition: Option<Vec<Fibre>>,
    pub sanforized: Option<bool>,
}

impl Fabric {
    /// The percentage of stretch fibre, 0 for rigid or unknown.
    pub fn elastane_percent(&self) -> f64 {
        self.composition
            .iter()
            .flatten()
            .filter(|f| STRETCH_FIBRES.contains(&f.fibre.trim().to_ascii_lowercase().as_str()))
            .map(|f| f.percent)
            .sum()
    }
}

/// What a brand says about how much a product shrinks in its first wash.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Shrinkage {
    /// Percentages: `length` for the rises and inseam, `width` for the rest.
    Percent {
        length: f64,
        width: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        note: Option<String>,
        source: Source,
    },
    /// Amounts off the waist (all the way around) and the inseam. Other
    /// widths shrink in proportion to the waist, rises to the inseam.
    Amount {
        unit: Unit,
        #[serde(default)]
        waist: Option<f64>,
        #[serde(default)]
        inseam: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        note: Option<String>,
        source: Source,
    },
}

impl Shrinkage {
    pub fn source(&self) -> &Source {
        match self {
            Shrinkage::Percent { source, .. } | Shrinkage::Amount { source, .. } => source,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListPrice {
    pub amount: f64,
    pub currency: String,
    pub url: String,
    pub seen: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub url: String,
    pub status: Status,
    pub status_seen: String,
    pub fabric: Fabric,
    pub sold: Option<Sold>,
    pub list_price: Option<ListPrice>,
    pub shrinkage: Option<Shrinkage>,
    pub charts: Vec<Chart>,
}

impl Product {
    pub fn is_stretch(&self) -> bool {
        self.fabric.elastane_percent() > 0.0
    }
    pub fn is_current(&self) -> bool {
        self.status == Status::Current
    }
    pub fn chart(&self, state: ChartState) -> Option<&Chart> {
        self.charts.iter().find(|c| c.state == state)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChartState {
    AsSold,
    AfterWash,
}

/// One size's row in a chart. Numbers are in the chart's unit and the
/// effective conventions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Size {
    /// The brand's own text, such as `32` or `W32 L32`.
    pub label: String,
    pub waist_tag: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub length_tag: Option<f64>,
    pub measurements: BTreeMap<Measurement, f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Chart {
    pub state: ChartState,
    /// The brand calls this after-wash chart an estimate.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub estimated_by_brand: bool,
    pub source: Source,
    pub unit: Unit,
    pub from_image: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conventions: Option<BTreeMap<Measurement, Convention>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub measuring_points: Option<MeasuringPoints>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tolerance: Option<Quantity>,
    pub sizes: Vec<Size>,
}

/// A chart with what it inherits from its brand filled in.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Effective {
    pub conventions: BTreeMap<Measurement, Convention>,
    pub measuring_points: MeasuringPoints,
    pub tolerance: Option<Quantity>,
}

impl Chart {
    pub fn effective(&self, brand: &Brand) -> Effective {
        let mut conventions = brand.conventions.clone();
        if let Some(over) = &self.conventions {
            conventions.extend(over.iter().map(|(m, c)| (*m, *c)));
        }
        let mut points = brand.measuring_points.clone();
        if let Some(over) = &self.measuring_points {
            if over.thigh.is_some() {
                points.thigh = over.thigh.clone();
            }
            if over.knee.is_some() {
                points.knee = over.knee.clone();
            }
            if over.seat.is_some() {
                points.seat = over.seat.clone();
            }
        }
        Effective {
            conventions,
            measuring_points: points,
            tolerance: self.tolerance.or(brand.tolerance),
        }
    }

    /// How a measurement in this chart was taken.
    pub fn way(&self, brand: &Brand, m: Measurement) -> Option<Way> {
        if m.is_length() {
            return Some(Way::Length);
        }
        self.conventions
            .as_ref()
            .and_then(|c| c.get(&m))
            .or(brand.conventions.get(&m))
            .map(|c| c.way())
    }
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

/// A product's key across the catalogue: `<brand>/<product>`.
pub fn product_key(brand: &str, product: &str) -> String {
    format!("{brand}/{product}")
}

impl Catalogue {
    /// Finds a product by its key, `<brand>/<product>`.
    pub fn product(&self, key: &str) -> Option<(&Brand, &Cut, &Product)> {
        let (brand_id, product_id) = key.split_once('/')?;
        let brand = self.brands.iter().find(|b| b.id == brand_id)?;
        brand
            .products()
            .find(|(_, p)| p.id == product_id)
            .map(|(c, p)| (brand, c, p))
    }

    /// Builds a catalogue from files: `(path, text)` pairs with paths like
    /// `iron-heart/brand.json`, `iron-heart/634.json` and
    /// `exchange-rates.json`, relative to `catalogue/`.
    pub fn from_files(files: &[(String, String)]) -> Result<Catalogue, String> {
        let mut rates = None;
        let mut brands: BTreeMap<String, Brand> = BTreeMap::new();
        let mut cuts: BTreeMap<String, Vec<Cut>> = BTreeMap::new();
        for (path, text) in files {
            let path = path.replace('\\', "/");
            let parts: Vec<&str> = path.split('/').collect();
            match parts.as_slice() {
                ["exchange-rates.json"] => {
                    rates = Some(serde_json::from_str(text).map_err(|e| format!("{path}: {e}"))?)
                }
                [folder, "brand.json"] => {
                    let b: Brand = serde_json::from_str(text).map_err(|e| format!("{path}: {e}"))?;
                    if b.id != *folder {
                        return Err(format!("{path}: id {:?} doesn't match its folder", b.id));
                    }
                    brands.insert(folder.to_string(), b);
                }
                [folder, file] if file.ends_with(".json") => {
                    let c: Cut = serde_json::from_str(text).map_err(|e| format!("{path}: {e}"))?;
                    if c.brand != *folder {
                        return Err(format!("{path}: brand {:?} doesn't match its folder", c.brand));
                    }
                    cuts.entry(folder.to_string()).or_default().push(c);
                }
                _ => {}
            }
        }
        for (folder, list) in cuts {
            let brand = brands
                .get_mut(&folder)
                .ok_or_else(|| format!("{folder}/ has cut files but no brand.json"))?;
            brand.cuts = list;
            brand.cuts.sort_by(|a, b| a.name.cmp(&b.name));
        }
        let mut brands: Vec<Brand> = brands.into_values().collect();
        brands.sort_by_key(|b| b.name.to_lowercase());
        Ok(Catalogue {
            brands,
            exchange_rates: rates.ok_or("exchange-rates.json is missing")?,
        })
    }

    /// Reads `catalogue/` from disk.
    pub fn from_dir(dir: &std::path::Path) -> Result<Catalogue, String> {
        let mut files = vec![];
        let read = |p: &std::path::Path| std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()));
        for entry in std::fs::read_dir(dir)
            .map_err(|e| format!("{}: {e}", dir.display()))?
            .flatten()
        {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                for f in std::fs::read_dir(&path).map_err(|e| e.to_string())?.flatten() {
                    let fname = f.file_name().to_string_lossy().to_string();
                    if fname.ends_with(".json") {
                        files.push((format!("{name}/{fname}"), read(&f.path())?));
                    }
                }
            } else if name == "exchange-rates.json" {
                files.push((name, read(&path)?));
            }
        }
        Catalogue::from_files(&files)
    }
}
