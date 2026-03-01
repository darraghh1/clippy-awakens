#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///

"""
Stop hook -- Clippy celebrates when Claude finishes a task.

Triggers the /complete event which plays a congratulatory animation
and a witty remark about the completed work.
"""

import json
import sys
from pathlib import Path

# Add this directory to path so notify.py can be imported
sys.path.insert(0, str(Path(__file__).parent))
from notify import notify


def main():
    try:
        input_data = json.load(sys.stdin)

        # Prevent infinite loops if stop_hook_active
        if input_data.get("stop_hook_active", False):
            sys.exit(0)

        notify("complete")
    except Exception:
        pass
    sys.exit(0)


if __name__ == "__main__":
    main()
