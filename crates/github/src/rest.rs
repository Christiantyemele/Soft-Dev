// crates/github/src/rest.rs
//
// Direct GitHub REST API client for operations that require low-latency
// or precise control (CI polling, merge execution).
//
// Separation of concerns: McpGithubClient handles high-level operations
// via MCP subprocess; this handles direct REST calls for VESSEL's needs.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};
use pocketflow_core::{CiStatus, MergeMethod, MergeResult, PrInfo, PrState};

const GITHUB_API_BASE: &str = "https://api.github.com";

/// Direct GitHub REST API client for CI status polling and merge operations.
#[derive(Clone)]
pub struct GithubRestClient {
    client: reqwest::Client,
    token: String,
}

impl GithubRestClient {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("AgentFlow-VESSEL/0.1")
                .build()
                .expect("Failed to build reqwest client"),
            token: token.into(),
        }
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.token)
    }

    async fn get_json<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<T> {
        debug!(url, "GitHub API GET");
        let resp = self
            .client
            .get(url)
            .header("Authorization", self.auth_header())
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await
            .context("GitHub API request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API error {}: {}", status, body);
        }

        resp.json::<T>().await.context("Failed to parse GitHub response")
    }

    async fn put_json<T: for<'de> Deserialize<'de>, B: Serialize>(&self, url: &str, body: &B) -> Result<T> {
        debug!(url, "GitHub API PUT");
        let resp = self
            .client
            .put(url)
            .header("Authorization", self.auth_header())
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .json(body)
            .send()
            .await
            .context("GitHub API PUT request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API error {}: {}", status, body);
        }

        resp.json::<T>().await.context("Failed to parse GitHub response")
    }

    // ── CI Status Polling ────────────────────────────────────────────────

    /// Get combined CI status for a commit ref.
    /// Returns the aggregated status across all status contexts.
    pub async fn get_combined_status(&self, owner: &str, repo: &str, ref_sha: &str) -> Result<CiStatus> {
        let url = format!("{}/repos/{}/{}/commits/{}/status", GITHUB_API_BASE, owner, repo, ref_sha);
        let resp: CombinedStatusResponse = self.get_json(&url).await?;
        Ok(map_status_state(&resp.state))
    }

    /// Get check suites for a commit ref.
    /// Returns the aggregated status across all check runs.
    pub async fn get_check_suites_status(&self, owner: &str, repo: &str, ref_sha: &str) -> Result<CiStatus> {
        let url = format!("{}/repos/{}/{}/commits/{}/check-suites", GITHUB_API_BASE, owner, repo, ref_sha);
        let resp: CheckSuitesResponse = self.get_json(&url).await?;
        
        if resp.check_suites.is_empty() {
            return Ok(CiStatus::Success);
        }

        let mut has_pending = false;
        for suite in &resp.check_suites {
            match suite.status.as_str() {
                "queued" | "in_progress" | "pending" => has_pending = true,
                "completed" => {
                    if suite.conclusion.as_deref() == Some("failure") 
                        || suite.conclusion.as_deref() == Some("timed_out")
                        || suite.conclusion.as_deref() == Some("cancelled") {
                        return Ok(CiStatus::Failure);
                    }
                }
                _ => {}
            }
        }

        if has_pending {
            Ok(CiStatus::Pending)
        } else {
            Ok(CiStatus::Success)
        }
    }

    /// Get the overall CI status (combines check suites and status API).
    pub async fn get_ci_status(&self, owner: &str, repo: &str, ref_sha: &str) -> Result<CiStatus> {
        let combined = self.get_combined_status(owner, repo, ref_sha).await?;
        if combined.is_terminal() {
            return Ok(combined);
        }

        let checks = self.get_check_suites_status(owner, repo, ref_sha).await?;
        if checks.is_terminal() && checks != CiStatus::Success {
            return Ok(checks);
        }

        if combined == CiStatus::Pending || checks == CiStatus::Pending {
            Ok(CiStatus::Pending)
        } else {
            Ok(CiStatus::Success)
        }
    }

    // ── PR Operations ─────────────────────────────────────────────────────

    /// Get PR details including head SHA and state.
    pub async fn get_pull_request(&self, owner: &str, repo: &str, pr_number: u64) -> Result<PrInfo> {
        let url = format!("{}/repos/{}/{}/pulls/{}", GITHUB_API_BASE, owner, repo, pr_number);
        let resp: PullRequestResponse = self.get_json(&url).await?;
        
        Ok(PrInfo {
            number: resp.number,
            head_sha: resp.head.sha,
            head_branch: resp.head.ref_field,
            base_branch: resp.base.ref_field,
            ticket_id: extract_ticket_id(&resp.title, &resp.body),
            title: resp.title,
            state: match resp.state.as_str() {
                "open" => PrState::Open,
                "closed" if resp.merged => PrState::Merged,
                _ => PrState::Closed,
            },
        })
    }

    /// Merge a pull request.
    pub async fn merge_pull_request(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        commit_title: &str,
        merge_method: MergeMethod,
    ) -> Result<MergeResult> {
        let url = format!("{}/repos/{}/{}/pulls/{}/merge", GITHUB_API_BASE, owner, repo, pr_number);
        
        let body = MergeRequestBody {
            commit_title: Some(commit_title.to_string()),
            merge_method,
        };

        let resp: MergeResponse = self.put_json(&url, &body).await?;
        
        Ok(MergeResult {
            merged: resp.merged,
            sha: resp.sha,
            message: resp.message,
        })
    }

    /// Check if a PR is already merged (for startup reconciliation).
    pub async fn is_pr_merged(&self, owner: &str, repo: &str, pr_number: u64) -> Result<bool> {
        match self.get_pull_request(owner, repo, pr_number).await {
            Ok(info) => Ok(info.state == PrState::Merged),
            Err(e) => {
                warn!(error = %e, pr = pr_number, "Failed to check PR merge status");
                Ok(false)
            }
        }
    }
}

