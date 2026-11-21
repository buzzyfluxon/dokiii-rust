#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod apps;
mod audio;
mod bin;
mod commands;
mod config;
mod control_center;
mod photos;
mod cursor;
mod display;
mod filesystem;
mod icon;
mod lnk;
mod media;
mod netimg;
mod process;
mod shellpaths;
mod snow;
mod screenshot;
mod startup;
mod state;
mod system;
mod tray;
mod updater;
mod wallpaper;

use std::sync::atomic::Ordering;

use tauri::{Emitter, Manager, WindowEvent};

use state::AppState;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
            }
            let _ = app.emit("dock:showApp", ());
        }))
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::config_get,
            commands::config_save,
            commands::get_displays,
            commands::report_hit,
            commands::set_modal_open,
            commands::set_popover_open,
            commands::app_ready,
            commands::hide_window,
            commands::quit_app,
            commands::show_widget_library,
            commands::show_settings,
            commands::set_auto_hide,
            process::open_target,
            process::run_command,
            system::get_system_metrics,
            system::get_app_memory,
            system::get_storage_info,
            system::get_battery_status,
            audio::get_audio_volume,
            audio::set_audio_volume,
            audio::toggle_audio_mute,
            bin::recycle_bin_count,
            bin::recycle_bin_empty,
            startup::get_launch_on_startup,
            startup::set_launch_on_startup,
            filesystem::get_recent_items,
            filesystem::get_downloads,
            filesystem::get_recent_screenshots,
            filesystem::get_screenshots_dir,
            filesystem::search_files,
            snow::set_snow_active,
            control_center::open_control_center,
            control_center::cc_ready,
            control_center::cc_show,
            control_center::prewarm_control_center,
            control_center::cc_close,
            control_center::cc_minimize,
            control_center::cc_toggle_maximize,
            control_center::cc_drag,
            snow::set_snow_front,
            snow::clear_snow,
            media::get_media_info,
            media::media_control,
            media::media_seek,
            apps::get_installed_apps,
            apps::get_app_icon,
            apps::select_app_file,
            apps::select_image_file,
            photos::save_photo,
            apps::pick_icon_image,
            netimg::cache_image,
            netimg::fetch_favicon,
            wallpaper::get_wallpaper_colors,
            screenshot::capture_screenshot,
            process::show_uninstall,
            updater::check_for_updates,
            updater::start_update_download,
            updater::install_update,
            updater::dismiss_update,
            updater::get_updater_status,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            let window = app
                .get_webview_window("main")
                .ok_or("main window is missing")?;
            display::fit_window_to_work_area(&handle, &window);
            let _ = window.set_ignore_cursor_events(true);
            tray::create(&handle)?;
            wallpaper::start_watcher(handle.clone());
            updater::start(handle.clone());
            audio::start_watcher(handle.clone());
            cursor::start(handle);
            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } if window.label() == "main" => {
                api.prevent_close();
                let modal = window.state::<AppState>().modal.load(Ordering::Relaxed);
                if modal {
                    let _ = window.emit("dock:closeApp", ());
                } else {
                    let _ = window.hide();
                }
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                let app = window.app_handle();
                if let Some(main) = app.get_webview_window("main") {
                    display::fit_window_to_work_area(app, &main);
                }
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("failed to run DOKIII");
}
