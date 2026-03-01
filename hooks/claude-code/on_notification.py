#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///

"""
Notification hook -- Clippy alerts you when Claude needs input.

Only fires on actionable events where you need to respond:
permission prompts (approve/deny) and question dialogs.

Ignores idle_prompt and other non-actionable notification types.
"""

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from notify import notify

# Only trigger Clippy for events that require user action
ACTIONABLE_TYPES = frozenset({
    "permission_prompt",   # approve/deny destructive commands
    "elicitation_dialog",  # AskUserQuestion choices
})


def main():
    try:
        input_data = json.loads(sys.stdin.read())
        event_type = input_data.get("type", "")

        if event_type in ACTIONABLE_TYPES:
            notify("attention")
    except Exception:
        pass
    sys.exit(0)


if __name__ == "__main__":
    main()
