# Clippy Awakens

A Tauri desktop app that brings Clippy back to life as a transparent overlay on Windows.
Receives events from Claude Code hooks via HTTP and responds with animations, speech
bubbles, and notification sounds.

## Prerequisites

- Windows 10 (21H2+) or Windows 11
- Rust toolchain (install via https://rustup.rs)
- MSVC Build Tools (Visual Studio Build Tools 2022)

## Development

### Setup

```bash
cargo install tauri-cli
```

### Run (dev mode)

```bash
cargo tauri dev
```

### Build (production)

```bash
cargo tauri build
```

### Run tests

```bash
cd src-tauri && cargo test
```

### Lint

```bash
cd src-tauri && cargo clippy -- -D warnings
```

## Usage

1. Install the app from the `.msi` or `.exe` installer in `src-tauri/target/release/bundle/`
2. Clippy appears in your system tray
3. Configure your SSH connection with `-R 9999:localhost:9999`
4. Claude Code hooks trigger Clippy animations on your Windows desktop

## Event Routes

| Route | Animation | Sound | Description |
|-------|-----------|-------|-------------|
| `GET /complete` | Congratulate | Pleasant chime | Task completed |
| `GET /error` | Alert | Critical stop | Error occurred |
| `GET /attention` | GetAttention | Calendar notification | Input needed |
| `GET /stop` | Wave | Email notification | Process stopped |
| `GET /session-end` | GoodBye | Logoff sound | Session ended |
| `GET /message?text=...` | Random | Attention sound | Custom message |
| `GET /health` | — | — | Health check |

### Custom Messages

Send custom text through Clippy's speech bubble:

```bash
curl "http://localhost:9999/message?text=Found%20the%20bug%20-%20missing%20semicolon"
```

Clippy pops up, plays a random animation, and speaks the provided text.

## Tray Menu

- **Left-click tray icon**: Toggle Clippy visibility
- **Show/Hide Agent**: Toggle Clippy on screen
- **Mute Sounds**: Toggle notification sounds
- **Switch Agent**: Choose from 10 bundled agents
- **Quit**: Exit the app

## Bundled Agents

Bonzi, Clippy, F1, Genie, Genius, Links, Merlin, Peedy, Rocky, Rover

## Config Persistence

Settings are saved to `%APPDATA%/com.digitalmastery.clippy-awakens/config.json`:
- Preferred agent
- Mute state
- Last drag position

## Architecture

```
src-tauri/src/
  main.rs     — Tauri app entry, setup
  server.rs   — axum HTTP server on :9999
  events.rs   — Event type definitions
  sounds.rs   — Windows sound playback via rodio
  tray.rs     — System tray management
  config.rs   — Config persistence
  agents.rs   — Agent discovery

ui/
  index.html       — Tauri webview entry point
  clippy-bridge.js — Event-to-animation mapping

agents/           — 10 bundled MS Agent characters
build/            — Compiled clippy.js engine
src/              — Original clippy.js source (do not modify)
```

## Claude Code Hooks Integration

Pre-built hooks are included in `hooks/claude-code/` to connect Claude Code to Clippy automatically.

### What fires when

| Hook | Event | Clippy does... |
|------|-------|----------------|
| `on_stop.py` | Claude finishes a task | Congratulatory animation + witty remark |
| `on_notification.py` | Claude needs your input | Attention-grabbing animation |
| `on_session_end.py` | Session ends | Farewell wave |
| `on_error.py` | A tool call fails | Alert animation + snarky comment |

### Prerequisites

- Python 3.11+
- [uv](https://docs.astral.sh/uv/) (Python package runner)
- `curl` (pre-installed on most systems)
- SSH tunnel: `ssh -R 9999:localhost:9999 your-linux-host`

### Install

1. Copy the hooks directory somewhere permanent (or use it from the cloned repo):

```bash
cp -r hooks/claude-code ~/.claude/hooks/clippy-awakens
```

2. Add hooks to your `.claude/settings.json` (merge with existing hooks if you have them):

```json
{
  "hooks": {
    "Stop": [
      {
        "hooks": [{ "type": "command", "command": "uv run ~/.claude/hooks/clippy-awakens/on_stop.py", "timeout": 5000 }]
      }
    ],
    "Notification": [
      {
        "hooks": [{ "type": "command", "command": "uv run ~/.claude/hooks/clippy-awakens/on_notification.py", "timeout": 5000 }]
      }
    ],
    "SessionEnd": [
      {
        "hooks": [{ "type": "command", "command": "uv run ~/.claude/hooks/clippy-awakens/on_session_end.py", "timeout": 5000 }]
      }
    ],
    "PostToolUseFailure": [
      {
        "matcher": "",
        "hooks": [{ "type": "command", "command": "uv run ~/.claude/hooks/clippy-awakens/on_error.py", "timeout": 5000 }]
      }
    ]
  }
}
```

3. Test it:

```bash
curl http://localhost:9999/health    # should return OK
curl http://localhost:9999/complete  # Clippy should celebrate
```

### Custom messages from hooks

Use `notify.py` directly in your own hooks:

```python
from notify import notify, message

notify("complete")                           # predefined event
message("Refactored 12 files successfully")  # custom speech bubble
```

### Full settings example

See `hooks/claude-code/settings.example.json` for a complete hooks configuration template.

## Testing from Linux

```bash
# Health check
curl http://localhost:9999/health

# Test events
curl http://localhost:9999/complete
curl http://localhost:9999/error
curl http://localhost:9999/attention
curl http://localhost:9999/stop
curl http://localhost:9999/session-end

# Custom message
curl "http://localhost:9999/message?text=Hello%20from%20Linux"
```
