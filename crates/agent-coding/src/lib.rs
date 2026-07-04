#[path = "loop.rs"]
pub mod agent_loop;
pub mod client;
pub mod execution;
pub mod mcp;
pub mod provider_select;
pub mod tui;

pub use agent_loop::{
    AgentLoop, AgentLoopResult, AgentLoopStatus, AgentModel, LoopMessage, LoopRole, ModelTurn,
    ToolCall, ToolExecutor, ToolRun,
};
pub use client::{
    ChatMessage, Completion, EgressMode, EgressState, LeakguardEgressState, OpenAiCompatClient,
    StaticEgressState,
};
pub use execution::{ExecutionRegistry, ToolDescriptor, ToolSurface};
pub use mcp::{
    normalize_name_for_mcp, qualify_mcp_tool_name, serve_stdio_echo_server, EchoMcpServer,
    EchoParams,
};
pub use provider_select::{
    provider_select, ProviderSelection, ProviderWarning, DEFAULT_LOCAL_BASE_URL,
    DEFAULT_LOCAL_MODEL, DEFAULT_VENICE_BASE_URL,
};
pub use tui::{
    escape_terminal_text, run_agent_chat_headless, run_headless, run_status_headless,
    AgentChatEvent, AgentChatFrameLog, ChatViewState, DaemonHealth, FrameLog, HeadlessEvent,
    StatusEvent, StatusFrameLog, StatusSnapshot, Theme, ThemeKind,
};
