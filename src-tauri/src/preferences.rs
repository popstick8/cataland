use cataland_core::preferences::Preferences;
use tauri::{AppHandle, Emitter, State, WebviewWindow};

use crate::{desktop::Desktop, storage};

#[tauri::command]
pub fn preferences(
    preferences: Preferences,
    window: WebviewWindow,
    app: AppHandle,
    desktop: State<'_, Desktop>,
) -> Result<(), String> {
    preferences.validate()?;
    let mut state = desktop.state.lock().map_err(|error| error.to_string())?;
    window
        .set_zoom(preferences.scale)
        .map_err(|error| format!("界面缩放失败：{error}"))?;
    storage::write(&desktop.directory.join("preferences.json"), &preferences)?;
    state.view.preferences = preferences;
    app.emit("session", &state.view)
        .map_err(|error| error.to_string())
}
