use adad_core::Error;
use ratatui::{
    backend::TestBackend,
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Paragraph},
    Frame, Terminal,
};

use crate::{ProvisionHandle, ProvisionTarget};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VpsEvent {
    Render,
    Key(char),
    SetTarget(ProvisionTarget),
    Provisioned(ProvisionHandle),
    Error(Error),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VpsViewState {
    Empty,
    Loading,
    Ready,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VpsFrameLog {
    pub frames: Vec<String>,
    pub actions: Vec<VpsAction>,
    pub state: VpsViewState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VpsAction {
    Provision,
}

pub fn run_headless(events: &[VpsEvent]) -> Result<VpsFrameLog, Error> {
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).map_err(|_| Error::Io)?;
    let mut state = VpsState::default();
    let mut frames = Vec::new();

    for event in events {
        match event {
            VpsEvent::Render => {}
            VpsEvent::Key(key) => state.handle_key(*key),
            VpsEvent::SetTarget(target) => state.target = target.clone(),
            VpsEvent::Provisioned(handle) => state.set_handle(handle),
            VpsEvent::Error(error) => state.set_error(error),
        }

        let snapshot = state.snapshot();
        terminal
            .draw(|frame| render_vps(frame, &state))
            .map_err(|_| Error::Io)?;
        frames.push(snapshot);
    }

    Ok(VpsFrameLog {
        frames,
        actions: state.actions,
        state: state.view_state,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct VpsState {
    target: ProvisionTarget,
    actions: Vec<VpsAction>,
    view_state: VpsViewState,
    handle: Option<ProvisionHandle>,
    error_message: Option<String>,
}

impl Default for VpsState {
    fn default() -> Self {
        Self {
            target: ProvisionTarget::new("mock-hidden-service.onion", "debian", 22),
            actions: Vec::new(),
            view_state: VpsViewState::Empty,
            handle: None,
            error_message: None,
        }
    }
}

impl VpsState {
    fn handle_key(&mut self, key: char) {
        if matches!(key, 'p' | 'P') {
            self.actions.push(VpsAction::Provision);
            self.error_message = None;
            self.view_state = VpsViewState::Loading;
        }
    }

    fn set_handle(&mut self, handle: &ProvisionHandle) {
        self.handle = Some(handle.clone());
        self.error_message = None;
        self.view_state = VpsViewState::Ready;
    }

    fn set_error(&mut self, error: &Error) {
        self.error_message = Some(error.user_message());
        self.view_state = VpsViewState::Error;
    }

    fn lines(&self) -> Vec<String> {
        let mut lines = vec![
            "ADAD VPS Deploy".to_owned(),
            self.state_line(),
            "Keys: p provision".to_owned(),
            format!(
                "Target: {}@{}:{}",
                self.target.user,
                redact_host(&self.target.host),
                self.target.port
            ),
        ];

        if let Some(handle) = &self.handle {
            lines.push(format!("Provisioned: {}", redact_host(&handle.target.host)));
            lines.push(format!("Output: {}", handle.stdout));
        } else {
            lines.push("Provision result: empty".to_owned());
        }

        lines
    }

    fn state_line(&self) -> String {
        match self.view_state {
            VpsViewState::Empty => "State: Empty - choose a deploy action".to_owned(),
            VpsViewState::Loading => "State: Loading - SSH setup pending".to_owned(),
            VpsViewState::Ready => "State: Ready".to_owned(),
            VpsViewState::Error => format!(
                "State: Error - {}",
                self.error_message
                    .as_deref()
                    .unwrap_or("Provisioning failed")
            ),
        }
    }

    fn snapshot(&self) -> String {
        self.lines().join("\n")
    }
}

fn render_vps(frame: &mut Frame<'_>, state: &VpsState) {
    let lines = state
        .lines()
        .into_iter()
        .enumerate()
        .map(|(index, line)| {
            if index == 0 {
                Line::styled(line, title_style())
            } else if line.starts_with("State: Loading") {
                Line::styled(line, loading_style())
            } else if line.starts_with("State: Error") {
                Line::styled(line, error_style())
            } else {
                Line::styled(line, body_style())
            }
        })
        .collect::<Vec<_>>();
    let paragraph = Paragraph::new(Text::from(lines))
        .style(body_style())
        .block(Block::bordered().title("VPS"));

    frame.render_widget(paragraph, frame.area());
}

fn redact_host(host: &str) -> String {
    if let Some((prefix, _)) = host.split_once('.') {
        return format!("{prefix}.[REDACTED]");
    }

    "[REDACTED]".to_owned()
}

fn body_style() -> Style {
    Style::default().fg(Color::White).bg(Color::Black)
}

fn title_style() -> Style {
    body_style().fg(Color::Cyan).add_modifier(Modifier::BOLD)
}

fn loading_style() -> Style {
    body_style().fg(Color::Yellow).add_modifier(Modifier::BOLD)
}

fn error_style() -> Style {
    body_style()
        .fg(Color::LightRed)
        .add_modifier(Modifier::BOLD)
}
