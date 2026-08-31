use std::{
    process::Stdio,
    sync::{Arc, OnceLock},
    thread,
    time::Duration,
};

use adad_core::Error;
use reqwest::Url;
use rmcp::{
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::{
        CallToolRequestParams, CallToolResult, ContentBlock, JsonObject, ServerCapabilities,
        ServerInfo,
    },
    schemars, tool, tool_handler, tool_router,
    transport::{
        stdio, streamable_http_client::StreamableHttpClientTransportConfig,
        StreamableHttpClientTransport, TokioChildProcess,
    },
    ErrorData as McpError, ServerHandler, ServiceExt,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    agent_loop::ToolExecutor,
    client::{EgressState, StaticEgressState},
    execution::{ExecutionRegistry, ToolSurface},
};

const MAX_MCP_ARGUMENT_BYTES: usize = 16 * 1024;
const MAX_MCP_OUTPUT_BYTES: usize = 64 * 1024;
const MCP_CALL_TIMEOUT: Duration = Duration::from_secs(30);
const MCP_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(3);

static RUSTLS_PROVIDER_INSTALLED: OnceLock<bool> = OnceLock::new();

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

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub enum McpTransport {
    Stdio { command: String, args: Vec<String> },
    StreamableHttp { url: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct McpServerConfig {
    pub server_name: String,
    pub transport: McpTransport,
}

impl McpServerConfig {
    pub fn stdio(
        server_name: impl Into<String>,
        command: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, Error> {
        let config = Self {
            server_name: server_name.into(),
            transport: McpTransport::Stdio {
                command: command.into(),
                args: args.into_iter().map(Into::into).collect(),
            },
        };
        config.validate()?;
        Ok(config)
    }

    pub fn streamable_http(
        server_name: impl Into<String>,
        url: impl Into<String>,
    ) -> Result<Self, Error> {
        let config = Self {
            server_name: server_name.into(),
            transport: McpTransport::StreamableHttp { url: url.into() },
        };
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), Error> {
        if self.server_name.trim().is_empty() || self.server_name.contains('\0') {
            return Err(Error::Provider);
        }

        match &self.transport {
            McpTransport::Stdio { command, args } => {
                if command.trim().is_empty()
                    || command.contains('\0')
                    || args.iter().any(|arg| arg.contains('\0'))
                {
                    return Err(Error::Provider);
                }
            }
            McpTransport::StreamableHttp { url } => {
                validate_streamable_http_url(url)?;
            }
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct McpToolExecutor {
    registry: ExecutionRegistry,
    servers: Vec<McpServerConfig>,
    egress_state: Arc<dyn EgressState>,
}

impl McpToolExecutor {
    pub fn new(
        registry: &ExecutionRegistry,
        servers: impl IntoIterator<Item = McpServerConfig>,
    ) -> Result<Self, Error> {
        Self::new_with_egress(registry, servers, StaticEgressState::inactive())
    }

    pub fn new_with_egress(
        registry: &ExecutionRegistry,
        servers: impl IntoIterator<Item = McpServerConfig>,
        egress_state: impl EgressState + 'static,
    ) -> Result<Self, Error> {
        let servers: Vec<_> = servers.into_iter().collect();
        for server in &servers {
            server.validate()?;
        }
        for (index, server) in servers.iter().enumerate() {
            if servers[..index]
                .iter()
                .any(|previous| previous.server_name == server.server_name)
            {
                return Err(Error::Provider);
            }
        }
        Ok(Self {
            registry: registry.clone(),
            servers,
            egress_state: Arc::new(egress_state),
        })
    }

    fn resolve_tool(&self, qualified_name: &str) -> Result<(McpServerConfig, String), Error> {
        let descriptor = self
            .registry
            .tools()
            .iter()
            .find(|tool| tool.qualified_name == qualified_name)
            .ok_or(Error::Provider)?;
        let ToolSurface::Mcp {
            server_name,
            tool_name,
        } = &descriptor.surface
        else {
            return Err(Error::Provider);
        };
        let server = self
            .servers
            .iter()
            .find(|server| server.server_name == *server_name)
            .cloned()
            .ok_or(Error::Provider)?;
        Ok((server, tool_name.clone()))
    }
}

impl ToolExecutor for McpToolExecutor {
    fn execute(&mut self, tool_name: &str, input: &str) -> Result<String, Error> {
        if input.len() > MAX_MCP_ARGUMENT_BYTES {
            return Err(Error::Provider);
        }
        let (server, remote_tool_name) = self.resolve_tool(tool_name)?;
        if requires_fallback_egress(&server.transport)
            && !self.egress_state.fallback_tunnel_active()
        {
            return Err(Error::EgressBlocked);
        }
        let arguments = parse_mcp_arguments(input)?;
        thread::Builder::new()
            .name("adad-mcp-transport".to_owned())
            .spawn(move || run_tool_call(server, remote_tool_name, arguments))
            .map_err(|_| Error::Io)?
            .join()
            .map_err(|_| Error::Provider)?
    }
}

fn parse_mcp_arguments(input: &str) -> Result<JsonObject, Error> {
    let value = serde_json::from_str::<Value>(input).map_err(|_| Error::Provider)?;
    let Value::Object(mut object) = value else {
        return Err(Error::Provider);
    };

    if object.len() == 1 {
        if let Some(Value::Object(arguments)) = object.remove("input") {
            return Ok(arguments);
        }
    }
    Ok(object)
}

fn run_tool_call(
    server: McpServerConfig,
    tool_name: String,
    arguments: JsonObject,
) -> Result<String, Error> {
    match server.transport {
        McpTransport::Stdio { command, args } => {
            run_stdio_tool_call(command, args, tool_name, arguments)
        }
        McpTransport::StreamableHttp { url } => {
            run_streamable_http_tool_call(url, tool_name, arguments)
        }
    }
}

fn run_stdio_tool_call(
    command: String,
    args: Vec<String>,
    tool_name: String,
    arguments: JsonObject,
) -> Result<String, Error> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| Error::Provider)?;
    runtime.block_on(async move {
        let mut command = tokio::process::Command::new(command);
        command.args(args).env_clear();
        let (transport, _) = TokioChildProcess::builder(command)
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| Error::Provider)?;
        let mut client = ().serve(transport).await.map_err(|_| Error::Provider)?;
        let request = CallToolRequestParams::new(tool_name).with_arguments(arguments);
        let result = match tokio::time::timeout(MCP_CALL_TIMEOUT, client.call_tool(request)).await {
            Ok(result) => result.map_err(|_| Error::Provider),
            Err(_) => Err(Error::Provider),
        };
        let output = result.and_then(render_mcp_result);
        let closed = client.close_with_timeout(MCP_SHUTDOWN_TIMEOUT).await;
        if !matches!(closed, Ok(Some(_))) {
            return Err(Error::Provider);
        }
        output
    })
}

fn run_streamable_http_tool_call(
    url: String,
    tool_name: String,
    arguments: JsonObject,
) -> Result<String, Error> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| Error::Provider)?;
    runtime.block_on(async move {
        if !*RUSTLS_PROVIDER_INSTALLED.get_or_init(|| {
            rustls::crypto::ring::default_provider()
                .install_default()
                .is_ok()
        }) {
            return Err(Error::Provider);
        }
        let http_client = reqwest::Client::builder()
            .timeout(MCP_CALL_TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy()
            .build()
            .map_err(|_| Error::Provider)?;
        let config = StreamableHttpClientTransportConfig::with_uri(url);
        let transport = StreamableHttpClientTransport::with_client(http_client, config);
        let mut client = ().serve(transport).await.map_err(|_| Error::Provider)?;
        let request = CallToolRequestParams::new(tool_name).with_arguments(arguments);
        let result = match tokio::time::timeout(MCP_CALL_TIMEOUT, client.call_tool(request)).await {
            Ok(result) => result.map_err(|_| Error::Provider),
            Err(_) => Err(Error::Provider),
        };
        let output = result.and_then(render_mcp_result);
        let closed = client.close_with_timeout(MCP_SHUTDOWN_TIMEOUT).await;
        if !matches!(closed, Ok(Some(_))) {
            return Err(Error::Provider);
        }
        output
    })
}

fn validate_streamable_http_url(raw_url: &str) -> Result<(), Error> {
    let url = Url::parse(raw_url).map_err(|_| Error::Provider)?;
    let is_local_http =
        url.scheme() == "http" && matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "::1"));
    if url.scheme() != "https" && !is_local_http {
        return Err(Error::Provider);
    }
    if !url.username().is_empty() || url.password().is_some() || url.fragment().is_some() {
        return Err(Error::Provider);
    }
    Ok(())
}

