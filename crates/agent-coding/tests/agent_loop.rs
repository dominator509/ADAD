use std::{collections::VecDeque, fs, path::PathBuf};

use adad_core::Error;
use agent_coding::{
    AgentLoop, AgentLoopStatus, AgentModel, ExecutionRegistry, LoopMessage, ModelTurn, ToolCall,
    ToolExecutor,
};

#[test]
fn loop_runs_registered_tool_and_returns_final_answer() {
    let mut registry = ExecutionRegistry::new();
    registry.register_native("echo", "Echo input");
    let agent_loop = AgentLoop::new(registry, 4);
    let mut model = ScriptedModel::new([
        ModelTurn::ToolCall(ToolCall::new("echo", "payload")),
        ModelTurn::FinalAnswer("tool said echo: payload".to_owned()),
    ]);
    let mut executor = EchoExecutor::default();

    let result = agent_loop
        .run("say payload", &mut model, &mut executor)
        .expect("loop should complete");

    assert_eq!(result.status, AgentLoopStatus::Complete);
    assert_eq!(
        result.final_answer.as_deref(),
        Some("tool said echo: payload")
    );
    assert_eq!(result.tool_runs.len(), 1);
    assert_eq!(result.tool_runs[0].name, "echo");
    assert_eq!(result.tool_runs[0].input, "payload");
    assert_eq!(result.tool_runs[0].output, "echo: payload");
    assert_eq!(
        executor.calls,
        vec![("echo".to_owned(), "payload".to_owned())]
    );
    assert_eq!(model.transcripts_seen.len(), 2);
    assert_eq!(model.transcripts_seen[1][1].content, "echo: payload");
}

#[test]
fn loop_respects_iteration_budget() {
    let mut registry = ExecutionRegistry::new();
    registry.register_native("echo", "Echo input");
    let agent_loop = AgentLoop::new(registry, 1);
    let mut model = ScriptedModel::new([
        ModelTurn::ToolCall(ToolCall::new("echo", "first")),
        ModelTurn::ToolCall(ToolCall::new("echo", "second")),
    ]);
    let mut executor = EchoExecutor::default();

    let result = agent_loop
        .run("repeat", &mut model, &mut executor)
        .expect("budget exhaustion is a typed loop outcome");

    assert_eq!(result.status, AgentLoopStatus::IterationBudgetExhausted);
    assert_eq!(result.final_answer, None);
    assert_eq!(result.tool_runs.len(), 1);
    assert_eq!(
        executor.calls,
        vec![("echo".to_owned(), "first".to_owned())]
    );
}

#[test]
fn only_agent_coding_imports_mcp_runtime_crates() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root should resolve");
    let crates_dir = repo_root.join("crates");
    let mut crates_with_rmcp = Vec::new();

    for entry in fs::read_dir(crates_dir).expect("crates directory should exist") {
        let entry = entry.expect("crate directory entry should be readable");
        let cargo_toml = entry.path().join("Cargo.toml");
        if !cargo_toml.exists() {
            continue;
        }
        let manifest = fs::read_to_string(&cargo_toml).expect("crate manifest should read");
        if manifest.contains("rmcp") {
            crates_with_rmcp.push(
                entry
                    .file_name()
                    .to_str()
                    .expect("crate name should be utf-8")
                    .to_owned(),
            );
        }
    }

    assert_eq!(crates_with_rmcp, vec!["agent-coding".to_owned()]);
}

#[derive(Default)]
struct EchoExecutor {
    calls: Vec<(String, String)>,
}

impl ToolExecutor for EchoExecutor {
    fn execute(&mut self, tool_name: &str, input: &str) -> Result<String, Error> {
        self.calls.push((tool_name.to_owned(), input.to_owned()));
        if tool_name == "echo" {
            Ok(format!("echo: {input}"))
        } else {
            Err(Error::Provider)
        }
    }
}

struct ScriptedModel {
    turns: VecDeque<ModelTurn>,
    transcripts_seen: Vec<Vec<LoopMessage>>,
}

impl ScriptedModel {
    fn new(turns: impl IntoIterator<Item = ModelTurn>) -> Self {
        Self {
            turns: turns.into_iter().collect(),
            transcripts_seen: Vec::new(),
        }
    }
}

impl AgentModel for ScriptedModel {
    fn next_turn(&mut self, transcript: &[LoopMessage]) -> Result<ModelTurn, Error> {
        self.transcripts_seen.push(transcript.to_vec());
        self.turns.pop_front().ok_or(Error::Provider)
    }
}
