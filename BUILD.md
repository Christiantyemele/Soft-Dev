# Building AgentFlow from Source

> 🌐 Official site: [openflows.dev](https://openflows.dev)

This guide walks you through building AgentFlow from source. Whether you're contributing to the project or want to run the latest development version, follow these steps.

## Prerequisites

### Required

| Tool | Version | Purpose |
|------|---------|---------|
| **Rust** | 1.70+ | Core runtime and build system |
| **Node.js** | 18+ | GitHub MCP server dependency |
| **Claude Code CLI** | Latest | AI agent execution |

### Installing Prerequisites

**Rust (via rustup):**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustc --version  # Verify installation
```

**Node.js:**
```bash
# macOS (Homebrew)
brew install node

# Ubuntu/Debian
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs

# Windows (winget)
winget install OpenJS.NodeJS.LTS
```

**Claude Code CLI:**
See [docs/setup-claude-cli.md](docs/setup-claude-cli.md) for detailed setup instructions.

## Clone the Repository

```bash
git clone https://github.com/The-AgenticFlow/AgentFlow.git
cd AgentFlow
```

## Build Commands

### Development Build

Fast build with debug symbols:

```bash
cargo build
```

Binary location: `target/debug/agentflow`

### Release Build

Optimized build for production use:

```bash
cargo build --release
```

Binary location: `target/release/agentflow`

Release builds are significantly faster but take longer to compile.

### Build Specific Binary

```bash
# Main orchestration binary
cargo build --bin agentflow

# Demo with mock data
cargo build --bin demo

# Dry-run demo (local nodes only)
cargo build --bin agentflow-demo
```

## Install Globally

Install `agentflow` to `~/.cargo/bin/`:

```bash
cargo install --path binary
```

After installation, run from anywhere:
```bash
agentflow
```

## Available Binaries

| Binary | Purpose |
|--------|---------|
| `agentflow` | **Main entrypoint** - Production orchestration with real GitHub API and Claude CLI |
| `agentflow-demo` | Dry-run mode using local node implementations |
| `demo` | Mocked demonstration with fake data |

## Build Troubleshooting

### "Cargo not found"
Install Rust via rustup (see Prerequisites above).

### Compilation errors
```bash
# Update Rust toolchain
rustup update

# Clean and rebuild
cargo clean
cargo build
```

### "linker 'cc' not found"
Install a C compiler:
```bash
# Ubuntu/Debian
sudo apt-get install build-essential

# macOS (Xcode Command Line Tools)
xcode-select --install

# Windows
# Install Visual Studio Build Tools or MSVC
```

### OpenSSL errors
```bash
# Ubuntu/Debian
sudo apt-get install pkg-config libssl-dev

# macOS
brew install openssl
export OPENSSL_DIR=$(brew --prefix openssl)
```

### "node: command not found" (when running orchestration)
Install Node.js (see Prerequisites above). Required for GitHub MCP server.

## Build Artifacts

After a successful build:

```
target/
├── debug/
│   ├── agentflow          # Development binary
│   ├── agentflow-demo
│   └── demo
└── release/
    ├── agentflow          # Production binary (faster)
    ├── agentflow-demo
    └── demo
```

## Next Steps

After building, see [RUN.md](RUN.md) for configuration and execution instructions.
