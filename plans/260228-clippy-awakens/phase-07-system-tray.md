---
title: "Phase 07: System Tray Integration"
description: "Add a system tray icon with context menu for Show/Hide Clippy, Mute Sounds, Agent Picker submenu, and Quit. Implement auto-hide timer so Clippy retreats to tray after events, and manual summon via tray click."
skill: none
status: pending
group: "system-polish"
dependencies: ["phase-04", "phase-06"]
tags: [phase, tauri, tray, system-tray, auto-hide]
created: 2026-02-28
updated: 2026-02-28
---

# Phase 07: System Tray Integration

**Context:** [Master Plan](./plan.md) | **Dependencies:** P04, P06 | **Status:** Pending

---

## Overview

Add a Windows system tray icon for Clippy Awakens. The tray provides a persistent presence — the agent lives in the tray by default, pops up on events, retreats after timeout. The tray icon's context menu gives quick access to Show/Hide, Mute Sounds, an Agent Picker submenu to switch between all 10 bundled agents (plus any user-added agents), and Quit.

**Goal:** Agent lives in the system tray. Events cause it to pop up temporarily. Users can summon/dismiss via tray icon click, switch agents via the picker submenu, and quit the app cleanly from the tray menu.

---

## Context & Workflow

### How This Phase Fits Into the Project

- **UI Layer:** Coordinates with the JS bridge's auto-hide timer (Phase 04)
  - Tray "Show" sends a Tauri event to make Clippy visible
  - Tray "Hide" sends a Tauri event to hide Clippy

- **Server Layer:** Creates tray icon and menu in Rust
  - Uses Tauri's built-in tray API
  - Manages app lifecycle (quit from tray)

- **Database Layer:** N/A

- **Integrations:** Windows system tray (notification area)

### User Workflow

**Trigger:** User wants to control Clippy's visibility or quit the app.

**Steps:**
1. App starts — Clippy shows briefly with greeting, then retreats to tray
2. Tray icon appears in Windows notification area (paperclip icon)
3. Event arrives — agent pops up, animates, speaks, then auto-hides after 8s
4. User clicks tray icon — toggles agent visibility
5. User right-clicks tray icon — context menu: Show/Hide, Mute Sounds, Agent Picker, Quit
6. User opens "Switch Agent >" submenu — sees all 10 bundled + any user agents, current has checkmark
7. User selects "Merlin" — agent swaps via `switchAgent()` (Phase 03), preference saved to config
8. User selects "Quit" — app exits cleanly

**Success Outcome:** Clippy is unobtrusive — it lives in the tray and only appears when something happens or the user wants it.

### Problem Being Solved

**Pain Point:** Without a tray icon, there's no way to quit the app (transparent window has no close button) and no way to manually summon/dismiss Clippy.
**Alternative Approach:** Could use a keyboard shortcut to toggle, but tray is the Windows-native pattern for background apps.

### Integration Points

**Upstream Dependencies:**
- Phase 04: JS bridge manages Clippy show/hide lifecycle
- Phase 06: Sound mute state needs to be accessible from tray menu

**Downstream Consumers:**
- Phase 08: Final integration verifies tray + events + animations work together

**Data Flow:**
```
Tray click → Rust handler → emit("clippy-toggle") → JS bridge → agent.show/hide
Tray "Quit" → Rust handler → app.exit(0)
Tray "Mute" → Rust handler → set mute state → emit("clippy-mute", true/false)
Tray "Switch Agent > Merlin" → Rust handler → emit("clippy-switch-agent", "Merlin")
  → JS bridge → switchAgent("Merlin") → save to config
```

---

## Prerequisites & Clarifications

### Questions for User

1. **Tray Icon Image:** Should we use a custom paperclip icon or the default Tauri icon?
   - **Context:** A recognizable paperclip icon makes it easy to find in the tray. Custom icons need to be created/sourced.
   - **Assumptions if unanswered:** Use a simple paperclip icon. Can be generated or use a free clipart. For MVP, the app icon works.
   - **Impact:** Cosmetic only — doesn't affect functionality

2. **Startup Behavior:** Should Clippy show immediately on app launch, or start hidden in tray?
   - **Context:** Showing on launch lets the user know the app is running. Starting hidden is less intrusive.
   - **Assumptions if unanswered:** Show briefly on launch with a greeting ("Clippy Awakens! I'm listening for events."), then auto-hide to tray after 5 seconds.
   - **Impact:** Affects first-run user experience

