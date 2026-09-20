use std::{
    io,
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
    time::Duration,
};

use adad_core::Error;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    backend::TestBackend,
    text::{Line, Text},
    widgets::{Block, Paragraph},
    Frame, Terminal,
};

use crate::{
    AgentLoop, ExecutionRegistry, LoopMessage, OpenAiAgentModel, OpenAiCompatClient,
    WorkspaceToolExecutor, DEFAULT_LOCAL_MODEL,
};

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

/// Run the keyboard-driven agent chat against the supplied provider client.
///
/// Provider I/O runs on a worker thread so the terminal event loop can keep
/// repainting while a local model responds. SSE text fragments are delivered
/// to the renderer as they arrive; arbitrary provider text is never interpreted
/// as a tool call.
pub fn run_agent_chat(client: OpenAiCompatClient) -> Result<(), Error> {
    run_agent_chat_with_provider(client, "local", DEFAULT_LOCAL_MODEL)
}

/// Run the interactive agent chat with the provider metadata selected by the
/// runtime configuration. The legacy `run_agent_chat` wrapper remains the
/// local-default API for callers that do not need a custom label.
pub fn run_agent_chat_with_provider(
    client: OpenAiCompatClient,
    provider: impl Into<String>,
    model: impl Into<String>,
) -> Result<(), Error> {
    enable_raw_mode().map_err(|_| Error::Io)?;
    let _cleanup = TerminalCleanup;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(|_| Error::Io)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|_| Error::Io)?;
    let mut state = AgentChatState::with_provider_model(provider, model);
    let mut transcript = Vec::<LoopMessage>::new();
    let workspace = std::env::current_dir().map_err(|_| Error::Io)?;
    let registry = ExecutionRegistry::with_workspace_tools();
    let mut pending: Option<(Receiver<StreamEvent>, thread::JoinHandle<()>)> = None;

    loop {
        let mut request_finished = false;
        if let Some((receiver, _)) = pending.as_ref() {
            loop {
                match receiver.try_recv() {
                    Ok(StreamEvent::Delta(delta)) => state.stream_delta(&delta),
                    Ok(StreamEvent::Complete(content)) => {
                        state.finish_stream();
                        transcript.push(LoopMessage::assistant(content));
                        request_finished = true;
                    }
                    Ok(StreamEvent::Failed(error)) => {
                        state.set_error(&error);
                        request_finished = true;
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        state.set_error(&Error::Provider);
                        request_finished = true;
                        break;
                    }
                }
            }
        }
        if request_finished {
            if let Some((_, handle)) = pending.take() {
                let _ = handle.join();
            }
        }

        terminal
            .draw(|frame| render_agent_chat(frame, &state))
            .map_err(|_| Error::Io)?;

        if event::poll(Duration::from_millis(50)).map_err(|_| Error::Io)? {
            let Event::Key(key) = event::read().map_err(|_| Error::Io)? else {
                continue;
            };
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Esc => break,
                KeyCode::Backspace if pending.is_none() => {
                    state.input.pop();
                }
                KeyCode::Enter if pending.is_none() => {
                    let previous_count = state.sent_prompts.len();
                    state.handle_key('\n');
                    if state.sent_prompts.len() == previous_count {
                        continue;
                    }
                    let prompt = state.sent_prompts.last().cloned().ok_or(Error::Provider)?;
                    transcript.push(LoopMessage::user(prompt));
                    let messages = transcript.clone();
                    let request_client = client.clone();
                    let worker_registry = registry.clone();
                    let worker_workspace = workspace.clone();
                    let (sender, receiver) = mpsc::channel();
                    let handle = thread::spawn(move || {
                        let mut model =
                            OpenAiAgentModel::with_tools(request_client, &worker_registry);
                        let mut executor = match WorkspaceToolExecutor::new(worker_workspace) {
                            Ok(executor) => executor,
                            Err(error) => {
                                let _ = sender.send(StreamEvent::Failed(error));
                                return;
                            }
                        };
                        let mut on_delta = |delta: &str| {
                            let _ = sender.send(StreamEvent::Delta(delta.to_owned()));
                        };
                        let result = AgentLoop::new(worker_registry, 8)
                            .run_transcript_with_callback(
                                messages,
                                &mut model,
                                &mut executor,
                                &mut on_delta,
                            );
                        match result {
                            Ok(result) => match result.final_answer {
                                Some(content) => {
                                    let _ = sender.send(StreamEvent::Complete(content));
                                }
                                None => {
                                    let _ = sender.send(StreamEvent::Failed(Error::Provider));
                                }
                            },
                            Err(error) => {
                                let _ = sender.send(StreamEvent::Failed(error));
                            }
                        }
                    });
                    pending = Some((receiver, handle));
                }
                KeyCode::Char(ch) if pending.is_none() => state.handle_key(ch),
                _ => {}
            }
        }
    }

    if let Some((_, handle)) = pending.take() {
        let _ = handle.join();
    }

    Ok(())
}

enum StreamEvent {
    Delta(String),
    Complete(String),
    Failed(Error),
}

struct TerminalCleanup;

impl Drop for TerminalCleanup {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen);
    }
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
    fn with_provider_model(provider: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            model: model.into(),
            ..Self::default()
        }
    }

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
