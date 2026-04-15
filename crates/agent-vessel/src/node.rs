// crates/agent-vessel/src/node.rs
//
// VesselNode — orchestrates CI polling, merging, and notification.
// Implements the Node trait for integration with the Flow.

use anyhow::Result;
use async_trait::async_trait;
use pocketflow_core::{Action, CiStatus, Node, PrInfo, SharedStore};
use serde_json::{json, Value};
use tracing::{debug, info, warn};

use crate::ci_poller::CiPollResult;
use crate::types::{VesselConfig, VesselOutcome};
use crate::{CiPoller, PrMerger, VesselNotifier};

/// VESSEL Node — DevOps Specialist and Merge Gatekeeper.
///
/// Three-phase workflow:
/// 1. prep: Read pending PRs from SharedStore
/// 2. exec: Poll CI, merge if green, return outcomes
/// 3. post: Emit events, update tickets, return routing action
pub struct VesselNode {
    _config: VesselConfig,
    client: github::GithubRestClient,
    poller: CiPoller,
    merger: PrMerger,
}

impl VesselNode {
    pub fn new(config: VesselConfig) -> Self {
        let client = github::GithubRestClient::new(&config.github_token);
        
        Self {
            poller: CiPoller::new(config.ci_poll.clone(), client.clone()),
            merger: PrMerger::new(client.clone(), config.merge_method),
            client,
            _config: config,
        }
    }

    pub fn from_env() -> Self {
        Self::new(VesselConfig::from_env())
    }
}

#[async_trait]
impl Node for VesselNode {
    fn name(&self) -> &str {
        "vessel"
    }

    /// Phase 1: Read pending PRs from SharedStore.
    async fn prep(&self, store: &SharedStore) -> Result<Value> {
        debug!("VESSEL prep: reading pending PRs");

        let repository: Option<String> = store.get_typed("repository").await;
        let pending_prs: Option<Vec<Value>> = store.get_typed("pending_prs").await;

        let (owner, repo) = parse_repository(repository.as_deref());

        Ok(json!({
            "owner": owner,
            "repo": repo,
            "pending_prs": pending_prs.unwrap_or_default(),
        }))
    }

    /// Phase 2: Process each pending PR (poll CI → merge → return outcome).
    async fn exec(&self, prep_result: Value) -> Result<Value> {
        let owner = prep_result["owner"].as_str().unwrap_or("");
        let repo = prep_result["repo"].as_str().unwrap_or("");
        let pending_prs = prep_result["pending_prs"].as_array().cloned().unwrap_or_default();

        if pending_prs.is_empty() {
            info!("No pending PRs to process");
            return Ok(json!({ "outcomes": [], "has_work": false }));
        }

        info!(count = pending_prs.len(), "Processing pending PRs");

        let mut outcomes = Vec::new();

        for pr in pending_prs {
            let pr_number = pr["number"].as_u64().unwrap_or(0);

            if pr_number == 0 {
                warn!(pr = ?pr, "Skipping invalid PR entry");
                continue;
            }

            debug!(pr_number, "Fetching PR details");
            
            let pr_info = match self.client.get_pull_request(owner, repo, pr_number).await {
                Ok(info) => info,
                Err(e) => {
                    warn!(pr_number, error = %e, "Failed to fetch PR details, skipping");
                    continue;
                }
            };

            let outcome = self.process_single_pr(owner, repo, pr_info).await?;
            outcomes.push(outcome);
        }

        Ok(json!({
            "outcomes": outcomes,
            "has_work": !outcomes.is_empty(),
        }))
    }

    /// Phase 3: Emit events, update SharedStore, return routing action.
    async fn post(&self, store: &SharedStore, exec_result: Value) -> Result<Action> {
        let outcomes: Vec<VesselOutcome> = serde_json::from_value(exec_result["outcomes"].clone())
            .unwrap_or_default();
        let has_work = exec_result["has_work"].as_bool().unwrap_or(false);

        if !has_work {
            debug!("No PRs were processed");
            return Ok(Action::new("no_work"));
        }

        let mut any_success = false;
        let mut any_failure = false;

        for outcome in outcomes {
            match &outcome {
                VesselOutcome::Merged { ticket_id, pr_number, sha } => {
                    VesselNotifier::emit_ticket_merged(store, ticket_id, *pr_number, sha).await;
                    VesselNotifier::set_ticket_status_merged(store, ticket_id).await;
                    
                    self.update_ticket_status(store, ticket_id, "merged").await;
                    self.remove_from_pending_prs(store, *pr_number).await;
                    
                    any_success = true;
                }
                VesselOutcome::CiFailed { ticket_id, pr_number, reason } => {
                    VesselNotifier::emit_ci_failed(store, ticket_id.as_deref(), *pr_number, reason).await;
                    any_failure = true;
                }
                VesselOutcome::MergeBlocked { ticket_id, pr_number, reason } => {
                    VesselNotifier::emit_merge_blocked(store, ticket_id.as_deref(), *pr_number, reason).await;
                    any_failure = true;
                }
                VesselOutcome::CiTimeout { ticket_id, pr_number } => {
                    VesselNotifier::emit_ci_timeout(store, ticket_id.as_deref(), *pr_number).await;
                    any_failure = true;
                }
            }
        }

        if any_success {
            Ok(Action::DEPLOYED.into())
        } else if any_failure {
            Ok(Action::DEPLOY_FAILED.into())
        } else {
            Ok(Action::new("no_work"))
        }
    }
}

