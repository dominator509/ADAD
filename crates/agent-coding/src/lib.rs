pub mod execution;
pub mod mcp;

pub use execution::{ExecutionRegistry, ToolDescriptor, ToolSurface};
pub use mcp::{
    normalize_name_for_mcp, qualify_mcp_tool_name, serve_stdio_echo_server, EchoMcpServer,
    EchoParams,
};
