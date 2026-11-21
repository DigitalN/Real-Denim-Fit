//! The Real Denim Fit app: the screens in `ui/` call these commands.
//!
//! Everything the user enters is stored in one JSON file in the app's data
//! folder on this computer. The only network access is the update check.

mod currency;
mod updates;

use std::fs;
use std::path::PathBuf;

use fit_core::after_wash::{after_wash, Washed};
use fit_core::catalogue::{product_key, Brand, Catalogue, ChartState, Cut, Effective, Product};
use fit_core::ease::EaseTable;
use fit_core::matching::{find_matches_prepared, pair_readings, prepare, Filters, Prepared, Results};
use fit_core::pair_ease::{pair_ease, PairEase};
use fit_core::profile::{Body, Profile, ReferencePair};
use serde::Serialize;
use serde_json::Value;
use tauri::Manager;

/// The built-in data, loaded once.
pub struct Data {
    catalogue: &'static Catalogue,
    ease: EaseTable,
    prepared: Prepared<'static>,
}

pub fn data() -> Data {
    let (catalogue, ease) = fit_core::embedded::load().expect("the built-in catalogue loads");
    let catalogue: &'static Catalogue = Box::leak(Box::new(catalogue));
    Data {
        catalogue,
        ease,
        prepared: prepare(catalogue),
    }
}

fn state_file(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("real-denim-fit.json"))
}

/// Everything the user has entered, or `null` on first run.
#[tauri::command]
fn load_state(app: tauri::AppHandle) -> Result<Value, String> {
    read_state(&state_file(&app)?)
}

/// The saved state, `null` when there's no file yet (a first run). Any
/// other problem is an error, so the screens never mistake an unreadable
/// or damaged file for a first run and save over it.
pub fn read_state(path: &std::path::Path) -> Result<Value, String> {
    match fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text)
            .map_err(|e| format!("Your saved measurements in {} can't be read: {e}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Value::Null),
        Err(e) => Err(format!(
            "Your saved measurements in {} can't be read: {e}",
            path.display()
        )),
    }
}

#[tauri::command]
fn save_state(app: tauri::AppHandle, state: Value) -> Result<(), String> {
    let path = state_file(&app)?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(&state).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())
}

#[derive(Serialize)]
pub struct ProductSummary<'a> {
    /// `<brand>/<product>`.
    id: String,
    cut_id: &'a str,
    cut: &'a str,
    brand_label: Option<&'a str>,
    line: Option<fit_core::catalogue::Line>,
    name: &'a str,
    fabric_weight_oz: Option<f64>,
    elastane_percent: f64,
    sanforized: Option<bool>,
    current: bool,
    url: &'a str,
    list_price: Option<&'a fit_core::catalogue::ListPrice>,
    sizes: Vec<&'a str>,
}

#[derive(Serialize)]
pub struct BrandSummary<'a> {
    id: &'a str,
    name: &'a str,
    site: &'a str,
    sold_in_lengths: bool,
    products: Vec<ProductSummary<'a>>,
}

#[derive(Serialize)]
pub struct CatalogueIndex<'a> {
    brands: Vec<BrandSummary<'a>>,
    currencies: Vec<String>,
    rates: &'a std::collections::BTreeMap<String, f64>,
    rates_date: &'a str,
    rates_source: &'a str,
    default_currency: String,
}

fn summary<'a>(brand: &'a Brand, cut: &'a Cut, p: &'a Product) -> ProductSummary<'a> {
    let chart = p.chart(ChartState::AfterWash).or(p.chart(ChartState::AsSold));
    ProductSummary {
        id: product_key(&brand.id, &p.id),
        cut_id: &cut.id,
        cut: &cut.name,
        brand_label: cut.brand_label.as_deref(),
        line: cut.line,
        name: &p.name,
        fabric_weight_oz: p.fabric.weight_oz,
        elastane_percent: p.fabric.elastane_percent(),
        sanforized: p.fabric.sanforized,
        current: p.is_current(),
        url: &p.url,
        list_price: p.list_price.as_ref(),
        // Only sizes with measurements: the others can't be a reference pair.
        sizes: chart
            .map(|c| {
                c.sizes
                    .iter()
                    .filter(|s| !s.measurements.is_empty())
                    .map(|s| s.label.as_str())
                    .collect()
            })
            .unwrap_or_default(),
    }
}

