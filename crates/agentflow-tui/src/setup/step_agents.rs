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

const MODELS: &[&str] = &[
    "anthropic/claude-sonnet-4-5",
    "anthropic/claude-3-5-sonnet",
    "gemini/gemini-2.5-pro",
    "openai/gpt-4o",
    "openai/gpt-4o-mini",
    "groq/llama-3.3-70b-versatile",
    "fireworks/accounts/fireworks/models/glm-5",
];

enum AgentConfigState {
    MainList {
        agents: Vec<AgentConfig>,
        selected: usize,
        focused_field: usize,
    },
    ModelPicker {
        agents: Vec<AgentConfig>,
        agent_idx: usize,
        selected: usize,
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

        // Default agents if registry doesn't exist
        // Nexus always has exactly 1 instance (immutable)
        if agents.is_empty() {
            agents.push(AgentConfig {
                id: "nexus".to_string(),
                cli: "claude".to_string(),
                active: true,
                instances: 1, // Nexus is always 1 instance (orchestrator singleton)
                model_backend: Some("anthropic/claude-sonnet-4-5".to_string()),
                routing_key: Some("nexus-key".to_string()),
                github_token_env: Some("AGENT_NEXUS_GITHUB_TOKEN".to_string()),
            });
            agents.push(AgentConfig {
                id: "forge".to_string(),
                cli: "claude".to_string(),
                active: true,
                instances: 2,
                model_backend: Some("anthropic/claude-sonnet-4-5".to_string()),
                routing_key: Some("forge-key".to_string()),
                github_token_env: Some("AGENT_FORGE_GITHUB_TOKEN".to_string()),
            });
            agents.push(AgentConfig {
                id: "sentinel".to_string(),
                cli: "claude".to_string(),
                active: true,
                instances: 1,
                model_backend: Some("gemini/gemini-2.5-pro".to_string()),
                routing_key: Some("sentinel-key".to_string()),
                github_token_env: Some("AGENT_SENTINEL_GITHUB_TOKEN".to_string()),
            });
            agents.push(AgentConfig {
                id: "vessel".to_string(),
                cli: "claude".to_string(),
                active: true,
                instances: 1,
                model_backend: Some("groq/llama-3.3-70b-versatile".to_string()),
                routing_key: Some("vessel-key".to_string()),
                github_token_env: Some("AGENT_VESSEL_GITHUB_TOKEN".to_string()),
            });
            agents.push(AgentConfig {
                id: "lore".to_string(),
                cli: "claude".to_string(),
                active: true,
                instances: 1,
                model_backend: Some("openai/gpt-4o-mini".to_string()),
                routing_key: Some("lore-key".to_string()),
                github_token_env: Some("AGENT_LORE_GITHUB_TOKEN".to_string()),
            });
        }

        let mut state = AgentConfigState::MainList {
            agents,
            selected: 0,
            focused_field: 0,
        };

        loop {
            match &mut state {
                AgentConfigState::MainList { agents, selected, focused_field } => {
                    loop {
                        terminal.draw(|f| {
                            let area = f.area();
                            let chunks = Layout::default()
                                .direction(Direction::Vertical)
                                .margin(2)
                                .constraints([
                                    Constraint::Length(4),
                                    Constraint::Length(1),
                                    Constraint::Min(5),
                                    Constraint::Length(3),
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
                                "  Edit instances, model, and active status per agent",
                                Style::default().fg(theme.muted()),
                            );
                            let title_para = ratatui::widgets::Paragraph::new(vec![title, subtitle]);
                            title_para.render(inner_title, f.buffer_mut());

                            // Header row
                            let header = Line::styled(
                                format!(
                                    "  {:<12} {:<10} {:<12} {}",
                                    "AGENT", "ACTIVE", "INSTANCES", "MODEL BACKEND"
                                ),
                                Style::default().fg(theme.accent()).add_modifier(Modifier::BOLD),
                            );
                            let header_para = Paragraph::new(header);
                            header_para.render(chunks[1], f.buffer_mut());

                            // Agent rows
                            let mut current_y = chunks[2].y;
                            let row_height = 1u16;

                            for (i, agent) in agents.iter().enumerate() {
                                if current_y + row_height > chunks[2].y + chunks[2].height {
                                    break;
                                }

                                let active_str = if agent.active { "✓ ON " } else { "✗ OFF" };
                                let instances_str = format!("{}", agent.instances);
                                let model_str = agent.model_backend.as_deref().unwrap_or("none");

                                let is_selected = i == *selected;
                                let row_style = if is_selected {
                                    Style::default()
                                        .fg(theme.accent())
                                        .add_modifier(Modifier::BOLD)
                                } else {
                                    Style::default().fg(theme.fg())
                                };

                                let prefix = if is_selected { "▶ " } else { "  " };

                                // Build the row with field highlighting
                                let mut row_text = String::new();
                                row_text.push_str(&format!("{}{:<12}", prefix, agent.id));

                                // Active field
                                if is_selected && *focused_field == 0 {
                                    row_text.push_str(&format!("[{}]", active_str));
                                } else {
                                    row_text.push_str(&format!(" {:<10}", active_str));
                                }

                                // Instances field (nexus is locked at 1)
                                let is_nexus = agent.id == "nexus";
                                let instances_editable = !is_nexus;
                                let instances_display = if is_nexus {
                                    format!("{} (locked)", instances_str)
                                } else {
                                    instances_str.clone()
                                };
                                if is_selected && *focused_field == 1 {
                                    if instances_editable {
                                        row_text.push_str(&format!("[{:<12}]", instances_str));
                                    } else {
                                        row_text.push_str(&format!(" {:<12}", instances_display));
                                    }
                                } else {
                                    row_text.push_str(&format!(" {:<12}", instances_display));
                                }

                                // Model field
                                if is_selected && *focused_field == 2 {
                                    row_text.push_str(&format!("[{}]", model_str));
                                } else {
                                    row_text.push_str(&format!(" {}", model_str));
                                }

                                let row_line = Line::styled(row_text, row_style);
                                let row_para = Paragraph::new(row_line);
                                row_para.render(
                                    ratatui::layout::Rect::new(
                                        chunks[2].x,
                                        current_y,
                                        chunks[2].width,
                                        row_height,
                                    ),
                                    f.buffer_mut(),
                                );
                                current_y += row_height;
                            }

                            // Help text
                            let help_lines = vec![
                                Line::styled(
                                    "  ↑↓ select agent  │  Tab: next field  │  Space: toggle active  │  ←→: adjust instances",
                                    Style::default().fg(theme.muted()),
                                ),
                                Line::styled(
                                    "  Enter on model: pick model  │  Shift+Tab: finish  │  Nexus: always 1 instance (locked)",
                                    Style::default().fg(theme.muted()),
                                ),
                            ];
                            let help_para = Paragraph::new(help_lines);
                            help_para.render(chunks[3], f.buffer_mut());
                        })?;

                        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
                            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                                use crossterm::event::KeyCode;
                                use crossterm::event::KeyModifiers;

                                match key.code {
                                    KeyCode::Up => {
                                        if *selected > 0 {
                                            *selected -= 1;
                                        }
                                    }
                                    KeyCode::Down => {
                                        if *selected + 1 < agents.len() {
                                            *selected += 1;
                                        }
                                    }
                                    KeyCode::Tab => {
                                        if key.modifiers.contains(KeyModifiers::SHIFT) {
                                            config.agents = agents.clone();
                                            return Ok(());
                                        } else {
                                            *focused_field = (*focused_field + 1) % 3;
                                        }
                                    }
                                    KeyCode::BackTab => {
                                        config.agents = agents.clone();
                                        return Ok(());
                                    }
                                    KeyCode::Char(' ') => {
                                        if *focused_field == 0 {
                                            agents[*selected].active = !agents[*selected].active;
                                        }
                                    }
                                    KeyCode::Left => {
                                        if *focused_field == 1 && agents[*selected].id != "nexus" && agents[*selected].instances > 1 {
                                            agents[*selected].instances -= 1;
                                        }
                                    }
                                    KeyCode::Right => {
                                        if *focused_field == 1 && agents[*selected].id != "nexus" && agents[*selected].instances < 10 {
                                            agents[*selected].instances += 1;
                                        }
                                    }
                                    KeyCode::Enter => {
                                        if *focused_field == 2 {
                                            let current_model = agents[*selected]
                                                .model_backend
                                                .as_deref()
                                                .unwrap_or("");
                                            let initial_idx = MODELS
                                                .iter()
                                                .position(|m| *m == current_model)
                                                .unwrap_or(0);
                                            state = AgentConfigState::ModelPicker {
                                                agents: agents.clone(),
                                                agent_idx: *selected,
                                                selected: initial_idx,
                                            };
                                            break;
                                        }
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
                AgentConfigState::ModelPicker { agents, agent_idx, selected } => {
                    let agent_idx_val = *agent_idx;
                    let mut list_state = SelectableListState::new(
                        MODELS.iter().map(|s| s.to_string()).collect(),
                    );
                    list_state.selected = *selected;

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
                                format!("  Choose model for agent: {}", agents[agent_idx_val].id),
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
                                        agents[agent_idx_val].model_backend =
                                            Some(MODELS[list_state.selected].to_string());
                                        state = AgentConfigState::MainList {
                                            agents: agents.clone(),
                                            selected: agent_idx_val,
                                            focused_field: 2,
                                        };
                                        break;
                                    }
                                    KeyCode::Esc => {
                                        state = AgentConfigState::MainList {
                                            agents: agents.clone(),
                                            selected: agent_idx_val,
                                            focused_field: 2,
                                        };
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
