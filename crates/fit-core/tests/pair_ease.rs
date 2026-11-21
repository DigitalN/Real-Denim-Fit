//! A reference pair's own ease, and the room it suggests.
mod common;

use common::*;
use fit_core::catalogue::Measurement;
use fit_core::pair_ease::{pair_ease, Place};
use fit_core::profile::{PairReadings, Room};

#[test]
fn a_pairs_ease_is_shown_around_and_placed_in_a_room() {
    // Thigh 57.2 cm around; the pair's thigh at the crotch is 31.85 cm flat:
    // +6.5 cm around, which is regular.
    let readings = PairReadings {
        thigh_crotch: Some(31.85),
        ..Default::default()
    };
    let e = pair_ease(&body(), &readings, &ease());
    let thigh = e.points.iter().find(|p| p.measurement == Measurement::Thigh).unwrap();
    assert!(close(thigh.ease_around, 6.5));
    assert_eq!(thigh.place, Place::Room(Room::Regular));
    assert_eq!(thigh.at, "at the crotch");
}

#[test]
fn ease_beyond_the_table_is_tighter_than_snug_or_roomier_than_roomy() {
    let readings = PairReadings {
        seat: Some(97.8 / 2.0 + 0.2 * IN),
        knee_13in: Some(40.0 / 2.0 + 4.0 * IN),
        ..Default::default()
    };
    let e = pair_ease(&body(), &readings, &ease());
    assert_eq!(
        e.points
            .iter()
            .find(|p| p.measurement == Measurement::Seat)
            .unwrap()
            .place,
        Place::TighterThanSnug
    );
    assert_eq!(
        e.points
            .iter()
            .find(|p| p.measurement == Measurement::Knee)
            .unwrap()
            .place,
        Place::RoomierThanRoomy
    );
}

#[test]
fn the_suggested_room_is_the_one_the_seat_and_thigh_sit_closest_to() {
    let readings = PairReadings {
        seat: Some(97.8 / 2.0 + 2.0 * IN),
        thigh_crotch: Some(57.2 / 2.0 + 1.9 * IN),
        ..Default::default()
    };
    assert_eq!(pair_ease(&body(), &readings, &ease()).suggested_room, Some(Room::Roomy));
    let readings = PairReadings {
        thigh_1in: Some(55.0 / 2.0 + 0.7 * IN),
        ..Default::default()
    };
    assert_eq!(pair_ease(&body(), &readings, &ease()).suggested_room, Some(Room::Snug));
}

#[test]
fn without_seat_or_thigh_readings_no_room_is_suggested() {
    let readings = PairReadings {
        waist: Some(42.0),
        ..Default::default()
    };
    let e = pair_ease(&body(), &readings, &ease());
    assert_eq!(e.suggested_room, None);
    assert_eq!(e.points.len(), 1);
}
