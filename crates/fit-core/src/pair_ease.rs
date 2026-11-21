//! A reference pair's own ease: how much bigger than the body it is at each
//! point, and which room that suggests.

use serde::Serialize;

use crate::catalogue::{KneePoint, Measurement, ThighPoint};
use crate::ease::EaseTable;
use crate::profile::{Body, PairReadings, Room};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "room", rename_all = "kebab-case")]
pub enum Place {
    TighterThanSnug,
    Room(Room),
    RoomierThanRoomy,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PointEase {
    pub measurement: Measurement,
    /// Where on the pair, such as "at the crotch" or "33 cm (13″) below the crotch".
    pub at: &'static str,
    /// Ease all the way around, cm, as the body was measured.
    pub ease_around: f64,
    pub place: Place,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PairEase {
    pub points: Vec<PointEase>,
    pub suggested_room: Option<Room>,
}

/// The pair's ease at every point both it and the body have a reading.
pub fn pair_ease(body: &Body, pair: &PairReadings, ease: &EaseTable) -> PairEase {
    let readings: Vec<(Measurement, &'static str, Option<f64>, f64)> = vec![
        (Measurement::Waist, "at the waist", pair.waist, body.waist),
        (Measurement::Seat, "at the seat", pair.seat, body.seat),
        (
            Measurement::Thigh,
            "at the crotch",
            pair.thigh_crotch,
            body.thigh_at(ThighPoint::Crotch),
        ),
        (
            Measurement::Thigh,
            "2.5 cm (1″) below the crotch",
            pair.thigh_1in,
            body.thigh_at(ThighPoint::OneInchBelow),
        ),
        (
            Measurement::Knee,
            "30.5 cm (12″) below the crotch",
            pair.knee_12in,
            body.leg_at(KneePoint::Below12),
        ),
        (
            Measurement::Knee,
            "33 cm (13″) below the crotch",
            pair.knee_13in,
            body.leg_at(KneePoint::Below13),
        ),
        (
            Measurement::Knee,
            "35.5 cm (14″) below the crotch",
            pair.knee_14in,
            body.leg_at(KneePoint::Below14),
        ),
        (
            Measurement::Knee,
            "halfway down the inseam",
            pair.knee_half_inseam,
            body.leg_at(KneePoint::HalfInseam),
        ),
    ];
    let points: Vec<PointEase> = readings
        .into_iter()
        .filter_map(|(m, at, reading, body_around)| {
            let flat = reading? - body_around / 2.0;
            let place = match ease.room_for(m, flat) {
                Some(room) => Place::Room(room),
                None if flat < ease.range(m, Room::Snug, false)?.low => Place::TighterThanSnug,
                None => Place::RoomierThanRoomy,
            };
            Some(PointEase {
                measurement: m,
                at,
                ease_around: flat * 2.0,
                place,
            })
        })
        .collect();

    // The room whose seat and thigh ranges the pair sits closest to.
    let room_points: Vec<&PointEase> = points
        .iter()
        .filter(|p| matches!(p.measurement, Measurement::Seat | Measurement::Thigh))
        .collect();
    let suggested_room = if room_points.is_empty() {
        None
    } else {
        Room::ALL.into_iter().min_by(|a, b| {
            let miss = |room: Room| -> f64 {
                room_points
                    .iter()
                    .filter_map(|p| {
                        let r = ease.range(p.measurement, room, false)?;
                        let flat = p.ease_around / 2.0;
                        Some((r.low - flat).max(0.0) + (flat - r.high).max(0.0))
                    })
                    .sum()
            };
            miss(*a).total_cmp(&miss(*b))
        })
    };
    PairEase { points, suggested_room }
}
