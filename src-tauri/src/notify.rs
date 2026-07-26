//! Desktop notifications with click-to-focus.
//!
//! tauri-plugin-notification has no desktop click events (mobile only), so
//! this goes through notify-rust directly with a "default" action: clicking
//! the notification focuses the profile's window.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use tauri::AppHandle;

/// tag → last notification id, so repeated messages replace instead of stack.
static TAG_IDS: LazyLock<Mutex<HashMap<String, u32>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[tauri::command]
pub fn notify_profile(
    app: AppHandle,
    profile: String,
    title: String,
    body: String,
    tag: Option<String>,
) -> Result<(), String> {
    let mut n = notify_rust::Notification::new();
    n.summary(&title)
        .body(&body)
        .appname("Wangsap")
        .icon("com.wangsap")
        .action("default", "Open");

    if let Some(tag) = &tag {
        if let Some(id) = TAG_IDS.lock().ok().and_then(|m| m.get(tag).copied()) {
            n.id(id);
        }
    }

    let handle = n.show().map_err(|e| e.to_string())?;

    if let Some(tag) = tag {
        if let Ok(mut m) = TAG_IDS.lock() {
            m.insert(tag, handle.id());
        }
    }

    // wait_for_action parks until the notification is actioned/closed; some
    // daemons ignore actions entirely, then this just degrades to no-op.
    std::thread::spawn(move || {
        handle.wait_for_action(move |action| {
            if action == "default" {
                let app2 = app.clone();
                let _ = app.run_on_main_thread(move || {
                    crate::window::restore_profile(&app2, &profile);
                });
            }
        });
    });

    Ok(())
}
