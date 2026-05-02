# IdentityManager — Centralized Agent Identity Management

The `IdentityManager` provides a unified, thread-safe interface for managing agent identities across the AgentFlow system. It centralizes configuration resolution for all agents, including model backends, GitHub tokens, and routing keys.

## Overview

### Problem Solved

Previously, agent identity information was scattered across multiple locations:
- `Registry` struct loaded from `registry.json`
- Per-agent config structs (`VesselConfig`, `LoreConfig`, etc.)
- Direct environment variable access in various crates
- Token resolution duplicated across agents

The `IdentityManager` consolidates this into a single source of truth with caching for performance.

### Key Features

- **Thread-safe**: `Arc<RwLock<...>>` enables sharing across threads/agents
- **Token caching**: Avoids repeated environment variable lookups
- **Hot-reloadable**: Configuration can be reloaded from disk without restart
- **Instance-aware**: Handles both base roles (`forge`) and instance slots (`forge-1`, `forge-2`)

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     IdentityManager                          │
│  ┌─────────────────┐    ┌─────────────────────────────────┐ │
│  │   Registry      │    │        TokenCache                │ │
│  │ (Arc<RwLock>)   │    │     (Arc<RwLock>)                │ │
│  │                 │    │                                  │ │
│  │  - nexus        │    │  {"nexus": "ghp_xxx",            │ │
│  │  - forge (x2)   │    │   "forge": "ghp_yyy", ...}       │ │
│  │  - sentinel     │    │                                  │ │
│  │  - vessel       │    │                                  │ │
│  │  - lore         │    │                                  │ │
│  └─────────────────┘    └─────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
         │                    │
         ▼                    ▼
    resolve_entry()    resolve_token_for_entry()
         │                    │
         └────────┬───────────┘
                  ▼
         AgentIdentity {
           id: "forge-1",
           role: AgentRole::Forge,
           model_backend: Some("accounts/fireworks/models/kimi-k2p6"),
           github_token: "ghp_yyy",
           routing_key: Some("forge-key"),
           ...
         }
```

## Core Types

### AgentRole

Distinguishes the five agent types in the system:

```rust
pub enum AgentRole {
    Nexus,    // Orchestration coordinator
    Forge,    // Implementation worker (multi-instance)
    Sentinel, // Code reviewer
    Vessel,   // Merge/deploy agent
    Lore,     // Documentation agent
}
```

### AgentIdentity

Complete resolved identity for a single agent instance:

```rust
pub struct AgentIdentity {
    pub id: String,                    // "nexus", "forge-1", "sentinel", etc.
    pub role: AgentRole,               // Role classification
    pub instance_num: Option<u32>,     // Instance number (only for forge-N)
    pub model_backend: Option<String>, // LLM model to use
    pub github_token: String,          // Resolved GitHub PAT
    pub routing_key: Option<String>,   // Proxy routing key
    pub cli: String,                   // CLI type: "claude" | "gemini" | "codex"
    pub active: bool,                  // Whether agent is active
}
```

### IdentityManager

The main manager struct, cloneable for sharing:

```rust
#[derive(Debug, Clone)]
pub struct IdentityManager {
    registry: Arc<RwLock<Registry>>,
    token_cache: Arc<RwLock<TokenCache>>,
}
```

## Usage Examples

### Loading and Basic Usage

```rust
use config::identity::{IdentityManager, AgentRole};

// Load from default registry path
let manager = IdentityManager::load("orchestration/agent/registry.json")?;

// Get identity for a specific agent
let nexus = manager.get_identity("nexus")?;
println!("Nexus model: {:?}", nexus.model_backend);
println!("Nexus token: {}...", &nexus.github_token[..10]);

// Get forge instance identity
let forge_1 = manager.get_identity("forge-1")?;
assert_eq!(forge_1.role, AgentRole::Forge);
assert_eq!(forge_1.instance_num, Some(1));
```

### Getting All Identities for a Role

```rust
// Get all forge instances (forge-1, forge-2, ...)
let forge_identities = manager.get_identities_for_role(AgentRole::Forge)?;
for identity in &forge_identities {
    println!("Forge instance: {} with model {:?}", identity.id, identity.model_backend);
}

// Get single identity for non-multi-instance roles
let sentinel = manager.get_identities_for_role(AgentRole::Sentinel)?;
assert_eq!(sentinel.len(), 1);
```

### Worker Slot Resolution

```rust
// Get all worker slot names (for SharedStore initialization)
let slots = manager.all_worker_slots()?;
// Returns: ["nexus", "forge-1", "forge-2", "sentinel", "vessel", "lore"]

// Get forge slots only
let forge_slots = manager.forge_slots()?;
// Returns: ["forge-1", "forge-2"]
```

### GitHub Token Resolution

```rust
// Resolve token for any agent by ID
let token = manager.resolve_github_token("forge-1")?;

// Tokens are cached after first resolution
let token2 = manager.resolve_github_token("forge-2"); // Uses cache
```

### Hot Reloading

```rust
// Reload registry from disk (e.g., after config change)
manager.reload("orchestration/agent/registry.json")?;

