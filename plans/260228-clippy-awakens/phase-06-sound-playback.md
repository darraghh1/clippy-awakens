---
title: "Phase 06: Windows Sound Playback"
description: "Implement Windows notification sound playback via the rodio crate, mapping each hook event type to an appropriate .wav file from the Windows system sounds directory."
skill: none
status: pending
group: "rust-services"
dependencies: ["phase-05"]
tags: [phase, rust, rodio, sound, windows, audio]
created: 2026-02-28
updated: 2026-02-28
---

# Phase 06: Windows Sound Playback

**Context:** [Master Plan](./plan.md) | **Dependencies:** P05 | **Status:** Pending

---

## Overview

Add Windows notification sound playback to the Tauri app. When an event is received via the HTTP server (Phase 05), play an appropriate Windows system sound (.wav file) alongside the Clippy animation. This replaces the planned PowerShell sound server's audio functionality entirely.

**Goal:** Each hook event triggers a distinct, semantically appropriate Windows notification sound that plays non-blocking alongside Clippy's animation.

---

## Context & Workflow

### How This Phase Fits Into the Project

- **UI Layer:** No changes — sounds play from the Rust backend
  - Clippy animations (Phase 04) continue to work independently

- **Server Layer:** Creates `src-tauri/src/sounds.rs` — sound playback engine
  - Integrates with HTTP server handlers (Phase 05)
  - Uses rodio crate for non-blocking .wav playback

- **Database Layer:** N/A

- **Integrations:** Windows system sound files from `C:\Windows\Media\`

### User Workflow

**Trigger:** Same as Phase 05 — `curl GET localhost:9999/complete`

**Steps:**
1. HTTP handler receives event (Phase 05)
2. Handler calls sound engine with event type
3. Sound engine looks up .wav file for the event type
4. rodio plays the .wav file non-blocking on the default audio output
5. Simultaneously, Tauri event emits to webview for Clippy animation

**Success Outcome:** User hears a pleasant notification chime AND sees Clippy animate simultaneously.

### Problem Being Solved

**Pain Point:** The user has Bluetooth hearing aids with ~1-2 second reconnect delay when audio starts after silence. Sounds need to play promptly.
**Alternative Approach:** The original plan was a PowerShell script using `System.Media.SoundPlayer`. Rust's rodio is more reliable and integrates directly into the app.

### Integration Points

**Upstream Dependencies:**
- Phase 05: HTTP server handlers trigger sound playback

**Downstream Consumers:**
- Phase 07: System tray may allow muting sounds
- Phase 08: Final integration testing verifies sound + animation sync

**Data Flow:**
```
HTTP handler receives "complete"
  → sounds::play_event_sound("complete")
  → Look up: "complete" → "Windows Notify System Generic.wav"
  → rodio::Decoder opens .wav file
  → rodio::Sink plays on default audio output (non-blocking)
  → Handler also emits Tauri event (Clippy animation)
