//! The catalogue and ease table built into the app.

use crate::catalogue::{Catalogue, ExchangeRates};
use crate::ease::EaseTable;

include!(concat!(env!("OUT_DIR"), "/embedded_files.rs"));

/// The built-in catalogue and ease table. Before any catalogue has been
/// added (an empty `catalogue/`), the catalogue is empty.
pub fn load() -> Result<(Catalogue, EaseTable), String> {
    let files: Vec<(String, String)> = CATALOGUE_FILES
        .iter()
        .map(|(p, t)| (p.to_string(), t.to_string()))
        .collect();
    let catalogue = if files.is_empty() {
        Catalogue {
            brands: vec![],
            exchange_rates: ExchangeRates {
                source: "https://www.ecb.europa.eu/stats/eurofxref/eurofxref-daily.xml".into(),
                date: "1970-01-01".into(),
                base: "EUR".into(),
                rates: Default::default(),
            },
        }
    } else {
        Catalogue::from_files(&files)?
    };
    Ok((catalogue, EaseTable::from_json(EASE_TABLE)?))
}
