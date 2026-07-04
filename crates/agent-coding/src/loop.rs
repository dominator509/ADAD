use adad_core::Error;

use crate::ExecutionRegistry;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LoopRole {
    User,
    ToolResult,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoopMessage {
    pub role: LoopRole,
    pub name: Option<String>,
    pub content: String,
}

impl LoopMessage {
    #[must_use]
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: LoopRole::User,
            name: None,
            content: content.into(),
        }
    }

    #[must_use]
    pub fn tool_result(name: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: LoopRole::ToolResult,
            name: Some(name.into()),
            content: content.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolCall {
    pub name: String,
    pub input: String,
}

impl ToolCall {
    #[must_use]
    pub fn new(name: impl Into<String>, input: impl Into<String>) -> Self {
        Self {
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
}

pub trait ToolExecutor {
    fn execute(&mut self, tool_name: &str, input: &str) -> Result<String, Error>;
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
        let mut transcript = vec![LoopMessage::user(prompt)];
        let mut tool_runs = Vec::new();

        loop {
            match model.next_turn(&transcript)? {
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

                    let output = executor.execute(&call.name, &call.input)?;
                    transcript.push(LoopMessage::tool_result(&call.name, output.clone()));
                    tool_runs.push(ToolRun {
                        name: call.name,
                        input: call.input,
                        output,
                    });
                }
            }
        }
    }
}
