# Repo layout

Where everything lives. Parts of the first build are being written in parallel, so these paths and names are fixed: keep to them.

| Path | What it is |
|---|---|
| `Cargo.toml` | Cargo workspace at the root. `cargo test --workspace` runs every test, including catalogue validation. |
| `src-tauri/` | The Tauri 2 app crate, `real-denim-fit`. Product name "Real Denim Fit", bundle identifier `io.github.digitaln.realdenimfit`. |
| `crates/` | Rust library crates, such as the matching core. |
| `ui/` | The screens: static HTML, CSS and JavaScript, with no bundler and no npm. `tauri.conf.json` has `"frontendDist": "../ui"`. |
| `ui/guide/` | The measuring guide: one SVG drawing per step, plus `steps.json` (format below). |
| `catalogue/` | The catalogue, in the format in [catalogue-format.md](catalogue-format.md). The app builds in only `catalogue/exchange-rates.json` and `catalogue/<brand>/*.json`. |
| `tools/catalogue/<brand-id>/` | Each brand's extraction scripts, used by `/update-catalogue`: Python 3 with the standard library only, plus browser JavaScript snippets. Never built into the app or read by validation. |
| `data/ease-table.json` | The ease table, with a source on every number. |
| `.github/workflows/` | `ci.yml` (on pull requests) and `release.yml` (run by hand). |
| `.claude/commands/` | `update-catalogue.md` and `release.md`. |

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
- Distances give cm first, then inches: "30.5 cm (12″)".
- Drawings are simple line art that uses `currentColor`, so they work in light and dark mode. They have a `viewBox` and no embedded text; the words live in `steps.json`.
