// Injected into web.whatsapp.com before page scripts run (see build_inject in
// window.rs, which substitutes __WA_PROFILE_PLACEHOLDER__).
(function () {
  window.__WA_PROFILE__ = '__WA_PROFILE_PLACEHOLDER__';
  try {
    document.documentElement.setAttribute('data-wa-linux', '1');
    document.documentElement.setAttribute('data-wa-profile', window.__WA_PROFILE__);
  } catch (e) {}

  function invoke(cmd, args) {
    try {
      var t = window.__TAURI__;
      if (t && t.core && t.core.invoke) {
        return t.core.invoke(cmd, args).catch(function () {});
      }
    } catch (e) {}
    return null;
  }

  // --- Desktop notifications --------------------------------------------------
  try {
    var OriginalNotification = window.Notification;
    function WaNotification(title, options) {
      options = options || {};
      var sent = false;
      try {
        var t = window.__TAURI__;
        if (t && t.core && t.core.invoke) {
          var payload = {
            title: String(title || 'WhatsApp'),
            body: String(options.body || '')
          };
          if (options.tag) payload.tag = String(options.tag);
          t.core.invoke('plugin:notification|notify', { options: payload }).catch(function () {});
          sent = true;
        }
      } catch (err) {}
      if (!sent && OriginalNotification) {
        try { return new OriginalNotification(title, options); } catch (err2) {}
      }
      var noop = function () {};
      return { close: noop, onclick: null, onshow: null, onerror: null, onclose: null };
    }
    WaNotification.permission = 'granted';
    WaNotification.requestPermission = function () { return Promise.resolve('granted'); };
    if (OriginalNotification) {
      WaNotification.maxActions = OriginalNotification.maxActions;
    }
    window.Notification = WaNotification;
  } catch (e) {}

  // --- Unread badge from the page title ---------------------------------------
  try {
    var lastCount = -1;
    function parseUnread() {
      var n = 0;
      try {
        // Past 99 the title reads "(99+) WhatsApp" — count that as 99.
        var m = document.title && document.title.match(/^\((\d+)\+?\)/);
        if (m) n = parseInt(m[1], 10) || 0;
      } catch (e1) {}
      return n;
    }
    function report(n) {
      if (n === lastCount) return;
      lastCount = n;
      invoke('set_unread_count', { profile: window.__WA_PROFILE__ || 'default', count: n });
    }
    function tick() {
      try { report(parseUnread()); } catch (e) {}
    }
    var obs = new MutationObserver(tick);
    function observeTitle() {
      // Observing only <title>/<head> instead of the whole document keeps the
      // observer quiet while WhatsApp's hot DOM churns.
      var target = document.querySelector('title') || document.head;
      if (!target) return;
      try {
        obs.observe(target, { subtree: true, childList: true, characterData: true });
      } catch (e) {}
    }
    var start = function () {
      observeTitle();
      tick();
      // Safety net for a late or replaced <title> element.
      setInterval(tick, 4000);
    };
    if (document.readyState === 'loading') {
      document.addEventListener('DOMContentLoaded', start);
    } else {
      start();
    }
  } catch (e) {}

  // --- External links open in the system browser ------------------------------
  // WebKitGTK has no new-window handler here, so window.open would otherwise
  // be silently dropped.
  try {
    window.open = function (url) {
      if (url) invoke('open_external', { url: String(url) });
      return null;
    };
    document.addEventListener('click', function (ev) {
      var el = ev.target && ev.target.closest ? ev.target.closest('a[target="_blank"]') : null;
      if (el && el.href) {
        ev.preventDefault();
        ev.stopPropagation();
        invoke('open_external', { url: el.href });
      }
    }, true);
  } catch (e) {}
})();
