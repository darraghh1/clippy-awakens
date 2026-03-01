---
title: "Phase 05: HTTP Event Server"
description: "Create an axum HTTP server in Rust that listens on 127.0.0.1:9999, accepts GET requests for hook events, and emits Tauri events to the webview to trigger Clippy animations."
skill: none
status: pending
group: "rust-services"
dependencies: ["phase-03"]
tags: [phase, rust, axum, http, server, events]
created: 2026-02-28
updated: 2026-02-28
---

# Phase 05: HTTP Event Server

**Context:** [Master Plan](./plan.md) | **Dependencies:** P03 | **Status:** Pending

---

## Overview

Create an axum HTTP server that runs inside the Tauri application process, listening on `127.0.0.1:9999`. It accepts GET requests matching the existing hook event paths (`/complete`, `/error`, `/attention`, `/stop`, `/session-end`) and emits Tauri events to the webview. This is the bridge between the Linux-side `curl` calls and the Clippy animation system.

**Goal:** `curl -s http://localhost:9999/complete` triggers a Tauri event that the JavaScript bridge (Phase 04) picks up to animate Clippy.

---

## Context & Workflow

### How This Phase Fits Into the Project

- **UI Layer:** No changes — Phase 04's JS bridge already listens for `clippy-event`
  - This phase emits the events the bridge consumes

- **Server Layer:** Creates `src-tauri/src/server.rs` — axum HTTP server
  - Spawns on a background tokio task during Tauri setup
  - Routes: GET `/complete`, `/error`, `/attention`, `/stop`, `/session-end`
  - Each route emits `clippy-event` via Tauri's `AppHandle`

- **Database Layer:** N/A

- **Integrations:** Receives `curl` requests from Linux hooks via SSH reverse tunnel

### User Workflow

**Trigger:** Claude Code hook fires on Linux, sends `curl -s -m 2 http://localhost:9999/complete`.

**Steps:**
1. SSH reverse tunnel forwards port 9999 from Linux to Windows localhost
2. `curl GET http://localhost:9999/complete` hits the axum server
3. axum handler emits `clippy-event` with `{ "type": "complete" }` payload
4. WebView JS bridge receives event (Phase 04)
5. Clippy plays Congratulate animation + speaks witty remark
6. axum returns HTTP 200 "OK" immediately (non-blocking)

**Success Outcome:** `curl` gets a 200 response, and Clippy animates within ~200ms.

### Problem Being Solved

**Pain Point:** The Claude Code hooks already send `curl` to `localhost:9999`. Something needs to listen. Previously this was going to be a PowerShell script that only played sounds. Now it's a full Tauri app that also animates Clippy.
**Alternative Approach:** Without this HTTP server, no events reach Clippy. The SSH tunnel and hook infrastructure is already in place — we just need the listener.

### Integration Points

**Upstream Dependencies:**
- Phase 01: Tauri project structure, tokio dependency
- Phase 03: Tauri app running with webview

**Downstream Consumers:**
- Phase 04: JS bridge receives the emitted events
- Phase 06: Sound playback triggered alongside event emission

**Data Flow:**
```
Linux: curl -s -m 2 http://localhost:9999/complete
  → SSH tunnel (-R 9999:localhost:9999)
  → Windows: axum GET /complete handler
  → app_handle.emit("clippy-event", { "type": "complete" })
  → WebView: ClippyBridge.handleEvent("complete")
  → Clippy: play Congratulate + speak remark
```

---

## Prerequisites & Clarifications

### Questions for User

1. **Port Conflict:** What if port 9999 is already in use (e.g., old PowerShell server)?
   - **Context:** Only one process can bind a port. If the old sound server is running, axum will fail to bind.
   - **Assumptions if unanswered:** Log a clear error message. Try binding once — if it fails, log the error and continue without the HTTP server (Clippy still works, just no events).
   - **Impact:** User needs to stop the old server before starting Clippy Awakens

2. **Response Format:** Should the HTTP response include any data, or just 200 OK?
   - **Context:** The `curl` calls use `-s` (silent) and discard output. Response body doesn't matter.
   - **Assumptions if unanswered:** Return plain text "OK" with 200 status. Log the event to console.
   - **Impact:** None — response is discarded by caller

