use std::env;
use std::env::VarError;

use adad_core::{Config, ConfigField, Error, Provider, SecretString};
use agent_coding::{
    check_all, provider_select, run_agent_chat_with_provider, run_status_monitor_with_provider,
    serve_stdio_echo_server, AgentLoop, ExecutionRegistry, OpenAiAgentModel, OpenAiCompatClient,
    ProviderSelection, ProviderWarning, SystemDaemonProbe, WorkspaceToolExecutor,
    DEFAULT_LOCAL_BASE_URL, DEFAULT_LOCAL_MODEL,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("agent-coding: {error}");
        std::process::exit(error.exit_code());
    }
}

fn run() -> Result<(), Error> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "tui".to_owned());
    match command.as_str() {
        "--help" | "-h" => {
            print_help();
            Ok(())
        }
        "--version" | "-V" => {
            println!("agent-coding {}", adad_core::version());
            Ok(())
        }
        "chat" => {
            let prompt = args.collect::<Vec<_>>().join(" ");
            if prompt.trim().is_empty() {
                return Err(Error::Provider);
            }
            let (client, _) = runtime_client()?;
            let registry = ExecutionRegistry::with_workspace_tools();
            let mut model = OpenAiAgentModel::with_tools(client, &registry);
            let workspace = env::current_dir().map_err(|_| Error::Io)?;
            let mut tools = WorkspaceToolExecutor::new(workspace)?;
            let result = AgentLoop::new(registry, 8).run(prompt, &mut model, &mut tools)?;
            let answer = result.final_answer.ok_or(Error::Provider)?;
            print!("{answer}");
            Ok(())
        }
        "status" => {
            let selection = runtime_selection()?;
            let report = check_all(&mut SystemDaemonProbe::new());
            println!("Provider: {}", selection.provider.as_str());
            println!("Model: {}", selection.model);
            println!("Tor: {}", report.tor.label());
            println!("WireGuard: {}", report.wireguard.label());
            println!("llama-server: {}", report.llama_server.label());
            println!("Monero: {}", report.monero.label());
            println!("Git: {}", report.git.label());
            println!("Killswitch: {}", report.killswitch.label());
            Ok(())
        }
        "mcp-echo-server" => run_mcp_echo_server(),
        "status-tui" => {
            let selection = runtime_selection()?;
            run_status_monitor_with_provider(selection.provider.as_str(), selection.model)
        }
        "tui" => {
            let (client, selection) = runtime_client()?;
            run_agent_chat_with_provider(client, selection.provider.as_str(), selection.model)
        }
        _ => Err(Error::Provider),
    }
}

fn runtime_client() -> Result<(OpenAiCompatClient, ProviderSelection), Error> {
    let selection = runtime_selection()?;
    let client = OpenAiCompatClient::new(
        selection.base_url.clone(),
        selection.api_key.clone(),
        selection.model.clone(),
    );
    Ok((client, selection))
}

fn runtime_selection() -> Result<ProviderSelection, Error> {
    let provider = Provider::parse(
        optional_env("ADAD_PROVIDER", ConfigField::Provider)?
            .as_deref()
            .unwrap_or("local"),
    )?;
    let config = Config {
        config_version: 1,
        provider,
        local_base_url: optional_env("ADAD_LOCAL_BASE_URL", ConfigField::LocalBaseUrl)?,
        openai_base_url: optional_env("ADAD_OPENAI_BASE_URL", ConfigField::OpenAiBaseUrl)?,
        venice_base_url: optional_env("ADAD_VENICE_BASE_URL", ConfigField::VeniceBaseUrl)?,
        venice_allow_anonymized: parse_bool_env(
            "ADAD_VENICE_ALLOW_ANONYMIZED",
            ConfigField::VeniceAllowAnonymized,
        )?,
        openai_api_key: optional_env("OPENAI_API_KEY", ConfigField::OpenAiApiKey)?
            .map(SecretString::new),
        venice_api_key: optional_env("VENICE_API_KEY", ConfigField::VeniceApiKey)?
            .map(SecretString::new),
        model: optional_env("ADAD_MODEL", ConfigField::Model)?,
        dms_window_hours: None,
        wg_conf: None,
        monero_rpc_url: None,
    };
    config.validate()?;
    let selection = provider_select(&config)?;
    emit_provider_warnings(&selection);
    Ok(selection)
}

fn optional_env(name: &str, field: ConfigField) -> Result<Option<String>, Error> {
    match env::var(name) {
        Ok(value) => Ok(Some(value)),
        Err(VarError::NotPresent) => Ok(None),
        Err(VarError::NotUnicode(_)) => Err(Error::Config { field }),
    }
}

fn parse_bool_env(name: &str, field: ConfigField) -> Result<bool, Error> {
    match optional_env(name, field)?.as_deref() {
        None | Some("false") => Ok(false),
        Some("true") => Ok(true),
        Some(_) => Err(Error::Config { field }),
    }
}

fn emit_provider_warnings(selection: &ProviderSelection) {
    for warning in &selection.warnings {
        match warning {
            ProviderWarning::VeniceAnonymizedModelEnabled => {
                eprintln!("agent-coding: warning: Venice anonymized model enabled")
            }
        }
    }
}

fn print_help() {
    println!(
        "agent-coding {}\n\nUsage:\n  agent-coding [tui]\n  agent-coding chat <prompt>\n  agent-coding status\n  agent-coding status-tui\n  agent-coding mcp-echo-server\n\nThe interactive TUI and chat command select ADAD_PROVIDER (default: local), read the provider URL/model/API key from the documented runtime environment, and default local inference to {} / {}. Chat exposes bounded read-only workspace tools; it does not execute shell commands or write files. The MCP echo server is a local protocol self-test. The status commands perform conservative host service/interface checks. Non-local fallback URLs remain fail-closed until an authoritative egress service is active; configuring a fallback alone never authorizes network access.",
        adad_core::version(),
        DEFAULT_LOCAL_BASE_URL,
        DEFAULT_LOCAL_MODEL
    );
}

fn run_mcp_echo_server() -> Result<(), Error> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| Error::Provider)?;
    runtime
        .block_on(serve_stdio_echo_server())
        .map_err(|_| Error::Provider)
}
