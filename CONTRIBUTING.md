# Contributing to Autonomous AI Dev Team

Thank you for your interest in contributing to the Autonomous AI Dev Team project! This guide will help you get started with the codebase, understand the architecture, and follow our development standards.

## 🏗️ Architecture Overview

The project is built on **PocketFlow (Rust)** and operates as a multi-agent system where agents coordinate through a **Graph + Shared Store** model.

### The Team (Agents)
- **NEXUS**: The Orchestrator (Scrum Master). Manages the high-level flow and tool calling.
- **FORGE**: The Builder. Implements features, writes code, and handles technical tasks.
- **SENTINEL**: The Reviewer. Audits code for security and quality errors.
- **VESSEL**: The DevOps expert. Manages deployments and infrastructure.
- **LORE**: The Documenter. Maintains project history and ADRs.

### Crate Structure
- `crates/agent-client`: Core client for Anthropic API and MCP (Model Context Protocol).
- `crates/agent-nexus`: Implementation of the Nexus agent's logic.
- `crates/agent-forge`: Implementation of the Forge agent's logic.
- `crates/config`: Shared state and configuration types.
- `binary`: The main entry point and integration test suite.

## 🛠️ Getting Started

### Prerequisites
1. **Rust**: Latest stable version.
2. **Docker**: Required for local MCP server isolation (optional if using `hosted` mode).
3. **Node.js/npx**: Required for the `hosted` MCP bridge.

### Environment Setup
Create a `.env` file or export the following variables:
```bash
ANTHROPIC_API_KEY=your_key_here
GITHUB_PERSONAL_ACCESS_TOKEN=your_pat_here
GITHUB_MCP_TYPE=hosted # or 'docker'
```

### Running the Project
```bash
cargo run -p agent-team
```

## 🧪 Testing

We prioritize high test coverage and reliability.

### Unit Tests
Run unit tests for all crates:
```bash
cargo test --workspace
```

### End-to-End (E2E) Tests
The Nexus agent's E2E test simulates a full orchestration loop with real MCP connectivity:
```bash
cargo test -p agent-team --test nexus_e2e
```

## 📖 Development Workflow (Agentic Coding)

We follow the **Agentic Coding** methodology:
1. **Planning**: Create an `implementation_plan.md` documenting your proposed changes.
2. **Implementation**: Code your changes iteratively.
3. **Verification**: Run all tests (unit + E2E).
4. **Documentation**: Create a `walkthrough.md` to demonstrate the changes.

## 📜 Coding Standards

- **Simplicity**: Favor readability over complex abstractions.
- **Fail Fast**: Use `anyhow` for application errors and `thiserror` for library crates.
- **Async**: Use `tokio` for all asynchronous operations.
- **Commits**: Use descriptive, conventional commit messages.

For more detailed rules, see `.agent/standards/`.
