# Tauri 2 for the app

The app is a Tauri 2 app: the matching, the catalogue and the updater are Rust, and the screens are HTML and CSS in the system webview. We picked it over all-Rust UI toolkits (egui, iced, Slint) because Tauri comes with an updater that reads GitHub Releases and checks a signature on every download, and an official GitHub Action that builds the macOS and Windows releases. With the others we'd write and maintain both ourselves, for two operating systems.

## Consequences

- Updates are verified with Tauri's own update signing key, not the operating system's code signature, so updates stay safe on a Windows build that isn't code-signed.
- The private half of that key lives in the repository's GitHub secrets. If it's lost, installed copies can't take updates any more and everyone has to reinstall by hand, so it needs a backup outside GitHub.
