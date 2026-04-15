// crates/agent-vessel/src/lib.rs
//
// VESSEL Agent — DevOps Specialist and Merge Gatekeeper.
//
// The only agent authorized to perform destructive/irreversible actions
// (merging PRs, deploying). Ensures CI passes before merging and emits
// ticket_merged events critical for dependency resolution.

pub mod ci_poller;
pub mod merger;
pub mod node;
pub mod notifier;
pub mod types;

pub use ci_poller::CiPoller;
pub use merger::PrMerger;
pub use node::VesselNode;
pub use notifier::VesselNotifier;
pub use types::VesselConfig;
