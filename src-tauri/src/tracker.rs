use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::time::{interval, Duration};

use crate::tray::TrayState;

/// Window position and size info emitted to the webview
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct WindowInfo {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub found: bool,
    /// True when the terminal is the foreground (active) window.
    /// Used to hide Clippy when another window covers the terminal.
    pub foreground: bool,
}

/// Find the Windows Terminal window and return its position/size.
/// Returns None if no Windows Terminal window is found.
fn find_terminal_window() -> Option<WindowInfo> {
    #[cfg(target_os = "windows")]
    {
        use windows::core::w;
        use windows::Win32::Foundation::RECT;
        use windows::Win32::UI::WindowsAndMessaging::{
            FindWindowW, GetForegroundWindow, GetWindowRect, IsWindowVisible,
        };

        unsafe {
            // Windows Terminal uses this window class
            let hwnd = match FindWindowW(w!("CASCADIA_HOSTING_WINDOW_CLASS"), None) {
                Ok(h) => h,
                Err(_) => return None,
            };

            if hwnd.is_invalid() {
                return None;
            }

            // Only track if the window is visible
            if !IsWindowVisible(hwnd).as_bool() {
                return None;
            }

            let mut rect = RECT::default();
            if GetWindowRect(hwnd, &mut rect).is_ok() {
                // Check if the terminal is the currently active (foreground) window
                let fg = GetForegroundWindow();
                let is_foreground = fg == hwnd;

                Some(WindowInfo {
                    x: rect.left,
                    y: rect.top,
                    width: rect.right - rect.left,
                    height: rect.bottom - rect.top,
                    found: true,
                    foreground: is_foreground,
                })
            } else {
                None
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

/// Start polling for Windows Terminal position changes.
/// Emits "terminal-position" events to the webview when the position changes.
pub async fn start_tracker(app_handle: AppHandle) {
    let mut ticker = interval(Duration::from_millis(300));
    let mut last_info: Option<WindowInfo> = None;
    let mut emit_count: u64 = 0;

    log::info!("Terminal window tracker started (polling every 300ms)");

    loop {
        ticker.tick().await;

        // Check if app is still visible (respect tray hide)
        let tray_state = app_handle.state::<Arc<TrayState>>();
        if !tray_state.is_visible() {
            continue;
        }

        let current = find_terminal_window();

        // Always emit the first few ticks so the JS side gets the position
        // even if the terminal hasn't moved since startup.
        let force_emit = emit_count < 5;

        match &current {
            Some(info) => {
                // Emit if position changed, OR during the first few ticks
                // to guarantee the JS side receives the initial position.
                if force_emit || last_info.as_ref() != Some(info) {
                    log::debug!(
                        "Terminal: {}x{} at ({}, {}) fg={}",
                        info.width,
                        info.height,
                        info.x,
                        info.y,
                        info.foreground
                    );
                    let _ = app_handle.emit("terminal-position", info);
                    last_info = current;
                    emit_count += 1;
                }
            }
            None => {
                // Terminal disappeared — emit a "not found" if we were tracking
                if last_info.is_some() {
                    log::debug!("Terminal window lost");
                    let lost = WindowInfo {
                        x: 0,
                        y: 0,
                        width: 0,
                        height: 0,
                        found: false,
                        foreground: false,
                    };
                    let _ = app_handle.emit("terminal-position", &lost);
                    last_info = None;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_info_equality() {
        let a = WindowInfo {
            x: 100,
            y: 200,
            width: 800,
            height: 600,
            found: true,
            foreground: true,
        };
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn test_window_info_serialization() {
        let info = WindowInfo {
            x: 50,
            y: 75,
            width: 1024,
            height: 768,
            found: true,
            foreground: false,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"x\":50"));
        assert!(json.contains("\"found\":true"));
    }

    #[test]
    fn test_find_terminal_window_returns_option() {
        // May or may not find a terminal — just verify it doesn't panic
        let result = find_terminal_window();
        if let Some(info) = result {
            assert!(info.width > 0);
            assert!(info.height > 0);
            assert!(info.found);
        }
    }
}
