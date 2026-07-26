use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Manager};

use crate::unread::UnreadState;
use crate::window;

pub const DEFAULT_PROFILE: &str = "default";

/// Resolve `app_data_dir/profiles/<name>` and ensure it exists.
pub fn profile_dir(app: &AppHandle, name: &str) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app_data_dir: {e}"))?;
    let dir = base.join("profiles").join(sanitize_name(name));
    fs::create_dir_all(&dir).map_err(|e| format!("create profile dir: {e}"))?;
    Ok(dir)
}

pub fn profiles_root(app: &AppHandle) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app_data_dir: {e}"))?
        .join("profiles");
    fs::create_dir_all(&base).map_err(|e| e.to_string())?;
    Ok(base)
}

pub fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .chars()
        .take(32)
        .collect()
}

pub fn list_profile_names(app: &AppHandle) -> Result<Vec<String>, String> {
    let base = profiles_root(app)?;

    let mut names = Vec::new();
    for entry in fs::read_dir(&base).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            if let Some(name) = entry.file_name().to_str() {
                names.push(name.to_string());
            }
        }
    }

    if names.is_empty() {
        let _ = profile_dir(app, DEFAULT_PROFILE);
        names.push(DEFAULT_PROFILE.to_string());
    }
    names.sort();
    Ok(names)
}

/// Next free name: account-2, account-3, ...
pub fn next_account_name(app: &AppHandle) -> Result<String, String> {
    next_free_account_name(&list_profile_names(app)?)
}

fn next_free_account_name(existing: &[String]) -> Result<String, String> {
    let mut n = 2u32;
    loop {
        let candidate = format!("account-{n}");
        if !existing.iter().any(|e| e == &candidate) {
            return Ok(candidate);
        }
        n += 1;
        if n > 100 {
            return Err("too many accounts".into());
        }
    }
}

/// Destroy the profile's window and block until `Destroyed` has fired, so the
/// webview releases its locks on the data dir before we touch it. Must only be
/// called from async commands (blocking the main thread here would deadlock:
/// the destroy is processed on the main event loop).
fn destroy_window_and_wait(app: &AppHandle, name: &str, timeout: Duration) -> Result<(), String> {
    let label = window::window_label(name);
    if app.get_webview_window(&label).is_none() {
        return Ok(());
    }
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    window::destroy_then(app, &label, move |_| {
        let _ = tx.send(());
    })?;
    rx.recv_timeout(timeout)
        .map_err(|_| "window did not close in time".to_string())
}

/// WebKit's helper processes can hold files for a beat after `Destroyed`.
fn retry_fs<T>(mut op: impl FnMut() -> std::io::Result<T>) -> Result<T, String> {
    const ATTEMPTS: u32 = 5;
    let mut attempt = 1;
    loop {
        match op() {
            Ok(v) => return Ok(v),
            Err(e) if attempt >= ATTEMPTS => return Err(e.to_string()),
            Err(_) => {
                attempt += 1;
                std::thread::sleep(Duration::from_millis(200));
            }
        }
    }
}

#[tauri::command]
pub fn list_profiles(app: AppHandle) -> Result<Vec<String>, String> {
    list_profile_names(&app)
}

#[tauri::command]
pub fn create_profile(app: AppHandle, name: String) -> Result<String, String> {
    let clean = sanitize_name(&name);
    if clean.is_empty() {
        return Err("profile name empty".into());
    }
    let existing = list_profile_names(&app)?;
    if existing.iter().any(|e| e == &clean) {
        return Err("profile already exists".into());
    }
    let dir = profile_dir(&app, &clean)?;
    let _ = crate::tray::rebuild_tray_menu(&app);
    Ok(dir.display().to_string())
}

#[tauri::command]
pub fn create_next_profile(app: AppHandle) -> Result<String, String> {
    let name = next_account_name(&app)?;
    profile_dir(&app, &name)?;
    let _ = crate::tray::rebuild_tray_menu(&app);
    Ok(name)
}

#[tauri::command]
pub async fn rename_profile(app: AppHandle, from: String, to: String) -> Result<String, String> {
    let from = sanitize_name(&from);
    let to = sanitize_name(&to);
    if from.is_empty() || to.is_empty() {
        return Err("invalid name".into());
    }
    if from == to {
        return Ok(to);
    }

    let names = list_profile_names(&app)?;
    if !names.iter().any(|n| n == &from) {
        return Err("source profile not found".into());
    }
    if names.iter().any(|n| n == &to) {
        return Err("target name already exists".into());
    }

    destroy_window_and_wait(&app, &from, Duration::from_secs(5))?;

    let root = profiles_root(&app)?;
    let src = root.join(&from);
    let dst = root.join(&to);
    retry_fs(|| fs::rename(&src, &dst)).map_err(|e| format!("rename failed: {e}"))?;

    let unread = app.state::<UnreadState>();
    unread.rename_key(&from, &to);
    let _ = crate::tray::rebuild_tray_menu(&app);
    crate::tray::refresh_unread(&app, unread.total());

    Ok(to)
}

#[tauri::command]
pub async fn delete_profile(app: AppHandle, name: String) -> Result<(), String> {
    let name = sanitize_name(&name);
    if name.is_empty() {
        return Err("invalid name".into());
    }

    let names = list_profile_names(&app)?;
    if !names.iter().any(|n| n == &name) {
        return Err("profile not found".into());
    }
    if names.len() <= 1 {
        return Err("cannot delete the last account".into());
    }

    destroy_window_and_wait(&app, &name, Duration::from_secs(5))?;

    let dir = profiles_root(&app)?.join(&name);
    retry_fs(|| fs::remove_dir_all(&dir)).map_err(|e| format!("delete failed: {e}"))?;

    let unread = app.state::<UnreadState>();
    unread.remove_key(&name);
    let _ = crate::tray::rebuild_tray_menu(&app);
    crate::tray::refresh_unread(&app, unread.total());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_keeps_allowed_chars() {
        assert_eq!(sanitize_name("work-2_ok"), "work-2_ok");
    }

    #[test]
    fn sanitize_replaces_and_trims() {
        assert_eq!(sanitize_name("héllo world!"), "h_llo_world");
        assert_eq!(sanitize_name("___x___"), "x");
        assert_eq!(sanitize_name("../../etc"), "etc");
    }

    #[test]
    fn sanitize_caps_length_and_handles_empty() {
        assert_eq!(sanitize_name(&"a".repeat(64)).len(), 32);
        assert_eq!(sanitize_name(""), "");
        assert_eq!(sanitize_name("!!!"), "");
    }

    #[test]
    fn next_name_skips_taken_slots() {
        let existing = vec!["default".to_string(), "account-2".to_string()];
        assert_eq!(next_free_account_name(&existing).unwrap(), "account-3");
        assert_eq!(next_free_account_name(&[]).unwrap(), "account-2");
    }

    #[test]
    fn next_name_errors_when_full() {
        let existing: Vec<String> = (2..=100).map(|n| format!("account-{n}")).collect();
        assert!(next_free_account_name(&existing).is_err());
    }
}
