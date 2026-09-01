use std::{
    collections::BTreeMap,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use serde_json::Value;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct RecordedRequest {
    pub method: String,
    pub path: String,
    pub headers: BTreeMap<String, String>,
    pub body: Value,
}

impl RecordedRequest {
    #[allow(dead_code)]
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .get(&name.to_ascii_lowercase())
            .map(String::as_str)
    }
}

pub struct MockInferenceServer {
    addr: SocketAddr,
    requests: Arc<Mutex<Vec<RecordedRequest>>>,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
#[allow(dead_code)]
enum ResponseMode {
    Text,
    ToolCall,
    ToolThenText,
}

impl MockInferenceServer {
    pub fn start() -> Self {
        Self::start_with_mode(ResponseMode::Text)
    }

    #[allow(dead_code)]
    pub fn start_with_tool_call() -> Self {
        Self::start_with_mode(ResponseMode::ToolCall)
    }

    #[allow(dead_code)]
    pub fn start_with_tool_then_text() -> Self {
        Self::start_with_mode(ResponseMode::ToolThenText)
    }

    fn start_with_mode(response_mode: ResponseMode) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock inference server");
        let addr = listener.local_addr().expect("mock listener address");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));

        let thread_requests = Arc::clone(&requests);
        let thread_stop = Arc::clone(&stop);
        let handle = thread::spawn(move || {
            while !thread_stop.load(Ordering::SeqCst) {
                if let Ok((mut stream, _)) = listener.accept() {
                    if let Some(request) = read_request(&mut stream) {
                        let streaming = request.body["stream"].as_bool().unwrap_or(false);
                        let has_tool_result = request_has_tool_result(&request.body);
                        thread_requests
                            .lock()
                            .expect("request log is available")
                            .push(request);
                        let effective_mode =
                            if response_mode == ResponseMode::ToolThenText && has_tool_result {
                                ResponseMode::Text
                            } else {
                                response_mode
                            };
                        write_response(&mut stream, streaming, effective_mode);
                    }
                }
            }
        });

        Self {
            addr,
            requests,
            stop,
            handle: Some(handle),
        }
    }

    pub fn base_url(&self, provider: &str) -> String {
        format!("http://{}/{provider}/v1", self.addr)
    }

    pub fn requests(&self) -> Vec<RecordedRequest> {
        self.requests
            .lock()
            .expect("request log is available")
            .clone()
    }
}

impl Drop for MockInferenceServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        let _ = TcpStream::connect(self.addr);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn read_request(stream: &mut TcpStream) -> Option<RecordedRequest> {
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("set read timeout");
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 1024];

    loop {
        let read = stream.read(&mut chunk).ok()?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);

        if let Some(header_end) = header_end(&buffer) {
            let content_length = content_length(&buffer[..header_end])?;
            let target_len = header_end + 4 + content_length;
            while buffer.len() < target_len {
                let read = stream.read(&mut chunk).ok()?;
                if read == 0 {
                    break;
                }
                buffer.extend_from_slice(&chunk[..read]);
            }
            return parse_request(&buffer[..target_len], header_end);
        }
    }

    None
}

fn header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

fn content_length(header_bytes: &[u8]) -> Option<usize> {
    let header = std::str::from_utf8(header_bytes).ok()?;
    for line in header.lines() {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        if name.eq_ignore_ascii_case("content-length") {
            return value.trim().parse::<usize>().ok();
        }
    }
    None
}

fn parse_request(buffer: &[u8], header_end: usize) -> Option<RecordedRequest> {
    let header = std::str::from_utf8(&buffer[..header_end]).ok()?;
    let body = &buffer[header_end + 4..];
    let mut lines = header.lines();
    let mut request_line = lines.next()?.split_whitespace();
    let method = request_line.next()?.to_owned();
    let path = request_line.next()?.to_owned();
    let mut headers = BTreeMap::new();

    for line in lines {
        let (name, value) = line.split_once(':')?;
        headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_owned());
    }

    Some(RecordedRequest {
        method,
        path,
        headers,
        body: serde_json::from_slice(body).ok()?,
    })
}

fn write_response(stream: &mut TcpStream, streaming: bool, response_mode: ResponseMode) {
    let body = match (response_mode, streaming) {
        (ResponseMode::Text, true) => {
            "data: {\"choices\":[{\"delta\":{\"content\":\"mock \"}}]}\n\
             data: {\"choices\":[{\"delta\":{\"content\":\"stream\"}}]}\n\
             data: [DONE]\n"
                .to_owned()
        }
        (ResponseMode::Text, false) => r#"{"id":"mock","object":"chat.completion","choices":[{"index":0,"message":{"role":"assistant","content":"mock completion"},"finish_reason":"stop"}]}"#.to_owned(),
        (ResponseMode::ToolCall | ResponseMode::ToolThenText, false) => r#"{"id":"mock-tool-call","object":"chat.completion","choices":[{"index":0,"message":{"role":"assistant","content":null,"tool_calls":[{"id":"call-1","type":"function","function":{"name":"echo","arguments":"{\"input\":\"payload\"}"}}]},"finish_reason":"tool_calls"}]}"#.to_owned(),
        (ResponseMode::ToolCall | ResponseMode::ToolThenText, true) => {
            r#"data: {"choices":[{"delta":{"tool_calls":[{"index":0,"id":"call-1","type":"function","function":{"name":"echo","arguments":"{\"input\":\"payload\"}"}}]}}]}
data: [DONE]
"#
                .to_owned()
        }
    };
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream
        .write_all(response.as_bytes())
        .expect("write mock response");
}

fn request_has_tool_result(body: &Value) -> bool {
    body["messages"]
        .as_array()
        .is_some_and(|messages| messages.iter().any(|message| message["role"] == "tool"))
}
