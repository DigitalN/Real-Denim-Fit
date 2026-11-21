//! The checks CI runs on the catalogue and ease table, following
//! `docs/catalogue-format.md`. `cargo test --workspace` runs them on the
//! shipped data.

use std::collections::{BTreeMap, BTreeSet};

use crate::after_wash::after_wash;
use crate::catalogue::{
    Brand, Catalogue, Chart, ChartState, Cut, Finish, Measurement, Product, Shrinkage, Sold, Source, KNEE_POINTS,
    SEAT_POINTS, THIGH_POINTS,
};
use crate::ease::EaseTable;
use crate::profile::Room;
use crate::units::{Unit, CM_PER_INCH};

/// Sensible ranges in cm, widths flat. Wide enough for sizes 24–44,
/// women's skinny cuts, wide-leg trousers and pre-hemmed inseams.
fn sensible(m: Measurement) -> (f64, f64) {
    match m {
        Measurement::Waist => (28.0, 70.0),
        Measurement::Seat => (34.0, 80.0),
        Measurement::Thigh => (18.0, 45.0),
        Measurement::Knee => (11.0, 40.0),
        Measurement::LegOpening => (10.0, 40.0),
        Measurement::FrontRise => (18.0, 40.0),
        Measurement::BackRise => (25.0, 52.0),
        Measurement::Inseam => (55.0, 110.0),
    }
}

fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
}

fn is_link(s: &str) -> bool {
    s.starts_with("https://") && s.len() > "https://".len() + 3
}

