//! What the user brings: body measurements, a desired fit and, optionally, a
//! reference pair.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::catalogue::{KneePoint, Measurement, ThighPoint};

/// Body measurements in cm, all the way around, except the inseam you want,
/// which is a length.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Body {
    /// Where the jeans are worn, which isn't always the natural waist.
    pub waist: f64,
    pub seat: f64,
    /// The fullest part at the top of the thigh.
    pub thigh_top: f64,
    /// 2.5 cm (1") below the top reading.
    pub thigh_1in: f64,
    /// Around the leg 30.5 cm (12") below the crotch.
    pub leg_12in: f64,
    pub leg_13in: f64,
    pub leg_14in: f64,
    /// Halfway down the inseam.
    pub leg_half_inseam: f64,
    pub inseam_wanted: f64,
}

impl Body {
    /// The body's thigh reading at a brand's thigh point, around.
    pub fn thigh_at(&self, point: ThighPoint) -> f64 {
        match point {
            ThighPoint::Crotch => self.thigh_top,
            ThighPoint::OneInchBelow => self.thigh_1in,
        }
    }
    /// The body's leg reading at a brand's knee point, around.
    pub fn leg_at(&self, point: KneePoint) -> f64 {
        match point {
            KneePoint::Below12 => self.leg_12in,
            KneePoint::Below13 => self.leg_13in,
            KneePoint::Below14 => self.leg_14in,
            KneePoint::HalfInseam => self.leg_half_inseam,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Room {
    Snug,
    Regular,
    Roomy,
}

impl Room {
    pub const ALL: [Room; 3] = [Room::Snug, Room::Regular, Room::Roomy];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LegShape {
    Tapered,
    Straight,
    Wide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Rise {
    Low,
    Mid,
    High,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesiredFit {
    pub room: Room,
    /// `None` means any.
    pub leg_shape: Option<LegShape>,
    /// `None` means any.
    pub rise: Option<Rise>,
    /// How much each measurement counts, from 1 (least) to 5 (most). A
    /// measurement that isn't listed is at 3.
    #[serde(default)]
    pub importance: BTreeMap<Measurement, u8>,
}

/// The importance every measurement starts at.
pub const DEFAULT_IMPORTANCE: u8 = 3;

impl DesiredFit {
    /// A measurement's importance, 1 to 5.
    pub fn importance_of(&self, m: Measurement) -> u8 {
        self.importance
            .get(&m)
            .copied()
            .unwrap_or(DEFAULT_IMPORTANCE)
            .clamp(1, 5)
    }

    /// The highest importance currently set across all measurements.
    pub fn highest_importance(&self) -> u8 {
        Measurement::ALL
            .iter()
            .map(|m| self.importance_of(*m))
            .max()
            .unwrap_or(DEFAULT_IMPORTANCE)
    }
}

impl DesiredFit {
    /// The usual name for the combination, such as "slim tapered".
    pub fn usual_name(&self) -> String {
        let room = match self.room {
            Room::Snug => "slim",
            Room::Regular => "regular",
            Room::Roomy => "relaxed",
        };
        let leg = match self.leg_shape {
            Some(LegShape::Tapered) => "tapered",
            Some(LegShape::Straight) => "straight",
            Some(LegShape::Wide) => "wide leg",
            None => "fit",
        };
        let name = format!("{room} {leg}");
        match self.rise {
            Some(Rise::Low) => format!("low-rise {name}"),
            Some(Rise::High) => format!("high-rise {name}"),
            Some(Rise::Mid) | None => name,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Feel {
    TooTight,
    JustRight,
    TooLoose,
    TooShort,
    TooLong,
}

impl Feel {
    /// The nudge this feel asks for, in quarter inches: a pair that's too
    /// tight wants the target a little bigger.
    pub fn nudge_steps(self) -> f64 {
        match self {
            Feel::TooTight | Feel::TooShort => 1.0,
            Feel::JustRight => 0.0,
            Feel::TooLoose | Feel::TooLong => -1.0,
        }
    }
}

/// A reference pair's readings in cm: widths flat, lengths as lengths.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PairReadings {
    pub waist: Option<f64>,
    pub front_rise: Option<f64>,
    pub back_rise: Option<f64>,
    pub seat: Option<f64>,
    pub thigh_crotch: Option<f64>,
    pub thigh_1in: Option<f64>,
    pub knee_12in: Option<f64>,
    pub knee_13in: Option<f64>,
    pub knee_14in: Option<f64>,
    pub knee_half_inseam: Option<f64>,
    pub leg_opening: Option<f64>,
    pub inseam: Option<f64>,
}

impl PairReadings {
    pub fn thigh_at(&self, point: ThighPoint) -> Option<f64> {
        match point {
            ThighPoint::Crotch => self.thigh_crotch,
            ThighPoint::OneInchBelow => self.thigh_1in,
        }
    }
    pub fn knee_at(&self, point: KneePoint) -> Option<f64> {
        match point {
            KneePoint::Below12 => self.knee_12in,
            KneePoint::Below13 => self.knee_13in,
            KneePoint::Below14 => self.knee_14in,
            KneePoint::HalfInseam => self.knee_half_inseam,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum PairSource {
    /// Measured flat by the user.
    Measured { readings: PairReadings },
    /// Picked from the catalogue: its size's after-wash measurements are the
    /// readings, marked "from the brand's chart".
    FromCatalogue { product_id: String, size: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReferencePair {
    pub id: String,
    pub name: String,
    pub source: PairSource,
    #[serde(default)]
    pub feel: BTreeMap<Measurement, Feel>,
    /// For the user only; never used in matching.
    #[serde(default)]
    pub notes: String,
}

/// Everything matching needs from the user.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub body: Body,
    pub desired_fit: DesiredFit,
    /// The reference pair matches are compared with, if any.
    pub reference_pair: Option<ReferencePair>,
}