```

---

## Prerequisites & Clarifications

### Questions for User

1. **Sound Source:** Should we use Windows built-in system sounds or bundle custom .wav files?
   - **Context:** Windows has ~50+ .wav files in `C:\Windows\Media\`. We can also bundle custom sounds in the app resources.
   - **Assumptions if unanswered:** Use Windows built-in sounds for MVP. If specific sounds aren't available, bundle fallback .wav files in `src-tauri/sounds/`.
   - **Impact:** Using system sounds means no extra files to distribute. Bundled sounds add ~500KB to the installer.

2. **Bluetooth Hearing Aid Workaround:** Should we implement a "keep audio session alive" trick?
   - **Context:** The user mentioned Bluetooth hearing aids with a reconnect delay. Playing a very short silent audio clip periodically could keep the Bluetooth audio session active.
   - **Assumptions if unanswered:** Don't implement the keepalive for MVP. If the delay is problematic, add it in Phase 08 or as a follow-up.
   - **Impact:** Without keepalive, first sound after silence may be delayed ~1-2 seconds.

3. **Volume Control:** Should the app have its own volume control, or rely on Windows system volume?
   - **Context:** rodio supports volume adjustment per Sink.
   - **Assumptions if unanswered:** Use system volume. No app-specific volume control for MVP.
   - **Impact:** Minimal — can be added later via tray menu.

### Validation Checklist

- [ ] Phase 05 completed — HTTP server running
- [ ] rodio in Cargo.toml (added in Phase 01)
- [ ] Windows machine has `C:\Windows\Media\` directory with .wav files
- [ ] Default audio output device configured

---

## Requirements

### Functional

- Each event type plays a distinct, semantically appropriate sound
- Sounds play non-blocking (don't delay HTTP response or Clippy animation)
- Missing .wav files are handled gracefully (log warning, don't crash)
- Sound plays on the default audio output device
- Multiple rapid events can overlap sounds (or queue them)

### Technical

- `rodio` crate for cross-platform audio playback
- Sound file paths resolved at runtime from Windows system directory
- Fallback to bundled sounds if system sounds not found
- Non-blocking playback via `rodio::Sink` or `rodio::OutputStream`
- Error handling: audio device unavailable, file not found, decode error

---

## Decision Log

### Windows System Sounds Over Custom (ADR-06-01)

**Date:** 2026-02-28
**Status:** Accepted

**Context:** Need appropriate notification sounds for 5 event types. Options: (1) Use Windows built-in .wav files, (2) Bundle custom .wav files, (3) Generate tones programmatically.

**Decision:** Use Windows system sounds from `C:\Windows\Media\` as primary, with bundled fallbacks.

**Consequences:**
- **Positive:** No extra files to distribute, sounds are familiar to the user
- **Negative:** File names/availability may vary across Windows versions
- **Neutral:** Can always add custom sounds later

**Alternatives Considered:**
1. Bundle custom sounds: Adds ~500KB, more control but extra work
2. Programmatic tones: Too robotic, no warmth

### Event-to-Sound Mapping (ADR-06-02)

**Date:** 2026-02-28
**Status:** Accepted

**Context:** Need to map 5 event types to semantically appropriate Windows sounds.

**Decision:** Use the following mapping (verified on Windows 10/11):

| Event | Sound File | Semantic Match |
|-------|-----------|----------------|
| complete | `Windows Notify System Generic.wav` | Pleasant completion chime |
| error | `Windows Critical Stop.wav` | Attention-grabbing error |
| attention | `Windows Notify Calendar.wav` | Gentle "look here" tone |
| stop | `Windows Notify Email.wav` | Neutral informational |
| session-end | `Windows Logoff Sound.wav` | Wrap-up / goodbye feel |

**Consequences:**
- **Positive:** Each event has a distinct, recognizable sound
- **Negative:** Sound file names may differ slightly across Windows versions
- **Neutral:** Easily changed by modifying the mapping

---

## Implementation Steps

### Step 0: Test Definition (TDD)

**Purpose:** Define Rust tests for sound module

#### 0.1: Rust Unit Tests

- [ ] Create `src-tauri/src/sounds.rs` with test module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_mapping_all_events() {
        let events = ["complete", "error", "attention", "stop", "session-end"];
        for event in &events {
            let path = get_sound_path(event);
            assert!(
                path.is_some(),
                "Event '{}' should have a sound mapping",
                event
            );
        }
    }

    #[test]
    fn test_sound_mapping_unknown_event() {
        assert!(get_sound_path("unknown").is_none());
    }

    #[test]
    fn test_sound_file_names() {
        // Verify the expected file names are in our mapping
        let path = get_sound_path("complete").unwrap();
        assert!(
            path.to_string_lossy().contains(".wav"),
            "Sound path should be a .wav file"
        );
    }
}
```

---

### Step 1: Create Sound Playback Module

#### 1.1: Create src-tauri/src/sounds.rs

- [ ] Implement sound playback engine:

```rust
use std::path::PathBuf;
use std::io::BufReader;
use std::fs::File;
use log::{info, warn, error};

/// Map event types to Windows system sound file names
fn get_system_sound_name(event_type: &str) -> Option<&'static str> {
    match event_type {
        "complete" => Some("Windows Notify System Generic.wav"),
        "error" => Some("Windows Critical Stop.wav"),
        "attention" => Some("Windows Notify Calendar.wav"),
        "stop" => Some("Windows Notify Email.wav"),
        "session-end" => Some("Windows Logoff Sound.wav"),
        _ => None,
    }
}

/// Get the full path to a sound file, checking system dir then bundled fallback
pub fn get_sound_path(event_type: &str) -> Option<PathBuf> {
    let sound_name = get_system_sound_name(event_type)?;

    // Try Windows system sounds directory
    let system_path = PathBuf::from(r"C:\Windows\Media").join(sound_name);
    if system_path.exists() {
        return Some(system_path);
    }

    // Try alternative Windows sound names (some versions differ)
    let alt_names = get_alternative_sound_names(event_type);
    for alt_name in alt_names {
        let alt_path = PathBuf::from(r"C:\Windows\Media").join(alt_name);
        if alt_path.exists() {
            return Some(alt_path);
        }
    }

    // Fallback: bundled sound (if exists)
    // These would be in src-tauri/sounds/ and resolved via Tauri's resource dir
    warn!(
        "System sound '{}' not found for event '{}', no fallback available",
        sound_name, event_type
    );
    None
}

/// Alternative sound file names for different Windows versions
fn get_alternative_sound_names(event_type: &str) -> Vec<&'static str> {
    match event_type {
        "complete" => vec!["notify.wav", "tada.wav", "Windows Notify.wav"],
        "error" => vec!["Windows Error.wav", "Windows Hardware Fail.wav", "chord.wav"],
        "attention" => vec!["Windows Notify.wav", "Windows Balloon.wav", "ding.wav"],
        "stop" => vec!["Windows Information Bar.wav", "Windows Unlock.wav"],
        "session-end" => vec!["Windows Shutdown.wav", "Windows Logoff.wav"],
        _ => vec![],
    }
}

/// Play a sound for the given event type (non-blocking)
pub fn play_event_sound(event_type: &str) {
    let path = match get_sound_path(event_type) {
        Some(p) => p,
        None => {
            warn!("No sound file found for event: {}", event_type);
            return;
        }
    };

    info!("Playing sound for '{}': {:?}", event_type, path);

    // Spawn a thread for non-blocking playback
    std::thread::spawn(move || {
        if let Err(e) = play_wav_file(&path) {
            error!("Failed to play sound {:?}: {}", path, e);
        }
    });
}

/// Play a .wav file using rodio
fn play_wav_file(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let (_stream, stream_handle) = rodio::OutputStream::try_default()?;
    let file = File::open(path)?;
    let source = rodio::Decoder::new(BufReader::new(file))?;
    let sink = rodio::Sink::try_new(&stream_handle)?;
    sink.append(source);
    sink.sleep_until_end(); // Block this thread until done
    Ok(())
}
```

---

### Step 2: Integrate Sound with HTTP Handlers

#### 2.1: Update server.rs Event Handlers

- [ ] Modify each handler in `src-tauri/src/server.rs` to also play sound:

```rust
use crate::sounds;

fn emit_event(state: &AppState, event_type: &str) {
    let payload = ClippyEvent {
        event_type: event_type.to_string(),
    };
    info!("Event received: {}", event_type);

    // Play notification sound (non-blocking — spawns a thread)
    sounds::play_event_sound(event_type);

    // Emit to webview for Clippy animation
    if let Err(e) = state.app_handle.emit("clippy-event", &payload) {
        warn!("Failed to emit clippy-event: {}", e);
    }
}
```

---

### Step 3: Update main.rs Module Declarations

#### 3.1: Add sounds Module

- [ ] Update `src-tauri/src/main.rs`:

```rust
mod events;
mod server;
mod sounds;
```

---

### Step 4: Verify Sound Playback

#### 4.1: Test Each Event Sound

- [ ] Start the Tauri app with `cargo tauri dev`
- [ ] Test each event with curl:

```bash
curl -s http://localhost:9999/complete   # Pleasant chime
curl -s http://localhost:9999/error      # Attention-grabbing
curl -s http://localhost:9999/attention  # Gentle notification
curl -s http://localhost:9999/stop       # Neutral tone
curl -s http://localhost:9999/session-end # Wrap-up sound
```

