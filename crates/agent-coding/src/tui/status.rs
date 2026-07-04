use adad_core::Error;
use ratatui::{
    backend::TestBackend,
    text::{Line, Text},
    widgets::{Block, Paragraph},
    Frame, Terminal,
};

use crate::{DaemonHealth, HealthReport, DEFAULT_LOCAL_MODEL};

use super::{escape_terminal_text, Theme, ThemeKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StatusEvent {
    Render,
    SelectTheme(ThemeKind),
    SetStatus(StatusSnapshot),
    SetHealth(HealthReport),
    Error(Error),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusSnapshot {
    pub tor: DaemonHealth,
    pub wireguard: DaemonHealth,
    pub llama_server: DaemonHealth,
    pub monero: DaemonHealth,
    pub git: DaemonHealth,
    pub killswitch: DaemonHealth,
    pub dms_hours_remaining: Option<u32>,
    pub vault_lock_minutes_remaining: Option<u32>,
    pub provider: String,
    pub model: String,
}

impl Default for StatusSnapshot {
    fn default() -> Self {
        Self {
            tor: DaemonHealth::Unknown,
            wireguard: DaemonHealth::Unknown,
            llama_server: DaemonHealth::Unknown,
            monero: DaemonHealth::Unknown,
            git: DaemonHealth::Unknown,
            killswitch: DaemonHealth::Unknown,
            dms_hours_remaining: None,
            vault_lock_minutes_remaining: None,
            provider: "local".to_owned(),
            model: DEFAULT_LOCAL_MODEL.to_owned(),
        }
    }
}

impl StatusSnapshot {
    #[must_use]
    pub fn from_health_report(
        report: HealthReport,
        provider: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            tor: report.tor,
            wireguard: report.wireguard,
            llama_server: report.llama_server,
            monero: report.monero,
            git: report.git,
            killswitch: report.killswitch,
            dms_hours_remaining: report.dms_hours_remaining,
            vault_lock_minutes_remaining: None,
            provider: provider.into(),
            model: model.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StatusAlert {
    KillswitchFired,
    TunnelDown,
    DmsNearExpiry,
    VaultLockImminent,
}

impl StatusAlert {
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::KillswitchFired => "Alert: KILLSWITCH FIRED - all egress dropped",
            Self::TunnelDown => "Alert: TUNNEL DOWN - fallback API egress blocked",
            Self::DmsNearExpiry => "Alert: DMS NEAR EXPIRY - access vault or prepare wipe",
            Self::VaultLockImminent => "Alert: VAULT LOCK IMMINENT - save and seal soon",
        }
    }
}

impl StatusSnapshot {
    #[must_use]
    pub fn alerts(&self) -> Vec<StatusAlert> {
        let mut alerts = Vec::new();
        if self.killswitch == DaemonHealth::Down {
            alerts.push(StatusAlert::KillswitchFired);
        }
        if self.wireguard == DaemonHealth::Down {
            alerts.push(StatusAlert::TunnelDown);
        }
        if self.dms_hours_remaining.is_some_and(|hours| hours <= 2) {
            alerts.push(StatusAlert::DmsNearExpiry);
        }
        if self
            .vault_lock_minutes_remaining
            .is_some_and(|minutes| minutes <= 15)
        {
            alerts.push(StatusAlert::VaultLockImminent);
        }
        alerts
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusFrameLog {
    pub theme: ThemeKind,
    pub frames: Vec<String>,
    pub last_status: StatusSnapshot,
}

pub fn run_status_headless(events: &[StatusEvent]) -> Result<StatusFrameLog, Error> {
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).map_err(|_| Error::Io)?;
    let mut state = StatusState::default();
    let mut frames = Vec::new();

    for event in events {
        match event {
            StatusEvent::Render => {}
            StatusEvent::SelectTheme(kind) => state.theme = Theme::select(*kind),
            StatusEvent::SetStatus(status) => {
                state.status = status.clone();
                state.error_message = None;
            }
            StatusEvent::SetHealth(report) => {
                state.status = StatusSnapshot::from_health_report(
                    report.clone(),
                    "local",
                    DEFAULT_LOCAL_MODEL,
                );
                state.error_message = None;
            }
            StatusEvent::Error(error) => state.error_message = Some(error.user_message()),
        }

        let snapshot = state.snapshot();
        terminal
            .draw(|frame| render_status(frame, &state))
            .map_err(|_| Error::Io)?;
        frames.push(snapshot);
    }

    Ok(StatusFrameLog {
        theme: state.theme.kind,
        frames,
        last_status: state.status,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StatusState {
    theme: Theme,
    status: StatusSnapshot,
    error_message: Option<String>,
}

impl Default for StatusState {
    fn default() -> Self {
        Self {
            theme: Theme::high_contrast(),
            status: StatusSnapshot::default(),
            error_message: None,
        }
    }
}

impl StatusState {
    fn lines(&self) -> Vec<String> {
        let mut lines = vec![
            "ADAD Status".to_owned(),
            format!("Tor: {}", self.status.tor.label()),
            format!("WireGuard: {}", self.status.wireguard.label()),
            format!("llama-server: {}", self.status.llama_server.label()),
            format!("Monero: {}", self.status.monero.label()),
            format!("Git: {}", self.status.git.label()),
            format!("Killswitch: {}", self.status.killswitch.label()),
            format!(
                "DMS: {}",
                self.status
                    .dms_hours_remaining
                    .map(|hours| format!("{hours}h remaining"))
                    .unwrap_or_else(|| "unknown".to_owned())
            ),
            format!("Provider: {}", escape_terminal_text(&self.status.provider)),
            format!("Model: {}", escape_terminal_text(&self.status.model)),
        ];

        for alert in self.status.alerts() {
            lines.push(alert.label().to_owned());
        }

        if let Some(error) = &self.error_message {
            lines.push(format!("Alert: ERROR - {error}"));
        }

        lines
    }

    fn snapshot(&self) -> String {
        self.lines().join("\n")
    }
}

fn render_status(frame: &mut Frame<'_>, state: &StatusState) {
    let lines = state
        .lines()
        .into_iter()
        .enumerate()
        .map(|(index, line)| {
            if index == 0 {
                Line::styled(line, state.theme.title_style())
            } else if line.starts_with("Alert: ") {
                Line::styled(line, state.theme.error_style())
            } else {
                Line::styled(line, state.theme.body_style())
            }
        })
        .collect::<Vec<_>>();
    let paragraph = Paragraph::new(Text::from(lines))
        .style(state.theme.body_style())
        .block(Block::bordered().title("Status"));

    frame.render_widget(paragraph, frame.area());
}