/// Brands and products for the pickers and result rows, without charts.
#[tauri::command]
fn catalogue_index(data: tauri::State<'_, Data>) -> CatalogueIndex<'static> {
    index(&data)
}

pub fn index(data: &Data) -> CatalogueIndex<'static> {
    let c = data.catalogue;
    let currencies = c.exchange_rates.currencies();
    let default_currency = sys_locale::get_locale()
        .and_then(|l| currency::for_locale(&l))
        .filter(|cur| currencies.iter().any(|c| c == cur))
        .unwrap_or("USD")
        .to_string();
    CatalogueIndex {
        brands: c
            .brands
            .iter()
            .map(|b| BrandSummary {
                id: &b.id,
                name: &b.name,
                site: &b.site,
                sold_in_lengths: b.sold_in_lengths(),
                products: b.products().map(|(c, p)| summary(b, c, p)).collect(),
            })
            .collect(),
        currencies,
        rates: &c.exchange_rates.rates,
        rates_date: &c.exchange_rates.date,
        rates_source: &c.exchange_rates.source,
        default_currency,
    }
}

#[tauri::command]
fn match_catalogue(data: tauri::State<'_, Data>, profile: Profile, filters: Filters) -> Results {
    matches(&data, &profile, &filters)
}

pub fn matches(data: &Data, profile: &Profile, filters: &Filters) -> Results {
    find_matches_prepared(&data.prepared, data.catalogue, &data.ease, profile, filters)
}

#[derive(Serialize)]
pub struct ProductDetails {
    brand: Brand,
    cut_id: String,
    cut: String,
    brand_label: Option<String>,
    line: Option<fit_core::catalogue::Line>,
    product: Product,
    /// Each chart's conventions, measuring points and tolerance, with what
    /// it inherits from the brand filled in.
    effective: Vec<Effective>,
    washed: Option<Washed>,
}

/// Everything about one product: its charts with their sources, and its
/// as-sold and after-wash measurements side by side.
#[tauri::command]
fn product_details(data: tauri::State<'_, Data>, product_id: String) -> Result<ProductDetails, String> {
    details(&data, &product_id)
}

pub fn details(data: &Data, product_id: &str) -> Result<ProductDetails, String> {
    let (brand, cut, product) = data.catalogue.product(product_id).ok_or("No such product")?;
    let mut shown = brand.clone();
    shown.cuts.clear();
    Ok(ProductDetails {
        brand: shown,
        cut_id: cut.id.clone(),
        cut: cut.name.clone(),
        brand_label: cut.brand_label.clone(),
        line: cut.line,
        product: product.clone(),
        effective: product.charts.iter().map(|c| c.effective(brand)).collect(),
        washed: after_wash(brand, product),
    })
}

/// A reference pair's own ease against the body, and the room it suggests.
#[tauri::command]
fn reference_pair_ease(data: tauri::State<'_, Data>, body: Body, pair: ReferencePair) -> Option<PairEase> {
    own_ease(&data, &body, &pair)
}

pub fn own_ease(data: &Data, body: &Body, pair: &ReferencePair) -> Option<PairEase> {
    let readings = pair_readings(pair, data.catalogue)?;
    Some(pair_ease(body, &readings, &data.ease))
}

/// The ease table, for showing what each room means.
#[tauri::command]
fn ease_table(data: tauri::State<'_, Data>) -> EaseTable {
    data.ease.clone()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        // Opens brand pages and sources in the default browser: the webview
        // ignores target="_blank" links.
        .plugin(tauri_plugin_opener::init())
        .manage(data())
        .manage(updates::Pending::default())
        .invoke_handler(tauri::generate_handler![
            load_state,
            save_state,
            catalogue_index,
            match_catalogue,
            product_details,
            reference_pair_ease,
            ease_table,
            updates::check_for_update,
            updates::install_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Real Denim Fit");
}

#[cfg(test)]
mod tests {
    use super::read_state;
    use serde_json::json;

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("rdf-state-test-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_missing_file_is_a_first_run() {
        let dir = scratch("missing");
        assert_eq!(
            read_state(&dir.join("real-denim-fit.json")),
            Ok(serde_json::Value::Null)
        );
    }

    #[test]
    fn a_saved_file_is_read_back() {
        let dir = scratch("saved");
        let path = dir.join("real-denim-fit.json");
        std::fs::write(&path, r#"{"version":1,"onboarded":true}"#).unwrap();
        assert_eq!(read_state(&path), Ok(json!({"version": 1, "onboarded": true})));
    }

    #[test]
    fn any_other_read_error_is_reported_not_treated_as_a_first_run() {
        let dir = scratch("unreadable");
        // A folder where the file should be can't be read as a file.
        let path = dir.join("real-denim-fit.json");
        std::fs::create_dir_all(&path).unwrap();
        assert!(read_state(&path).is_err());
    }

    #[test]
    fn a_damaged_file_is_reported() {
        let dir = scratch("damaged");
        let path = dir.join("real-denim-fit.json");
        std::fs::write(&path, "{ not json").unwrap();
        assert!(read_state(&path).is_err());
    }
}