3. **Additional Routes:** Should there be a health/status endpoint?
   - **Context:** Could be useful for debugging (e.g., `GET /health` returns server info).
   - **Assumptions if unanswered:** Add `GET /health` that returns `{ "status": "ok", "app": "clippy-awakens" }` — useful for verifying the server is running.
   - **Impact:** Minimal — one extra route

### Validation Checklist

- [ ] Phase 03 completed — Tauri app running
- [ ] tokio and axum in Cargo.toml (added in Phase 01)
- [ ] Port 9999 is available on Windows machine

---

## Requirements

### Functional

- HTTP server listens on 127.0.0.1:9999
- GET `/complete` — emits clippy-event type "complete", returns 200
- GET `/error` — emits clippy-event type "error", returns 200
- GET `/attention` — emits clippy-event type "attention", returns 200
- GET `/stop` — emits clippy-event type "stop", returns 200
- GET `/session-end` — emits clippy-event type "session-end", returns 200
- GET `/health` — returns JSON status, no event emitted
- Unknown paths return 404
- Server starts automatically with the Tauri app
- Server stops when the Tauri app closes

### Technical

- axum `Router` with GET route handlers
- Server runs on a tokio background task (`tokio::spawn`)
- `AppHandle` shared via axum state for event emission
- Bind to `127.0.0.1:9999` (localhost only — security requirement)
- Non-blocking: HTTP response returns before animation completes
- Graceful error handling: port-in-use logs error, doesn't crash app

---

## Decision Log

### axum State for AppHandle Sharing (ADR-05-01)

**Date:** 2026-02-28
**Status:** Accepted

**Context:** axum route handlers need access to Tauri's `AppHandle` to emit events. axum supports shared state via `Extension` or `State` extractors.

**Decision:** Use axum's `State` extractor with `AppHandle` wrapped in `Arc`.

**Consequences:**
- **Positive:** Clean, idiomatic axum code. Type-safe state access.
- **Negative:** Requires cloning `AppHandle` (cheap — it's reference-counted internally).
- **Neutral:** Standard pattern for axum + Tauri integration.

**Alternatives Considered:**
1. Global static: Requires `unsafe` or `OnceCell` — less idiomatic
2. axum `Extension`: Works but `State` is preferred in axum 0.7+

---

## Implementation Steps

### Step 0: Test Definition (TDD)

**Purpose:** Define Rust tests for HTTP server logic

#### 0.1: Rust Unit Tests

- [ ] Create `src-tauri/src/server.rs` with test module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_from_path() {
        assert_eq!(event_type_from_path("complete"), Some("complete"));
        assert_eq!(event_type_from_path("error"), Some("error"));
        assert_eq!(event_type_from_path("attention"), Some("attention"));
        assert_eq!(event_type_from_path("stop"), Some("stop"));
        assert_eq!(event_type_from_path("session-end"), Some("session-end"));
        assert_eq!(event_type_from_path("unknown"), None);
        assert_eq!(event_type_from_path(""), None);
    }

    #[test]
    fn test_valid_event_types() {
        let valid = ["complete", "error", "attention", "stop", "session-end"];
        for event_type in &valid {
            assert!(
                is_valid_event(event_type),
                "{} should be valid",
                event_type
            );
        }
    }

    #[test]
    fn test_invalid_event_types() {
        let invalid = ["", "unknown", "hack", "../etc/passwd", "complete/extra"];
        for event_type in &invalid {
            assert!(
                !is_valid_event(event_type),
                "{} should be invalid",
                event_type
            );
        }
    }
}
```

#### 0.2: Run Tests

- [ ] `cd src-tauri && cargo test` — tests should fail initially (functions not yet implemented)

---

### Step 1: Create Event Types Module

#### 1.1: Create src-tauri/src/events.rs

- [ ] Define event types and validation:

```rust
use serde::Serialize;

/// Valid event types that map to Claude Code hook events
const VALID_EVENTS: &[&str] = &[
    "complete",
    "error",
    "attention",
    "stop",
    "session-end",
];

/// Payload emitted to the webview
#[derive(Debug, Clone, Serialize)]
pub struct ClippyEvent {
    #[serde(rename = "type")]
    pub event_type: String,
}

