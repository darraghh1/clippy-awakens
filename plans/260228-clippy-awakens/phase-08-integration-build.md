---
title: "Phase 08: Final Integration & Build"
description: "End-to-end integration testing, position persistence, config persistence (preferred agent + settings), error resilience hardening, Windows installer configuration with all 10 agent assets bundled, README documentation, and CLAUDE.md setup."
skill: none
status: pending
group: "system-polish"
dependencies: ["phase-07"]
tags: [phase, integration, build, installer, documentation, polish]
created: 2026-02-28
updated: 2026-02-28
---

# Phase 08: Final Integration & Build

**Context:** [Master Plan](./plan.md) | **Dependencies:** P07 | **Status:** Pending

---

## Overview

The final phase brings everything together: end-to-end integration testing of the full event pipeline (curl → HTTP → agent animation + sound), position persistence so the agent remembers where the user dragged it, config persistence (`config.json` for preferred agent, mute state, and settings), error resilience hardening, Windows installer configuration via Tauri's bundler (bundling all 10 agent directories), and project documentation (README.md + CLAUDE.md).

**Goal:** A production-ready, installable Windows desktop app with all 10 agents bundled, config persistence, that reliably receives Claude Code events and responds with animated agent notifications.

---

## Context & Workflow

### How This Phase Fits Into the Project

- **UI Layer:** Adds position persistence (save/restore agent's drag position)
  - Uses Tauri's app data directory for storage

- **Server Layer:** Error resilience hardening across all Rust modules
  - Graceful degradation when components fail independently
  - Config persistence: preferred agent, mute state, position

- **Build Layer:** Tauri bundler configuration for Windows .msi/.exe installer
  - App icon, metadata, installer settings
  - All 10 agent directories (~13MB) bundled as resources

- **Documentation:** README.md with setup/build instructions, CLAUDE.md for future Claude Code sessions

### User Workflow

**Trigger:** All previous phases complete — time to polish and ship.

**Steps:**
1. Run full integration test suite (curl all 5 events, verify Clippy + sound)
2. Test error scenarios (port in use, no audio, missing files)
3. Build Windows installer with `cargo tauri build`
4. Install on Windows, verify the installed app works identically
5. Write documentation for setup and future development

**Success Outcome:** A polished `.msi` installer that the user can install on Windows, start from the Start menu or tray, and immediately receive Claude Code notifications with Clippy animations.

### Problem Being Solved

**Pain Point:** Individual phases work in isolation but need end-to-end verification. Also missing installer and docs.
**Alternative Approach:** Shipping without integration testing risks subtle interaction bugs between phases.

### Integration Points

**Upstream Dependencies:**
- All previous phases (P01-P07)

**Downstream Consumers:**
- End user — this is the final deliverable
- Future development sessions (CLAUDE.md)

---

## Prerequisites & Clarifications

### Questions for User

1. **Auto-Start on Boot:** Should the app start automatically when Windows boots?
   - **Context:** Can add to Windows Startup via registry or Task Scheduler.
   - **Assumptions if unanswered:** Don't auto-start for MVP. Document how to add it manually. Can be added as a tray menu option later.
   - **Impact:** Requires registry modification or startup folder shortcut

2. **Installer Type:** Prefer .msi (Windows Installer) or .exe (NSIS)?
   - **Context:** Tauri supports both. .msi is more "proper" for enterprise. .exe is simpler for personal use.
   - **Assumptions if unanswered:** Build both — Tauri can generate both with `targets: "all"`.
   - **Impact:** Minimal — both work, just different installer UX

3. **App Signing:** Should the app be code-signed?
   - **Context:** Unsigned apps trigger Windows SmartScreen warnings. Code signing certificates cost ~$100-400/year.
   - **Assumptions if unanswered:** Don't sign for MVP. Accept SmartScreen warning. Document how to sign later.
   - **Impact:** Users see "Windows protected your PC" warning on first run

### Validation Checklist

- [ ] All phases P01-P07 completed and working
- [ ] Tauri app compiles and runs with `cargo tauri dev`
- [ ] HTTP server responds on port 9999
- [ ] All 5 event types trigger animations and sounds
- [ ] System tray working with show/hide/mute/quit

---

## Requirements

### Functional

- Full end-to-end event pipeline works (curl → animation + sound)
- Agent remembers its last drag position across app restarts
- Preferred agent persists across restarts (saved in config.json)
- Mute state persists across restarts (saved in config.json)
- App loads last-used agent on startup instead of always defaulting to Clippy
- App handles error scenarios gracefully (no crashes)
- Windows installer builds successfully with all 10 agents bundled
- README.md and CLAUDE.md provide complete documentation

### Technical

- Config persistence via `config.json` in Tauri's app data dir (`%APPDATA%/com.digitalmastery.clippy-awakens/`)
  - Fields: `agent` (string), `muted` (bool), `position` (object with x/y)
- Tauri bundler config for Windows installer (.msi/.exe)
- All 10 agent directories bundled as Tauri resources (~13MB)
- App icon set (multiple sizes for Windows)
- Error handling audit across all Rust modules
- Comprehensive logging for debugging

---

## Decision Log

### Unified Config File in App Data Dir (ADR-08-01)

**Date:** 2026-02-28
**Status:** Accepted

**Context:** Need to persist user preferences across restarts: preferred agent, mute state, screen position. Options: (1) Single config.json, (2) Separate files per setting, (3) Windows Registry.

**Decision:** Use a single `config.json` in Tauri's `app_data_dir()` at `%APPDATA%/com.digitalmastery.clippy-awakens/config.json`.

**Config schema:**
```json
{
  "agent": "Clippy",
  "muted": false,
  "position": { "x": 1536.0, "y": 864.0 }
}
```

**Consequences:**
- **Positive:** Single file for all settings, clean, portable, easy to debug/edit
- **Negative:** Need to handle missing/corrupt file on first run (use defaults)
- **Neutral:** Replaces the previous position-only `position.json` approach

---

## Implementation Steps

### Step 0: Test Definition (TDD)

#### 0.1: Rust Tests for Config Persistence

- [ ] Add tests for config save/load:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.agent, "Clippy");
        assert!(!config.muted);
        assert!(config.position.x > 0.0);
        assert!(config.position.y > 0.0);
    }

    #[test]
    fn test_save_and_load_config() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");

        let config = AppConfig {
            agent: "Merlin".to_string(),
            muted: true,
            position: Position { x: 100.0, y: 200.0 },
        };
        save_config(&path, &config).unwrap();

        let loaded = load_config(&path).unwrap();
        assert_eq!(loaded.agent, "Merlin");
        assert!(loaded.muted);
        assert_eq!(loaded.position.x, 100.0);
        assert_eq!(loaded.position.y, 200.0);
    }

    #[test]
    fn test_load_missing_file_returns_defaults() {
        let path = PathBuf::from("/nonexistent/config.json");
        let config = load_config(&path).unwrap_or_default();
        assert_eq!(config.agent, "Clippy");
    }
}
```

---

### Step 1: Config Persistence

#### 1.1: Create Config Module

- [ ] Create `src-tauri/src/config.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use log::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

