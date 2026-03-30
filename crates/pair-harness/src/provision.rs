// crates/pair-harness/src/provision.rs
//! Provisioning for pair configuration files.
//!
//! Generates settings.json for FORGE and SENTINEL with auto-mode
//! permissions and explicit allow/deny lists.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::fs;
use serde_json::{json, Value};
use tracing::{info, debug};

/// Provisions configuration files for pairs.
pub struct Provisioner {
    /// Project root directory
    project_root: PathBuf,
}

impl Provisioner {
    /// Create a new provisioner.
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        Self {
            project_root: project_root.into(),
        }
    }

    /// Provision all configuration for a pair.
    pub async fn provision_pair(
        &self,
        pair_id: &str,
        worktree: &Path,
        shared: &Path,
        github_token: &str,
        redis_url: &str,
    ) -> Result<()> {
        info!(pair = pair_id, "Provisioning pair configuration");

        // 1. Create FORGE settings.json
        self.create_forge_settings(worktree)?;

        // 2. Create SENTINEL settings.json
        self.create_sentinel_settings(shared)?;

        // 3. Create FORGE mcp.json
        let mcp_gen = crate::mcp_config::McpConfigGenerator::new(github_token, redis_url);
        mcp_gen.generate_forge_config(
            worktree,
            shared,
            &worktree.join(".claude").join("mcp.json"),
        )?;

        // 4. Create SENTINEL mcp.json
        mcp_gen.generate_sentinel_config(
            worktree,
            shared,
            &shared.join("sentinel").join(".claude").join("mcp.json"),
        )?;

        // 5. Symlink plugin to FORGE
        self.symlink_plugin(worktree, "forge")?;

        // 6. Symlink plugin to SENTINEL
        self.symlink_plugin(&shared.join("sentinel"), "sentinel")?;

        // 7. Create shared directory structure
        self.create_shared_structure(shared)?;

        info!(pair = pair_id, "Pair provisioning complete");
        Ok(())
    }

    /// Create FORGE's settings.json with auto-mode permissions.
    pub fn create_forge_settings(&self, worktree: &Path) -> Result<()> {
        let claude_dir = worktree.join(".claude");
        fs::create_dir_all(&claude_dir)
            .context("Failed to create .claude directory")?;

        let settings_path = claude_dir.join("settings.json");

        info!(path = %settings_path.display(), "Creating FORGE settings.json");

        let settings = json!({
            "permissions": {
                "defaultMode": "auto",
                "allow": FORGE_ALLOW_LIST,
                "deny": FORGE_DENY_LIST
            }
        });

        self.write_json(&settings_path, &settings)
    }

    /// Create SENTINEL's settings.json with read-only permissions.
    pub fn create_sentinel_settings(&self, shared: &Path) -> Result<()> {
        let claude_dir = shared.join("sentinel").join(".claude");
        fs::create_dir_all(&claude_dir)
            .context("Failed to create sentinel .claude directory")?;

        let settings_path = claude_dir.join("settings.json");

        info!(path = %settings_path.display(), "Creating SENTINEL settings.json");

        let settings = json!({
            "permissions": {
                "defaultMode": "auto",
                "allow": SENTINEL_ALLOW_LIST,
                "deny": SENTINEL_DENY_LIST
            }
        });

        self.write_json(&settings_path, &settings)
    }

    /// Symlink the Sprintless plugin to a .claude directory.
    pub fn symlink_plugin(&self, target_dir: &Path, role: &str) -> Result<()> {
        let plugin_source = self.project_root.join(".sprintless").join("plugin");
        let plugins_dir = target_dir.join(".claude").join("plugins");

        fs::create_dir_all(&plugins_dir)
            .context("Failed to create plugins directory")?;

        let symlink_path = plugins_dir.join("sprintless");

        // Remove existing symlink if present
        if symlink_path.exists() || symlink_path.symlink_metadata().is_ok() {
            let _ = fs::remove_file(&symlink_path);
        }

        // Create symlink
        #[cfg(unix)]
        std::os::unix::fs::symlink(&plugin_source, &symlink_path)
            .context("Failed to create plugin symlink")?;

        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&plugin_source, &symlink_path)
            .context("Failed to create plugin symlink")?;

        debug!(
            role = role,
            source = %plugin_source.display(),
            target = %symlink_path.display(),
            "Plugin symlinked"
        );

        Ok(())
    }

    /// Create the shared directory structure.
    pub fn create_shared_structure(&self, shared: &Path) -> Result<()> {
        fs::create_dir_all(shared)
            .context("Failed to create shared directory")?;

        // Create sentinel subdirectory
        let sentinel_dir = shared.join("sentinel");
        fs::create_dir_all(&sentinel_dir)
            .context("Failed to create sentinel directory")?;

        // Create .gitignore for shared directory
        let gitignore = shared.join(".gitignore");
        fs::write(&gitignore, "# Shared artifacts are runtime state, not committed\n*\n!.gitignore\n")
            .context("Failed to write .gitignore")?;

        debug!(path = %shared.display(), "Shared directory structure created");
        Ok(())
    }

    /// Write JSON to file atomically.
    fn write_json(&self, path: &Path, value: &Value) -> Result<()> {
        let temp_path = path.with_extension("json.tmp");
        let content = serde_json::to_string_pretty(value)
            .context("Failed to serialize JSON")?;

        fs::write(&temp_path, content)
            .context("Failed to write JSON")?;

        fs::rename(&temp_path, path)
            .context("Failed to rename JSON file")?;

        Ok(())
    }

    /// Write TICKET.md to shared directory.
    pub fn write_ticket(&self, shared: &Path, ticket: &crate::types::Ticket) -> Result<()> {
        let path = shared.join("TICKET.md");

        let content = format!(
            "# {}\n\n**Issue:** #{} \n**URL:** {}\n\n{}\n\n## Acceptance Criteria\n\n{}\n",
            ticket.title,
            ticket.issue_number,
            ticket.url,
            ticket.body,
            ticket.acceptance_criteria
                .iter()
                .map(|c| format!("- {}", c))
                .collect::<Vec<_>>()
                .join("\n")
        );

        fs::write(&path, content)
            .context("Failed to write TICKET.md")?;

        info!(path = %path.display(), "TICKET.md written");
        Ok(())
    }

    /// Write TASK.md to shared directory.
    pub fn write_task(&self, shared: &Path, task: &str) -> Result<()> {
        let path = shared.join("TASK.md");

        fs::write(&path, task)
            .context("Failed to write TASK.md")?;

        info!(path = %path.display(), "TASK.md written");
        Ok(())
    }
}

