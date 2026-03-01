/**
 * ClippyBridge — Event-to-Animation Mapping & Speech Bubbles
 *
 * Listens for Tauri events and drives the clippy.js API.
 * Each event type maps to an animation + pool of witty speech bubbles.
 * Speech pools use Fisher-Yates shuffle to avoid repeats.
 */
var ClippyBridge = (function() {
    'use strict';

    // How long Clippy stays visible after an event (ms)
    var HIDE_TIMEOUT = 8000;
    var _hideTimer = null;
    var _agent = null;
    var _isVisible = false;

    // Shuffled speech pools — tracks position to avoid repeats
    var _speechState = {};

    // Event configurations: animation + fallback + speech pool
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
                "Task complete! Time to mass-reply 'per my last email' to everyone.",
                "Another PR merged. At this rate you might finish before heat death of the universe.",
                "It looks like you're shipping code. Would you like to mass-deploy to prod on a Friday?"
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
                "It looks like your code is throwing a tantrum.",
                "Have you tried turning it off and mass-googling the error?",
                "Stack trace detected. I'd read it for you but I'm just a paperclip."
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
                "Claude needs you! Don't leave it hanging like Microsoft left me.",
                "Excuse me! It looks like you're being asked a question. Would you like to answer it?"
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
                "All done! Want me to file that under 'things that actually worked'?",
                "Claude has left the chat. Just like everyone at my birthday party.",
                "Process terminated. No processes were harmed. Probably."
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
                "Farewell! May your merge conflicts be few and your PRs be approved.",
                "Logging off? I'll keep the lights on. Not like I have a choice.",
                "Session ended! I'll just wait here. In the dark. It's fine. I'm fine."
            ]
        }
    };

    /**
     * Shuffle array in place using Fisher-Yates algorithm.
     */
    function shuffle(arr) {
        for (var i = arr.length - 1; i > 0; i--) {
            var j = Math.floor(Math.random() * (i + 1));
            var temp = arr[i];
            arr[i] = arr[j];
            arr[j] = temp;
        }
        return arr;
    }

    /**
     * Get configuration for an event type.
     * Returns null for unknown event types.
     */
    function getEventConfig(eventType) {
        return EVENT_CONFIG[eventType] || null;
    }

    /**
     * Get a random speech from the pool for the given event type.
     * Uses Fisher-Yates shuffle — exhausts entire pool before reshuffling.
     */
    function getRandomSpeech(eventType) {
        var config = EVENT_CONFIG[eventType];
        if (!config || !config.speeches) return null;

        // Initialize or reset when pool exhausted
        if (!_speechState[eventType] ||
            _speechState[eventType].index >= _speechState[eventType].order.length) {
            _speechState[eventType] = {
                order: shuffle(config.speeches.slice()),
                index: 0
            };
        }

        var state = _speechState[eventType];
        var speech = state.order[state.index];
        state.index++;
        return speech;
    }

    /**
     * Handle an incoming event — show agent, play animation, speak, auto-hide.
     */
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
        if (_agent.hasAnimation && !_agent.hasAnimation(animation)) {
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
            if (_agent) {
                _agent.play('GoodBye');
                setTimeout(function() {
                    if (_agent) {
                        _agent.hide();
                        _isVisible = false;
                    }
                }, 2000);
            }
        }, HIDE_TIMEOUT);
    }

    /**
     * Initialize the bridge with a clippy.js agent.
     */
    function init(agent) {
        _agent = agent;
        _isVisible = true; // Agent is shown at init
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
