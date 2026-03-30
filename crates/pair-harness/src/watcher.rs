// crates/pair-harness/src/watcher.rs
//! Filesystem watcher for event-driven harness.
//!
//! Uses notify crate for cross-platform inotify/FSEvents support.

use anyhow::{Context, Result};
use notify::{Watcher, RecursiveMode, Event, EventKind, Config, RecommendedWatcher};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;
use tracing::{debug, warn, error};
use crate::types::FsEvent;

/// Watches the shared directory for file changes.
pub struct SharedDirWatcher {
    /// The underlying notify watcher
    watcher: RecommendedWatcher,
    /// Receiver for filesystem events
    receiver: Receiver<FsEvent>,
}

impl SharedDirWatcher {
    /// Create a new watcher for the shared directory.
    pub fn new(shared_dir: &Path) -> Result<Self> {
        let (tx, rx) = channel::<FsEvent>();
        
        let watcher = Self::create_watcher(tx.clone(), shared_dir)?;
        
        Ok(Self {
            watcher,
            receiver: rx,
        })
    }

    /// Create and configure the notify watcher.
    fn create_watcher(tx: Sender<FsEvent>, shared_dir: &Path) -> Result<RecommendedWatcher> {
        // Create a watcher with a callback
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            match res {
                Ok(event) => {
                    if let Some(fs_event) = Self::classify_event(&event) {
                        debug!(event = ?fs_event, paths = ?event.paths, "Filesystem event detected");
                        let _ = tx.send(fs_event);
                    }
                }
                Err(e) => {
                    error!(error = %e, "Watch error");
                }
            }
        }).context("Failed to create filesystem watcher")?;

        // Configure for low latency
        watcher.configure(Config::default()
            .with_poll_interval(Duration::from_millis(100)))
            .context("Failed to configure watcher")?;

        // Watch the shared directory (non-recursive since we only care about top-level files)
        watcher.watch(shared_dir, RecursiveMode::NonRecursive)
            .context("Failed to start watching shared directory")?;

        debug!(path = %shared_dir.display(), "Started watching shared directory");
        Ok(watcher)
    }

    /// Classify a filesystem event into our FsEvent type.
    fn classify_event(event: &Event) -> Option<FsEvent> {
        // Only care about create and modify events
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Access(_) => {}
            _ => return None,
        }

        // Check each path in the event
        for path in &event.paths {
            let filename = path.file_name()?.to_str()?;
            
            let fs_event = match filename {
                "PLAN.md" => Some(FsEvent::PlanWritten),
                "CONTRACT.md" => Some(FsEvent::ContractWritten),
                "WORKLOG.md" => Some(FsEvent::WorklogUpdated),
                "final-review.md" => Some(FsEvent::FinalReviewWritten),
                "STATUS.json" => Some(FsEvent::StatusJsonWritten),
                "HANDOFF.md" => Some(FsEvent::HandoffWritten),
                s if s.starts_with("segment-") && s.ends_with("-eval.md") => {
                    // Extract segment number from "segment-N-eval.md"
                    let n = s
                        .strip_prefix("segment-")?
                        .strip_suffix("-eval.md")?
                        .parse::<u32>()
                        .ok()?;
                    Some(FsEvent::SegmentEvalWritten(n))
                }
                _ => None,
            };

            if fs_event.is_some() {
                return fs_event;
            }
        }

        None
    }

    /// Try to receive an event without blocking.
    pub fn try_recv(&self) -> Option<FsEvent> {
        self.receiver.try_recv().ok()
    }

    /// Receive an event with a timeout.
    pub fn recv_timeout(&self, timeout: Duration) -> Option<FsEvent> {
        self.receiver.recv_timeout(timeout).ok()
    }

    /// Get a reference to the underlying receiver for use in async contexts.
    pub fn receiver(&self) -> &Receiver<FsEvent> {
        &self.receiver
    }
}

/// Async wrapper for the watcher that integrates with tokio.
pub struct AsyncWatcher {
    /// The underlying watcher
    watcher: SharedDirWatcher,
}

impl AsyncWatcher {
    /// Create a new async watcher.
    pub fn new(shared_dir: &Path) -> Result<Self> {
        let watcher = SharedDirWatcher::new(shared_dir)?;
        Ok(Self { watcher })
    }

    /// Receive events as a stream.
    pub fn recv(&self) -> Option<FsEvent> {
        // This is a blocking call, but that's okay for our event-driven architecture
        self.watcher.receiver().recv().ok()
    }

    /// Try to receive without blocking.
    pub fn try_recv(&self) -> Option<FsEvent> {
        self.watcher.try_recv()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_classify_plan_event() {
        let dir = tempdir().unwrap();
        let shared = dir.path();
        
        let watcher = SharedDirWatcher::new(shared).unwrap();
        
        // Write PLAN.md
        let plan_path = shared.join("PLAN.md");
        fs::write(&plan_path, "# Plan\n").unwrap();
        
        // Give the watcher time to detect
        std::thread::sleep(Duration::from_millis(200));
        
        let event = watcher.try_recv();
        assert!(matches!(event, Some(FsEvent::PlanWritten)));
    }

    #[test]
    fn test_classify_segment_eval_event() {
        let dir = tempdir().unwrap();
        let shared = dir.path();
        
        let watcher = SharedDirWatcher::new(shared).unwrap();
        
        // Write segment-3-eval.md
        let eval_path = shared.join("segment-3-eval.md");
        fs::write(&eval_path, "# Eval\n").unwrap();
        
        // Give the watcher time to detect
        std::thread::sleep(Duration::from_millis(200));
        
        let event = watcher.try_recv();
        assert!(matches!(event, Some(FsEvent::SegmentEvalWritten(3))));
    }
}