use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent};

use crate::profile;

pub const WA_URL: &str = "https://web.whatsapp.com";

pub fn window_label(profile: &str) -> String {
    format!("wa-{}", profile::sanitize_name(profile))
}

pub fn open_profile_window(app: &AppHandle, profile: &str) -> Result<WebviewWindow, String> {
    let clean = profile::sanitize_name(profile);
    if clean.is_empty() {
        return Err("profile name empty".into());
    }

    let label = window_label(&clean);

    if let Some(existing) = app.get_webview_window(&label) {
        let _ = existing.show();
        let _ = existing.unminimize();
        let _ = existing.set_focus();
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
    window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            if let Some(w) = handle.get_webview_window(&win_label) {
                let _ = w.hide();
            }
        }
    });

    Ok(window)
}

pub fn open_settings_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    const LABEL: &str = "settings";

    if let Some(existing) = app.get_webview_window(LABEL) {
        let _ = existing.show();
        let _ = existing.unminimize();
        let _ = existing.set_focus();
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
    let label = window_label(&name);
    if let Some(w) = app.get_webview_window(&label) {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
        Ok(())
    } else {
        open_profile_window(&app, &name)?;
        Ok(())
    }
}

#[tauri::command]
pub fn hide_all_profiles(app: AppHandle) -> Result<(), String> {
    for (label, w) in app.webview_windows() {
        if label.starts_with("wa-") {
            let _ = w.hide();
        }
    }
    Ok(())
}

fn build_inject(profile: &str) -> String {
    let safe = profile.replace('\\', "\\\\").replace('\'', "\\'");
    format!(
        r#"(function () {{
  window.__WA_PROFILE__ = '{safe}';
  try {{
    document.documentElement.setAttribute('data-wa-linux', '1');
    document.documentElement.setAttribute('data-wa-profile', '{safe}');
  }} catch (e) {{}}

  try {{
    var OriginalNotification = window.Notification;
    function WaNotification(title, options) {{
      options = options || {{}};
      var body = options.body || '';
      var sent = false;
      try {{
        var t = window.__TAURI__;
        if (t && t.core && t.core.invoke) {{
          t.core.invoke('plugin:notification|notify', {{
            options: {{ title: String(title || 'WhatsApp'), body: String(body) }}
          }}).catch(function () {{}});
          sent = true;
        }} else if (t && t.notification && t.notification.sendNotification) {{
          t.notification.sendNotification({{
            title: String(title || 'WhatsApp'),
            body: String(body)
          }});
          sent = true;
        }}
      }} catch (err) {{}}
      if (!sent && OriginalNotification) {{
        try {{ return new OriginalNotification(title, options); }} catch (err2) {{}}
      }}
      var noop = function () {{}};
      return {{ close: noop, onclick: null, onshow: null, onerror: null, onclose: null }};
    }}
    WaNotification.permission = 'granted';
    WaNotification.requestPermission = function () {{ return Promise.resolve('granted'); }};
    if (OriginalNotification) {{
      WaNotification.maxActions = OriginalNotification.maxActions;
    }}
    window.Notification = WaNotification;
  }} catch (e) {{}}

  try {{
    var lastCount = -1;
    function parseUnread() {{
      var n = 0;
      try {{
        var m = document.title && document.title.match(/^\((\d+)\)/);
        if (m) n = parseInt(m[1], 10) || 0;
      }} catch (e1) {{}}
      return n;
    }}
    function report(n) {{
      if (n === lastCount) return;
      lastCount = n;
      try {{
        var t = window.__TAURI__;
        if (t && t.core && t.core.invoke) {{
          t.core.invoke('set_unread_count', {{
            profile: window.__WA_PROFILE__ || 'default',
            count: n
          }}).catch(function () {{}});
        }}
      }} catch (e) {{}}
    }}
    function tick() {{
      try {{ report(parseUnread()); }} catch (e) {{}}
    }}
    var obs = new MutationObserver(tick);
    var start = function () {{
      try {{
        obs.observe(document.documentElement, {{
          subtree: true,
          childList: true,
          characterData: true,
          attributes: true,
          attributeFilter: ['title', 'aria-label']
        }});
      }} catch (e) {{}}
      tick();
      setInterval(tick, 4000);
    }};
    if (document.readyState === 'loading') {{
      document.addEventListener('DOMContentLoaded', start);
    }} else {{
      start();
    }}
  }} catch (e) {{}}
}})();"#
    )
}
