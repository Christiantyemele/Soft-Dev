# AgentFlow Complete Tutorial: Build an App from Zero

This tutorial walks you through running AgentFlow to autonomously build a web application from scratch. You'll see exactly what logs to expect, which files are created, and where everything happens.

## Table of Contents

1. [Prerequisites Setup](#prerequisites-setup)
2. [Environment Configuration](#environment-configuration)
3. [Creating a Target Project](#creating-a-target-project)
4. [Running the Orchestration](#running-the-orchestration)
5. [Understanding the Logs](#understanding-the-logs)
6. [Inspecting Generated Files](#inspecting-generated-files)
7. [Troubleshooting](#troubleshooting)

---

## Prerequisites Setup

### 1. Install Required Tools

```bash
# Check Rust version (need 1.70+)
rustc --version
# If not installed: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Check Node.js version (need 18+)
node --version
# If not installed: https://nodejs.org/

# Install Claude Code CLI (REQUIRED - all agents use it)
claude --version
# If not installed: https://www.anthropic.com/claude-code
```

### 2. Get API Keys

You'll need:

| Key | Purpose | Where to Get |
|-----|---------|--------------|
| `ANTHROPIC_API_KEY` | Powers Claude Code (all agents) | https://console.anthropic.com/ |
| `GITHUB_PERSONAL_ACCESS_TOKEN` | GitHub operations (issues, PRs, CI) | https://github.com/settings/tokens |

Optional provider keys (used when `PROXY_URL` is not set, or as fallback):

| Key | Purpose |
|-----|---------|
| `OPENAI_API_KEY` | OpenAI models via proxy or direct |
| `GEMINI_API_KEY` | Gemini models via proxy or direct |
| `FIREWORKS_API_KEY` | Fireworks AI models (default gateway) |
| `GROQ_API_KEY` | Groq models via proxy |

For the GitHub token, ensure these scopes:
- `repo` (full control of private repositories)
- `workflow` (update GitHub Action workflows)
- `write:packages` (upload packages to GitHub Package Registry)

---

## Environment Configuration

### 1. Clone AgentFlow

```bash
git clone https://github.com/The-AgenticFlow/AgentFlow.git
cd AgentFlow
```

### 2. Create `.env` File

```bash
cp .env.example .env
nano .env  # or use your preferred editor
```

### 3. Configure Your `.env`

#### Proxy Mode (Recommended)

```env
# Proxy URL (enables proxy-first routing)
PROXY_URL=http://localhost:8080/v1
PROXY_API_KEY=your-proxy-key

# Upstream gateway (for local Anthropic-to-OpenAI proxy)
GATEWAY_URL=https://api.fireworks.ai/inference/v1/
GATEWAY_API_KEY=your-gateway-api-key

# Model provider mapping
MODEL_PROVIDER_MAP=glm=openai,deepseek=openai,gpt=openai

# Claude CLI path
CLAUDE_PATH=claude

# GitHub
GITHUB_REPOSITORY=your-username/enterprise-inventory
GITHUB_PERSONAL_ACCESS_TOKEN=ghp_xxxxxxxxxxxxx

# Fallback API keys (used when proxy has transient errors)
ANTHROPIC_API_KEY=sk-ant-xxxxxxxxxxxxx

# Logging
RUST_LOG=info,agent_team=debug,pocketflow_core=debug
```

#### Direct Mode (No Proxy)

```env
# Fallback order (tried in order when PROXY_URL is not set)
LLM_FALLBACK=anthropic,gemini

# Model-to-provider mapping
MODEL_PROVIDER_MAP=glm=openai,deepseek=openai,gpt=openai

# Provider API keys
ANTHROPIC_API_KEY=sk-ant-xxxxxxxxxxxxx
OPENAI_API_KEY=sk-proj-xxxxxxxxxxxxx
GEMINI_API_KEY=AIzaSyxxxxxxxxxxxxx
FIREWORKS_API_KEY=your_fireworks_api_key

# GitHub
GITHUB_REPOSITORY=your-username/enterprise-inventory
GITHUB_PERSONAL_ACCESS_TOKEN=ghp_xxxxxxxxxxxxx
```

### 4. Verify Your Setup

Run the setup checker:

```bash
./scripts/check_setup.sh
```

**Expected output:**

```
AgentFlow Setup Checker
=============================

1. Checking System Requirements...
-----------------------------------
✓ Rust 1.75.0 is installed
✓ Node.js v20.11.0 is installed
✓ Claude Code CLI is installed
✓ Git 2.43.0 is installed

2. Checking Environment Configuration...
----------------------------------------
✓ .env file exists
✓ GITHUB_PERSONAL_ACCESS_TOKEN is set
✓ GITHUB_REPOSITORY is set to: your-username/enterprise-inventory

3. Checking Project Build...
----------------------------
✓ Cargo.toml found
✓ Project compiles successfully

4. Checking AgentFlow Configuration...
--------------------------------------
✓ NEXUS persona found
✓ FORGE persona found
✓ SENTINEL persona found
✓ VESSEL persona found
✓ LORE persona found
✓ Registry has 5 agents configured

=============================
✓ All checks passed!

You're ready to run AgentFlow:
  cargo run --bin agentflow
```

If any checks fail, follow the error messages to fix the issues.

---

## Creating a Target Project

AgentFlow needs a GitHub repository with issues to work on. Let's create a fullstack enterprise inventory management system with REST API, database models, authentication, and a React frontend.

### Option A: Using GitHub CLI

```bash
# Create a new public repository
gh repo create enterprise-inventory --public --clone

cd enterprise-inventory

# Initialize project structure (see docs/example-issues.md for full details)
bash /path/to/AgentFlow/scripts/init-target-project.sh

# Create issues for the agents to work on
# See docs/example-issues.md for the complete issue bodies and ready-to-run gh commands
```

For the full issue creation commands and detailed issue bodies, see **[docs/example-issues.md](docs/example-issues.md)**.

### Option B: Using GitHub Web UI

1. Go to https://github.com/new
2. Create a repository named `enterprise-inventory`
3. Make it public
4. Initialize with a README
5. Go to Issues tab
6. Create 3 issues using the titles and descriptions from [docs/example-issues.md](docs/example-issues.md)

### 3. Update AgentFlow `.env`

```bash
cd /path/to/AgentFlow
nano .env
```

Update the `GITHUB_REPOSITORY` line:
```env
GITHUB_REPOSITORY=your-username/enterprise-inventory
```

---

## Running the Orchestration

### 1. Build and Run

```bash
cd /path/to/AgentFlow

# Build the project (first time only)
cargo build --release --bin agentflow

# Run the orchestration
cargo run --bin agentflow
```

**Expected output on startup:**

```
2026-05-02T00:00:01.234Z  INFO agentflow: Starting REAL End-to-End Orchestration (Event-Driven FORGE-SENTINEL Pairs + VESSEL)
2026-05-02T00:00:02.456Z  INFO agentflow: Target repository workspace ready workspace=/home/christian/.agentflow/workspaces/your-username-enterprise-inventory
2026-05-02T00:00:02.789Z  INFO agentflow: Running orchestration loop for repository: your-username/enterprise-inventory
2026-05-02T00:00:03.000Z  INFO agentflow: Each worker will use event-driven FORGE-SENTINEL pair with:
2026-05-02T00:00:03.100Z  INFO agentflow:   - PLAN.md -> CONTRACT.md (plan review)
2026-05-02T00:00:03.200Z  INFO agentflow:   - WORKLOG.md -> segment-N-eval.md (segment evaluation)
2026-05-02T00:00:03.300Z  INFO agentflow:   - final-review.md (final approval)
2026-05-02T00:00:03.400Z  INFO agentflow:   - STATUS.json (completion status)
2026-05-02T00:00:03.500Z  INFO agentflow: VESSEL will handle merge gate:
2026-05-02T00:00:03.600Z  INFO agentflow:   - CI status polling (10s interval, 10min timeout)
2026-05-02T00:00:03.700Z  INFO agentflow:   - Squash merge with ticket reference
2026-05-02T00:00:03.800Z  INFO agentflow:   - ticket_merged event emission
```

### 2. Understanding the Workspace

AgentFlow creates an isolated workspace structure:

```
~/.agentflow/
└── workspaces/
    └── your-username-enterprise-inventory/
        ├── main/                    # Main repository clone
        ├── worktrees/               # Isolated work areas for each agent
        │   ├── forge-1/             # FORGE worker #1 workspace
        │   └── forge-2/             # FORGE worker #2 workspace
        └── orchestration/
            └── pairs/
                └── forge-1/
                    └── T-001/
                        └── shared/  # FORGE-SENTINEL communication
                            ├── PLAN.md
                            ├── CONTRACT.md
                            ├── WORKLOG.md
                            ├── segment-1-eval.md
                            ├── final-review.md
                            └── STATUS.json
```

---

## Understanding the Logs

### Step 1: NEXUS Discovers Issues

**You'll see:**

```
2026-05-02T00:00:05.123Z  INFO agent_nexus: Syncing worker slots from registry
2026-05-02T00:00:05.234Z  INFO agent_nexus: Loaded 6 worker slots: ["nexus", "forge-1", "forge-2", "sentinel", "vessel", "lore"]
2026-05-02T00:00:06.345Z  INFO agent_nexus: Fetching open issues from your-username/enterprise-inventory
2026-05-02T00:00:07.456Z  INFO agent_nexus: Found 3 open issues
2026-05-02T00:00:07.567Z  INFO agent_nexus: Synced new ticket from GitHub ticket_id=T-001 title="Implement backend authentication and authorization system"
2026-05-02T00:00:07.678Z  INFO agent_nexus: Synced new ticket from GitHub ticket_id=T-002 title="Build inventory CRUD API with stock management and audit logging"
2026-05-02T00:00:08.789Z  INFO agent_nexus: Synced new ticket from GitHub ticket_id=T-003 title="Build React frontend with dashboard, inventory management UI, and analytics"
2026-05-02T00:00:08.890Z  INFO agent_nexus: Assigning issue #1 "Implement backend authentication and authorization system" to forge-1
```

**What's happening:**
1. NEXUS loads worker slots from [`registry.json`](orchestration/agent/registry.json:1)
2. Connects to GitHub via MCP server
3. Fetches open issues from your repository and syncs them as tickets
4. Assigns first issue to `forge-1`

**Output format:**
```json
{
  "action": "work_assigned",
  "assign_to": "forge-1",
  "ticket_id": "T-001",
  "issue_url": "https://github.com/your-username/enterprise-inventory/issues/1"
}
```

### Step 2: FORGE Creates Worktree

**You'll see:**

```
2026-05-02T00:00:10.123Z  INFO agent_forge: Processing work_assigned for worker forge-1
2026-05-02T00:00:10.234Z  INFO pair_harness::worktree: Creating worktree for forge-1
2026-05-02T00:00:11.345Z  INFO pair_harness::worktree: Worktree created at /home/christian/.agentflow/workspaces/your-username-enterprise-inventory/worktrees/forge-1
2026-05-02T00:00:11.456Z  INFO pair_harness::worktree: Checked out new branch: forge-1/T-001
```

**What's happening:**
1. FORGE receives work assignment
2. Creates an isolated Git worktree for this task
3. Creates a new branch named after the worker and ticket

### Step 3: FORGE Spawns Claude Code

**You'll see:**

```
2026-05-02T00:00:12.567Z  INFO agent_forge: Spawning Claude Code for worker forge-1
2026-05-02T00:00:12.678Z  INFO pair_harness::process: Running: claude run --persona /path/to/orchestration/agent/agents/forge.agent.md
2026-05-02T00:00:13.789Z  INFO agent_forge: Claude Code process started (PID: 12345)
2026-05-02T00:00:13.890Z  INFO agent_forge: Worker forge-1 is now working on T-001
```

**What's happening:**
1. Spawns Claude Code CLI with FORGE persona
2. Provides the issue context
3. Claude Code starts autonomous development

**This step takes 5-15 minutes** depending on task complexity.

### Step 4: FORGE-SENTINEL Pair Lifecycle

While Claude Code is working, the event-driven pair lifecycle proceeds:

1. **FORGE writes PLAN.md** - Implementation plan with segment breakdown
2. **SENTINEL reviews PLAN.md** - Writes CONTRACT.md with AGREED or CHANGES_REQUESTED
3. **FORGE implements segments** - Writes WORKLOG.md with progress
4. **SENTINEL evaluates each segment** - Writes segment-N-eval.md with APPROVED or CHANGES_REQUESTED
5. **SENTINEL final review** - Writes final-review.md with overall verdict
6. **FORGE opens PR** - Writes STATUS.json with completion status

You can monitor progress:

```bash
# Watch the worker log in real-time
tail -f ~/.agentflow/workspaces/your-username-enterprise-inventory/forge/workers/forge-1/worker.log
```

**Example log snippets:**

```
[Claude Code] Reading issue #1: Implement backend authentication and authorization system
[Claude Code] Planning implementation...
[Claude Code] Writing PLAN.md with 4 segments
[Claude Code] Creating backend/src/models/User.js with auth schema
[Claude Code] Writing backend/src/routes/auth.js with 6 endpoints
[Claude Code] Adding backend/src/middleware/auth.js for JWT verification
[Claude Code] Writing backend/src/controllers/authController.js
[Claude Code] Creating backend/tests/auth.test.js with 15 test cases
[Claude Code] Running tests...
[Claude Code] All 15 tests passed (87% coverage)
[Claude Code] Committing changes...
[Claude Code] Writing STATUS.json...
```

### Step 5: Work Completion

**You'll see:**

```
2026-05-02T00:15:45.123Z  INFO agent_forge: Worker forge-1 completed work on T-001
2026-05-02T00:15:45.234Z  INFO agent_forge: STATUS.json found at /home/christian/.agentflow/workspaces/your-username-enterprise-inventory/orchestration/pairs/forge-1/T-001/shared/STATUS.json
2026-05-02T00:15:45.345Z  INFO agent_forge: Work result: pr_opened, PR: https://github.com/your-username/enterprise-inventory/pull/1
```

**Output format:**
```json
{
  "status": "PR_OPENED",
  "ticket_id": "T-001",
  "pr_url": "https://github.com/your-username/enterprise-inventory/pull/1",
  "pr_number": 1,
  "branch": "forge-1/T-001",
  "files_changed": 12,
  "segments_completed": [
    {"segment": 1, "status": "APPROVED", "eval_file": "segment-1-eval.md"},
    {"segment": 2, "status": "APPROVED", "eval_file": "segment-2-eval.md"},
    {"segment": 3, "status": "APPROVED", "eval_file": "segment-3-eval.md"},
    {"segment": 4, "status": "APPROVED", "eval_file": "segment-4-eval.md"}
  ],
  "test_results": {"passed": 32, "failed": 0, "skipped": 0},
  "sentinel_approved": true,
  "context_resets": 0
}
```

### Step 6: VESSEL Handles Merge

**You'll see:**

```
2026-05-02T00:15:46.456Z  INFO agent_vessel: Processing pending PRs
2026-05-02T00:15:46.567Z  INFO agent_vessel::ci_poller: Polling CI status for PR #1
2026-05-02T00:15:56.678Z  INFO agent_vessel::ci_poller: CI status: success for PR #1
2026-05-02T00:15:56.789Z  INFO agent_vessel::merger: Merging PR #1 with squash
2026-05-02T00:15:57.890Z  INFO agent_vessel: PR #1 merged successfully
2026-05-02T00:15:57.901Z  INFO agent_vessel: Emitted ticket_merged event for T-001
```

**What's happening:**
1. VESSEL polls CI status (10s interval, 10min timeout)
2. Detects merge conflicts early via GitHub's `mergeable` field
3. Attempts conflict resolution if needed
4. Squash-merges green PRs
5. Emits `ticket_merged` event for dependency resolution

### Step 7: NEXUS Assigns More Work

**You'll see:**

```
2026-05-02T00:15:58.123Z  INFO agent_nexus: Worker forge-1 marked as available
2026-05-02T00:15:58.234Z  INFO agent_nexus: Assigning issue #2 "Build inventory CRUD API with stock management and audit logging" to forge-2
```

The cycle repeats for each issue!

### Step 8: All Work Complete

**You'll see:**

```
2026-05-02T00:30:12.123Z  INFO agent_nexus: No more open issues
2026-05-02T00:30:12.234Z  INFO agent_nexus: All workers idle
2026-05-02T00:30:12.345Z  INFO agentflow: Orchestration flow halted with action: no_work
```

---

## Inspecting Generated Files

### 1. Understanding the File Structure

AgentFlow uses a specific directory structure for work completion:

```bash
~/.agentflow/workspaces/your-username-enterprise-inventory/
├── main/                    # Main repository clone
├── worktrees/               # Agent work areas (CODE FILES)
│   └── forge-1/             # Worker #1 isolated workspace
│       ├── src/             # Generated source code
│       ├── tests/           # Generated test files
│       ├── PLAN.md          # Copy of implementation plan
│       ├── WORKLOG.md       # Copy of progress log
│       ├── CONTRACT.md      # Copy of SENTINEL-approved contract
│       ├── segment-1-eval.md # Copy of segment evaluation
│       ├── final-review.md  # Copy of final review
│       └── STATUS.json      # Copy of completion status
└── orchestration/           # Worker management directory
    └── pairs/
        └── forge-1/
            └── T-001/
                └── shared/  # FORGE-SENTINEL communication (SOURCE OF TRUTH)
                    ├── TICKET.md       # GitHub issue details
                    ├── TASK.md         # Task instructions
                    ├── PLAN.md         # Implementation plan
                    ├── CONTRACT.md     # SENTINEL approval
                    ├── WORKLOG.md      # Progress tracking
                    ├── segment-1-eval.md # Segment evaluation
                    ├── final-review.md # Final review
                    └── STATUS.json     # Completion status
```

### 2. Check the Code Files

```bash
# View the generated code
cd ~/.agentflow/workspaces/your-username-enterprise-inventory/worktrees/forge-1

# List all files
ls -la

# View specific files
cat backend/src/models/User.js
cat backend/tests/auth.test.js
```

### 3. View STATUS.json (Work Completion)

```bash
# STATUS.json is in the shared directory (source of truth)
cat ~/.agentflow/workspaces/your-username-enterprise-inventory/orchestration/pairs/forge-1/T-001/shared/STATUS.json
```

**Example content:**

```json
{
  "status": "PR_OPENED",
  "ticket_id": "T-001",
  "pr_url": "https://github.com/your-username/enterprise-inventory/pull/1",
  "pr_number": 1,
  "branch": "forge-1/T-001",
  "files_changed": 12,
  "segments_completed": [
    {"segment": 1, "status": "APPROVED", "eval_file": "segment-1-eval.md"},
    {"segment": 2, "status": "APPROVED", "eval_file": "segment-2-eval.md"},
    {"segment": 3, "status": "APPROVED", "eval_file": "segment-3-eval.md"},
    {"segment": 4, "status": "APPROVED", "eval_file": "segment-4-eval.md"}
  ],
  "test_results": {"passed": 32, "failed": 0, "skipped": 0},
  "sentinel_approved": true,
  "context_resets": 0
}
```


### 4. View SENTINEL Evaluation Files

```bash
cd ~/.agentflow/workspaces/your-username-enterprise-inventory/orchestration/pairs/forge-1/T-001/shared

# View the implementation plan
cat PLAN.md

# View SENTINEL's contract approval
cat CONTRACT.md
```

**Example CONTRACT.md:**

```markdown
# Contract for T-001: Implement backend authentication and authorization system

status: AGREED

## Acceptance Criteria

1. User registration with email validation and uniqueness constraint
2. JWT access token (15min) and refresh token (7d) generation
3. Password hashing with bcrypt (12 rounds)
4. Role-based middleware (admin/manager/viewer)
5. Rate limiting on auth endpoints (100 req/15min)
6. Password reset flow with email verification
7. Token blacklisting for logout
8. All endpoints return standardized error responses
9. Test coverage >= 80%
10. Winston logging for auth attempts

## Definition of Done

- All auth endpoints working correctly
- Password never stored in plaintext
- Role-based authorization blocks unauthorized access
- Token refresh and rotation works correctly
- All tests passing with >= 80% coverage
- Error responses include actionable error codes
- Auth attempts logged with Winston
```

**Example segment-1-eval.md:**

```markdown
# Segment 1 Evaluation

verdict: APPROVED

## Correctness
User model validates email uniqueness and format
Password hashing uses bcrypt with 12 rounds
JWT generation includes correct expiration times
Role-based middleware correctly checks permissions

## Test Coverage
- User model: 8 unit tests passing
- Auth routes: 12 integration tests passing
- Middleware: 5 unit tests passing
- Overall coverage: 87%

## Standards Compliance
- Express.js follows MVC pattern
- Proper error handling with try/catch
- Input validation with express-validator
- Winston logging on all auth events

## Code Quality
- Well-organized module structure
- Descriptive variable and function names
- JSDoc comments on public methods
- No hardcoded secrets (all from env vars)

## No Regressions
- Existing project structure preserved
- No breaking changes to API contract
```

### 5. Check Git History

```bash
cd ~/.agentflow/workspaces/your-username-enterprise-inventory/worktrees/forge-1

# View commits
git log --oneline -5

# Check git status
git status

# View changes
git diff origin/main
```

### 6. Test the App Locally

```bash
cd ~/.agentflow/workspaces/your-username-enterprise-inventory/worktrees/forge-1

# For HTML/CSS/JS projects
python3 -m http.server 8000
# Open http://localhost:8000 in your browser

# For Node.js projects (if package.json exists)
npm install
npm run dev

# For React/Vite projects
npm install
npm run dev
```

### 7. Review the Pull Request

```bash
# List all PRs
gh pr list --repo your-username/enterprise-inventory

# View PR details
gh pr view 1 --repo your-username/enterprise-inventory

# Review the code changes
gh pr diff 1 --repo your-username/enterprise-inventory

# Merge the PR (when ready)
gh pr merge 1 --repo your-username/enterprise-inventory --squash
```

---

## Troubleshooting

### Issue: "GITHUB_PERSONAL_ACCESS_TOKEN must be set"

**Cause:** Missing or incorrectly named environment variable.

**Fix:**
```bash
# Check if .env file exists
ls -la .env

# Verify the variable is set
cat .env | grep GITHUB_PERSONAL_ACCESS_TOKEN

# Ensure no extra spaces
# Wrong: GITHUB_PERSONAL_ACCESS_TOKEN = ghp_xxx
# Right: GITHUB_PERSONAL_ACCESS_TOKEN=ghp_xxx
```

### Issue: "No issues found"

**Cause:** Repository has no open issues or `GITHUB_REPOSITORY` is incorrect.

**Fix:**
```bash
# Verify repository format (must be: owner/repo)
echo $GITHUB_REPOSITORY

# Check issues exist
gh issue list --repo your-username/enterprise-inventory

# Create an issue manually
gh issue create --repo your-username/enterprise-inventory --title "Test Issue" --body "Test description"
```

### Issue: "Claude Code CLI not found"

**Cause:** Claude Code CLI is not installed or not in PATH.

**Fix:**
```bash
# Check if installed
which claude

# If not found, download from:
# https://www.anthropic.com/claude-code

# After installation, verify
claude --version
```

### Issue: "Worker timed out"

**Cause:** Task is too complex or Claude Code encountered an error.

**Check the logs:**
```bash
tail -100 ~/.agentflow/workspaces/your-username-enterprise-inventory/forge/workers/forge-1/worker.log
```

**Common causes:**
- API rate limits
- Complex task requiring longer timeout
- Missing dependencies in target repository

**Fix:**
```rust
// In crates/agent-forge/src/lib.rs
// Increase timeout from default (30 min) to 60 min
let timeout_dur = std::time::Duration::from_secs(3600); // 60 minutes
```

### Issue: "Permission denied" when creating worktree

**Cause:** File permissions or disk space.

**Fix:**
```bash
# Check disk space
df -h ~/.agentflow

# Check permissions
ls -la ~/.agentflow/workspaces/

# Fix permissions
chmod -R u+w ~/.agentflow/workspaces/
```

### Issue: "FORGE exited quickly without progress" or "Failed to authenticate. API Error: 403"

**Cause:** Claude CLI can't authenticate through your gateway because it only supports OpenAI format, not the Anthropic Messages API.

**Fix:** Start the local Anthropic-to-OpenAI proxy before running the orchestration:
```bash
# Terminal 1
./scripts/start_proxy.sh

# Terminal 2
cargo run --bin agentflow
```

Ensure `.env` has `GATEWAY_URL` and `GATEWAY_API_KEY` set. See the [Proxy Configuration](#proxy-mode-recommended) section above.

### Issue: "GitHub MCP server fails to start"

**Cause:** Missing Node.js or incorrect GitHub token permissions.

**Fix:**
```bash
# Check Node.js
node --version

# Test GitHub token manually
curl -H "Authorization: token $GITHUB_PERSONAL_ACCESS_TOKEN" \
  https://api.github.com/user

# Ensure token has correct scopes
# Go to: https://github.com/settings/tokens
# Token needs: repo, workflow, write:packages
```

### Issue: "CI setup required" or "No CI workflows found"

**Cause:** Repository has no GitHub Actions workflows configured.

**Fix:**
AgentFlow will automatically inject a CI setup ticket (T-CI-001) that must be completed before any other work. The FORGE agent will create `.github/workflows/ci.yml` with build, test, and lint checks.

If you want to skip this, create a basic CI workflow manually:
```bash
mkdir -p .github/workflows
cat > .github/workflows/ci.yml << 'EOF'
name: CI
on: [push, pull_request]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build
        run: echo "Build step"
EOF
git add .github/workflows/ci.yml
git commit -m "Add basic CI workflow"
git push
```

---

## Directory Structure Reference

```
AgentFlow/                                    # Orchestrator project
├── .env                                      # Your API keys (DO NOT COMMIT)
├── orchestration/agent/
│   ├── agents/
│   │   ├── nexus.agent.md                    # Orchestrator persona
│   │   ├── forge.agent.md                    # Builder persona
│   │   ├── sentinel.agent.md                 # Reviewer persona
│   │   ├── vessel.agent.md                   # DevOps persona
│   │   └── lore.agent.md                     # Writer persona
│   ├── registry.json                         # Agent definitions with model routing
│   └── standards/                            # Coding standards
├── crates/
│   ├── agent-nexus/                          # Orchestrator node
│   ├── agent-forge/                          # Builder node (spawns Claude Code)
│   ├── agent-sentinel/                       # Reviewer node (ephemeral)
│   ├── agent-vessel/                         # Merge gatekeeper node
│   ├── agent-lore/                           # Documentation node
│   ├── agent-client/                         # LLM client + MCP integration
│   ├── pair-harness/                         # Worktree + FORGE-SENTINEL lifecycle
│   ├── pocketflow-core/                      # Flow engine, shared store, routing
│   ├── config/                               # Configuration and state types
│   └── github/                               # GitHub API client
└── binary/src/bin/
    ├── agentflow.rs                          # Main entry point
    └── demo.rs                               # Mocked demonstration

~/.agentflow/                                 # AgentFlow runtime directory
└── workspaces/
    └── your-username-enterprise-inventory/        # Target project workspace
        ├── main/                             # Main repository clone
        │   ├── .git/
        │   └── README.md
        ├── worktrees/                        # Agent work areas (CODE FILES)
        │   ├── forge-1/                      # Worker #1 isolated workspace
        │   │   ├── src/                      # Generated source code
        │   │   ├── tests/                    # Generated test files
        │   │   ├── PLAN.md                   # Copy of plan
        │   │   ├── WORKLOG.md                # Copy of progress
        │   │   ├── CONTRACT.md               # Copy of SENTINEL approval
        │   │   ├── segment-1-eval.md         # Copy of evaluation
        │   │   ├── final-review.md           # Copy of final review
        │   │   └── STATUS.json               # Copy of completion status
        │   └── forge-2/                      # Worker #2 isolated workspace
        └── orchestration/                    # Worker management (SOURCE OF TRUTH)
            └── pairs/
                ├── forge-1/
                │   └── T-001/
                │       └── shared/           # FORGE-SENTINEL communication
                │           ├── TICKET.md     # GitHub issue details
                │           ├── TASK.md       # Task instructions
                │           ├── PLAN.md       # Implementation plan
                │           ├── CONTRACT.md   # SENTINEL approval
                │           ├── WORKLOG.md    # Progress tracking
                │           ├── segment-1-eval.md # Segment evaluation
                │           ├── final-review.md   # Final review
                │           └── STATUS.json   # Completion status
                └── forge-2/
                    └── T-002/
                        └── shared/
                            └── ...
```

---

## Next Steps

1. **Customize Agent Personas**: Edit files in [`orchestration/agent/agents/`](orchestration/agent/agents/) to change how agents work
2. **Add More Workers**: Edit [`orchestration/agent/registry.json`](orchestration/agent/registry.json:1) to change agent instances or add new agents
3. **Configure Per-Agent Model Routing**: Set `model_backend` and `routing_key` in registry.json for LiteLLM proxy routing
4. **Production Deployment**: Use `cargo build --release` and deploy with systemd or Docker

---

## Additional Resources

- [DEMO.md](DEMO.md) - Quick demo guide
- [CONTRIBUTING.md](CONTRIBUTING.md) - Development guidelines
- [docs/forge-sentinel-arch.md](docs/forge-sentinel-arch.md) - Architecture deep dive
- [GitHub Discussions](https://github.com/The-AgenticFlow/AgentFlow/discussions) - Ask questions

---

**Happy Building!**

*Created by [The-AgenticFlow](https://github.com/The-AgenticFlow)*
