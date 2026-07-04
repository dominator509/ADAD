use ratatui::style::{Color, Modifier, Style};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThemeKind {
    HighContrast,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Theme {
    pub kind: ThemeKind,
    pub foreground: Color,
    pub background: Color,
    pub accent: Color,
    pub error: Color,
    pub loading: Color,
}

impl Theme {
    #[must_use]
    pub fn select(kind: ThemeKind) -> Self {
        match kind {
            ThemeKind::HighContrast => Self::high_contrast(),
        }
    }

    #[must_use]
    pub fn high_contrast() -> Self {
        Self {
            kind: ThemeKind::HighContrast,
            foreground: Color::White,
            background: Color::Black,
            accent: Color::Cyan,
            error: Color::LightRed,
            loading: Color::Yellow,
        }
    }

    #[must_use]
    pub fn body_style(&self) -> Style {
        Style::default().fg(self.foreground).bg(self.background)
    }

    #[must_use]
    pub fn title_style(&self) -> Style {
        self.body_style()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    #[must_use]
    pub fn loading_style(&self) -> Style {
        self.body_style()
            .fg(self.loading)
            .add_modifier(Modifier::BOLD)
    }

    #[must_use]
    pub fn error_style(&self) -> Style {
        self.body_style()
            .fg(self.error)
            .add_modifier(Modifier::BOLD)
    }
}
