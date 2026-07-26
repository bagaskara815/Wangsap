//! Install FreeDesktop icon + .desktop for Wayland/KDE taskbar.
//!
//! Identifier / app id: com.wangsap
//! Binary / WM class fallback: wangsap

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const APP_ID: &str = "com.wangsap";
const BIN_ID: &str = "wangsap";

const ICON_32: &[u8] = include_bytes!("../icons/32x32.png");
const ICON_64: &[u8] = include_bytes!("../icons/64x64.png");
const ICON_128: &[u8] = include_bytes!("../icons/128x128.png");
const ICON_256: &[u8] = include_bytes!("../icons/128x128@2x.png");
const ICON_512: &[u8] = include_bytes!("../icons/icon.png");
const ICON_SVG: &[u8] = include_bytes!("../icons/whatsapp.svg");

pub fn install() {
    if let Err(e) = install_inner() {
        eprintln!("[wangsap] desktop icon install: {e}");
    }
}

/// `$XDG_DATA_HOME` if set to an absolute path, else `~/.local/share`.
fn data_home(home: &Path) -> PathBuf {
    match std::env::var_os("XDG_DATA_HOME") {
        Some(v) if Path::new(&v).is_absolute() => PathBuf::from(v),
        _ => home.join(".local/share"),
    }
}

fn install_inner() -> Result<(), String> {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or("HOME not set")?;
    let share = data_home(&home);
    let hicolor = share.join("icons/hicolor");
    let apps = share.join("applications");

    let mut changed = false;
    for name in [APP_ID, BIN_ID] {
        changed |= write_icon(&hicolor, "32x32", "png", name, ICON_32)?;
        changed |= write_icon(&hicolor, "64x64", "png", name, ICON_64)?;
        changed |= write_icon(&hicolor, "128x128", "png", name, ICON_128)?;
        changed |= write_icon(&hicolor, "256x256", "png", name, ICON_256)?;
        changed |= write_icon(&hicolor, "512x512", "png", name, ICON_512)?;
        changed |= write_icon(&hicolor, "scalable", "svg", name, ICON_SVG)?;
    }

    fs::create_dir_all(&apps).map_err(|e| e.to_string())?;

    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_s = exe
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");

    // Prefer reverse-DNS icon name for GTK app id; StartupWMClass covers both.
    let desktop = format!(
        r#"[Desktop Entry]
Type=Application
Version=1.0
Name=Wangsap
GenericName=WhatsApp
Comment=WhatsApp Web for Linux
Exec="{exe_s}"
Icon={APP_ID}
Terminal=false
Categories=Network;InstantMessaging;Chat;
StartupNotify=true
StartupWMClass={APP_ID}
X-GNOME-UsesNotifications=true
Keywords=whatsapp;chat;wa;wangsap;
"#
    );

    changed |= write_if_changed(&apps.join(format!("{APP_ID}.desktop")), desktop.as_bytes())?;

    // Binary-name desktop (KWin often uses resourceClass = binary name)
    let desktop_bin = desktop
        .replace(&format!("Icon={APP_ID}"), &format!("Icon={BIN_ID}"))
        .replace(
            &format!("StartupWMClass={APP_ID}"),
            &format!("StartupWMClass={BIN_ID}"),
        );
    changed |= write_if_changed(&apps.join(format!("{BIN_ID}.desktop")), desktop_bin.as_bytes())?;

    // Drop stale ids from earlier iterations
    for stale in ["wa-linux.desktop", "com.bagas.wa.desktop"] {
        if fs::remove_file(apps.join(stale)).is_ok() {
            changed = true;
        }
    }

    migrate_profiles_if_needed(&share);

    // The cache refreshers (kbuildsycoca especially) can take seconds — keep
    // them off the startup path, and skip them entirely when nothing changed.
    if changed {
        std::thread::spawn(move || {
            let _ = Command::new("update-desktop-database").arg(&apps).status();
            let _ = Command::new("gtk-update-icon-cache")
                .args(["-f", "-t"])
                .arg(&hicolor)
                .status();
            let _ = Command::new("kbuildsycoca6")
                .arg("--noincremental")
                .status();
            let _ = Command::new("kbuildsycoca5")
                .arg("--noincremental")
                .status();
        });
    }

    Ok(())
}

/// One-shot: copy old com.bagas.wa profiles → com.wangsap if target empty.
fn migrate_profiles_if_needed(share: &Path) {
    let old = share.join("com.bagas.wa/profiles");
    let new = share.join("com.wangsap/profiles");
    if !old.is_dir() {
        return;
    }
    if new.is_dir() {
        if let Ok(rd) = fs::read_dir(&new) {
            if rd.filter_map(|e| e.ok()).next().is_some() {
                return; // already has data
            }
        }
    }
    let _ = fs::create_dir_all(new.parent().unwrap_or(Path::new(".")));
    // best-effort recursive copy
    let _ = copy_dir_all(&old, &new);
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &to)?;
        } else {
            fs::copy(entry.path(), to)?;
        }
    }
    Ok(())
}

/// Write only when content differs. Returns whether the file was (re)written.
fn write_if_changed(path: &Path, bytes: &[u8]) -> Result<bool, String> {
    if fs::read(path).map(|cur| cur == bytes).unwrap_or(false) {
        return Ok(false);
    }
    fs::write(path, bytes).map_err(|e| e.to_string())?;
    Ok(true)
}

fn write_icon(
    hicolor: &Path,
    size_dir: &str,
    ext: &str,
    name: &str,
    bytes: &[u8],
) -> Result<bool, String> {
    let dir = hicolor.join(size_dir).join("apps");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    write_if_changed(&dir.join(format!("{name}.{ext}")), bytes)
}
