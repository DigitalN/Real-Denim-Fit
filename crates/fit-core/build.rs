//! Builds the catalogue and ease table into the binary.
use std::{env, fs, path::Path};

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../catalogue");
    let root = root.canonicalize().expect("catalogue folder");
    println!("cargo:rerun-if-changed={}", root.display());
    let brands_dir = root.join("brands");
    println!("cargo:rerun-if-changed={}", brands_dir.display());
    let mut files: Vec<_> = fs::read_dir(&brands_dir)
        .map(|d| d.filter_map(|e| e.ok()).map(|e| e.path()).collect())
        .unwrap_or_default();
    files.retain(|p| p.extension().is_some_and(|e| e == "json"));
    files.sort();
    let mut out = String::from("pub const BRAND_FILES: &[(&str, &str)] = &[\n");
    for f in &files {
        println!("cargo:rerun-if-changed={}", f.display());
        out += &format!(
            "    ({:?}, include_str!({:?})),\n",
            f.file_name().unwrap().to_string_lossy(),
            f.display().to_string()
        );
    }
    out += "];\n";
    for (name, file) in [
        ("EXCHANGE_RATES", "exchange-rates.json"),
        ("EASE_TABLE", "ease-table.json"),
    ] {
        let path = root.join(file);
        println!("cargo:rerun-if-changed={}", path.display());
        out += &format!(
            "pub const {name}: &str = include_str!({:?});\n",
            path.display().to_string()
        );
    }
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("embedded_files.rs"),
        out,
    )
    .unwrap();
}
