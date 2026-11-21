//! The checks CI runs on the catalogue and ease table.

use std::collections::BTreeSet;

use crate::after_wash::after_wash;
use crate::catalogue::{Catalogue, Chart, ChartState, Measurement, Product, Tolerance};
use crate::ease::EaseTable;
use crate::profile::Room;

/// Sensible ranges in cm, widths flat. Wide enough for sizes 24–44.
fn sensible(m: Measurement) -> (f64, f64) {
    match m {
        Measurement::Waist => (28.0, 70.0),
        Measurement::Seat => (38.0, 80.0),
        Measurement::Thigh => (22.0, 45.0),
        Measurement::Knee => (15.0, 35.0),
        Measurement::LegOpening => (13.0, 35.0),
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

fn check_chart(where_: &str, chart: &Chart, out: &mut Vec<String>) {
    if !is_link(&chart.source) {
        out.push(format!("{where_}: chart has no link to its source"));
    }
    if !is_date(&chart.seen) {
        out.push(format!(
            "{where_}: chart has no date it was seen (YYYY-MM-DD)"
        ));
    }
    if let Tolerance::NotStated(s) = &chart.tolerance {
        if s != "not stated" {
            out.push(format!(
                "{where_}: tolerance must be a value with a unit or \"not stated\""
            ));
        }
    }
    for (m, p) in &chart.points {
        let known = match m {
            Measurement::Thigh => p.thigh_point().is_some() || p.at == "not-stated",
            Measurement::Knee => p.knee_point().is_some() || p.at == "not-stated",
            _ => !p.at.trim().is_empty(),
        };
        if !known {
            out.push(format!(
                "{where_}: {} measuring point {:?} isn't one the matching knows",
                m.label(),
                p.at
            ));
        }
    }
    let mut seen = BTreeSet::new();
    for row in &chart.sizes {
        if !seen.insert(&row.size) {
            out.push(format!("{where_}: size {} appears twice", row.size));
        }
        for (m, v) in &row.numbers {
            let Some(point) = chart.points.get(m) else {
                out.push(format!(
                    "{where_} size {}: {} has no measuring point",
                    row.size,
                    m.label()
                ));
                continue;
            };
            let core = point.way.to_core(chart.unit.to_cm(*v));
            let (lo, hi) = sensible(*m);
            if !(lo..=hi).contains(&core) {
                out.push(format!(
                    "{where_} size {}: {} {v} is out of the sensible range ({lo}–{hi} cm{})",
                    row.size,
                    m.label(),
                    if m.is_length() { "" } else { " flat" }
                ));
            }
        }
    }
}

fn check_product(where_: &str, p: &Product, catalogue: &Catalogue, out: &mut Vec<String>) {
    if p.charts.is_empty() {
        // Products with no measurements are kept but left out of matching.
        return;
    }
    for c in &p.charts {
        let state = match c.state {
            ChartState::AsSold => "as-sold",
            ChartState::AfterWash => "after-wash",
        };
        check_chart(&format!("{where_} ({state})"), c, out);
    }
    if p.charts
        .iter()
        .filter(|c| c.state == ChartState::AsSold)
        .count()
        > 1
        || p.charts
            .iter()
            .filter(|c| c.state == ChartState::AfterWash)
            .count()
            > 1
    {
        out.push(format!("{where_}: more than one chart in the same state"));
    }
    if let Some(w) = after_wash(p) {
        for s in &w.sizes {
            if !s.after.contains_key(&Measurement::Waist)
                || !s.after.contains_key(&Measurement::Thigh)
            {
                out.push(format!(
                    "{where_} size {}: needs at least a waist and a thigh",
                    s.size
                ));
            }
            if let Some(sold) = &s.as_sold {
                for (m, after) in &s.after {
                    if let Some(before) = sold.get(m) {
                        if *after > before + 1e-6 {
                            out.push(format!(
                                "{where_} size {}: after-wash {} is bigger than as sold",
                                s.size,
                                m.label()
                            ));
                        }
                    }
                }
            }
        }
    }
    if let Some(s) = &p.shrinkage {
        if !(0.0..=15.0).contains(&s.width_percent) || !(0.0..=15.0).contains(&s.length_percent) {
            out.push(format!(
                "{where_}: shrinkage out of the sensible range (0–15%)"
            ));
        }
        if !is_link(&s.source) || !is_date(&s.seen) {
            out.push(format!("{where_}: shrinkage needs a link and a date"));
        }
    }
    if let Some(lp) = &p.list_price {
        if lp.amount <= 0.0 {
            out.push(format!("{where_}: list price must be positive"));
        }
        if catalogue
            .exchange_rates
            .convert(1.0, &lp.currency, "EUR")
            .is_none()
        {
            out.push(format!("{where_}: no exchange rate for {}", lp.currency));
        }
        if !is_link(&lp.source) || !is_date(&lp.seen) {
            out.push(format!("{where_}: list price needs a link and a date"));
        }
    }
    if !(0.0..=10.0).contains(&p.elastane_percent) {
        out.push(format!("{where_}: elastane % out of range"));
    }
    if let Some(w) = p.fabric_weight_oz {
        if !(5.0..=35.0).contains(&w) {
            out.push(format!(
                "{where_}: fabric weight {w} oz is out of the sensible range"
            ));
        }
    }
    if !is_link(&p.url) {
        out.push(format!(
            "{where_}: product needs a link to the brand's page"
        ));
    }
}

/// Every problem found, as a readable line. Empty means valid.
pub fn validate(catalogue: &Catalogue, ease: &EaseTable) -> Vec<String> {
    let mut out = vec![];
    let mut ids = BTreeSet::new();
    for b in &catalogue.brands {
        if b.products.is_empty() {
            out.push(format!("{}: brand has no products", b.id));
        }
        for p in &b.products {
            if !ids.insert(p.id.clone()) {
                out.push(format!("{}: product id {} appears twice", b.id, p.id));
            }
            let where_ = format!("{} / {}", b.id, p.id);
            check_product(&where_, p, catalogue, &mut out);
            if b.sold_in_lengths {
                for c in &p.charts {
                    if c.sizes.iter().any(|s| s.tag_length.is_none()) {
                        out.push(format!(
                            "{where_}: brand is sold in lengths but a size has no tag_length"
                        ));
                    }
                }
            }
        }
    }
    let r = &catalogue.exchange_rates;
    if !is_link(&r.source) || !is_date(&r.date) || r.rates.values().any(|v| *v <= 0.0) {
        out.push("exchange rates need a link, a date and positive rates".into());
    }

    for (m, point) in &ease.points {
        for room in Room::ALL {
            let range = point.range(room);
            if range.from > range.to {
                out.push(format!(
                    "ease table: {} {:?} runs backwards",
                    m.label(),
                    room
                ));
            }
            if range.from < point.minimum.value - 1e-9 {
                out.push(format!(
                    "ease table: {} {:?} starts below the minimum",
                    m.label(),
                    room
                ));
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
