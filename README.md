# Clippy Awakens

A Tauri v2 desktop app that brings Microsoft's beloved paperclip back to life. Clippy sits as a transparent overlay on your Windows desktop, anchored to your terminal window, and reacts to Claude Code events with animations, speech bubbles, and sounds.

## Features

- **Terminal tracking** — Clippy follows your Windows Terminal window in real-time, anchored to the corner of your choice
- **Multi-monitor** — overlay spans all monitors automatically
- **Foreground-aware** — Clippy hides when you switch to another app, reappears when you return to terminal
- **Click-through** — transparent overlay doesn't interfere with your desktop
- **10 bundled agents** — Clippy, Bonzi, Merlin, Genie, and more
- **Configurable position** — anchor to any corner + pixel-level vertical offset via tray menu
- **Persistent** — Clippy stays visible between events (no more pop-in/pop-out)
- **SSH tunnel support** — works with remote Claude Code sessions via `RemoteForward`

## Quick Start

### Prerequisites

- Windows 10 (21H2+) or Windows 11
- Rust toolchain — [rustup.rs](https://rustup.rs)
- MSVC Build Tools (Visual Studio Build Tools 2022, C++ workload)
- Tauri CLI: `cargo install tauri-cli`

### Build & Install

```bash
git clone https://github.com/darraghh1/clippy-awakens.git
cd clippy-awakens
bash build.sh
```

This runs tests and produces both installers in `src-tauri/target/release/bundle/`:

| Installer | Use case |
|-----------|----------|
| `Clippy Awakens_x.x.x_x64-setup.exe` | Standard install (recommended) |
| `Clippy Awakens_x.x.x_x64_en-US.msi` | Silent/managed deployment |

The NSIS installer uses per-user mode — no admin privileges required.

### Development

```bash
bash build.sh --dev    # prepares frontend + launches dev mode
```

Or manually:

```bash
# Copy symlinked dirs for Windows (build.sh does this automatically)
rm -rf ui/build ui/agents && cp -r build ui/build && cp -r agents ui/agents

cargo tauri dev        # dev mode
cd src-tauri && cargo test   # run tests (39 tests)
cd src-tauri && cargo clippy # lint
```

## Connecting Claude Code

Clippy listens on `127.0.0.1:9999`. When Claude Code runs on a remote server, you need an SSH tunnel to forward events back to your Windows machine.

### 1. SSH Config

Add to `~/.ssh/config` on your Windows machine:

```
Host your-server
    HostName your-server.example.com
    User yourname
    RemoteForward 9999 127.0.0.1:9999
    ExitOnForwardFailure no
```

`ExitOnForwardFailure no` lets multiple SSH sessions share the tunnel — the first one claims port 9999, the rest connect normally.

### 2. Claude Code Hooks

Pre-built hooks are included in `hooks/claude-code/`. Copy them and configure:

```bash
# On your remote server
cp -r hooks/claude-code ~/.claude/hooks/clippy-awakens
```

Add to your `.claude/settings.json`:

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

### 3. Test

```bash
curl http://localhost:9999/health     # {"app":"clippy-awakens","status":"ok",...}
curl http://localhost:9999/complete   # Clippy celebrates
curl http://localhost:9999/error      # Clippy alerts
curl "http://localhost:9999/message?text=Hello+from+Linux"  # Custom speech bubble
```

## Event Routes

| Route | Animation | Sound | Description |
|-------|-----------|-------|-------------|
| `GET /complete` | Congratulate | Pleasant chime | Task completed |
| `GET /error` | Alert | Critical stop | Error occurred |
| `GET /attention` | GetAttention | Calendar notification | Input needed |
| `GET /stop` | Wave | Email notification | Process stopped |
| `GET /session-end` | GoodBye | Logoff sound | Session ended |
| `GET /message?text=...` | Random | Attention sound | Custom speech bubble |
| `GET /health` | — | — | Health check JSON |

## Tray Menu

Right-click the system tray icon for:

- **Show/Hide Agent** — toggle Clippy visibility
- **Mute Sounds** — toggle notification audio
- **Switch Agent** — Bonzi, Clippy, F1, Genie, Genius, Links, Merlin, Peedy, Rocky, Rover
- **Position** — anchor Clippy to any terminal corner (top-left/right, bottom-left/right)
- **Vertical Offset** — nudge up/down 10px at a time (handy for clearing your status line)
- **Quit** — exit the app

Left-click the tray icon to quickly toggle visibility.

## Config

Settings persist to `%APPDATA%/com.digitalmastery.clippy-awakens/config.json`:

- Preferred agent
- Mute state
- Last position
- Anchor corner
- Vertical offset

## Architecture

```
src-tauri/src/
  main.rs      — App entry, virtual screen sizing, click-through
  server.rs    — axum HTTP server on :9999
  tracker.rs   — Win32 API: terminal position + foreground polling
  events.rs    — Event type definitions
  sounds.rs    — Windows sound playback via rodio
  tray.rs      — System tray + position/offset menus
  config.rs    — JSON config persistence
  agents.rs    — Agent discovery

ui/
  index.html        — Webview: terminal tracking, anchor logic
  clippy-bridge.js  — Event-to-animation mapping, speech pools

hooks/claude-code/  — Python hooks for Claude Code integration
agents/             — 10 bundled MS Agent sprite sheets
build/              — Compiled clippy.js engine
```

## How It Works

1. Claude Code hook fires on your remote server
2. Hook script `curl`s `localhost:9999` (tunnelled via SSH back to Windows)
3. axum HTTP handler emits a Tauri event + plays a Windows system sound
4. JavaScript listener triggers a clippy.js animation + randomised speech bubble
5. `tracker.rs` polls Windows Terminal's position via Win32 `FindWindowW` every 300ms
6. Clippy follows the terminal, hides when you alt-tab away, reappears when you return

## License

MIT — see [MIT-LICENSE.txt](MIT-LICENSE.txt).

The clippy.js engine is from [smore-inc/clippy.js](https://github.com/smore-inc/clippy.js) (2012, Fireplace Inc).
Microsoft Agent characters and the Clippy brand are property of Microsoft Corporation.