// Clear token cache (e.g., after token rotation)
manager.clear_cache()?;
```

### Sharing Across Threads

```rust
// IdentityManager is Clone and thread-safe
let manager = IdentityManager::load("orchestration/agent/registry.json")?;

// Clone for each thread/agent
let manager_for_nexus = manager.clone();
let manager_for_forge = manager.clone();

// Use in parallel
std::thread::spawn(move || {
    let identity = manager_for_nexus.get_identity("nexus").unwrap();
    // ... use identity
});
```

## Configuration

### registry.json Format

The identity manager reads from `orchestration/agent/registry.json`:

```json
{
  "team": [
    {
      "id": "nexus",
      "cli": "claude",
      "active": true,
      "instances": 1,
      "model_backend": "accounts/fireworks/models/kimi-k2p6",
      "routing_key": "nexus-key",
      "github_token_env": "AGENT_NEXUS_GITHUB_TOKEN"
    },
    {
      "id": "forge",
      "cli": "claude",
      "active": true,
      "instances": 2,
      "model_backend": "accounts/fireworks/models/kimi-k2p6",
      "routing_key": "forge-key",
      "github_token_env": "AGENT_FORGE_GITHUB_TOKEN"
    }
  ]
}
```

### GitHub Token Resolution Priority

1. If `github_token_env` is set: read from that environment variable
2. Fallback: read from `GITHUB_PERSONAL_ACCESS_TOKEN`

This allows:
- Per-agent tokens with different scopes (recommended for production)
- Single shared token for development/testing

## Integration Points

### For Nexus

```rust
// In agent-nexus
let identity = identity_manager.get_identity("nexus")?;
let runner = AgentRunner::from_env_with_token(
    identity.model_backend.as_deref(),
    &identity.github_token,
)?;
```

### For Forge/Sentinel (via pair-harness)

```rust
// In pair-harness
let identity = identity_manager.get_identity(&worker_id)?;
let routing_key = identity.routing_key.unwrap_or_default();

// Pass to spawned process
cmd.env("ANTHROPIC_API_KEY", routing_key);
cmd.env("SPRINTLESS_GITHUB_TOKEN", &identity.github_token);
```

### For Vessel/Lore

```rust
// In agent-vessel/agent-lore
let identity = identity_manager.get_identity("vessel")?;
// Use identity.github_token for GitHub API calls
```

## Testing

The module includes comprehensive unit tests. Run with:

```bash
cargo test -p config --lib identity
```

Tests use a fallback `GITHUB_PERSONAL_ACCESS_TOKEN` to avoid requiring per-agent tokens in the test environment.

## API Reference

### IdentityManager Methods

| Method | Description |
|--------|-------------|
| `load(path)` | Load from registry.json at given path |
| `from_registry(registry)` | Create from existing Registry struct |
| `reload(path)` | Hot-reload registry from disk |
| `registry()` | Get cloned Registry |
| `get_identity(agent_id)` | Get identity for specific agent |
| `resolve_github_token(agent_id)` | Resolve GitHub token for agent |
| `get_identities_for_role(role)` | Get all identities for a role |
| `all_identities()` | Get all active identities |
| `all_worker_slots()` | Get all worker slot names |
| `forge_slots()` | Get forge slot names only |
| `get_model_backend(agent_id)` | Get model backend for agent |
| `get_routing_key(agent_id)` | Get routing key for agent |
| `clear_cache()` | Clear token cache |

### AgentIdentity Methods

| Method | Description |
|--------|-------------|
| `is_forge_instance()` | Returns true if role is Forge with instance_num |
| `slot_name()` | Returns the agent ID (alias for `id`) |
| `is_active()` | Returns the active status |

## Migration Guide

### From Direct Registry Usage

**Before:**
```rust
let registry = Registry::load("orchestration/agent/registry.json")?;
let entry = registry.get("nexus").unwrap();
let model = entry.model_backend.clone();
let token = registry.resolve_github_token("nexus")?;
```

**After:**
```rust
let manager = IdentityManager::load("orchestration/agent/registry.json")?;
let identity = manager.get_identity("nexus")?;
// identity.model_backend and identity.github_token already resolved
```

### From Per-Agent Config Structs

**Before (VesselConfig):**
```rust
let registry = Registry::load(path)?;
let config = VesselConfig::from_registry(&registry)?;
// config.github_token, config.model_backend
```

**After:**
```rust
let manager = IdentityManager::load(path)?;
let identity = manager.get_identity("vessel")?;
// identity.github_token, identity.model_backend
```

## Future Enhancements

Potential improvements for the identity system:

1. **Per-agent timeout configuration**: Add `timeout_secs` field to registry
2. **Agent capabilities**: Define what tools/actions each agent can perform
3. **Identity rotation**: Support for automatic token rotation
4. **Audit logging**: Track identity access for security
5. **Dynamic agent registration**: Add/remove agents at runtime
