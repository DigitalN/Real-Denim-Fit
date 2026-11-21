//! Consistency checks that catch extraction slips in a chart: numbers that
//! go the wrong way between sizes, or sit far off the line through their
//! neighbours. Tolerance is the brand's (±0.5" in these tests).
mod common;

use common::*;
use fit_core::catalogue::{ChartState, Measurement::*, Size};
use fit_core::units::Unit;
use fit_core::validate::validate;

/// A smooth Iron Heart-like chart: each size up adds 1" around the waist,
/// 0.3" to the thigh, 0.2" to the knee and 0.1" to each rise.
fn smooth() -> Vec<Size> {
    (0..6)
        .map(|i| {
            let i = i as f64;
            row(
                &format!("{}", 30.0 + i),
                30.0 + i,
                &[
                    (Waist, 31.0 + i),
                    (FrontRise, 10.5 + 0.1 * i),
                    (BackRise, 14.5 + 0.1 * i),
                    (Thigh, 11.8 + 0.3 * i),
                    (Knee, 8.8 + 0.2 * i),
                    (LegOpening, 8.0 + 0.15 * i),
                    (Inseam, 34.0),
                ],
            )
        })
        .collect()
}

fn problems_with(sizes: Vec<Size>) -> Vec<String> {
    let p = product("p", "C", vec![chart(ChartState::AsSold, Unit::In, sizes)]);
    validate(&catalogue(vec![brand("b", vec![p])]), &ease())
}

fn set(sizes: &mut [Size], label: &str, m: fit_core::catalogue::Measurement, v: f64) {
    sizes
        .iter_mut()
        .find(|s| s.label == label)
        .unwrap()
        .measurements
        .insert(m, v);
}

#[test]
fn a_smooth_chart_passes() {
    assert_eq!(problems_with(smooth()), Vec::<String>::new());
}

#[test]
fn the_waist_must_grow_with_the_size() {
    let mut s = smooth();
    // Size 33's waist (34") typed as 32": smaller than size 32's 33".
    set(&mut s, "33", Waist, 32.0);
    let found = problems_with(s);
    assert!(
        found
            .iter()
            .any(|p| p.contains("waist") && p.contains("smaller than size 32")),
        "{found:?}"
    );
}

#[test]
fn a_waist_that_dips_within_the_tolerance_is_fine() {
    let mut s = smooth();
    // Same waist for two sizes, or a 0.4" dip: within ±0.5".
    set(&mut s, "33", Waist, 32.6);
    set(&mut s, "34", Waist, 34.6);
    assert_eq!(problems_with(s), Vec::<String>::new());
}

#[test]
fn widths_and_rises_must_not_go_the_wrong_way_between_neighbouring_sizes() {
    for m in [Thigh, Knee, LegOpening, FrontRise, BackRise] {
        let mut s = smooth();
        let before = s[2].measurements[&m];
        // Two inches smaller than the size below it.
        set(&mut s, "33", m, before - 2.0);
        let found = problems_with(s);
        assert!(
            found.iter().any(|p| p.contains(m.label()) && p.contains("size 33")),
            "{m:?}: {found:?}"
        );
    }
}

#[test]
fn a_value_far_off_the_line_through_its_neighbours_is_flagged() {
    let mut s = smooth();
    // Size 33's knee as 14" among 9.2" and 9.6": a slip for 9.4".
    set(&mut s, "33", Knee, 14.0);
    let found = problems_with(s);
    assert!(
        found
            .iter()
            .any(|p| p.contains("knee") && p.contains("size 33") && p.contains("off the line")),
        "{found:?}"
    );
}

#[test]
fn a_value_a_little_off_the_line_is_fine() {
    let mut s = smooth();
    // 1" off the line through its neighbours is within 3 × the 0.5" tolerance.
    set(&mut s, "33", Inseam, 35.0);
    assert_eq!(problems_with(s), Vec::<String>::new());
}

#[test]
fn sizes_sold_in_lengths_are_only_compared_within_a_length() {
    let mut sizes = vec![];
    for (length, inseam) in [(30.0, 30.0), (32.0, 32.0), (34.0, 34.0)] {
        for w in [30.0, 31.0, 32.0] {
            let mut s = row(
                &format!("W{w} L{length}"),
                w,
                &[(Waist, w + 1.0), (Thigh, 11.0 + 0.3 * (w - 30.0)), (Inseam, inseam)],
            );
            s.length_tag = Some(length);
            sizes.push(s);
        }
    }
    let p = product("p", "C", vec![chart(ChartState::AsSold, Unit::In, sizes)]);
    let mut b = brand("b", vec![p]);
    b.sizing = fit_core::catalogue::Sizing::WaistAndLength;
    assert_eq!(validate(&catalogue(vec![b]), &ease()), Vec::<String>::new());
}

#[test]
fn a_cm_chart_uses_the_tolerance_converted_to_cm() {
    // 1 cm dips are within ±0.5" (1.27 cm); 3 cm isn't.
    let sizes = |dip: f64| {
        (0..4)
            .map(|i| {
                let i = i as f64;
                let waist = if i == 2.0 {
                    82.0 + 2.5 * i - dip - 2.5
                } else {
                    82.0 + 2.5 * i
                };
                row(
                    &format!("{}", 30.0 + i),
                    30.0 + i,
                    &[(Waist, waist), (Thigh, 30.0 + 0.8 * i)],
                )
            })
            .collect::<Vec<_>>()
    };
    let cm = |s: Vec<Size>| {
        let mut c = chart(ChartState::AsSold, Unit::Cm, s);
        c.conventions = Some([(Waist, fit_core::catalogue::Convention::Around)].into());
        let p = product("p", "C", vec![c]);
        validate(&catalogue(vec![brand("b", vec![p])]), &ease())
    };
    assert_eq!(cm(sizes(1.0)), Vec::<String>::new());
    assert!(!cm(sizes(3.0)).is_empty());
}

#[test]
fn an_inseam_that_steps_up_for_bigger_sizes_is_fine() {
    // 3sixteen's RS-220x: 34.5" up to size 35, then 37" from size 36.
    let sizes = [
        (33.0, 34.5),
        (34.0, 34.5),
        (35.0, 34.5),
        (36.0, 37.0),
        (38.0, 37.0),
        (40.0, 37.0),
    ]
    .into_iter()
    .map(|(tag, inseam)| {
        let i = tag - 33.0;
        row(
            &format!("{tag}"),
            tag,
            &[(Waist, 34.0 + i), (Thigh, 12.5 + 0.3 * i), (Inseam, inseam)],
        )
    })
    .collect();
    assert_eq!(problems_with(sizes), Vec::<String>::new());
}
