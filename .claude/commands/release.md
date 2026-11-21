---
description: Release a new version of Real Denim Fit (build, draft, notes, publish)
argument-hint: "[version, only to override the one Claude picks]"
---

Release a new version of Real Denim Fit. The user ran `/release`, which is their go-ahead to commit the version bump, push it to `main` and run the Release workflow. Publishing is different: ask once, in step 6, and publish only on a clear yes.

Read `docs/RELEASING.md` first. The workflow is `.github/workflows/release.yml`. It makes a **draft** release and never publishes.

The user doesn't know Rust or GitHub Actions. Explain each decision in a sentence of plain English, and keep the jargon out.

## 0. Check where things stand

- Run `git fetch origin --tags`. You must be on `main`, with no uncommitted changes, level with `origin/main`. If not, stop and say what's off.
- Look at the latest CI run on `main`: `gh run list --workflow ci.yml --branch main --limit 1`. If it failed, or is still running, stop and say so. Don't release a broken `main`.

## 1. Pick the version

- Find the last release tag: `git describe --tags --abbrev=0 --match 'v*'`.
- List what changed since: `git log <tag>..HEAD --oneline --no-merges`, and look at the diffs where a subject line isn't clear.
- If there's no tag yet, this is the first release. Use the version already in `src-tauri/tauri.conf.json` (normally `0.1.0`).
- Otherwise bump from the last tag, following semantic versioning while the app is below 1.0:
  - **Minor** (0.2.0 → 0.3.0): anything new the user can see or use: a new screen or setting, a new brand in the catalogue, a change to how matching works, a new filter.
  - **Patch** (0.2.0 → 0.2.1): only fixes and catalogue refreshes: updated numbers, prices, discontinued products, exchange rates, small wording changes.
  - **Major** (→ 1.0.0): only when the user asks for it.
- If nothing changed that a user would notice (only docs, CI or tests), say so and stop. There's nothing to release.
- If the user gave a version as an argument (`$ARGUMENTS`), use it, but still say whether it matches what the changes suggest.
- Tell the user the version and why, in one or two sentences. For example: "0.3.0, because it adds Momotaro to the catalogue and a new leg-shape filter."

## 2. Set the version, commit, push

- Set the version in both places:
  - `src-tauri/tauri.conf.json`: the top-level `"version"`
  - `src-tauri/Cargo.toml`: `version` under `[package]`
- Run `cargo update --workspace` so `Cargo.lock` picks up the new version. Without that, the `--locked` builds in CI fail.
- `git diff` must show only those three files, and only the version lines. If the files already say this version (first release), skip the commit.
- Commit as `Release <version>` and `git push origin main`. If the push is refused (for example, branch protection), stop and tell the user.

## 3. Run the workflow and wait

- Start it: `gh workflow run release.yml --ref main -f version=<version>`.
- Find the run for the commit you pushed: `gh run list --workflow release.yml --limit 5 --json databaseId,headSha,status,createdAt`. Pick the one whose `headSha` is `git rev-parse HEAD`. It can take a few seconds to appear.
- Wait with `gh run watch <id> --exit-status`. A build takes 15–30 minutes, so run it in the background and don't poll.
- If it fails, show the user the failing step from `gh run view <id> --log-failed` and explain it plainly. Don't publish anything.
  - **A flaky step**, such as a network error or the DMG step timing out: `gh run rerun <id> --failed` is safe. The workflow reuses the same tag and draft.
  - **A fault in the code:** the tag already points at the old commit. Fix it in a normal change first. Then ask the user before deleting the draft and its tag with `gh release delete v<version> --cleanup-tag --yes`, and start again from step 2 with the same version.

## 4. Check the draft

The workflow's last job, "Check the draft", already checks `latest.json` against the uploaded files. Look for yourself as well:

- `gh release view v<version> --json isDraft,assets --jq '{isDraft, assets: [.assets[].name]}'`
- It must still be a draft. It must have:
  - exactly one `.dmg` (`Real-Denim-Fit-<version>-macOS-universal.dmg`)
  - exactly one Windows installer ending in `-setup.exe`
  - `latest.json`
  - the macOS update file `.app.tar.gz` and the `.sig` files
- Download `latest.json` with `gh release download v<version> -p latest.json -D <scratchpad>`. Check that `version` is right, and that `platforms` has `darwin-aarch64`, `darwin-x86_64` and `windows-x86_64`, each with a `url` and a `signature`.
- If anything is missing, stop and tell the user. Don't publish.

## 5. Write the release notes

Copy the style of Mac Engine Room's release notes. Read them first with `gh release view v1.5.0 -R DigitalN/mac-engine-room`, and match their tone: short, plain, second person, the specific thing each change does for you, and **bold** for buttons and settings. Don't copy their content.

Write the notes to a file in your scratchpad, in this shape:

```markdown
## What's new

- **<Short name>.** <What it does for you, in one or two sentences.>
- …

## Install

### macOS

1. **Download** `Real-Denim-Fit-<version>-macOS-universal.dmg` below and open it.
2. **Drag Real Denim Fit into Applications.**
3. **Open Real Denim Fit** from Applications.

The first time you open it, macOS may say it can't verify Real Denim Fit. Click **Done**, then go to **System Settings → Privacy & Security**, scroll down, and click **Open Anyway**. You only do this once: Real Denim Fit isn't distributed through Apple, so Apple hasn't checked it. All of its code is in this repository.

### Windows

1. **Download** `<exact name of the -setup.exe asset>` below and open it.
2. If Windows says **Windows protected your PC**, click **More info**, then **Run anyway**. You only do this once: the installer isn't signed with a paid certificate. All of its code is in this repository.
3. Follow the installer. It installs just for you, so it doesn't ask for an administrator password.

After this, Real Denim Fit updates itself when you open it.
```

- **What's new** comes from the commits since the last tag. Describe what people can now do, not how it was built: no commit hashes, crate names or file paths.
  - Catalogue changes: name the brands and products that were added or refreshed.
  - For the first release, the section describes the app itself: measure yourself, choose a fit, optionally add a pair you know, and see which raw denim and which size come closest. Say that nothing you enter leaves your computer, and that the only network use is checking for updates.
- Use the exact asset file names from step 4.
- Don't claim anything you haven't checked, such as "tested on macOS 27". Leave out what you don't know.
- Put the notes on the draft: `gh release edit v<version> --notes-file <file>`.

## 6. Ask once, then publish

- Show the user the version, the asset list and the notes. Then ask one question: "Publish Real Denim Fit <version> now?" Once it's published, every installed copy updates to it the next time it opens.
- Only on a clear yes: `gh release edit v<version> --draft=false --latest`.
- Then confirm the updater can see it: `curl -fsSL https://github.com/DigitalN/Real-Denim-Fit/releases/latest/download/latest.json | jq -r .version` must print the new version.
- Report the release's URL (`gh release view v<version> --json url --jq .url`).
- If the user says no, or wants changes, leave it as a draft and tell them it's there to review.
