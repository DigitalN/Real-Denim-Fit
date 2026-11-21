//! The catalogue and ease table built into the app.

use crate::catalogue::{Brand, Catalogue, ExchangeRates};
use crate::ease::EaseTable;

include!(concat!(env!("OUT_DIR"), "/embedded_files.rs"));

/// Parses a catalogue from the brand files and the exchange rates.
pub fn parse(brand_files: &[(&str, &str)], rates: &str) -> Result<Catalogue, String> {
    let mut brands: Vec<Brand> = brand_files
        .iter()
        .map(|(name, text)| serde_json::from_str(text).map_err(|e| format!("{name}: {e}")))
        .collect::<Result<_, _>>()?;
    brands.sort_by(|a, b| a.name.cmp(&b.name));
    let exchange_rates: ExchangeRates =
        serde_json::from_str(rates).map_err(|e| format!("exchange-rates.json: {e}"))?;
    Ok(Catalogue {
        brands,
        exchange_rates,
    })
}

/// The built-in catalogue and ease table.
pub fn load() -> Result<(Catalogue, EaseTable), String> {
    Ok((
        parse(BRAND_FILES, EXCHANGE_RATES)?,
        EaseTable::from_json(EASE_TABLE)?,
    ))
}
