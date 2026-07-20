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

fn install_inner() -> Result<(), String> {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or("HOME not set")?;
    let hicolor = home.join(".local/share/icons/hicolor");
    let apps = home.join(".local/share/applications");

    for name in [APP_ID, BIN_ID] {
        write_icon(&hicolor, "32x32", "png", name, ICON_32)?;
        write_icon(&hicolor, "64x64", "png", name, ICON_64)?;
        write_icon(&hicolor, "128x128", "png", name, ICON_128)?;
        write_icon(&hicolor, "256x256", "png", name, ICON_256)?;
        write_icon(&hicolor, "512x512", "png", name, ICON_512)?;
        write_icon(&hicolor, "scalable", "svg", name, ICON_SVG)?;
    }

    fs::create_dir_all(&apps).map_err(|e| e.to_string())?;

    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_s = exe.to_string_lossy().replace('"', "\\\"");

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

    fs::write(apps.join(format!("{APP_ID}.desktop")), &desktop).map_err(|e| e.to_string())?;

    // Binary-name desktop (KWin often uses resourceClass = binary name)
    let desktop_bin = desktop
        .replace(&format!("Icon={APP_ID}"), &format!("Icon={BIN_ID}"))
        .replace(
            &format!("StartupWMClass={APP_ID}"),
            &format!("StartupWMClass={BIN_ID}"),
        );
    fs::write(apps.join(format!("{BIN_ID}.desktop")), desktop_bin).map_err(|e| e.to_string())?;

    // Drop stale ids from earlier iterations
    for stale in ["wa-linux.desktop", "com.bagas.wa.desktop"] {
        let p = apps.join(stale);
        let _ = fs::remove_file(p);
    }

    migrate_profiles_if_needed(&home);

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

    Ok(())
}

/// One-shot: copy old com.bagas.wa profiles → com.wangsap if target empty.
fn migrate_profiles_if_needed(home: &Path) {
    let old = home.join(".local/share/com.bagas.wa/profiles");
    let new = home.join(".local/share/com.wangsap/profiles");
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

fn write_icon(
    hicolor: &Path,
    size_dir: &str,
    ext: &str,
    name: &str,
    bytes: &[u8],
) -> Result<(), String> {
    let dir = hicolor.join(size_dir).join("apps");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    fs::write(dir.join(format!("{name}.{ext}")), bytes).map_err(|e| e.to_string())?;
    Ok(())
}
