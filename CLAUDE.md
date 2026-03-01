# Clippy Awakens

## Project Type

Tauri v2 desktop app (Rust backend + vanilla JavaScript frontend).
Built on Linux, targeting Windows.

## Commands

- `cargo tauri dev` — Run in development mode
- `cargo tauri build` — Build production installer (.msi/.exe)
- `cd src-tauri && cargo test` — Run Rust tests
- `cd src-tauri && cargo clippy` — Lint Rust code

## Architecture

### Rust Backend (src-tauri/src/)

| File | Purpose |
|------|---------|
| `main.rs` | Tauri app entry, setup, command registration |
| `server.rs` | axum HTTP server on 127.0.0.1:9999 with event + message routes |
| `events.rs` | ClippyEvent and ClippyMessage payload types, event validation |
| `sounds.rs` | Windows sound playback via rodio (graceful no-op on Linux) |
| `tray.rs` | System tray icon, menu, TrayState (mute/visibility) |
| `config.rs` | Config persistence (agent, mute, position) via JSON in app data dir |
| `agents.rs` | Agent discovery (10 bundled + user-installed agents) |

### Frontend (ui/)

| File | Purpose |
|------|---------|
| `index.html` | Tauri webview entry, agent loading, config restore, event listeners |
| `clippy-bridge.js` | Event-to-animation mapping, Fisher-Yates speech pools, custom messages |
| `clippy-test.js` | Self-test suite for clippy.js API |
| `clippy-bridge-test.js` | Unit tests for bridge event mapping |

### Other Directories

| Directory | Purpose |
|-----------|---------|
| `src/` | Original clippy.js engine source (DO NOT MODIFY) |
| `build/` | Compiled clippy.js + CSS (DO NOT MODIFY) |
| `agents/` | 10 bundled MS Agent characters (Bonzi, Clippy, F1, Genie, Genius, Links, Merlin, Peedy, Rocky, Rover) |

## Key Patterns

- **Rust to WebView**: `app_handle.emit("event-name", payload)` — Tauri event system
- **WebView to Rust**: `window.__TAURI__.core.invoke("command", args)` — Tauri IPC
- **HTTP to Clippy**: `curl -> axum route -> emit -> JS listener -> clippy.js API`
- **Custom messages**: `GET /message?text=URL-encoded -> emit("clippy-message") -> agent.speak(text)`
- **Config storage**: `%APPDATA%/com.digitalmastery.clippy-awakens/config.json`
- **Error handling**: All errors fail silently with logging — app never blocks Claude Code
- **Sound playback**: Spawned on separate thread, graceful no-op when no audio device

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

## Testing

Run `cd src-tauri && cargo test` to execute all unit tests covering:
- Event type validation
- Config save/load/roundtrip
- Sound mapping
- Tray state management
- Agent listing
