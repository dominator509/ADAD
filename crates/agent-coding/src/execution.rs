use std::{
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
};

use adad_core::Error;
use serde::Deserialize;

use crate::{agent_loop::ToolExecutor, mcp::qualify_mcp_tool_name};

pub const WORKSPACE_READ_FILE: &str = "workspace_read_file";
pub const WORKSPACE_LIST_DIR: &str = "workspace_list_dir";
const MAX_WORKSPACE_INPUT_BYTES: usize = 4096;
const MAX_WORKSPACE_OUTPUT_BYTES: usize = 64 * 1024;

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

    /// Register the native tools that are safe to expose from the shipped
    /// local agent.  These tools are deliberately read-only and do not provide
    /// a process, network, credential, or arbitrary filesystem escape hatch.
    pub fn with_workspace_tools() -> Self {
        let mut registry = Self::new();
        registry.register_workspace_tools();
        registry
    }

    pub fn register_workspace_tools(&mut self) {
        if !self.contains(WORKSPACE_READ_FILE) {
            self.register_native(
                WORKSPACE_READ_FILE,
                "Read bounded UTF-8 text from a relative workspace file. Input is {\"input\":\"relative/path\"}.",
            );
        }
        if !self.contains(WORKSPACE_LIST_DIR) {
            self.register_native(
                WORKSPACE_LIST_DIR,
                "List safe direct children of a relative workspace directory. Input is {\"input\":\"relative/path\"}.",
            );
        }
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

#[derive(Clone, Debug)]
pub struct WorkspaceToolExecutor {
    root: PathBuf,
}

impl WorkspaceToolExecutor {
    pub fn new(root: impl AsRef<Path>) -> Result<Self, Error> {
        let root = fs::canonicalize(root).map_err(|_| Error::Io)?;
        if !root.is_dir() {
            return Err(Error::Io);
        }
        Ok(Self { root })
    }

    fn input_path(input: &str) -> Result<String, Error> {
        if input.len() > MAX_WORKSPACE_INPUT_BYTES {
            return Err(Error::Provider);
        }

        let input = input.trim();
        if input.is_empty() {
            return Err(Error::Provider);
        }

        if input.starts_with('{') {
            let request =
                serde_json::from_str::<WorkspaceToolInput>(input).map_err(|_| Error::Provider)?;
            if request.input.trim().is_empty() {
                return Err(Error::Provider);
            }
            Ok(request.input)
        } else {
            Ok(input.to_owned())
        }
    }

    fn resolve(&self, requested: &str) -> Result<PathBuf, Error> {
        let requested_path = Path::new(requested);
        if requested_path.is_absolute()
            || requested_path
                .components()
                .any(|component| matches!(component, Component::ParentDir | Component::Prefix(_)))
        {
            return Err(Error::Provider);
        }

        let candidate = self.root.join(requested_path);
        let resolved = candidate.canonicalize().map_err(|_| Error::Io)?;
        if !resolved.starts_with(&self.root) || is_sensitive_path(&resolved, &self.root) {
            return Err(Error::Provider);
        }
        Ok(resolved)
    }

    fn read_file(&self, path: &Path) -> Result<String, Error> {
        if !path.is_file() {
            return Err(Error::Provider);
        }

        let mut file = fs::File::open(path).map_err(|_| Error::Io)?;
        let mut bytes = Vec::with_capacity(MAX_WORKSPACE_OUTPUT_BYTES.min(8192));
        file.by_ref()
            .take((MAX_WORKSPACE_OUTPUT_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|_| Error::Io)?;
        if bytes.len() > MAX_WORKSPACE_OUTPUT_BYTES {
            return Err(Error::Io);
        }
        String::from_utf8(bytes).map_err(|_| Error::Io)
    }

    fn list_directory(&self, path: &Path) -> Result<String, Error> {
        if !path.is_dir() {
            return Err(Error::Provider);
        }

        let mut entries = Vec::new();
        for entry in fs::read_dir(path).map_err(|_| Error::Io)? {
            let entry = entry.map_err(|_| Error::Io)?;
            let name = entry.file_name();
            let name = name.to_str().ok_or(Error::Io)?.to_owned();
            let entry_path = entry.path();
            let file_type = entry.file_type().map_err(|_| Error::Io)?;

            if is_sensitive_name(&name) {
                continue;
            }
            if file_type.is_symlink() {
                let Ok(resolved) = entry_path.canonicalize() else {
                    continue;
                };
                if !resolved.starts_with(&self.root) || is_sensitive_path(&resolved, &self.root) {
                    continue;
                }
            }

            let kind = if file_type.is_dir() { 'd' } else { 'f' };
            entries.push(format!("{kind} {name}"));
        }

        entries.sort_unstable();
        let output = entries.join("\n");
        if output.len() > MAX_WORKSPACE_OUTPUT_BYTES {
            return Err(Error::Io);
        }
        Ok(output)
    }
}

impl ToolExecutor for WorkspaceToolExecutor {
    fn execute(&mut self, tool_name: &str, input: &str) -> Result<String, Error> {
        let requested = Self::input_path(input)?;
        let path = self.resolve(&requested)?;
        match tool_name {
            WORKSPACE_READ_FILE => self.read_file(&path),
            WORKSPACE_LIST_DIR => self.list_directory(&path),
            _ => Err(Error::Provider),
        }
    }
}

#[derive(Deserialize)]
struct WorkspaceToolInput {
    input: String,
}

fn is_sensitive_path(path: &Path, root: &Path) -> bool {
    path.strip_prefix(root)
        .map(|relative| relative.components().any(|component| {
            matches!(component, Component::Normal(name) if name.to_str().is_some_and(is_sensitive_name))
        }))
        .unwrap_or(true)
}

fn is_sensitive_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        ".git"
            | ".env"
            | ".env.local"
            | ".env.production"
            | "secrets"
            | "credentials"
            | "wallet.keys"
            | "wallet.keys.keys"
            | "wg0.conf"
            | "id_rsa"
            | "id_ed25519"
    ) || lower.ends_with(".pem")
        || lower.ends_with(".key")
        || lower.ends_with(".p12")
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::{
        ExecutionRegistry, ToolDescriptor, ToolSurface, WorkspaceToolExecutor, WORKSPACE_LIST_DIR,
        WORKSPACE_READ_FILE,
    };
    use crate::agent_loop::ToolExecutor;

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

    #[test]
    fn workspace_registry_contains_only_bounded_native_tools() {
        let registry = ExecutionRegistry::with_workspace_tools();

        assert!(registry.contains(WORKSPACE_READ_FILE));
        assert!(registry.contains(WORKSPACE_LIST_DIR));
        assert_eq!(registry.tools().len(), 2);
        assert!(registry
            .tools()
            .iter()
            .all(|tool| tool.surface == ToolSurface::Native));
    }

    #[test]
    fn workspace_executor_reads_files_and_lists_safe_children() {
        let fixture = WorkspaceFixture::new();
        fs::write(fixture.root.join("notes.md"), "safe note").expect("fixture file writes");
        fs::write(fixture.root.join(".env"), "secret").expect("fixture secret writes");
        fs::create_dir(fixture.root.join("src")).expect("fixture directory creates");

        let mut executor = WorkspaceToolExecutor::new(&fixture.root).expect("root resolves");
        assert_eq!(
            executor
                .execute(WORKSPACE_READ_FILE, r#"{"input":"notes.md"}"#)
                .expect("safe file reads"),
            "safe note"
        );
        assert_eq!(
            executor
                .execute(WORKSPACE_LIST_DIR, r#"{"input":"."}"#)
                .expect("safe directory lists"),
            "d src\nf notes.md"
        );
    }

    #[test]
    fn workspace_executor_rejects_escape_and_sensitive_paths() {
        let fixture = WorkspaceFixture::new();
        fs::write(fixture.root.join(".env"), "secret").expect("fixture secret writes");
        fs::write(fixture.parent.join("outside.txt"), "outside").expect("outside fixture writes");
        let mut executor = WorkspaceToolExecutor::new(&fixture.root).expect("root resolves");

        assert_eq!(
            executor.execute(WORKSPACE_READ_FILE, r#"{"input":".env"}"#),
            Err(adad_core::Error::Provider)
        );
        assert_eq!(
            executor.execute(WORKSPACE_READ_FILE, r#"{"input":"../outside.txt"}"#),
            Err(adad_core::Error::Provider)
        );
        assert_eq!(
            executor.execute(WORKSPACE_READ_FILE, r#"{"input":"missing.txt"}"#),
            Err(adad_core::Error::Io)
        );
    }

    struct WorkspaceFixture {
        root: PathBuf,
        parent: PathBuf,
    }

    impl WorkspaceFixture {
        fn new() -> Self {
            let parent = std::env::temp_dir().join(format!(
                "adad-agent-execution-{}-{}",
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
}
