# AgentFlow - Autonomous AI Development Team

> 🌐 Official site: [openflows.dev](https://openflows.dev)

An autonomous software development team composed of AI agents working in a unified Rust/Tokio flow. The team can take GitHub issues and turn them into working code with pull requests - all autonomously.

![AgentFlow Architecture](image.png)

## Quick Start

```bash
# 1. Clone and setup
git clone https://github.com/The-AgenticFlow/AgentFlow.git
cd AgentFlow
cp .env.example .env
# Edit .env with your API keys

# 2. Start the local proxy (required when gateway doesn't support Anthropic format)
source .env && ./scripts/start_proxy.sh &
# Or if your provider supports Anthropic directly, skip this step

# 3. Verify setup (optional but recommended)
./scripts/check_setup.sh

# 4. Run the orchestration
cargo run --bin real_test
```

## Getting Started

### 📖 Complete Tutorial
**NEW: [TUTORIAL.md](TUTORIAL.md)** - Detailed walkthrough with:
- ✅ Step-by-step setup from zero
- ✅ Expected logs and outputs at each step
- ✅ File structure and locations explained
- ✅ Troubleshooting common issues
- ✅ How to inspect generated code and PRs

### 🚀 Live Flow Walkthrough
**[docs/demo.md](docs/demo.md)** - Step-by-step walkthrough of a live orchestration run with:
- What each log line means as NEXUS discovers issues and assigns work
- How the FORGE-SENTINEL pair communicates through the shared directory
- Where to find generated plans, evaluations, and code changes on disk
- Troubleshooting table for common failures

## The Team

| Agent | Role | Description |
|-------|------|-------------|
| **NEXUS** | Orchestrator | Scrum Master & Tech Lead. Assigns tickets, approves dangerous commands. |
| **FORGE** | Builder | Senior Engineer. Writes code, tests, opens PRs via Claude Code. |
| **SENTINEL** | Reviewer | Security auditor. Reviews PRs, ensures all logic is tested. |
| **VESSEL** | DevOps | Deployment expert. Manages CI/CD and rollbacks. |
| **LORE** | Writer | Documenter. Writes ADRs, maintains project history. |

## Registry System

The registry at [`orchestration/agent/registry.json`](orchestration/agent/registry.json) is the **single source of truth** for team membership, worker scaling, and per-agent routing. NEXUS reloads it on every poll cycle, so changes take effect without restarting the orchestration.

### Structure

```json
{
  "team": [
    {
      "id": "forge",
      "cli": "claude",
      "active": true,
      "instances": 2,
      "model_backend": "accounts/fireworks/models/glm-5",
      "routing_key": "forge-key",
      "github_token_env": "AGENT_FORGE_GITHUB_TOKEN"
    }
  ]
}
```

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Agent name. Used in logs, worktree names (`forge-1`, `forge-2`), and branch names. |
| `cli` | string | CLI tool to spawn. Currently only `"claude"` (Claude Code) is supported. |
| `active` | bool | When `false`, the agent is excluded from orchestration entirely. |
| `instances` | int | Number of parallel worker slots. FORGE uses this directly (`forge-1`, `forge-2`, ...). Other agents with `instances > 1` get numbered slots (`vessel-1`, `vessel-2`). Agents with `instances == 1` use their bare ID (`nexus`, `sentinel`). |
| `model_backend` | string | Model identifier passed to the LLM client. Can be a direct provider path (`anthropic/claude-sonnet-4-5`) or a gateway path (`accounts/fireworks/models/glm-5`). |
| `routing_key` | string | LiteLLM proxy routing key. When `PROXY_URL` is set, this key is used to route requests to the correct backend model. When unset, the agent falls back to direct API access. |
| `github_token_env` | string | Environment variable name that holds the GitHub PAT for this agent. Falls back to `GITHUB_PERSONAL_ACCESS_TOKEN` if not set. Enables per-agent token rotation and scoping. |

### Worker Slots

The registry generates worker slot names that appear throughout the system (worktrees, branches, logs, `STATUS.json`):

```json
{
  "team": [
    { "id": "nexus",    "instances": 1 },   // → slot: "nexus"
    { "id": "forge",    "instances": 2 },   // → slots: "forge-1", "forge-2"
    { "id": "sentinel", "instances": 1 },   // → slot: "sentinel"
    { "id": "vessel",   "instances": 1 },   // → slot: "vessel"
    { "id": "lore",     "instances": 1 }    // → slot: "lore"
  ]
}
```

### Common Operations

**Scale FORGE workers** — change `instances` on the forge entry:
```json
{ "id": "forge", "instances": 4 }  // → forge-1, forge-2, forge-3, forge-4
```

**Disable an agent** — set `active` to `false`:
```json
{ "id": "lore", "active": false }  // LORE will not be invoked
```

**Rotate a GitHub token** — update the env var referenced by `github_token_env`:
```bash
export AGENT_FORGE_GITHUB_TOKEN=ghp_new_token_here
```

**Switch a model backend** — update `model_backend`:
```json
{ "id": "forge", "model_backend": "anthropic/claude-sonnet-4-5" }
```

### Per-Agent GitHub Tokens

Each agent can use a separate GitHub PAT. This is useful for:
- **Rate limit isolation** — one agent hitting rate limits doesn't block others
- **Scope restriction** — give VESSEL only `repo` scope, give FORGE `repo` + `workflow`
- **Token rotation** — rotate one agent's token without affecting the rest of the team

Set the env vars referenced in `github_token_env` before running:
```bash
export AGENT_NEXUS_GITHUB_TOKEN=ghp_nexus_token
export AGENT_FORGE_GITHUB_TOKEN=ghp_forge_token
export AGENT_SENTINEL_GITHUB_TOKEN=ghp_sentinel_token
export AGENT_VESSEL_GITHUB_TOKEN=ghp_vessel_token
export AGENT_LORE_GITHUB_TOKEN=ghp_lore_token
```

If any `github_token_env` variable is unset, the system falls back to `GITHUB_PERSONAL_ACCESS_TOKEN`.

## Architecture

```
AgentFlow/
|-- orchestration/agent/agents/           # Agent personas (nexus.agent.md, forge.agent.md)
|-- crates/
|   |-- agent-nexus/         # Orchestrator node
|   |-- agent-forge/         # Builder node (spawns Claude Code)
|   |-- agent-client/        # LLM client + MCP integration
|   |-- pair-harness/        # Worktree management, process spawning
|   |-- pocketflow-core/     # Flow engine, shared store, routing
|
|-- binary/src/bin/
    |-- real_test.rs          # Live orchestration entry point
    |-- demo.rs               # Mocked demonstration
```

## How It Works

```
                    GitHub Issues
                         |
                         v
                    +---------+
                    |  NEXUS  |  ← Orchestrator: discovers issues, assigns work
                    +---------+
                         |
              ACTION_WORK_ASSIGNED
                         |
                         v
              +--------------------+
              |   FORGE-SENTINEL   |  ← Builder + Reviewer pair
              |       PAIR         |
              +--------------------+
                |                |
                |  PLAN.md       |  CONTRACT.md
                |  CODE          |  segment-N-eval.md
                |  STATUS.json   |  final-review.md
                |                |
                v                v
              PR Opened     CI Checks
                         |
                         v
                    +---------+
                    | VESSEL  |  ← DevOps: polls CI, resolves conflicts, merges
                    +---------+
                         |
              ACTION_DEPLOYED
                         |
                         v
                    +---------+
                    |  LORE   |  ← Writer: ADRs, changelogs, documentation
                    +---------+
                         |
              ACTION_DOCS_COMPLETE
                         |
                         v
                    +---------+
                    |  NEXUS  |  ← Loop: assigns next ticket or halts
                    +---------+
```

### The Orchestration Cycle

1. **NEXUS** fetches open GitHub issues and assigns them to available FORGE workers
2. **FORGE** creates an isolated worktree, writes PLAN.md, then implements code via Claude Code
3. **SENTINEL** reviews the plan (CONTRACT.md), evaluates each code segment, and performs final review
4. **FORGE** opens a PR once SENTINEL approves
5. **VESSEL** polls CI status, detects merge conflicts, attempts resolution, and squash-merges green PRs
6. **LORE** generates documentation: ADRs, changelogs, and project history updates
7. **NEXUS** loops back to assign the next ticket or halts when no work remains

### Shared State

All agents communicate through a **SharedStore** (in-memory or Redis):

| Key | Purpose |
|-----|---------|
| `tickets` | GitHub issues converted to internal work items |
| `worker_slots` | Available FORGE workers and their status |
| `pending_prs` | PRs awaiting CI completion |

### File Artifacts

Each FORGE-SENTINEL pair produces artifacts in two locations:

**Shared directory** (`~/.agentflow/workspaces/<repo>/orchestration/pairs/forge-<N>/shared/`):

| File | Written By | Purpose |
|------|-----------|---------|
| `TICKET.md` | NEXUS | GitHub issue details assigned to this pair |
| `TASK.md` | NEXUS | Task instructions and acceptance criteria |
| `PLAN.md` | FORGE | Implementation plan with segment breakdown |
| `CONTRACT.md` | SENTINEL | Plan review verdict (AGREED or CHANGES_REQUESTED) |
| `WORKLOG.md` | FORGE | Running log of segment implementation progress |
| `segment-N-eval.md` | SENTINEL | Evaluation result for segment N (APPROVED / CHANGES_REQUESTED) |
| `final-review.md` | SENTINEL | Final overall review verdict |
| `HANDOFF.md` | FORGE | Context reset request when context window is full |
| `STATUS.json` | FORGE | Terminal status: `PR_OPENED`, `BLOCKED`, or `FUEL_EXHAUSTED` |

**Worktree** (`~/.agentflow/workspaces/<repo>/worktrees/forge-<N>/`):

```
worktrees/forge-1/
├── src/                     # Code changes on isolated branch
├── tests/                   # Test files added/modified by FORGE
├── PLAN.md                  # Copy of implementation plan
├── WORKLOG.md               # Copy of progress log
├── CONTRACT.md              # Copy of SENTINEL-approved contract
├── segment-1-eval.md        # Copy of segment evaluation
├── final-review.md          # Copy of final review
└── STATUS.json              # Copy of completion status
```

The `STATUS.json` structure:

```json
{
  "status": "PR_OPENED",
  "ticket_id": "T-001",
  "pr_url": "https://github.com/owner/repo/pull/42",
  "pr_number": 42,
  "branch": "forge-1/T-001",
  "files_changed": 5,
  "segments_completed": [
    {"segment": 1, "status": "APPROVED", "eval_file": "segment-1-eval.md"},
    {"segment": 2, "status": "APPROVED", "eval_file": "segment-2-eval.md"}
  ],
  "test_results": {"passed": 12, "failed": 0, "skipped": 0},
  "sentinel_approved": true,
  "context_resets": 0
}
```

## Key Files

| File | Purpose |
|------|---------|
| [`orchestration/agent/agents/nexus.agent.md`](orchestration/agent/agents/nexus.agent.md) | Orchestrator persona and workflow |
| [`orchestration/agent/agents/forge.agent.md`](orchestration/agent/agents/forge.agent.md) | Builder persona and instructions |
| [`orchestration/agent/registry.json`](orchestration/agent/registry.json) | Worker slot definitions |
| [`binary/src/bin/real_test.rs`](binary/src/bin/real_test.rs) | Main entry point |
| [`crates/agent-forge/src/lib.rs`](crates/agent-forge/src/lib.rs) | Forge node implementation |

## Documentation

- **[TUTORIAL.md](TUTORIAL.md)** - Complete tutorial with logs, file structure, and troubleshooting
- **[docs/demo.md](docs/demo.md)** - Live flow walkthrough: logs, file locations, and troubleshooting
- **[docs/setup-claude-cli.md](docs/setup-claude-cli.md)** - Claude CLI setup and troubleshooting
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Development guidelines
- **[docs/forge-sentinel-arch.md](docs/forge-sentinel-arch.md)** - Architecture details

## Per-Agent LLM Routing (LiteLLM Proxy)

Each agent has different workload requirements. Instead of routing all agents through the same expensive model, AgentFlow supports per-agent model routing via a **LiteLLM proxy** — each agent uses the most cost-effective model for its task.

### Default Model Assignments

| Agent | Model | Why |
|-------|-------|-----|
| **FORGE** | `anthropic/claude-sonnet-4-5` | Primary coding agent, needs top-tier reasoning |
| **NEXUS** | `anthropic/claude-sonnet-4-5` | Orchestrator, needs reliable decision-making |
| **SENTINEL** | `gemini/gemini-2.5-pro` | Code review, strong reasoning at lower cost |
| **VESSEL** | `groq/llama-3.3-70b-versatile` | CI/CD scripting, fast and cheap (free tier) |
| **LORE** | `openai/gpt-4o-mini` | Documentation, lightweight task |

### How It Works

1. Claude Code supports `ANTHROPIC_BASE_URL` and `ANTHROPIC_API_KEY` env vars
2. A LiteLLM proxy receives all requests and routes based on the API key (routing key)
3. Each agent is spawned with its own routing key (e.g., `forge-key`, `sentinel-key`)
4. The proxy maps each routing key to the correct backend model via `litellm_config.yaml`
5. Fallback is configured — any provider failure falls back to `anthropic/claude-sonnet-4-5`

### Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `PROXY_URL` | Optional | LiteLLM proxy URL. When set, agents route through the proxy. When unset, agents use direct API access. |
| `PROXY_API_KEY` | Optional | API key for a **hosted** LiteLLM proxy. When set, `ANTHROPIC_API_KEY` is set to this value for auth. When unset, the routing key is used as `ANTHROPIC_API_KEY` (for self-hosted LiteLLM). |
| `ANTHROPIC_API_KEY` | Required* | Anthropic API key (used by FORGE, NEXUS, and as fallback) |
| `GEMINI_API_KEY` | Optional | Google Gemini API key (used by SENTINEL via proxy) |
| `OPENAI_API_KEY` | Optional | OpenAI API key (used by LORE via proxy) |
| `GROQ_API_KEY` | Optional | Groq API key (used by VESSEL via proxy, free tier available) |
| `GATEWAY_URL` | Optional | Remote OpenAI-compatible gateway URL. Used by the local Anthropic proxy to forward requests. Required only when the gateway doesn't support Anthropic protocol. |
| `GATEWAY_API_KEY` | Optional | API key for the remote gateway. Falls back to `PROXY_API_KEY` if unset. |

### Self-Hosted LiteLLM (Docker Compose)

```bash
# Start the proxy, Redis, and agent-team
docker compose up

# Or just the proxy for local dev
docker compose up proxy redis
```

The proxy runs on port 4000 with a health check. See `docker-compose.yml` and `litellm_config.yaml` for configuration.

### Hosted LiteLLM (e.g., LiteLLM Cloud)

```bash
# .env
PROXY_URL=https://your-litellm-instance.example.com
PROXY_API_KEY=sk-your-hosted-litellm-key
```

When using a hosted proxy, set `PROXY_API_KEY` to your proxy authentication key. The provider API keys (`ANTHROPIC_API_KEY`, `GEMINI_API_KEY`, etc.) are configured on the proxy side, not in the AgentFlow `.env`.

### OpenAI-Only Gateways (Local Anthropic Proxy)

If your LLM gateway only supports the OpenAI Chat Completions format (`/v1/chat/completions`), Claude CLI will fail because it speaks Anthropic Messages API (`/v1/messages`). AgentFlow includes a local protocol translator:

```bash
# Terminal 1: Start the proxy (reads .env automatically)
./scripts/start_proxy.sh

# Terminal 2: Run orchestration
cargo run --bin real_test
```

Configure `.env`:
```env
# Claude CLI and Nexus send Anthropic requests to the LOCAL proxy
PROXY_URL=http://localhost:8080/v1
PROXY_API_KEY=your-gateway-api-key

# The LOCAL proxy forwards OpenAI-format requests to the REMOTE gateway
GATEWAY_URL=https://api.ai.camer.digital/v1/
GATEWAY_API_KEY=your-gateway-api-key
```

When your provider adds native Anthropic support, change `PROXY_URL` to point directly to the gateway and remove `GATEWAY_*`.

### Disabling Proxy (Direct API Access)

If `PROXY_URL` is not set, all agents use direct API access with `ANTHROPIC_API_KEY` — this is the default behavior and requires no proxy setup.

## Requirements

- Rust 1.70+
- Node.js 18+ (for GitHub MCP server)
- **Claude Code CLI** - [Setup Guide](docs/setup-claude-cli.md)
- API keys: `ANTHROPIC_API_KEY` (required), `GITHUB_PERSONAL_ACCESS_TOKEN` (required), plus optional provider keys for proxy routing (`GEMINI_API_KEY`, `OPENAI_API_KEY`, `GROQ_API_KEY`)

## License

MIT
