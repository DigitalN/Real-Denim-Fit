//! Updates: on every launch the screens ask for the latest published
//! release on GitHub, waiting a few seconds at most. A newer one is
//! downloaded, checked against the update signing key, installed, and the
//! app reopens. Offline or slow means the app just opens.

use std::sync::Mutex;
use std::time::Duration;

use tauri_plugin_updater::{Update, UpdaterExt};

/// How long the launch check waits for GitHub.
const CHECK_TIMEOUT: Duration = Duration::from_secs(4);
/// How long the download may take once an update is found.
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(600);

#[derive(Default)]
pub struct Pending(Mutex<Option<Update>>);

/// The newer version's number, or `None` when there's nothing newer, the
/// computer is offline, or GitHub is slow.
#[tauri::command]
pub async fn check_for_update(
    app: tauri::AppHandle,
    pending: tauri::State<'_, Pending>,
) -> Result<Option<String>, String> {
    if cfg!(debug_assertions) {
        // Development builds never replace themselves.
        return Ok(None);
    }
    let Ok(updater) = app.updater_builder().timeout(CHECK_TIMEOUT).build() else {
        return Ok(None);
    };
    match updater.check().await {
        Ok(Some(update)) => {
            let version = update.version.clone();
            *pending.0.lock().unwrap() = Some(update);
            Ok(Some(version))
        }
        _ => Ok(None),
    }
}

/// Downloads and installs the update found by `check_for_update`, then
/// reopens the app.
#[tauri::command]
pub async fn install_update(
    app: tauri::AppHandle,
    pending: tauri::State<'_, Pending>,
) -> Result<(), String> {
    let update = pending
        .0
        .lock()
        .unwrap()
        .take()
        .ok_or("No update to install")?;
    let mut update = update;
    update.timeout = Some(DOWNLOAD_TIMEOUT);
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|e| e.to_string())?;
    app.restart();
}
