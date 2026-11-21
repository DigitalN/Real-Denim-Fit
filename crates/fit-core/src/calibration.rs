//! The ease-table calibration: compares every cut with Levi's published
//! body chart for its tag size (docs/research/ease.md, ADR 0009).
//!
//! For each rigid product and size, the thigh and seat ease (after the
//! first wash, garment minus Levi's body for that tag size) is placed in a
//! room using the ease table. A cut lands in the room most of its sizes
//! fall in. If the cuts brands call slim mostly land in regular, the snug
//! and regular ranges need moving up.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::after_wash::after_wash;
use crate::catalogue::{Catalogue, Measurement, Source};
use crate::ease::EaseTable;
use crate::profile::Room;

#[derive(Debug, Clone, Deserialize)]
pub struct BodySize {
    pub size: f64,
    pub waist: f64,
    pub seat: f64,
    pub thigh: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BodyChart {
    pub source: Source,
    pub sizes: Vec<BodySize>,
}

impl BodyChart {
    pub fn from_json(text: &str) -> Result<Self, String> {
        serde_json::from_str(text).map_err(|e| format!("body chart: {e}"))
    }
}

/// Where one ease falls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Landing {
    TighterThanSnug,
    Snug,
    Regular,
    Roomy,
    RoomierThanRoomy,
}

impl Landing {
    fn of(ease: &EaseTable, m: Measurement, flat_ease: f64) -> Option<Landing> {
        Some(match ease.room_for(m, flat_ease) {
            Some(Room::Snug) => Landing::Snug,
            Some(Room::Regular) => Landing::Regular,
            Some(Room::Roomy) => Landing::Roomy,
            None if flat_ease < ease.range(m, Room::Snug, false)?.low => Landing::TighterThanSnug,
            None => Landing::RoomierThanRoomy,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CutResult {
    pub brand: String,
    pub cut: String,
    pub brand_label: Option<String>,
    /// How many sizes landed where, by their thigh ease.
    pub thigh: BTreeMap<Landing, usize>,
    /// The same by seat, for brands that publish it.
    pub seat: BTreeMap<Landing, usize>,
    /// The average thigh ease, flat cm.
    pub mean_thigh_ease: f64,
    /// Where most sizes landed by thigh.
    pub lands: Landing,
}

impl CutResult {
    pub fn called_slim(&self) -> bool {
        self.brand_label
            .as_deref()
            .is_some_and(|l| l.to_lowercase().contains("slim"))
    }
}

/// Compares every rigid cut with the body chart.
pub fn calibrate(catalogue: &Catalogue, ease: &EaseTable, body: &BodyChart) -> Vec<CutResult> {
    let body_for = |tag: f64| body.sizes.iter().find(|b| (b.size - tag).abs() < 1e-9);
    let mut out = vec![];
    for brand in &catalogue.brands {
        for cut in &brand.cuts {
            let mut thigh: BTreeMap<Landing, usize> = BTreeMap::new();
            let mut seat: BTreeMap<Landing, usize> = BTreeMap::new();
            let mut eases = vec![];
            for p in cut.products.iter().filter(|p| !p.is_stretch()) {
                let Some(w) = after_wash(brand, p) else { continue };
                for s in &w.sizes {
                    let Some(b) = body_for(s.waist_tag) else { continue };
                    if let Some(t) = s.after.get(&Measurement::Thigh) {
                        let e = t - b.thigh / 2.0;
                        eases.push(e);
                        if let Some(l) = Landing::of(ease, Measurement::Thigh, e) {
                            *thigh.entry(l).or_default() += 1;
                        }
                    }
                    if let Some(v) = s.after.get(&Measurement::Seat) {
                        if let Some(l) = Landing::of(ease, Measurement::Seat, v - b.seat / 2.0) {
                            *seat.entry(l).or_default() += 1;
                        }
                    }
                }
            }
            let Some((lands, _)) = thigh.iter().max_by_key(|(_, n)| **n).map(|(l, n)| (*l, *n)) else {
                continue;
            };
            out.push(CutResult {
                brand: brand.name.clone(),
                cut: cut.name.clone(),
                brand_label: cut.brand_label.clone(),
                mean_thigh_ease: eases.iter().sum::<f64>() / eases.len() as f64,
                thigh,
                seat,
                lands,
            });
        }
    }
    out
}

/// The share of slim-called cuts that land in regular or roomier, or
/// `None` when no cut is called slim.
pub fn slim_in_regular_share(results: &[CutResult]) -> Option<f64> {
    let slim: Vec<_> = results.iter().filter(|r| r.called_slim()).collect();
    if slim.is_empty() {
        return None;
    }
    let regular = slim.iter().filter(|r| r.lands >= Landing::Regular).count();
    Some(regular as f64 / slim.len() as f64)
}

/// A readable table of the results.
pub fn report(results: &[CutResult]) -> String {
    let mut s = String::from("brand / cut (brand's label): sizes by thigh ease, average thigh ease flat -> lands\n");
    for r in results {
        let counts: Vec<String> = r.thigh.iter().map(|(l, n)| format!("{l:?} {n}")).collect();
        s += &format!(
            "{}{} / {} ({}): {}, {:+.1} cm -> {:?}\n",
            if r.called_slim() { "* " } else { "  " },
            r.brand,
            r.cut,
            r.brand_label.as_deref().unwrap_or("no label"),
            counts.join(", "),
            r.mean_thigh_ease,
            r.lands
        );
    }
    if let Some(share) = slim_in_regular_share(results) {
        s += &format!(
            "\nCuts called slim (*) landing in regular or roomier: {:.0}%\n",
            share * 100.0
        );
    }
    s
}
