//! Finds matches across the catalogue: one row per cut, closest first.

use std::cmp::Ordering;
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::after_wash::{after_wash, Basis, Washed, WashedSize};
use crate::catalogue::{product_key, Brand, Catalogue, Cut, KneePoint, Line, Measurement, Product, ThighPoint};
use crate::classify::{leg_shape, rises, RiseEntry};
use crate::closeness::closeness;
use crate::ease::EaseTable;
use crate::profile::{LegShape, PairReadings, PairSource, Profile, ReferencePair, Rise};
use crate::targets::{targets, Points, TargetFor};

/// A cut and, for each of its products, that product's matching sizes.
type CutMatches<'a> = (&'a Cut, Vec<Vec<Match>>);

/// The filters on the results screen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Filters {
    /// Brand ids to keep; `None` keeps them all.
    pub brands: Option<Vec<String>>,
    /// Your currency: prices are converted to it.
    pub currency: String,
    pub price_min: Option<f64>,
    pub price_max: Option<f64>,
    pub try_on_only: bool,
    pub try_on_brands: Vec<String>,
    /// Show men's cuts. Unisex cuts show while either line is ticked;
    /// cuts whose line isn't stated always show.
    pub mens: bool,
    pub womens: bool,
    /// Ounces per square yard.
    pub weight_min: Option<f64>,
    pub weight_max: Option<f64>,
    pub sanforized_only: bool,
    pub no_stretch: bool,
    pub current_only: bool,
    /// Show the leg shapes and rises you didn't choose too.
    pub show_other_shapes_and_rises: bool,
}

impl Default for Filters {
    fn default() -> Self {
        Filters {
            brands: None,
            currency: "USD".into(),
            price_min: None,
            price_max: None,
            try_on_only: false,
            try_on_brands: vec![],
            mens: true,
            womens: true,
            weight_min: None,
            weight_max: None,
            sanforized_only: false,
            no_stretch: false,
            current_only: true,
            show_other_shapes_and_rises: false,
        }
    }
}

/// One measurement of a match against its target.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MeasurementResult {
    pub measurement: Measurement,
    /// After-wash, cm, widths flat. `None` when the brand doesn't publish it.
    pub value: Option<f64>,
    pub target: Option<TargetFor>,
    pub closeness: Option<f64>,
    /// Value minus the target's ideal, cm.
    pub off_by: Option<f64>,
}

/// A product in one size and how close it comes.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Match {
    pub brand_id: String,
    pub cut_id: String,
    /// `<brand>/<product>`.
    pub product_id: String,
    pub size: String,
    /// The worst measurement's closeness.
    pub closeness: f64,
    pub average: f64,
    /// How many measurements were compared.
    pub compared: usize,
    pub measurements: Vec<MeasurementResult>,
    pub leg_shape: Option<LegShape>,
    pub rise: Option<Rise>,
    pub fits_shape_and_rise: bool,
    pub basis: Basis,
    pub approximate: bool,
    pub stretch: bool,
    pub on_brands_site: bool,
    /// The list price in your currency, about.
    pub price: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CutRow {
    pub brand_id: String,
    pub brand_name: String,
    pub cut_id: String,
    pub cut: String,
    /// The brand's own name for the fit, shown but never matched on.
    pub brand_label: Option<String>,
    pub line: Option<Line>,
    pub best: Match,
    /// The cut's other products, each at its closest size, closest first.
    pub other_products: Vec<Match>,
    /// The best product's other sizes, closest first.
    pub other_sizes: Vec<Match>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Results {
    pub rows: Vec<CutRow>,
    /// Rows hidden because their leg shape or rise isn't the one you chose.
    pub hidden_by_shape_or_rise: usize,
    /// No reference pair: matched from the body and typical ease.
    pub from_body_only: bool,
}

/// Every product's after-wash sizes, with each size's rise across the
/// whole catalogue.
pub struct Prepared<'a> {
    pub items: Vec<PreparedProduct<'a>>,
}

