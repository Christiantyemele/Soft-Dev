use anyhow::Result;
use ratatui::backend::CrosstermBackend;
use ratatui::prelude::Widget;
use ratatui::Terminal;
use std::io;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

use crate::setup::SetupConfig;
use crate::util::theme::Theme;
use crate::widgets::check::{CheckList, CheckState};
use crate::widgets::input::InputWidget;

pub struct RepoStep;

impl RepoStep {
    pub fn new() -> Self {
        Self
    }

    pub async fn render(
        &self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        _theme: &Theme,
        config: &mut SetupConfig,
    ) -> Result<()> {
        let mut repo_input = Input::new(config.repo.clone());
        let mut workspace_input = Input::new(config.workspace_dir.clone());
        let mut focused_field = 0;

        let repo_regex = regex::Regex::new(r"^[a-zA-Z0-9_-]+/[a-zA-Z0-9_.-]+$").unwrap();

        loop {
            let repo_valid = repo_regex.is_match(repo_input.value());
            let workspace_valid = !workspace_input.value().is_empty();

            terminal.draw(|f| {
                let area = f.area();
                let y_start = area.height / 2 - 4;

                let repo_widget_area = ratatui::layout::Rect {
                    x: 2,
                    y: y_start,
                    width: area.width - 4,
                    height: 3,
                };
                let repo_widget = InputWidget::new(&repo_input, "GitHub Repository (owner/repo)")
                    .focused(focused_field == 0);
                repo_widget.render(repo_widget_area, f.buffer_mut());

                let ws_widget_area = ratatui::layout::Rect {
                    x: 2,
                    y: y_start + 4,
                    width: area.width - 4,
                    height: 3,
                };
                let ws_widget = InputWidget::new(&workspace_input, "Workspace directory")
                    .focused(focused_field == 1);
                ws_widget.render(ws_widget_area, f.buffer_mut());

                let mut checks = Vec::new();
                if repo_valid {
                    checks.push(("Repository format valid".to_string(), CheckState::Pass));
                } else {
                    checks.push((
                        "Invalid repository format (owner/repo)".to_string(),
                        CheckState::Fail,
                    ));
                }
                if workspace_valid {
                    checks.push(("Workspace directory set".to_string(), CheckState::Pass));
                } else {
                    checks.push(("Workspace directory empty".to_string(), CheckState::Fail));
                }
                let check_area = ratatui::layout::Rect {
                    x: 2,
                    y: y_start + 8,
                    width: area.width - 4,
                    height: 4,
                };
                let check_list = CheckList::new(checks);
                check_list.render(check_area, f.buffer_mut());
            })?;

            if crossterm::event::poll(std::time::Duration::from_millis(100))? {
                if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                    use crossterm::event::KeyCode;
                    match key.code {
                        KeyCode::Tab => {
                            focused_field = (focused_field + 1) % 2;
                        }
                        KeyCode::Enter => {
                            if repo_valid && workspace_valid {
                                config.repo = repo_input.value().to_string();
                                config.workspace_dir = workspace_input.value().to_string();
                                break;
                            }
                        }
                        KeyCode::Esc => {
                            return Err(anyhow::anyhow!("Setup cancelled"));
                        }
                        _ => {
                            let event = crossterm::event::Event::Key(key);
                            let input = match focused_field {
                                0 => &mut repo_input,
                                1 => &mut workspace_input,
                                _ => unreachable!(),
                            };
                            input.handle_event(&event);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
