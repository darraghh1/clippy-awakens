---
title: "Phase 01: Tauri Project Scaffolding"
description: "Initialize the Tauri v2 project with Rust backend, configure Cargo.toml dependencies, create tauri.conf.json, and establish the basic project structure that wraps the existing clippy.js codebase."
skill: none
status: pending
group: "tauri-foundation"
dependencies: []
tags: [phase, tauri, rust, setup]
created: 2026-02-28
updated: 2026-02-28
---

# Phase 01: Tauri Project Scaffolding

**Context:** [Master Plan](./plan.md) | **Dependencies:** None | **Status:** Pending

---

## Overview

Initialize the Tauri v2 project structure inside the existing clippy.js repository. This creates the `src-tauri/` directory with Rust backend code, configures all Cargo dependencies, sets up tauri.conf.json for the window, and creates a minimal `ui/index.html` that the Tauri webview will load (separate from the original demo `index.html`).

**Goal:** A Tauri app that compiles and opens a basic window loading a simple HTML page, proving the Tauri + Rust + WebView2 toolchain works.

---

## Context & Workflow

### How This Phase Fits Into the Project

- **UI Layer:** Creates `ui/index.html` — the Tauri webview entry point (not the original demo page)
  - Minimal HTML that will later load jQuery + clippy.js (Phase 03)

- **Server Layer:** Creates `src-tauri/src/main.rs` — Tauri app entry point
  - Basic Tauri builder with default window configuration

- **Database Layer:** N/A — no database in this project

- **Integrations:** N/A — no external services yet (HTTP server comes in Phase 05)

### User Workflow

**Trigger:** Developer clones the repo and wants to build/run the Tauri app.

**Steps:**
1. Install Rust toolchain (`rustup`) and Tauri CLI
2. Run `cargo tauri dev` from project root
3. Tauri compiles Rust backend and opens a window with the HTML page
4. Developer sees a basic window confirming the toolchain works

**Success Outcome:** `cargo tauri dev` compiles without errors and opens a window displaying "Clippy Awakens" text.

### Problem Being Solved

**Pain Point:** No desktop app exists yet — the clippy.js engine only runs in a browser.
**Alternative Approach:** Without this phase, there's no Tauri project to build on. Everything downstream depends on this scaffold.

### Integration Points

**Upstream Dependencies:**
- Existing clippy.js codebase (files preserved, not modified)
- Rust toolchain installed on developer machine

**Downstream Consumers:**
- Phase 02 (Transparent Overlay Window) — modifies tauri.conf.json and window config
- Phase 03 (Clippy.js WebView) — modifies ui/index.html to load clippy.js
- Phase 05 (HTTP Server) — adds axum server to main.rs
- All subsequent phases build on this foundation

**Data Flow:**
```
cargo tauri dev
  → Compiles src-tauri/src/main.rs (Rust)
  → Launches WebView2 window
  → Loads ui/index.html
  → Displays basic page
```

---

## Prerequisites & Clarifications

### Questions for User

1. **Rust Toolchain:** Is the Rust toolchain already installed on the Windows machine, or should this phase include installation instructions?
   - **Context:** Tauri v2 requires Rust 1.77.2+ and the MSVC build tools
   - **Assumptions if unanswered:** Include setup instructions in the phase for reference, but assume the user will install before implementation begins
   - **Impact:** Build will fail without Rust + MSVC installed

2. **Tauri Version:** Should we use Tauri v2 stable (2.x) or the latest release?
   - **Context:** Tauri v2 has stable transparent window support on Windows
   - **Assumptions if unanswered:** Use latest Tauri v2 stable release
   - **Impact:** API differences between v1 and v2 are significant

3. **Node.js Requirement:** Tauri CLI can be installed via `cargo install` or via npm. Which approach?
   - **Context:** `cargo install tauri-cli` is self-contained; `npm install @tauri-apps/cli` requires Node.js
   - **Assumptions if unanswered:** Use `cargo install tauri-cli` to avoid Node.js dependency
   - **Impact:** Build workflow differs slightly

### Validation Checklist

- [ ] Rust toolchain installed (rustup, cargo, rustc)
- [ ] MSVC build tools installed (Windows SDK, C++ build tools)
- [ ] WebView2 runtime available (pre-installed on Windows 10 21H2+)
- [ ] Project root identified as `/home/darragh/Projects/clippy/`

---

## Requirements

### Functional

- Tauri app compiles and runs with `cargo tauri dev`
- Basic window opens and displays HTML content
- Existing clippy.js files (src/, agents/, build/) are not modified
- New `ui/index.html` created for Tauri (separate from demo `index.html`)

### Technical

