use adad_core::Error;
use agent_coding::{
    escape_terminal_text, run_agent_chat_headless, run_headless, run_status_headless,
    AgentChatEvent, ChatViewState, DaemonHealth, HeadlessEvent, StatusEvent, StatusSnapshot, Theme,
    ThemeKind,
};
use ratatui::style::{Color, Modifier};

#[test]
fn high_contrast_theme_is_selectable() {
    let theme = Theme::select(ThemeKind::HighContrast);

    assert_eq!(theme.foreground, Color::White);
    assert_eq!(theme.background, Color::Black);
    assert_eq!(theme.accent, Color::Cyan);
    assert!(theme.title_style().add_modifier.contains(Modifier::BOLD));
}

#[test]
fn headless_driver_renders_a_loading_scaffold() {
    let log = run_headless(&[
        HeadlessEvent::SelectTheme(ThemeKind::HighContrast),
        HeadlessEvent::Render,
    ])
    .expect("headless render should succeed");

    assert_eq!(log.theme, ThemeKind::HighContrast);
    assert_eq!(log.frames.len(), 2);
    assert!(log.frames[0].contains("ADAD"));
    assert!(log.frames[0].contains("Theme: high-contrast"));
    assert!(log.frames[0].contains("Loading: initializing TUI scaffold"));
}

#[test]
fn headless_driver_records_keyboard_input_without_mouse_paths() {
    let log = run_headless(&[HeadlessEvent::Key('j'), HeadlessEvent::Key('\n')])
        .expect("headless key events should be accepted");

    assert_eq!(log.keys, vec!['j', '\n']);
}

#[test]
fn agent_chat_renders_provider_model_and_empty_state() {
    let log = run_agent_chat_headless(&[AgentChatEvent::Render]).expect("agent chat should render");

    assert_eq!(log.state, ChatViewState::Empty);
    assert!(log.frames[0].contains("ADAD Agent"));
    assert!(log.frames[0].contains("Provider: local"));
    assert!(log.frames[0].contains("Model: qwen2.5-coder"));
    assert!(log.frames[0].contains("State: Empty"));
}

#[test]
fn agent_chat_scripted_keys_reach_send_and_loading_state() {
    let log = run_agent_chat_headless(&[
        AgentChatEvent::Key('h'),
        AgentChatEvent::Key('i'),
        AgentChatEvent::Key('\n'),
    ])
    .expect("agent chat key script should run");

    assert_eq!(log.sent_prompts, vec!["hi".to_owned()]);
    assert_eq!(log.state, ChatViewState::Loading);
    assert!(log
        .frames
        .last()
        .expect("frame exists")
        .contains("User: hi"));
    assert!(log
        .frames
        .last()
        .expect("frame exists")
        .contains("State: Loading"));
}

#[test]
fn agent_chat_streaming_delta_renders_incrementally() {
    let log = run_agent_chat_headless(&[
        AgentChatEvent::StreamDelta("hel".to_owned()),
        AgentChatEvent::StreamDelta("lo".to_owned()),
        AgentChatEvent::FinishStream,
    ])
    .expect("agent chat stream should render");

    assert_eq!(log.state, ChatViewState::Ready);
    assert!(log.frames[0].contains("Assistant: hel"));
    assert!(log.frames[1].contains("Assistant: hello"));
}

#[test]
fn agent_chat_error_state_uses_redacted_error_message() {
    let log = run_agent_chat_headless(&[AgentChatEvent::Error(Error::Provider)])
        .expect("agent chat error should render");

    assert_eq!(log.state, ChatViewState::Error);
    assert!(log.frames[0].contains("State: Error - AI provider unavailable"));
    assert!(!log.frames[0].contains("sk-"));
    assert!(!log.frames[0].contains(".onion"));
}

#[test]
fn status_monitor_reflects_mocked_daemon_states_including_unknown() {
    let status = StatusSnapshot {
        tor: DaemonHealth::Ready,
        wireguard: DaemonHealth::Down,
        llama_server: DaemonHealth::Ready,
        monero: DaemonHealth::Unknown,
        git: DaemonHealth::Ready,
        killswitch: DaemonHealth::Ready,
        dms_hours_remaining: Some(72),
        vault_lock_minutes_remaining: None,
        provider: "local".to_owned(),
        model: "qwen2.5-coder".to_owned(),
    };

    let log = run_status_headless(&[StatusEvent::SetStatus(status)])
        .expect("status monitor should render");

    assert!(log.frames[0].contains("ADAD Status"));
    assert!(log.frames[0].contains("Tor: ready"));
    assert!(log.frames[0].contains("WireGuard: down"));
    assert!(log.frames[0].contains("Monero: unknown"));
    assert!(log.frames[0].contains("DMS: 72h remaining"));
    assert!(log.frames[0].contains("Provider: local"));
    assert!(log.frames[0].contains("Model: qwen2.5-coder"));
}

#[test]
fn crafted_control_sequences_are_escaped_before_rendering() {
    let raw = "\x1b[2Jmodel says hi\r\nnext";
    let escaped = escape_terminal_text(raw);
    let log = run_agent_chat_headless(&[AgentChatEvent::StreamDelta(raw.to_owned())])
        .expect("escaped stream should render");

    assert_eq!(escaped, "\\x1b[2Jmodel says hi\\r\\nnext");
    assert!(log.frames[0].contains("\\x1b[2Jmodel says hi\\r\\nnext"));
    assert!(!log.frames[0].contains('\x1b'));
}

#[test]
fn status_error_state_has_text_label_not_color_only() {
    let log = run_status_headless(&[StatusEvent::Error(Error::Killswitch)])
        .expect("status error should render");

    assert!(log.frames[0].contains("Alert: ERROR - Network dropped (killswitch)"));
}