fn is_id(s: &str) -> bool {
    !s.is_empty()
        && !s.starts_with('-')
        && !s.ends_with('-')
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn check_source(what: &str, s: &Source, out: &mut Vec<String>) {
    if !is_link(&s.url) {
        out.push(format!("{what}: needs a link (https://…)"));
    }
    if !is_date(&s.seen) {
        out.push(format!("{what}: needs the date it was seen (YYYY-MM-DD)"));
    }
}

fn check_points(what: &str, points: &crate::catalogue::MeasuringPoints, out: &mut Vec<String>) {
    for (m, allowed) in [
        (Measurement::Thigh, &THIGH_POINTS[..]),
        (Measurement::Knee, &KNEE_POINTS[..]),
        (Measurement::Seat, &SEAT_POINTS[..]),
    ] {
        if let Some(p) = points.get(m) {
            if !allowed.contains(&p) {
                out.push(format!(
                    "{what}: {} measuring point {p:?} isn't one of {allowed:?}",
                    m.label()
                ));
            }
        }
    }
}

fn check_chart(what: &str, brand: &Brand, chart: &Chart, out: &mut Vec<String>) {
    check_source(&format!("{what}: chart source"), &chart.source, out);
    if let Some(points) = &chart.measuring_points {
        check_points(what, points, out);
    }
    if chart.sizes.is_empty() {
        out.push(format!("{what}: chart has no sizes"));
    }
    let effective = chart.effective(brand);
    let mut labels = BTreeSet::new();
    for size in &chart.sizes {
        if !labels.insert(&size.label) {
            out.push(format!("{what}: size {} appears twice", size.label));
        }
        if brand.sold_in_lengths() && size.length_tag.is_none() {
            out.push(format!(
                "{what} size {}: the brand is sold in lengths but the size has no length_tag",
                size.label
            ));
        }
        for (m, v) in &size.measurements {
            if matches!(m, Measurement::Thigh | Measurement::Knee | Measurement::Seat)
                && effective.measuring_points.get(*m).is_none()
            {
                out.push(format!(
                    "{what}: {} has no measuring point on the brand or chart (use \"unknown\" if the brand doesn't say)",
                    m.label()
                ));
            }
            let Some(way) = chart.way(brand, *m) else {
                out.push(format!(
                    "{what}: {} has no convention (flat or around) on the brand or chart",
                    m.label()
                ));
                continue;
            };
            let core = way.to_core(chart.unit.to_cm(*v));
            let (lo, hi) = sensible(*m);
            if !(lo..=hi).contains(&core) {
                out.push(format!(
                    "{what} size {}: {} {v} {} is out of the sensible range ({lo}–{hi} cm{})",
                    size.label,
                    m.label(),
                    if chart.unit == Unit::In { "in" } else { "cm" },
                    if m.is_length() { "" } else { " flat" }
                ));
            }
        }
    }
}

/// Consistency within a chart, to catch extraction slips. Values are
/// compared as published, in the chart's unit, using its tolerance (±0.5"
/// when the brand states none):
/// - the waist must grow with the size, allowing for the tolerance
/// - seat, thigh, knee, leg opening and rises mustn't shrink between
///   neighbouring sizes by more than the tolerance plus a quarter inch
/// - no value but the inseam may sit more than 3 × the tolerance off the
///   line through its neighbouring sizes (inseams step up in bands)
///
/// Sizes sold in lengths are only compared with sizes of the same length.
fn check_consistency(what: &str, brand: &Brand, chart: &Chart, out: &mut Vec<String>) {
    let per_unit = chart.unit.to_cm(1.0);
    let tolerance = chart
        .effective(brand)
        .tolerance
        .map(|q| q.cm())
        .unwrap_or(0.5 * CM_PER_INCH)
        / per_unit;
    let margin = 0.25 * CM_PER_INCH / per_unit;
    let unit = if chart.unit == Unit::In { "in" } else { "cm" };

    let mut groups: BTreeMap<String, Vec<&crate::catalogue::Size>> = BTreeMap::new();
    for s in &chart.sizes {
        groups
            .entry(s.length_tag.map(|l| l.to_string()).unwrap_or_default())
            .or_default()
            .push(s);
    }
    for sizes in groups.values_mut() {
        sizes.sort_by(|a, b| a.waist_tag.total_cmp(&b.waist_tag));
        for m in Measurement::ALL {
            let seq: Vec<(&str, f64, f64)> = sizes
                .iter()
                .filter_map(|s| Some((s.label.as_str(), s.waist_tag, *s.measurements.get(&m)?)))
                .collect();
            let allowed_drop = match m {
                Measurement::Waist => Some(tolerance),
                Measurement::Inseam => None,
                _ => Some(tolerance + margin),
            };
            for pair in seq.windows(2) {
                let ((prev_label, _, prev), (label, _, v)) = (pair[0], pair[1]);
                if let Some(drop) = allowed_drop {
                    if v < prev - drop - 1e-9 {
                        out.push(format!(
                            "{what} size {label}: {} {v} {unit} is smaller than size {prev_label}'s {prev} {unit}, \
                             beyond the ±{tolerance:.2} {unit} tolerance",
                            m.label()
                        ));
                    }
                }
            }
            // Inseams step up in bands for bigger sizes, so they're left out.
            if m == Measurement::Inseam {
                continue;
            }
            for three in seq.windows(3) {
                let ((a_label, a_tag, a), (label, tag, v), (b_label, b_tag, b)) = (three[0], three[1], three[2]);
                if (b_tag - a_tag).abs() < 1e-9 {
                    continue;
                }
                let expected = a + (b - a) * (tag - a_tag) / (b_tag - a_tag);
                if (v - expected).abs() > 3.0 * tolerance + 1e-9 {
                    out.push(format!(
                        "{what} size {label}: {} {v} {unit} is off the line through sizes {a_label} and {b_label} \
                         (about {expected:.2} {unit} expected, more than 3 × the ±{tolerance:.2} {unit} tolerance away)",
                        m.label()
                    ));
                }
            }
        }
    }
}

fn check_product(
    what: &str,
    brand: &Brand,
    p: &Product,
    rates: &crate::catalogue::ExchangeRates,
    out: &mut Vec<String>,
) {
    if !is_id(&p.id) {
        out.push(format!("{what}: id must be lowercase kebab-case"));
    }
    if !is_link(&p.url) {
        out.push(format!("{what}: needs a link to the brand's page"));
    }
    if !is_date(&p.status_seen) {
        out.push(format!("{what}: status_seen must be a date (YYYY-MM-DD)"));
    }
    match (p.finish, p.sold) {
        (Some(Finish::Raw), Some(Sold::OneWashed)) | (Some(Finish::Rinsed | Finish::Washed), Some(Sold::Raw)) => {
            out.push(format!(
                "{what}: finish {:?} doesn't go with sold {:?} (raw goes with \"raw\"; rinsed and washed with \"one-washed\")",
                p.finish.unwrap(),
                p.sold.unwrap()
            ))
        }
        _ => {}
    }
    if p.charts.is_empty() {
        out.push(format!(
            "{what}: has no chart; leave it out and note it in EXTRACTING.md"
        ));
    }
    for state in [ChartState::AsSold, ChartState::AfterWash] {
        if p.charts.iter().filter(|c| c.state == state).count() > 1 {
            out.push(format!("{what}: more than one chart in the same state"));
        }
    }
    for c in &p.charts {
        let state = if c.state == ChartState::AsSold {
            "as-sold"
        } else {
            "after-wash"
        };
        check_chart(&format!("{what} ({state})"), brand, c, out);
        check_consistency(&format!("{what} ({state})"), brand, c, out);
    }
    let publishes = |m: Measurement| {
        p.charts
            .iter()
            .any(|c| c.sizes.iter().any(|s| s.measurements.contains_key(&m)))
    };
    if !p.charts.is_empty() && !(publishes(Measurement::Waist) && publishes(Measurement::Thigh)) {
        out.push(format!("{what}: needs at least a waist and a thigh"));
    }
    if let Some(w) = after_wash(brand, p) {
        for s in &w.sizes {
            let Some(sold) = &s.as_sold else { continue };
            if w.basis != crate::after_wash::Basis::BrandChart {
                continue;
            }
            for (m, after) in &s.after {
                if let Some(before) = sold.get(m) {
                    if *after > before + 1e-6 {
                        out.push(format!(
                            "{what} size {}: after-wash {} is bigger than as sold",
                            s.size,
                            m.label()
                        ));
                    }
                }
            }
        }
    }
    match &p.shrinkage {
        Some(Shrinkage::Percent {
            length, width, source, ..
        }) => {
            if !(0.0..=15.0).contains(length) || !(0.0..=15.0).contains(width) {
                out.push(format!("{what}: shrinkage percent out of the sensible range (0–15)"));
            }
            check_source(&format!("{what}: shrinkage source"), source, out);
        }
        Some(Shrinkage::Amount {
            unit,
            waist,
            inseam,
            source,
            ..
        }) => {
            let ok = |v: &Option<f64>| v.is_none_or(|v| (0.0..=10.0).contains(&unit.to_cm(v)));
            if !ok(waist) || !ok(inseam) || (waist.is_none() && inseam.is_none()) {
                out.push(format!(
                    "{what}: shrinkage amount needs a waist or inseam between 0 and 10 cm"
                ));
            }
            check_source(&format!("{what}: shrinkage source"), source, out);
        }
        None => {}
    }
    if let Some(lp) = &p.list_price {
        if lp.amount <= 0.0 {
            out.push(format!("{what}: list price must be positive"));
        }
        if rates.convert(1.0, &lp.currency, "EUR").is_none() {
            out.push(format!("{what}: no exchange rate for {}", lp.currency));
        }
        check_source(
            &format!("{what}: list price"),
            &Source {
                url: lp.url.clone(),
                seen: lp.seen.clone(),
            },
            out,
        );
    }
    if let Some(w) = p.fabric.weight_oz {
        if !(4.0..=35.0).contains(&w) {
            out.push(format!("{what}: fabric weight {w} oz is out of the sensible range"));
        }
    }
    if let Some(comp) = &p.fabric.composition {
        let total: f64 = comp.iter().map(|f| f.percent).sum();
        if comp.is_empty() || !(98.0..=102.0).contains(&total) {
            out.push(format!("{what}: composition adds up to {total}%, not 100%"));
        }
    }
}

fn check_cut(
    brand: &Brand,
    cut: &Cut,
    ids: &mut BTreeSet<String>,
    rates: &crate::catalogue::ExchangeRates,
    out: &mut Vec<String>,
) {
    let what = format!("{}/{}", brand.id, cut.id);
    if !is_id(&cut.id) {
        out.push(format!("{what}: cut id must be lowercase kebab-case"));
    }
    if cut.products.is_empty() {
        out.push(format!("{what}: cut has no products"));
    }
    for p in &cut.products {
        if !ids.insert(p.id.clone()) {
            out.push(format!("{what}: product id {} appears twice in {}", p.id, brand.id));
        }
        check_product(&format!("{what} / {}", p.id), brand, p, rates, out);
    }
}

/// Every problem found, as a readable line. Empty means valid.
pub fn validate(catalogue: &Catalogue, ease: &EaseTable) -> Vec<String> {
    let mut out = vec![];
    let rates = &catalogue.exchange_rates;
    for b in &catalogue.brands {
        if !is_id(&b.id) {
            out.push(format!("{}: brand id must be lowercase kebab-case", b.id));
        }
        if !is_link(&b.site) {
            out.push(format!("{}: site needs a link", b.id));
        }
        if rates.convert(1.0, &b.currency, "EUR").is_none() {
            out.push(format!(
                "{}: no exchange rate for the brand's currency {}",
                b.id, b.currency
            ));
        }
        check_source(&format!("{}: method_source", b.id), &b.method_source, &mut out);
        check_points(&b.id, &b.measuring_points, &mut out);
        if b.cuts.is_empty() {
            out.push(format!("{}: brand has no cut files", b.id));
        }
        let mut cut_ids = BTreeSet::new();
        for c in &b.cuts {
            if !cut_ids.insert(&c.id) {
                out.push(format!("{}: cut id {} appears twice", b.id, c.id));
            }
        }
        let mut ids = BTreeSet::new();
        for c in &b.cuts {
            check_cut(b, c, &mut ids, rates, &mut out);
        }
    }
    if !is_link(&rates.source)
        || !is_date(&rates.date)
        || rates.base != "EUR"
        || rates.rates.values().any(|v| *v <= 0.0)
    {
        out.push("exchange-rates.json: needs the ECB link, a date, base EUR and positive rates".into());
    }

    for (m, point) in &ease.points {
        for room in Room::ALL {
            let range = point.range(room);
            if range.from > range.to {
                out.push(format!("ease table: {} {:?} runs backwards", m.label(), room));
            }
            if range.from < point.minimum.value - 1e-9 {
                out.push(format!("ease table: {} {:?} starts below the minimum", m.label(), room));
            }
            for s in &range.sources {
                if !ease.sources.contains_key(s) {
                    out.push(format!("ease table: unknown source {s}"));
                }
            }
        }
        for s in &point.minimum.sources {
            if !ease.sources.contains_key(s) {
                out.push(format!("ease table: unknown source {s}"));
            }
        }
    }
    out
}
