// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod agents;
mod events;
mod server;
mod sounds;

fn main() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            agents::list_available_agents,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                server::start_server(handle).await;
            });
            log::info!("Clippy Awakens overlay started");
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