impl VesselNode {
    /// Process a single PR: poll CI → merge if green → return outcome.
    async fn process_single_pr(&self, owner: &str, repo: &str, pr_info: PrInfo) -> Result<VesselOutcome> {
        let ticket_id = pr_info.ticket_id.clone();
        let pr_number = pr_info.number;

        info!(pr_number, ticket_id = ?ticket_id, "Processing PR");

        let poll_result = self.poller.poll_until_terminal(owner, repo, &pr_info).await?;

        match poll_result {
            CiPollResult::Status(CiStatus::Success) => {
                match self.merger.merge(owner, repo, &pr_info).await {
                    Ok(result) if result.merged => Ok(VesselOutcome::Merged {
                        ticket_id: ticket_id.unwrap_or_else(|| format!("T-{}", pr_number)),
                        pr_number,
                        sha: result.sha.unwrap_or_default(),
                    }),
                    Ok(result) => Ok(VesselOutcome::MergeBlocked {
                        ticket_id,
                        pr_number,
                        reason: result.message,
                    }),
                    Err(e) => Ok(VesselOutcome::MergeBlocked {
                        ticket_id,
                        pr_number,
                        reason: e.to_string(),
                    }),
                }
            }
            CiPollResult::Status(status) => Ok(VesselOutcome::CiFailed {
                ticket_id,
                pr_number,
                reason: format!("CI status: {:?}", status),
            }),
            CiPollResult::Timeout => Ok(VesselOutcome::CiTimeout { ticket_id, pr_number }),
        }
    }

    /// Update ticket status in SharedStore.
    async fn update_ticket_status(&self, store: &SharedStore, ticket_id: &str, status: &str) {
        let mut tickets: Vec<Value> = store.get_typed("tickets").await.unwrap_or_default();
        
        for ticket in tickets.iter_mut() {
            if ticket["id"].as_str() == Some(ticket_id) {
                ticket["status"] = json!({ "type": status });
                break;
            }
        }
        
        store.set("tickets", json!(tickets)).await;
    }

    /// Remove PR from pending_prs list.
    async fn remove_from_pending_prs(&self, store: &SharedStore, pr_number: u64) {
        let mut pending: Vec<Value> = store.get_typed("pending_prs").await.unwrap_or_default();
        pending.retain(|pr| pr["number"].as_u64() != Some(pr_number));
        store.set("pending_prs", json!(pending)).await;
    }

    /// Reconcile startup: check for PRs that are already merged on GitHub.
    pub async fn reconcile(&self, store: &SharedStore) -> Result<()> {
        info!("Running VESSEL startup reconciliation");

        let repository: Option<String> = store.get_typed("repository").await;
        let pending_prs: Option<Vec<Value>> = store.get_typed("pending_prs").await;
        let (owner, repo) = parse_repository(repository.as_deref());

        let pending = pending_prs.unwrap_or_default();

        for pr in pending {
            let pr_number = pr["number"].as_u64().unwrap_or(0);
            if pr_number == 0 {
                continue;
            }

            if self.client.is_pr_merged(owner, repo, pr_number).await? {
                warn!(pr_number, "Found already-merged PR during reconciliation");
                
                let ticket_id = pr["ticket_id"].as_str().map(String::from);
                let pr_info = self.client.get_pull_request(owner, repo, pr_number).await;
                
                if let Ok(info) = pr_info {
                    let tid = ticket_id.or(info.ticket_id).unwrap_or_else(|| format!("T-{}", pr_number));
                    VesselNotifier::emit_ticket_merged(store, &tid, pr_number, &info.head_sha).await;
                    VesselNotifier::set_ticket_status_merged(store, &tid).await;
                    self.remove_from_pending_prs(store, pr_number).await;
                }
            }
        }

        Ok(())
    }
}

