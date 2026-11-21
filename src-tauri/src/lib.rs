//! The Real Denim Fit app: the screens in `ui/` call these commands.
//!
//! Everything the user enters is stored in one JSON file in the app's data
//! folder on this computer. The only network access is the update check.

mod currency;
mod updates;

use std::fs;
use std::path::PathBuf;

use fit_core::after_wash::{after_wash, Washed};
use fit_core::catalogue::{Brand, Catalogue, Product};
use fit_core::ease::EaseTable;
use fit_core::matching::{
    find_matches_prepared, pair_readings, prepare, Filters, Prepared, Results,
};
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
    let path = state_file(&app)?;
    match fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text).map_err(|e| e.to_string()),
        Err(_) => Ok(Value::Null),
    }
}

#[tauri::command]
fn save_state(app: tauri::AppHandle, state: Value) -> Result<(), String> {
    let path = state_file(&app)?;
    let tmp = path.with_extension("json.tmp");
    fs::write(
        &tmp,
        serde_json::to_vec_pretty(&state).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())
}

#[derive(Serialize)]
pub struct ProductSummary<'a> {
    id: &'a str,
    cut: &'a str,
    name: &'a str,
    fabric: &'a str,
    fabric_weight_oz: Option<f64>,
    fabric_state: Option<&'a str>,
    composition: &'a str,
    elastane_percent: f64,
    sanforized: Option<bool>,
    on_brands_site: bool,
    url: &'a str,
    list_price: Option<&'a fit_core::catalogue::ListPrice>,
    sizes: Vec<&'a str>,
}

#[derive(Serialize)]
pub struct BrandSummary<'a> {
    id: &'a str,
    name: &'a str,
    site: &'a str,
    says_where_it_measures: bool,
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

fn summary(p: &Product) -> ProductSummary<'_> {
    let sizes = p
        .charts
        .first()
        .map(|c| c.sizes.iter().map(|s| s.size.as_str()).collect())
        .unwrap_or_default();
    ProductSummary {
        id: &p.id,
        cut: &p.cut,
        name: &p.name,
        fabric: &p.fabric,
        fabric_weight_oz: p.fabric_weight_oz,
        fabric_state: p.fabric_state.as_deref(),
        composition: &p.composition,
        elastane_percent: p.elastane_percent,
        sanforized: p.sanforized,
        on_brands_site: p.on_brands_site,
        url: &p.url,
        list_price: p.list_price.as_ref(),
        sizes,
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
                says_where_it_measures: b.says_where_it_measures,
                sold_in_lengths: b.sold_in_lengths,
                products: b.products.iter().map(summary).collect(),
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
    product: Product,
    washed: Option<Washed>,
}

/// Everything about one product: its charts with their sources, and its
/// as-sold and after-wash measurements side by side.
#[tauri::command]
fn product_details(
    data: tauri::State<'_, Data>,
    product_id: String,
) -> Result<ProductDetails, String> {
    details(&data, &product_id)
}

pub fn details(data: &Data, product_id: &str) -> Result<ProductDetails, String> {
    let (brand, product) = data
        .catalogue
        .product(product_id)
        .ok_or("No such product")?;
    let mut brand = brand.clone();
    brand.products.clear();
    Ok(ProductDetails {
        brand,
        product: product.clone(),
        washed: after_wash(product),
    })
}

/// A reference pair's own ease against the body, and the room it suggests.
#[tauri::command]
fn reference_pair_ease(
    data: tauri::State<'_, Data>,
    body: Body,
    pair: ReferencePair,
) -> Option<PairEase> {
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
