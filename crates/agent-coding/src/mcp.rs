use rmcp::{
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ContentBlock, ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
    transport::stdio,
    ErrorData as McpError, ServerHandler, ServiceExt,
};
use serde::Deserialize;

pub fn normalize_name_for_mcp(name: &str) -> String {
    let mut normalized = String::with_capacity(name.len());
    let mut previous_was_separator = false;

    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch);
            previous_was_separator = false;
        } else if !previous_was_separator {
            normalized.push('_');
            previous_was_separator = true;
        }
    }

    normalized
}

pub fn qualify_mcp_tool_name(server_name: &str, tool_name: &str) -> String {
    format!(
        "mcp__{}__{}",
        normalize_name_for_mcp(server_name),
        normalize_name_for_mcp(tool_name)
    )
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EchoParams {
    text: String,
}

pub struct EchoMcpServer {
    tool_router: ToolRouter<Self>,
}

impl EchoMcpServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

impl Default for EchoMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router]
impl EchoMcpServer {
    #[tool(description = "Echo text back to the caller")]
    async fn echo(
        &self,
        Parameters(EchoParams { text }): Parameters<EchoParams>,
    ) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for EchoMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
    }
}

pub async fn serve_stdio_echo_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let service = EchoMcpServer::new().serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{normalize_name_for_mcp, qualify_mcp_tool_name};

    #[test]
    fn normalization_matches_separator_collapse_rules() {
        assert_eq!(normalize_name_for_mcp("github.com"), "github_com");
        assert_eq!(normalize_name_for_mcp("tool name!"), "tool_name_");
        assert_eq!(
            normalize_name_for_mcp("claude.ai Example   Server!!"),
            "claude_ai_Example_Server_"
        );
    }

    #[test]
    fn qualified_names_keep_the_claude_code_shape() {
        assert_eq!(
            qualify_mcp_tool_name("claude.ai Example Server", "weather tool"),
            "mcp__claude_ai_Example_Server__weather_tool"
        );
    }
}
