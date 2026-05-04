use anyhow::Result;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use ratatui::Terminal;
use std::io;

use crate::setup::{AgentConfig, SetupConfig};
use crate::util::theme::Theme;
use crate::widgets::select::SelectableListState;

enum AgentConfigState {
    SelectingAction {
        agents: Vec<AgentConfig>,
        selected: usize,
    },
    ChoosingModel {
        agent: AgentConfig,
        models: Vec<String>,
        selected: usize,
    },
    ChoosingInstances {
        agent: AgentConfig,
        instances: u32,
    },
}

pub struct AgentsStep;

impl AgentsStep {
    pub fn new() -> Self {
        Self
    }

    pub async fn render(
        &self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        theme: &Theme,
        config: &mut SetupConfig,
    ) -> Result<()> {
        let mut agents: Vec<AgentConfig> = Vec::new();
        let current_state: Option<AgentConfigState>;

        // Check if registry exists to pre-populate
        let registry_path = std::env::current_dir()?
            .join("orchestration")
            .join("agent")
            .join("registry.json");

        if registry_path.exists() {
            if let Ok(registry) = config::Registry::load(&registry_path) {
                for entry in registry.team {
                    agents.push(AgentConfig {
                        id: entry.id,
                        cli: entry.cli,
                        active: entry.active,
                        instances: entry.instances,
                        model_backend: entry.model_backend,
                        routing_key: entry.routing_key,
                        github_token_env: entry.github_token_env,
                    });
                }
            }
        }

        if agents.is_empty() {
            // Default agents
            agents.push(AgentConfig {
                id: "nexus".to_string(),
                cli: "claude".to_string(),
                active: true,
                instances: 1,
                model_backend: Some("anthropic/claude-sonnet-4-5".to_string()),
                routing_key: Some("nexus-key".to_string()),
                github_token_env: None,
            });
            agents.push(AgentConfig {
                id: "forge".to_string(),
                cli: "claude".to_string(),
                active: true,
                instances: 2,
                model_backend: Some("anthropic/claude-sonnet-4-5".to_string()),
                routing_key: Some("forge-key".to_string()),
                github_token_env: None,
            });
            agents.push(AgentConfig {
                id: "sentinel".to_string(),
                cli: "claude".to_string(),
                active: true,
                instances: 1,
                model_backend: Some("gemini/gemini-2.5-pro".to_string()),
                routing_key: Some("sentinel-key".to_string()),
                github_token_env: None,
            });
            agents.push(AgentConfig {
                id: "vessel".to_string(),
                cli: "claude".to_string(),
                active: true,
                instances: 1,
                model_backend: Some("groq/llama-3.3-70b-versatile".to_string()),
                routing_key: Some("vessel-key".to_string()),
                github_token_env: None,
            });
            agents.push(AgentConfig {
                id: "lore".to_string(),
                cli: "claude".to_string(),
                active: true,
                instances: 1,
                model_backend: Some("openai/gpt-4o-mini".to_string()),
                routing_key: Some("lore-key".to_string()),
                github_token_env: None,
            });
        }

        // Start with action selection
        current_state = Some(AgentConfigState::SelectingAction {
            agents: agents.clone(),
            selected: 0,
        });

        loop {
            match current_state.take() {
                Some(AgentConfigState::SelectingAction { agents: current_agents, selected }) => {
                    agents = current_agents;
                    let agent_names: Vec<String> = agents.iter().map(|a| {
                        let status = if a.active { "✓" } else { "✗" };
                        format!("{} {} ({} instances)", status, a.id, a.instances)
                    }).collect();
                    
                    let mut list_state = SelectableListState::new(agent_names.clone());
                    list_state.selected = selected;

                    loop {
                        terminal.draw(|f| {
                            let area = f.area();
                            let chunks = Layout::default()
                                .direction(Direction::Vertical)
                                .margin(3)
                                .constraints([
                                    Constraint::Length(4),
                                    Constraint::Min(8),
                                    Constraint::Length(2),
                                ])
                                .split(area);

                            let title_block = ratatui::widgets::Block::default()
                                .borders(ratatui::widgets::Borders::BOTTOM)
                                .border_style(Style::default().fg(theme.border()));
                            let inner_title = title_block.inner(chunks[0]);
                            title_block.render(chunks[0], f.buffer_mut());

                            let title = Line::styled(
                                "◇ CONFIGURE AGENTS",
                                Style::default().fg(theme.accent()).add_modifier(Modifier::BOLD),
                            );
                            let subtitle = Line::styled(
                                "  Select agent to configure or press Enter to finish",
                                Style::default().fg(theme.muted()),
                            );
                            let title_para = ratatui::widgets::Paragraph::new(vec![title, subtitle]);
                            title_para.render(inner_title, f.buffer_mut());

                            let list_widget = crate::widgets::select::SelectableList::new(
                                &list_state.items,
                                list_state.selected,
                            ).title("Select agent to configure");
                            list_widget.render(chunks[1], f.buffer_mut());

                            let help = Line::styled(
                                "  ↑↓ navigate  │  Enter: select agent  │  Tab: finish configuration",
                                Style::default().fg(theme.muted()),
                            );
                            let help_para = Paragraph::new(help);
                            help_para.render(chunks[2], f.buffer_mut());
                        })?;

                        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
                            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                                use crossterm::event::KeyCode;
                                match key.code {
                                    KeyCode::Up => list_state.move_up(),
                                    KeyCode::Down => list_state.move_down(),
                                    KeyCode::Tab => {
                                        config.agents = agents.clone();
                                        return Ok(());
                                    }
                                    KeyCode::Enter => {
                                        current_state = Some(AgentConfigState::ChoosingModel {
                                            agent: agents[list_state.selected].clone(),
                                            models: vec![
                                                "anthropic/claude-sonnet-4-5".to_string(),
                                                "anthropic/claude-3-5-sonnet".to_string(),
                                                "gemini/gemini-2.5-pro".to_string(),
                                                "openai/gpt-4o".to_string(),
                                                "openai/gpt-4o-mini".to_string(),
                                                "groq/llama-3.3-70b-versatile".to_string(),
                                                "fireworks/accounts/fireworks/models/glm-5".to_string(),
                                            ],
                                            selected: 0,
                                        });
                                        break;
                                    }
                                    KeyCode::Esc => {
                                        return Err(anyhow::anyhow!("Setup cancelled"));
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                Some(AgentConfigState::ChoosingModel { agent, models, selected }) => {
                    let mut list_state = SelectableListState::new(models.clone());
                    list_state.selected = selected;

                    loop {
                        terminal.draw(|f| {
                            let area = f.area();
                            let chunks = Layout::default()
                                .direction(Direction::Vertical)
                                .margin(3)
                                .constraints([
                                    Constraint::Length(4),
                                    Constraint::Min(8),
                                    Constraint::Length(2),
                                ])
                                .split(area);

                            let title_block = ratatui::widgets::Block::default()
                                .borders(ratatui::widgets::Borders::BOTTOM)
                                .border_style(Style::default().fg(theme.border()));
                            let inner_title = title_block.inner(chunks[0]);
                            title_block.render(chunks[0], f.buffer_mut());

                            let title = Line::styled(
                                "◇ SELECT MODEL BACKEND",
                                Style::default().fg(theme.accent()).add_modifier(Modifier::BOLD),
                            );
                            let subtitle = Line::styled(
                                format!("  Choose model for agent: {}", agent.id),
                                Style::default().fg(theme.muted()),
                            );
                            let title_para = ratatui::widgets::Paragraph::new(vec![title, subtitle]);
                            title_para.render(inner_title, f.buffer_mut());

                            let list_widget = crate::widgets::select::SelectableList::new(
                                &list_state.items,
                                list_state.selected,
                            ).title("Select model backend");
                            list_widget.render(chunks[1], f.buffer_mut());

                            let help = Line::styled(
                                "  ↑↓ navigate  │  Enter: select  │  Esc: cancel",
                                Style::default().fg(theme.muted()),
                            );
                            let help_para = Paragraph::new(help);
                            help_para.render(chunks[2], f.buffer_mut());
                        })?;

                        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
                            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                                use crossterm::event::KeyCode;
                                match key.code {
                                    KeyCode::Up => list_state.move_up(),
                                    KeyCode::Down => list_state.move_down(),
                                    KeyCode::Enter => {
                                        let mut updated_agent = agent.clone();
                                        updated_agent.model_backend = Some(models[list_state.selected].clone());
                                        current_state = Some(AgentConfigState::ChoosingInstances {
                                            agent: updated_agent,
                                            instances: agent.instances,
                                        });
                                        break;
                                    }
                                    KeyCode::Esc => {
                                        current_state = Some(AgentConfigState::SelectingAction {
                                            agents: agents.clone(),
                                            selected: agents.iter().position(|a| a.id == agent.id).unwrap_or(0),
                                        });
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                Some(AgentConfigState::ChoosingInstances { agent, instances }) => {
                    let mut current_instances = instances;
                    let min_instances = 1u32;
                    let max_instances = 10u32;

                    loop {
                        terminal.draw(|f| {
                            let area = f.area();
                            let chunks = Layout::default()
                                .direction(Direction::Vertical)
                                .margin(3)
                                .constraints([
                                    Constraint::Length(4),
                                    Constraint::Length(3),
                                    Constraint::Min(1),
                                    Constraint::Length(2),
                                ])
                                .split(area);

                            let title_block = ratatui::widgets::Block::default()
                                .borders(ratatui::widgets::Borders::BOTTOM)
                                .border_style(Style::default().fg(theme.border()));
                            let inner_title = title_block.inner(chunks[0]);
                            title_block.render(chunks[0], f.buffer_mut());

                            let title = Line::styled(
                                "◇ SET INSTANCES",
                                Style::default().fg(theme.accent()).add_modifier(Modifier::BOLD),
                            );
                            let subtitle = Line::styled(
                                format!("  Agent: {} | Model: {}", 
                                    agent.id, 
                                    agent.model_backend.as_deref().unwrap_or("default")),
                                Style::default().fg(theme.muted()),
                            );
                            let title_para = ratatui::widgets::Paragraph::new(vec![title, subtitle]);
                            title_para.render(inner_title, f.buffer_mut());

                            let instances_text = Line::styled(
                                format!("  Instances: {} (use ← → to adjust)", current_instances),
                                Style::default().fg(theme.fg()).add_modifier(Modifier::BOLD),
                            );
                            let instances_para = Paragraph::new(instances_text)
                                .alignment(Alignment::Center);
                            instances_para.render(chunks[1], f.buffer_mut());

                            let help = Line::styled(
                                "  ← → adjust  │  Enter: confirm  │  Esc: back",
                                Style::default().fg(theme.muted()),
                            );
                            let help_para = Paragraph::new(help);
                            help_para.render(chunks[3], f.buffer_mut());
                        })?;

                        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
                            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                                use crossterm::event::KeyCode;
                                match key.code {
                                    KeyCode::Left => {
                                        if current_instances > min_instances {
                                            current_instances -= 1;
                                        }
                                    }
                                    KeyCode::Right => {
                                        if current_instances < max_instances {
                                            current_instances += 1;
                                        }
                                    }
                                    KeyCode::Enter => {
                                        let mut updated_agent = agent.clone();
                                        updated_agent.instances = current_instances;
                                        
                                        // Update the agent in the list
                                        if let Some(pos) = agents.iter().position(|a| a.id == updated_agent.id) {
                                            agents[pos] = updated_agent;
                                        }
                                        
                                        current_state = Some(AgentConfigState::SelectingAction {
                                            agents: agents.clone(),
                                            selected: agents.iter().position(|a| a.id == agent.id).unwrap_or(0),
                                        });
                                        break;
                                    }
                                    KeyCode::Esc => {
                                        current_state = Some(AgentConfigState::ChoosingModel {
                                            agent: agent.clone(),
                                            models: vec![
                                                "anthropic/claude-sonnet-4-5".to_string(),
                                                "anthropic/claude-3-5-sonnet".to_string(),
                                                "gemini/gemini-2.5-pro".to_string(),
                                                "openai/gpt-4o".to_string(),
                                                "openai/gpt-4o-mini".to_string(),
                                                "groq/llama-3.3-70b-versatile".to_string(),
                                                "fireworks/accounts/fireworks/models/glm-5".to_string(),
                                            ],
                                            selected: 0,
                                        });
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                None => {
                    current_state = Some(AgentConfigState::SelectingAction {
                        agents: agents.clone(),
                        selected: 0,
                    });
                }
            }
        }
    }
}