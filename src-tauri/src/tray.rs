use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

use crate::badge;
use crate::profile;
use crate::window;

pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let menu = build_menu(app)?;
    let icon = match badge::make_tray_image(0) {
        Ok(i) => i,
        Err(_) => app.default_window_icon().unwrap().clone(),
    };

    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .menu(&menu)
        .tooltip("Wangsap")
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            let id = event.id.as_ref();
            match id {
                "show_all" => show_all(app),
                "hide_all" => {
                    let _ = window::hide_all_profiles(app.clone());
                }
                "new_account" => {
                    if let Ok(name) = profile::next_account_name(app) {
                        let _ = profile::profile_dir(app, &name);
                        let _ = window::open_profile_window(app, &name);
                        let _ = rebuild_tray_menu(app);
                    }
                }
                "settings" => {
                    let _ = window::open_settings_window(app);
                }
                "quit" => app.exit(0),
                other if other.starts_with("profile:") => {
                    let name = other.trim_start_matches("profile:");
                    let _ = window::open_profile_window(app, name);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_all(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

pub fn rebuild_tray_menu(app: &AppHandle) -> tauri::Result<()> {
    if let Some(tray) = app.tray_by_id("main-tray") {
        let menu = build_menu(app)?;
        tray.set_menu(Some(menu))?;
    }
    Ok(())
}

/// Update tray tooltip + badged icon from total unread.
pub fn refresh_unread(app: &AppHandle, total: u32) {
    let Some(tray) = app.tray_by_id("main-tray") else {
        return;
    };

    let tip = if total == 0 {
        "Wangsap".to_string()
    } else if total == 1 {
        "Wangsap — 1 unread".to_string()
    } else {
        format!("Wangsap — {total} unread")
    };
    let _ = tray.set_tooltip(Some(&tip));

    if let Ok(icon) = badge::make_tray_image(total) {
        let _ = tray.set_icon(Some(icon));
    }
}

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let profiles = profile::list_profile_names(app)
        .unwrap_or_else(|_| vec![profile::DEFAULT_PROFILE.to_string()]);

    let mut profile_items: Vec<MenuItem<tauri::Wry>> = Vec::new();
    for name in &profiles {
        let id = format!("profile:{name}");
        let label = if name == profile::DEFAULT_PROFILE {
            "Default".to_string()
        } else {
            name.clone()
        };
        profile_items.push(MenuItem::with_id(app, id, label, true, None::<&str>)?);
    }

    let refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = profile_items
        .iter()
        .map(|i| i as &dyn tauri::menu::IsMenuItem<tauri::Wry>)
        .collect();
    let profiles_sub = Submenu::with_items(app, "Accounts", true, &refs)?;

    let show_all = MenuItem::with_id(app, "show_all", "Show all", true, None::<&str>)?;
    let hide_all = MenuItem::with_id(app, "hide_all", "Hide all", true, None::<&str>)?;
    let new_account = MenuItem::with_id(app, "new_account", "New account", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Manage accounts…", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    Menu::with_items(
        app,
        &[
            &show_all,
            &hide_all,
            &sep,
            &profiles_sub,
            &new_account,
            &settings,
            &sep,
            &quit,
        ],
    )
}

fn show_all(app: &AppHandle) {
    let mut any = false;
    for (label, w) in app.webview_windows() {
        if label.starts_with("wa-") {
            let _ = w.show();
            let _ = w.unminimize();
            any = true;
        }
    }
    if !any {
        let _ = window::open_profile_window(app, profile::DEFAULT_PROFILE);
    } else if let Some(w) = app.get_webview_window(&window::window_label(profile::DEFAULT_PROFILE))
    {
        let _ = w.set_focus();
    }
}
