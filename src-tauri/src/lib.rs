mod badge;
mod linux_desktop;
mod notify;
mod profile;
mod settings;
mod tray;
mod unread;
mod window;

use profile::DEFAULT_PROFILE;
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_window_state::StateFlags;
use unread::UnreadState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Must be first: a second launch would open the same WebKit data
        // directories and corrupt profiles.
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            tray::show_all(app);
        }))
        .plugin(
            // Not VISIBLE/MINIMIZED: hide_to_tray minimizes+hides, and
            // persisting those would restore windows invisible after restart.
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(StateFlags::SIZE | StateFlags::POSITION | StateFlags::MAXIMIZED)
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
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
            notify::notify_profile,
            settings::get_app_settings,
            settings::set_app_settings,
        ])
        .setup(|app| {
            // Wayland/KDE: app_id → hicolor icon + .desktop (fixes monogram "W")
            #[cfg(target_os = "linux")]
            linux_desktop::install();

            let start_hidden = settings::load(app.handle()).start_hidden
                || std::env::args().any(|a| a == "--hidden");
            if !start_hidden {
                window::open_profile_window(app.handle(), DEFAULT_PROFILE)
                    .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
            }

            tray::setup_tray(app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Wangsap");
}
