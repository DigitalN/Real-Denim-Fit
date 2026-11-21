# Releasing Real Denim Fit

How a new version gets from `main` to people's computers. There's a one-time setup first, and then each release is one command: `/release`.

## The pieces

- **CI** (`.github/workflows/ci.yml`) runs on every pull request and every push to `main`. It checks formatting, runs Clippy (Rust's linter) and runs every test, including the catalogue checks, on macOS and on Windows. It also lints the workflow files.
- **The Release workflow** (`.github/workflows/release.yml`) is only ever run by hand. Given a version, it:
  - checks that both version files say that version
  - makes the tag `v<version>`
  - builds the macOS app and the Windows installer
  - signs the update files
  - puts everything in a **draft** release
  - checks the draft

  It never publishes.
- **`/release`** is the Claude Code command that drives all of this and asks you before publishing.

Installed copies check `latest.json` on the latest **published** release every time they open. Drafts are invisible to them, so nothing reaches anyone until you say yes.

## One-time setup

You do this once, on your Mac. It makes the **update signing key**: every installed copy checks each update against it before installing, so nobody else can push an update to your users.

1. **Install the Tauri command-line tool** if it isn't installed already. It takes a few minutes:

   ```sh
   cargo install tauri-cli --version "^2.0" --locked
   ```

2. **Make the key.** It asks for a password. Choose a strong one and save it in your password manager straight away:

   ```sh
   cargo tauri signer generate -w ~/.tauri/real-denim-fit.key
   ```

   This makes two files:
   - `~/.tauri/real-denim-fit.key`: the **private key**. It's secret.
   - `~/.tauri/real-denim-fit.key.pub`: the **public key**. It's safe to share, and it goes in the app.

3. **Add the two secrets to the GitHub repository.** Run these from the repo folder. The second command asks for the password from step 2:

   ```sh
   gh secret set TAURI_SIGNING_PRIVATE_KEY < ~/.tauri/real-denim-fit.key
   ```

   ```sh
   gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD
   ```

   Or add them in the browser, under **Settings → Secrets and variables → Actions → New repository secret**. For the first one, paste the whole contents of the `.key` file.

4. **Put the public key in the app.** Its contents go in `src-tauri/tauri.conf.json` as `plugins.updater.pubkey` (see the list below). Claude can do this for you: it's public.

5. **Back up the private key outside GitHub.** Save the `.key` file and its password in your password manager (1Password can store the file as a document), or on an encrypted drive you keep safe. GitHub secrets can't be read back once they're set.

   **If the private key is lost,** no installed copy can take another update, and everyone has to download and reinstall by hand. **If it leaks,** someone else could sign updates. Never commit it, and never paste it into a chat.

## Making a release

Run `/release` in Claude Code, from an up-to-date `main` with CI passing. It:

1. **Picks the version** from the commits since the last tag, and tells you why:
   - new features or a new brand: minor (0.2.0 → 0.3.0)
   - only fixes and catalogue refreshes: patch (0.2.0 → 0.2.1)
   - 1.0 only when you ask for it
2. **Sets it** in `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml`, then commits `Release <version>` and pushes it to `main`.
3. **Runs the Release workflow** and waits for it.
   - A run that starts cold takes 15–30 minutes. That's the first release, the first after the dependencies or the build settings in the root `Cargo.toml` change, and any release more than 7 days after the last one, because GitHub deletes unused caches after a week.
   - Otherwise the run reuses the last release's compiled dependencies and a ready-made Tauri command-line tool, and only rebuilds the app itself.
4. **Checks the draft.** It must have the `.dmg`, the Windows `-setup.exe` and `latest.json`, and `latest.json` must cover Apple silicon Macs, Intel Macs and Windows.
5. **Writes the release notes**, in the style of Mac Engine Room's: what's new, then install steps for both systems, including the one-time warning on each.
6. **Asks you once**, then publishes. From then on, every copy updates the next time it opens.

You can also run the workflow yourself: **Actions → Release → Run workflow**, from `main`, with the version (for example `0.2.0`). Both files must already say that version. It stops at the draft, and you can publish the draft on the release's page.

### If a release run fails

- **Something flaky**, such as a download timing out: re-run the failed jobs from the run's page, or with `gh run rerun <id> --failed`. The workflow reuses the same tag and draft.
- **A real bug:** fix it in a normal change. Then delete the draft and its tag (`gh release delete v<version> --cleanup-tag`) and run `/release` again. It uses the same version.

### Why people see a warning the first time

There's no paid code signing on either system yet, so each shows a one-time warning on first launch. The release notes explain how to get past it:

- **macOS:** "can't verify" → **System Settings → Privacy & Security → Open Anyway**. The app is signed "ad hoc". That isn't an Apple certificate, but without it, Apple silicon Macs would call the app "damaged" and offer no way to open it.
- **Windows:** "Windows protected your PC" → **More info → Run anyway**.

Updates don't show the warning again. They're checked against the update signing key, not against Apple's or Microsoft's signature.

### The macOS disk image

When someone opens the `.dmg`, Finder shows Real Denim Fit and the Applications folder on a light background, with an arrow between them, the words "Drag Real Denim Fit into Applications", and a card explaining the one-time Open Anyway step. It's made to look like Mac Engine Room's.

- **The background** is `src-tauri/dmg/background.tiff`, drawn by `src-tauri/dmg/make-dmg-background.swift`, which sits next to it. It holds a 1× and a 2× image, so it's sharp on Retina screens. It's committed, so builds don't run the script. After changing the script, regenerate it on a Mac with `swift src-tauri/dmg/make-dmg-background.swift src-tauri/dmg/background.tiff` and commit both.
- **The layout** is `bundle.macOS.dmg` in `src-tauri/tauri.conf.json`: a 660 × 448 pt window (420 pt of content under the title bar), the app at (170, 205) and Applications at (490, 205). The background's arrow, platforms and card are drawn for exactly these positions, so change them together.
- **Tauri does the rest:** it hides the toolbar and status bar, uses the app's icon as the disk's icon, hides the `.app` extension, and compresses the image (UDZO, zlib level 9). Its icons are always 128 pt with 16 pt names, a little larger than Mac Engine Room's 112 pt and 13 pt, and that can't be changed in `tauri.conf.json`.
- **Finder lays out the window**, through AppleScript, during the build. On a Mac, the first local `cargo tauri build` asks to let your terminal control Finder (**System Settings → Privacy & Security → Automation**). On GitHub's Macs, `osascript` already has that permission, and tauri-action sets `TAURI_BUNDLER_DMG_IGNORE_CI=true` so the layout step isn't skipped on CI.

## Settings the workflows depend on

The release workflow assumes the following. They live in files the app's build owns, so whoever edits those files needs to keep them.

### `src-tauri/tauri.conf.json`

```json
{
  "productName": "Real Denim Fit",
  "version": "0.1.0",
  "identifier": "io.github.digitaln.realdenimfit",
  "build": {
    "frontendDist": "../ui"
  },
  "bundle": {
    "active": true,
    "targets": ["app", "dmg", "nsis"],
    "createUpdaterArtifacts": true,
    "macOS": {
      "signingIdentity": "-"
    },
    "windows": {
      "nsis": {
        "installMode": "currentUser"
      }
    }
  },
  "plugins": {
    "updater": {
      "pubkey": "<the whole contents of ~/.tauri/real-denim-fit.key.pub>",
      "endpoints": [
        "https://github.com/DigitalN/Real-Denim-Fit/releases/latest/download/latest.json"
      ]
    }
  }
}
```

What each one is for:

| Setting | Why the release needs it |
|---|---|
| `version` | A plain version such as `"0.1.0"`, not a path to another file. The workflow fails unless it equals the version you asked for and the one in `Cargo.toml`. |
| `productName` | The app's name, "Real Denim Fit". The asset names don't depend on it, because the workflow sets them itself. |
| `bundle.active: true` | Without it there are no installers at all. |
| `bundle.targets` | `app` and `dmg` on macOS, `nsis` on Windows. The workflow asks for exactly these with `--bundles`, so this matters only for local builds. Don't add `msi`: it installs for everyone and needs an administrator. |
| `bundle.createUpdaterArtifacts: true` | Makes the signed update files: `.app.tar.gz` and `.sig` on macOS, the `.sig` for `-setup.exe` on Windows. Without them there's no `latest.json`, and the "Check the draft" job fails. Use `true`, not `"v1Compatible"`. |
| `bundle.macOS.signingIdentity: "-"` | Ad-hoc signing, so Apple silicon Macs show "can't verify" (with Open Anyway) instead of "damaged". The workflow also sets `APPLE_SIGNING_IDENTITY=-`, so releases are covered either way. This setting makes local builds match. |
| `bundle.windows.nsis.installMode: "currentUser"` | Installs into the user's own folder (`%LOCALAPPDATA%`), so the installer and every update run without an administrator password. It's Tauri's default; set it anyway so nobody changes it by accident. Not `perMachine` or `both`. |
| `plugins.updater.pubkey` | The public half of the update signing key, pasted in as text. It can't be a file path. It must match the private key in the secrets, or every update is refused. |
| `plugins.updater.endpoints` | Exactly the URL above. GitHub's "latest" skips drafts and pre-releases, so only a published release reaches people. |

Optional: `bundle.macOS.dmg` sets how the disk image's window looks (see [The macOS disk image](#the-macos-disk-image)). Without it the release still works, with a plain white window.

Optional: `plugins.updater.windows.installMode` sets how an update installs on Windows. The default, `"passive"`, shows a small progress window and needs no clicks. `"quiet"` shows nothing, and works only because the app is installed per user. The workflows work with either.

### Elsewhere

- **`src-tauri/Cargo.toml`**: `version` under `[package]` must be a plain version equal to `tauri.conf.json`'s, not `version.workspace = true`.
- **`Cargo.lock`** is committed. CI and the release build use `--locked`.
- **The Tauri project is at `src-tauri/`**, with the Cargo workspace at the repo root, and `ui/` holds the screens with no npm. The workflow runs `cargo tauri build` from the root.
- **The updater plugin** (`tauri-plugin-updater`) is set up in the app. It's what reads `latest.json` and checks the signature when the app opens.