- [ ] Verify: each event plays a distinct sound AND shows Clippy animation

#### 4.2: Test Sound Independence

- [ ] Verify sound plays even if Clippy animation fails
- [ ] Verify Clippy animates even if sound fails (e.g., no audio device)
- [ ] Send rapid events — sounds should overlap or queue, not crash

#### 4.3: Test Error Handling

- [ ] Temporarily rename a sound file — verify graceful handling (logged warning, no crash)
- [ ] Disconnect audio device — verify app continues running

---

## Verifiable Acceptance Criteria

**Critical Path:**

- [ ] All 5 event types play a distinct notification sound
- [ ] Sounds play non-blocking (HTTP response returns before sound finishes)
- [ ] Missing sound files are handled gracefully (warning logged, no crash)
- [ ] Audio device unavailable doesn't crash the app
- [ ] Sound and Clippy animation play simultaneously

**Quality Gates:**

- [ ] Sound starts within 500ms of event receipt
- [ ] No audio artifacts (clicks, pops, distortion)
- [ ] Each event's sound is semantically appropriate (error sounds alarming, complete sounds positive)

**Integration:**

- [ ] Works alongside Phase 04 Clippy animations
- [ ] Phase 07 can add a mute toggle

---

## Quality Assurance

### Test Plan

#### Manual Testing

- [ ] **Complete sound:** `curl http://localhost:9999/complete`
  - Expected: Pleasant chime plays, Clippy animates simultaneously
  - Actual: [To be filled]

- [ ] **Error sound:** `curl http://localhost:9999/error`
  - Expected: Attention-grabbing sound plays
  - Actual: [To be filled]

- [ ] **Rapid fire:** Send 3 events within 1 second
  - Expected: Sounds overlap or queue, no crash
  - Actual: [To be filled]

- [ ] **No audio device:** Disconnect audio, send event
  - Expected: Error logged, app continues, Clippy still animates
  - Actual: [To be filled]

#### Automated Testing

```bash
cd src-tauri && cargo test
```

### Review Checklist

- [ ] **Code Quality:**
  - [ ] `cargo test` passes
  - [ ] Sound playback on separate thread (non-blocking)
  - [ ] Proper error handling with logging

- [ ] **Security:**
  - [ ] Sound files loaded from known paths only (no user-controlled paths)
  - [ ] No arbitrary file access

---

## Dependencies

### Upstream (Required Before Starting)

- Phase 05: HTTP server with event handlers
- rodio crate in Cargo.toml (Phase 01)
- Windows machine with `C:\Windows\Media\` sounds

### Downstream (Will Use This Phase)

- Phase 07: System tray mute toggle
- Phase 08: Final integration testing

### External Services

- None — all local filesystem

---

## Completion Gate

### Sign-off

- [ ] All acceptance criteria met
- [ ] All Rust tests passing
- [ ] All 5 events play correct sounds
- [ ] Code review passed
- [ ] Phase marked DONE in plan.md
- [ ] Committed: `feat(sounds): phase 06 — Windows sound playback via rodio`

---

## Notes

### Technical Considerations

- rodio's `OutputStream` must stay alive while sound plays — that's why we use `sleep_until_end()` on a spawned thread
- The `_stream` variable must be kept alive (not dropped) or audio stops — Rust's RAII pattern handles this in the function scope
- Windows `.wav` files in `C:\Windows\Media\` are PCM encoded — rodio handles these natively

### Known Limitations

- No volume control in MVP — uses system volume
- No Bluetooth keepalive trick — first sound after silence may have ~1-2s delay
- Sound file names are hardcoded — different Windows versions may have different files

### Future Enhancements

- Bluetooth audio keepalive (play silent clip every 30 seconds)
- App-specific volume control via tray menu
- Custom sound file upload/selection
- Sound pack support (choose between different sound themes)

---

**Previous:** [[phase-05-http-server|Phase 05: HTTP Event Server]]
**Next:** [[phase-07-system-tray|Phase 07: System Tray Integration]]
