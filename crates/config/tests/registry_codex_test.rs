// crates/config/tests/registry_codex_test.rs
//! Integration tests for Codex CLI backend support in registry.

use config::registry::{CliBackend, Registry};
use std::io::Write;
use tempfile::NamedTempFile;

fn write_temp(content: &str) -> NamedTempFile {
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(content.as_bytes()).unwrap();
    f
}

#[test]
fn test_registry_with_codex_agents() {
    let json = r#"{
      "default_cli": "claude",
      "team": [
        { "id": "nexus",    "cli": "codex",  "active": true,  "instances": 1, "model_backend": "openai/gpt-4o", "routing_key": "nexus-key" },
        { "id": "forge",    "cli": "codex",  "active": true,  "instances": 2, "model_backend": "openai/gpt-4o", "routing_key": "forge-key" },
        { "id": "sentinel", "cli": "claude", "active": true,  "instances": 1, "model_backend": "anthropic/claude-sonnet-4-5", "routing_key": "sentinel-key" },
        { "id": "vessel",   "cli": "claude", "active": true,  "instances": 1, "model_backend": "anthropic/claude-sonnet-4-5", "routing_key": "vessel-key" },
        { "id": "lore",     "cli": "codex",  "active": true,  "instances": 1, "model_backend": "openai/gpt-4o", "routing_key": "lore-key" }
      ]
    }"#;

    let f = write_temp(json);
    let reg = Registry::load(f.path()).unwrap();

    // Check default_cli
    assert_eq!(reg.default_cli, "claude");

    // Check individual agents have correct CLI backend
    let nexus = reg.get("nexus").unwrap();
    assert_eq!(nexus.cli_backend(&reg.default_cli), CliBackend::Codex);

    let forge = reg.get("forge").unwrap();
    assert_eq!(forge.cli_backend(&reg.default_cli), CliBackend::Codex);

    let sentinel = reg.get("sentinel").unwrap();
    assert_eq!(sentinel.cli_backend(&reg.default_cli), CliBackend::Claude);

    let lore = reg.get("lore").unwrap();
    assert_eq!(lore.cli_backend(&reg.default_cli), CliBackend::Codex);
}

#[test]
fn test_registry_default_cli_fallback() {
    let json = r#"{
      "default_cli": "codex",
      "team": [
        { "id": "nexus", "cli": "", "active": true, "instances": 1 },
        { "id": "forge", "cli": "claude", "active": true, "instances": 1 }
      ]
    }"#;

    let f = write_temp(json);
    let reg = Registry::load(f.path()).unwrap();

    // nexus has empty cli, should use default (codex)
    let nexus = reg.get("nexus").unwrap();
    assert_eq!(nexus.cli_backend(&reg.default_cli), CliBackend::Codex);

    // forge has explicit claude
    let forge = reg.get("forge").unwrap();
    assert_eq!(forge.cli_backend(&reg.default_cli), CliBackend::Claude);
}

#[test]
fn test_registry_all_codex() {
    let json = r#"{
      "default_cli": "codex",
      "team": [
        { "id": "nexus",    "cli": "codex", "active": true, "instances": 1 },
        { "id": "forge",    "cli": "codex", "active": true, "instances": 2 },
        { "id": "sentinel", "cli": "codex", "active": true, "instances": 1 },
        { "id": "vessel",   "cli": "codex", "active": true, "instances": 1 },
        { "id": "lore",     "cli": "codex", "active": true, "instances": 1 }
      ]
    }"#;

    let f = write_temp(json);
    let reg = Registry::load(f.path()).unwrap();

    // All agents should use Codex
    for entry in reg.active_agents() {
        assert_eq!(entry.cli_backend(&reg.default_cli), CliBackend::Codex);
    }
}

#[test]
fn test_cli_backend_from_str() {
    assert_eq!(CliBackend::from_str("claude"), CliBackend::Claude);
    assert_eq!(CliBackend::from_str("CLAUDE"), CliBackend::Claude);
    assert_eq!(CliBackend::from_str("Claude"), CliBackend::Claude);
    
    assert_eq!(CliBackend::from_str("codex"), CliBackend::Codex);
    assert_eq!(CliBackend::from_str("CODEX"), CliBackend::Codex);
    assert_eq!(CliBackend::from_str("Codex"), CliBackend::Codex);
    
    // Unknown values default to Claude
    assert_eq!(CliBackend::from_str("unknown"), CliBackend::Claude);
    assert_eq!(CliBackend::from_str(""), CliBackend::Claude);
}

#[test]
fn test_cli_backend_as_str() {
    assert_eq!(CliBackend::Claude.as_str(), "claude");
    assert_eq!(CliBackend::Codex.as_str(), "codex");
}

#[test]
fn test_registry_backward_compatible() {
    // Test that old registry format (without default_cli) still works
    let json = r#"{
      "team": [
        { "id": "nexus", "cli": "claude", "active": true, "instances": 1 },
        { "id": "forge", "cli": "claude", "active": true, "instances": 2 }
      ]
    }"#;

    let f = write_temp(json);
    let reg = Registry::load(f.path()).unwrap();

    // Default should be "claude" for backward compatibility
    assert_eq!(reg.default_cli, "claude");

    // All agents should use Claude
    for entry in reg.active_agents() {
        assert_eq!(entry.cli_backend(&reg.default_cli), CliBackend::Claude);
    }
}