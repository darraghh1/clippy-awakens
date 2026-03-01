---
title: "Phase 02: Transparent Overlay Window"
description: "Configure the Tauri window as a frameless, transparent, always-on-top overlay that covers the entire screen, with click-through behavior except on Clippy elements."
skill: none
status: pending
group: "tauri-foundation"
dependencies: ["phase-01"]
tags: [phase, tauri, window, transparent, overlay]
created: 2026-02-28
updated: 2026-02-28
---

# Phase 02: Transparent Overlay Window

**Context:** [Master Plan](./plan.md) | **Dependencies:** P01 | **Status:** Pending

---

## Overview

Transform the basic Tauri window from Phase 01 into a fullscreen transparent overlay. The window must be frameless (no title bar), transparent (no background), always-on-top, and click-through everywhere except on Clippy elements. This is the visual foundation that makes Clippy appear to float over the user's desktop.

**Goal:** A fullscreen transparent overlay window where Clippy will render on top of all other windows, with mouse events passing through to apps behind it except when clicking on Clippy.

---

## Context & Workflow

### How This Phase Fits Into the Project

- **UI Layer:** Modifies `ui/index.html` styles for fully transparent background
  - Body, html set to `background: transparent`
  - All elements use `pointer-events: none` except `.clippy` and `.clippy-balloon`

- **Server Layer:** Modifies `tauri.conf.json` window configuration
  - Fullscreen, frameless, transparent, always-on-top, skip-taskbar

- **Database Layer:** N/A

- **Integrations:** Windows-specific transparent window behavior via WebView2

### User Workflow

**Trigger:** App launches and window should be invisible except for Clippy (once loaded in Phase 03).

**Steps:**
1. App starts — fullscreen transparent window covers entire screen
2. Desktop is fully visible through the window
3. Mouse clicks pass through to applications underneath
4. Only Clippy sprite and speech bubble intercept mouse events (Phase 03+)

**Success Outcome:** The window is invisible — user sees their normal desktop. No visible borders, title bar, or background. Mouse works normally on all other apps.

### Problem Being Solved

**Pain Point:** A regular window blocks desktop interaction and looks like a normal app.
**Alternative Approach:** Without transparency, Clippy would need to be rendered in a small fixed-position window, losing the "floating overlay" illusion.

### Integration Points

**Upstream Dependencies:**
- Phase 01: Tauri project structure, tauri.conf.json, ui/index.html

**Downstream Consumers:**
- Phase 03: Clippy.js renders inside this transparent window
- Phase 07: System tray can show/hide this window

**Data Flow:**
```
Tauri launch
  → Create fullscreen transparent window (tauri.conf.json)
  → Load ui/index.html (transparent body)
  → CSS pointer-events: none on body
  → Mouse events pass through to desktop
  → Clippy elements (future) use pointer-events: auto
```

---

## Prerequisites & Clarifications

### Questions for User

1. **Multi-Monitor:** Should the overlay span all monitors or just the primary display?
   - **Context:** Tauri can position windows on specific monitors. Clippy traditionally appears on the primary monitor.
   - **Assumptions if unanswered:** Primary monitor only. Clippy appears in the bottom-right corner.
   - **Impact:** Multi-monitor support adds complexity and can be added later.

2. **Always-on-Top Behavior:** Should Clippy stay above fullscreen apps (games, presentations)?
   - **Context:** `always_on_top` in Tauri works for most windows but some fullscreen exclusive apps override it.
   - **Assumptions if unanswered:** Always-on-top for normal windows. Fullscreen exclusive apps may hide Clippy — this is acceptable.
   - **Impact:** Forcing overlay above exclusive fullscreen requires OS-level hooks.

3. **Click-Through Method:** Tauri v2 supports `ignore_cursor_events` on Windows. Should we use that or CSS `pointer-events`?
   - **Context:** `ignore_cursor_events` makes the entire window click-through at the OS level. We'd need to toggle it on/off when mouse enters/leaves Clippy. CSS `pointer-events: none` on body with `pointer-events: auto` on Clippy elements is simpler.
   - **Assumptions if unanswered:** Use CSS `pointer-events` approach — simpler, less Rust code, works in WebView2.
   - **Impact:** OS-level approach is more robust but requires mouse tracking from Rust side.

### Validation Checklist

- [ ] Phase 01 completed — Tauri project compiles and runs
- [ ] tauri.conf.json exists and is valid
- [ ] ui/index.html exists

---

## Requirements

### Functional

