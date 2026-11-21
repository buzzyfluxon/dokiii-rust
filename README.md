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
  <a href="https://github.com/buzzyfluxon/dokiii-rust/releases/latest/download/DOKIII-Setup.exe">
    <img src="https://img.shields.io/badge/Download-DOKIII--Setup.exe-b39ddb?style=for-the-badge&logo=windows&logoColor=white" alt="Download DOKIII Installer" />
  </a>
  <br/>
  <sub>Single installer &nbsp;•&nbsp; Windows 10/11 x64 &nbsp;•&nbsp; Choose your install folder</sub>
</p>

Download `DOKIII-Setup.exe` and run it. The installer lets you pick the install folder and sets up the shortcuts you choose. Windows SmartScreen may show a warning the first time because the app is not code-signed: select **More info**, then **Run anyway**.

Every version is on the [Releases page](https://github.com/buzzyfluxon/dokiii-rust/releases). The app checks this repository for updates.

---

## What's New in 2.1.3

- **Control Center opens instantly**: it prepares itself when you hover the dock logo and stays ready for a few minutes after you close it. The open and close animations now use only GPU-friendly motion, so they stay smooth.

## What's New in 2.1.2

- **Halo is instant**: volume and mute changes show in the Halo right away, with no delay. Battery changes are picked up within about a second and a half.

## What's New in 2.1.1

- **Fix**: the Control Center now opens reliably from the dock logo and the tray.
- **Frosted glass**: the Control Center window now blurs what's behind it, like macOS.
- **Photo widget**: Choose Photo works again, and you can drag and drop an image onto the widget.

## What's New in 2.1.0

- **Snowfall rebuilt**: snow now builds up slowly into soft, rounded piles on the dock, widgets and window tops. Snowballs only appear when you move your cursor through a pile, and they grow as you roll them. Move a window or widget quickly and the snow on it slides and falls off. New option: **Snow in Front of Windows**.
- **Apple style animations**: the Halo expands and collapses like the Dynamic Island, and the Control Center opens and closes with a spring animation.
- **Control Center is its own window**: it shows in the taskbar and in Alt+Tab. Changes you make there apply to the dock right away.
- **Music shows up on its own**: the Halo and dock player now pick up music as soon as it starts, with no need to press play inside DOKIII first.
- **Cleaner widget library**: Car Card is now **Photo**. Polaroid, Gallery and Mood Board were removed because they duplicated it.

---

## What It Does

- **macOS-Style Dock**: an animated bottom or side dock with fluid magnification, app badges and quick folder access, including Downloads and the Recycle Bin.
- **Halo and Floating Music**: shows the current song, artist and album art from Spotify and Windows media players, with playback controls.
- **Desktop Widgets**: world time, analog clock, calendar, system health (CPU, RAM, SSD), battery, volume, photo, music, movie and more.
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

The installer is written to `src-tauri/target/release/bundle/nsis`. Pushing a tag that starts with `v` (for example `v2.1.3`) builds it and publishes `DOKIII-Setup.exe` through GitHub Actions. See `docs/ARCHITECTURE.md` for how the pieces fit together.

---

## License

Released under the [MIT License](LICENSE).
