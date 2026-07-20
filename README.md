# Wangsap

**Unofficial WhatsApp Web client for Linux**

Wangsap wraps [WhatsApp Web](https://web.whatsapp.com) in a native desktop shell so you get a proper window, system tray, multi-account sessions, and desktop notifications — without running a full browser window for chat.

| | |
|---|---|
| **App ID** | `com.wangsap` |
| **Binary** | `wangsap` |
| **License** | MIT |
| **Status** | Personal / experimental |

> **Not affiliated with Meta or WhatsApp.**  
> This is a third-party WebView wrapper. Use at your own risk. Features that depend on WhatsApp Web (especially calls) inherit WebKitGTK limitations on Linux.

---

## What it is (and isn’t)

**Is**
- A lightweight desktop shell around the official WhatsApp Web UI
- Multi-profile (isolated sessions / QR logins)
- System tray + close-to-tray
- FreeDesktop icons / `.desktop` entries for X11 and Wayland (tested on KDE Plasma)

**Isn’t**
- An official Meta product
- A reverse-engineered protocol client (no Baileys / whatsmeow)
- A guarantee of call quality, ToS safety, or long-term API stability

If WhatsApp changes Web, Wangsap can break until updated. That’s the trade-off of the wrapper approach.

---

## Stack

| Layer | Tech |
|-------|------|
| Shell | [Tauri 2](https://tauri.app/) |
| Backend | Rust |
| Frontend (settings UI) | Svelte 5 + SvelteKit (static) |
| Content | WhatsApp Web via OS WebView (**WebKitGTK** on Linux) |
| Packaging | Arch `PKGBUILD` → `.pkg.tar.zst` (also Tauri deb/rpm/AppImage targets) |

---

## Features

- **Single window WhatsApp Web** with persistent session storage per profile  
- **System tray** — left-click show; menu for accounts / hide / quit  
- **Close to tray** (session stays alive)  
- **Multi-account** — tray → New account, or **Manage accounts…** (rename / delete / open)  
- **Unread badge** on tray icon (best-effort from page title `(N) …`)  
- **Notification bridge** (best-effort native desktop notifications)  
- **Wayland taskbar icons** via installed hicolor icons + desktop files (`wangsap` / `com.wangsap`)

### Known limitations

- Audio/video **calls** depend on WebKitGTK WebRTC — often weaker than Chromium  
- Heavy DOM injection is avoided on purpose; WhatsApp Web is brittle  
- NVIDIA + WebKitGTK may need `WEBKIT_DISABLE_DMABUF_RENDERER=1`  
- Unofficial clients can always be restricted by Meta policy changes  

---

## Install (Arch Linux)

Prebuilt package from this tree:

```bash
cd packaging/arch
./build.sh          # or: makepkg -f
sudo pacman -U wangsap-*.pkg.tar.zst
```

Then launch **Wangsap** from the app menu, or:

```bash
wangsap
```

### Runtime dependencies

`webkit2gtk-4.1`, `gtk3`, `libsoup3`, `libappindicator`, `openssl`, `hicolor-icon-theme`, and related GTK stack packages (pulled in by pacman).

### What the package installs

| Path | Purpose |
|------|---------|
| `/usr/bin/wangsap` | Main binary |
| `/usr/share/applications/com.wangsap.desktop` | Desktop entry |
| `/usr/share/applications/wangsap.desktop` | Alias (matches WM class) |
| `/usr/share/icons/hicolor/*/apps/{com.wangsap,wangsap}.*` | App icons |

### Other distros / CI packages

| Format | How |
|--------|-----|
| `.deb` | CircleCI job `build-deb-rpm`, or `npm run tauri build -- --bundles deb` |
| `.rpm` | Same job / `--bundles rpm` |
| Arch `.pkg.tar.zst` | CircleCI job `build-arch`, or `packaging/arch/build.sh` |

**Continuous release:** every push to `main` updates  
https://github.com/bagaskara815/Wangsap/releases/tag/continuous  
(changelog + latest deb/rpm/Arch). Setup: [packaging/CIRCLECI.md](./packaging/CIRCLECI.md).

---

## Development

### Prerequisites

- Rust (rustup recommended) + Cargo  
- Node.js + npm  
- Linux WebKitGTK 4.1 development stack (`webkit2gtk-4.1`, etc.)

```bash
export PATH="$HOME/.cargo/bin:$PATH"
cd /path/to/wa
npm install
npm run tauri:dev
```

Release binary (no installer bundles):

```bash
npm run tauri build -- --no-bundle
# → src-tauri/target/release/wangsap
```

### NVIDIA / blank WebView

```bash
WEBKIT_DISABLE_DMABUF_RENDERER=1 wangsap
# or for dev:
WEBKIT_DISABLE_DMABUF_RENDERER=1 npm run tauri:dev
```

---

## Data layout

```
~/.local/share/com.wangsap/profiles/<name>/
```

Each profile is an isolated WebView data directory (cookies / service worker / QR session).

On first run after the id rename, profiles from `~/.local/share/com.bagas.wa/profiles` are copied once if the new directory is empty.

---

## Project layout

```
wa/
├── src/                    # Svelte settings / shell UI
├── src-tauri/
│   ├── src/                # Rust: window, tray, profiles, badge, desktop install
│   ├── icons/              # App icon set
│   └── tauri.conf.json     # identifier: com.wangsap
├── packaging/arch/         # PKGBUILD + build.sh
├── LICENSE
└── README.md
```

---

## Attribution

Wangsap was designed and implemented in a human–AI pairing session:

- **Human:** product direction, Linux/Arch integration choices, testing on KDE Wayland  
- **AI:** implementation assistance by **Grok** (xAI), driven through the Hermes agent environment  

The codebase is ordinary open-source software under MIT. Using an AI assistant does not change the license, the unofficial status, or your responsibility when running it against WhatsApp’s services.

Icon artwork is an **unofficial** WhatsApp-style mark for desktop recognition — not Meta brand assets.

---

## Disclaimer

```
THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND.
WhatsApp is a trademark of Meta Platforms, Inc.
This project is not endorsed by Meta.
Automated or unofficial access may violate WhatsApp Terms of Service.
```

See [LICENSE](./LICENSE) for the full MIT text.

---

## Changelog (short)

| Version | Notes |
|---------|--------|
| 0.1.0 | Initial Linux client: tray, multi-account, unread badge, Arch package, `com.wangsap` id |
