---
title: "Clippy Awakens — Tauri Desktop Notification Overlay"
description: "A Tauri desktop app that renders Clippy as a transparent overlay on Windows, receives events from Claude Code hooks via HTTP, plays notification sounds, and shows Clippy animations with witty speech bubbles."
status: pending
priority: P1
tags: [tauri, rust, clippy, desktop-app, notifications]
created: 2026-02-28
updated: 2026-02-28
---

# Clippy Awakens Master Plan

## Executive Summary

**The Mission:** Build a Tauri desktop app that resurrects Clippy as a transparent desktop overlay, receiving real-time events from Claude Code hooks via HTTP on port 9999 and responding with animated Clippy reactions, witty speech bubbles, and notification sounds.

**The Big Shift:** Replacing the planned PowerShell sound server with a full-featured Tauri app that wraps the existing clippy.js engine. Instead of a simple sound player, we get a visual notification system with personality — Clippy pops up, animates, speaks, and plays sounds in response to Claude Code hook events.

> [!NOTE]
> _The existing clippy.js engine (jQuery + sprite sheets + JSONP agent data) is bundled as-is inside the Tauri webview. No rewrite of clippy.js — we wrap it._

**Primary Deliverables:**

1. **Foundation:** Tauri app with transparent overlay window and bundled clippy.js engine with all 10 agents
2. **Intelligence:** HTTP event server + event-to-animation mapping with randomized witty speech bubbles
3. **Experience:** System tray with agent picker, Windows sound playback, config persistence, auto-hide/show behavior

---

## Phasing Strategy (Roadmap)

We follow a **Bottom-Up Foundation** strategy. The Tauri scaffold and transparent window come first, then the clippy.js integration, then the Rust backend services (HTTP + sound), and finally system-level polish (tray, persistence).

### Phase Constraints

- **Size:** 10-15KB max per phase document
- **Scope:** Single implementation session target
- **Dependencies:** Explicit in phase header
- **Review gate:** Code review via `code-quality-reviewer` sub-agent before marking DONE

### Phase File Naming

- Pattern: `phase-NN-descriptive-slug.md`
- Example: `phase-01-tauri-scaffolding.md`, `phase-05-http-server.md`
- No sub-phases (no 01a, 01b) — flat sequential numbering only

### Phase Table

| Phase  | Title                                                                | Group              | Focus                          | Status  |
| :----- | :------------------------------------------------------------------- | :----------------- | :----------------------------- | :------ |
| **01** | [Tauri Project Scaffolding](./phase-01-tauri-scaffolding.md)         | tauri-foundation   | Project init & basic window    | Pending |
| **02** | [Transparent Overlay Window](./phase-02-transparent-overlay.md)      | tauri-foundation   | Frameless transparent window   | Pending |
| **03** | [Clippy.js WebView Integration](./phase-03-clippy-webview.md)        | clippy-engine      | Bundle & load clippy.js        | Pending |
| **04** | [Event Animation Mapping & Speech](./phase-04-event-mapping.md)     | clippy-engine      | Animation mapping & speech     | Pending |
| **05** | [HTTP Event Server](./phase-05-http-server.md)                      | rust-services      | Rust HTTP server on :9999      | Pending |
| **06** | [Windows Sound Playback](./phase-06-sound-playback.md)              | rust-services      | Sound playback via rodio       | Pending |
| **07** | [System Tray Integration](./phase-07-system-tray.md)                | system-polish      | Tray icon & auto-hide          | Pending |
| **08** | [Final Integration & Build](./phase-08-integration-build.md)        | system-polish      | End-to-end polish & installer  | Pending |

### Group Summary

Groups define audit boundaries — connected phases are reviewed together after the group completes.

| Group            | Phases  | Description                                                        |
|------------------|---------|--------------------------------------------------------------------|
| tauri-foundation | P01-P02 | Tauri project init + transparent overlay window                    |
| clippy-engine    | P03-P04 | Clippy.js webview integration + event mapping with speech bubbles  |
| rust-services    | P05-P06 | HTTP event server + Windows sound playback                         |
| system-polish    | P07-P08 | System tray, auto-hide, position persistence, build & packaging    |

