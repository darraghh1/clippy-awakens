---
title: "Phase 04: Event Animation Mapping & Speech Bubbles"
description: "Map Claude Code hook events to Clippy animations with randomized witty speech bubbles. Create the JavaScript event bridge that listens for Tauri events and drives the clippy.js API."
skill: none
status: pending
group: "clippy-engine"
dependencies: ["phase-03"]
tags: [phase, clippy, events, animations, speech, personality]
created: 2026-02-28
updated: 2026-02-28
---

# Phase 04: Event Animation Mapping & Speech Bubbles

**Context:** [Master Plan](./plan.md) | **Dependencies:** P03 | **Status:** Pending

---

## Overview

Create the JavaScript event bridge (`clippy-bridge.js`) that listens for Tauri events emitted from the Rust backend and translates them into Clippy animations and witty speech bubbles. Each hook event type (`complete`, `error`, `attention`, `stop`, `session-end`) maps to specific animations and draws from a randomized pool of sarcastic/helpful remarks.

**Goal:** When a Tauri event is emitted (e.g., `clippy-event` with type `complete`), Clippy plays the appropriate animation and speaks a randomly selected witty remark.

---

## Context & Workflow

### How This Phase Fits Into the Project

- **UI Layer:** Creates `ui/clippy-bridge.js` — the event-to-animation mapping layer
  - Listens for `clippy-event` Tauri events
  - Maps event types to animation names and speech bubble text pools
  - Manages Clippy show/hide lifecycle (pop up on event, retreat after timeout)

- **Server Layer:** No Rust changes — this phase is frontend-only
  - Phase 05 will emit the events this phase listens for

- **Database Layer:** N/A

- **Integrations:** Tauri's JS event API (`window.__TAURI__.event.listen`)

### User Workflow

