// crates/agent-vessel/src/ci_poller.rs
//
// CI Status Polling — separated for modularity and reusability.
// Handles the "gate" phase of VESSEL's workflow.

use anyhow::Result;
use pocketflow_core::{CiPollConfig, CiStatus, PrInfo};
use tracing::{debug, info, warn};

/// CI Poller — polls GitHub for CI status until terminal state or timeout.
pub struct CiPoller {
    config: CiPollConfig,
    client: github::GithubRestClient,
}

impl CiPoller {
    pub fn new(config: CiPollConfig, client: github::GithubRestClient) -> Self {
        Self { config, client }
    }

    /// Get a reference to the GitHub client for additional operations.
    pub fn client(&self) -> &github::GithubRestClient {
        &self.client
    }

    /// Poll CI status until it reaches a terminal state or times out.
    /// Returns the final status.
    pub async fn poll_until_terminal(
        &self,
        owner: &str,
        repo: &str,
        pr_info: &PrInfo,
    ) -> Result<CiPollResult> {
        let mut attempts = 0u32;
        
        loop {
            if attempts >= self.config.max_attempts {
                warn!(
                    pr = pr_info.number,
                    attempts,
                    "CI polling timed out after {} attempts",
                    attempts
                );
                return Ok(CiPollResult::Timeout);
            }

            let status = self
                .client
                .get_ci_status(owner, repo, &pr_info.head_sha)
                .await?;

            debug!(pr = pr_info.number, status = ?status, attempt = attempts, "CI status check");

            if status.is_terminal() {
                info!(pr = pr_info.number, status = ?status, "CI reached terminal state");
                return Ok(CiPollResult::Status(status));
            }

            attempts += 1;
            tokio::time::sleep(std::time::Duration::from_secs(self.config.interval_secs)).await;
        }
    }
}

/// Result of CI polling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CiPollResult {
    /// CI reached a terminal status
    Status(CiStatus),
    /// Polling timed out before terminal state
    Timeout,
}

impl CiPollResult {
    pub fn is_success(&self) -> bool {
        matches!(self, CiPollResult::Status(CiStatus::Success))
    }

    pub fn is_failure(&self) -> bool {
        matches!(self, CiPollResult::Status(CiStatus::Failure | CiStatus::Error))
    }

    pub fn is_timeout(&self) -> bool {
        matches!(self, CiPollResult::Timeout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ci_poll_result_is_success() {
        assert!(CiPollResult::Status(CiStatus::Success).is_success());
        assert!(!CiPollResult::Status(CiStatus::Failure).is_success());
        assert!(!CiPollResult::Timeout.is_success());
    }

    #[test]
    fn test_ci_poll_result_is_failure() {
        assert!(CiPollResult::Status(CiStatus::Failure).is_failure());
        assert!(CiPollResult::Status(CiStatus::Error).is_failure());
        assert!(!CiPollResult::Status(CiStatus::Success).is_failure());
        assert!(!CiPollResult::Timeout.is_failure());
    }

    #[test]
    fn test_ci_poll_result_is_timeout() {
        assert!(CiPollResult::Timeout.is_timeout());
        assert!(!CiPollResult::Status(CiStatus::Success).is_timeout());
    }
}