impl Default for Position {
    fn default() -> Self {
        // Bottom-right area (80% of a 1920x1080 screen)
        Self { x: 1536.0, y: 864.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub agent: String,
    pub muted: bool,
    pub position: Position,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            agent: "Clippy".to_string(),
            muted: false,
            position: Position::default(),
        }
    }
}

pub fn get_config_file(app_handle: &tauri::AppHandle) -> PathBuf {
    let mut path = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir");
    fs::create_dir_all(&path).ok();
    path.push("config.json");
    path
}

pub fn load_config(path: &PathBuf) -> Result<AppConfig, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let config: AppConfig = serde_json::from_str(&content)?;
    Ok(config)
}

pub fn save_config(
    path: &PathBuf,
    config: &AppConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = serde_json::to_string_pretty(config)?;
    fs::write(path, content)?;
    Ok(())
}
```

#### 1.2: Add Tauri Commands for Config

- [ ] Add commands to main.rs or a commands module:

```rust
#[tauri::command]
fn get_config(app_handle: tauri::AppHandle) -> AppConfig {
    let path = config::get_config_file(&app_handle);
    config::load_config(&path).unwrap_or_default()
}

#[tauri::command]
fn save_config_cmd(app_handle: tauri::AppHandle, agent: String, muted: bool, x: f64, y: f64) {
    let path = config::get_config_file(&app_handle);
    let cfg = AppConfig {
        agent,
        muted,
        position: Position { x, y },
    };
    if let Err(e) = config::save_config(&path, &cfg) {
        warn!("Failed to save config: {}", e);
    }
}

