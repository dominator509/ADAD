#[path = "support/mock_inference.rs"]
mod mock_inference;

use agent_coding::{ChatMessage, OpenAiCompatClient};
use mock_inference::MockInferenceServer;

#[test]
fn regression_stream_chat_sets_stream_true_and_parses_sse() {
    let server = MockInferenceServer::start();
    let client = OpenAiCompatClient::new(server.base_url("local"), "", "qwen2.5-coder");

    let completion = client
        .chat_stream(&[ChatMessage::user("stream please")])
        .expect("streaming mock completion should parse");

    assert_eq!(completion.content, "mock stream");
    let requests = server.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].path, "/local/v1/chat/completions");
    assert_eq!(requests[0].body["stream"], true);
}
