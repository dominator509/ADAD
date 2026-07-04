use adad_core::Error;
use agent_coding::{ChatMessage, OpenAiCompatClient};

#[test]
fn failure_provider_rejects_unsupported_url_without_socket_write() {
    let client = OpenAiCompatClient::new("https://api.example.invalid/v1", "sk-test", "model");

    let error = client
        .chat(&[ChatMessage::user("hello")])
        .expect_err("unsupported scheme should fail before egress");

    assert_eq!(error, Error::Provider);
}
