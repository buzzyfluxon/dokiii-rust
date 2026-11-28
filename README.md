<p align="center">
  <img src="assets/app-logo.png" alt="DOKIII Logo" width="96" />
</p>

<h1 align="center">DOKIII</h1>

<p align="center">
  A macOS-inspired dock, Dynamic Island style Halo, music player and desktop widgets for Windows.<br/>
  Rewritten on <b>Tauri 2 + Rust</b>: smaller, lighter and quicker than the Electron version.
</p>

<p align="center">
  <a href="https://github.com/buzzyfluxon/dokiii-rust/releases/latest">
    <img src="https://img.shields.io/github/v/release/buzzyfluxon/dokiii-rust?style=for-the-badge&color=b39ddb&label=version" alt="Latest Release" />
  </a>
  <a href="https://github.com/buzzyfluxon/dokiii-rust/releases">
    <img src="https://img.shields.io/github/downloads/buzzyfluxon/dokiii-rust/total?style=for-the-badge&color=b39ddb&label=downloads" alt="Downloads" />
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/license-MIT-b39ddb?style=for-the-badge" alt="License" />
  </a>
  <img src="https://img.shields.io/badge/platform-Windows%2010%20%7C%2011-b39ddb?style=for-the-badge" alt="Platform" />
</p>

---

## Download

<p align="center">
  <a href="https://github.com/buzzyfluxon/dokiii-rust/releases/latest/download/DOKIII-Installer.exe">
    <img src="https://img.shields.io/badge/Download-DOKIII--Installer.exe-b39ddb?style=for-the-badge&logo=windows&logoColor=white" alt="Download DOKIII Installer" />
  </a>
  <br/>
  <sub>Single installer &nbsp;•&nbsp; Windows 10/11 x64 &nbsp;•&nbsp; Choose your install folder</sub>
</p>

Download `DOKIII-Installer.exe` and run it. The setup window lets you pick the install folder and choose whether to add a desktop shortcut. `DOKIII-Setup.exe` is the plain installer the in-app updater uses. Windows SmartScreen may show a warning the first time because the app is not code-signed: select **More info**, then **Run anyway**.

