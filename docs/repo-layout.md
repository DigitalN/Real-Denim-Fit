# Repo layout

Where everything lives. Paths and names can change, but only in a PR that updates this file in the same change, so this table always matches the repo.

| Path | What it is |
|---|---|
| `Cargo.toml`, `Cargo.lock` | Cargo workspace at the root. `cargo test --workspace` runs every test, including catalogue validation. |
| `.rustfmt.toml` | `cargo fmt` settings (`max_width = 120`). |
| `README.md`, `LICENSE` | What the app is, and its licence. |
| `src-tauri/` | The Tauri 2 app crate, `real-denim-fit`. Product name "Real Denim Fit", bundle identifier `io.github.digitaln.realdenimfit`. |
| `src-tauri/examples/preview.rs` | Serves the screens at http://localhost:1420 with the same Rust behind them, without building the app (`cargo run -p real-denim-fit --example preview`). What you enter is kept in `target/preview-state.json`, or in the file `RDF_STATE_FILE` names. `RDF_CATALOGUE_DIR` (at build time) and `RDF_GUIDE_DIR` point it at another catalogue or guide folder. |
| `src-tauri/dmg/` | The macOS disk image's window background, `background.tiff`, and `make-dmg-background.swift`, the script that draws it. The script is run by hand on a Mac and never built into the app. See [RELEASING.md](RELEASING.md#the-macos-disk-image). |
| `ui/` | The screens: static HTML, CSS and JavaScript, with no bundler and no npm. `tauri.conf.json` has `"frontendDist": "../ui"`. |
| `ui/guide/` | The measuring guide: one SVG drawing per step, plus `steps.json` (format below). |
| `crates/fit-core/` | The matching core: matching, catalogue and ease table. Its tests are in `crates/fit-core/tests/`. |
| `crates/fit-core/data/` | `ease-table.json`, the ease table, with a source on every number, and `levis-body-chart.json`, which only the calibration test reads. `crates/fit-core/build.rs` builds both into the app. |
| `catalogue/` | The catalogue, in the format in [catalogue-format.md](catalogue-format.md). The app builds in only `catalogue/exchange-rates.json` and `catalogue/<brand>/*.json`. |
| `catalogue/<brand-id>/tools/` | Each brand's extraction scripts, used by `/update-catalogue`: Python 3 with the standard library only, plus browser JavaScript snippets. Run them from the repo root. The build (`crates/fit-core/build.rs`) and validation (`Catalogue::from_dir`) read only the JSON files directly inside a brand folder and skip sub-folders, so nothing here, not even a JSON file such as `naked-and-famous/tools/decisions.json`, reaches the app. |
| `docs/` | [PLAN.md](PLAN.md) (the design), [GLOSSARY.md](GLOSSARY.md) (the words), [catalogue-format.md](catalogue-format.md), [RELEASING.md](RELEASING.md) and this file. |
| `docs/adr/` | The decisions behind the design, one file each. |
| `docs/screenshots/` | The README's pictures, `<name>-light.png` and `<name>-dark.png`; `take-screenshots.swift`, the script that draws them, run by hand on a Mac (its header says how); and `example-profile.json`, the made-up profile they show, in the format the app saves. The script reads a copy of the profile, never the file itself. |
| `docs/research/` | The facts the design rests on: what brands publish and the ease research. |
| `docs/verification/` | Dated records of checks on a build. They describe the repo as it was on that date, so their paths aren't updated. |
| `.github/workflows/` | `ci.yml` (on pull requests) and `release.yml` (run by hand). |
| `.claude/CLAUDE.md` | Instructions Claude Code reads in every session here. |
| `.claude/commands/` | `update-catalogue.md` and `release.md`. |
| `.claude/launch.json` | The `preview` dev server config, on port 1420, and `preview-example`, on port 1421, which shows `target/example-state.json`: copy `docs/screenshots/example-profile.json` there first. |

## Building

- **Tauri CLI:** install with `cargo install tauri-cli --version "^2.0" --locked`, then build with `cargo tauri build`. There's no Node toolchain.
- **Version:** kept in two places, `src-tauri/tauri.conf.json` (`version`) and `src-tauri/Cargo.toml`, and the release process sets both. Tags are `v<version>`, for example `v0.1.0`.

## Updates

- **Updater endpoint:** `https://github.com/DigitalN/Real-Denim-Fit/releases/latest/download/latest.json`. GitHub's "latest" ignores drafts and pre-releases.
- **Signing secrets:** the update signing key is stored as `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. Its public key goes in `src-tauri/tauri.conf.json`.

## `ui/guide/steps.json`

One entry per guide screen, in the order they're shown.

```json
[
  {
    "id": "body-leg-12in",
    "for": "body",
    "measurement": "knee",
    "point": "12in-below-crotch",
    "title": "Leg, 30.5 cm (12″) below the crotch",
    "steps": ["…", "…"],
    "mistake": "…",
    "image": "body-leg-12in.svg"
  }
]
```

- `for` is `"body"` or `"pair"`.
- `measurement` and `point` use the keys and measuring points from [catalogue-format.md](catalogue-format.md). `point` is `null` where there's only one place to measure.
- `fields` is optional. Without it, a screen with a `measurement` asks for one value. With it, the screen asks for each field in order, as `{ "id", "label", "optional" }`. The `body-inseam` screen uses it for `inseam` ("To where the hem should sit") and an optional `inseam_extra` ("Extra for cuffs or stacking"). The app aims at an inseam of at least their sum, an empty extra counting as 0. The halfway point on the leg screens is half of `inseam` alone.
- Distances give cm first, then inches: "30.5 cm (12″)".
- Drawings are simple line art that uses `currentColor`, so they work in light and dark mode. They have a `viewBox` and no embedded text; the words live in `steps.json`.
