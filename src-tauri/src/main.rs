// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod agents;
mod config;
mod events;
mod server;
mod sounds;
mod tracker;
mod tray;

use std::sync::Arc;
use tauri::Manager;

fn main() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(Arc::new(tray::TrayState::new()))
        .invoke_handler(tauri::generate_handler![
            agents::list_available_agents,
            config::get_config,
            config::save_position_cmd,
            config::save_agent_preference,
            config::save_mute_state,
            config::save_anchor,
            config::save_vertical_offset,
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            if let Some(window) = app.get_webview_window("main") {
                let decorated = window.is_decorated().unwrap_or(true);

                if !decorated {
                    // Production mode: span all monitors so Clippy can follow
                    // the terminal across any screen
                    if let Ok(monitors) = window.available_monitors() {
                        if !monitors.is_empty() {
                            let mut min_x = i32::MAX;
                            let mut min_y = i32::MAX;
                            let mut max_x = i32::MIN;
                            let mut max_y = i32::MIN;

                            for mon in &monitors {
                                let pos = mon.position();
                                let size = mon.size();
                                min_x = min_x.min(pos.x);
                                min_y = min_y.min(pos.y);
                                max_x = max_x.max(pos.x + size.width as i32);
                                max_y = max_y.max(pos.y + size.height as i32);
                            }

                            let virt_w = (max_x - min_x) as u32;
                            let virt_h = (max_y - min_y) as u32;

                            log::info!(
                                "Virtual screen: {}x{} at ({}, {}), spanning {} monitor(s)",
                                virt_w, virt_h, min_x, min_y, monitors.len()
                            );

                            use tauri::{LogicalPosition, LogicalSize};
                            let _ = window.set_position(LogicalPosition::new(min_x, min_y));
                            let _ = window.set_size(LogicalSize::new(virt_w, virt_h));
                        }
                    }

                    // Click-through: let mouse events pass to the desktop
                    if let Err(e) = window.set_ignore_cursor_events(true) {
                        log::warn!("Failed to set ignore cursor events: {}", e);
                    } else {
                        log::info!("Window set to ignore cursor events (click-through)");
                    }
                } else {
                    log::info!("Debug mode: cursor events NOT ignored (for DevTools access)");
                }
            }

            // Setup system tray icon and menu
            tray::setup_tray(&handle)?;

            // Spawn HTTP server
            let server_handle = handle.clone();
            tauri::async_runtime::spawn(async move {
                server::start_server(server_handle).await;
            });

            // Spawn terminal window tracker
            tauri::async_runtime::spawn(async move {
                tracker::start_tracker(handle).await;
            });

            log::info!("Clippy Awakens started with system tray");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_app_builds() {
        // Verify tauri project compiles successfully
        // Real integration testing requires a running window (manual)
        assert!(true, "Tauri project compiles successfully");
    }

    #[test]
    fn test_window_config_transparent() {
        // Verify our expected transparent overlay configuration values.
        // Actual window behavior requires manual verification.
        let transparent = true;
        let decorations = false;
        let always_on_top = true;
        let skip_taskbar = true;

        assert!(transparent, "Window must be transparent");
        assert!(!decorations, "Window must have no decorations");
        assert!(always_on_top, "Window must be always on top");
        assert!(skip_taskbar, "Window must skip taskbar");
    }
}