#[tauri::command]
fn save_position_cmd(app_handle: tauri::AppHandle, x: f64, y: f64) {
    let path = config::get_config_file(&app_handle);
    let mut cfg = config::load_config(&path).unwrap_or_default();
    cfg.position = Position { x, y };
    if let Err(e) = config::save_config(&path, &cfg) {
        warn!("Failed to save position: {}", e);
    }
}

#[tauri::command]
fn save_agent_preference(app_handle: tauri::AppHandle, agent: String) {
    let path = config::get_config_file(&app_handle);
    let mut cfg = config::load_config(&path).unwrap_or_default();
    cfg.agent = agent;
    if let Err(e) = config::save_config(&path, &cfg) {
        warn!("Failed to save agent preference: {}", e);
    }
}
```

#### 1.3: Wire Config into JS Bridge

- [ ] Add to `ui/index.html` or `ui/clippy-bridge.js`:

```javascript
// Restore config on load (agent, position, mute)
if (window.__TAURI__) {
    window.__TAURI__.core.invoke('get_config').then(function(config) {
        // Load preferred agent (default: 'Clippy')
        var agentName = config.agent || 'Clippy';

        clippy.load(agentName, function(agent) {
            currentAgent = agent;
            currentAgentName = agentName;

            // Restore position
            if (config.position) {
                agent.moveTo(config.position.x, config.position.y, 0);
            }

            agent.show();
            agent.speak('Hello! ' + agentName + ' is ready.');
            clippySelfTest(agent);
        }, function(err) {
            console.error('Failed to load agent ' + agentName + ', falling back to Clippy:', err);
            // Fallback to Clippy if preferred agent fails
            clippy.load('Clippy', function(agent) {
                currentAgent = agent;
                currentAgentName = 'Clippy';
                agent.show();
            });
        });
    });
}

// Save position after drag (hook into clippy.js drag end)
var originalFinishDrag = clippy.Agent.prototype._finishDrag;
clippy.Agent.prototype._finishDrag = function() {
    originalFinishDrag.call(this);
    var offset = this._el.offset();
    if (window.__TAURI__) {
        window.__TAURI__.core.invoke('save_position_cmd', {
            x: offset.left,
            y: offset.top
        });
    }
};

// Save agent preference after switch (called from switchAgent)
function saveAgentPreference(agentName) {
    if (window.__TAURI__) {
        window.__TAURI__.core.invoke('save_agent_preference', {
            agent: agentName
        });
    }
}
```

---

### Step 2: Error Resilience Audit

#### 2.1: Audit All Error Paths

- [ ] HTTP server: port in use → logged, app continues
- [ ] Sound playback: no audio device → logged, animation still plays
- [ ] Sound file missing → logged, no crash
- [ ] Tauri event emission fails → logged, HTTP still returns 200
- [ ] JSONP loading fails → logged in console, app still runs (no Clippy but no crash)
- [ ] Position file corrupt → use default position, no crash

#### 2.2: Add Graceful Degradation

- [ ] Each component operates independently — failure in one doesn't cascade
- [ ] Verify: app starts even if port 9999 is taken (no server, but tray still works)
- [ ] Verify: app starts even if no audio device (no sounds, but animations work)

---

### Step 3: Build Configuration

#### 3.1: Update tauri.conf.json for Production Build

- [ ] Configure bundler settings:

```json
{
  "bundle": {
    "active": true,
    "targets": ["msi", "nsis"],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.ico"
    ],
    "resources": [
      "../agents/Bonzi/**/*",
      "../agents/Clippy/**/*",
      "../agents/F1/**/*",
      "../agents/Genie/**/*",
      "../agents/Genius/**/*",
      "../agents/Links/**/*",
      "../agents/Merlin/**/*",
      "../agents/Peedy/**/*",
      "../agents/Rocky/**/*",
      "../agents/Rover/**/*"
    ],
    "windows": {
      "certificateThumbprint": null,
      "digestAlgorithm": "sha256",
      "timestampUrl": ""
    }
  }
}
```

#### 3.2: Create App Icons

- [ ] Generate icon set for Windows:
  - `icons/32x32.png`
  - `icons/128x128.png`
  - `icons/128x128@2x.png` (256x256)
  - `icons/icon.ico` (multi-size .ico for Windows)

#### 3.3: Build Installer

- [ ] Run `cargo tauri build`
- [ ] Verify installer created in `src-tauri/target/release/bundle/`
- [ ] Test installation and uninstallation

---

### Step 4: End-to-End Integration Testing

#### 4.1: Full Pipeline Test

- [ ] Start app from installer (not dev mode)
- [ ] Verify tray icon appears
- [ ] Test all 5 events via curl:

```bash
# Test each event
for event in complete error attention stop session-end; do
    echo "Testing: $event"
    curl -s -m 2 http://localhost:9999/$event
    echo " -> $(date)"
    sleep 10  # Wait for auto-hide between tests
