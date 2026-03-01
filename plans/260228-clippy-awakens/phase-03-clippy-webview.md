---
title: "Phase 03: Clippy.js WebView Integration"
description: "Bundle jQuery and clippy.js inside the Tauri webview, load all 10 agents from local agent data, support extensible agent loading from a user directory, provide a switchAgent() function, and verify sprite animations render correctly in the transparent overlay."
skill: none
status: pending
group: "clippy-engine"
dependencies: ["phase-02"]
tags: [phase, clippy, jquery, webview, integration]
created: 2026-02-28
updated: 2026-02-28
---

# Phase 03: Clippy.js WebView Integration

**Context:** [Master Plan](./plan.md) | **Dependencies:** P02 | **Status:** Pending

---

## Overview

Bundle jQuery 1.12.4 and the existing clippy.js engine inside the Tauri webview. Bundle all 10 agents (Bonzi, Clippy, F1, Genie, Genius, Links, Merlin, Peedy, Rocky, Rover) and configure asset loading. Support extensible agent loading from a user directory (`%APPDATA%/clippy-awakens/agents/`). Provide a `switchAgent()` function to swap agents at runtime. Verify sprite animations render correctly in the transparent overlay.

**Goal:** All 10 agents load from bundled data. Clippy renders as the default draggable sprite. `switchAgent(name)` swaps to any bundled or user-added agent at runtime.

---

## Context & Workflow

### How This Phase Fits Into the Project

- **UI Layer:** Updates `ui/index.html` to load jQuery, clippy.js, clippy.css, and initialize agents
  - jQuery loaded from bundled local file (not CDN)
  - clippy.js loaded from existing `build/clippy.js`
  - All 10 bundled agents loaded via JSONP from `agents/{Name}/agent.js`
  - User agents loaded from `%APPDATA%/clippy-awakens/agents/{Name}/agent.js`
  - `switchAgent(name)` destroys current agent and loads a new one

- **Server Layer:** Tauri asset protocol + Tauri commands for user agent directory
  - `clippy.BASE_PATH` pointed to local agents directory
  - Tauri command `list_available_agents` scans bundled + user dirs

- **Database Layer:** N/A

- **Integrations:** clippy.js JSONP loading mechanism must work inside WebView2

### User Workflow

**Trigger:** App starts — Clippy should automatically appear.

**Steps:**
1. Tauri launches transparent overlay (Phase 02)
2. WebView loads jQuery + clippy.js
3. `clippy.load('Clippy', callback)` initializes the default agent (or last-used from config)
4. Agent appears in bottom-right corner with 'Greeting' animation (or fallback)
5. User can drag agent around the screen
6. Double-clicking agent plays a random animation
7. `switchAgent('Merlin')` can swap to any available agent at runtime

**Success Outcome:** Default agent sprite visible on screen, draggable, plays animations on double-click. `switchAgent()` swaps between all 10 bundled agents plus any user-added agents.

### Problem Being Solved

**Pain Point:** clippy.js currently only runs in a browser. It needs to work inside Tauri's WebView2.
**Alternative Approach:** Without bundling the existing engine, we'd need to rewrite the animation system in Rust or modern JS — a weeks-long effort.

### Integration Points

**Upstream Dependencies:**
- Phase 02: Transparent overlay window with `pointer-events` CSS

**Downstream Consumers:**
- Phase 04: Uses `agent.play()` and `agent.speak()` for event-driven animations
- Phase 05: HTTP server triggers events that ultimately call clippy.js API

**Data Flow:**
```
Tauri webview loads ui/index.html
  → <script> loads jQuery 1.12.4
  → <script> loads build/clippy.js
  → <link> loads build/clippy.css
  → JS: clippy.BASE_PATH = './agents/'
  → JS: clippy.load('Clippy', callback)   # Default agent
    → JSONP: loads agents/Clippy/agent.js (calls clippy.ready())
    → Image: loads agents/Clippy/map.png (sprite sheet)
    → JSONP: loads agents/Clippy/sounds-mp3.js (calls clippy.soundsReady())
  → Agent created, agent.show() called
  → Sprite renders at fixed position, draggable

switchAgent('Merlin') called (from tray menu or Tauri command):
  → currentAgent.hide() + destroy
  → clippy.load('Merlin', callback)
  → New agent appears at same position
```

