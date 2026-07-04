use adad_core::Error;
use ratatui::{
    backend::TestBackend,
    text::{Line, Text},
    widgets::{Block, Paragraph},
    Frame, Terminal,
};

use crate::DEFAULT_LOCAL_MODEL;

use super::{escape_terminal_text, Theme, ThemeKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgentChatEvent {
    Render,
    SelectTheme(ThemeKind),
    Key(char),
    StreamDelta(String),
    FinishStream,
    Error(Error),
    SetProviderModel { provider: String, model: String },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChatViewState {
    Loading,
    Empty,
    Ready,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentChatFrameLog {
    pub theme: ThemeKind,
    pub frames: Vec<String>,
    pub sent_prompts: Vec<String>,
    pub state: ChatViewState,
}

pub fn run_agent_chat_headless(events: &[AgentChatEvent]) -> Result<AgentChatFrameLog, Error> {
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).map_err(|_| Error::Io)?;
    let mut state = AgentChatState::default();
    let mut frames = Vec::new();

    for event in events {
        match event {
            AgentChatEvent::Render => {}
            AgentChatEvent::SelectTheme(kind) => state.theme = Theme::select(*kind),
            AgentChatEvent::Key(key) => state.handle_key(*key),
            AgentChatEvent::StreamDelta(delta) => state.stream_delta(delta),
            AgentChatEvent::FinishStream => state.finish_stream(),
            AgentChatEvent::Error(error) => state.set_error(error),
            AgentChatEvent::SetProviderModel { provider, model } => {
                state.provider = provider.clone();
                state.model = model.clone();
            }
        }

        let snapshot = state.snapshot();
        terminal
            .draw(|frame| render_agent_chat(frame, &state))
            .map_err(|_| Error::Io)?;
        frames.push(snapshot);
    }

    Ok(AgentChatFrameLog {
        theme: state.theme.kind,
        frames,
        sent_prompts: state.sent_prompts,
        state: state.view_state,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AgentChatState {
    theme: Theme,
    provider: String,
    model: String,
    input: String,
    sent_prompts: Vec<String>,
    messages: Vec<RenderedMessage>,
    view_state: ChatViewState,
    error_message: Option<String>,
}

impl Default for AgentChatState {
    fn default() -> Self {
        Self {
            theme: Theme::high_contrast(),
            provider: "local".to_owned(),
            model: DEFAULT_LOCAL_MODEL.to_owned(),
            input: String::new(),
            sent_prompts: Vec::new(),
            messages: Vec::new(),
            view_state: ChatViewState::Empty,
            error_message: None,
        }
    }
}

impl AgentChatState {
    fn handle_key(&mut self, key: char) {
        if key == '\n' {
            let prompt = self.input.trim().to_owned();
            if prompt.is_empty() {
                return;
            }
            self.sent_prompts.push(prompt.clone());
            self.messages.push(RenderedMessage {
                role: "User".to_owned(),
                content: prompt,
            });
            self.input.clear();
            self.error_message = None;
            self.view_state = ChatViewState::Loading;
            return;
        }

        self.input.push(key);
        if self.view_state == ChatViewState::Empty {
            self.view_state = ChatViewState::Ready;
        }
    }

    fn stream_delta(&mut self, delta: &str) {
        if !matches!(self.messages.last(), Some(message) if message.role == "Assistant") {
            self.messages.push(RenderedMessage {
                role: "Assistant".to_owned(),
                content: String::new(),
            });
        }

        let last = self
            .messages
            .last_mut()
            .expect("assistant message just inserted");
        last.content.push_str(&escape_terminal_text(delta));
        self.error_message = None;
        self.view_state = ChatViewState::Ready;
    }

    fn finish_stream(&mut self) {
        if self.view_state == ChatViewState::Loading {
            self.view_state = ChatViewState::Ready;
        }
    }

    fn set_error(&mut self, error: &Error) {
        self.error_message = Some(error.user_message());
        self.view_state = ChatViewState::Error;
    }

    fn lines(&self) -> Vec<String> {
        let mut lines = vec![
            "ADAD Agent".to_owned(),
            format!("Provider: {}", self.provider),
            format!("Model: {}", self.model),
            self.state_line(),
            format!("Input: {}", self.input),
        ];

        if self.messages.is_empty() {
            lines.push("Transcript: empty".to_owned());
        } else {
            lines.push("Transcript:".to_owned());
            for message in &self.messages {
                lines.push(format!(
                    "{}: {}",
                    message.role,
                    escape_terminal_text(&message.content)
                ));
            }
        }

        lines
    }

    fn state_line(&self) -> String {
        match self.view_state {
            ChatViewState::Loading => "State: Loading - waiting for model stream".to_owned(),
            ChatViewState::Empty => "State: Empty - type a prompt and press Enter".to_owned(),
            ChatViewState::Ready => "State: Ready".to_owned(),
            ChatViewState::Error => format!(
                "State: Error - {}",
                self.error_message.as_deref().unwrap_or("Operation failed")
            ),
        }
    }

    fn snapshot(&self) -> String {
        self.lines().join("\n")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RenderedMessage {
    role: String,
    content: String,
}

fn render_agent_chat(frame: &mut Frame<'_>, state: &AgentChatState) {
    let lines = state
        .lines()
        .into_iter()
        .enumerate()
        .map(|(index, line)| {
            if index == 0 {
                Line::styled(line, state.theme.title_style())
            } else if line.starts_with("State: Loading") {
                Line::styled(line, state.theme.loading_style())
            } else if line.starts_with("State: Error") {
                Line::styled(line, state.theme.error_style())
            } else {
                Line::styled(line, state.theme.body_style())
            }
        })
        .collect::<Vec<_>>();
    let paragraph = Paragraph::new(Text::from(lines))
        .style(state.theme.body_style())
        .block(Block::bordered().title("Agent"));

    frame.render_widget(paragraph, frame.area());
}