/// Check if an event type string is valid
pub fn is_valid_event(event_type: &str) -> bool {
    VALID_EVENTS.contains(&event_type)
}

/// Extract event type from URL path segment, returns None if invalid
pub fn event_type_from_path(path: &str) -> Option<&str> {
    let path = path.trim_matches('/');
    if is_valid_event(path) {
        Some(path)
    } else {
        None
    }
}
```

---

### Step 2: Create HTTP Server Module

#### 2.1: Create src-tauri/src/server.rs

- [ ] Implement axum HTTP server:

```rust
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use std::sync::Arc;
use tauri::AppHandle;
use log::{info, warn, error};

use crate::events::{ClippyEvent, is_valid_event};

/// Shared state for axum handlers
#[derive(Clone)]
pub struct AppState {
    pub app_handle: Arc<AppHandle>,
}

/// Start the HTTP server on 127.0.0.1:9999
pub async fn start_server(app_handle: AppHandle) {
    let state = AppState {
        app_handle: Arc::new(app_handle),
    };

    let app = Router::new()
        .route("/complete", get(handle_complete))
        .route("/error", get(handle_error))
        .route("/attention", get(handle_attention))
        .route("/stop", get(handle_stop))
        .route("/session-end", get(handle_session_end))
        .route("/health", get(handle_health))
        .with_state(state);

    let addr = "127.0.0.1:9999";
    info!("Clippy HTTP server starting on {}", addr);

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            error!("Failed to bind to {}: {}. Is another instance running?", addr, e);
            return;
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        error!("HTTP server error: {}", e);
    }
}

/// Emit a clippy event to the webview
fn emit_event(state: &AppState, event_type: &str) {
    let payload = ClippyEvent {
        event_type: event_type.to_string(),
    };
    info!("Event received: {}", event_type);
    if let Err(e) = state.app_handle.emit("clippy-event", &payload) {
        warn!("Failed to emit clippy-event: {}", e);
    }
}

async fn handle_complete(State(state): State<AppState>) -> impl IntoResponse {
    emit_event(&state, "complete");
    (StatusCode::OK, "OK")
}

async fn handle_error(State(state): State<AppState>) -> impl IntoResponse {
    emit_event(&state, "error");
    (StatusCode::OK, "OK")
}

async fn handle_attention(State(state): State<AppState>) -> impl IntoResponse {
    emit_event(&state, "attention");
    (StatusCode::OK, "OK")
}

async fn handle_stop(State(state): State<AppState>) -> impl IntoResponse {
    emit_event(&state, "stop");
    (StatusCode::OK, "OK")
}

async fn handle_session_end(State(state): State<AppState>) -> impl IntoResponse {
    emit_event(&state, "session-end");
    (StatusCode::OK, "OK")
}

async fn handle_health() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({
        "status": "ok",
        "app": "clippy-awakens",
        "version": env!("CARGO_PKG_VERSION")
    })))
}
```

---

### Step 3: Integrate Server with Tauri Main

#### 3.1: Update main.rs

- [ ] Modify `src-tauri/src/main.rs` to spawn the HTTP server:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod events;
mod server;

fn main() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let handle = app.handle().clone();
            // Spawn HTTP server on background task
            tauri::async_runtime::spawn(async move {
                server::start_server(handle).await;
            });
            log::info!("Clippy Awakens started");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

### Step 4: Test with curl

#### 4.1: Manual Integration Test

- [ ] Start the Tauri app with `cargo tauri dev`
- [ ] In a separate terminal, test each route:

```bash
curl -s http://localhost:9999/health
# Expected: {"status":"ok","app":"clippy-awakens","version":"0.1.0"}

curl -s http://localhost:9999/complete
# Expected: "OK" + Clippy plays Congratulate animation

curl -s http://localhost:9999/error
# Expected: "OK" + Clippy plays Alert animation

curl -s http://localhost:9999/attention
# Expected: "OK" + Clippy plays GetAttention animation

curl -s http://localhost:9999/stop
# Expected: "OK" + Clippy plays Wave animation

curl -s http://localhost:9999/session-end
# Expected: "OK" + Clippy plays GoodBye animation

