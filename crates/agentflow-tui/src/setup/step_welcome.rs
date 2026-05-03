use anyhow::Result;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use ratatui::Terminal;
use std::io;

use crate::util::logo::{get_logo_lines, version_string, TAGLINE};
use crate::util::theme::Theme;

pub struct WelcomeStep;

impl WelcomeStep {
    pub fn new() -> Self {
        Self
    }

    pub fn render(
        &self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        _theme: &Theme,
    ) -> Result<()> {
        terminal.draw(|f| {
            let area = f.area();
            let logo_lines = get_logo_lines();

            let total_content_height = logo_lines.len() as u16 + 5;
            let start_y = if area.height > total_content_height {
                (area.height - total_content_height) / 2
            } else {
                0
            };

            let content_area = Rect::new(0, start_y, area.width, total_content_height);

            let mut lines: Vec<Line> = Vec::new();

            for logo_line in &logo_lines {
                lines.push(Line::styled(
                    logo_line.clone(),
                    Style::default()
                        .fg(Theme::default().accent())
                        .add_modifier(Modifier::BOLD),
                ));
            }

            lines.push(Line::raw(""));
            lines.push(Line::styled(
                TAGLINE.to_string(),
                Style::default().fg(Theme::default().muted()),
            ));
            lines.push(Line::raw(""));
            lines.push(Line::styled(
                version_string(),
                Style::default().fg(Theme::default().fg()),
            ));
            lines.push(Line::raw(""));

            let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
            paragraph.render(content_area, f.buffer_mut());

            let help_line = Line::styled(
                "Press Enter to start setup...",
                Style::default().fg(Theme::default().muted()),
            );
            let help_para = Paragraph::new(help_line).alignment(Alignment::Center);
            let help_area = Rect::new(0, area.height - 2, area.width, 1);
            help_para.render(help_area, f.buffer_mut());
        })?;

        loop {
            if crossterm::event::poll(std::time::Duration::from_millis(100))? {
                if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                    if key.code == crossterm::event::KeyCode::Enter
                        || key.code == crossterm::event::KeyCode::Esc
                        || key.code == crossterm::event::KeyCode::Char(' ')
                    {
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}
