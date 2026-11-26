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

The installer is written to `src-tauri/target/release/bundle/nsis`. Pushing a tag that starts with `v` (for example `v2.2.2`) builds it and publishes `DOKIII-Installer.exe` and `DOKIII-Setup.exe` through GitHub Actions. The setup window lives in `installer/`; the workflow copies `DOKIII-Setup.exe` into `installer/src-tauri/payload` and builds it with `cargo tauri build --no-bundle` from `installer/src-tauri`. See `docs/ARCHITECTURE.md` for how the pieces fit together.

---

## License

Released under the [MIT License](LICENSE).
