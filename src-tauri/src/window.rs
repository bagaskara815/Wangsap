use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent};

use crate::profile;

pub const WA_URL: &str = "https://web.whatsapp.com";

type DestroyCallback = Box<dyn FnOnce(&AppHandle) + Send>;

/// Callbacks to run once a window with the given label is actually destroyed.
static ON_DESTROY: LazyLock<Mutex<HashMap<String, Vec<DestroyCallback>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn window_label(profile: &str) -> String {
    format!("wa-{}", profile::sanitize_name(profile))
}

pub(crate) fn profile_from_label(label: &str) -> Option<String> {
    label
        .strip_prefix("wa-")
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

/// Register `f` to run when `label`'s window is destroyed, then request destroy.
/// `destroy()` is asynchronous and never emits `CloseRequested`, so the only
/// reliable completion signal is the `Destroyed` event handled in
/// `open_profile_window`. Runs `f` immediately if the window does not exist.
pub fn destroy_then(
    app: &AppHandle,
    label: &str,
    f: impl FnOnce(&AppHandle) + Send + 'static,
) -> Result<(), String> {
    let Some(w) = app.get_webview_window(label) else {
        f(app);
        return Ok(());
    };
    ON_DESTROY
        .lock()
        .map_err(|_| "destroy registry poisoned".to_string())?
        .entry(label.to_string())
        .or_default()
        .push(Box::new(f));
    if let Err(e) = w.destroy() {
        if let Ok(mut m) = ON_DESTROY.lock() {
            if let Some(v) = m.get_mut(label) {
                v.pop();
                if v.is_empty() {
                    m.remove(label);
                }
            }
        }
        return Err(format!("destroy window: {e}"));
    }
    Ok(())
}

/// Close → tray. Prefer minimize over hide so the WM keeps a live frame
/// (KDE Wayland often kills min/max/close hit-testing after hide()+show()).
pub fn hide_to_tray(w: &WebviewWindow) {
    let _ = w.set_skip_taskbar(true);
    // Minimize first (keeps XDG toplevel), then hide as soft fallback.
    let _ = w.minimize();
    let _ = w.hide();
}

/// Soft restore after tray (no window recreate).
pub fn restore_window(w: &WebviewWindow) {
    let _ = w.set_ignore_cursor_events(false);
    let _ = w.set_skip_taskbar(false);
    let _ = w.set_fullscreen(false);
    // Show before unminimize: unminimizing an unmapped toplevel is ignored
    // by most WMs, which would leave the window minimized.
    let _ = w.show();
    let _ = w.unminimize();
    // Focus nudge without touching decorations (toggling decorations breaks CSD on Wayland).
    let _ = w.set_focus();
    let _ = w.set_always_on_top(true);
    let _ = w.set_always_on_top(false);
    let _ = w.set_focus();
}

/// Primary tray/show path: reuse the live window (no reload), else open it.
pub fn restore_profile(app: &AppHandle, profile: &str) {
    let label = window_label(profile);
    if let Some(w) = app.get_webview_window(&label) {
        restore_window(&w);
    } else {
        let _ = open_profile_window(app, profile);
    }
}

/// Hard fallback: destroy the webview and reopen the same profile (the session
/// survives in the data dir). Costs a full WhatsApp Web reload, so it is only
/// used from the explicit "Reload windows" tray item when the soft restore
/// leaves broken window chrome (KDE Wayland).
pub fn restore_profile_hard(app: &AppHandle, profile: &str) {
    let clean = profile::sanitize_name(profile);
    let label = window_label(&clean);
    let reopen = {
        let p = clean.clone();
        move |app: &AppHandle| {
            let h = app.clone();
            // Destroyed fires mid-teardown; defer the reopen one event-loop tick.
            let _ = app.run_on_main_thread(move || {
                let _ = open_profile_window(&h, &p);
            });
        }
    };
    if destroy_then(app, &label, reopen).is_err() {
        // Destroy could not even be requested — the window is still alive.
        if let Some(w) = app.get_webview_window(&label) {
            restore_window(&w);
        }
    }
}

pub fn open_profile_window(app: &AppHandle, profile: &str) -> Result<WebviewWindow, String> {
    let clean = profile::sanitize_name(profile);
    if clean.is_empty() {
        return Err("profile name empty".into());
    }

    let label = window_label(&clean);

    if let Some(existing) = app.get_webview_window(&label) {
        restore_window(&existing);
        if let Some(icon) = app.default_window_icon() {
            let _ = existing.set_icon(icon.clone());
        }
        return Ok(existing);
    }

    let data_dir = profile::profile_dir(app, &clean)?;
    let title = if clean == profile::DEFAULT_PROFILE {
        "WhatsApp".to_string()
    } else {
        format!("WhatsApp — {clean}")
    };

    let inject = build_inject(&clean);

    let mut builder = WebviewWindowBuilder::new(
        app,
        &label,
        WebviewUrl::External(WA_URL.parse().map_err(|e| format!("url: {e}"))?),
    )
    .title(title)
    .inner_size(1280.0, 840.0)
    .min_inner_size(960.0, 640.0)
    .resizable(true)
    .maximizable(true)
    .minimizable(true)
    .closable(true)
    .decorations(true)
    .fullscreen(false)
    .disable_drag_drop_handler()
    .data_directory(data_dir)
    .initialization_script(&inject);

    if let Some(icon) = app.default_window_icon() {
        builder = builder
            .icon(icon.clone())
            .map_err(|e| format!("window icon: {e}"))?;
    }

    let window = builder.build().map_err(|e| format!("open window: {e}"))?;

    let handle = app.clone();
    let win_label = label.clone();
    window.on_window_event(move |event| match event {
        WindowEvent::CloseRequested { api, .. } => {
            // destroy() never emits CloseRequested, so anything arriving here
            // is a user close → keep the window alive in the tray.
            api.prevent_close();
            if let Some(w) = handle.get_webview_window(&win_label) {
                hide_to_tray(&w);
            }
        }
        WindowEvent::Destroyed => {
            let callbacks = ON_DESTROY
                .lock()
                .ok()
                .and_then(|mut m| m.remove(&win_label))
                .unwrap_or_default();
            for f in callbacks {
                f(&handle);
            }
        }
        _ => {}
    });

    Ok(window)
}

pub fn open_settings_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    const LABEL: &str = "settings";

    if let Some(existing) = app.get_webview_window(LABEL) {
        restore_window(&existing);
        if let Some(icon) = app.default_window_icon() {
            let _ = existing.set_icon(icon.clone());
        }
        return Ok(existing);
    }

    let mut builder = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("/settings".into()))
        .title("Wangsap — Accounts")
        .inner_size(520.0, 560.0)
        .min_inner_size(420.0, 400.0)
        .resizable(true)
        .maximizable(true)
        .minimizable(true)
        .closable(true)
        .decorations(true)
        .center();

    if let Some(icon) = app.default_window_icon() {
        builder = builder
            .icon(icon.clone())
            .map_err(|e| format!("window icon: {e}"))?;
    }

    let window = builder.build().map_err(|e| format!("open settings: {e}"))?;
    Ok(window)
}

