//! Builds the catalogue and ease table into the binary.
//!
//! The catalogue comes from `catalogue/` at the repo root, or from the
//! folder in `RDF_CATALOGUE_DIR` when it's set (used to try fixtures).
use std::{env, fs, path::Path, path::PathBuf};

fn main() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    println!("cargo:rerun-if-env-changed=RDF_CATALOGUE_DIR");
    let catalogue: PathBuf = env::var("RDF_CATALOGUE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| repo.join("catalogue"));
    println!("cargo:rerun-if-changed={}", catalogue.display());

    let mut files: Vec<(String, PathBuf)> = vec![];
    if let Ok(dir) = fs::read_dir(&catalogue) {
        for entry in dir.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                println!("cargo:rerun-if-changed={}", path.display());
                for f in fs::read_dir(&path).unwrap().flatten() {
                    let fname = f.file_name().to_string_lossy().to_string();
                    // Only the brand's own JSON files: never sub-folders.
                    if fname.ends_with(".json") && f.path().is_file() {
                        files.push((format!("{name}/{fname}"), f.path()));
                    }
                }
            } else if name == "exchange-rates.json" {
                files.push((name, path));
            }
        }
    }
    files.sort();
    let mut out = String::from("pub const CATALOGUE_FILES: &[(&str, &str)] = &[\n");
    for (rel, path) in &files {
        println!("cargo:rerun-if-changed={}", path.display());
        let path = path.canonicalize().unwrap();
        out += &format!("    ({rel:?}, include_str!({:?})),\n", path.display().to_string());
    }
    out += "];\n";
    let ease = repo
        .join("data/ease-table.json")
        .canonicalize()
        .expect("data/ease-table.json");
    println!("cargo:rerun-if-changed={}", ease.display());
    out += &format!(
        "pub const EASE_TABLE: &str = include_str!({:?});\n",
        ease.display().to_string()
    );
    let levis = repo
        .join("data/levis-body-chart.json")
        .canonicalize()
        .expect("data/levis-body-chart.json");
    println!("cargo:rerun-if-changed={}", levis.display());
    out += &format!(
        "pub const LEVIS_BODY_CHART: &str = include_str!({:?});\n",
        levis.display().to_string()
    );
    fs::write(Path::new(&env::var("OUT_DIR").unwrap()).join("embedded_files.rs"), out).unwrap();
}