**Group ordering:** Groups are implemented sequentially. `tauri-foundation` -> `clippy-engine` -> `rust-services` -> `system-polish`.

---

## Architectural North Star

**Purpose:** Define the immutable patterns that every phase must follow.

### 1. Tauri Bridge Pattern (Rust <-> WebView)

- **Core Principle:** Rust backend owns HTTP server, sound playback, and system tray. WebView owns Clippy rendering and animations. Communication flows via Tauri's `emit`/`listen` event system.
- **Enforcement:** All cross-boundary communication uses `app_handle.emit("event-name", payload)` from Rust and `window.__TAURI__.event.listen("event-name", handler)` in JS. No direct DOM manipulation from Rust.

### 2. Existing Engine Preservation

- **Core Principle:** The clippy.js engine (jQuery + agent.js + animator.js + balloon.js + queue.js + load.js) is bundled verbatim. No refactoring, no module conversion.
- **Enforcement:** jQuery is loaded as a script tag. clippy.js is loaded as a script tag. Agent data loaded via JSONP. Original clippy.css used as-is.

### 3. Event-Driven Architecture

- **Core Principle:** Hook events flow: `curl GET :9999/{event}` -> Rust HTTP handler -> Tauri event emit -> JS listener -> Clippy animation + speech bubble. Each layer is decoupled.
- **Enforcement:** The HTTP server responds immediately (200 OK), then emits the event asynchronously. No blocking between HTTP response and animation.

### 4. Fail-Silent Operation

- **Core Principle:** The app must never block or crash Claude Code workflows. If something fails (port in use, sound not found, animation missing), it fails silently and logs internally.
- **Enforcement:** All error paths log to Tauri's built-in logging. HTTP server returns 200 even if downstream animation fails. 2-second timeout on client side (`curl -m 2`) already ensures non-blocking.

---

## Project Framework Alignment

This is a **Tauri v2 + Rust + vanilla JavaScript** project. Not Next.js/Supabase.

### Technology Stack

| Layer      | Technology                     | Purpose                                |
|------------|-------------------------------|----------------------------------------|
| Backend    | Rust + Tauri v2               | Window management, HTTP server, sounds |
| Frontend   | Vanilla JS + jQuery 1.12.4    | Clippy.js engine, animations, UI       |
| HTTP       | axum (via Tauri plugin or standalone) | Listen on 127.0.0.1:9999       |
| Sound      | rodio crate                   | Play .wav files on Windows             |
| Tray       | Tauri tray API                | System tray icon + context menu        |
| Build      | Tauri CLI + cargo             | Windows .msi/.exe installer            |

### Required Patterns

| Task                  | Pattern                                                        |
|-----------------------|----------------------------------------------------------------|
| Rust -> WebView event | `app_handle.emit("clippy-event", payload)`                     |
| WebView -> Rust cmd   | `#[tauri::command]` + `invoke("command_name", args)`           |
| HTTP routes           | axum `Router` with GET handlers returning `StatusCode::OK`     |
| Sound playback        | `rodio::Sink` with `.append(source)` for non-blocking play     |
| Agent loading         | `clippy.BASE_PATH = './agents/'; clippy.load('Clippy', cb)`   |
| Agent switching       | `switchAgent('Merlin')` — destroys current, loads new agent    |
| Config persistence    | `config.json` in `%APPDATA%/com.digitalmastery.clippy-awakens/` |
| User agents dir       | `%APPDATA%/clippy-awakens/agents/{Name}/` for custom agents    |
| Animation trigger     | `agent.play('AnimationName'); agent.speak('text')`             |

### File Structure (Target)

