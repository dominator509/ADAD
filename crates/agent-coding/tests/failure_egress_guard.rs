#[path = "support/mock_inference.rs"]
mod mock_inference;

use adad_core::{EgressSnapshot, Error};
use agent_coding::{ChatMessage, EgressMode, LeakguardEgressState, OpenAiCompatClient};
use mock_inference::MockInferenceServer;

#[test]
fn failure_unsafe_fallback_returns_egress_blocked_without_partial_request() {
    let server = MockInferenceServer::start();
    let client = OpenAiCompatClient::new(server.base_url("openai"), "sk-test", "model")
        .with_egress_mode(EgressMode::Fallback)
        .with_egress_state(LeakguardEgressState::new(EgressSnapshot::new(
            false, true, true, true, true,
        )));

    let error = client
        .chat(&[ChatMessage::user("blocked")])
        .expect_err("unsafe fallback should fail closed");

    assert_eq!(error, Error::EgressBlocked);
    assert!(server.requests().is_empty());
}
