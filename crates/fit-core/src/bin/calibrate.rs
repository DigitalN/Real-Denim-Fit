//! Prints the ease-table calibration against Levi's body chart.
//! Usage: cargo run -p fit-core --bin calibrate

use fit_core::calibration::{calibrate, report, BodyChart};

fn main() {
    let (catalogue, ease) = fit_core::embedded::load().expect("the built-in data loads");
    let body = BodyChart::from_json(fit_core::embedded::LEVIS_BODY_CHART).expect("Levi's chart loads");
    print!("{}", report(&calibrate(&catalogue, &ease, &body)));
}