curl -s http://localhost:9999/unknown
# Expected: 404 Not Found

# Test with timeout (matches hook behavior)
curl -s -m 2 http://localhost:9999/complete
# Expected: "OK" within 2 seconds
```

#### 4.2: Rapid Fire Test

- [ ] Send multiple events quickly:

```bash
for event in complete error attention stop session-end; do
    curl -s http://localhost:9999/$event &
done
wait
```

- [ ] Verify: all events received, Clippy queues animations, no crashes

---

## Verifiable Acceptance Criteria

**Critical Path:**

- [ ] Server binds to 127.0.0.1:9999 on app startup
- [ ] All 5 event routes return HTTP 200
- [ ] Each event route emits `clippy-event` to webview
- [ ] Clippy animations trigger from curl requests
- [ ] `/health` returns JSON status
- [ ] Unknown paths return 404
- [ ] Server stops when app closes

**Quality Gates:**

- [ ] Response time < 50ms for event routes
- [ ] No panics or crashes under rapid requests
- [ ] Port-in-use error is logged clearly, doesn't crash app

**Integration:**

- [ ] Works with existing `curl -s -m 2 http://localhost:9999/{event}` pattern
- [ ] Phase 06 can add sound playback triggered from the same handlers

---

## Quality Assurance

### Test Plan

#### Manual Testing

- [ ] **Health check:** `curl http://localhost:9999/health`
  - Expected: JSON with status "ok"
  - Actual: [To be filled]

- [ ] **Complete event:** `curl http://localhost:9999/complete`
  - Expected: "OK" response, Clippy animates
  - Actual: [To be filled]

- [ ] **404 test:** `curl -I http://localhost:9999/invalid`
  - Expected: 404 status
  - Actual: [To be filled]

- [ ] **Rapid fire:** 5 events in quick succession
  - Expected: All received, Clippy queues them
  - Actual: [To be filled]

#### Automated Testing

```bash
cd src-tauri && cargo test
```

#### Performance Testing

- [ ] **Response time:** Target: < 50ms, Actual: [To be measured]
- [ ] **Concurrent requests:** Target: handles 10 concurrent, Actual: [To be measured]

### Review Checklist

- [ ] **Code Quality:**
  - [ ] `cargo test` passes
  - [ ] `cargo clippy` clean (no warnings)
  - [ ] Error handling on port bind failure
  - [ ] Logging on all event routes

- [ ] **Security:**
  - [ ] Binds to 127.0.0.1, NOT 0.0.0.0
  - [ ] No sensitive data in responses
  - [ ] Input validation on paths (only known events accepted)

---

## Dependencies

### Upstream (Required Before Starting)

- Phase 01: Cargo.toml with axum and tokio dependencies
- Phase 03: Tauri app running with webview

### Downstream (Will Use This Phase)

- Phase 04: JS bridge consumes the emitted events
- Phase 06: Sound playback may be triggered from these handlers

### External Services

- None (localhost only)

---

## Completion Gate

### Sign-off

- [ ] All acceptance criteria met
- [ ] All Rust tests passing (`cargo test`)
- [ ] All 5 curl tests return 200 and trigger Clippy
- [ ] Code review passed
- [ ] Phase marked DONE in plan.md
- [ ] Committed: `feat(server): phase 05 — HTTP event server on :9999`

---

## Notes

### Technical Considerations

- axum runs on tokio, which Tauri already uses — no runtime conflict
- `AppHandle` is cheap to clone (Arc internally) — safe to share with axum state
- The server runs as a background task — if it crashes, the app continues without events
- `serde_json` is already a dependency for Tauri — no extra bloat

### Known Limitations

- No HTTPS — localhost only, unnecessary
- No authentication — localhost only, unnecessary
- No rate limiting — trusted local clients only
- Single port (9999) — not configurable in MVP (could be added later)

### Future Enhancements

- Configurable port (via settings file or CLI arg)
- WebSocket upgrade for bidirectional communication
- Event logging/history for debugging

---

**Previous:** [[phase-04-event-mapping|Phase 04: Event Animation Mapping & Speech Bubbles]]
**Next:** [[phase-06-sound-playback|Phase 06: Windows Sound Playback]]