---

## Prerequisites & Clarifications

### Questions for User

1. **jQuery Source:** Bundle jQuery 1.12.4 minified locally or use a specific version?
   - **Context:** The existing `index.html` loads jQuery from `cdnjs.cloudflare.com`. For Tauri, we need a local copy since the app may run without internet.
   - **Assumptions if unanswered:** Download and bundle jQuery 1.12.4 minified (~85KB) in `ui/vendor/jquery.min.js`
   - **Impact:** CDN loading would fail offline and add latency

2. **Clippy Start Position:** Where should Clippy appear on screen?
   - **Context:** The original clippy.js defaults to 80% from left, 80% from top. In a transparent fullscreen overlay, this means bottom-right area of the screen.
   - **Assumptions if unanswered:** Bottom-right corner — consistent with classic Clippy behavior
   - **Impact:** Position is easily changed later; this is just the initial default

3. **Auto-Show on Launch:** Should Clippy appear immediately on app start, or wait for an event?
   - **Context:** If Clippy shows immediately, the user sees it when they start the app. If hidden, it only appears on events (Phase 04+).
   - **Assumptions if unanswered:** Show immediately with a greeting on first launch. After initial greeting, hide and wait for events (controlled in Phase 07).
   - **Impact:** Affects whether `agent.show()` is called automatically or deferred

### Validation Checklist

- [ ] Phase 02 completed — transparent overlay window working
- [ ] `build/clippy.js` exists (existing build file)
- [ ] `build/clippy.css` exists (existing CSS)
- [ ] `agents/Clippy/` directory with agent.js, map.png, sounds-mp3.js

---

## Requirements

### Functional

- jQuery 1.12.4 loads successfully in WebView2
- clippy.js engine loads without errors
- All 10 bundled agents can load via JSONP callback mechanism
- Default agent (Clippy) sprite renders in the transparent overlay
- Sprite animations play correctly (correct frames, timing)
- Speech bubbles appear and display text with word-by-word typing effect
- Agent is draggable via mouse
- Double-click triggers random animation
- `switchAgent(name)` swaps to any available agent at runtime
- User agents from `%APPDATA%/clippy-awakens/agents/` can be loaded
- Tauri command `list_available_agents` returns bundled + user agent names

### Technical

- jQuery bundled locally in `ui/vendor/jquery.min.js`
- clippy.js loaded from existing `build/clippy.js`
- clippy.css loaded from existing `build/clippy.css`
- All 10 agent directories bundled: Bonzi, Clippy, F1, Genie, Genius, Links, Merlin, Peedy, Rocky, Rover (~13MB total)
- `clippy.BASE_PATH` set to resolve agent files from Tauri's asset protocol
- Tauri asset protocol configured to serve files from project root
- User agents directory at `%APPDATA%/clippy-awakens/agents/` created if missing
- Tauri `#[tauri::command]` for `list_available_agents` scans both directories
- No modifications to existing clippy.js source files

---

## Decision Log

### Local jQuery Bundle Over CDN (ADR-03-01)

**Date:** 2026-02-28
**Status:** Accepted

**Context:** clippy.js requires jQuery. The demo page loads it from CDN, but the Tauri app should work offline.

**Decision:** Bundle jQuery 1.12.4 minified in `ui/vendor/jquery.min.js`.

**Consequences:**
- **Positive:** Works offline, no network dependency, fast loading
- **Negative:** 85KB added to project (negligible for desktop app)
- **Neutral:** Same jQuery version the engine was built for

### Tauri Asset Protocol for Agent Data (ADR-03-02)

**Date:** 2026-02-28
**Status:** Accepted

**Context:** clippy.js loads agent data by injecting `<script>` tags with `src` attributes. In Tauri, local file access requires either the asset protocol or configuring the dev server to serve files.

**Decision:** Configure Tauri's `frontendDist` and asset resolution so relative paths in `<script src>` tags resolve to project files. Use Tauri's built-in dev server for development and asset protocol for production builds.

