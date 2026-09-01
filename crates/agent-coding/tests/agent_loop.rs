#[path = "support/mock_inference.rs"]
mod mock_inference;

use std::{collections::VecDeque, fs, path::PathBuf};

use adad_core::Error;
use agent_coding::{
    AgentLoop, AgentLoopStatus, AgentModel, ExecutionRegistry, LoopMessage, LoopRole,
    McpServerConfig, McpToolExecutor, ModelTurn, OpenAiAgentModel, OpenAiCompatClient, ToolCall,
    ToolExecutor, WorkspaceToolExecutor, WORKSPACE_READ_FILE,
};
use mock_inference::MockInferenceServer;

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
fn loop_preserves_provider_tool_call_ids_across_tool_result_messages() {
    let mut registry = ExecutionRegistry::new();
    registry.register_native("echo", "Echo input");
    let mut model = ScriptedModel::new([
        ModelTurn::ToolCall(ToolCall::with_id("call-1", "echo", "payload")),
        ModelTurn::FinalAnswer("done".to_owned()),
    ]);
    let mut executor = EchoExecutor::default();

    let result = AgentLoop::new(registry, 1)
        .run("say payload", &mut model, &mut executor)
        .expect("identified provider tool calls should complete");

    assert_eq!(result.transcript.len(), 3);
    assert_eq!(result.transcript[1].role, LoopRole::Assistant);
    assert_eq!(result.transcript[1].tool_call_id.as_deref(), Some("call-1"));
    assert_eq!(result.transcript[2].role, LoopRole::ToolResult);
    assert_eq!(result.transcript[2].tool_call_id.as_deref(), Some("call-1"));
}

#[test]
fn loop_executes_bounded_workspace_read_tool() {
    let fixture = WorkspaceFixture::new();
    fs::write(fixture.root.join("README.md"), "workspace content").expect("fixture file writes");

    let registry = ExecutionRegistry::with_workspace_tools();
    let mut model = ScriptedModel::new([
        ModelTurn::ToolCall(ToolCall::with_id(
            "call-read",
            WORKSPACE_READ_FILE,
            r#"{"input":"README.md"}"#,
        )),
        ModelTurn::FinalAnswer("read complete".to_owned()),
    ]);
    let mut executor = WorkspaceToolExecutor::new(&fixture.root).expect("root resolves");

    let result = AgentLoop::new(registry, 1)
        .run("read the readme", &mut model, &mut executor)
        .expect("workspace tool should complete");

    assert_eq!(result.final_answer.as_deref(), Some("read complete"));
    assert_eq!(result.tool_runs[0].output, "workspace content");
}

#[test]
fn mcp_stdio_executor_runs_the_shipped_echo_server() {
    let executable = PathBuf::from(env!("CARGO_BIN_EXE_agent-coding"));
    let server = McpServerConfig::stdio(
        "demo",
        executable.to_str().expect("test executable path is utf-8"),
        ["mcp-echo-server"],
    )
    .expect("valid stdio server config");
    let mut registry = ExecutionRegistry::new();
    registry.register_mcp("demo", "echo", "Echo text");
    let mut executor = McpToolExecutor::new(&registry, [server]).expect("valid MCP executor");

    let output = executor
        .execute("mcp__demo__echo", r#"{"text":"stdio works"}"#)
        .expect("stdio MCP call should complete");

    assert_eq!(output, "stdio works");
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
fn openai_provider_adapter_runs_the_agent_loop_to_a_final_answer() {
    let server = MockInferenceServer::start();
    let client = OpenAiCompatClient::new(server.base_url("local"), "", "qwen2.5-coder");
    let mut model = OpenAiAgentModel::new(client);
    let mut executor = EchoExecutor::default();

    let result = AgentLoop::new(ExecutionRegistry::new(), 0)
        .run("say hello", &mut model, &mut executor)
        .expect("provider adapter should return a final answer");

    assert_eq!(result.status, AgentLoopStatus::Complete);
    assert_eq!(result.final_answer.as_deref(), Some("mock completion"));
    let requests = server.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].body["messages"][0]["role"], "user");
    assert_eq!(requests[0].body["messages"][0]["content"], "say hello");
}

#[test]
fn openai_provider_adapter_decodes_registered_function_tool_call() {
    let server = MockInferenceServer::start_with_tool_call();
    let client = OpenAiCompatClient::new(server.base_url("local"), "", "qwen2.5-coder");
    let mut registry = ExecutionRegistry::new();
    registry.register_native("echo", "Echo input");
    let mut model = OpenAiAgentModel::with_tools(client, &registry);

    let turn = model
        .next_turn(&[LoopMessage::user("say payload")])
        .expect("provider tool call should parse");

    assert_eq!(
        turn,
        ModelTurn::ToolCall(ToolCall::with_id(
            "call-1",
            "echo",
            r#"{"input":"payload"}"#,
        ))
    );

    let requests = server.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].body["tool_choice"], "auto");
    assert_eq!(requests[0].body["tools"][0]["type"], "function");
    assert_eq!(requests[0].body["tools"][0]["function"]["name"], "echo");
}

#[test]
fn openai_provider_adapter_streams_final_text_after_a_bounded_tool_call() {
    let server = MockInferenceServer::start_with_tool_then_text();
    let client = OpenAiCompatClient::new(server.base_url("local"), "", "qwen2.5-coder");
    let mut registry = ExecutionRegistry::new();
    registry.register_native("echo", "Echo input");
    let mut model = OpenAiAgentModel::with_tools(client, &registry);
    let mut executor = EchoExecutor::default();
    let mut deltas = Vec::new();
    let mut on_text = |delta: &str| deltas.push(delta.to_owned());

    let result = AgentLoop::new(registry, 2)
        .run_with_callback(
            "use the tool, then answer",
            &mut model,
            &mut executor,
            &mut on_text,
        )
        .expect("streaming tool loop should complete");

    assert_eq!(result.final_answer.as_deref(), Some("mock stream"));
    assert_eq!(deltas, ["mock ", "stream"]);
    assert_eq!(
        executor.calls,
        vec![("echo".to_owned(), "{\"input\":\"payload\"}".to_owned())]
    );

    let requests = server.requests();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].body["stream"], true);
    assert_eq!(requests[0].body["tools"][0]["function"]["name"], "echo");
    assert_eq!(requests[1].body["messages"][2]["role"], "tool");
    assert_eq!(requests[1].body["stream"], true);
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

struct WorkspaceFixture {
    root: PathBuf,
    parent: PathBuf,
}

impl WorkspaceFixture {
    fn new() -> Self {
        let parent = std::env::temp_dir().join(format!(
            "adad-agent-loop-{}-{}",
            std::process::id(),
            unique_suffix()
        ));
        let root = parent.join("workspace");
        fs::create_dir_all(&root).expect("fixture root creates");
        Self { root, parent }
    }
}

impl Drop for WorkspaceFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.parent);
    }
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos()
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
