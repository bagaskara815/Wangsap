mod badge;
mod linux_desktop;
mod profile;
mod tray;
mod unread;
mod window;

use profile::DEFAULT_PROFILE;
use unread::UnreadState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .manage(UnreadState::new())
        .invoke_handler(tauri::generate_handler![
            profile::list_profiles,
            profile::create_profile,
            profile::create_next_profile,
            profile::rename_profile,
            profile::delete_profile,
            window::open_profile,
            window::show_profile,
            window::hide_all_profiles,
            window::open_settings,
            window::open_external,
            unread::set_unread_count,
            unread::get_unread_total,
        ])
        .setup(|app| {
            // Wayland/KDE: app_id → hicolor icon + .desktop (fixes monogram "W")
            #[cfg(target_os = "linux")]
            linux_desktop::install();

            window::open_profile_window(app.handle(), DEFAULT_PROFILE)
                .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

            tray::setup_tray(app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Wangsap");
}