**Consequences:**
- **Positive:** JSONP script injection works as-is — no clippy.js modifications needed
- **Negative:** May need careful path configuration for production builds
- **Neutral:** Tauri's asset protocol handles this natively

---

## Implementation Steps

### Step 0: Test Definition (TDD)

**Purpose:** Define acceptance tests before writing implementation code

#### 0.1: JavaScript Smoke Tests

Since this is vanilla JS in a webview, testing is primarily manual. However, we can add a self-test function:

- [ ] Create `ui/clippy-test.js` — a self-test that verifies the engine loaded:

```javascript
// Self-test — called after Clippy loads, logs to console
function clippySelfTest(agent) {
    var results = [];

    // Test 1: Agent exists
    results.push({
        test: 'Agent loaded',
        pass: !!agent
    });

    // Test 2: Has animations
    var anims = agent.animations();
    results.push({
        test: 'Has animations',
        pass: anims.length > 0,
        detail: anims.length + ' animations available'
    });

    // Test 3: Has required animations for events
    var required = ['Congratulate', 'Alert', 'Thinking', 'Wave', 'GetAttention'];
    required.forEach(function(name) {
        results.push({
            test: 'Has ' + name + ' animation',
            pass: agent.hasAnimation(name)
        });
    });

    // Test 4: Speak works
    results.push({
        test: 'speak() is a function',
        pass: typeof agent.speak === 'function'
    });

    // Log results
    console.log('=== Clippy Self-Test ===');
    results.forEach(function(r) {
        console.log((r.pass ? 'PASS' : 'FAIL') + ': ' + r.test + (r.detail ? ' (' + r.detail + ')' : ''));
    });

    var passed = results.filter(function(r) { return r.pass; }).length;
    console.log('Result: ' + passed + '/' + results.length + ' passed');

    return results;
}
```

#### 0.2: Visual Verification

- [ ] Clippy sprite appears on screen
- [ ] Animations play without visual glitches
- [ ] Speech bubble renders with correct styling
- [ ] Drag and drop works

---

### Step 1: Bundle jQuery Locally

#### 1.1: Download jQuery

- [ ] Create `ui/vendor/` directory
- [ ] Download jQuery 1.12.4 minified to `ui/vendor/jquery.min.js`
- [ ] Verify file size (~85KB)

```bash
mkdir -p ui/vendor
curl -o ui/vendor/jquery.min.js https://cdnjs.cloudflare.com/ajax/libs/jquery/1.12.4/jquery.min.js
```

---

### Step 2: Configure Tauri Asset Serving

#### 2.1: Update tauri.conf.json for Asset Resolution

- [ ] Ensure Tauri serves files from the project root so relative paths to `build/`, `agents/`, `src/` work
- [ ] Update `frontendDist` to point to `../ui` for the main HTML
- [ ] Add asset scope to allow loading from `../build/`, `../agents/`, `../src/`:

```json
{
  "app": {
    "security": {
      "csp": null
    }
  }
}
```

#### 2.2: Adjust Asset Paths

- [ ] Since Tauri serves from `ui/` directory, paths to `build/` and `agents/` need to go up one level: `../build/`, `../agents/`
- [ ] OR: Symlink or copy needed files into `ui/` directory
- [ ] Best approach: Set Tauri's `frontendDist` to `..` (project root) and rename `ui/index.html` to be loaded by a redirect, OR copy the needed assets into `ui/`

**Recommended:** Create symlinks or configure Tauri's dev server to serve from project root:

```json
{
  "build": {
    "frontendDist": "../ui",
    "devUrl": "http://localhost:1420"
  }
}
```

Then in `ui/index.html`, reference files relative to the Tauri asset root. During development, Tauri's dev server can be configured to serve additional directories.

Alternative simpler approach — copy/symlink needed files into `ui/`:
- [ ] `ui/build/clippy.js` → symlink to `../build/clippy.js`
- [ ] `ui/build/clippy.css` → symlink to `../build/clippy.css`
- [ ] `ui/agents/` → symlink to `../agents/`

---

### Step 3: Update ui/index.html to Load Clippy

#### 3.1: Add Script and Style Tags

