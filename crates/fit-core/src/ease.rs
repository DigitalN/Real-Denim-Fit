//! The ease table: the ease each room calls for at each measuring point.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::catalogue::Measurement;
use crate::profile::Room;
use crate::units::{Quantity, Unit, Way};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EaseRange {
    pub from: f64,
    pub to: f64,
    /// "well sourced" or "best estimate".
    pub quality: String,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EaseMinimum {
    pub value: f64,
    pub quality: String,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PointEase {
    pub snug: EaseRange,
    pub regular: EaseRange,
    pub roomy: EaseRange,
    pub minimum: EaseMinimum,
}

impl PointEase {
    pub fn range(&self, room: Room) -> &EaseRange {
        match room {
            Room::Snug => &self.snug,
            Room::Regular => &self.regular,
            Room::Roomy => &self.roomy,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StretchRule {
    pub about: String,
    /// Added to every range, flat.
    pub range_shift: Quantity,
    /// The minimum ease for stretch products, flat.
    pub minimum: Quantity,
    pub quality: String,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EaseTable {
    pub about: String,
    pub unit: Unit,
    pub way: Way,
    pub points: BTreeMap<Measurement, PointEase>,
    pub stretch: StretchRule,
    pub sources: BTreeMap<String, String>,
}

/// Flat ease in cm.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Ease {
    pub low: f64,
    pub high: f64,
}

impl EaseTable {
    pub fn from_json(text: &str) -> Result<Self, String> {
        serde_json::from_str(text).map_err(|e| format!("ease table: {e}"))
    }

    fn to_flat_cm(&self, v: f64) -> f64 {
        self.way.to_core(self.unit.to_cm(v))
    }

    /// The least ease a pair needs at this point, flat cm. `None` for points
    /// the table doesn't cover.
    pub fn minimum(&self, m: Measurement, stretch: bool) -> Option<f64> {
        let point = self.points.get(&m)?;
        Some(if stretch {
            self.stretch.minimum.cm()
        } else {
            self.to_flat_cm(point.minimum.value)
        })
    }

    /// The ease a room calls for at this point, flat cm. Stretch products
    /// have every range shifted down, never below their minimum.
    pub fn range(&self, m: Measurement, room: Room, stretch: bool) -> Option<Ease> {
        let point = self.points.get(&m)?;
        let r = point.range(room);
        let (mut low, mut high) = (self.to_flat_cm(r.from), self.to_flat_cm(r.to));
        if stretch {
            let shift = self.stretch.range_shift.cm();
            let floor = self.stretch.minimum.cm();
            low = (low + shift).max(floor);
            high = (high + shift).max(floor);
        }
        Some(Ease { low, high })
    }

    /// Which room a flat ease in cm falls in at this point, if any.
    pub fn room_for(&self, m: Measurement, ease: f64) -> Option<Room> {
        Room::ALL.into_iter().find(|room| {
            self.range(m, *room, false)
                .is_some_and(|r| ease >= r.low - 1e-9 && ease <= r.high + 1e-9)
        })
    }
}