pub struct PreparedProduct<'a> {
    pub brand: &'a Brand,
    pub cut: &'a Cut,
    pub product: &'a Product,
    pub washed: Washed,
    /// The rise of each of `washed.sizes`.
    pub rises: Vec<Option<Rise>>,
}

/// Works out after-wash measurements and rises for the whole catalogue.
pub fn prepare(catalogue: &Catalogue) -> Prepared<'_> {
    let mut items: Vec<PreparedProduct> = catalogue
        .brands
        .iter()
        .flat_map(|b| b.products().map(move |(c, p)| (b, c, p)))
        .filter_map(|(brand, cut, product)| {
            let washed = after_wash(brand, product)?;
            Some(PreparedProduct {
                brand,
                cut,
                product,
                rises: vec![None; washed.sizes.len()],
                washed,
            })
        })
        .collect();
    let entries: Vec<RiseEntry> = items
        .iter()
        .enumerate()
        .flat_map(|(i, it)| {
            it.washed.sizes.iter().map(move |s| RiseEntry {
                product: i,
                waist: s.after.get(&Measurement::Waist).copied(),
                front_rise: s.after.get(&Measurement::FrontRise).copied(),
            })
        })
        .collect();
    let mut all = rises(&entries).into_iter();
    for it in &mut items {
        for r in it.rises.iter_mut() {
            *r = all.next().flatten();
        }
    }
    Prepared { items }
}

/// The readings a reference pair brings, resolving a pair picked from the
/// catalogue to that size's after-wash measurements.
pub fn pair_readings(pair: &ReferencePair, catalogue: &Catalogue) -> Option<PairReadings> {
    match &pair.source {
        PairSource::Measured { readings } => Some(readings.clone()),
        PairSource::FromCatalogue { product_id, size } => {
            let (brand, _, product) = catalogue.product(product_id)?;
            let washed = after_wash(brand, product)?;
            let s = washed.sizes.iter().find(|s| &s.size == size)?;
            let points = Points::resolve(washed.thigh_point, washed.knee_point, washed.approximate);
            Some(readings_from_size(s, points))
        }
    }
}

fn readings_from_size(s: &WashedSize, points: Points) -> PairReadings {
    let get = |m: Measurement| s.after.get(&m).copied();
    let mut r = PairReadings {
        waist: get(Measurement::Waist),
        front_rise: get(Measurement::FrontRise),
        back_rise: get(Measurement::BackRise),
        seat: get(Measurement::Seat),
        leg_opening: get(Measurement::LegOpening),
        inseam: get(Measurement::Inseam),
        ..Default::default()
    };
    match points.thigh {
        ThighPoint::Crotch => r.thigh_crotch = get(Measurement::Thigh),
        ThighPoint::OneInchBelow => r.thigh_1in = get(Measurement::Thigh),
    }
    let knee = get(Measurement::Knee);
    match points.knee {
        KneePoint::Below12 => r.knee_12in = knee,
        KneePoint::Below13 => r.knee_13in = knee,
        KneePoint::Below14 => r.knee_14in = knee,
        KneePoint::HalfInseam => r.knee_half_inseam = knee,
    }
    r
}

fn rank(a: &Match, b: &Match) -> Ordering {
    let unknown = |m: &Match| m.basis == Basis::ShrinkageUnknown;
    let pct = |v: f64| (v * 10.0).round() as i64;
    unknown(a)
        .cmp(&unknown(b))
        .then(pct(b.closeness).cmp(&pct(a.closeness)))
        .then(pct(b.average).cmp(&pct(a.average)))
        .then(b.compared.cmp(&a.compared))
}

fn passes_line_filter(line: Option<Line>, f: &Filters) -> bool {
    match line {
        Some(Line::Men) => f.mens,
        Some(Line::Women) => f.womens,
        Some(Line::Unisex) => f.mens || f.womens,
        // Most brands don't file their cuts by line, so these always show.
        None => true,
    }
}