- [ ] Update `ui/index.html`:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Clippy Awakens</title>
    <link rel="stylesheet" type="text/css" href="build/clippy.css">
    <style>
        html, body {
            margin: 0;
            padding: 0;
            width: 100%;
            height: 100%;
            background: transparent;
            overflow: hidden;
            pointer-events: none;
        }

        /* Clippy elements intercept mouse events */
        .clippy,
        .clippy-balloon {
            pointer-events: auto;
        }
    </style>
</head>
<body>
    <script src="vendor/jquery.min.js"></script>
    <script src="build/clippy.js"></script>
    <script src="clippy-test.js"></script>
    <script>
        // Point to local agents directory
        clippy.BASE_PATH = 'agents/';

        var currentAgent = null;
        var currentAgentName = 'Clippy';

        // Switch to a different agent at runtime
        function switchAgent(name, callback) {
            console.log('Switching agent to:', name);

            // Save current position before destroying
            var lastPosition = null;
            if (currentAgent) {
                var offset = currentAgent._el ? currentAgent._el.offset() : null;
                if (offset) {
                    lastPosition = { left: offset.left, top: offset.top };
                }
                // Hide and destroy current agent
                currentAgent.hide(true, true);
                // Remove DOM elements
                if (currentAgent._el) currentAgent._el.remove();
                if (currentAgent._balloon && currentAgent._balloon._balloon) {
                    currentAgent._balloon._balloon.remove();
                }
                currentAgent = null;
            }

            clippy.load(name, function(agent) {
                currentAgent = agent;
                currentAgentName = name;

                // Restore position if we had one
                if (lastPosition) {
                    agent.moveTo(lastPosition.left, lastPosition.top, 0);
                }

                agent.show();

                // Run self-test on new agent
                clippySelfTest(agent);

                console.log('Switched to agent:', name);
                if (callback) callback(agent);
            }, function(err) {
                console.error('Failed to load agent ' + name + ':', err);
            });
        }

        // Load default agent on startup
        clippy.load('Clippy', function(agent) {
            currentAgent = agent;
            agent.show();
            agent.speak('Hello! Clippy Awakens is running.');

            // Run self-test
            clippySelfTest(agent);

            console.log('Clippy loaded successfully');
        }, function(err) {
            console.error('Failed to load Clippy:', err);
        });
    </script>
</body>
</html>
```

---

### Step 4: Create Tauri Command for Agent Discovery

#### 4.1: Create src-tauri/src/agents.rs

- [ ] Implement agent discovery across bundled + user directories:

```rust
use std::fs;
use std::path::PathBuf;
use log::info;

/// Known bundled agents (shipped with the app)
const BUNDLED_AGENTS: &[&str] = &[
    "Bonzi", "Clippy", "F1", "Genie", "Genius",
    "Links", "Merlin", "Peedy", "Rocky", "Rover",
];

#[derive(serde::Serialize)]
pub struct AgentInfo {
    pub name: String,
    pub source: String, // "bundled" or "user"
}

/// Get the user agents directory (%APPDATA%/clippy-awakens/agents/)
pub fn get_user_agents_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("clippy-awakens").join("agents"))
}

