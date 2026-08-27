use std::{fmt, sync::Arc, time::Duration};

use adad_core::{EgressSnapshot, Error};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    #[must_use]
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: content.into(),
        }
    }

    #[must_use]
    pub fn user(content: impl Into<String>) -> Self {
        Self::new("user", content)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Completion {
    pub content: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EgressMode {
    Local,
    Fallback,
}

impl EgressMode {
    fn infer(base_url: &str) -> Self {
        if base_url.starts_with("http://127.0.0.1") || base_url.starts_with("http://localhost") {
            Self::Local
        } else {
            Self::Fallback
        }
    }
}

pub trait EgressState: Send + Sync {
    fn fallback_tunnel_active(&self) -> bool;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StaticEgressState {
    fallback_tunnel_active: bool,
}

impl StaticEgressState {
    #[must_use]
    pub fn active() -> Self {
        Self {
            fallback_tunnel_active: true,
        }
    }

    #[must_use]
    pub fn inactive() -> Self {
        Self {
            fallback_tunnel_active: false,
        }
    }
}

impl EgressState for StaticEgressState {
    fn fallback_tunnel_active(&self) -> bool {
        self.fallback_tunnel_active
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LeakguardEgressState {
    snapshot: EgressSnapshot,
}

impl LeakguardEgressState {
    #[must_use]
    pub fn new(snapshot: EgressSnapshot) -> Self {
        Self { snapshot }
    }

    #[must_use]
    pub fn snapshot(self) -> EgressSnapshot {
        self.snapshot
    }
}

impl EgressState for LeakguardEgressState {
    fn fallback_tunnel_active(&self) -> bool {
        self.snapshot.leak_free_fallback_ready()
    }
}

#[derive(Clone)]
pub struct OpenAiCompatClient {
    base_url: String,
    api_key: String,
    model: String,
    egress_mode: EgressMode,
    egress_state: Arc<dyn EgressState>,
}

impl fmt::Debug for OpenAiCompatClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpenAiCompatClient")
            .field("base_url", &self.base_url)
            .field("api_key", &"[REDACTED]")
            .field("model", &self.model)
            .field("egress_mode", &self.egress_mode)
            .finish_non_exhaustive()
    }
}

impl OpenAiCompatClient {
    #[must_use]
    pub fn new(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        let base_url = base_url.into().trim_end_matches('/').to_owned();
        let egress_mode = EgressMode::infer(&base_url);

        Self {
            base_url,
            api_key: api_key.into(),
            model: model.into(),
            egress_mode,
            // A client cannot prove that WireGuard is active.  Fallback access
            // therefore remains blocked until the supervisor supplies the
            // authoritative leakguard snapshot.
            egress_state: Arc::new(StaticEgressState::inactive()),
        }
    }

    #[must_use]
    pub fn with_egress_mode(mut self, egress_mode: EgressMode) -> Self {
        self.egress_mode = egress_mode;
        self
    }

    #[must_use]
    pub fn with_egress_state(mut self, egress_state: impl EgressState + 'static) -> Self {
        self.egress_state = Arc::new(egress_state);
        self
    }

    pub fn chat(&self, messages: &[ChatMessage]) -> Result<Completion, Error> {
        self.chat_with_stream_flag(messages, false)
    }

    pub fn chat_stream(&self, messages: &[ChatMessage]) -> Result<Completion, Error> {
        self.chat_with_stream_flag(messages, true)
    }

    fn chat_with_stream_flag(
        &self,
        messages: &[ChatMessage],
        stream: bool,
    ) -> Result<Completion, Error> {
        let endpoint = Endpoint::parse(&self.base_url)?;
        self.ensure_egress_allowed()?;
        let body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "stream": stream,
        })
        .to_string();
        let response_body = self.post_json(&endpoint, &body)?;

        if stream {
            parse_stream_completion(&response_body)
        } else {
            parse_completion(&response_body)
        }
    }

    fn ensure_egress_allowed(&self) -> Result<(), Error> {
        if self.egress_mode == EgressMode::Fallback && !self.egress_state.fallback_tunnel_active() {
            return Err(Error::EgressBlocked);
        }

        Ok(())
    }

    fn post_json(&self, endpoint: &Endpoint, body: &str) -> Result<String, Error> {
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(10)))
            // Ambient proxy variables are not an authorized ADAD egress path.
            // Routing is provided by loopback for local inference or by the
            // leakguard/WireGuard boundary for fallbacks.
            .proxy(None)
            .build()
            .new_agent();
        let mut request = agent
            .post(&endpoint.url)
            .header("Content-Type", "application/json");
        if !self.api_key.is_empty() {
            request = request.header("Authorization", &format!("Bearer {}", self.api_key));
        }

        let mut response = request.send(body).map_err(|_| Error::Provider)?;
        response
            .body_mut()
            .read_to_string()
            .map_err(|_| Error::Provider)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Endpoint {
    url: String,
}

impl Endpoint {
    fn parse(base_url: &str) -> Result<Self, Error> {
        let (scheme, rest) = base_url.split_once("://").ok_or(Error::Provider)?;
        if !matches!(scheme, "http" | "https") {
            return Err(Error::Provider);
        }
        let authority = rest.split('/').next().ok_or(Error::Provider)?;
        if authority.is_empty() || authority.contains('@') {
            return Err(Error::Provider);
        }

        let path_prefix = match rest.split_once('/') {
            Some((_, path)) => format!("/{path}"),
            None => String::new(),
        };

        Ok(Self {
            url: format!(
                "{scheme}://{}{}{}",
                authority,
                path_prefix.trim_end_matches('/'),
                "/chat/completions"
            ),
        })
    }
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatAssistantMessage,
}

#[derive(Deserialize)]
struct ChatAssistantMessage {
    content: String,
}

fn parse_completion(body: &str) -> Result<Completion, Error> {
    let response: ChatCompletionResponse =
        serde_json::from_str(body).map_err(|_| Error::Provider)?;
    let first = response.choices.into_iter().next().ok_or(Error::Provider)?;
    Ok(Completion {
        content: first.message.content,
    })
}

#[derive(Deserialize)]
struct ChatCompletionChunk {
    choices: Vec<ChatChunkChoice>,
}

#[derive(Deserialize)]
struct ChatChunkChoice {
    delta: ChatDelta,
}

#[derive(Deserialize)]
struct ChatDelta {
    content: Option<String>,
}

fn parse_stream_completion(body: &str) -> Result<Completion, Error> {
    let mut content = String::new();

    for line in body.lines() {
        let Some(raw_event) = line.strip_prefix("data:") else {
            continue;
        };
        let raw_event = raw_event.trim();
        if raw_event == "[DONE]" {
            break;
        }
        let chunk: ChatCompletionChunk =
            serde_json::from_str(raw_event).map_err(|_| Error::Provider)?;
        for choice in chunk.choices {
            if let Some(delta) = choice.delta.content {
                content.push_str(&delta);
            }
        }
    }

    Ok(Completion { content })
}

#[cfg(test)]
mod tests {
    use super::{parse_completion, parse_stream_completion, ChatMessage, Endpoint};

    #[test]
    fn endpoint_joins_v1_chat_path() {
        let endpoint = Endpoint::parse("http://127.0.0.1:8080/v1").expect("valid endpoint");

        assert_eq!(endpoint.url, "http://127.0.0.1:8080/v1/chat/completions");
    }

    #[test]
    fn endpoint_accepts_https_for_fallback_providers() {
        let endpoint = Endpoint::parse("https://api.example.test/v1").expect("valid endpoint");

        assert_eq!(endpoint.url, "https://api.example.test/v1/chat/completions");
    }

    #[test]
    fn parses_standard_completion() {
        let completion =
            parse_completion(r#"{"choices":[{"message":{"role":"assistant","content":"done"}}]}"#)
                .expect("valid response");

        assert_eq!(completion.content, "done");
    }

    #[test]
    fn parses_stream_completion() {
        let completion = parse_stream_completion(
            "data: {\"choices\":[{\"delta\":{\"content\":\"hel\"}}]}\n\
             data: {\"choices\":[{\"delta\":{\"content\":\"lo\"}}]}\n\
             data: [DONE]\n",
        )
        .expect("valid stream");

        assert_eq!(completion.content, "hello");
    }

    #[test]
    fn chat_message_user_sets_role() {
        assert_eq!(ChatMessage::user("hello").role, "user");
    }
}
