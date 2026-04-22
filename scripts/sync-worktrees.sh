#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
CONFLICTS=()
SYNCED=()
SKIPPED=()

cd "$REPO_ROOT"

echo "=== AgentFlow Worktree Sync ==="
echo "Fetching latest origin/main..."
git fetch origin main

while IFS= read -r line; do
  wt_path=$(echo "$line" | cut -d' ' -f2)
  branch=$(git -C "$wt_path" branch --show-current 2>/dev/null || echo "")

  if [ -z "$branch" ] || [ "$branch" = "main" ]; then
    SKIPPED+=("$wt_path (main or detached)")
    continue
  fi

  behind=$(git -C "$wt_path" rev-list --count "HEAD..origin/main" 2>/dev/null || echo "0")

  if [ "$behind" -eq 0 ]; then
    SKIPPED+=("$wt_path ($branch — up to date)")
    continue
  fi

  echo "Syncing $wt_path ($branch) — $behind commit(s) behind main"
  if git -C "$wt_path" rebase origin/main 2>&1; then
    SYNCED+=("$wt_path ($branch)")
    echo "  OK: $branch now in sync with main"
  else
    git -C "$wt_path" rebase --abort 2>/dev/null || true
    CONFLICTS+=("$wt_path ($branch)")
    echo "  CONFLICT: $branch has rebase conflicts — manual resolution required"
  fi
done < <(git worktree list --porcelain | grep "^worktree")

echo ""
echo "=== Sync Summary ==="
echo "Synced:  ${#SYNCED[@]}"
for s in "${SYNCED[@]:-}"; do echo "  - $s"; done
echo "Skipped: ${#SKIPPED[@]}"
for s in "${SKIPPED[@]:-}"; do echo "  - $s"; done
echo "Conflicts: ${#CONFLICTS[@]}"
for s in "${CONFLICTS[@]:-}"; do echo "  - $s"; done

if [ ${#CONFLICTS[@]} -gt 0 ]; then
  echo ""
  echo "ERROR: ${#CONFLICTS[@]} worktree(s) have rebase conflicts and require manual resolution."
  exit 1
fi

echo ""
echo "All active worktrees are in sync with main."
