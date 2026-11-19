# DOKIII architecture

DOKIII is a Tauri 2 app. The Rust backend lives in `src-tauri` and the React interface lives in `src/renderer`.

## Build

1. Install Rust (stable, MSVC toolchain) and the Tauri CLI: `cargo install tauri-cli --version "^2" --locked`
2. `npm install`
3. Dev: `npm run tauri:dev`
4. Release installer: `npm run tauri:build` (output in `src-tauri/target/release/bundle/nsis`)

## How it works

- `src/renderer/platform/tauriBridge.ts` installs `window.electronAPI` when the page runs inside Tauri, so the UI code is unchanged.
- The window is transparent, frameless and always click-through. A Rust thread (`src-tauri/src/cursor.rs`) polls the cursor and emits `dokiii://cursor` only when it moves.
- The page runs `document.elementFromPoint` on those coordinates and reports whether the cursor is over real UI (`report_hit`). Rust then switches click-through off while the cursor is over UI, or while the left button is held after a grab, so drags never drop.
- Modals and popovers force click-through off until they close.
- Config is read and written at `%APPDATA%\dokiii\config.json`, the same file Electron uses, so existing settings carry over.

## Status

Everything the old Electron build did now lives in Rust. The Rust has not been compiled yet, so expect a first round of compiler fixes.

| Area | Module |
| --- | --- |
| Window, work area fitting, click-through loop | `main.rs`, `display.rs`, `cursor.rs` |
| Config store, displays, tray, single instance, modal flow | `config.rs`, `commands.rs`, `tray.rs` |
| Open, launch, run command, uninstaller | `process.rs` |
| Snow window tracker | `snow.rs` |
| CPU, RAM, disk, storage, battery | `system.rs` |
| Volume and mute | `audio.rs` |
| Recycle bin | `bin.rs` |
| Launch on startup | `startup.rs` |
| Recent items, downloads, screenshots list, file search | `filesystem.rs`, `lnk.rs`, `shellpaths.rs` |
| Media info, controls, seek | `media.rs` |
| Installed apps, app icons, pickers | `apps.rs`, `icon.rs` |
| Favicon and image cache | `netimg.rs` |
| Wallpaper palette and watcher | `wallpaper.rs` |
| Screenshot capture | `screenshot.rs` |
| Auto updater | `updater.rs` |

## Differences from the old Electron build

- Images picked with the image picker are served through Tauri's asset protocol. Image paths already saved in an Electron config as `file:///...` will not load until they are picked again.
- The Spotify window-title fallback for media info is not ported. Spotify reports through the Windows media session API, so it should not be needed.
- The updater reads the latest GitHub release of `buzzyfluxon/dokiii-fluxon`, downloads the asset ending in `-setup.exe` and runs it with `/P /UPDATE`. It relies on HTTPS and the repository for trust, with no signature check. Tauri's signed updater is the stricter option if you want it later.
- Screenshots use GDI (`BitBlt` and `PrintWindow`) instead of the old desktop capturer, so protected or hardware-accelerated windows may come out black.
- The `window.electronAPI` name is kept so the React code did not need to change. It is now provided by `src/renderer/platform/tauriBridge.ts`.
- The old custom installer and uninstaller pages are gone. The standard NSIS installer is used.
- The window position is no longer remembered between runs, and plugging in or removing a monitor does not refit the window yet.
