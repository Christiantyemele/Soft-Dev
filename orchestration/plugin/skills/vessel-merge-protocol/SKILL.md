---
name: merge-protocol
description: Protocol for safely merging approved PRs
---

# Merge Protocol Skill

## Pre-Merge Checklist

Before merging, verify:
- [ ] CI status: success
- [ ] SENTINEL approval: final-review.md exists with APPROVED
- [ ] No merge conflicts
- [ ] Branch up to date with main (or rebased)

## Merge Process

### Step 1: Fetch Latest
```bash
git fetch origin main
```

### Step 2: Check Divergence
```bash
BEHIND=$(git rev-list --count HEAD..origin/main)
if [ "$BEHIND" -gt 0 ]; then
  # Need rebase or merge
fi
```

### Step 3: Merge
Use `merge_pr` MCP tool with:
- Merge method: `squash` (recommended) or `merge`
- Commit message: From SENTINEL's PR description

### Step 4: Verify
- Confirm merge completed
- Confirm branch deleted (optional)

### Step 5: Sync Active Worktrees

After a successful merge into main, **all active FORGE worktrees MUST be rebased**
onto the updated main to prevent drift and merge conflicts.

**Policy:** No branch may fall behind main. All branches must be at main or ahead of main.

```bash
# Fetch the latest main (now including the merge we just performed)
git fetch origin main

# Sync every active worktree
for wt in $(git worktree list --porcelain | grep "^worktree" | cut -d' ' -f2); do
  BRANCH=$(git -C "$wt" branch --show-current)
  if [ "$BRANCH" != "main" ] && [ -n "$BRANCH" ]; then
    BEHIND=$(git -C "$wt" rev-list --count "HEAD..origin/main" 2>/dev/null || echo "0")
    if [ "$BEHIND" -gt 0 ]; then
      echo "Syncing worktree $wt ($BRANCH) — $BEHIND commits behind main"
      git -C "$wt" rebase origin/main
      if [ $? -ne 0 ]; then
        git -C "$wt" rebase --abort
        echo "CONFLICT in $wt on branch $BRANCH — requires manual resolution"
        # Report conflict to NEXUS
      fi
    else
      echo "Worktree $wt ($BRANCH) is up to date"
    fi
  fi
done
```

Alternatively, use the dedicated sync script:
```bash
bash scripts/sync-worktrees.sh
```

**Conflict handling:**
- If rebase conflicts, **abort immediately** — do NOT attempt auto-resolution
- Report the conflict to NEXUS with the worktree path and branch name
- Set the affected worker's STATUS.json to `BLOCKED` with reason `REBASE_CONFLICT`
- A human or fresh FORGE instance must resolve the conflict

### Step 6: Report
- Emit merge event
- Update shared store with merge status
- Include worktree sync results in the report

## Merge Methods

| Method | Use When |
|--------|----------|
| `squash` | Single logical change (recommended) |
| `merge` | Multiple commits should be preserved |
| `rebase` | Linear history preferred |

## Post-Merge

1. **Sync**: Rebase all active worktrees onto updated main (Step 5)
2. **Cleanup**: Remove worktree for the merged branch only
3. **Notify**: Emit event for NEXUS (include sync results)
4. **Update**: Set worker slot to Idle

## Failure Handling

If merge fails:
1. Check for conflicts
2. Report to NEXUS with `deploy_failed`
3. Do NOT attempt to resolve conflicts automatically

If worktree sync fails (rebase conflict):
1. Abort the rebase immediately
2. Report to NEXUS with worktree path and branch name
3. Set affected worker STATUS.json to `BLOCKED` with reason `REBASE_CONFLICT`
4. Do NOT attempt to resolve conflicts automatically — assign a fresh FORGE or human
