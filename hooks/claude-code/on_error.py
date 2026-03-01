#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///

"""
Tool failure hook -- Clippy reacts when Claude hits an error.

Triggers the /error event on tool use failures, which plays
an alert animation with a snarky error-related remark.
"""

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from notify import notify


def main():
    try:
        json.loads(sys.stdin.read())
        notify("error")
    except Exception:
        pass
    sys.exit(0)


if __name__ == "__main__":
    main()