fn passes_product_filters(brand: &Brand, product: &Product, price: Option<f64>, f: &Filters) -> bool {
    if let Some(brands) = &f.brands {
        if !brands.contains(&brand.id) {
            return false;
        }
    }
    if f.try_on_only && !f.try_on_brands.contains(&brand.id) {
        return false;
    }
    if f.current_only && !product.is_current() {
        return false;
    }
    if f.sanforized_only && product.fabric.sanforized != Some(true) {
        return false;
    }
    if f.no_stretch && product.is_stretch() {
        return false;
    }
    let in_range = |v: Option<f64>, min: Option<f64>, max: Option<f64>| {
        if min.is_none() && max.is_none() {
            return true;
        }
        let Some(v) = v else { return false };
        min.is_none_or(|m| v >= m - 1e-9) && max.is_none_or(|m| v <= m + 1e-9)
    };
    in_range(price, f.price_min, f.price_max) && in_range(product.fabric.weight_oz, f.weight_min, f.weight_max)
}

/// For brands sold in lengths, keeps one size per waist: the shortest
/// length that's at least `inseam`, or the longest if none is.
fn pick_lengths(sizes: Vec<usize>, washed: &Washed, inseam: f64) -> Vec<usize> {
    let length = |i: usize| {
        let s = &washed.sizes[i];
        s.after
            .get(&Measurement::Inseam)
            .copied()
            .or(s.length_tag.map(|l| l * crate::units::CM_PER_INCH))
            .unwrap_or(0.0)
    };
    let mut by_waist: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for i in sizes {
        by_waist
            .entry(washed.sizes[i].waist_tag.to_string())
            .or_default()
            .push(i);
    }
    let mut keep: Vec<usize> = by_waist
        .into_values()
        .filter_map(|mut group| {
            group.sort_by(|a, b| length(*a).total_cmp(&length(*b)));
            group
                .iter()
                .copied()
                .find(|i| length(*i) >= inseam - 1e-9)
                .or(group.last().copied())
        })
        .collect();
    keep.sort();
    keep
}

/// Finds every match, grouped into one row per cut, closest first.
pub fn find_matches(catalogue: &Catalogue, ease: &EaseTable, profile: &Profile, filters: &Filters) -> Results {
    find_matches_prepared(&prepare(catalogue), catalogue, ease, profile, filters)
}

