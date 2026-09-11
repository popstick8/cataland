mod desktop;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(desktop::Desktop::new(app.path().app_data_dir()?)?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            desktop::session,
            desktop::host,
            desktop::room_action,
            desktop::leave
        ])
        .run(tauri::generate_context!())
        .expect("Failed to run Cataland");
}