// ── Helper Functions ──────────────────────────────────────────────────────

fn map_status_state(state: &str) -> CiStatus {
    match state.to_lowercase().as_str() {
        "pending" => CiStatus::Pending,
        "success" => CiStatus::Success,
        "failure" => CiStatus::Failure,
        "error" => CiStatus::Error,
        _ => CiStatus::Pending,
    }
}

fn extract_ticket_id(title: &str, body: &Option<String>) -> Option<String> {
    let patterns = [
        regex::Regex::new(r"T-(\d+)").ok(),
        regex::Regex::new(r"#(\d+)").ok(),
    ];

    for pattern in patterns.iter().flatten() {
        if let Some(caps) = pattern.captures(title) {
            return Some(format!("T-{}", &caps[1]));
        }
        if let Some(body) = body {
            if let Some(caps) = pattern.captures(body) {
                return Some(format!("T-{}", &caps[1]));
            }
        }
    }
    None
}

// ── API Response Types ────────────────────────────────────────────────────

#[derive(Deserialize)]
struct CombinedStatusResponse {
    state: String,
}

#[derive(Deserialize)]
struct CheckSuitesResponse {
    check_suites: Vec<CheckSuite>,
}

#[derive(Deserialize)]
struct CheckSuite {
    status: String,
    conclusion: Option<String>,
}

#[derive(Deserialize)]
struct PullRequestResponse {
    number: u64,
    title: String,
    body: Option<String>,
    state: String,
    merged: bool,
    head: PrBranch,
    base: PrBranch,
}

#[derive(Deserialize)]
struct PrBranch {
    sha: String,
    #[serde(rename = "ref")]
    ref_field: String,
}

#[derive(Serialize)]
struct MergeRequestBody {
    #[serde(rename = "commit_title")]
    commit_title: Option<String>,
    merge_method: MergeMethod,
}

#[derive(Deserialize)]
struct MergeResponse {
    merged: bool,
    sha: Option<String>,
    message: String,
}