- Tauri v2 with Rust backend
- Cargo.toml includes: `tauri`, `serde`, `serde_json`, `tokio` (full features)
- tauri.conf.json configured with correct `frontendDist` pointing to `../ui`
- `src-tauri/` directory structure follows Tauri v2 conventions
- `.gitignore` updated to exclude `src-tauri/target/`

---

## Decision Log

### src-tauri Directory Placement (ADR-01-01)

**Date:** 2026-02-28
**Status:** Accepted

**Context:** Tauri projects typically have `src-tauri/` at the project root. Our project already has `src/` (clippy.js source). Need to avoid conflicts.

**Decision:** Place Tauri code in `src-tauri/` per convention. The existing `src/` directory contains clippy.js and is untouched.

**Consequences:**
- **Positive:** Standard Tauri layout, familiar to Tauri developers
- **Negative:** None — `src/` and `src-tauri/` are clearly distinct
- **Neutral:** `cargo tauri dev` knows to look in `src-tauri/`

### Separate ui/ Directory for Tauri HTML (ADR-01-02)

**Date:** 2026-02-28
**Status:** Accepted

**Context:** The existing `index.html` is a demo page with Windows 98 styling and control buttons. The Tauri webview needs a different HTML page (transparent background, no demo chrome).

**Decision:** Create `ui/index.html` for the Tauri webview. Keep original `index.html` as the browser demo.

**Consequences:**
- **Positive:** Original demo preserved for browser testing
- **Negative:** Two HTML entry points to maintain
- **Neutral:** Tauri's `frontendDist` config points to `../ui`

---

## Implementation Steps

### Step 0: Test Definition (TDD)

**Purpose:** Define acceptance tests before writing implementation code

#### 0.1: Rust Unit Tests

- [ ] Create `src-tauri/src/main.rs` with a basic test module
- [ ] Test that the app configuration is valid (smoke test)

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_app_builds() {
        // Verify tauri::Builder can be constructed without panic
        // This is a smoke test — real integration testing is manual
        assert!(true, "Tauri project compiles successfully");
    }
}
```

#### 0.2: Build Verification

- [ ] `cargo build` compiles without errors in `src-tauri/`
- [ ] `cargo test` passes in `src-tauri/`

---

### Step 1: Create src-tauri Directory Structure

#### 1.1: Initialize Tauri Project Structure

- [ ] Create `src-tauri/` directory
- [ ] Create `src-tauri/src/` directory
- [ ] Create `src-tauri/icons/` directory (Tauri requires app icons)
- [ ] Create `src-tauri/sounds/` directory (placeholder for Phase 06)

#### 1.2: Create Cargo.toml

- [ ] Create `src-tauri/Cargo.toml` with dependencies:

```toml
[package]
name = "clippy-awakens"
version = "0.1.0"
edition = "2021"
description = "Clippy desktop notification overlay for Claude Code"