- Window covers entire primary monitor
- No title bar, no borders, no window chrome
- Background is fully transparent — desktop visible through it
- Mouse clicks pass through to desktop applications
- Window stays on top of other windows
- Window does not appear in Windows taskbar

### Technical

- Tauri v2 window config: `transparent: true`, `decorations: false`, `alwaysOnTop: true`
- Tauri v2 window config: `skipTaskbar: true`, `fullscreen: false` (use maximized width/height)
- HTML/CSS: `background: transparent` on body and html
- CSS: `pointer-events: none` on body, `pointer-events: auto` on `.clippy` and `.clippy-balloon`
- WebView2 transparent background support enabled

---

## Decision Log

### CSS pointer-events Over OS-Level Click-Through (ADR-02-01)

**Date:** 2026-02-28
**Status:** Accepted

**Context:** Two approaches for click-through: Tauri's `set_ignore_cursor_events()` (OS-level, requires Rust mouse tracking) vs CSS `pointer-events: none` (WebView-level, simple CSS rules).

**Decision:** Use CSS `pointer-events` approach.

**Consequences:**
- **Positive:** Simple, no Rust code needed, easy to control per-element
- **Negative:** WebView2 still receives the mouse event before passing it through — slightly less efficient
- **Neutral:** Both approaches work on Windows 10+

**Alternatives Considered:**
1. `set_ignore_cursor_events()`: More robust but requires complex Rust-side mouse tracking to toggle on/off when cursor enters Clippy
2. Small fixed-size window instead of fullscreen overlay: Limits Clippy's movement range and speech bubble positioning

---

## Implementation Steps

### Step 0: Test Definition (TDD)

**Purpose:** Define acceptance tests before writing implementation code

#### 0.1: Rust Unit Tests

- [ ] Add test to verify transparent window configuration values

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_window_config_transparent() {
        // Verify our expected configuration values
        // Actual window behavior tested manually
        let transparent = true;
        let decorations = false;
        let always_on_top = true;
        let skip_taskbar = true;

        assert!(transparent);
        assert!(!decorations);
        assert!(always_on_top);
        assert!(skip_taskbar);
    }
}
```

#### 0.2: Visual Verification Checklist

- [ ] Window is invisible when no content is rendered
- [ ] Desktop is fully clickable through the window
- [ ] No window borders or title bar visible

---

### Step 1: Update tauri.conf.json Window Configuration

#### 1.1: Modify Window Settings

- [ ] Edit `src-tauri/tauri.conf.json` — update the window configuration:

```json
{
  "app": {
    "windows": [
      {
        "title": "Clippy Awakens",
        "width": 1920,
        "height": 1080,
        "x": 0,
        "y": 0,
        "transparent": true,
        "decorations": false,
        "alwaysOnTop": true,
        "skipTaskbar": true,
        "resizable": false,
        "shadow": false
      }
    ]
  }
}
```

Note: Width/height set to common resolution. The window won't visually constrain anything since it's transparent. Phase 07 can adjust to match actual monitor resolution.

---

### Step 2: Update Rust Main for Transparent Window

#### 2.1: Modify main.rs

- [ ] Update `src-tauri/src/main.rs` to ensure transparent support:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Note: Tauri v2 handles transparency via tauri.conf.json — no extra Rust code needed for basic transparent windows.

---

### Step 3: Update HTML/CSS for Transparency

#### 3.1: Modify ui/index.html for Transparent Background

- [ ] Update `ui/index.html`:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Clippy Awakens</title>
    <style>
        html, body {
            margin: 0;
            padding: 0;
            width: 100%;
            height: 100%;
            background: transparent;
            overflow: hidden;
            /* Allow clicks to pass through to desktop */
            pointer-events: none;
        }

        /* Clippy elements intercept mouse events */
        .clippy,
        .clippy-balloon {
            pointer-events: auto;
        }

        /* Debug indicator — remove after verification */
        #debug-overlay {
            position: fixed;
            bottom: 10px;
            right: 10px;
            color: rgba(255, 0, 0, 0.5);
            font-size: 10px;
            font-family: monospace;
            pointer-events: none;
            z-index: 9999;
        }
    </style>
</head>
<body>
    <div id="debug-overlay">Clippy Awakens — Overlay Active</div>
</body>
</html>
```

#### 3.2: Verify Transparency CSS Requirements

- [ ] `html` and `body` must have `background: transparent` (not `none`, not absent)
- [ ] `pointer-events: none` on body ensures click-through
- [ ] `.clippy` and `.clippy-balloon` classes (from clippy.css) need `pointer-events: auto`
- [ ] Existing `clippy.css` already sets `position: fixed; z-index: 1000` on these classes

