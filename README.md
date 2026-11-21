# Real Denim Fit

A free desktop app for macOS and Windows that finds raw denim jeans worth looking into, in the right size.

You measure your body, choose how you want your jeans to fit, and optionally add a pair you already know. The app compares that with the measurements brands publish for every size, as each pair will be after its first wash, and shows which jeans come closest and how far off each measurement is. It never says where to buy them.

Brands in the first version: Iron Heart, Naked & Famous, 3sixteen, Nudie, Momotaro and Rogue Territory.

Everything you enter stays on your computer. The only thing the app sends over the internet is a check for updates when it opens.

## Install

Download the latest release from the [Releases page](https://github.com/DigitalN/Real-Denim-Fit/releases/latest).

### macOS (Apple silicon and Intel)

1. Download `Real-Denim-Fit-…-macOS-universal.dmg` and open it.
2. Drag **Real Denim Fit** into **Applications**.
3. Open it. The first time, macOS says it can't verify the app, because the app isn't paid-signed with Apple. Click **Done** (not Move to Trash).
4. Open **System Settings → Privacy & Security**, scroll down, and click **Open Anyway** next to the message about Real Denim Fit. Confirm with your password or Touch ID.

You only do this once. Updates install themselves after that.

### Windows (64-bit, including ARM laptops)

1. Download `Real-Denim-Fit-…-Windows-x64-setup.exe` and run it.
2. Windows may say **"Windows protected your PC"**, because the installer isn't paid-signed. Click **More info**, then **Run anyway**.
3. The installer doesn't need administrator rights.

You only do this once. Updates install themselves after that.

## Updates

Each time it opens, the app asks GitHub for the latest release, waiting a few seconds at most. If there's a newer one, it shows "Updating to …", installs it and reopens. If you're offline it just opens. Every update is checked against the project's own signing key before it's installed.

## Building it yourself

You need [Rust](https://rustup.rs) and the Tauri CLI:

```bash
cargo install tauri-cli --version "^2.0" --locked
```

```bash
cargo tauri dev
```

```bash
cargo test --workspace
```

`cargo tauri build` makes the app for the computer you're on. See [CLAUDE.md](CLAUDE.md) and [docs/](docs/) for how the project is put together.

## Licence

GPL-3.0. See [LICENSE](LICENSE).