#[tauri::command]
pub fn open_profile(app: AppHandle, name: String) -> Result<(), String> {
    open_profile_window(&app, &name)?;
    Ok(())
}

#[tauri::command]
pub fn open_settings(app: AppHandle) -> Result<(), String> {
    open_settings_window(&app)?;
    Ok(())
}

#[tauri::command]
pub fn show_profile(app: AppHandle, name: String) -> Result<(), String> {
    restore_profile(&app, &name);
    Ok(())
}

#[tauri::command]
pub fn hide_all_profiles(app: AppHandle) -> Result<(), String> {
    for (label, w) in app.webview_windows() {
        if label.starts_with("wa-") {
            hide_to_tray(&w);
        }
    }
    Ok(())
}

/// Open an http(s) URL in the system browser. Called by the injected script
/// for links that would otherwise navigate the WhatsApp webview away.
#[tauri::command]
pub fn open_external(url: String) -> Result<(), String> {
    let parsed = tauri::Url::parse(&url).map_err(|e| format!("url: {e}"))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err("unsupported scheme".into());
    }
    tauri_plugin_opener::open_url(parsed.as_str(), None::<&str>).map_err(|e| e.to_string())
}

const INJECT_TEMPLATE: &str = include_str!("../injected/whatsapp.js");

fn build_inject(profile: &str) -> String {
    // Profile names are sanitized to [A-Za-z0-9_-] upstream; escape defensively
    // anyway since this lands inside a single-quoted JS string.
    let safe = profile.replace('\\', "\\\\").replace('\'', "\\'");
    INJECT_TEMPLATE.replace("__WA_PROFILE_PLACEHOLDER__", &safe)
}
