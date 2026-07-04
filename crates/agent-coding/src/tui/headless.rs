use adad_core::Error;
use ratatui::{
    backend::TestBackend,
    text::{Line, Text},
    widgets::{Block, Paragraph},
    Frame, Terminal,
};

use super::{Theme, ThemeKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HeadlessEvent {
    Render,
    SelectTheme(ThemeKind),
    Key(char),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrameLog {
    pub theme: ThemeKind,
    pub frames: Vec<String>,
    pub keys: Vec<char>,
}

pub fn run_headless(events: &[HeadlessEvent]) -> Result<FrameLog, Error> {
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).map_err(|_| Error::Io)?;
    let mut state = HeadlessState::default();
    let mut frames = Vec::new();
    let mut keys = Vec::new();

    for event in events {
        match event {
            HeadlessEvent::Render => {}
            HeadlessEvent::SelectTheme(kind) => state.theme = Theme::select(*kind),
            HeadlessEvent::Key(key) => keys.push(*key),
        }

        let snapshot = state.snapshot();
        terminal
            .draw(|frame| render_scaffold(frame, &state))
            .map_err(|_| Error::Io)?;
        frames.push(snapshot);
    }

    Ok(FrameLog {
        theme: state.theme.kind,
        frames,
        keys,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HeadlessState {
    theme: Theme,
    title: String,
    status: String,
}

impl Default for HeadlessState {
    fn default() -> Self {
        Self {
            theme: Theme::high_contrast(),
            title: "ADAD".to_owned(),
            status: "Loading: initializing TUI scaffold".to_owned(),
        }
    }
}

impl HeadlessState {
    fn lines(&self) -> Vec<String> {
        vec![
            self.title.clone(),
            "Theme: high-contrast".to_owned(),
            self.status.clone(),
        ]
    }

    fn snapshot(&self) -> String {
        self.lines().join("\n")
    }
}

fn render_scaffold(frame: &mut Frame<'_>, state: &HeadlessState) {
    let lines = state
        .lines()
        .into_iter()
        .enumerate()
        .map(|(index, line)| {
            if index == 0 {
                Line::styled(line, state.theme.title_style())
            } else if line.starts_with("Loading:") {
                Line::styled(line, state.theme.loading_style())
            } else {
                Line::styled(line, state.theme.body_style())
            }
        })
        .collect::<Vec<_>>();
    let paragraph = Paragraph::new(Text::from(lines))
        .style(state.theme.body_style())
        .block(Block::bordered().title("ADAD"));

    frame.render_widget(paragraph, frame.area());
}
