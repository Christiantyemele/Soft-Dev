# Running AgentFlow

> 🌐 Official site: [openflows.dev](https://openflows.dev)

This guide covers configuration and execution of AgentFlow after building. See [BUILD.md](BUILD.md) for build instructions.

## Quick Start

```bash
# 1. Configure environment
cp .env.example .env
# Edit .env with your API keys

# 2. Run orchestration
cargo run --bin agentflow
```

## Environment Configuration

### Required Environment Variables

| Variable | Description |
|----------|-------------|
| `GITHUB_REPOSITORY` | Target repository in `owner/repo` format |
| `GITHUB_PERSONAL_ACCESS_TOKEN` | GitHub PAT with `repo` scope |
| `ANTHROPIC_API_KEY` | Anthropic API key (or use proxy mode) |

### Setup

1. **Copy the example file:**
   ```bash
   cp .env.example .env
   ```

2. **Edit `.env` with your credentials:**
   ```bash
   # Required
   GITHUB_REPOSITORY=your-org/your-repo
   GITHUB_PERSONAL_ACCESS_TOKEN=ghp_xxxxx
   ANTHROPIC_API_KEY=sk-ant-xxxxx
   ```

3. **Verify Claude CLI is installed:**
   ```bash
   which claude
   claude --version
   ```

## Execution Modes

### Production Mode (Recommended)

Run the full orchestration with real GitHub API and Claude CLI:

```bash
# Via cargo
cargo run --bin agentflow

# Or directly after build
./target/release/agentflow
```

This mode:
- Connects to GitHub API to fetch issues
- Spawns Claude CLI for code generation
- Creates real pull requests
- Polls CI status and merges PRs

### Development Mode

Dry-run with local node implementations:

```bash
cargo run --bin agentflow-demo
```

Uses in-memory implementations without external API calls.

### Mocked Demo

Pre-configured demonstration with fake data:

```bash
cargo run --bin demo
```

## Configuration Options

### LLM Provider Configuration

**Proxy Mode (Recommended):**
```env
PROXY_URL=http://localhost:4000/v1
PROXY_API_KEY=your-proxy-key
```

Route all LLM requests through a LiteLLM proxy for:
- Per-agent model routing
- Cost optimization
- Rate limit management

**Direct Mode:**
```env
ANTHROPIC_API_KEY=sk-ant-xxxxx
GEMINI_API_KEY=xxxxx
OPENAI_API_KEY=sk-xxxxx
```

Set individual provider keys. The system falls back through providers on failure.

### Redis Backend (Optional)

For persistent state across runs:

```env
REDIS_URL=redis://localhost:6379
```

Without Redis, the system uses an in-memory store (state lost on restart).

### Logging

Control log verbosity:

```env
RUST_LOG=info,agent_team=debug,pocketflow_core=debug
```

Levels: `error`, `warn`, `info`, `debug`, `trace`

## Directory Structure

AgentFlow creates workspaces in `~/.agentflow/`:

```
~/.agentflow/
└── workspaces/
    └── your-repo/           # Cloned target repository
        ├── .git/
        ├── worktrees/       # Agent worktrees
        │   ├── forge-1/
        │   └── forge-2/
        └── orchestration/   # Status files, logs
```

## Running Examples

### Basic Run

```bash
# Ensure .env is configured
cargo run --bin agentflow
```

Expected output:
```
INFO Starting REAL End-to-End Orchestration...
INFO Target repository workspace ready: /home/user/.agentflow/workspaces/my-repo
INFO Running orchestration loop for repository: owner/repo
```

### With Custom Workspace Root

```env
AGENTFLOW_WORKSPACE_ROOT=/custom/path/workspaces
```

### With Redis Persistence

```bash
# Start Redis
docker run -d -p 6379:6379 redis

# Run with Redis backend
REDIS_URL=redis://localhost:6379 cargo run --bin agentflow
```

## Troubleshooting

### "GITHUB_REPOSITORY must be set"
Set the target repository in `.env`:
```env
GITHUB_REPOSITORY=owner/repo
```

### "GitHub token must be set"
Set your GitHub PAT:
```env
GITHUB_PERSONAL_ACCESS_TOKEN=ghp_xxxxx
```

### "claude: command not found"
Install Claude CLI or set the path:
```env
CLAUDE_PATH=/path/to/claude
```

See [docs/setup-claude-cli.md](docs/setup-claude-cli.md) for detailed setup.

### Connection refused (Redis)
Ensure Redis is running:
```bash
docker run -d -p 6379:6379 redis
```

Or remove `REDIS_URL` to use in-memory store.

### API rate limit exceeded
- Use LiteLLM proxy for rate limit management
- Reduce concurrent workers in `orchestration/agent/registry.json`
- Add fallback providers in `LLM_FALLBACK`

## Next Steps

- [TUTORIAL.md](TUTORIAL.md) - Step-by-step walkthrough
- [docs/demo.md](docs/demo.md) - Live flow demonstration
- [CONTRIBUTING.md](CONTRIBUTING.md) - Development guidelines
