use adad_core::Error;

use crate::client::{ChatMessage, OpenAiCompatClient};
use crate::{ExecutionRegistry, ToolDescriptor};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LoopRole {
    User,
    Assistant,
    ToolResult,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoopMessage {
    pub role: LoopRole,
    pub name: Option<String>,
    pub content: String,
    pub tool_call_id: Option<String>,
}

impl LoopMessage {
    #[must_use]
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: LoopRole::User,
            name: None,
            content: content.into(),
            tool_call_id: None,
        }
    }

    #[must_use]
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: LoopRole::Assistant,
            name: None,
            content: content.into(),
            tool_call_id: None,
        }
    }

    #[must_use]
    pub fn tool_result(name: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: LoopRole::ToolResult,
            name: Some(name.into()),
            content: content.into(),
            tool_call_id: None,
        }
    }

    #[must_use]
    pub fn assistant_tool_call(call: &ToolCall) -> Self {
        Self {
            role: LoopRole::Assistant,
            name: Some(call.name.clone()),
            content: call.input.clone(),
            tool_call_id: call.id.clone(),
        }
    }

    #[must_use]
    pub fn tool_result_with_id(
        id: impl Into<String>,
        name: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            role: LoopRole::ToolResult,
            name: Some(name.into()),
            content: content.into(),
            tool_call_id: Some(id.into()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolCall {
    pub id: Option<String>,
    pub name: String,
    pub input: String,
}

impl ToolCall {
    #[must_use]
    pub fn new(name: impl Into<String>, input: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            input: input.into(),
        }
    }

    #[must_use]
    pub fn with_id(
        id: impl Into<String>,
        name: impl Into<String>,
        input: impl Into<String>,
    ) -> Self {
        Self {
            id: Some(id.into()),
            name: name.into(),
            input: input.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModelTurn {
    ToolCall(ToolCall),
    FinalAnswer(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolRun {
    pub name: String,
    pub input: String,
    pub output: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgentLoopStatus {
    Complete,
    IterationBudgetExhausted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentLoopResult {
    pub status: AgentLoopStatus,
    pub final_answer: Option<String>,
    pub tool_runs: Vec<ToolRun>,
    pub transcript: Vec<LoopMessage>,
}

pub trait AgentModel {
    fn next_turn(&mut self, transcript: &[LoopMessage]) -> Result<ModelTurn, Error>;

    /// Variant used by an interactive client that wants incremental assistant
    /// text. Models that do not support streaming retain the same semantics as
    /// [`AgentModel::next_turn`].
    fn next_turn_with_callback(
        &mut self,
        transcript: &[LoopMessage],
        _on_text: &mut dyn FnMut(&str),
    ) -> Result<ModelTurn, Error> {
        self.next_turn(transcript)
    }
}

/// Adapter from the OpenAI-compatible provider to the agent-loop model seam.
///
/// Adapter from a standard OpenAI-compatible response to the agent-loop model
/// seam. Tool calls are returned as data and remain subject to the registry and
/// executor policy enforced by [`AgentLoop`].
pub struct OpenAiAgentModel {
    client: OpenAiCompatClient,
    tools: Vec<ToolDescriptor>,
}

impl OpenAiAgentModel {
    #[must_use]
    pub fn new(client: OpenAiCompatClient) -> Self {
        Self {
            client,
            tools: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_tools(client: OpenAiCompatClient, registry: &ExecutionRegistry) -> Self {
        Self {
            client,
            tools: registry.tools().to_vec(),
        }
    }
}

impl AgentModel for OpenAiAgentModel {
    fn next_turn(&mut self, transcript: &[LoopMessage]) -> Result<ModelTurn, Error> {
        let messages = transcript_to_chat_messages(transcript);
        let completion = if self.tools.is_empty() {
            self.client.chat(&messages)?
        } else {
            self.client.chat_with_tools(&messages, &self.tools)?
        };

        completion_to_turn(completion)
    }

    fn next_turn_with_callback(
        &mut self,
        transcript: &[LoopMessage],
        on_text: &mut dyn FnMut(&str),
    ) -> Result<ModelTurn, Error> {
        let messages = transcript_to_chat_messages(transcript);
        let completion = if self.tools.is_empty() {
            self.client.chat_stream_with_callback(&messages, on_text)?
        } else {
            self.client
                .chat_stream_with_tools(&messages, &self.tools, on_text)?
        };
        completion_to_turn(completion)
    }
}

fn completion_to_turn(completion: crate::Completion) -> Result<ModelTurn, Error> {
    let mut tool_calls = completion.tool_calls.into_iter();
    if let Some(tool_call) = tool_calls.next() {
        if tool_calls.next().is_some() {
            return Err(Error::Provider);
        }
        if tool_call.id.is_empty() || tool_call.name.is_empty() {
            return Err(Error::Provider);
        }
        return Ok(ModelTurn::ToolCall(ToolCall::with_id(
            tool_call.id,
            tool_call.name,
            tool_call.arguments,
        )));
    }

    Ok(ModelTurn::FinalAnswer(completion.content))
}

pub trait ToolExecutor {
    fn execute(&mut self, tool_name: &str, input: &str) -> Result<String, Error>;
}

fn transcript_to_chat_messages(transcript: &[LoopMessage]) -> Vec<ChatMessage> {
    transcript
        .iter()
        .map(|message| match &message.role {
            LoopRole::User => ChatMessage::new("user", &message.content),
            LoopRole::Assistant => message.tool_call_id.as_ref().map_or_else(
                || ChatMessage::new("assistant", &message.content),
                |id| {
                    ChatMessage::assistant_tool_call(
                        id,
                        message.name.as_deref().unwrap_or_default(),
                        &message.content,
                    )
                },
            ),
            LoopRole::ToolResult => message.tool_call_id.as_ref().map_or_else(
                || ChatMessage::new("tool", &message.content),
                |id| ChatMessage::tool_result(id, &message.content),
            ),
        })
        .collect()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentLoop {
    registry: ExecutionRegistry,
    iteration_budget: usize,
}

impl AgentLoop {
    #[must_use]
    pub fn new(registry: ExecutionRegistry, iteration_budget: usize) -> Self {
        Self {
            registry,
            iteration_budget,
        }
    }

    pub fn run(
        &self,
        prompt: impl Into<String>,
        model: &mut impl AgentModel,
        executor: &mut impl ToolExecutor,
    ) -> Result<AgentLoopResult, Error> {
        self.run_internal(
            vec![LoopMessage::user(prompt)],
            model,
            executor,
            false,
            &mut |_| {},
        )
    }

    /// Run one prompt while forwarding streamed final-answer text to the
    /// caller. The regular [`AgentLoop::run`] path remains non-streaming for
    /// deterministic service callers.
    pub fn run_with_callback(
        &self,
        prompt: impl Into<String>,
        model: &mut impl AgentModel,
        executor: &mut impl ToolExecutor,
        on_text: &mut impl FnMut(&str),
    ) -> Result<AgentLoopResult, Error> {
        self.run_internal(
            vec![LoopMessage::user(prompt)],
            model,
            executor,
            true,
            on_text,
        )
    }

    /// Continue an existing conversation while forwarding streamed text.
    /// The supplied transcript must already contain the latest user message.
    pub fn run_transcript_with_callback(
        &self,
        transcript: Vec<LoopMessage>,
        model: &mut impl AgentModel,
        executor: &mut impl ToolExecutor,
        on_text: &mut impl FnMut(&str),
    ) -> Result<AgentLoopResult, Error> {
        if transcript.is_empty() {
            return Err(Error::Provider);
        }
        self.run_internal(transcript, model, executor, true, on_text)
    }

    fn run_internal(
        &self,
        mut transcript: Vec<LoopMessage>,
        model: &mut impl AgentModel,
        executor: &mut impl ToolExecutor,
        stream_text: bool,
        on_text: &mut impl FnMut(&str),
    ) -> Result<AgentLoopResult, Error> {
        let mut tool_runs = Vec::new();

        loop {
            let turn = if stream_text {
                model.next_turn_with_callback(&transcript, on_text)?
            } else {
                model.next_turn(&transcript)?
            };
            match turn {
                ModelTurn::FinalAnswer(final_answer) => {
                    return Ok(AgentLoopResult {
                        status: AgentLoopStatus::Complete,
                        final_answer: Some(final_answer),
                        tool_runs,
                        transcript,
                    });
                }
                ModelTurn::ToolCall(call) => {
                    if tool_runs.len() >= self.iteration_budget {
                        return Ok(AgentLoopResult {
                            status: AgentLoopStatus::IterationBudgetExhausted,
                            final_answer: None,
                            tool_runs,
                            transcript,
                        });
                    }
                    if !self.registry.contains(&call.name) {
                        return Err(Error::Provider);
                    }

                    let tool_name = call.name.clone();
                    let tool_input = call.input.clone();
                    let tool_call_id = call.id.clone();
                    if tool_call_id.is_some() {
                        transcript.push(LoopMessage::assistant_tool_call(&call));
                    }
                    let output = executor.execute(&tool_name, &tool_input)?;
                    if let Some(id) = tool_call_id {
                        transcript.push(LoopMessage::tool_result_with_id(
                            id,
                            &tool_name,
                            output.clone(),
                        ));
                    } else {
                        transcript.push(LoopMessage::tool_result(&tool_name, output.clone()));
                    }
                    tool_runs.push(ToolRun {
                        name: tool_name,
                        input: tool_input,
                        output,
                    });
                }
            }
        }
    }
}
