#[path = "support/mock_inference.rs"]
mod mock_inference;

use agent_coding::{ChatMessage, OpenAiCompatClient};
use mock_inference::MockInferenceServer;

#[test]
fn client_sends_openai_compatible_chat_requests_for_each_base_url() {
    let server = MockInferenceServer::start();
    let cases = [
        ("local", "", "qwen2.5-coder"),
        ("openai", "sk-test", "gpt-compatible"),
        ("venice", "vapi-test", "venice-private-model"),
    ];

    for (provider, api_key, model) in cases {
        let client = OpenAiCompatClient::new(server.base_url(provider), api_key, model);
        let completion = client
            .chat(&[ChatMessage::user(format!("hello from {provider}"))])
            .expect("mock completion should parse");

        assert_eq!(completion.content, "mock completion");
    }

    let requests = server.requests();
    assert_eq!(requests.len(), 3);

    for (request, (provider, api_key, model)) in requests.iter().zip(cases) {
        assert_eq!(request.method, "POST");
        assert_eq!(request.path, format!("/{provider}/v1/chat/completions"));
        assert_eq!(request.body["model"], model);
        assert_eq!(request.body["stream"], false);
        assert_eq!(request.body["messages"][0]["role"], "user");
        assert_eq!(
            request.body["messages"][0]["content"],
            format!("hello from {provider}")
        );

        let authorization = request.header("authorization");
        if api_key.is_empty() {
            assert!(authorization.is_none());
        } else {
            assert_eq!(authorization, Some(format!("Bearer {api_key}").as_str()));
        }
    }
}

#[test]
fn streaming_client_delivers_sse_deltas_in_order() {
    let server = MockInferenceServer::start();
    let client = OpenAiCompatClient::new(server.base_url("local"), "", "qwen2.5-coder");
    let mut deltas = Vec::new();

    let completion = client
        .chat_stream_with_callback(&[ChatMessage::user("stream")], |delta| {
            deltas.push(delta.to_owned());
        })
        .expect("mock stream should parse");

    assert_eq!(deltas, ["mock ", "stream"]);
    assert_eq!(completion.content, "mock stream");
}