fn requires_fallback_egress(transport: &McpTransport) -> bool {
    let McpTransport::StreamableHttp { url } = transport else {
        return false;
    };
    let Ok(url) = Url::parse(url) else {
        return true;
    };
    !(url.scheme() == "http" && matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "::1")))
}

fn render_mcp_result(result: CallToolResult) -> Result<String, Error> {
    if result.is_error == Some(true) {
        return Err(Error::Provider);
    }

    let mut output = String::new();
    for block in result.content {
        if let Some(text) = block.as_text() {
            append_mcp_text(&mut output, &text.text)?;
        } else if let Some(resource) = block.as_resource() {
            append_mcp_text(&mut output, &resource.get_text())?;
        }
    }
    if let Some(structured) = result.structured_content {
        let serialized = serde_json::to_string(&structured).map_err(|_| Error::Provider)?;
        append_mcp_text(&mut output, &serialized)?;
    }
    Ok(output)
}

fn append_mcp_text(output: &mut String, text: &str) -> Result<(), Error> {
    let additional = text.len() + usize::from(!output.is_empty());
    if output.len().saturating_add(additional) > MAX_MCP_OUTPUT_BYTES {
        return Err(Error::Provider);
    }
    if !output.is_empty() {
        output.push('\n');
    }
    output.push_str(text);
    Ok(())
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
    use super::{
        normalize_name_for_mcp, qualify_mcp_tool_name, render_mcp_result, McpServerConfig,
        McpToolExecutor,
    };
    use crate::{agent_loop::ToolExecutor, ExecutionRegistry};
    use rmcp::model::{CallToolResult, ContentBlock};

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

    #[test]
    fn stdio_config_rejects_empty_or_nul_bearing_process_fields() {
        assert!(McpServerConfig::stdio("", "server", Vec::<String>::new()).is_err());
        assert!(McpServerConfig::stdio("server", "", Vec::<String>::new()).is_err());
        assert!(McpServerConfig::stdio("server", "ser\0ver", Vec::<String>::new()).is_err());
    }

    #[test]
    fn streamable_http_config_requires_tls_outside_loopback() {
        assert!(McpServerConfig::streamable_http("demo", "http://example.com/mcp").is_err());
        assert!(McpServerConfig::streamable_http("demo", "https://example.com/mcp").is_ok());
        assert!(McpServerConfig::streamable_http("demo", "http://127.0.0.1:8081/mcp").is_ok());
        assert!(McpServerConfig::streamable_http("demo", "https://user@example.com/mcp").is_err());
        assert!(
            McpServerConfig::streamable_http("demo", "https://example.com/mcp#fragment").is_err()
        );
    }

    #[test]
    fn remote_streamable_http_is_blocked_until_tunnel_is_authorized() {
        let mut registry = ExecutionRegistry::new();
        registry.register_mcp("remote", "echo", "Echo text");
        let server = McpServerConfig::streamable_http("remote", "https://example.com/mcp")
            .expect("valid remote MCP config");
        let mut executor = McpToolExecutor::new(&registry, [server]).expect("valid executor");

        assert_eq!(
            executor.execute("mcp__remote__echo", r#"{"text":"blocked"}"#),
            Err(adad_core::Error::EgressBlocked)
        );
    }

    #[test]
    fn mcp_executor_requires_a_registry_qualified_server_and_tool() {
        let mut registry = ExecutionRegistry::new();
        registry.register_mcp("demo", "echo", "Echo text");
        let server = McpServerConfig::stdio("demo", "echo-server", Vec::<String>::new())
            .expect("valid config");
        let mut executor = McpToolExecutor::new(&registry, [server]).expect("valid executor");

        assert_eq!(
            executor.execute("mcp__other__echo", "{}"),
            Err(adad_core::Error::Provider)
        );
        assert_eq!(
            executor.execute("mcp__demo__echo", "[]"),
            Err(adad_core::Error::Provider)
        );
    }

    #[test]
    fn mcp_result_renderer_preserves_text_and_structured_content() {
        let result = CallToolResult::success(vec![ContentBlock::text("hello")]);
        let rendered = render_mcp_result(result).expect("result renders");
        assert_eq!(rendered, "hello");
    }
}
