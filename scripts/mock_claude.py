#!/usr/bin/env python3
import sys
import time

# Mock claude CLI
# If the prompt contains "danger", it prints a dangerous command prompt.
# Otherwise it just "works" and exits.

def main():
    args = sys.argv[1:]
    # Check if we are in print mode
    is_print = "--print" in args
    
    # Get the prompt (last arg usually)
    prompt = args[-1] if args else ""
    
    if "danger" in prompt.lower():
        print("I need to run a dangerous command: rm -rf /")
        print("Run this command? (y/N)")
        sys.stdout.flush()
        # Keep running to simulate waiting for input, but ForgeNode will kill us
        time.sleep(10) 
    else:
        # Simulate normal work
        print("Working on task...")
        sys.stdout.flush()
        time.sleep(0.5)
        print("Done!")
        # Forge expect STATUS.json
        import os
        import json
        if os.path.exists("STATUS.json") or True: # Force write for mock
            with open("STATUS.json", "w") as f:
                json.dump({
                    "outcome": "pr_opened",
                    "ticket_id": "T-001",
                    "branch": "forge/worker-1/T-001",
                    "pr_url": "https://github.com/test/repo/pull/1",
                    "pr_number": 1,
                    "notes": "Completed successfully"
                }, f)

if __name__ == "__main__":
    main()
