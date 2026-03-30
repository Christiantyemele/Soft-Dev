// crates/pair-harness/src/types.rs
//! Core types for the pair-harness system.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Filesystem events detected by the watcher.
/// These drive the event-driven harness state machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FsEvent {
    /// FORGE submitted a segment (WORKLOG.md modified)
    WorklogUpdated,
    /// FORGE finished planning (PLAN.md created)
    PlanWritten,
    /// SENTINEL reviewed plan (CONTRACT.md created)
    ContractWritten,
    /// SENTINEL finished segment-N evaluation
    SegmentEvalWritten(u32),
    /// SENTINEL approved all segments (final-review.md created)
    FinalReviewWritten,
    /// Terminal signal (PR_OPENED, BLOCKED, FUEL_EXHAUSTED)
    StatusJsonWritten,
    /// Context reset requested (HANDOFF.md created)
    HandoffWritten,
}

/// Ticket information for assignment to a pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    /// Ticket identifier (e.g., "T-42")
    pub id: String,
    /// GitHub issue number
    pub issue_number: u64,
    /// Ticket title
    pub title: String,
    /// Ticket description/body
    pub body: String,
    /// GitHub issue URL
    pub url: String,
    /// Files that will be touched (for initial locking)
    pub touched_files: Vec<String>,
    /// Acceptance criteria extracted from the issue
    pub acceptance_criteria: Vec<String>,
}

/// Configuration for a pair slot.
#[derive(Debug, Clone)]
pub struct PairConfig {
    /// Pair identifier (e.g., "pair-1")
    pub pair_id: String,
    /// Path to the Git worktree for this pair
    pub worktree: PathBuf,
    /// Path to the shared directory for FORGE-SENTINEL communication
    pub shared: PathBuf,
    /// Redis URL for shared store
    pub redis_url: String,
    /// GitHub token for MCP tools
    pub github_token: String,
    /// Maximum number of context resets allowed
    pub max_resets: u32,
    /// Timeout in seconds for watchdog (default: 1200 = 20 minutes)
    pub watchdog_timeout_secs: u64,
}

impl PairConfig {
    /// Create a new pair configuration.
    pub fn new(
        pair_id: impl Into<String>,
        project_root: &std::path::Path,
        redis_url: impl Into<String>,
        github_token: impl Into<String>,
    ) -> Self {
        let pair_id = pair_id.into();
        Self {
            worktree: project_root.join("worktrees").join(&pair_id),
            shared: project_root.join(".sprintless").join("pairs").join(&pair_id).join("shared"),
            pair_id,
            redis_url: redis_url.into(),
            github_token: github_token.into(),
            max_resets: 10,
            watchdog_timeout_secs: 1200,
        }
    }
}

/// Outcome of a pair's work on a ticket.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PairOutcome {
    /// PR was opened successfully
    PrOpened {
        pr_url: String,
        pr_number: u64,
        branch: String,
    },
    /// Pair is blocked (needs human intervention)
    Blocked {
        reason: String,
        blockers: Vec<Blocker>,
    },
    /// Fuel exhausted (too many context resets or timeout)
    FuelExhausted {
        reason: String,
        reset_count: u32,
    },
}

/// A blocker preventing progress.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Blocker {
    /// Type of blocker
    #[serde(rename = "type")]
    pub blocker_type: String,
    /// Human-readable description
    pub description: String,
    /// Suggested action for NEXUS
    pub nexus_action: String,
}

/// Status written to STATUS.json by FORGE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusJson {
    /// Current status
    pub status: String,
    /// Pair identifier
    pub pair: String,
    /// Ticket identifier
    pub ticket_id: String,
    /// PR URL (if PR_OPENED)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr_url: Option<String>,
    /// PR number (if PR_OPENED)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr_number: Option<u64>,
    /// Branch name
    pub branch: String,
    /// Files changed
    pub files_changed: Vec<String>,
    /// Test results
    pub test_results: TestResults,
    /// Number of segments completed
    pub segments_completed: u32,
    /// Number of context resets
    pub context_resets: u32,
    /// Whether SENTINEL approved
    pub sentinel_approved: bool,
    /// Active blockers (if BLOCKED)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub blockers: Vec<Blocker>,
    /// Elapsed time in milliseconds
    pub elapsed_ms: u64,
    /// Timestamp
    pub timestamp: String,
}

/// Test results summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResults {
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
}

/// Contract status written by SENTINEL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    /// Status: AGREED or ISSUES
    pub status: String,
    /// Contract terms (definition of done)
    pub terms: Vec<ContractTerm>,
    /// Objections (if status is ISSUES)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub objections: Vec<String>,
}

/// A single contract term.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractTerm {
    pub criterion: String,
    pub verification: String,
}

/// Segment evaluation written by SENTINEL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentEval {
    /// Segment number
    pub segment: u32,
    /// Verdict: APPROVED or CHANGES_REQUESTED
    pub verdict: String,
    /// Specific feedback items (if CHANGES_REQUESTED)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub feedback: Vec<FeedbackItem>,
}

/// A specific feedback item for changes requested.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackItem {
    pub file: String,
    pub line: u32,
    pub problem: String,
    pub fix: String,
}

/// Final review written by SENTINEL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalReview {
    /// Verdict: APPROVED or REJECTED
    pub verdict: String,
    /// PR description (if APPROVED)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr_description: Option<String>,
    /// Remaining issues (if REJECTED)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub issues: Vec<String>,
}

/// File lock metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileLock {
    /// Pair that owns the lock
    pub pair: String,
    /// File path (relative to project root)
    pub file: String,
    /// When the lock was acquired
    pub acquired_at: String,
}

impl FileLock {
    /// Create a new file lock for a pair.
    pub fn new(pair: impl Into<String>, file: impl Into<String>) -> Self {
        Self {
            pair: pair.into(),
            file: file.into(),
            acquired_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}