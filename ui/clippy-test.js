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