pub fn find_matches_prepared(
    prepared: &Prepared,
    catalogue: &Catalogue,
    ease: &EaseTable,
    profile: &Profile,
    filters: &Filters,
) -> Results {
    let pair = profile
        .reference_pair
        .as_ref()
        .and_then(|p| pair_readings(p, catalogue));
    let fit = &profile.desired_fit;

    // Per cut: every product's matching sizes, ranked.
    let mut cuts: BTreeMap<(String, String), CutMatches> = BTreeMap::new();
    for item in &prepared.items {
        let (brand, cut, product, washed) = (item.brand, item.cut, item.product, &item.washed);
        let price = product.list_price.as_ref().and_then(|lp| {
            catalogue
                .exchange_rates
                .convert(lp.amount, &lp.currency, &filters.currency)
        });
        if !passes_line_filter(cut.line, filters) || !passes_product_filters(brand, product, price, filters) {
            continue;
        }
        let points = Points::resolve(washed.thigh_point, washed.knee_point, washed.approximate);
        let stretch = product.is_stretch();
        let set = targets(profile, pair.as_ref(), ease, points, stretch);

        let mut candidates: Vec<usize> = (0..washed.sizes.len())
            .filter(|i| {
                let s = &washed.sizes[*i];
                set.limits
                    .iter()
                    .all(|(m, limit)| s.after.get(m).is_none_or(|v| *v >= limit - 1e-9))
            })
            .collect();
        if brand.sold_in_lengths() {
            let inseam = set
                .targets
                .get(&Measurement::Inseam)
                .map(|t| t.target.ideal())
                .unwrap_or(profile.body.inseam_wanted);
            candidates = pick_lengths(candidates, washed, inseam);
        }

        let mut matches: Vec<Match> = candidates
            .into_iter()
            .map(|i| {
                let s = &washed.sizes[i];
                let measurements: Vec<MeasurementResult> = Measurement::ALL
                    .iter()
                    .filter_map(|m| {
                        let value = s.after.get(m).copied();
                        let target = set.targets.get(m).copied();
                        if value.is_none() && target.is_none() {
                            return None;
                        }
                        let (closeness, off_by) = match (value, target) {
                            (Some(v), Some(t)) => (Some(closeness(v, &t.target)), Some(v - t.target.ideal())),
                            _ => (None, None),
                        };
                        Some(MeasurementResult {
                            measurement: *m,
                            value,
                            target,
                            closeness,
                            off_by,
                        })
                    })
                    .collect();
                let scores: Vec<f64> = measurements.iter().filter_map(|m| m.closeness).collect();
                let worst = scores.iter().copied().fold(100.0, f64::min);
                let average = if scores.is_empty() {
                    100.0
                } else {
                    scores.iter().sum::<f64>() / scores.len() as f64
                };
                let shape = leg_shape(&s.after);
                let rise = item.rises[i];
                let fits = fit.leg_shape.is_none_or(|want| shape == Some(want))
                    && fit.rise.is_none_or(|want| rise == Some(want));
                Match {
                    brand_id: brand.id.clone(),
                    cut_id: cut.id.clone(),
                    product_id: product_key(&brand.id, &product.id),
                    size: s.size.clone(),
                    closeness: worst,
                    average,
                    compared: scores.len(),
                    measurements,
                    leg_shape: shape,
                    rise,
                    fits_shape_and_rise: fits,
                    basis: washed.basis,
                    approximate: points.approximate,
                    stretch,
                    on_brands_site: product.is_current(),
                    price,
                }
            })
            .collect();
        if matches.is_empty() {
            continue;
        }
        matches.sort_by(rank);
        cuts.entry((brand.id.clone(), cut.id.clone()))
            .or_insert((cut, vec![]))
            .1
            .push(matches);
    }

    let build_rows = |shown: &dyn Fn(&Match) -> bool| -> Vec<CutRow> {
        let mut rows: Vec<CutRow> = cuts
            .iter()
            .filter_map(|((brand_id, cut_id), (cut, products))| {
                let mut products: Vec<Vec<Match>> = products
                    .iter()
                    .map(|sizes| sizes.iter().filter(|m| shown(m)).cloned().collect::<Vec<_>>())
                    .filter(|sizes| !sizes.is_empty())
                    .collect();
                if products.is_empty() {
                    return None;
                }
                products.sort_by(|a, b| rank(&a[0], &b[0]));
                let mut products = products.into_iter();
                let mut best_sizes = products.next()?.into_iter();
                let best = best_sizes.next()?;
                let brand_name = catalogue
                    .brands
                    .iter()
                    .find(|b| &b.id == brand_id)
                    .map(|b| b.name.clone())
                    .unwrap_or_default();
                Some(CutRow {
                    brand_id: brand_id.clone(),
                    brand_name,
                    cut_id: cut_id.clone(),
                    cut: cut.name.clone(),
                    brand_label: cut.brand_label.clone(),
                    line: cut.line,
                    best,
                    other_products: products.map(|sizes| sizes.into_iter().next().unwrap()).collect(),
                    other_sizes: best_sizes.collect(),
                })
            })
            .collect();
        rows.sort_by(|a, b| rank(&a.best, &b.best));
        rows
    };

    let all = build_rows(&|_| true);
    let chosen = build_rows(&|m| m.fits_shape_and_rise);
    let hidden = all.len() - chosen.len();
    Results {
        rows: if filters.show_other_shapes_and_rises {
            all
        } else {
            chosen
        },
        hidden_by_shape_or_rise: hidden,
        from_body_only: pair.is_none(),
    }
}