done
```

#### 4.2: Simulate Real Usage

- [ ] SSH to a Linux machine with reverse tunnel
- [ ] Trigger actual Claude Code hooks
- [ ] Verify Clippy responds on Windows desktop

#### 4.3: Edge Case Testing

- [ ] Start app with no internet — works (all local)
- [ ] Start two instances — second fails gracefully on port bind
- [ ] Rapid 10+ events — app handles without crash
- [ ] Leave running overnight — no memory leak

---

### Step 5: Documentation

#### 5.1: Create README.md

- [ ] Write comprehensive README.md:

```markdown
# Clippy Awakens

A Tauri desktop app that brings Clippy back to life as a transparent overlay on Windows.
Receives events from Claude Code hooks via HTTP and responds with animations, speech
bubbles, and notification sounds.

## Prerequisites

- Windows 10 (21H2+) or Windows 11
- Rust toolchain (install via https://rustup.rs)
- MSVC Build Tools (Visual Studio Build Tools)

## Development

### Setup
cargo install tauri-cli

### Run (dev mode)
cargo tauri dev

### Build (production)
cargo tauri build

## Usage

1. Install the app from the `.msi` or `.exe` installer
2. Clippy appears in your system tray
3. Configure your SSH connection with `-R 9999:localhost:9999`
4. Claude Code hooks will trigger Clippy animations

## Event Routes

| Route | Animation | Sound |
|-------|-----------|-------|
| GET /complete | Congratulate | Pleasant chime |
| GET /error | Alert | Attention sound |
| GET /attention | GetAttention | Calendar notification |
| GET /stop | Wave | Email notification |
| GET /session-end | GoodBye | Logoff sound |

## Tray Menu

- **Left-click**: Toggle Clippy visibility
- **Show/Hide Clippy**: Toggle Clippy on screen
- **Mute Sounds**: Toggle notification sounds
- **Quit**: Exit the app

## Testing

curl http://localhost:9999/health
curl http://localhost:9999/complete
```

#### 5.2: Create CLAUDE.md

- [ ] Write CLAUDE.md for future Claude Code sessions:

```markdown
# Clippy Awakens

## Project Type
Tauri v2 desktop app (Rust + vanilla JavaScript)

## Commands
- `cargo tauri dev` — Run in development mode
- `cargo tauri build` — Build production installer
- `cd src-tauri && cargo test` — Run Rust tests
- `cd src-tauri && cargo clippy` — Lint Rust code

## Architecture
- `src-tauri/src/main.rs` — Tauri app entry, setup
- `src-tauri/src/server.rs` — axum HTTP server on :9999
- `src-tauri/src/sounds.rs` — Windows sound playback
- `src-tauri/src/events.rs` — Event type definitions
- `src-tauri/src/tray.rs` — System tray management
- `src-tauri/src/config.rs` — Config persistence (agent, mute, position)
- `src-tauri/src/agents.rs` — Agent discovery (bundled + user)
- `ui/index.html` — Tauri webview entry point
- `ui/clippy-bridge.js` — Event-to-animation mapping
- `src/` — Original clippy.js engine (DO NOT MODIFY)
- `agents/` — All 10 bundled agents (Bonzi, Clippy, F1, Genie, Genius, Links, Merlin, Peedy, Rocky, Rover)
- `build/` — Compiled clippy.js (DO NOT MODIFY)

## Key Patterns
- Rust → WebView: `app_handle.emit("event-name", payload)`
- WebView → Rust: `window.__TAURI__.core.invoke("command", args)`
- HTTP → Clippy: curl → axum → emit → JS listener → clippy.js API
- All errors fail silently — app never blocks Claude Code
```

---

## Verifiable Acceptance Criteria

**Critical Path:**

- [ ] All 5 event routes work end-to-end (curl → animation + sound)
- [ ] Position persists across app restarts via config.json
- [ ] Preferred agent persists across app restarts via config.json
- [ ] Mute state persists across app restarts via config.json
- [ ] App loads last-used agent on startup (not always Clippy)
- [ ] Windows installer builds and installs successfully
- [ ] All 10 agent directories bundled in installer
- [ ] App runs correctly from installed location (not just dev mode)
- [ ] README.md and CLAUDE.md exist and are accurate

**Quality Gates:**

- [ ] No crashes under any tested error scenario
- [ ] Memory usage stable over extended run (< 50MB)
- [ ] Installer size < 25MB (10 agents add ~13MB)
- [ ] All `cargo test` pass
- [ ] `cargo clippy` clean (no warnings)

**Integration:**

- [ ] Full pipeline: Linux hook → SSH tunnel → Windows HTTP → agent + sound
- [ ] App works identically in dev mode and installed mode
- [ ] Config persists correctly across agent switches, mute toggles, and position drags

---

## Quality Assurance

### Test Plan

#### Manual Testing

- [ ] **Full pipeline:** SSH tunnel + curl from Linux
  - Expected: Clippy animates and sound plays on Windows
  - Actual: [To be filled]

- [ ] **Position persistence:** Drag Clippy, restart app
  - Expected: Clippy appears at last drag position
  - Actual: [To be filled]

- [ ] **Installer test:** Install from .msi, run, uninstall
  - Expected: Clean install, run, and removal
  - Actual: [To be filled]

- [ ] **Error resilience:** Start with port 9999 occupied
  - Expected: App runs without HTTP server, logs error
  - Actual: [To be filled]

#### Automated Testing

```bash
cd src-tauri && cargo test
cd src-tauri && cargo clippy -- -D warnings
```

### Review Checklist

- [ ] **Code Quality:**
  - [ ] All Rust tests pass
  - [ ] No compiler/clippy warnings
  - [ ] Consistent error handling patterns

- [ ] **Documentation:**
  - [ ] README.md covers setup, build, usage
  - [ ] CLAUDE.md covers architecture and patterns

- [ ] **Security:**
  - [ ] Final audit: no hardcoded secrets, localhost-only binding
  - [ ] No sensitive data in logs

---

## Dependencies

### Upstream (Required Before Starting)

- All phases P01-P07 completed and working

### Downstream (Will Use This Phase)

- End user: final deliverable
- Future development sessions: CLAUDE.md

### External Services

- None (all local)

---

## Completion Gate

### Sign-off

- [ ] All acceptance criteria met
- [ ] All Rust tests passing
- [ ] End-to-end pipeline verified
- [ ] Installer built and tested
- [ ] README.md and CLAUDE.md complete
- [ ] Code review passed
- [ ] Phase marked DONE in plan.md
- [ ] Committed: `feat(release): phase 08 — final integration & build`

---

## Notes

### Technical Considerations

- `cargo tauri build` requires all icon sizes — missing icons will fail the build
- Windows SmartScreen will warn about unsigned apps — this is expected for personal use
- The `.msi` installer handles creating Start Menu shortcuts automatically
- Tauri's `app_data_dir` on Windows is `%APPDATA%/com.digitalmastery.clippy-awakens/`

### Known Limitations

- No auto-update mechanism (could add Tauri's updater plugin later)
- No code signing (SmartScreen warning on first run)
- Position doesn't adjust for resolution changes (if user switches monitors)
- No auto-start on boot (manual setup required)

### Future Enhancements

- Auto-update via Tauri updater plugin
- Code signing for SmartScreen suppression
- Auto-start on boot (tray menu option)
- Custom speech bubble text pools (user-editable)
- Bluetooth audio keepalive
- Event history/log viewer
- Agent marketplace / community sharing
- Drag-and-drop agent installation

---

**Previous:** [[phase-07-system-tray|Phase 07: System Tray Integration]]
