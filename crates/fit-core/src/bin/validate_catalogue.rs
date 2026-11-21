//! Validates the catalogue and ease table. CI runs this on every pull request.
//! Usage: cargo run -p fit-core --bin validate-catalogue

use fit_core::{after_wash::Basis, embedded, validate::validate};

fn main() {
    let (catalogue, ease) = match embedded::load() {
        Ok(data) => data,
        Err(e) => {
            eprintln!("The catalogue doesn't load: {e}");
            std::process::exit(1);
        }
    };
    let problems = validate(&catalogue, &ease);
    for brand in &catalogue.brands {
        let sizes: usize = brand
            .products
            .iter()
            .flat_map(|p| p.charts.first())
            .map(|c| c.sizes.len())
            .sum();
        let from_image = brand
            .products
            .iter()
            .filter(|p| p.charts.iter().any(|c| c.from_image))
            .count();
        let unknown = brand
            .products
            .iter()
            .filter(|p| {
                fit_core::after_wash::after_wash(p)
                    .is_some_and(|w| w.basis == Basis::ShrinkageUnknown)
            })
            .count();
        println!(
            "{:<18} {:>4} products, {:>5} sizes, {:>3} read from images, {:>3} shrinkage unknown",
            brand.name,
            brand.products.len(),
            sizes,
            from_image,
            unknown
        );
    }
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
