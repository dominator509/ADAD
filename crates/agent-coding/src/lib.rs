#[path = "loop.rs"]
pub mod agent_loop;
pub mod client;
pub mod execution;
pub mod health;
pub mod mcp;
pub mod provider_select;
pub mod tui;

pub use agent_loop::{
    AgentLoop, AgentLoopResult, AgentLoopStatus, AgentModel, LoopMessage, LoopRole, ModelTurn,
    OpenAiAgentModel, ToolCall, ToolExecutor, ToolRun,
};
pub use client::{
    ChatFunctionCall, ChatMessage, ChatToolCall, Completion, CompletionToolCall, EgressMode,
    EgressState, LeakguardEgressState, OpenAiCompatClient, StaticEgressState,
};
pub use execution::{
    ExecutionRegistry, ToolDescriptor, ToolSurface, WorkspaceToolExecutor, WORKSPACE_LIST_DIR,
    WORKSPACE_READ_FILE,
};
pub use health::{check_all, Daemon, DaemonHealth, DaemonProbe, HealthReport, SystemDaemonProbe};
pub use mcp::{
    normalize_name_for_mcp, qualify_mcp_tool_name, serve_stdio_echo_server, EchoMcpServer,
    EchoParams, McpServerConfig, McpToolExecutor, McpTransport,
};
pub use provider_select::{
    provider_select, ProviderSelection, ProviderWarning, DEFAULT_LOCAL_BASE_URL,
    DEFAULT_LOCAL_MODEL, DEFAULT_VENICE_BASE_URL,
};
pub use tui::{
    escape_terminal_text, run_agent_chat, run_agent_chat_headless, run_agent_chat_with_provider,
    run_headless, run_status_headless, run_status_monitor, run_status_monitor_with_provider,
    AgentChatEvent, AgentChatFrameLog, ChatViewState, FrameLog, HeadlessEvent, StatusAlert,
    StatusEvent, StatusFrameLog, StatusSnapshot, Theme, ThemeKind,
};