3. **Mute Persistence:** Should the mute state persist across app restarts?
   - **Context:** Could save mute state to a config file. For MVP, it resets on restart.
   - **Assumptions if unanswered:** Mute resets on restart. Persistence is a future enhancement.
   - **Impact:** Minor UX consideration

### Validation Checklist

- [ ] Phase 04 completed — JS bridge with show/hide lifecycle
- [ ] Phase 06 completed — Sound playback with mute capability
- [ ] Tauri tray icon feature enabled in Cargo.toml (`features = ["tray-icon"]`)

---

## Requirements

### Functional

- System tray icon appears when app starts
- Left-click tray icon toggles agent visibility (show/hide)
- Right-click tray icon shows context menu with:
  - "Show/Hide Agent" (toggles based on current state)
  - "Mute Sounds" / "Unmute Sounds" (toggles)
  - Separator
  - "Switch Agent >" submenu listing all available agents (bundled + user)
    - Current agent has a checkmark/indicator
    - Selecting an agent calls `switchAgent()` and saves preference
  - Separator
  - "Quit"
- "Quit" exits the app cleanly
- Agent auto-hides to tray after event timeout (8 seconds)
- Agent appears on events even when manually hidden (events override manual hide)
- Agent picker dynamically populated from `list_available_agents` (Phase 03)

### Technical

- Tauri v2 tray API (`tauri::tray::TrayIconBuilder`)
- Tray icon built in Rust during `setup()`
- Event emission for show/hide/mute to coordinate with JS bridge
- App icon used as tray icon (PNG format)
- Menu items update text dynamically based on state

---

## Decision Log

### Events Override Manual Hide (ADR-07-01)

**Date:** 2026-02-28
**Status:** Accepted

**Context:** If the user manually hides Clippy via tray, and then an event arrives — should Clippy stay hidden or pop up?

**Decision:** Events always show Clippy, even if manually hidden. The purpose of the app is notifications — hiding should suppress idle Clippy, not silence events.

**Consequences:**
- **Positive:** User never misses an event
- **Negative:** User can't fully silence Clippy without quitting (but can mute sounds)
- **Neutral:** Auto-hide timer still works — Clippy retreats after 8 seconds

**Alternatives Considered:**
1. Manual hide suppresses events: Defeats the purpose of the notification app
2. "Do Not Disturb" mode: Good future enhancement, but too complex for MVP

---

## Implementation Steps

### Step 0: Test Definition (TDD)

#### 0.1: Rust Unit Tests

- [ ] Add tests for tray state management:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mute_toggle() {
        let state = TrayState::new();
        assert!(!state.is_muted());
        state.toggle_mute();
        assert!(state.is_muted());
        state.toggle_mute();
        assert!(!state.is_muted());
    }

    #[test]
    fn test_visibility_toggle() {
        let state = TrayState::new();
        assert!(state.is_visible()); // Visible by default
        state.toggle_visibility();
        assert!(!state.is_visible());
        state.toggle_visibility();
        assert!(state.is_visible());
    }
}
```

---

### Step 1: Create Tray State Management

#### 1.1: Create src-tauri/src/tray.rs

- [ ] Implement tray state and setup:

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    AppHandle, Manager,
};
use log::info;

/// Shared state for tray icon
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
```

---

### Step 2: Build Tray Icon and Menu

#### 2.1: Add Tray Setup Function

- [ ] Add tray builder to `src-tauri/src/tray.rs`:

