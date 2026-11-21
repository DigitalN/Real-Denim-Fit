# Real Denim Fit

Desktop app (Tauri 2) that recommends raw denim jeans in a size from body measurements, a desired fit and an optional reference pair.

- **Design:** read [docs/PLAN.md](../docs/PLAN.md) before changing behaviour. The decisions behind it are in [docs/adr/](../docs/adr/); check the matching ADR before reversing one.
- **Words:** use [docs/GLOSSARY.md](../docs/GLOSSARY.md)'s terms in code, data and UI, and avoid the words it lists under _Avoid_.
- **Catalogue data:** follow [docs/catalogue-format.md](../docs/catalogue-format.md) exactly. [docs/research/brand-size-charts.md](../docs/research/brand-size-charts.md) says what each brand publishes; each brand's `catalogue/<brand>/EXTRACTING.md` says how to read it.
- **Ease table:** `data/ease-table.json`, background in [docs/research/ease.md](../docs/research/ease.md). Change it only by PR; the calibration test (`crates/fit-core/tests/calibration.rs`) must stay green.
- **Where things live:** [docs/repo-layout.md](../docs/repo-layout.md). Those paths and names are fixed.

## Working here

- The matching core is `crates/fit-core`, built test-first: add behaviour through a failing test in `crates/fit-core/tests/` first.
- `cargo test --workspace` runs everything, including catalogue validation. CI also runs `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all --check`.
- To see the screens without building the app, run `cargo run -p real-denim-fit --example preview` and open http://localhost:1420 (same Rust behind them). `RDF_CATALOGUE_DIR` and `RDF_GUIDE_DIR` point it at other folders.
- The app never calls Claude, never adds shop links, stock, sale prices or product photos, and its only network use is the update check (ADRs 0002, 0003, 0007).
- Catalogue updates: `/update-catalogue <brand>`. Releases: `/release`.
