// crates/agent-nexus/src/lib.rs
use anyhow::Result;
use async_trait::async_trait;
use pocketflow_core::{Node, SharedStore, Action};
use agent_client::{AgentRunner, AgentPersona, AgentDecision};
use serde_json::{json, Value};
use tracing::info;
use std::path::PathBuf;

pub struct NexusNode {
    pub persona_path: PathBuf,
}

impl NexusNode {
    pub fn new(persona_path: impl Into<PathBuf>) -> Self {
        Self { persona_path: persona_path.into() }
    }

    async fn load_persona(&self) -> Result<AgentPersona> {
        let content = tokio::fs::read_to_string(&self.persona_path).await?;
        // For now, we assume the persona file context is the system prompt.
        // In a more robust version, we'd parse the frontmatter.
        Ok(AgentPersona {
            id: "nexus".to_string(),
            role: "orchestrator".to_string(),
            system_prompt: content,
        })
    }
}

#[async_trait]
impl Node for NexusNode {
    fn name(&self) -> &str { "nexus" }

    async fn prep(&self, store: &SharedStore) -> Result<Value> {
        // Read everything the orchestrator needs to see
        let tickets = store.get("tickets").await.unwrap_or(json!([]));
        let worker_slots = store.get("worker_slots").await.unwrap_or(json!({}));
        let open_prs = store.get("open_prs").await.unwrap_or(json!([]));
        let command_gate = store.get("command_gate").await.unwrap_or(json!({}));

        Ok(json!({
            "tickets": tickets,
            "worker_slots": worker_slots,
            "open_prs": open_prs,
            "command_gate": command_gate,
        }))
    }

    async fn exec(&self, context: Value) -> Result<Value> {
        info!("Nexus calling AgentRunner for orchestration...");
        
        let mut runner = AgentRunner::from_env().await?;
        let persona = self.load_persona().await?;
        
        // The runner drives the tool-calling loop (Anthropic + MCP)
        let decision: AgentDecision = runner.run(&persona, context, 10).await?;
        
        Ok(json!(decision))
    }

    async fn post(&self, _store: &SharedStore, result: Value) -> Result<Action> {
        let decision: AgentDecision = serde_json::from_value(result)?;
        
        info!(action = decision.action, notes = decision.notes, "Nexus decision reached");
        
        // Here we could parse specific decisions to update the store 
        // (e.g. if the LLM called a tool to assign a ticket, the tool result 
        // would have been updated in the loop, but we might want to sync back 
        // internal structures if they aren't fully MCP-driven yet).
        
        // For now, we trust the LLM's returned Action string.
        Ok(Action::new(decision.action))
    }
}