/// List all available agents from bundled + user directories
#[tauri::command]
pub fn list_available_agents() -> Vec<AgentInfo> {
    let mut agents: Vec<AgentInfo> = BUNDLED_AGENTS
        .iter()
        .map(|name| AgentInfo {
            name: name.to_string(),
            source: "bundled".to_string(),
        })
        .collect();

    // Scan user agents directory
    if let Some(user_dir) = get_user_agents_dir() {
        if user_dir.exists() {
            if let Ok(entries) = fs::read_dir(&user_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        // Verify it has agent.js and map.png
                        if path.join("agent.js").exists() && path.join("map.png").exists() {
                            let name = entry.file_name().to_string_lossy().to_string();
                            // Don't duplicate bundled agents
                            if !BUNDLED_AGENTS.contains(&name.as_str()) {
                                agents.push(AgentInfo {
                                    name,
                                    source: "user".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        } else {
            // Create user agents directory for future use
            fs::create_dir_all(&user_dir).ok();
            info!("Created user agents directory: {:?}", user_dir);
        }
    }

    agents
}
```

#### 4.2: Add agents Module to main.rs

- [ ] Update `src-tauri/src/main.rs`:

```rust
mod agents;

// In builder:
.invoke_handler(tauri::generate_handler![
    agents::list_available_agents,
    // ... other commands
])
```

#### 4.3: Add dirs Dependency to Cargo.toml

- [ ] Add `dirs = "5"` to `[dependencies]` in `src-tauri/Cargo.toml`

---

### Step 5: Verify JSONP Loading in WebView2

#### 5.1: Test Agent Data Loading

- [ ] The JSONP mechanism works by:
  1. `clippy.load._loadAgent('Clippy', path)` creates a `$.Deferred()`
  2. Injects `<script src="agents/Clippy/agent.js">` into `<head>`
  3. `agent.js` calls `clippy.ready('Clippy', data)` which resolves the deferred
- [ ] Verify this works in WebView2 by checking console for "Clippy loaded successfully"
- [ ] If JSONP fails, the fallback is to inline the agent data (but this should work since WebView2 supports dynamic script injection)

#### 5.2: Test Sprite Sheet Loading

- [ ] `map.png` loads via `new Image()` with `img.setAttribute('src', path + '/map.png')`
- [ ] Verify sprite renders correctly — no broken image, correct frame positions

#### 5.3: Test Sound Loading

- [ ] `sounds-mp3.js` loads via script injection, calls `clippy.soundsReady()`
- [ ] Built-in clippy.js sounds are base64-encoded — no file system access needed
- [ ] Note: These are clippy.js animation sounds, NOT the notification event sounds (Phase 06)

---

### Step 6: Verify Animation and Interaction

#### 6.1: Test Core Animations

- [ ] `agent.show()` — Clippy appears with 'Show' animation
- [ ] `agent.play('Greeting')` — plays greeting animation
- [ ] `agent.play('Wave')` — plays wave animation
- [ ] `agent.speak('Test message')` — speech bubble appears with word-by-word typing
- [ ] Double-click Clippy — triggers random animation
- [ ] Drag Clippy — repositions the sprite

#### 6.2: Test Agent Switching

- [ ] Call `switchAgent('Merlin')` in console — Clippy disappears, Merlin appears
- [ ] Call `switchAgent('Bonzi')` — Merlin disappears, Bonzi appears
- [ ] Verify position is preserved across agent switch
- [ ] Verify all 10 bundled agents can load without errors
- [ ] Test `list_available_agents` Tauri command returns all 10 agents

#### 6.3: Verify Transparency Interaction

- [ ] Clippy sprite is visible against desktop (transparent window)
- [ ] Clicking on Clippy works (pointer-events: auto)
- [ ] Clicking outside Clippy passes through to desktop (pointer-events: none)
- [ ] Speech bubble is clickable/visible
- [ ] Sprite sheet renders correctly on transparent background (no black box around sprite)

---

## Verifiable Acceptance Criteria

**Critical Path:**

- [ ] jQuery loads without errors in WebView2 console
- [ ] clippy.js loads without errors
- [ ] Default agent (Clippy) loads via JSONP (agent.js, map.png, sounds-mp3.js)
- [ ] Default agent sprite renders visibly on transparent overlay
- [ ] At least 5 animations play correctly (Show, Greeting, Wave, Thinking, Congratulate)
- [ ] Speech bubble displays text with typing effect
- [ ] Agent is draggable
- [ ] `switchAgent(name)` swaps to a different agent at runtime
- [ ] All 10 bundled agents can be loaded without errors
- [ ] `list_available_agents` Tauri command returns all bundled agent names
- [ ] User agents directory created at `%APPDATA%/clippy-awakens/agents/`

**Quality Gates:**

- [ ] No JavaScript console errors during normal operation
- [ ] Sprite renders without visual artifacts on transparent background
- [ ] Animation timing matches original browser behavior
- [ ] Agent switching preserves screen position

**Integration:**

- [ ] `window.currentAgent` is accessible for Phase 04 event handler bridge
- [ ] `window.currentAgentName` reflects current agent name
- [ ] `window.switchAgent()` is accessible for Phase 07 tray agent picker
- [ ] Phase 04 can call `currentAgent.play()` and `currentAgent.speak()`

---

## Quality Assurance

### Test Plan

#### Manual Testing

- [ ] **Load test:** Open dev tools console, check for errors
  - Expected: No errors, "Clippy loaded successfully" in console
  - Actual: [To be filled]

- [ ] **Animation test:** Call `currentAgent.play('Congratulate')` in console
  - Expected: Congratulate animation plays
  - Actual: [To be filled]

- [ ] **Speech test:** Call `currentAgent.speak('Testing speech bubble')` in console
  - Expected: Speech bubble appears with word-by-word text
  - Actual: [To be filled]

- [ ] **Drag test:** Click and drag Clippy
  - Expected: Clippy follows mouse, repositions on release
  - Actual: [To be filled]

- [ ] **Self-test:** Check console for self-test results
  - Expected: All tests pass (5/5 required animations available)
  - Actual: [To be filled]

#### Automated Testing

```bash
cd src-tauri && cargo test
cd src-tauri && cargo build
```

### Review Checklist

- [ ] **Code Quality:**
  - [ ] No modifications to existing clippy.js source
  - [ ] Clean HTML structure in ui/index.html
  - [ ] Console logging for debugging

- [ ] **Security:**
  - [ ] jQuery loaded from local bundle, not CDN
  - [ ] No external network requests

---

## Dependencies

### Upstream (Required Before Starting)

- Phase 02: Transparent overlay window
- jQuery 1.12.4 minified (downloaded during implementation)

### Downstream (Will Use This Phase)

- Phase 04: Event-to-animation mapping uses `currentAgent.play()` and `currentAgent.speak()`
- Phase 05: HTTP server events ultimately trigger Clippy animations

### External Services

- crates.io: Rust dependencies (already resolved in Phase 01)
- CDN: One-time jQuery download during implementation (not runtime)

---

## Completion Gate

### Sign-off

- [ ] All acceptance criteria met
- [ ] Self-test passes (all required animations available)
- [ ] Code review passed
- [ ] Phase marked DONE in plan.md
- [ ] Committed: `feat(clippy): phase 03 — clippy.js webview integration`

---

## Notes

### Technical Considerations

- clippy.js uses `$(document.body).append()` to add DOM elements — works in WebView2
- The JSONP callback pattern (`clippy.ready()`, `clippy.soundsReady()`) requires script injection which works in WebView2 since CSP is null
- Sprite sheet `map.png` is 1.3MB — loads fine for a desktop app, but verify no flash of unstyled content
- clippy.css uses `position: fixed` and `z-index: 1000` — both work in the overlay

### Known Limitations

- Sound files in agent data are base64-encoded — they play through the browser audio API, separate from the Windows notification sounds in Phase 06.
- User agents directory requires manually copying agent folders with correct structure (agent.js, map.png, sounds-mp3.js)
- Agent animation availability varies — some agents lack key animations (Rover only has 3 of 7 key animations). Fallback handling is in Phase 04.

### Agent Animation Availability

All 10 bundled agents and their key animation support:

| Agent | Congratulate | Alert | GetAttention | Thinking | Wave | Greeting | GoodBye |
|-------|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| Clippy | Y | Y | Y | Y | Y | Y | Y |
| Links | Y | Y | Y | Y | Y | Y | Y |
| F1 | Y | Y | Y | Y | Y | Y | - |
| Genius | Y | Y | Y | Y | Y | Y | - |
| Rocky | Y | Y | Y | Y | Y | Y | - |
| Genie | Y | Y | Y | Y | Y | - | - |
| Merlin | Y | Y | Y | Y | Y | - | - |
| Peedy | Y | Y | Y | Y | Y | - | - |
| Bonzi | Y | Y | Y | - | Y | - | - |
| Rover | - | - | Y | Y | - | - | - |

### Future Enhancements

- Add keyboard shortcut to dismiss/summon agent
- Agent marketplace / community agent sharing
- Drag-and-drop agent installation

---

**Previous:** [[phase-02-transparent-overlay|Phase 02: Transparent Overlay Window]]
**Next:** [[phase-04-event-mapping|Phase 04: Event Animation Mapping & Speech Bubbles]]
