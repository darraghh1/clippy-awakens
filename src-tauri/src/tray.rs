use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

/// Shared state for tray icon — thread-safe via AtomicBool
pub struct TrayState {
    muted: AtomicBool,
    visible: AtomicBool,
}

impl TrayState {
    pub fn new() -> Self {
        Self {
            muted: AtomicBool::new(false),
            visible: AtomicBool::new(true),
        }
    }

    pub fn is_muted(&self) -> bool {
        self.muted.load(Ordering::Relaxed)
    }

    pub fn toggle_mute(&self) -> bool {
        let was_muted = self.muted.fetch_xor(true, Ordering::Relaxed);
        !was_muted // Return new state
    }

    #[allow(dead_code)]
    pub fn is_visible(&self) -> bool {
        self.visible.load(Ordering::Relaxed)
    }

    pub fn toggle_visibility(&self) -> bool {
        let was_visible = self.visible.fetch_xor(true, Ordering::Relaxed);
        !was_visible // Return new state
    }

    pub fn set_visible(&self, visible: bool) {
        self.visible.store(visible, Ordering::Relaxed);
    }
}

/// Build and register the system tray icon with context menu
pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let show_hide =
        MenuItem::with_id(app, "show_hide", "Hide Agent", true, None::<&str>)?;
    let mute =
        MenuItem::with_id(app, "mute", "Mute Sounds", true, None::<&str>)?;
    let separator1 = PredefinedMenuItem::separator(app)?;

    // Build agent picker submenu from available agents
    let agents = crate::agents::list_available_agents();
    let agent_items: Vec<MenuItem<tauri::Wry>> = agents
        .iter()
        .map(|agent| {
            let label = if agent.source == "user" {
                format!("{} (custom)", agent.name)
            } else {
                agent.name.clone()
            };
            let id = format!("agent_{}", agent.name);
            MenuItem::with_id(app, &id, &label, true, None::<&str>)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let agent_refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = agent_items
        .iter()
        .map(|i| i as &dyn tauri::menu::IsMenuItem<tauri::Wry>)
        .collect();

    let agent_submenu = Submenu::with_items(app, "Switch Agent", true, &agent_refs)?;

    // Build position (anchor) picker submenu
    let pos_br = MenuItem::with_id(app, "anchor_bottom-right", "Bottom Right", true, None::<&str>)?;
    let pos_bl = MenuItem::with_id(app, "anchor_bottom-left", "Bottom Left", true, None::<&str>)?;
    let pos_tr = MenuItem::with_id(app, "anchor_top-right", "Top Right", true, None::<&str>)?;
    let pos_tl = MenuItem::with_id(app, "anchor_top-left", "Top Left", true, None::<&str>)?;
    let position_submenu = Submenu::with_items(
        app,
        "Position",
        true,
        &[&pos_br, &pos_bl, &pos_tr, &pos_tl],
    )?;

    // Build vertical offset submenu
    let offset_up = MenuItem::with_id(app, "offset_up", "Nudge Up (+10px)", true, None::<&str>)?;
    let offset_down = MenuItem::with_id(app, "offset_down", "Nudge Down (-10px)", true, None::<&str>)?;
    let offset_reset = MenuItem::with_id(app, "offset_reset", "Reset Offset", true, None::<&str>)?;
    let offset_submenu = Submenu::with_items(
        app,
        "Vertical Offset",
        true,
        &[&offset_up, &offset_down, &offset_reset],
    )?;

    let separator2 = PredefinedMenuItem::separator(app)?;
    let quit =
        MenuItem::with_id(app, "quit", "Quit Clippy Awakens", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &show_hide,
            &mute,
            &separator1,
            &agent_submenu,
            &position_submenu,
            &offset_submenu,
            &separator2,
            &quit,
        ],
    )?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("Clippy Awakens — Listening for Claude Code events")
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| {
            let id = event.id.as_ref();
            match id {
                "show_hide" => {
                    let state = app.state::<Arc<TrayState>>();
                    let now_visible = state.toggle_visibility();
                    log::info!(
                        "Tray: {} Clippy",
                        if now_visible { "show" } else { "hide" }
                    );
                    let _ = app.emit("clippy-visibility", now_visible);
                }
                "mute" => {
                    let state = app.state::<Arc<TrayState>>();
                    let now_muted = state.toggle_mute();
                    log::info!(
                        "Tray: sounds {}",
                        if now_muted { "muted" } else { "unmuted" }
                    );
                    let _ = app.emit("clippy-mute", now_muted);
                }
                "quit" => {
                    log::info!("Tray: quit requested");
                    app.exit(0);
                }
                _ if id.starts_with("agent_") => {
                    let agent_name = id.strip_prefix("agent_").unwrap_or(id);
                    log::info!("Tray: switch agent to {}", agent_name);
                    let _ = app.emit("clippy-switch-agent", agent_name);
                }
                _ if id.starts_with("anchor_") => {
                    let anchor = id.strip_prefix("anchor_").unwrap_or(id);
                    log::info!("Tray: set anchor to {}", anchor);
                    let _ = app.emit("clippy-anchor", anchor);
                }
                "offset_up" | "offset_down" | "offset_reset" => {
                    let delta: i32 = match id {
                        "offset_up" => 10,
                        "offset_down" => -10,
                        "offset_reset" => 0, // sentinel: JS will interpret 0 as reset
                        _ => 0,
                    };
                    let is_reset = id == "offset_reset";
                    log::info!("Tray: offset {} (reset={})", delta, is_reset);
                    // Send both the delta and whether it's a reset
                    let payload = serde_json::json!({ "delta": delta, "reset": is_reset });
                    let _ = app.emit("clippy-offset", payload);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                let state = app.state::<Arc<TrayState>>();
                let now_visible = state.toggle_visibility();
                log::info!(
                    "Tray click: {} Clippy",
                    if now_visible { "show" } else { "hide" }
                );
                let _ = app.emit("clippy-visibility", now_visible);
            }
        })
        .build(app)?;

    log::info!("System tray initialized");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let state = TrayState::new();
        assert!(!state.is_muted(), "Should start unmuted");
        assert!(state.is_visible(), "Should start visible");
    }

    #[test]
    fn test_mute_toggle() {
        let state = TrayState::new();
        assert!(!state.is_muted());

        let now_muted = state.toggle_mute();
        assert!(now_muted, "toggle_mute should return new state (true)");
        assert!(state.is_muted());

        let now_muted = state.toggle_mute();
        assert!(!now_muted, "toggle_mute should return new state (false)");
        assert!(!state.is_muted());
    }

    #[test]
    fn test_visibility_toggle() {
        let state = TrayState::new();
        assert!(state.is_visible());

        let now_visible = state.toggle_visibility();
        assert!(!now_visible, "toggle should return new state (false)");
        assert!(!state.is_visible());

        let now_visible = state.toggle_visibility();
        assert!(now_visible, "toggle should return new state (true)");
        assert!(state.is_visible());
    }

    #[test]
    fn test_set_visible() {
        let state = TrayState::new();
        assert!(state.is_visible());

        state.set_visible(false);
        assert!(!state.is_visible());

        state.set_visible(true);
        assert!(state.is_visible());
    }

    #[test]
    fn test_set_visible_idempotent() {
        let state = TrayState::new();
        state.set_visible(true);
        state.set_visible(true);
        assert!(state.is_visible());

        state.set_visible(false);
        state.set_visible(false);
        assert!(!state.is_visible());
    }

    #[test]
    fn test_mute_and_visibility_independent() {
        let state = TrayState::new();
        state.toggle_mute();
        assert!(state.is_muted());
        assert!(state.is_visible(), "Mute should not affect visibility");

        state.toggle_visibility();
        assert!(state.is_muted(), "Visibility should not affect mute");
        assert!(!state.is_visible());
    }
}
