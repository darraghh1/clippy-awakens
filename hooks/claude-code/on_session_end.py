#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///

"""
Session end hook -- Clippy waves goodbye when a session ends.

Triggers the /session-end event which plays a farewell animation.
"""

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from notify import notify


def main():
    try:
        json.loads(sys.stdin.read())
        notify("session-end")
    except Exception:
        pass
    sys.exit(0)


if __name__ == "__main__":
    main()
