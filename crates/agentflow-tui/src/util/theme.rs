use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Theme {
    Dark,
    Light,
}

impl Default for Theme {
    fn default() -> Self {
        match std::env::var("AGENTFLOW_THEME").as_deref() {
            Ok("light") => Theme::Light,
            _ => Theme::Dark,
        }
    }
}

impl Theme {
    pub fn bg(&self) -> Color {
        match self {
            Theme::Dark => Color::Rgb(30, 30, 46),
            Theme::Light => Color::Rgb(239, 241, 245),
        }
    }

    pub fn fg(&self) -> Color {
        match self {
            Theme::Dark => Color::Rgb(205, 214, 244),
            Theme::Light => Color::Rgb(68, 73, 90),
        }
    }

    pub fn border(&self) -> Color {
        match self {
            Theme::Dark => Color::Rgb(137, 180, 250),
            Theme::Light => Color::Rgb(114, 135, 253),
        }
    }

    pub fn success(&self) -> Color {
        match self {
            Theme::Dark => Color::Rgb(166, 227, 161),
            Theme::Light => Color::Rgb(64, 160, 43),
        }
    }

    pub fn error(&self) -> Color {
        match self {
            Theme::Dark => Color::Rgb(243, 139, 168),
            Theme::Light => Color::Rgb(218, 41, 71),
        }
    }

    pub fn warning(&self) -> Color {
        match self {
            Theme::Dark => Color::Rgb(249, 226, 175),
            Theme::Light => Color::Rgb(223, 142, 29),
        }
    }

    pub fn accent(&self) -> Color {
        match self {
            Theme::Dark => Color::Rgb(137, 180, 250),
            Theme::Light => Color::Rgb(114, 135, 253),
        }
    }

    pub fn muted(&self) -> Color {
        match self {
            Theme::Dark => Color::Rgb(88, 91, 112),
            Theme::Light => Color::Rgb(160, 166, 187),
        }
    }

    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.accent())
            .add_modifier(Modifier::BOLD)
    }

    pub fn success_style(&self) -> Style {
        Style::default().fg(self.success())
    }

    pub fn error_style(&self) -> Style {
        Style::default().fg(self.error())
    }

    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.warning())
    }

    pub fn text_style(&self) -> Style {
        Style::default().fg(self.fg())
    }

    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.muted())
    }
}
