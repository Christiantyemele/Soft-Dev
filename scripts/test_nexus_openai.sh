#!/bin/bash
set -e

# scripts/test_nexus_openai.sh
# 
# This script runs the Nexus dev team flow using a local Anthropic-to-OpenAI Proxy.
# It enables testing Phase 2 logic using your OpenAI API KEY.

# 1. Validation
if [ -z "$OPENAI_API_KEY" ]; then
    echo "Error: OPENAI_API_KEY is not set."
    echo "Usage: OPENAI_API_KEY=sk-... ./scripts/test_nexus_openai.sh"
    exit 1
fi

# 2. Compile the proxy server
echo "Building Anthropic-to-OpenAI Proxy..."
cargo build -p anthropic-mock --quiet

# 3. Start the proxy server in the background
echo "Starting Proxy on port 8080 (Forwarding to OpenAI)..."
./target/debug/anthropic-proxy &
PROXY_PID=$!

# Ensure the proxy is killed on exit
trap "kill $PROXY_PID" EXIT

# Wait a moment for the server to start
sleep 2

# 4. Compile and Run the Dev Team Binary
echo "Running Nexus Agent via OpenAI Proxy..."
export ANTHROPIC_API_KEY="proxy-active"
export ANTHROPIC_API_URL="http://localhost:8080/v1/messages"
export OPENAI_MODEL=${OPENAI_MODEL:-"gpt-4o"}
export RUST_LOG="info,agent_team=debug,agent_nexus=debug,agent_client=debug"

# Ensure we use the hosted MCP bridge
export GITHUB_MCP_TYPE="hosted"

# If GITHUB_PERSONAL_ACCESS_TOKEN is not set, we use a dummy for the bridge initialization
# but real GitHub calls will fail unless a real PAT is provided.
if [ -z "$GITHUB_PERSONAL_ACCESS_TOKEN" ]; then
    echo "Warning: GITHUB_PERSONAL_ACCESS_TOKEN not set. Real GitHub tool calls will fail."
    export GITHUB_PERSONAL_ACCESS_TOKEN="ghp_dummy"
fi

cargo run -p agent-team

echo "Done."