**Trigger:** A Claude Code hook fires, sending `curl GET localhost:9999/complete` (via Phase 05's HTTP server).

**Steps:**
1. Rust HTTP handler receives the request (Phase 05)
2. Rust emits `clippy-event` with `{ type: "complete" }` payload
3. JS bridge receives the event
4. Bridge looks up `complete` → animation: `Congratulate`, speech pool
5. If Clippy is hidden, show Clippy first
6. Play animation, then speak a random witty remark
7. After timeout, hide Clippy (return to tray)

**Success Outcome:** Clippy pops up with a celebration animation and says something like "Ship it! Another masterpiece deployed to production."

### Problem Being Solved

**Pain Point:** Without this mapping layer, events from the HTTP server have no way to trigger Clippy behavior.
**Alternative Approach:** Could hardcode animations in Rust, but JS is the right layer since clippy.js API is JavaScript.

### Integration Points

**Upstream Dependencies:**
- Phase 03: `currentAgent` global with `.play()`, `.speak()`, `.show()`, `.hide()` methods

**Downstream Consumers:**
- Phase 05: HTTP server emits the events this bridge consumes
- Phase 07: System tray may trigger manual show/hide

**Data Flow:**
```
Tauri event: { type: "complete" }
  → clippy-bridge.js receives event
  → Look up event config: { animation: "Congratulate", speeches: [...] }
  → Pick random speech from pool
  → agent.show() if hidden
  → agent.play("Congratulate")
  → agent.speak("Ship it! Another masterpiece.")
  → setTimeout → agent.hide() after 8 seconds
```

---

## Prerequisites & Clarifications

### Questions for User

1. **Personality Tone:** How sarcastic vs helpful should the remarks be?
   - **Context:** Classic Clippy was earnestly unhelpful. Dev humor Clippy would be more self-aware and snarky.
   - **Assumptions if unanswered:** Mix of both — 60% sarcastic dev humor, 40% classic Clippy pastiche. Always friendly, never mean.
   - **Impact:** Affects the speech bubble text pools

2. **Hide Timer Duration:** How long should Clippy stay visible after an event?
   - **Context:** Too short and you miss the animation. Too long and Clippy gets annoying.
   - **Assumptions if unanswered:** 8 seconds after speech completes. User can dismiss earlier by double-clicking.
   - **Impact:** Affects the auto-hide timeout value

3. **Stacking Events:** What happens if multiple events fire rapidly?
   - **Context:** clippy.js has an internal queue — calling `.play()` and `.speak()` multiple times queues them sequentially.
   - **Assumptions if unanswered:** Queue events naturally. Reset the hide timer on each new event. Don't drop events.
   - **Impact:** Rapid events could keep Clippy visible for a long time — acceptable.

### Validation Checklist

- [ ] Phase 03 completed — Clippy loads and animates in webview
- [ ] `currentAgent` global is accessible
- [ ] Tauri JS API available (`window.__TAURI__`)

---

## Requirements

### Functional

- Each event type maps to at least one animation and a pool of 5+ speech bubble texts
- Speech bubbles are randomly selected from the pool (no repeats until pool exhausted)
- Clippy shows automatically when an event arrives (if hidden)
- Clippy hides automatically after a configurable timeout
- Events queue naturally — rapid events don't crash or overlap badly

### Technical

- `ui/clippy-bridge.js` created as a standalone module
- Uses Tauri's `window.__TAURI__.event.listen()` for event subscription
- No modifications to existing clippy.js source
- Event type validation — unknown events logged but don't crash
- Configurable timeout duration

---

## Decision Log

### Randomized Speech Without Repeats (ADR-04-01)

**Date:** 2026-02-28
**Status:** Accepted

**Context:** Showing the same speech bubble every time gets stale fast. Need variety without full randomness (which could repeat).

**Decision:** Use a Fisher-Yates shuffle of the speech pool. Cycle through the shuffled array, reshuffle when exhausted.

**Consequences:**
- **Positive:** Every remark gets shown before any repeats, feels fresh
- **Negative:** Slightly more code than pure random
- **Neutral:** Pool size of 5-8 per event type is sufficient

---

## Implementation Steps

### Step 0: Test Definition (TDD)

**Purpose:** Define tests for event mapping logic before implementation

#### 0.1: Unit Tests for Event Mapping

- [ ] Create `ui/clippy-bridge.test.js` (can be run in browser console or with a test runner):

```javascript
// Test suite for clippy-bridge.js
function testEventMapping() {
    var results = [];

    // Test 1: All event types have mappings
    var eventTypes = ['complete', 'error', 'attention', 'stop', 'session-end'];
    eventTypes.forEach(function(type) {
        var config = ClippyBridge.getEventConfig(type);
        results.push({
            test: 'Event "' + type + '" has config',
            pass: !!config
        });
        results.push({
            test: 'Event "' + type + '" has animation',
            pass: config && !!config.animation
        });
        results.push({
            test: 'Event "' + type + '" has speeches (5+)',
            pass: config && config.speeches && config.speeches.length >= 5
        });
    });

    // Test 2: Unknown event returns null
    results.push({
        test: 'Unknown event returns null',
        pass: ClippyBridge.getEventConfig('unknown') === null
    });

    // Test 3: Random speech returns a string
    var speech = ClippyBridge.getRandomSpeech('complete');
    results.push({
        test: 'Random speech returns string',
        pass: typeof speech === 'string' && speech.length > 0
    });

    // Log results
    console.log('=== ClippyBridge Tests ===');
    results.forEach(function(r) {
        console.log((r.pass ? 'PASS' : 'FAIL') + ': ' + r.test);
    });
    var passed = results.filter(function(r) { return r.pass; }).length;
    console.log('Result: ' + passed + '/' + results.length + ' passed');
    return results;
}
```

---

### Step 1: Create Event-to-Animation Mapping

#### 1.1: Define Event Configuration

- [ ] Create `ui/clippy-bridge.js` with event mapping:

```javascript
var ClippyBridge = (function() {
    'use strict';

    // How long Clippy stays visible after an event (ms)
    var HIDE_TIMEOUT = 8000;
    var _hideTimer = null;
    var _agent = null;
    var _isVisible = false;

    // Shuffled speech pools — tracks position to avoid repeats
    var _speechIndex = {};

    // Event configurations
    var EVENT_CONFIG = {
        'complete': {
            animation: 'Congratulate',
            fallbackAnimation: 'GetTechy',
            speeches: [
                "Ship it! Another masterpiece deployed to production.",
                "Task complete! I'd high-five you but... you know... paperclip.",
                "It looks like you've finished a task. Would you like me to take credit?",
                "Done! That was mass-produce-able. I mean impressive.",
                "Compiling success.exe... no errors found. I'm as shocked as you are.",
                "Achievement unlocked: Actually Finished Something.",
                "Your code compiled without errors. Is this a simulation?",
                "Task complete! Time to mass-reply 'per my last email' to everyone."
            ]
        },
        'error': {
            animation: 'Alert',
            fallbackAnimation: 'GetAttention',
            speeches: [
                "It looks like you're experiencing a crash. Would you like to cry?",
                "Error detected! Have you tried mass-deleting... I mean debugging?",
                "Something broke. But hey, at least it's not in production. ...right?",
                "I see you've chosen the path of error. Bold strategy.",
                "Oops! That's what we in the biz call a 'learning opportunity.'",
                "Error! Even I know that's not how you code that.",
                "Houston, we have a problem. And by Houston, I mean you.",
                "It looks like your code is throwing a tantrum."
            ]
        },
        'attention': {
            animation: 'GetAttention',
            fallbackAnimation: 'Thinking',
            speeches: [
                "Hey! I need your attention. Yes, YOU. Stop scrolling.",
                "It looks like Claude needs your input. I'm just the messenger.",
                "Attention required! And no, I can't answer it for you.",
                "Claude is waiting. You know, like I waited in that drawer for 20 years.",
                "Psst! Your AI assistant needs you. The irony is not lost on me.",
                "Input needed! I'd help, but I'm literally a paperclip.",
                "Claude needs you! Don't leave it hanging like Microsoft left me."
            ]
        },
        'stop': {
            animation: 'Wave',
            fallbackAnimation: 'GoodBye',
            speeches: [
                "Claude has stopped. Time to pretend you were productive.",
                "Process complete! Or was it? I can never tell with you.",
                "Claude stopped. Back to staring at the screen, I see.",
                "Done! You can stop holding your breath now.",
                "Claude has finished. Resume your regularly scheduled procrastination.",
                "All done! Want me to file that under 'things that actually worked'?"
            ]
        },
        'session-end': {
            animation: 'GoodBye',
            fallbackAnimation: 'Wave',
            speeches: [
                "Session over! Don't forget to save. Oh wait, wrong decade.",
                "Goodbye! I'll just be here... in the system tray... alone... again.",
                "Session ended. It looks like you're leaving. Would you like me to be sad?",
                "See you next time! I'll be here. I'm always here.",
                "Session complete! Remember: Ctrl+S early, Ctrl+S often. Trust me on this.",
                "Farewell! May your merge conflicts be few and your PRs be approved."
            ]
        }
    };

    // Shuffle array in place (Fisher-Yates)
    function shuffle(arr) {
        for (var i = arr.length - 1; i > 0; i--) {
            var j = Math.floor(Math.random() * (i + 1));
            var temp = arr[i];
            arr[i] = arr[j];
            arr[j] = temp;
        }
        return arr;
    }

    function getEventConfig(eventType) {
        return EVENT_CONFIG[eventType] || null;
    }

    function getRandomSpeech(eventType) {
        var config = EVENT_CONFIG[eventType];
        if (!config || !config.speeches) return null;

        // Initialize or reset shuffle index
        if (!_speechIndex[eventType] || _speechIndex[eventType].index >= _speechIndex[eventType].order.length) {
            _speechIndex[eventType] = {
                order: shuffle(config.speeches.slice()),
                index: 0
            };
        }

        var speech = _speechIndex[eventType].order[_speechIndex[eventType].index];
        _speechIndex[eventType].index++;
        return speech;
    }

    function handleEvent(eventType) {
        var config = getEventConfig(eventType);
        if (!config) {
            console.warn('ClippyBridge: Unknown event type:', eventType);
            return;
        }

        if (!_agent) {
            console.error('ClippyBridge: No agent loaded');
            return;
        }

        // Clear any existing hide timer
        if (_hideTimer) {
            clearTimeout(_hideTimer);
            _hideTimer = null;
        }

        // Show Clippy if hidden
        if (!_isVisible) {
            _agent.show();
            _isVisible = true;
        }

        // Play animation (use fallback if primary not available)
        var animation = config.animation;
        if (!_agent.hasAnimation(animation)) {
            animation = config.fallbackAnimation;
        }
        _agent.play(animation);

        // Speak a random remark
        var speech = getRandomSpeech(eventType);
        if (speech) {
            _agent.speak(speech);
        }

        // Set auto-hide timer
        _hideTimer = setTimeout(function() {
            _agent.play('GoodBye');
            setTimeout(function() {
                _agent.hide();
                _isVisible = false;
            }, 2000);
        }, HIDE_TIMEOUT);
    }

    function init(agent) {
        _agent = agent;
        console.log('ClippyBridge: Initialized with agent');
    }

    // Public API
    return {
        init: init,
        handleEvent: handleEvent,
        getEventConfig: getEventConfig,
        getRandomSpeech: getRandomSpeech,
        HIDE_TIMEOUT: HIDE_TIMEOUT
    };
})();
```

---

### Step 2: Wire Up Tauri Event Listener

#### 2.1: Add Tauri Event Listener to ui/index.html

- [ ] Update `ui/index.html` to include the bridge and Tauri event listener:

```html
<script src="clippy-bridge.js"></script>
<script>
    // After Clippy loads, initialize the bridge and listen for events
    clippy.load('Clippy', function(agent) {
        currentAgent = agent;
        agent.show();

        // Initialize the event bridge
        ClippyBridge.init(agent);

        // Listen for events from Rust backend
        if (window.__TAURI__) {
            window.__TAURI__.event.listen('clippy-event', function(event) {
                console.log('ClippyBridge: Received event:', event.payload);
                var eventType = event.payload.type || event.payload;
                ClippyBridge.handleEvent(eventType);
            });
            console.log('ClippyBridge: Tauri event listener registered');
        } else {
            console.warn('ClippyBridge: Tauri API not available (running in browser?)');
        }

        // Initial greeting
        agent.speak('Clippy Awakens! I\'m listening for Claude Code events.');

        // Run self-test
        clippySelfTest(agent);
    }, function(err) {
        console.error('Failed to load Clippy:', err);
    });
</script>
```

---

### Step 3: Add Manual Test Triggers

#### 3.1: Create Debug Console Functions

- [ ] Add debug functions to `ui/index.html` for manual testing:

```javascript
// Debug: Trigger events manually from browser console
window.testEvent = function(type) {
    ClippyBridge.handleEvent(type || 'complete');
};

window.testAllEvents = function() {
    var events = ['complete', 'error', 'attention', 'stop', 'session-end'];
    var delay = 0;
    events.forEach(function(type) {
        setTimeout(function() {
            console.log('Testing event:', type);
            ClippyBridge.handleEvent(type);
        }, delay);
        delay += 10000; // 10 seconds between each
    });
};
```

---

### Step 4: Verify Event Handling

#### 4.1: Test Each Event Type

- [ ] Open Tauri dev tools console
- [ ] Call `testEvent('complete')` — Clippy plays Congratulate + speaks
- [ ] Call `testEvent('error')` — Clippy plays Alert + speaks
- [ ] Call `testEvent('attention')` — Clippy plays GetAttention + speaks
- [ ] Call `testEvent('stop')` — Clippy plays Wave + speaks
- [ ] Call `testEvent('session-end')` — Clippy plays GoodBye + speaks

#### 4.2: Test Auto-Hide

- [ ] Trigger an event, wait 8+ seconds
- [ ] Clippy should play GoodBye and hide
- [ ] Trigger another event — Clippy should reappear

#### 4.3: Test Event Queueing

- [ ] Call `testEvent('complete')` immediately followed by `testEvent('error')`
- [ ] Both animations should play sequentially (clippy.js queue handles this)

#### 4.4: Run Unit Tests

- [ ] Call `testEventMapping()` in console
- [ ] All tests should pass

---

## Verifiable Acceptance Criteria

**Critical Path:**

- [ ] All 5 event types (`complete`, `error`, `attention`, `stop`, `session-end`) trigger correct animations
- [ ] Each event type has 5+ unique speech bubble texts
- [ ] Speech bubbles rotate without immediate repeats (shuffle behavior)
- [ ] Clippy auto-shows on event and auto-hides after timeout
- [ ] Unknown event types are logged but don't crash

**Quality Gates:**

- [ ] Speech bubble text is witty and appropriate (mix of sarcastic + classic Clippy)
- [ ] No JavaScript errors in console during normal event flow
- [ ] Animation + speech sequence completes without visual glitches

**Integration:**

- [ ] `ClippyBridge.handleEvent(type)` can be called from Tauri event listener
- [ ] Phase 05 can emit events that this bridge consumes

---

## Quality Assurance

### Test Plan

#### Manual Testing

- [ ] **Complete event:** `testEvent('complete')` in console
  - Expected: Congratulate animation + witty completion remark
  - Actual: [To be filled]

- [ ] **Error event:** `testEvent('error')` in console
  - Expected: Alert animation + witty error remark
  - Actual: [To be filled]

- [ ] **Speech variety:** Trigger `complete` 8+ times
  - Expected: All 8 speeches appear before any repeat
  - Actual: [To be filled]

- [ ] **Auto-hide:** Trigger event, wait 10 seconds
  - Expected: Clippy plays GoodBye and hides
  - Actual: [To be filled]

- [ ] **Rapid events:** `testEvent('complete'); testEvent('error');`
  - Expected: Both play sequentially, no crash
  - Actual: [To be filled]

#### Automated Testing

```bash
# Rust tests still pass
cd src-tauri && cargo test
```

### Review Checklist

- [ ] **Code Quality:**
  - [ ] ClippyBridge is a clean IIFE module
  - [ ] No global variable pollution beyond `ClippyBridge`
  - [ ] Speech texts are grammatically correct and funny

- [ ] **Security:**
  - [ ] No user-supplied content in speech bubbles
  - [ ] Event types validated before processing

---

## Dependencies

### Upstream (Required Before Starting)

- Phase 03: Clippy agent loaded with working `.play()`, `.speak()`, `.show()`, `.hide()`
- Tauri JS API (`window.__TAURI__`) available in webview

### Downstream (Will Use This Phase)

- Phase 05: HTTP server emits `clippy-event` that this bridge listens for
- Phase 07: System tray may trigger show/hide alongside this bridge

### External Services

- None

---

## Completion Gate

### Sign-off

- [ ] All acceptance criteria met
- [ ] All unit tests pass (testEventMapping)
- [ ] Code review passed
- [ ] Phase marked DONE in plan.md
- [ ] Committed: `feat(clippy): phase 04 — event animation mapping & speech bubbles`

---

## Notes

### Technical Considerations

- clippy.js's internal queue handles rapid `.play()` and `.speak()` calls gracefully
- The `$.Deferred` pattern in clippy.js means animations resolve asynchronously — the bridge doesn't need to manage timing
- Speech bubble WORD_SPEAK_TIME is 200ms per word — a 10-word speech takes ~2 seconds

### Known Limitations

- No persistent speech history — past remarks aren't saved
- Hide timeout resets on each new event — rapid events keep Clippy visible
- Only the 'Clippy' agent is supported (other agents have different animation names)

### Future Enhancements

- User-configurable speech pools (add custom remarks)
- Adjustable hide timeout
- Event-specific sounds (handled in Phase 06, separate from speech)
- Context-aware remarks (e.g., different messages for first event vs tenth)

---

**Previous:** [[phase-03-clippy-webview|Phase 03: Clippy.js WebView Integration]]
**Next:** [[phase-05-http-server|Phase 05: HTTP Event Server]]