```rust
pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let show_hide = MenuItem::with_id(app, "show_hide", "Hide Agent", true, None::<&str>)?;
    let mute = MenuItem::with_id(app, "mute", "Mute Sounds", true, None::<&str>)?;
    let separator1 = PredefinedMenuItem::separator(app)?;

    // Build agent picker submenu
    let agents = crate::agents::list_available_agents();
    let mut agent_items: Vec<MenuItem<tauri::Wry>> = Vec::new();
    for agent in &agents {
        let label = if agent.source == "user" {
            format!("{} (custom)", agent.name)
        } else {
            agent.name.clone()
        };
        let id = format!("agent_{}", agent.name);
        agent_items.push(MenuItem::with_id(app, &id, &label, true, None::<&str>)?);
    }
    let agent_refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> =
        agent_items.iter().map(|i| i as &dyn tauri::menu::IsMenuItem<tauri::Wry>).collect();
    let agent_submenu = Submenu::with_items(app, "Switch Agent", true, &agent_refs)?;

    let separator2 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Clippy Awakens", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[
        &show_hide, &mute, &separator1, &agent_submenu, &separator2, &quit
    ])?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("Clippy Awakens — Listening for Claude Code events")
        .on_menu_event(move |app, event| {
            match event.id.as_ref() {
                "show_hide" => {
                    let state = app.state::<Arc<TrayState>>();
                    let now_visible = state.toggle_visibility();
                    let action = if now_visible { "show" } else { "hide" };
                    info!("Tray: {} Clippy", action);
                    let _ = app.emit("clippy-visibility", now_visible);
                    // Update menu item text
                    if let Some(item) = app.tray_by_id("main") {
                        // Menu text update would go here
                    }
                }
                "mute" => {
                    let state = app.state::<Arc<TrayState>>();
                    let now_muted = state.toggle_mute();
                    info!("Tray: sounds {}", if now_muted { "muted" } else { "unmuted" });
                    let _ = app.emit("clippy-mute", now_muted);
                }
                "quit" => {
                    info!("Tray: quit requested");
                    app.exit(0);
                }
                id if id.starts_with("agent_") => {
                    let agent_name = id.strip_prefix("agent_").unwrap_or(id);
                    info!("Tray: switch agent to {}", agent_name);
                    let _ = app.emit("clippy-switch-agent", agent_name);
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
                info!("Tray click: {} Clippy", if now_visible { "show" } else { "hide" });
                let _ = app.emit("clippy-visibility", now_visible);
            }
        })
        .build(app)?;

    Ok(())
}
```

---

### Step 3: Integrate Tray with Main

#### 3.1: Update main.rs

- [ ] Add tray module and setup:

```rust
mod events;
mod server;
mod sounds;
mod tray;

use std::sync::Arc;

fn main() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(Arc::new(tray::TrayState::new()))
        .setup(|app| {
            let handle = app.handle().clone();

            // Setup system tray
            tray::setup_tray(&handle)?;

            // Spawn HTTP server
            tauri::async_runtime::spawn(async move {
                server::start_server(handle).await;
            });

            log::info!("Clippy Awakens started with system tray");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

### Step 4: Update JS Bridge for Tray Events

#### 4.1: Add Visibility Event Listener

- [ ] Update `ui/clippy-bridge.js` or `ui/index.html` to handle tray events:

```javascript
// Listen for tray visibility toggle
if (window.__TAURI__) {
    window.__TAURI__.event.listen('clippy-visibility', function(event) {
        var visible = event.payload;
        if (visible && currentAgent) {
            currentAgent.show();
            currentAgent.play('Greeting');
            currentAgent.speak('You summoned me! What can I do for you?');
        } else if (!visible && currentAgent) {
            currentAgent.play('GoodBye');
            setTimeout(function() {
                currentAgent.hide();
            }, 1500);
        }
    });

    window.__TAURI__.event.listen('clippy-mute', function(event) {
        var muted = event.payload;
        console.log('Sound muted:', muted);
        // Mute state is handled on the Rust side for notification sounds
        // This event is informational for the JS side
    });

    // Listen for agent switch from tray menu
    window.__TAURI__.event.listen('clippy-switch-agent', function(event) {
        var agentName = event.payload;
        console.log('Switching agent to:', agentName);
        switchAgent(agentName, function(agent) {
            agent.speak("I'm " + agentName + "! Nice to meet you.");
        });
    });
}
```

---

### Step 5: Integrate Mute State with Sound Playback

#### 5.1: Update server.rs to Check Mute State

- [ ] Modify `emit_event` in `server.rs` to check mute state before playing sound:

```rust
use crate::tray::TrayState;

fn emit_event(state: &AppState, event_type: &str) {
    let payload = ClippyEvent {
        event_type: event_type.to_string(),
    };
    info!("Event received: {}", event_type);

    // Check mute state before playing sound
    let tray_state = state.app_handle.state::<Arc<TrayState>>();
    if !tray_state.is_muted() {
        sounds::play_event_sound(event_type);
    } else {
        info!("Sound muted, skipping playback for: {}", event_type);
    }

    // Always emit to webview (animation still plays even when muted)
    if let Err(e) = state.app_handle.emit("clippy-event", &payload) {
        warn!("Failed to emit clippy-event: {}", e);
    }
}
```

---

### Step 6: Verify Tray Functionality

#### 6.1: Test Tray Icon

- [ ] Start app — tray icon appears in notification area
- [ ] Hover over tray icon — tooltip shows "Clippy Awakens"
- [ ] Left-click tray icon — toggles Clippy visibility
- [ ] Right-click tray icon — context menu appears

#### 6.2: Test Menu Actions

- [ ] Click "Hide Agent" — agent hides, menu changes to "Show Agent"
- [ ] Click "Show Agent" — agent appears with greeting
- [ ] Click "Mute Sounds" — sounds stop, menu changes to "Unmute Sounds"
- [ ] Send event while muted — agent animates but no sound
- [ ] Click "Quit" — app exits cleanly

#### 6.3: Test Agent Picker

- [ ] Open "Switch Agent >" submenu — lists all 10 bundled agents
- [ ] Select "Merlin" — current agent disappears, Merlin appears with intro speech
- [ ] Select "Bonzi" — Merlin disappears, Bonzi appears
- [ ] Verify current agent reflected in submenu (checkmark or similar)
- [ ] If user agents exist in `%APPDATA%/clippy-awakens/agents/`, they appear with "(custom)" suffix

#### 6.4: Test Event Override

- [ ] Manually hide agent via tray
- [ ] Send `curl http://localhost:9999/complete`
- [ ] Verify: agent pops up despite being manually hidden (events override)
- [ ] After 8 seconds, agent auto-hides back to tray

