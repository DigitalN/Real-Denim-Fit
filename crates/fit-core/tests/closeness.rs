//! Closeness of one measurement to its target (ADR 0009).
mod common;

use common::close;
use fit_core::closeness::closeness;
use fit_core::targets::Target;
use fit_core::units::EIGHTH_INCH;

fn range() -> Target {
    // 30 cm ± 1 cm.
    Target::Range {
        low: 29.0,
        middle: 30.0,
        high: 31.0,
    }
}

#[test]
fn the_middle_of_a_target_scores_100() {
    assert!(close(closeness(30.0, &range()), 100.0));
}

#[test]
fn the_edges_score_80_and_inside_is_in_between() {
    assert!(close(closeness(31.0, &range()), 80.0));
    assert!(close(closeness(29.0, &range()), 80.0));
    assert!(close(closeness(30.5, &range()), 90.0));
}

#[test]
fn beyond_an_edge_each_eighth_inch_costs_10_percent() {
    assert!(close(closeness(31.0 + EIGHTH_INCH, &range()), 70.0));
    assert!(close(closeness(29.0 - 2.0 * EIGHTH_INCH, &range()), 60.0));
}

#[test]
fn closeness_never_drops_below_zero() {
    assert!(close(closeness(31.0 + 20.0 * EIGHTH_INCH, &range()), 0.0));
}

#[test]
fn an_at_least_target_is_met_by_anything_as_long_or_longer() {
    let inseam = Target::AtLeast { min: 81.0 };
    assert!(close(closeness(81.0, &inseam), 100.0));
    assert!(close(closeness(90.0, &inseam), 100.0));
}

#[test]
fn falling_short_of_an_at_least_target_scores_like_passing_an_edge() {
    let inseam = Target::AtLeast { min: 81.0 };
    assert!(close(closeness(81.0 - EIGHTH_INCH, &inseam), 70.0));
}
