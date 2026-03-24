// crates/agent-forge/src/lib.rs
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use pocketflow_core::{BatchNode, SharedStore, Action};
use config::{WorkerSlot, WorkerStatus, state::{KEY_WORKER_SLOTS, ACTION_PR_OPENED, ACTION_FAILED, ACTION_EMPTY}};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeStatus {
    pub outcome: String,
    pub ticket_id: String,
    pub branch: String,
    pub pr_url: Option<String>,
    pub pr_number: Option<u32>,
    pub notes: Option<String>,
}

pub struct ForgeNode {
    pub workspace_root: PathBuf,
}

impl ForgeNode {
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self { workspace_root: workspace_root.into() }
    }
}

#[async_trait]
impl BatchNode for ForgeNode {
    fn name(&self) -> &str { "forge" }

    async fn prep_batch(&self, store: &SharedStore) -> Result<Vec<Value>> {
        let slots: HashMap<String, WorkerSlot> = store
            .get_typed(KEY_WORKER_SLOTS)
            .await
            .unwrap_or_default();

        let active_workers: Vec<Value> = slots.values()
            .filter(|s| matches!(s.status, WorkerStatus::Assigned { .. } | WorkerStatus::Working { .. }))
            .map(|s| json!(s))
            .collect();
        
        Ok(active_workers)
    }

    async fn exec_one(&self, item: Value) -> Result<Value> {
        let slot: WorkerSlot = serde_json::from_value(item)?;
        let worker_id = slot.id.clone();
        
        let ticket_id = match &slot.status {
            WorkerStatus::Assigned { ticket_id } => ticket_id.clone(),
            WorkerStatus::Working { ticket_id }  => ticket_id.clone(),
            _ => return Ok(json!({"outcome": "idle", "worker_id": worker_id})),
        };

        let worker_dir = self.workspace_root.join("forge").join("workers").join(&worker_id);
        let status_path = worker_dir.join("STATUS.json");

        info!(worker = worker_id, ticket = ticket_id, "Spawning Claude Code...");

        // 1. Prepare command
        let prompt = format!(
            "You are FORGE agent {}. \
             Implement ticket {}. \
             Branch: forge/{}/{}. \
             When done, open a PR and write STATUS.json.",
            worker_id, ticket_id, worker_id, ticket_id
        );

        let mut child = tokio::process::Command::new("claude")
            .args(["--print", "--output-format", "json"])
            .arg(&prompt)
            .current_dir(&worker_dir)
            .env("ANTHROPIC_API_KEY", std::env::var("ANTHROPIC_API_KEY").unwrap_or_default())
            .spawn()
            .map_err(|e| anyhow!("Failed to spawn Claude Code: {}", e))?;

        // 2. Wait with timeout (30 min)
        let timeout_dur = std::time::Duration::from_secs(1800);
        let result = tokio::time::timeout(timeout_dur, child.wait()).await;

        match result {
            Err(_) => {
                child.kill().await?;
                warn!(worker = worker_id, "Claude Code timed out after 30m");
                return Ok(json!({
                    "worker_id": worker_id,
                    "ticket_id": ticket_id,
                    "outcome": "fuel_exhausted",
                    "reason": "timeout"
                }));
            }
            Ok(Ok(status)) if !status.success() => {
                warn!(worker = worker_id, exit = ?status.code(), "Claude Code failed");
            }
            _ => {}
        }

        // 3. Read STATUS.json
        if tokio::fs::try_exists(&status_path).await? {
            let content = tokio::fs::read_to_string(&status_path).await?;
            let forge_status: ForgeStatus = serde_json::from_str(&content)?;
            return Ok(json!({
                "worker_id": worker_id,
                "ticket_id": ticket_id,
                "outcome": forge_status.outcome,
                "branch": forge_status.branch,
                "pr_url": forge_status.pr_url,
                "pr_number": forge_status.pr_number,
                "notes": forge_status.notes,
            }));
        }

        Ok(json!({
            "worker_id": worker_id,
            "ticket_id": ticket_id,
            "outcome": "failed",
            "reason": "STATUS.json not written"
        }))
    }

    async fn post_batch(
        &self,
        store: &SharedStore,
        results: Vec<Result<Value>>,
    ) -> Result<Action> {
        let mut slots: HashMap<String, WorkerSlot> = store
            .get_typed(KEY_WORKER_SLOTS)
            .await
            .unwrap_or_default();

        let mut all_success = true;

        for res_opt in &results {
            let res = match res_opt {
                Ok(v) => v,
                Err(e) => {
                    warn!("Batch item failed: {}", e);
                    all_success = false;
                    continue;
                }
            };
            let worker_id = res["worker_id"].as_str().unwrap_or("");
            let ticket_id = res["ticket_id"].as_str().unwrap_or("");
            let outcome   = res["outcome"].as_str().unwrap_or("failed");

            if let Some(slot) = slots.get_mut(worker_id) {
                if outcome == "pr_opened" {
                    info!(worker = worker_id, ticket = ticket_id, "Work completed successfully");
                    slot.status = WorkerStatus::Done { 
                        ticket_id: ticket_id.to_string(), 
                        outcome: outcome.to_string() 
                    };
                } else if outcome != "idle" {
                    warn!(worker = worker_id, ticket = ticket_id, outcome, "Work failed");
                    slot.status = WorkerStatus::Idle;
                    all_success = false;
                }
            }
        }

        store.set(KEY_WORKER_SLOTS, json!(slots)).await;

        if all_success && !results.is_empty() {
            Ok(Action::new(ACTION_PR_OPENED))
        } else if results.is_empty() {
            Ok(Action::new(ACTION_EMPTY))
        } else {
            Ok(Action::new(ACTION_FAILED))
        }
    }
}
