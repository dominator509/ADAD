#[path = "support/mock_inference.rs"]
mod mock_inference;

use adad_core::{EgressSnapshot, Error};
use agent_coding::{
    ChatMessage, EgressMode, LeakguardEgressState, OpenAiCompatClient, StaticEgressState,
};
use mock_inference::MockInferenceServer;

#[test]
fn fallback_request_is_blocked_before_socket_write_when_tunnel_inactive() {
    let server = MockInferenceServer::start();
    let client = OpenAiCompatClient::new(server.base_url("openai"), "sk-test", "model")
        .with_egress_mode(EgressMode::Fallback)
        .with_egress_state(StaticEgressState::inactive());

    let error = client
        .chat(&[ChatMessage::user("blocked")])
        .expect_err("inactive tunnel should block fallback");

    assert_eq!(error, Error::EgressBlocked);
    assert!(
        server.requests().is_empty(),
        "guard must fire before any request reaches the mock"
    );
}

#[test]
fn fallback_request_is_blocked_by_default_until_egress_is_authorized() {
    let server = MockInferenceServer::start();
    let client = OpenAiCompatClient::new(server.base_url("openai"), "sk-test", "model")
        .with_egress_mode(EgressMode::Fallback);

    let error = client
        .chat(&[ChatMessage::user("blocked by default")])
        .expect_err("fallback must default to blocked");

    assert_eq!(error, Error::EgressBlocked);
    assert!(server.requests().is_empty());
}

#[test]
fn local_provider_is_unaffected_by_inactive_fallback_tunnel() {
    let server = MockInferenceServer::start();
    let client = OpenAiCompatClient::new(server.base_url("local"), "", "qwen2.5-coder")
        .with_egress_mode(EgressMode::Local)
        .with_egress_state(StaticEgressState::inactive());

    let completion = client
        .chat(&[ChatMessage::user("local")])
        .expect("local loopback request should not require fallback tunnel");

    assert_eq!(completion.content, "mock completion");
    assert_eq!(server.requests().len(), 1);
}

#[test]
fn fallback_request_uses_leakguard_snapshot_before_socket_write() {
    let server = MockInferenceServer::start();
    let leakguard_state =
        LeakguardEgressState::new(EgressSnapshot::new(true, true, true, false, true));
    let client = OpenAiCompatClient::new(server.base_url("openai"), "sk-test", "model")
        .with_egress_mode(EgressMode::Fallback)
        .with_egress_state(leakguard_state);

    let error = client
        .chat(&[ChatMessage::user("blocked by ipv6 posture")])
        .expect_err("leakguard snapshot should block unsafe fallback");

    assert_eq!(error, Error::EgressBlocked);
    assert!(server.requests().is_empty());
}
