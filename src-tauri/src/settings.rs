//! Persisted app settings (app_config_dir/settings.json).

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct AppSettings {
    pub start_hidden: bool,
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("app_config_dir: {e}"))?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("settings.json"))
}

pub fn load(app: &AppHandle) -> AppSettings {
    settings_path(app)
        .ok()
        .and_then(|p| fs::read(p).ok())
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

pub fn save(app: &AppHandle, s: &AppSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    let json = serde_json::to_vec_pretty(s).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_app_settings(app: AppHandle) -> AppSettings {
    load(&app)
}

#[tauri::command]
pub fn set_app_settings(app: AppHandle, settings: AppSettings) -> Result<(), String> {
    save(&app, &settings)
}