```
clippy/
  src-tauri/
    src/
      main.rs          — Tauri app entry, HTTP server spawn
      server.rs         — axum HTTP server on :9999
      sounds.rs         — Sound playback engine
      events.rs         — Event types and mapping
      tray.rs           — System tray with agent picker submenu
      config.rs         — Config persistence (agent, mute, position)
      agents.rs         — Agent discovery (bundled + user directory)
    Cargo.toml          — Rust dependencies
    tauri.conf.json     — Tauri window config
    icons/              — App icons
  src/                  — Existing clippy.js source (untouched)
  agents/               — All 10 bundled agents (Bonzi, Clippy, F1, Genie, Genius,
                          Links, Merlin, Peedy, Rocky, Rover) — ~13MB
  build/                — Existing clippy.js build (untouched)
  ui/
    index.html          — Tauri webview entry point (new, for Tauri)
    clippy-bridge.js    — Event bridge: Tauri events -> clippy.js API
    clippy-test.js      — Self-test for agent loading
    vendor/             — jQuery 1.12.4 minified
  index.html            — Original demo page (preserved, not used by Tauri)
```

---

## Global Decision Log (Project ADRs)

### Use Tauri v2 over Electron (ADR-G-01)

**Status:** Accepted

**Context:** Need a desktop app framework that can render a transparent overlay on Windows. Electron works but produces 100MB+ binaries. Tauri uses the system WebView2 (pre-installed on Windows 10+) and produces ~5MB binaries.

**Decision:** Use Tauri v2 with Rust backend.

**Consequences:**
- **Positive:** Tiny binary, native performance, Rust safety, WebView2 is always available on modern Windows
- **Negative:** Smaller ecosystem than Electron, Rust learning curve
- **Neutral:** WebView2 rendering is identical to Edge/Chrome

### Bundle jQuery Instead of Rewriting (ADR-G-02)

**Status:** Accepted

**Context:** clippy.js depends heavily on jQuery ($.Deferred, $.proxy, $.when, DOM manipulation). Rewriting to vanilla JS would be a multi-week effort with high regression risk.

**Decision:** Bundle jQuery 1.12.4 minified (~85KB) in the Tauri webview.

**Consequences:**
- **Positive:** Zero risk of breaking existing clippy.js animations, immediate reuse
- **Negative:** Extra 85KB in bundle (negligible for desktop app)
- **Neutral:** jQuery runs fine in WebView2

### axum for HTTP Server (ADR-G-03)

**Status:** Accepted

**Context:** Need a lightweight HTTP server running inside the Tauri app process. Options: tiny_http, warp, axum, hyper directly.

**Decision:** Use axum — it's the most popular Rust HTTP framework, well-documented, async-native, and lightweight enough for 5 simple routes.

**Consequences:**
- **Positive:** Familiar API, good tokio integration (Tauri already uses tokio), excellent docs
- **Negative:** Slightly heavier than tiny_http
- **Neutral:** All routes are simple GET handlers — any framework would work

### rodio for Sound Playback (ADR-G-04)

**Status:** Accepted

**Context:** Need to play .wav files on Windows. Options: rodio crate, winapi directly, windows-rs crate with MediaPlayer.

**Decision:** Use rodio — cross-platform audio playback crate that handles .wav natively.

**Consequences:**
- **Positive:** Simple API, handles audio device selection, non-blocking playback
- **Negative:** Adds a dependency (~2MB compile impact)
- **Neutral:** Could switch to windows-rs later if needed for Bluetooth audio quirks

### Bundle All 10 Agents with Extensible Loading (ADR-G-05)

**Status:** Accepted

**Context:** The original plan only loaded Clippy. The user wants all 10 available agents (Bonzi, Clippy, F1, Genie, Genius, Links, Merlin, Peedy, Rocky, Rover) bundled, with the ability to add custom agents from a user directory.

**Decision:** Bundle all 10 agent directories (~13MB total) in the Tauri app resources. Support extensible loading from `%APPDATA%/clippy-awakens/agents/` for user-added agents. Provide `switchAgent()` in JS and agent picker submenu in the system tray. Persist preferred agent to `config.json`.

