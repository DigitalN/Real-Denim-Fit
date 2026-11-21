//! The target at each measuring point, and the hard limit (ADR 0006).

use std::collections::BTreeMap;

use serde::Serialize;

use crate::catalogue::{KneePoint, Measurement, ThighPoint};
use crate::ease::EaseTable;
use crate::profile::{PairReadings, Profile};
use crate::units::QUARTER_INCH;

/// The range a garment measurement should fall in, cm (widths flat).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Target {
    /// A range with an ideal middle.
    Range { low: f64, middle: f64, high: f64 },
    /// At least this long; longer is fine because raw denim gets hemmed.
    AtLeast { min: f64 },
}

impl Target {
    /// The ideal value: the middle of a range, or the least of an at-least.
    pub fn ideal(&self) -> f64 {
        match *self {
            Target::Range { middle, .. } => middle,
            Target::AtLeast { min } => min,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TargetFrom {
    /// The reference pair's reading, shifted by its nudges.
    Pair,
    /// The body plus the ease the room calls for.
    Body,
    /// The inseam you want.
    InseamWanted,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct TargetFor {
    pub target: Target,
    pub from: TargetFrom,
}

/// Where one chart takes its thigh and knee.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Points {
    pub thigh: ThighPoint,
    pub knee: KneePoint,
    /// The brand doesn't say where it measures, so the nearest reading is
    /// used and the match is marked approximate.
    pub approximate: bool,
}

impl Points {
    /// Fills in the nearest reading for points a chart doesn't state.
    pub fn resolve(thigh: Option<ThighPoint>, knee: Option<KneePoint>, approximate: bool) -> Points {
        Points {
            thigh: thigh.unwrap_or(ThighPoint::Crotch),
            knee: knee.unwrap_or(KneePoint::Below13),
            approximate,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TargetSet {
    pub targets: BTreeMap<Measurement, TargetFor>,
    /// The body plus minimum ease: anything smaller is left out.
    pub limits: BTreeMap<Measurement, f64>,
}

/// Works out the targets and hard limits for one chart's measuring points.
pub fn targets(
    profile: &Profile,
    pair: Option<&PairReadings>,
    ease: &EaseTable,
    points: Points,
    stretch: bool,
) -> TargetSet {
    let body = &profile.body;
    let fit = &profile.desired_fit;
    let feel = profile.reference_pair.as_ref().map(|p| &p.feel);
    let nudge = |m: Measurement| {
        feel.and_then(|f| f.get(&m))
            .map(|f| f.nudge_steps() * QUARTER_INCH)
            .unwrap_or(0.0)
    };

    let pair_reading = |m: Measurement| -> Option<f64> {
        let p = pair?;
        match m {
            Measurement::Waist => p.waist,
            Measurement::FrontRise => p.front_rise,
            Measurement::BackRise => p.back_rise,
            Measurement::Seat => p.seat,
            Measurement::Thigh => p.thigh_at(points.thigh),
            Measurement::Knee => p.knee_at(points.knee),
            Measurement::LegOpening => p.leg_opening,
            Measurement::Inseam => p.inseam,
        }
    };

    // The body at each point the ease table covers, flat cm.
    let body_flat = |m: Measurement| -> Option<f64> {
        Some(
            match m {
                Measurement::Waist => body.waist,
                Measurement::Seat => body.seat,
                Measurement::Thigh => body.thigh_at(points.thigh),
                Measurement::Knee => body.leg_at(points.knee),
                _ => return None,
            } / 2.0,
        )
    };

    let mut set = TargetSet {
        targets: BTreeMap::new(),
        limits: BTreeMap::new(),
    };

    for m in Measurement::ALL {
        if let (Some(b), Some(min)) = (body_flat(m), ease.minimum(m, stretch)) {
            set.limits.insert(m, b + min);
        }
        if fit.doesnt_matter.contains(&m) {
            continue;
        }
        let target = if m == Measurement::Inseam {
            match pair_reading(m) {
                Some(r) => TargetFor {
                    target: Target::AtLeast { min: r + nudge(m) },
                    from: TargetFrom::Pair,
                },
                None => TargetFor {
                    target: Target::AtLeast {
                        min: body.inseam_wanted,
                    },
                    from: TargetFrom::InseamWanted,
                },
            }
        } else if let Some(r) = pair_reading(m) {
            let middle = r + nudge(m);
            TargetFor {
                target: Target::Range {
                    low: middle - QUARTER_INCH,
                    middle,
                    high: middle + QUARTER_INCH,
                },
                from: TargetFrom::Pair,
            }
        } else if let (Some(b), Some(e)) = (body_flat(m), ease.range(m, fit.room, stretch)) {
            TargetFor {
                target: Target::Range {
                    low: b + e.low,
                    middle: b + (e.low + e.high) / 2.0,
                    high: b + e.high,
                },
                from: TargetFrom::Body,
            }
        } else {
            // Leg opening and rises: only from a reference pair.
            continue;
        };
        set.targets.insert(m, target);
    }
    set
}
