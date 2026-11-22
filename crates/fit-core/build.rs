//! Builds the catalogue and ease table into the binary.
//!
//! The ease table and Levi's body chart come from this crate's own `data/`
//! folder. The catalogue comes from `catalogue/` at the repo root, or from the
//! folder in `RDF_CATALOGUE_DIR` when it's set (used to try fixtures). A
//! relative `RDF_CATALOGUE_DIR` is taken from the repo root. A folder that
//! can't be read stops the build rather than building in an empty catalogue.
use std::{env, fs, path::Path, path::PathBuf};

fn main() {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo = crate_dir.join("../..");
    println!("cargo:rerun-if-env-changed=RDF_CATALOGUE_DIR");
    let catalogue: PathBuf = match env::var("RDF_CATALOGUE_DIR") {
        Ok(dir) if Path::new(&dir).is_absolute() => PathBuf::from(dir),
        Ok(dir) => repo.join(dir),
        Err(_) => repo.join("catalogue"),
    };
    println!("cargo:rerun-if-changed={}", catalogue.display());

    let mut files: Vec<(String, PathBuf)> = vec![];
    let dir = fs::read_dir(&catalogue)
        .unwrap_or_else(|e| panic!("can't read the catalogue folder {}: {e}", catalogue.display()));
    {
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
    let ease = crate_dir
        .join("data/ease-table.json")
        .canonicalize()
        .expect("crates/fit-core/data/ease-table.json");
    println!("cargo:rerun-if-changed={}", ease.display());
    out += &format!(
        "pub const EASE_TABLE: &str = include_str!({:?});\n",
        ease.display().to_string()
    );
    let levis = crate_dir
        .join("data/levis-body-chart.json")
        .canonicalize()
        .expect("crates/fit-core/data/levis-body-chart.json");
    println!("cargo:rerun-if-changed={}", levis.display());
    out += &format!(
        "pub const LEVIS_BODY_CHART: &str = include_str!({:?});\n",
        levis.display().to_string()
    );
    fs::write(Path::new(&env::var("OUT_DIR").unwrap()).join("embedded_files.rs"), out).unwrap();
}
