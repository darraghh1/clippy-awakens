# Clippy Awakens

## Project Type

Tauri v2 desktop app (Rust backend + vanilla JavaScript frontend).
Windows-only. Transparent always-on-top overlay that tracks the Windows Terminal window.

## Commands

- `bash build.sh` — Full production build (prepares frontend, tests, builds installers)
- `bash build.sh --dev` — Dev mode (prepares frontend, tests, launches dev server)
- `cargo tauri dev` — Dev mode (requires `ui/build/` and `ui/agents/` to exist)
- `cd src-tauri && cargo test` — Run Rust unit tests (39 tests)
- `cd src-tauri && cargo clippy` — Lint Rust code

## Build Notes

- **Windows symlinks**: Git clones `ui/agents` and `ui/build` as text files on Windows.
  `build.sh` copies the real directories into `ui/` before building.
  Do NOT commit `ui/agents/` or `ui/build/` — they're in `.gitignore`.
- **Icons**: Must be RGBA PNG, not indexed/palette. Use ImageMagick to convert if needed.
- **Installer**: NSIS `.exe` for end users, MSI for managed deployment. Both in
  `src-tauri/target/release/bundle/`.

## Architecture

### Rust Backend (src-tauri/src/)

| File | Purpose |
|------|---------|
| `main.rs` | Tauri app entry, setup, virtual screen sizing, click-through, command registration |
| `server.rs` | axum HTTP server on 127.0.0.1:9999 with event + message routes |
| `events.rs` | ClippyEvent and ClippyMessage payload types, event validation |
| `sounds.rs` | Windows sound playback via rodio (graceful no-op on Linux) |
| `tray.rs` | System tray icon, menu, TrayState (mute/visibility), position/offset submenus |
| `config.rs` | Config persistence (agent, mute, position, anchor, vertical_offset) |
| `agents.rs` | Agent discovery (10 bundled + user-installed agents) |
| `tracker.rs` | Win32 API polling — tracks Windows Terminal position + foreground state |

### Frontend (ui/)

| File | Purpose |
|------|---------|
| `index.html` | Tauri webview entry, agent loading, terminal tracking, anchor/offset logic |
| `clippy-bridge.js` | Event-to-animation mapping, Fisher-Yates speech pools, persistent visibility |
| `clippy-test.js` | Self-test suite for clippy.js API |
| `clippy-bridge-test.js` | Unit tests for bridge event mapping |

### Other Directories

| Directory | Purpose |
|-----------|---------|
| `src/` | Original clippy.js engine source (DO NOT MODIFY) |
| `build/` | Compiled clippy.js + CSS (DO NOT MODIFY) |
| `agents/` | 10 bundled MS Agent characters |
| `hooks/claude-code/` | Python hooks for Claude Code integration |

## Key Patterns

- **Rust to WebView**: `app_handle.emit("event-name", payload)` — Tauri event system
- **WebView to Rust**: `window.__TAURI__.core.invoke("command", args)` — Tauri IPC
- **HTTP to Clippy**: `curl -> axum route -> emit -> JS listener -> clippy.js API`
- **Terminal tracking**: `tracker.rs` polls `FindWindowW("CASCADIA_HOSTING_WINDOW_CLASS")` every 300ms, emits screen-absolute coordinates. JS converts to webview-relative using `window.screenX/Y`.
- **Foreground awareness**: `GetForegroundWindow()` compared to terminal HWND. Clippy hides when terminal is not the active window.
- **Multi-monitor**: `main.rs` computes virtual screen bounds from all monitors and resizes the overlay to span them all.
- **Click-through**: `set_ignore_cursor_events(true)` when `!is_decorated()` (production only). Debug mode keeps cursor events for DevTools access.
- **Config storage**: `%APPDATA%/com.digitalmastery.clippy-awakens/config.json`
- **Error handling**: All errors fail silently with logging — app never blocks Claude Code
- **Persistent agent**: ClippyBridge no longer auto-hides after events. Speech bubble dismisses after 8s but the agent stays visible.

## HTTP Routes

| Route | Event | Description |
|-------|-------|-------------|
| `GET /complete` | `clippy-event` | Task completed |
| `GET /error` | `clippy-event` | Error occurred |
| `GET /attention` | `clippy-event` | Input needed |
| `GET /stop` | `clippy-event` | Process stopped |
| `GET /session-end` | `clippy-event` | Session ended |
| `GET /message?text=...` | `clippy-message` | Custom speech bubble |
| `GET /health` | — | Health check JSON |

## Tray Menu Features

- Show/Hide Agent — toggle visibility
- Mute Sounds — toggle audio
- Switch Agent — 10 bundled characters
- Position — anchor to any corner (top-left, top-right, bottom-left, bottom-right)
- Vertical Offset — nudge up/down in 10px increments, reset
- Quit

## Testing

39 unit tests covering:
- Event type validation and serialization
- Config save/load/roundtrip (including anchor + vertical_offset)
- Sound mapping and fallback paths
- Tray state management (mute, visibility, independence)
- Agent listing (bundled + user)
- Window tracker (equality, serialization, no-panic)