Every version is on the [Releases page](https://github.com/buzzyfluxon/dokiii-rust/releases). The app checks this repository for updates.

---

## What's New in 2.3.1

- **Dock shows on the home screen again** with Replace Windows Taskbar on. DOKIII was mistaking its own full-screen window for a full-screen app and hiding the dock.
- **Clicks and hover.** Fixed clicks that did not reach the screen, jerky hover and magnification that stopped working with Replace Windows Taskbar on or after turning it off.
- **Halo, music ambient and desktop widgets stay on the desktop** while the taskbar is replaced, so they no longer appear over your windows or block clicks in them.
- **Lighter background work.** The taskbar check runs less often and no longer re-stacks the window when nothing changed.

## What's New in 2.3.0

- **Replace Windows Taskbar.** A new switch in the Dock tab hides the Windows taskbar and keeps the dock above your windows. The dock steps out of the way over full-screen apps and games and comes back when they close. Desktop widgets, Halo and music ambient stay on the desktop and do not float over your windows.
- **Cleaner first run.** A fresh install now starts with an empty desktop, four dock apps with Windows names (File Explorer, Browser, Terminal, Settings), and Halo and snow turned off. Add widgets from the Widget Library when you want them. Existing setups keep their layout.
- **Neutral colors.** The Quick Controls widget lost its purple glow. Its brightness slider, Wi-Fi and Bluetooth buttons now match the volume widget.
- **One set of window buttons.** The Control Center no longer shows macOS-style dots next to the Windows minimize, maximize and close buttons.
- **Dock and taskbar.** DOKIII now re-fits its window whenever the Windows work area changes, such as when the taskbar moves, resizes or auto-hides, so the dock no longer ends up under or over it.
- **Safe by design.** Quitting DOKIII, updating it, or a crash brings the taskbar back, and the next start repairs it if DOKIII was ever killed. The tray menu has a **Show Windows Taskbar** item that turns the switch off at any time.
- **Good to know.** Turn on Automatically Hide Dock with it, because maximized windows now use the whole screen and the dock sits on top of them. If the taskbar is ever stuck hidden after DOKIII is killed from Task Manager, start DOKIII once or restart Windows Explorer.

## What's New in 2.2.5

- **Dock on the right edge works.** With the dock set to Right it used to sit in the middle of the screen, and auto-hide could not slide it away from the right edge. It now sits flush against the right edge, just like the left.
- **Auto-hide moved to the Dock tab.** "Automatically Hide Dock" and "Show Dock on Hover" now live in an Auto Hide card on the Dock page, next to position and size, instead of on Home.

## What's New in 2.2.4

- **Widgets can be added again.** Adding a widget from the Control Center used the Control Center's own small window to find a free spot, so the desktop looked full. It now uses the real desktop size.
- **Quick Controls does nothing in the background.** Wi-Fi and Bluetooth are read once when the widget appears, and everything (brightness too) is read when your pointer reaches it. No timers, no repeated PowerShell. A display without brightness control is remembered, so it is not asked again.
- **Moon Phase is lighter and lets go of memory.** The moon texture is decoded once, shared, and released when the last Moon widget is removed. It checks every 5 minutes and repaints only when the phase has moved.
- **Memory trimming** hands unused memory back to Windows when you are idle and right after a widget is removed.
- **Music playback is much lighter.** The equalizer bars and the progress bars no longer repaint the blurred dock and Halo on every frame.
- **Halo checks Downloads and Screenshots less often.**
- **New glass icons** for Finder, Mail, Music and System Settings.

## What's New in 2.2.2

- **Moon Phase uses the real moon**: the phase, the lit side and the percentage lit are now calculated from the actual position of the Moon, so they match what you see in the sky (within a few hours). The widget also shows the moon's age in days.
- **Realistic moon surface**: the cartoon circles are gone. The Moon now shows its dark maria and craters in their real places, including Tycho with its bright rays, and the surface is lit by the Sun direction for the current phase, with craters casting shadows along the edge of the light and a faint earthshine on the dark side.
- **New Quick Controls widget**: a glass tile with a brightness slider and Wi-Fi and Bluetooth buttons. Brightness works on laptop screens, and a button greys out when the PC has no such radio.

---

## What It Does

- **macOS-Style Dock**: an animated bottom or side dock with fluid magnification, app badges and quick folder access, including Downloads and the Recycle Bin.
- **Halo and Floating Music**: shows the current song, artist and album art from Spotify and Windows media players, with playback controls.
- **Desktop Widgets**: world time, analog clock, calendar, system health (CPU, RAM, SSD), battery, volume, a glass Quick Controls tile (brightness, Wi-Fi, Bluetooth), a moon that follows the real phase, photo, music, movie and more.
- **Snowfall**: an optional Christmas effect that settles on the dock, widgets and windows.

---

## How to Use

### Dock
- **Launch apps**: click any icon.
- **Add apps**: click **+** to search installed programs or browse for an `.exe`.
- **Downloads**: click the downloads icon on the right of the dock.
- **Move the dock**: open Settings to place it on the bottom, left or right edge.

### Music and Halo
- Play music in Spotify or any Windows media app.
- Click the Halo or the floating player to expand it, then pause, play or skip. Click outside to collapse.

### Control Center
- Click the DOKIII logo or use the tray icon. It opens as its own window, so you can Alt+Tab to it.
- Turn desktop widgets on or off, pick widgets, change the dock, Halo and snow settings.

### Snowfall
Turn on Snowfall in Settings. Move your cursor through a pile to roll a snowball. Drag balls around and stack three to build a snowman, and double-click a snowman to knock it back into snowballs. Snow pauses while a maximized or full-screen app covers the desktop.

### Uninstalling
Open the Control Center and choose **Uninstall**, use **Uninstall DOKIII** in the tray menu, or remove it from Windows Settings under Installed apps.

---

## Building from Source

DOKIII is a Tauri 2 app: a Rust backend in `src-tauri` and a React interface in `src/renderer`.

### Requirements
- Windows 10 or 11
- Node.js 22 or newer
- Rust (stable, MSVC toolchain)
- Tauri CLI: `cargo install tauri-cli --version "^2" --locked`

### Run in development
```bash
git clone https://github.com/buzzyfluxon/dokiii-rust.git
cd dokiii-rust
npm install
npm run tauri:dev
```

### Build the installer
```bash
npm run tauri:build
```

The installer is written to `src-tauri/target/release/bundle/nsis`. Pushing a tag that starts with `v` (for example `v2.3.1`) builds it and publishes `DOKIII-Installer.exe` and `DOKIII-Setup.exe` through GitHub Actions. The setup window lives in `installer/`; the workflow copies `DOKIII-Setup.exe` into `installer/src-tauri/payload` and builds it with `cargo tauri build --no-bundle` from `installer/src-tauri`. See `docs/ARCHITECTURE.md` for how the pieces fit together.

---

## License

Released under the [MIT License](LICENSE).