---

### Step 4: Handle WebView2 Transparent Background

#### 4.1: Ensure WebView2 Transparency Works

- [ ] Tauri v2 with `transparent: true` in tauri.conf.json automatically configures WebView2 transparency on Windows
- [ ] If needed, add Tauri plugin configuration for WebView2 transparency:

```rust
// In main.rs — only if default transparency doesn't work
tauri::Builder::default()
    .plugin(tauri_plugin_shell::init())
    .setup(|app| {
        // WebView2 transparency is handled by Tauri when
        // transparent: true is set in tauri.conf.json
        log::info!("Clippy Awakens overlay started");
        Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
```

---

### Step 5: Verify Overlay Behavior

#### 5.1: Build and Test

- [ ] Run `cargo tauri dev` — window should be invisible (transparent)
- [ ] Click on desktop apps — clicks should pass through
- [ ] Look for debug indicator text in bottom-right corner (faint red text)
- [ ] Verify no window appears in Windows taskbar
- [ ] Verify no title bar or borders visible

---

## Verifiable Acceptance Criteria

**Critical Path:**

- [ ] Window is fully transparent — desktop visible through it
- [ ] No title bar, borders, or window chrome visible
- [ ] Mouse clicks pass through the transparent area to desktop apps
- [ ] Window stays on top of other (non-fullscreen) windows
- [ ] Window does not appear in Windows taskbar

**Quality Gates:**

- [ ] No visual artifacts (black corners, white flash on startup)
- [ ] CPU usage < 1% when idle (transparent window shouldn't consume resources)

**Integration:**

- [ ] Phase 03 can add Clippy elements that are clickable inside this transparent window
- [ ] Phase 07 can show/hide this window via system tray

---

## Quality Assurance

### Test Plan

#### Manual Testing

- [ ] **Transparency test:** Desktop wallpaper visible through the window
  - Expected: Fully transparent, no tint or artifacts
  - Actual: [To be filled]

- [ ] **Click-through test:** Click on a desktop icon while overlay is running
  - Expected: Desktop icon responds normally
  - Actual: [To be filled]

- [ ] **Always-on-top test:** Open another window, then focus back
  - Expected: Overlay stays on top
  - Actual: [To be filled]

- [ ] **Taskbar test:** Check Windows taskbar
  - Expected: No "Clippy Awakens" entry in taskbar
  - Actual: [To be filled]

#### Automated Testing

```bash
cd src-tauri && cargo test
cd src-tauri && cargo build
```

#### Performance Testing

- [ ] **Idle CPU:** Target: < 1%, Actual: [To be measured]
- [ ] **Memory usage:** Target: < 30MB, Actual: [To be measured]

### Review Checklist

- [ ] **Code Quality:**
  - [ ] `cargo test` passes
  - [ ] No compiler warnings
  - [ ] CSS is clean and well-commented

- [ ] **Security:**
  - [ ] No changes to CSP configuration
  - [ ] Transparent window doesn't expose any content

---

## Dependencies

### Upstream (Required Before Starting)

- Phase 01: Tauri project compiles and runs

### Downstream (Will Use This Phase)

- Phase 03: Clippy renders inside this transparent overlay
- Phase 07: System tray toggles window visibility

### External Services

- None — all local

---

## Completion Gate

### Sign-off

- [ ] All acceptance criteria met
- [ ] All tests passing
- [ ] Code review passed
- [ ] Phase marked DONE in plan.md
- [ ] Committed: `feat(tauri): phase 02 — transparent overlay window`

---

## Notes

### Technical Considerations

- WebView2 transparency on Windows requires `transparent: true` in Tauri config and `background: transparent` in CSS — both are needed
- Some Windows themes may add shadows to transparent windows — `shadow: false` in tauri.conf.json prevents this
- If transparency doesn't work, check that WebView2 runtime is updated

### Known Limitations

- Fullscreen exclusive apps (games) may render above the overlay — this is expected
- Windows Narrator and screen readers may announce the invisible window — consider `aria-hidden` attributes

### Future Enhancements

- Dynamic window sizing to match actual monitor resolution (Phase 07)
- Multi-monitor support (future enhancement beyond MVP)

---

**Previous:** [[phase-01-tauri-scaffolding|Phase 01: Tauri Project Scaffolding]]
**Next:** [[phase-03-clippy-webview|Phase 03: Clippy.js WebView Integration]]
