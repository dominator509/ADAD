#[path = "support/mock_inference.rs"]
mod mock_inference;

use adad_core::{EgressSnapshot, Error};
use agent_coding::{ChatMessage, EgressMode, LeakguardEgressState, OpenAiCompatClient};
use mock_inference::MockInferenceServer;

#[test]
fn regression_fallback_request_is_blocked_before_socket_write_when_leakguard_is_unsafe() {
    let server = MockInferenceServer::start();
    let unsafe_snapshot = EgressSnapshot::new(true, true, false, true, true);
    let client = OpenAiCompatClient::new(server.base_url("openai"), "sk-test", "model")
        .with_egress_mode(EgressMode::Fallback)
        .with_egress_state(LeakguardEgressState::new(unsafe_snapshot));

    let error = client
        .chat(&[ChatMessage::user("must not leak")])
        .expect_err("unsafe leakguard snapshot should block fallback");

    assert_eq!(error, Error::EgressBlocked);
    assert!(
        server.requests().is_empty(),
        "guard must fire before any request reaches the mock"
    );
}
