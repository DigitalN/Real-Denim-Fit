//! Validates the catalogue and ease table, and prints a summary per brand.
//! Usage: cargo run -p fit-core --bin validate-catalogue [catalogue-dir]
//!
//! With no folder it checks the data built into the app. CI also runs the
//! same checks as tests (`cargo test --workspace`).

use fit_core::after_wash::{after_wash, Basis};
use fit_core::catalogue::Catalogue;
use fit_core::{embedded, validate::validate};

fn main() {
    let loaded = match std::env::args().nth(1) {
        Some(dir) => {
            Catalogue::from_dir(std::path::Path::new(&dir)).and_then(|c| embedded::load().map(|(_, ease)| (c, ease)))
        }
        None => embedded::load(),
    };
    let (catalogue, ease) = match loaded {
        Ok(data) => data,
        Err(e) => {
            eprintln!("The catalogue doesn't load: {e}");
            std::process::exit(1);
        }
    };
    for brand in &catalogue.brands {
        let products: Vec<_> = brand.products().collect();
        let sizes: usize = products
            .iter()
            .flat_map(|(_, p)| p.charts.first())
            .map(|c| c.sizes.len())
            .sum();
        let from_image = products
            .iter()
            .filter(|(_, p)| p.charts.iter().any(|c| c.from_image))
            .count();
        let unknown = products
            .iter()
            .filter(|(_, p)| after_wash(brand, p).is_some_and(|w| w.basis == Basis::ShrinkageUnknown))
            .count();
        println!(
            "{:<18} {:>3} cuts {:>4} products {:>5} sizes {:>3} from images {:>3} shrinkage unknown",
            brand.name,
            brand.cuts.len(),
            products.len(),
            sizes,
            from_image,
            unknown
        );
    }
    let problems = validate(&catalogue, &ease);
    if problems.is_empty() {
        println!("The catalogue and ease table are valid.");
    } else {
        for p in &problems {
            eprintln!("- {p}");
        }
        eprintln!("{} problem(s).", problems.len());
        std::process::exit(1);
    }
}