fn parse_repository(repository: Option<&str>) -> (&str, &str) {
    match repository {
        Some(repo) => {
            let parts: Vec<&str> = repo.split('/').collect();
            if parts.len() == 2 {
                (parts[0], parts[1])
            } else {
                ("", "")
            }
        }
        None => ("", ""),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_repository() {
        assert_eq!(parse_repository(Some("owner/repo")), ("owner", "repo"));
        assert_eq!(parse_repository(Some("single")), ("", ""));
        assert_eq!(parse_repository(None), ("", ""));
    }

    #[tokio::test]
    async fn test_prep_reads_pending_prs() {
        let store = SharedStore::new_in_memory();
        store.set("repository", json!("test-owner/test-repo")).await;
        store.set("pending_prs", json!([
            {"number": 1, "ticket_id": "T-1"},
            {"number": 2, "ticket_id": "T-2"},
        ])).await;

        let config = VesselConfig::default();
        let node = VesselNode::new(config);

        let result = node.prep(&store).await.unwrap();
        
        assert_eq!(result["owner"], "test-owner");
        assert_eq!(result["repo"], "test-repo");
        assert_eq!(result["pending_prs"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_prep_empty_pending_prs() {
        let store = SharedStore::new_in_memory();

        let config = VesselConfig::default();
        let node = VesselNode::new(config);

        let result = node.prep(&store).await.unwrap();
        
        assert_eq!(result["pending_prs"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_post_handles_merged_outcome() {
        let store = SharedStore::new_in_memory();
        store.set("pending_prs", json!([{"number": 42, "ticket_id": "T-42"}])).await;
        store.set("tickets", json!([{"id": "T-42", "status": {"type": "in_progress"}}])).await;

        let config = VesselConfig::default();
        let node = VesselNode::new(config);

        let exec_result = json!({
            "outcomes": [VesselOutcome::Merged {
                ticket_id: "T-42".to_string(),
                pr_number: 42,
                sha: "abc123".to_string(),
            }],
            "has_work": true,
        });

        let action = node.post(&store, exec_result).await.unwrap();
        assert_eq!(action.as_str(), Action::DEPLOYED);

        let events = store.get_events_since(0).await;
        assert!(events.iter().any(|e| e.event_type == "ticket_merged"));

        let status = store.get("ticket:T-42:status").await;
        assert_eq!(status, Some(json!("Merged")));

        let pending: Vec<Value> = store.get_typed("pending_prs").await.unwrap_or_default();
        assert!(pending.is_empty());
    }

    #[tokio::test]
    async fn test_post_handles_ci_failed_outcome() {
        let store = SharedStore::new_in_memory();

        let config = VesselConfig::default();
        let node = VesselNode::new(config);

        let exec_result = json!({
            "outcomes": [VesselOutcome::CiFailed {
                ticket_id: Some("T-42".to_string()),
                pr_number: 42,
                reason: "Tests failed".to_string(),
            }],
            "has_work": true,
        });

        let action = node.post(&store, exec_result).await.unwrap();
        assert_eq!(action.as_str(), Action::DEPLOY_FAILED);

        let events = store.get_events_since(0).await;
        assert!(events.iter().any(|e| e.event_type == "ci_failed"));
    }

    #[tokio::test]
    async fn test_post_handles_merge_blocked_outcome() {
        let store = SharedStore::new_in_memory();

        let config = VesselConfig::default();
        let node = VesselNode::new(config);

        let exec_result = json!({
            "outcomes": [VesselOutcome::MergeBlocked {
                ticket_id: Some("T-42".to_string()),
                pr_number: 42,
                reason: "Merge conflict".to_string(),
            }],
            "has_work": true,
        });

        let action = node.post(&store, exec_result).await.unwrap();
        assert_eq!(action.as_str(), Action::DEPLOY_FAILED);

        let events = store.get_events_since(0).await;
        assert!(events.iter().any(|e| e.event_type == "merge_blocked"));
    }

    #[tokio::test]
    async fn test_post_handles_no_work() {
        let store = SharedStore::new_in_memory();

        let config = VesselConfig::default();
        let node = VesselNode::new(config);

        let exec_result = json!({
            "outcomes": [],
            "has_work": false,
        });

        let action = node.post(&store, exec_result).await.unwrap();
        assert_eq!(action.as_str(), "no_work");
    }

    #[tokio::test]
    async fn test_update_ticket_status() {
        let store = SharedStore::new_in_memory();
        store.set("tickets", json!([
            {"id": "T-1", "status": {"type": "open"}},
            {"id": "T-42", "status": {"type": "in_progress"}},
        ])).await;

        let config = VesselConfig::default();
        let node = VesselNode::new(config);

        node.update_ticket_status(&store, "T-42", "merged").await;

        let tickets: Vec<Value> = store.get_typed("tickets").await.unwrap();
        let ticket = tickets.iter().find(|t| t["id"] == "T-42").unwrap();
        assert_eq!(ticket["status"]["type"], "merged");
    }

    #[tokio::test]
    async fn test_remove_from_pending_prs() {
        let store = SharedStore::new_in_memory();
        store.set("pending_prs", json!([
            {"number": 1, "ticket_id": "T-1"},
            {"number": 42, "ticket_id": "T-42"},
            {"number": 100, "ticket_id": "T-100"},
        ])).await;

        let config = VesselConfig::default();
        let node = VesselNode::new(config);

        node.remove_from_pending_prs(&store, 42).await;

        let pending: Vec<Value> = store.get_typed("pending_prs").await.unwrap();
        assert_eq!(pending.len(), 2);
        assert!(pending.iter().all(|pr| pr["number"] != 42));
    }
}
