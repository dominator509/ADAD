use crate::mcp::qualify_mcp_tool_name;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolSurface {
    Native,
    Mcp {
        server_name: String,
        tool_name: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolDescriptor {
    pub qualified_name: String,
    pub description: String,
    pub surface: ToolSurface,
}

impl ToolDescriptor {
    pub fn native(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            qualified_name: name.into(),
            description: description.into(),
            surface: ToolSurface::Native,
        }
    }

    pub fn mcp(
        server_name: impl Into<String>,
        tool_name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let server_name = server_name.into();
        let tool_name = tool_name.into();

        Self {
            qualified_name: qualify_mcp_tool_name(&server_name, &tool_name),
            description: description.into(),
            surface: ToolSurface::Mcp {
                server_name,
                tool_name,
            },
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExecutionRegistry {
    tools: Vec<ToolDescriptor>,
}

impl ExecutionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_native(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> &ToolDescriptor {
        self.tools.push(ToolDescriptor::native(name, description));
        self.tools.last().expect("tool registry just pushed")
    }

    pub fn register_mcp(
        &mut self,
        server_name: impl Into<String>,
        tool_name: impl Into<String>,
        description: impl Into<String>,
    ) -> &ToolDescriptor {
        self.tools
            .push(ToolDescriptor::mcp(server_name, tool_name, description));
        self.tools.last().expect("tool registry just pushed")
    }

    pub fn tools(&self) -> &[ToolDescriptor] {
        &self.tools
    }

    pub fn contains(&self, qualified_name: &str) -> bool {
        self.tools
            .iter()
            .any(|tool| tool.qualified_name == qualified_name)
    }
}

#[cfg(test)]
mod tests {
    use super::{ExecutionRegistry, ToolDescriptor, ToolSurface};

    #[test]
    fn native_tools_keep_their_own_name() {
        let descriptor = ToolDescriptor::native("read_file", "Read a file");

        assert_eq!(descriptor.qualified_name, "read_file");
        assert_eq!(descriptor.surface, ToolSurface::Native);
    }

    #[test]
    fn mcp_tools_use_claude_code_style_qualification() {
        let descriptor = ToolDescriptor::mcp("github.com", "issue search", "Search issues");

        assert_eq!(descriptor.qualified_name, "mcp__github_com__issue_search");
        assert_eq!(
            descriptor.surface,
            ToolSurface::Mcp {
                server_name: "github.com".to_string(),
                tool_name: "issue search".to_string(),
            }
        );
    }

    #[test]
    fn registry_tracks_mixed_native_and_mcp_tools() {
        let mut registry = ExecutionRegistry::new();

        registry.register_native("read_file", "Read a file");
        registry.register_mcp("Demo Server", "echo tool", "Echo a payload");

        assert_eq!(registry.tools().len(), 2);
        assert!(registry.contains("read_file"));
        assert!(registry.contains("mcp__Demo_Server__echo_tool"));
    }
}
