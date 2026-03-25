# Contributing to Autonomous AI Dev Team

This guide explains how to set up your environment, run the project in different modes, and contribute effectively.

## 🛠️ Prerequisites

1. **Rust**: [Install Rust](https://rustup.rs/) (latest stable).
2. **Python 3**: Required for running mock servers.
3. **Claude Code CLI** (Optional but recommended for Forge):
   ```bash
   npm install -g @anthropic-ai/claude-code
   claude auth login
   ```

## ⚙️ Environment Setup

1. **Copy Template**:
   ```bash
   cp .env.example .env
   ```

2. **Configure Variables**:
   - `OPENAI_API_KEY`: Required for Nexus if using OpenAI.
   - `LLM_PROVIDER`: Set to `openai` or `anthropic`.
   - `GITHUB_PERSONAL_ACCESS_TOKEN`: Required for real-world PR creation.
   - `GITHUB_REPOSITORY`: The target repository (e.g., `owner/repo`).

## 🚀 Running the Project

### Option A: Local Mock Demo (Safe, No API Keys Needed)
This uses local mock servers for the LLM and MCP, and a mock Claude script for Forge.

1. **Start Mock Infrastructure**:
   ```bash
   # Terminal 1: Mock LLM (OpenAI-compatible)
   python3 scripts/mock_llm.py
   
   # Terminal 2: Mock GitHub MCP
   # (The demo binary starts this automatically via GITHUB_MCP_CMD)
   ```

2. **Run Demo**:
   ```bash
   cargo run -p agent-team --bin demo
   ```

### Option B: Real-World Orchestration
This connects to live GitHub and live LLM providers.

1. **Run Real Test**:
   ```bash
   cargo run -p agent-team --bin real_test
   ```

## 🧪 Testing

### Unit Tests
```bash
cargo test --workspace
```

### End-to-End Tests
We have specific E2E tests for core logic:
```bash
# Test Nexus decision making
cargo test -p agent-nexus

# Test Forge suspension logic (mocked)
cargo test -p agent-forge --test forge_claude_e2e
```

## 📂 Architecture Overview
- **SharedStore**: A key-value store where agents exchange state (e.g., `worker_slots`, `tickets`).
- **Graph Nodes**: Each agent is a `BatchNode` that reads from the store and writes back "actions" (e.g., `work_assigned`).
- **PocketFlow**: The engine that executes the graph and manages state transitions.

## 📜 Development Workflow
1. **Plan**: Propose changes in an `implementation_plan.md`.
2. **Implement**: Keep crates focused and minimal.
3. **Verify**: Ensure both unit tests and `demo` pass.
4. **Log**: Forge work is logged to `forge/workers/<id>/worker.log` during execution.

---
For more specific rules, see `.agent/standards/`.
