// crates/pair-harness/src/worktree.rs
//! Git worktree management for pair isolation.

use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, info, warn};

/// Manages Git worktrees for pair isolation.
pub struct WorktreeManager {
    /// Project root directory (contains .git)
    project_root: PathBuf,
    /// Directory where worktrees are created
    worktrees_dir: PathBuf,
}

impl WorktreeManager {
    /// Create a new worktree manager.
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        let project_root = project_root.into();
        Self {
            worktrees_dir: project_root.join("worktrees"),
            project_root,
        }
    }

    /// Create a worktree for a pair on a new branch.
    ///
    /// # Arguments
    /// * `pair_id` - Pair identifier (e.g., "pair-1")
    /// * `ticket_id` - Ticket identifier (e.g., "T-42")
    ///
    /// # Returns
    /// Path to the created worktree.
    pub fn create_worktree(&self, pair_id: &str, ticket_id: &str) -> Result<PathBuf> {
        let worktree_path = self.worktrees_dir.join(pair_id);
        let branch_name = Self::branch_name(pair_id, ticket_id);

        info!(pair_id, ticket_id, branch = %branch_name, "Creating worktree");

        // Update remote refs for origin/main (best-effort)
        if let Err(e) = self.run_git_in_main(&["fetch", "origin", "main"]) {
            warn!(error = %e, "git fetch origin/main failed, continuing");
        }

        // Keep stale worktree pruning but DO NOT delete existing branches by default.
        self.prune_stale_worktrees();

        // Acquire a per-pair filesystem lock to avoid concurrent creation/reuse races.
        let lock_path = self.worktrees_dir.join(format!("{}.lock", pair_id));
        struct LockGuard(PathBuf);
        impl Drop for LockGuard {
            fn drop(&mut self) {
                let _ = std::fs::remove_file(&self.0);
            }
        }

        let mut lock_acquired = false;
        let mut attempts = 0u8;
        while attempts < 50 {
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&lock_path)
            {
                Ok(_) => {
                    lock_acquired = true;
                    break;
                }
                Err(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    attempts = attempts.saturating_add(1);
                }
            }
        }

        if !lock_acquired {
            return Err(anyhow!("Failed to acquire worktree lock for {} after retries", pair_id));
        }
        let _lock_guard = LockGuard(lock_path.clone());

        std::fs::create_dir_all(&self.worktrees_dir)
            .context("Failed to create worktrees directory")?;

        if worktree_path.exists() {
            info!(path = %worktree_path.display(), "Worktree already exists, reusing");

            // If there are uncommitted changes in the worktree, stash them and keep the stash.
            let status = Command::new("git")
                .args(["status", "--porcelain"])
                .current_dir(&worktree_path)
                .output()
                .context("Failed to run git status")?;

            if !status.stdout.is_empty() {
                info!(path = %worktree_path.display(), "Stashing local changes before switching branches");
                let stash_msg = format!("autostash: reuse for ticket {}", ticket_id);
                let output = Command::new("git")
                    .args(["stash", "push", "-u", "-m"])
                    .arg(&stash_msg)
                    .current_dir(&worktree_path)
                    .output()
                    .context("Failed to run git stash")?;

                if !output.status.success() {
                    warn!(path = %worktree_path.display(), error = %String::from_utf8_lossy(&output.stderr), "git stash failed");
                }
            }

            // Ensure the worktree has recent refs from origin.
            let fetch_out = Command::new("git")
                .args(["fetch", "origin"])
                .current_dir(&worktree_path)
                .output()
                .context("Failed to fetch origin in existing worktree")?;

            if !fetch_out.status.success() {
                warn!(path = %worktree_path.display(), stderr = %String::from_utf8_lossy(&fetch_out.stderr), "git fetch origin in worktree failed");
            }

            // Try to ensure local 'main' branch matches origin/main. If local main exists, checkout and pull.
            // Otherwise, create/update local main from origin/main so we have a stable base to branch from.
            let rev_out = Command::new("git")
                .args(["rev-parse", "--verify", "main"])
                .current_dir(&worktree_path)
                .output()
                .context("Failed to check for local main branch")?;

            if rev_out.status.success() {
                let checkout_out = Command::new("git")
                    .args(["checkout", "main"])
                    .current_dir(&worktree_path)
                    .output()
                    .context("Failed to checkout local main in worktree")?;

                if !checkout_out.status.success() {
                    warn!(path = %worktree_path.display(), stderr = %String::from_utf8_lossy(&checkout_out.stderr), "git checkout main failed");
                } else {
                    let pull_out = Command::new("git")
                        .args(["pull", "--rebase", "origin", "main"])
                        .current_dir(&worktree_path)
                        .output()
                        .context("Failed to pull main in worktree")?;

                    if !pull_out.status.success() {
                        warn!(path = %worktree_path.display(), stderr = %String::from_utf8_lossy(&pull_out.stderr), "git pull origin/main failed in worktree");
                    }
                }
            } else {
                let checkout_out = Command::new("git")
                    .args(["checkout", "-B", "main", "origin/main"])
                    .current_dir(&worktree_path)
                    .output()
                    .context("Failed to create/update local main from origin/main in worktree")?;

                if !checkout_out.status.success() {
                    warn!(path = %worktree_path.display(), stderr = %String::from_utf8_lossy(&checkout_out.stderr), "git checkout -B main origin/main failed in worktree");
                }
            }

            // Decide whether to reuse an existing branch (local or remote) or create a new one from origin/main.
            let branch_exists_local = {
                let output = Command::new("git")
                    .args(["branch", "--list", &branch_name])
                    .current_dir(&self.project_root)
                    .output()
                    .context("Failed to check local branch existence")?;

                !output.stdout.is_empty()
            };

            let branch_exists_remote = if !branch_exists_local {
                let output = Command::new("git")
                    .args(["ls-remote", "--heads", "origin", &branch_name])
                    .current_dir(&self.project_root)
                    .output()
                    .context("Failed to check remote branch existence")?;

                !output.stdout.is_empty()
            } else {
                false
            };

            if branch_exists_local {
                // Checkout existing local branch in the worktree
                let output = Command::new("git")
                    .args(["checkout", &branch_name])
                    .current_dir(&worktree_path)
                    .output()
                    .context("Failed to checkout existing branch in worktree")?;

                if !output.status.success() {
                    return Err(anyhow!(
                        "Failed to checkout branch {}: {}",
                        branch_name,
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }
            } else if branch_exists_remote {
                // Create local tracking branch from origin/<branch> and check it out in the existing worktree
                let origin_ref = format!("origin/{}", branch_name);
                let output = Command::new("git")
                    .args(["checkout", "-B", &branch_name, &origin_ref])
                    .current_dir(&worktree_path)
                    .output()
                    .context("Failed to create local branch from origin/<branch> in worktree")?;

                if !output.status.success() {
                    return Err(anyhow!(
                        "Failed to create local branch {} from {}: {}",
                        branch_name,
                        origin_ref,
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }
            } else {
                // Create new branch from origin/main and check it out in the existing worktree
                let output = Command::new("git")
                    .args(["checkout", "-b", &branch_name, "origin/main"])
                    .current_dir(&worktree_path)
                    .output()
                    .context("Failed to create branch from origin/main")?;

                if !output.status.success() {
                    return Err(anyhow!(
                        "Failed to create branch {} from origin/main: {}",
                        branch_name,
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }
            }
        } else {
            // Worktree doesn't exist yet: create it from origin/main
            let output = Command::new("git")
                .args(["worktree", "add"])
                .arg(&worktree_path)
                .args(["-b", &branch_name])
                .current_dir(&self.project_root)
                .output()
                .context("Failed to run git worktree add")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains("already exists") {
                    info!(branch = %branch_name, "Branch exists, creating worktree from existing branch");
                    let output = Command::new("git")
                        .args(["worktree", "add"])
                        .arg(&worktree_path)
                        .arg(&branch_name)
                        .current_dir(&self.project_root)
                        .output()
                        .context("Failed to run git worktree add from existing branch")?;

                    if !output.status.success() {
                        return Err(anyhow!(
                            "Failed to create worktree from existing branch: {}",
                            String::from_utf8_lossy(&output.stderr)
                        ));
                    }
                } else {
                    return Err(anyhow!("Failed to create worktree: {}", stderr));
                }
            }
        }

        // Verify the worktree is clean
        let status = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&worktree_path)
            .output()
            .context("Failed to run git status")?;

        if !status.status.success() || !status.stdout.is_empty() {
            warn!(path = %worktree_path.display(), "Worktree is not clean");
        }

        info!(path = %worktree_path.display(), branch = %branch_name, "Worktree created successfully");
        Ok(worktree_path)
    }

    /// Remove a worktree and its associated branch.
    pub fn remove_worktree(&self, pair_id: &str) -> Result<()> {
        let worktree_path = self.worktrees_dir.join(pair_id);

        info!(path = %worktree_path.display(), "Removing worktree");

        let branch_name = self
            .detect_worktree_branch(pair_id)
            .unwrap_or_else(|| Self::branch_name(pair_id, "unknown"));

        let output = Command::new("git")
            .args(["worktree", "remove"])
            .arg(&worktree_path)
            .current_dir(&self.project_root)
            .output();

        match output {
            Ok(output) if output.status.success() => {
                info!(pair_id, "Worktree removed successfully");
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!(error = %stderr, "Git worktree remove failed, forcing removal");

                let output = Command::new("git")
                    .args(["worktree", "remove", "--force"])
                    .arg(&worktree_path)
                    .current_dir(&self.project_root)
                    .output()
                    .context("Failed to force remove worktree")?;

                if !output.status.success() {
                    warn!(path = %worktree_path.display(), "Forcing manual worktree removal");
                    if worktree_path.exists() {
                        std::fs::remove_dir_all(&worktree_path)
                            .context("Failed to manually remove worktree directory")?;
                    }
                }
            }
            Err(e) => {
                warn!(error = %e, "Failed to run git worktree remove");
                if worktree_path.exists() {
                    std::fs::remove_dir_all(&worktree_path)
                        .context("Failed to manually remove worktree directory")?;
                }
            }
        }

        self.prune_stale_worktrees();

        // By default preserve branches so history/audit is retained. Set
        // PAIR_HARNESS_PRUNE_BRANCHES=true to enable automatic branch deletion.
        let prune_branches = std::env::var("PAIR_HARNESS_PRUNE_BRANCHES")
            .unwrap_or_else(|_| "false".to_string()) == "true";
        if prune_branches {
            self.delete_branch_if_exists(&branch_name);
        } else {
            info!(branch = %branch_name, "Preserving branch by default (set PAIR_HARNESS_PRUNE_BRANCHES=true to delete)");
        }

        info!(pair_id, "Worktree removed");
        Ok(())
    }

    /// Create an idle worktree on main branch.
    pub fn create_idle_worktree(&self, pair_id: &str) -> Result<PathBuf> {
        let worktree_path = self.worktrees_dir.join(pair_id);

        info!(pair_id, "Creating idle worktree on main");

        // If worktree exists, update it to origin/main instead of removing it.
        if worktree_path.exists() {
            info!(path = %worktree_path.display(), "Idle worktree exists, updating to origin/main");
            if let Err(e) = self.run_git_in_main(&["fetch", "origin", "main"]) {
                warn!(error = %e, "git fetch origin/main failed, continuing");
            }
            let output = Command::new("git")
                .args(["checkout", "-B", "main", "origin/main"])
                .current_dir(&worktree_path)
                .output()
                .context("Failed to update existing worktree to origin/main")?;

            if !output.status.success() {
                warn!(path = %worktree_path.display(), stderr = %String::from_utf8_lossy(&output.stderr), "Failed to checkout/update main in existing idle worktree");
            }
        }

        // Create worktrees directory if needed
        std::fs::create_dir_all(&self.worktrees_dir)
            .context("Failed to create worktrees directory")?;

        // Create worktree on main branch
        let output = Command::new("git")
            .args(["worktree", "add"])
            .arg(&worktree_path)
            .arg("main")
            .current_dir(&self.project_root)
            .output()
            .context("Failed to run git worktree add")?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to create idle worktree: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        info!(path = %worktree_path.display(), "Idle worktree created");
        Ok(worktree_path)
    }

    /// Check for divergence from main and optionally rebase.
    pub fn check_divergence(
        &self,
        worktree_path: &Path,
        threshold: u32,
    ) -> Result<DivergenceStatus> {
        let behind = self.count_commits_behind(worktree_path)?;

        debug!(path = %worktree_path.display(), behind, "Divergence check");

        if behind > threshold {
            info!(behind, threshold, "Branch is behind main, rebase needed");
            return Ok(DivergenceStatus::NeedsRebase {
                commits_behind: behind,
            });
        }

        Ok(DivergenceStatus::UpToDate)
    }

    /// Rebase the worktree onto origin/main.
    pub fn rebase_onto_main(&self, worktree_path: &Path) -> Result<RebaseResult> {
        info!(path = %worktree_path.display(), "Rebasing onto origin/main");

        // Fetch latest
        let output = Command::new("git")
            .args(["fetch", "origin", "main"])
            .current_dir(worktree_path)
            .output()
            .context("Failed to fetch origin/main")?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to fetch: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Rebase
        let output = Command::new("git")
            .args(["rebase", "origin/main"])
            .current_dir(worktree_path)
            .output()
            .context("Failed to rebase")?;

        if output.status.success() {
            info!(path = %worktree_path.display(), "Rebase successful");
            return Ok(RebaseResult::Success);
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("conflict") {
            warn!(path = %worktree_path.display(), "Rebase has conflicts");
            return Ok(RebaseResult::Conflict);
        }

        Err(anyhow!("Rebase failed: {}", stderr))
    }

    /// Abort an in-progress rebase.
    pub fn abort_rebase(&self, worktree_path: &Path) -> Result<()> {
        let output = Command::new("git")
            .args(["rebase", "--abort"])
            .current_dir(worktree_path)
            .output()
            .context("Failed to abort rebase")?;

        if !output.status.success() {
            warn!(error = %String::from_utf8_lossy(&output.stderr), "Failed to abort rebase");
        }

        Ok(())
    }

    /// Get the current branch name in a worktree.
    pub fn get_current_branch(&self, worktree_path: &Path) -> Result<String> {
        let output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(worktree_path)
            .output()
            .context("Failed to get current branch")?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to get branch: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    /// Count commits behind origin/main.
    fn count_commits_behind(&self, worktree_path: &Path) -> Result<u32> {
        let output = Command::new("git")
            .args(["rev-list", "--count", "HEAD..origin/main"])
            .current_dir(worktree_path)
            .output()
            .context("Failed to count commits behind")?;

        if !output.status.success() {
            // If origin/main doesn't exist, return 0
            return Ok(0);
        }

        let count: u32 = String::from_utf8(output.stdout)?
            .trim()
            .parse()
            .unwrap_or(0);

        Ok(count)
    }

    fn run_git_in_main(&self, args: &[&str]) -> Result<()> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.project_root)
            .output()
            .context("Failed to run git command")?;

        if !output.status.success() {
            return Err(anyhow!(
                "Git command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    fn prune_stale_worktrees(&self) {
        let _ = Command::new("git")
            .args(["worktree", "prune"])
            .current_dir(&self.project_root)
            .output();
    }

    fn delete_branch_if_exists(&self, branch_name: &str) {
        let output = Command::new("git")
            .args(["branch", "-D"])
            .arg(branch_name)
            .current_dir(&self.project_root)
            .output();

        match output {
            Ok(o) if o.status.success() => {
                info!(branch = branch_name, "Deleted stale branch");
            }
            _ => {
                debug!(
                    branch = branch_name,
                    "Branch does not exist or could not be deleted"
                );
            }
        }
    }

    fn detect_worktree_branch(&self, pair_id: &str) -> Option<String> {
        let worktree_path = self.worktrees_dir.join(pair_id);
        if !worktree_path.exists() {
            return None;
        }
        let output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(&worktree_path)
            .output()
            .ok()?;

        if output.status.success() {
            let branch = String::from_utf8(output.stdout).ok()?.trim().to_string();
            if branch != "HEAD" && !branch.is_empty() {
                return Some(branch);
            }
        }
        None
    }

    /// Generate branch name for a pair/ticket.
    pub fn branch_name(pair_id: &str, ticket_id: &str) -> String {
        // Canonicalize branch names: prefer explicit `forge-` prefix.
        if pair_id.starts_with("forge-") {
            format!("{}/{}", pair_id, ticket_id)
        } else {
            format!("forge-{}/{}", pair_id, ticket_id)
        }
    }
}

/// Status of branch divergence from main.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DivergenceStatus {
    /// Branch is up to date with main
    UpToDate,
    /// Branch needs rebase
    NeedsRebase { commits_behind: u32 },
}

/// Result of a rebase operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RebaseResult {
    /// Rebase completed successfully
    Success,
    /// Rebase has conflicts that need resolution
    Conflict,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_name() {
        assert_eq!(
            WorktreeManager::branch_name("pair-1", "T-42"),
            "forge-pair-1/T-42"
        );
    }
}
