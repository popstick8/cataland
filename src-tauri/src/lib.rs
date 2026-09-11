mod desktop;
mod discovery;
mod network;
mod preferences;
mod storage;

use cataland_core::Text;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            let desktop = desktop::Desktop::new(app.path().app_data_dir()?)?;
            if let Some(window) = app.get_webview_window("main") {
                window.set_zoom(
                    desktop
                        .state
                        .lock()
                        .map_err(|error| Text::from(error.to_string()))?
                        .view
                        .preferences
                        .scale,
                )?;
            }
            app.manage(desktop);
            discovery::start(app.handle().clone())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            desktop::session,
            preferences::preferences,
            desktop::host,
            desktop::join,
            desktop::resume,
            desktop::room_action,
            desktop::leave
        ])
        .run(tauri::generate_context!())
        .expect("Failed to run Cataland");
}
