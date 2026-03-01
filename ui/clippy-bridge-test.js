/**
 * ClippyBridge Test Suite — runs in browser console.
 * Call testEventMapping() after ClippyBridge is loaded.
 */
function testEventMapping() {
    var results = [];

    // Test 1: All event types have mappings with animation and 5+ speeches
    var eventTypes = ['complete', 'error', 'attention', 'stop', 'session-end'];
    eventTypes.forEach(function(type) {
        var config = ClippyBridge.getEventConfig(type);
        results.push({
            test: 'Event "' + type + '" has config',
            pass: !!config
        });
        results.push({
            test: 'Event "' + type + '" has animation',
            pass: config && typeof config.animation === 'string' && config.animation.length > 0
        });
        results.push({
            test: 'Event "' + type + '" has fallback animation',
            pass: config && typeof config.fallbackAnimation === 'string' && config.fallbackAnimation.length > 0
        });
        results.push({
            test: 'Event "' + type + '" has 5+ speeches',
            pass: config && config.speeches && config.speeches.length >= 5
        });
    });

    // Test 2: Unknown event returns null
    results.push({
        test: 'Unknown event returns null',
        pass: ClippyBridge.getEventConfig('unknown') === null
    });

    // Test 3: getRandomSpeech returns a string for known type
    var speech = ClippyBridge.getRandomSpeech('complete');
    results.push({
        test: 'getRandomSpeech returns string for known type',
        pass: typeof speech === 'string' && speech.length > 0
    });

    // Test 4: getRandomSpeech returns null for unknown type
    results.push({
        test: 'getRandomSpeech returns null for unknown type',
        pass: ClippyBridge.getRandomSpeech('nonexistent') === null
    });

    // Test 5: Fisher-Yates exhausts pool before repeating
    var config = ClippyBridge.getEventConfig('stop');
    var poolSize = config.speeches.length;
    var seen = {};
    var allUnique = true;
    for (var i = 0; i < poolSize; i++) {
        var s = ClippyBridge.getRandomSpeech('stop');
        if (seen[s]) {
            allUnique = false;
            break;
        }
        seen[s] = true;
    }
    results.push({
        test: 'Fisher-Yates: ' + poolSize + ' speeches without repeat',
        pass: allUnique
    });

    // Test 6: After exhausting pool, reshuffle works (next call returns a string)
    var afterReshuffle = ClippyBridge.getRandomSpeech('stop');
    results.push({
        test: 'After pool exhaustion, reshuffle works',
        pass: typeof afterReshuffle === 'string' && afterReshuffle.length > 0
    });

    // Test 7: HIDE_TIMEOUT is a positive number
    results.push({
        test: 'HIDE_TIMEOUT is positive number',
        pass: typeof ClippyBridge.HIDE_TIMEOUT === 'number' && ClippyBridge.HIDE_TIMEOUT > 0
    });

    // Test 8: Total speeches across all types >= 40
    var totalSpeeches = 0;
    eventTypes.forEach(function(type) {
        var c = ClippyBridge.getEventConfig(type);
        if (c && c.speeches) totalSpeeches += c.speeches.length;
    });
    results.push({
        test: 'Total speeches >= 40 (got ' + totalSpeeches + ')',
        pass: totalSpeeches >= 40
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
