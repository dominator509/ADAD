use adad_core::Error;
use ratatui::{
    backend::TestBackend,
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Paragraph},
    Frame, Terminal,
};

use crate::{Balance, WalletAddress};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WalletEvent {
    Render,
    Key(char),
    BalanceLoaded(Balance),
    AddressLoaded(WalletAddress),
    Error(Error),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WalletViewState {
    Empty,
    Loading,
    Ready,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WalletFrameLog {
    pub frames: Vec<String>,
    pub actions: Vec<WalletAction>,
    pub state: WalletViewState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WalletAction {
    Balance,
    Address,
}

pub fn run_headless(events: &[WalletEvent]) -> Result<WalletFrameLog, Error> {
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).map_err(|_| Error::Io)?;
    let mut state = WalletState::default();
    let mut frames = Vec::new();

    for event in events {
        match event {
            WalletEvent::Render => {}
            WalletEvent::Key(key) => state.handle_key(*key),
            WalletEvent::BalanceLoaded(balance) => state.set_balance(balance),
            WalletEvent::AddressLoaded(address) => state.set_address(address),
            WalletEvent::Error(error) => state.set_error(error),
        }

        let snapshot = state.snapshot();
        terminal
            .draw(|frame| render_wallet(frame, &state))
            .map_err(|_| Error::Io)?;
        frames.push(snapshot);
    }

    Ok(WalletFrameLog {
        frames,
        actions: state.actions,
        state: state.view_state,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WalletState {
    actions: Vec<WalletAction>,
    view_state: WalletViewState,
    balance: Option<Balance>,
    address: Option<String>,
    error_message: Option<String>,
}

impl Default for WalletState {
    fn default() -> Self {
        Self {
            actions: Vec::new(),
            view_state: WalletViewState::Empty,
            balance: None,
            address: None,
            error_message: None,
        }
    }
}

impl WalletState {
    fn handle_key(&mut self, key: char) {
        let action = match key {
            'b' | 'B' => WalletAction::Balance,
            'a' | 'A' => WalletAction::Address,
            _ => return,
        };

        self.actions.push(action);
        self.error_message = None;
        self.view_state = WalletViewState::Loading;
    }

    fn set_balance(&mut self, balance: &Balance) {
        self.balance = Some(balance.clone());
        self.error_message = None;
        self.view_state = WalletViewState::Ready;
    }

    fn set_address(&mut self, address: &WalletAddress) {
        self.address = Some(address.address.clone());
        self.error_message = None;
        self.view_state = WalletViewState::Ready;
    }

    fn set_error(&mut self, error: &Error) {
        self.error_message = Some(error.user_message());
        self.view_state = WalletViewState::Error;
    }

    fn lines(&self) -> Vec<String> {
        let mut lines = vec!["ADAD Wallet".to_owned(), self.state_line()];
        lines.push("Keys: b balance, a address".to_owned());

        if let Some(balance) = &self.balance {
            lines.push(format!("Balance: {}", balance.balance));
            lines.push(format!("Unlocked: {}", balance.unlocked_balance));
        }
        if let Some(address) = &self.address {
            lines.push(format!("Address: {}", redact_address(address)));
        }
        if self.balance.is_none() && self.address.is_none() {
            lines.push("Wallet data: empty".to_owned());
        }

        lines
    }

    fn state_line(&self) -> String {
        match self.view_state {
            WalletViewState::Empty => "State: Empty - choose a wallet action".to_owned(),
            WalletViewState::Loading => "State: Loading - wallet RPC pending".to_owned(),
            WalletViewState::Ready => "State: Ready".to_owned(),
            WalletViewState::Error => format!(
                "State: Error - {}",
                self.error_message
                    .as_deref()
                    .unwrap_or("Wallet operation failed")
            ),
        }
    }

    fn snapshot(&self) -> String {
        self.lines().join("\n")
    }
}

fn render_wallet(frame: &mut Frame<'_>, state: &WalletState) {
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
        .block(Block::bordered().title("Wallet"));

    frame.render_widget(paragraph, frame.area());
}

fn redact_address(address: &str) -> String {
    if address.len() <= 8 {
        return "[REDACTED]".to_owned();
    }

    format!("{}...[REDACTED]", &address[..4])
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