/// FORGE's allow list for auto-mode.
const FORGE_ALLOW_LIST: &[&str] = &[
    "Read",
    "Write",
    "Edit",
    "MultiEdit",
    "Glob",
    "Grep",
    "Bash(git add:*)",
    "Bash(git commit:*)",
    "Bash(git status:*)",
    "Bash(git diff:*)",
    "Bash(git log:*)",
    "Bash(.agent/tooling/run-tests.sh:*)",
    "Bash(cargo clippy:*)",
    "Bash(cargo test:*)",
    "Bash(npx eslint:*)",
    "Bash(npx jest:*)",
    "Bash(ruff check:*)",
];

/// FORGE's deny list for auto-mode.
const FORGE_DENY_LIST: &[&str] = &[
    "Bash(git push:*)",
    "Bash(rm -rf:*)",
    "Bash(sudo:*)",
    "Bash(curl:*)",
    "Bash(wget:*)",
    "Bash(npm install:*)",
    "Bash(pip install:*)",
];

/// SENTINEL's allow list for auto-mode (read-only).
const SENTINEL_ALLOW_LIST: &[&str] = &[
    "Read",
    "Glob",
    "Grep",
    "Bash(.agent/tooling/run-tests.sh:*)",
    "Bash(npx eslint:*)",
    "Bash(ruff check:*)",
    "Bash(cargo clippy:*)",
];

/// SENTINEL's deny list for auto-mode.
const SENTINEL_DENY_LIST: &[&str] = &[
    "Write",
    "Edit",
    "MultiEdit",
    "Bash(git:*)",
    "Bash(rm:*)",
    "Bash(sudo:*)",
];

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_forge_settings() {
        let dir = tempdir().unwrap();
        let worktree = dir.path();

        let provisioner = Provisioner::new(dir.path());
        provisioner.create_forge_settings(worktree).unwrap();

        let settings_path = worktree.join(".claude").join("settings.json");
        assert!(settings_path.exists());

        let content = fs::read_to_string(&settings_path).unwrap();
        let settings: Value = serde_json::from_str(&content).unwrap();

        assert_eq!(settings["permissions"]["defaultMode"], "auto");
        assert!(settings["permissions"]["allow"].is_array());
        assert!(settings["permissions"]["deny"].is_array());
    }

    #[test]
    fn test_create_sentinel_settings() {
        let dir = tempdir().unwrap();
        let shared = dir.path();

        let provisioner = Provisioner::new(dir.path());
        provisioner.create_sentinel_settings(shared).unwrap();

        let settings_path = shared.join("sentinel").join(".claude").join("settings.json");
        assert!(settings_path.exists());

        let content = fs::read_to_string(&settings_path).unwrap();
        let settings: Value = serde_json::from_str(&content).unwrap();

        // SENTINEL should not have Write permission
        let allow = settings["permissions"]["allow"].as_array().unwrap();
        assert!(!allow.iter().any(|a| a == "Write"));
    }

    #[test]
    fn test_create_shared_structure() {
        let dir = tempdir().unwrap();
        let shared = dir.path().join("shared");

        let provisioner = Provisioner::new(dir.path());
        provisioner.create_shared_structure(&shared).unwrap();

        assert!(shared.exists());
        assert!(shared.join("sentinel").exists());
        assert!(shared.join(".gitignore").exists());
    }
}