---

## Verifiable Acceptance Criteria

**Critical Path:**

- [ ] System tray icon appears on app start
- [ ] Left-click tray toggles agent visibility
- [ ] Right-click shows context menu with Show/Hide, Mute, Agent Picker, Quit
- [ ] "Switch Agent >" submenu lists all 10 bundled agents
- [ ] Selecting an agent from submenu swaps the active agent
- [ ] "Quit" exits the app cleanly
- [ ] Mute stops notification sounds but keeps animations
- [ ] Events show agent even when manually hidden

**Quality Gates:**

- [ ] Tray icon is recognizable (not a generic white square)
- [ ] Menu items update their text based on current state
- [ ] Agent picker dynamically populated (includes user agents if present)
- [ ] No memory leaks from tray event handlers

**Integration:**

- [ ] Works with Phase 03 `switchAgent()` function
- [ ] Works with Phase 04 JS bridge auto-hide timer
- [ ] Works with Phase 06 sound mute state
- [ ] Phase 08 config persistence saves selected agent

---

## Quality Assurance

### Test Plan

#### Manual Testing

- [ ] **Tray appears:** Start app, check notification area
  - Expected: Clippy icon visible with tooltip
  - Actual: [To be filled]

- [ ] **Toggle visibility:** Click tray icon twice
  - Expected: Clippy hides then shows
  - Actual: [To be filled]

- [ ] **Mute toggle:** Mute, send event, unmute, send event
  - Expected: First event silent, second plays sound
  - Actual: [To be filled]

- [ ] **Quit:** Right-click → Quit
  - Expected: App exits, tray icon disappears
  - Actual: [To be filled]

#### Automated Testing

```bash
cd src-tauri && cargo test
```

### Review Checklist

- [ ] **Code Quality:**
  - [ ] `cargo test` passes
  - [ ] AtomicBool used correctly for thread-safe state
  - [ ] Clean tray API usage

- [ ] **Security:**
  - [ ] No privilege escalation via tray menu
  - [ ] Quit is clean (no orphaned processes)

---

## Dependencies

### Upstream (Required Before Starting)

- Phase 04: JS bridge with show/hide lifecycle
- Phase 06: Sound playback engine (for mute integration)
- Tauri `tray-icon` feature in Cargo.toml

### Downstream (Will Use This Phase)

- Phase 08: Final integration testing

### External Services

- None

---

## Completion Gate

### Sign-off

- [ ] All acceptance criteria met
- [ ] All Rust tests passing
- [ ] Code review passed
- [ ] Phase marked DONE in plan.md
- [ ] Committed: `feat(tray): phase 07 — system tray integration`

---

## Notes

### Technical Considerations

- Tauri v2 tray API differs from v1 — use `TrayIconBuilder` not `SystemTray`
- `AtomicBool` is sufficient for simple toggle state — no mutex needed
- Menu item text updates require re-building the menu or using Tauri's dynamic menu API
- The tray icon should use `.ico` format on Windows for best rendering

### Known Limitations

- Menu text doesn't dynamically update in MVP (shows "Show/Hide" as static text) — this can be improved
- Agent picker submenu is built at startup — new user agents added while running won't appear until restart
- No "Do Not Disturb" mode — events always show the agent

### Future Enhancements

- "Do Not Disturb" mode (suppress events entirely)
- Dynamic agent picker refresh (detect new agents without restart)
- Tray icon badge (show event count)
- Custom tray icon per agent

---

**Previous:** [[phase-06-sound-playback|Phase 06: Windows Sound Playback]]
**Next:** [[phase-08-integration-build|Phase 08: Final Integration & Build]]
