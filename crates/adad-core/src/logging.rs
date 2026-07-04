#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

impl LogLevel {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Component {
    Leakguard,
    Agent,
    Wallet,
    Vps,
    Persona,
    Metafuse,
    Gitspoof,
    Forge,
}

impl Component {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Leakguard => "leakguard",
            Self::Agent => "agent",
            Self::Wallet => "wallet",
            Self::Vps => "vps",
            Self::Persona => "persona",
            Self::Metafuse => "metafuse",
            Self::Gitspoof => "gitspoof",
            Self::Forge => "forge",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LogOutcome {
    Ok,
    Error,
    Blocked,
    Unknown,
}

impl LogOutcome {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Error => "error",
            Self::Blocked => "blocked",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FieldSensitivity {
    Public,
    Sensitive,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LogField {
    key: String,
    value: String,
    sensitivity: FieldSensitivity,
}

impl LogField {
    #[must_use]
    pub fn public(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            sensitivity: FieldSensitivity::Public,
        }
    }

    #[must_use]
    pub fn sensitive(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            sensitivity: FieldSensitivity::Sensitive,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LogEvent {
    ts: String,
    level: LogLevel,
    component: Component,
    event: String,
    outcome: LogOutcome,
    fields: Vec<LogField>,
}

impl LogEvent {
    #[must_use]
    pub fn new(
        ts: impl Into<String>,
        level: LogLevel,
        component: Component,
        event: impl Into<String>,
        outcome: LogOutcome,
    ) -> Self {
        Self {
            ts: ts.into(),
            level,
            component,
            event: event.into(),
            outcome,
            fields: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_field(mut self, field: LogField) -> Self {
        self.fields.push(field);
        self
    }

    #[must_use]
    pub fn render_redacted(&self) -> String {
        let mut pairs = vec![
            format!("ts={}", sanitize_public(&self.ts)),
            format!("level={}", self.level.as_str()),
            format!("component={}", self.component.as_str()),
            format!("event={}", sanitize_public(&self.event)),
            format!("outcome={}", self.outcome.as_str()),
        ];

        for field in &self.fields {
            let value = match field.sensitivity {
                FieldSensitivity::Public => sanitize_public(&field.value),
                FieldSensitivity::Sensitive => "[REDACTED]".to_owned(),
            };
            pairs.push(format!("{}={value}", sanitize_key(&field.key)));
        }

        pairs.join(" ")
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMemoryLogSink {
    lines: Vec<String>,
}

impl InMemoryLogSink {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn emit(&mut self, event: &LogEvent) {
        self.lines.push(event.render_redacted());
    }

    #[must_use]
    pub fn lines(&self) -> &[String] {
        &self.lines
    }
}

pub fn log_event(sink: &mut InMemoryLogSink, event: &LogEvent) {
    sink.emit(event);
}

fn sanitize_key(raw: &str) -> String {
    let key = raw
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();

    if key.is_empty() {
        "field".to_owned()
    } else {
        key
    }
}

fn sanitize_public(raw: &str) -> String {
    raw.chars()
        .flat_map(|ch| match ch {
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            ch if ch.is_control() => "\\x1f".chars().collect::<Vec<_>>(),
            ch if ch.is_whitespace() => "_".chars().collect::<Vec<_>>(),
            ch => vec![ch],
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{log_event, Component, InMemoryLogSink, LogEvent, LogField, LogLevel, LogOutcome};

    #[test]
    fn sensitive_fields_are_redacted_at_emit_boundary() {
        let event = LogEvent::new(
            "tor+123",
            LogLevel::Info,
            Component::Agent,
            "provider_request",
            LogOutcome::Blocked,
        )
        .with_field(LogField::public("provider", "venice"))
        .with_field(LogField::sensitive("api_key", "sk-real-secret"));
        let mut sink = InMemoryLogSink::new();

        log_event(&mut sink, &event);

        assert_eq!(sink.lines().len(), 1);
        assert!(sink.lines()[0].contains("api_key=[REDACTED]"));
        assert!(!sink.lines()[0].contains("sk-real-secret"));
    }
}
