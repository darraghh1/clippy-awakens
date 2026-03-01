"""
Clippy Awakens notification utility for Claude Code hooks.

Sends HTTP requests to the Clippy Awakens app running on Windows,
reached via SSH reverse tunnel (-R 9999:localhost:9999).

Usage from any hook:
    from notify import notify, message
    notify("complete")                    # task finished
    notify("error")                       # something failed
    notify("attention")                   # needs user input
    notify("stop")                        # process stopped
    notify("session-end")                 # session ending
    message("Found the bug!")             # custom speech bubble

Fails silently if Clippy isn't reachable (no tunnel, app not running).
Never blocks the hook -- fires and forgets.
"""

import subprocess
import urllib.parse

CLIPPY_URL = "http://localhost:9999"

VALID_EVENTS = frozenset({
    "complete",
    "error",
    "attention",
    "stop",
    "session-end",
})


def notify(event: str) -> None:
    """
    Send a notification event to Clippy.

    Fires curl in the background with a short timeout.
    Never raises -- hooks must not fail due to notifications.
    """
    if event not in VALID_EVENTS:
        return

    try:
        subprocess.Popen(
            ["curl", "-s", "-m", "2", f"{CLIPPY_URL}/{event}"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
    except (FileNotFoundError, OSError):
        pass


def message(text: str) -> None:
    """
    Send a custom message through Clippy's speech bubble.

    Clippy pops up, plays a random animation, and speaks the text.
    """
    if not text:
        return

    encoded = urllib.parse.quote(text)
    try:
        subprocess.Popen(
            ["curl", "-s", "-m", "2", f"{CLIPPY_URL}/message?text={encoded}"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
    except (FileNotFoundError, OSError):
        pass