**Consequences:**
- **Positive:** Full agent selection out of the box, extensible for custom agents, preference persists
- **Negative:** Installer size increases by ~13MB (acceptable for desktop app)
- **Neutral:** Animation fallback system (Phase 04) handles agents with missing animations

---

## Security Requirements

This is a local-only desktop app with no network exposure beyond localhost. Security surface is minimal.

### Network Security

- HTTP server binds to `127.0.0.1` only — never `0.0.0.0`
- No authentication needed (localhost-only, same machine)
- No sensitive data processed or stored

### Input Validation

- HTTP routes only accept known event names (`complete`, `error`, `attention`, `stop`, `session-end`)
- Unknown paths return 404, no error details
- No user-supplied content rendered in webview (speech bubbles are pre-defined pools)

### File System

- Sound files bundled in app resources, not loaded from arbitrary paths
- Config file writes only to Tauri's `app_data_dir` (safe, sandboxed)
- User agents directory at `%APPDATA%/clippy-awakens/agents/` — read-only scanning, no arbitrary path access
- Agent data loaded only from known bundled paths or user agents dir

---

## Implementation Standards

### Test Strategy

- **Rust unit tests:** `#[cfg(test)]` modules for HTTP route handlers, event mapping, sound path resolution
- **Integration tests:** Manual `curl` testing against running app (matches existing hook infrastructure)
- **Visual verification:** Clippy animations render correctly in transparent window
- **No E2E framework:** Desktop app testing is primarily manual for UI; automated for Rust logic

### Documentation Standard

1. `README.md` — Setup instructions, build steps, usage guide
2. `CLAUDE.md` — Project commands and patterns for future Claude Code sessions
3. Inline Rust doc comments on public functions

---

## Success Metrics & Quality Gates

### Project Success Metrics

- HTTP server responds to `curl` within 50ms
- Clippy animation starts within 200ms of event receipt
- Sound plays within 500ms of event receipt
- App binary < 10MB (Tauri typical)
- Memory usage < 50MB idle

### Global Quality Gates (Pre-Release)

- [ ] All 5 event routes (`/complete`, `/error`, `/attention`, `/stop`, `/session-end`) work end-to-end
- [ ] Clippy renders correctly as transparent overlay (no black box, no artifacts)
- [ ] System tray icon shows/hides Clippy correctly
- [ ] Sound plays for each event type
- [ ] All 10 bundled agents can be loaded and animated
- [ ] Agent picker in system tray switches between agents
- [ ] Preferred agent persists across restarts via config.json
- [ ] App starts on Windows boot (optional, configurable)
- [ ] `cargo test` passes all Rust unit tests
- [ ] No panics or crashes under normal operation
- [ ] Existing SSH tunnel + curl infrastructure works without modification

---

## Resources & References

- **Clippy.js Source:** `/home/darragh/Projects/clippy/src/` (agent.js, animator.js, balloon.js, queue.js, load.js)
- **Agent Data:** `/home/darragh/Projects/clippy/agents/` — All 10 agents (Bonzi, Clippy, F1, Genie, Genius, Links, Merlin, Peedy, Rocky, Rover)
- **Sound Server Spec (replaced):** `/home/darragh/Projects/DigitalMastery/docs/windows-sound-server-prompt.md`
- **Tauri v2 Docs:** https://v2.tauri.app/
- **axum Docs:** https://docs.rs/axum/latest/axum/
- **rodio Docs:** https://docs.rs/rodio/latest/rodio/
- **Clippy Animations Available:** Alert, CheckingSomething, Congratulate, Explain, GetAttention, GetTechy, GoodBye, Greeting, Searching, Thinking, Wave, Writing (+ idles, looks, gestures). Note: Animation availability varies across agents — Rover has only 3 of 7 key animations.
- **Hook Events:** complete, error, attention, stop, session-end (via `curl -s -m 2 http://localhost:9999/{event}`)

---

**Next:** [[phase-01-tauri-scaffolding|Phase 01: Tauri Project Scaffolding]]