[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
axum = "0.8"
rodio = "0.20"
log = "0.4"
env_logger = "0.11"

[build-dependencies]
tauri-build = { version = "2", features = [] }

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
```

#### 1.3: Create build.rs

- [ ] Create `src-tauri/build.rs`:

```rust
fn main() {
    tauri_build::build()
}
```

---

### Step 2: Create Tauri Configuration

#### 2.1: Create tauri.conf.json

- [ ] Create `src-tauri/tauri.conf.json`:

```json
{
  "$schema": "https://raw.githubusercontent.com/tauri-apps/tauri/dev/crates/tauri-cli/schema.json",
  "productName": "Clippy Awakens",
  "version": "0.1.0",
  "identifier": "com.digitalmastery.clippy-awakens",
  "build": {
    "frontendDist": "../ui"
  },
  "app": {
    "windows": [
      {
        "title": "Clippy Awakens",
        "width": 800,
        "height": 600
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/icon.png"
    ]
  }
}
```

Note: Window dimensions are temporary — Phase 02 changes to fullscreen transparent overlay.

#### 2.2: Create Capabilities File

- [ ] Create `src-tauri/capabilities/default.json`:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-open"
  ]
}
```

---

### Step 3: Create Rust Entry Point

#### 3.1: Create main.rs

- [ ] Create `src-tauri/src/main.rs`:

```rust
// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

### Step 4: Create Tauri WebView HTML

#### 4.1: Create ui/ Directory and index.html

- [ ] Create `ui/` directory
- [ ] Create `ui/index.html`:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Clippy Awakens</title>
    <style>
        body {
            margin: 0;
            padding: 0;
            font-family: 'Segoe UI', sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            background: #008080;
            color: white;
        }
        h1 { font-size: 24px; }
        p { font-size: 14px; color: #ccc; }
    </style>
</head>
<body>
    <div>
        <h1>Clippy Awakens</h1>
        <p>Tauri webview loaded successfully.</p>
        <p>Phase 01 complete — scaffold working.</p>
    </div>
</body>
</html>
```

---

### Step 5: Create Placeholder Icon

#### 5.1: Generate App Icon

- [ ] Create a simple placeholder icon at `src-tauri/icons/icon.png` (32x32 or larger PNG)
- [ ] Can use any paperclip image or generate a simple one

---

### Step 6: Update .gitignore

#### 6.1: Add Tauri Build Artifacts

- [ ] Add to `.gitignore`:

```
# Tauri build artifacts
src-tauri/target/
src-tauri/gen/
```

---

### Step 7: Verify Build

#### 7.1: Compile and Run

- [ ] Run `cd src-tauri && cargo build` — should compile without errors
- [ ] Run `cargo tauri dev` from project root — should open window with placeholder HTML
- [ ] Run `cd src-tauri && cargo test` — should pass smoke tests

---

## Verifiable Acceptance Criteria

**Critical Path:**

- [ ] `src-tauri/` directory exists with proper structure
- [ ] `cargo build` in `src-tauri/` completes without errors
- [ ] `cargo tauri dev` opens a window displaying "Clippy Awakens" text
- [ ] Existing clippy.js files (src/, agents/, build/) are unmodified
- [ ] `ui/index.html` is separate from the original demo `index.html`

**Quality Gates:**

- [ ] Compilation time < 5 minutes (first build, subsequent builds < 30s)
- [ ] No compiler warnings in main.rs
- [ ] All dependencies resolve correctly

**Integration:**

- [ ] Phase 02 can modify tauri.conf.json window config
- [ ] Phase 03 can add script tags to ui/index.html
- [ ] Phase 05 can add HTTP server code to main.rs

---

## Quality Assurance

### Test Plan

#### Manual Testing

- [ ] **Build test:** `cargo build` in src-tauri/ compiles without errors
  - Expected: Clean compilation
  - Actual: [To be filled during testing]

- [ ] **Run test:** `cargo tauri dev` opens a window
  - Expected: Window shows "Clippy Awakens" text
  - Actual: [To be filled during testing]

- [ ] **Preservation test:** Original index.html unchanged
  - Expected: `git diff index.html` shows no changes
  - Actual: [To be filled during testing]

#### Automated Testing

```bash
cd src-tauri && cargo test
cd src-tauri && cargo build
```

#### Performance Testing

- [ ] **Build time:** Target: < 5min first build, Actual: [To be measured]
- [ ] **Binary size:** Target: < 10MB, Actual: [To be measured]

### Review Checklist

- [ ] **Code Quality:**
  - [ ] `cargo test` passes
  - [ ] No compiler warnings
  - [ ] Clean Cargo.toml with pinned major versions

- [ ] **Security:**
  - [ ] No hardcoded secrets
  - [ ] CSP configured appropriately for local-only app

- [ ] **Documentation:**
  - [ ] README.md has setup instructions (Rust toolchain, build steps)

---

## Dependencies

### Upstream (Required Before Starting)

- Rust toolchain: rustup, cargo, rustc (1.77.2+)
- Windows: MSVC Build Tools, Windows SDK
- WebView2 Runtime (pre-installed on Windows 10 21H2+)

### Downstream (Will Use This Phase)

- Phase 02: Modifies tauri.conf.json for transparent overlay
- Phase 03: Adds clippy.js loading to ui/index.html
- Phase 05: Adds axum HTTP server to main.rs
- All subsequent phases depend on this foundation

### External Services

- crates.io: Rust package registry for dependencies
- No runtime external services

---

## Completion Gate

### Sign-off

- [ ] All acceptance criteria met
- [ ] All tests passing
- [ ] Code review passed
- [ ] Phase marked DONE in plan.md
- [ ] Committed: `feat(tauri): phase 01 — project scaffolding complete`

---

## Notes

### Technical Considerations

- Tauri v2 uses `tauri::Builder::default()` — v1 patterns won't work
- WebView2 is pre-installed on Windows 10 21H2+ — no need to bundle it
- The `custom-protocol` feature is needed for production builds to load local files

### Known Limitations

- No transparent window yet (Phase 02)
- No clippy.js loading yet (Phase 03)
- Placeholder icon — should be replaced with a proper paperclip icon later

### Future Enhancements

- Consider auto-update via Tauri's updater plugin
- Could add a dev mode that hot-reloads the webview

---

**Next:** [[phase-02-transparent-overlay|Phase 02: Transparent Overlay Window]]
