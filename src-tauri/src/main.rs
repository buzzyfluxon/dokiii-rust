#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod apps;
mod audio;
mod bin;
mod commands;
mod config;
mod control_center;
mod photos;
mod cursor;
mod debuglog;
mod display;
mod filesystem;
mod icon;
mod lnk;
mod media;
mod netimg;
mod process;
mod quick_controls;
mod shellpaths;
mod snow;
mod screenshot;
mod startup;
mod state;
mod system;
mod taskbar;
mod tray;
mod updater;
mod wallpaper;

use std::sync::atomic::Ordering;

use tauri::{Emitter, Manager, WindowEvent};

use state::AppState;

fn main() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        taskbar::restore_on_exit();
        default_hook(info);
    }));

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
            commands::get_desktop_size,
            commands::trim_memory,
            commands::report_hit,
            debuglog::get_debug_hit,
            commands::set_modal_open,
            commands::set_popover_open,
            commands::app_ready,
            commands::hide_window,
            commands::quit_app,
            commands::show_widget_library,
            commands::show_settings,
            commands::set_auto_hide,
            taskbar::set_replace_taskbar,
            process::open_target,
            process::run_command,
            system::get_system_metrics,
            system::get_storage_info,
            system::get_battery_status,
            audio::get_audio_volume,
            audio::set_audio_volume,
            audio::toggle_audio_mute,
            quick_controls::get_quick_controls,
            quick_controls::set_brightness,
            quick_controls::set_radio,
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
            taskbar::start_from_config(&handle);
            display::start_work_area_watcher(handle.clone());
            wallpaper::start_watcher(handle.clone());
            updater::start(handle.clone());
            audio::start_watcher(handle.clone());
            system::start_trimmer();
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
        .build(tauri::generate_context!())
        .expect("failed to build DOKIII")
        .run(|_app, event| {
            if let tauri::RunEvent::Exit = event {
                taskbar::restore_on_exit();
            }
        });
}
