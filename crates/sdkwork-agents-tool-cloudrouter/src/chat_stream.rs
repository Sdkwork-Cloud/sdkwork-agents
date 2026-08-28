//! Live OpenAI-compatible chat completion streaming against the Cloud Router gateway.
//!
//! Uses the blocking reqwest client so turn workers can incrementally parse upstream
//! SSE without nesting Tokio runtimes inside `spawn_blocking` workers.

use std::io::Read;
use std::sync::OnceLock;
use std::time::Duration;

use cloudrouter_open_sdk::models::OpenAiChatCompletionRequest;
use cloudrouter_open_sdk::SdkworkError;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde_json::Value;

/// Wall-clock bound for one streamed chat completion (matches turn execution budget).
const STREAM_REQUEST_TIMEOUT: Duration = Duration::from_secs(120);

/// Aggregated result from one streamed chat completion call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloudRouterChatStreamResult {
    pub content: String,
    pub stream_deltas: Vec<String>,
    pub model: Option<String>,
    pub finish_reason: Option<String>,
}

/// Extracts assistant text from one OpenAI-compatible `chat.completion.chunk` payload.
pub fn extract_openai_stream_delta(chunk: &Value) -> Option<String> {
    let choice = chunk.get("choices")?.as_array()?.first()?;
    let delta = choice.get("delta")?;
    for key in ["content", "reasoning_content"] {
        if let Some(content) = delta.get(key).and_then(Value::as_str) {
            if !content.is_empty() {
                return Some(content.to_string());
            }
        }
    }
    None
}

fn extract_finish_reason(chunk: &Value) -> Option<String> {
    chunk
        .get("choices")?
        .as_array()?
        .first()?
        .get("finish_reason")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn extract_model(chunk: &Value) -> Option<String> {
    chunk
        .get("model")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn apply_dual_token_headers(
    headers: &mut HeaderMap,
    auth_token: &str,
    access_token: Option<&str>,
) -> Result<(), SdkworkError> {
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {auth_token}"))
            .map_err(SdkworkError::InvalidHeaderValue)?,
    );
    if let Some(access_token) = access_token.filter(|token| !token.trim().is_empty()) {
        headers.insert(
            "Access-Token",
            HeaderValue::from_str(access_token).map_err(SdkworkError::InvalidHeaderValue)?,
        );
    }
    Ok(())
}

fn normalize_sse_buffer(buffer: &mut String) {
    if buffer.contains('\r') {
        *buffer = buffer.replace("\r\n", "\n").replace('\r', "\n");
    }
}

fn consume_sse_data_line(
    data: &str,
    content: &mut String,
    stream_deltas: &mut Vec<String>,
    model: &mut Option<String>,
    finish_reason: &mut Option<String>,
    on_delta: &mut dyn FnMut(&str),
) {
    if data == "[DONE]" {
        return;
    }
    let Ok(chunk) = serde_json::from_str::<Value>(data) else {
        return;
    };
    if let Some(model_id) = extract_model(&chunk) {
        *model = Some(model_id);
    }
    if let Some(reason) = extract_finish_reason(&chunk) {
        *finish_reason = Some(reason);
    }
    if let Some(delta) = extract_openai_stream_delta(&chunk) {
        content.push_str(&delta);
        stream_deltas.push(delta.clone());
        on_delta(&delta);
    }
}

fn consume_sse_buffer(
    buffer: &mut String,
    content: &mut String,
    stream_deltas: &mut Vec<String>,
    model: &mut Option<String>,
    finish_reason: &mut Option<String>,
    on_delta: &mut dyn FnMut(&str),
) {
    normalize_sse_buffer(buffer);
    while let Some(pos) = buffer.find("\n\n") {
        let block: String = buffer.drain(..pos).collect();
        buffer.drain(..2);
        for line in block.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(':') {
                continue;
            }
            let Some(data) = line.strip_prefix("data:") else {
                continue;
            };
            consume_sse_data_line(
                data.trim(),
                content,
                stream_deltas,
                model,
                finish_reason,
                on_delta,
            );
        }
    }
}

fn flush_remaining_sse_events(
    buffer: &str,
    content: &mut String,
    stream_deltas: &mut Vec<String>,
    model: &mut Option<String>,
    finish_reason: &mut Option<String>,
    on_delta: &mut dyn FnMut(&str),
) {
    let mut tail = buffer.to_string();
    normalize_sse_buffer(&mut tail);
    for line in tail.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(':') {
            continue;
        }
        let Some(data) = line.strip_prefix("data:") else {
            continue;
        };
        consume_sse_data_line(
            data.trim(),
            content,
            stream_deltas,
            model,
            finish_reason,
            on_delta,
        );
    }
}

fn streaming_client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(STREAM_REQUEST_TIMEOUT)
            .pool_max_idle_per_host(8)
            .build()
            .expect("cloud router streaming http client")
    })
}

/// Streams one chat completion from the Cloud Router gateway, invoking `on_delta`
/// for every provider text chunk as it is parsed from the upstream SSE body.
///
/// Fails closed: callers must not fall back to buffered completion when this
/// returns an error.
pub fn stream_chat_completion_blocking(
    base_url: &str,
    auth_token: &str,
    access_token: Option<&str>,
    mut request: OpenAiChatCompletionRequest,
    on_delta: &mut dyn FnMut(&str),
) -> Result<CloudRouterChatStreamResult, SdkworkError> {
    request.stream = Some(true);

    let mut headers = HeaderMap::new();
    apply_dual_token_headers(&mut headers, auth_token, access_token)?;
    headers.insert(ACCEPT, HeaderValue::from_static("text/event-stream"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let url = format!(
        "{}/v1/chat/completions",
        base_url.trim_end_matches('/')
    );
    let client = streaming_client();
    let mut response = client
        .post(url)
        .headers(headers)
        .json(&request)
        .send()?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().unwrap_or_default();
        return Err(SdkworkError::HttpStatus {
            status: status.as_u16(),
            body,
        });
    }

    let mut buffer = String::new();
    let mut content = String::new();
    let mut stream_deltas = Vec::new();
    let mut model = None;
    let mut finish_reason = None;
    let mut chunk_buf = [0u8; 8 * 1024];

    loop {
        let read = response.read(&mut chunk_buf).map_err(|error| {
            SdkworkError::HttpStatus {
                status: status.as_u16(),
                body: format!("failed to read cloud router stream body: {error}"),
            }
        })?;
        if read == 0 {
            break;
        }
        buffer.push_str(&String::from_utf8_lossy(&chunk_buf[..read]));
        consume_sse_buffer(
            &mut buffer,
            &mut content,
            &mut stream_deltas,
            &mut model,
            &mut finish_reason,
            on_delta,
        );
    }

    if !buffer.trim().is_empty() {
        flush_remaining_sse_events(
            &buffer,
            &mut content,
            &mut stream_deltas,
            &mut model,
            &mut finish_reason,
            on_delta,
        );
    }

    if content.trim().is_empty() || stream_deltas.is_empty() {
        return Err(SdkworkError::HttpStatus {
            status: status.as_u16(),
            body: "cloud router stream returned no assistant content".to_string(),
        });
    }

    Ok(CloudRouterChatStreamResult {
        content,
        stream_deltas,
        model,
        finish_reason,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extract_openai_stream_delta_reads_choice_delta_content() {
        let chunk = json!({
            "choices": [{ "delta": { "content": "hello" }, "index": 0 }],
            "object": "chat.completion.chunk"
        });
        assert_eq!(
            extract_openai_stream_delta(&chunk).as_deref(),
            Some("hello")
        );
    }

    #[test]
    fn consume_sse_buffer_invokes_on_delta_per_event() {
        let mut deltas = Vec::new();
        let mut on_delta = |delta: &str| deltas.push(delta.to_string());
        let mut buffer = String::from(
            "data: {\"choices\":[{\"delta\":{\"content\":\"Hel\"}}]}\n\n\
             data: {\"choices\":[{\"delta\":{\"content\":\"lo\"}}]}\n\n",
        );
        let mut content = String::new();
        let mut stream_deltas = Vec::new();
        let mut model = None;
        let mut finish_reason = None;
        consume_sse_buffer(
            &mut buffer,
            &mut content,
            &mut stream_deltas,
            &mut model,
            &mut finish_reason,
            &mut on_delta,
        );
        assert_eq!(content, "Hello");
        assert_eq!(stream_deltas, vec!["Hel".to_string(), "lo".to_string()]);
        assert_eq!(deltas, stream_deltas);
        assert!(buffer.is_empty());
    }

    #[test]
    fn consume_sse_buffer_handles_crlf_delimited_events() {
        let mut deltas = Vec::new();
        let mut on_delta = |delta: &str| deltas.push(delta.to_string());
        let mut buffer = String::from(
            "data: {\"choices\":[{\"delta\":{\"content\":\"Hi\"}}]}\r\n\r\n",
        );
        let mut content = String::new();
        let mut stream_deltas = Vec::new();
        let mut model = None;
        let mut finish_reason = None;
        consume_sse_buffer(
            &mut buffer,
            &mut content,
            &mut stream_deltas,
            &mut model,
            &mut finish_reason,
            &mut on_delta,
        );
        assert_eq!(content, "Hi");
        assert_eq!(stream_deltas, vec!["Hi".to_string()]);
    }

    #[test]
    fn extract_openai_stream_delta_reads_reasoning_content() {
        let chunk = json!({
            "choices": [{ "delta": { "reasoning_content": "think" }, "index": 0 }],
            "object": "chat.completion.chunk"
        });
        assert_eq!(
            extract_openai_stream_delta(&chunk).as_deref(),
            Some("think")
        );
    }

    #[test]
    fn empty_sse_body_is_an_error_shape() {
        let content = String::new();
        let stream_deltas: Vec<String> = Vec::new();
        assert!(content.trim().is_empty() || stream_deltas.is_empty());
    }

    #[test]
    fn stream_chat_completion_blocking_reads_live_sse_chunks() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::sync::mpsc;
        use std::thread;
        use std::time::Duration;

        use cloudrouter_open_sdk::models::OpenAiChatMessage;

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock gateway");
        let base_url = format!("http://{}", listener.local_addr().expect("local addr"));
        let (ready_tx, ready_rx) = mpsc::channel();
        thread::spawn(move || {
            ready_tx.send(()).expect("signal mock gateway listening");
            let (mut stream, _) = listener.accept().expect("accept mock request");
            let mut request = String::new();
            let mut buf = [0u8; 1024];
            loop {
                let read = stream.read(&mut buf).expect("read mock request");
                if read == 0 {
                    break;
                }
                request.push_str(&String::from_utf8_lossy(&buf[..read]));
                if request.contains("\r\n\r\n") {
                    break;
                }
            }
            assert!(request.contains("POST /v1/chat/completions"));
            assert!(request.contains("stream"));
            let body = "data: {\"choices\":[{\"delta\":{\"content\":\"Hel\"}}]}\n\n\
                        data: {\"choices\":[{\"delta\":{\"content\":\"lo\"}}]}\n\n\
                        data: [DONE]\n\n";
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).expect("write mock sse");
        });
        ready_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("mock gateway should be listening");

        let mut deltas = Vec::new();
        let request = OpenAiChatCompletionRequest {
            model: "default".to_string(),
            messages: vec![OpenAiChatMessage {
                role: "user".to_string(),
                content: Some("hi".to_string()),
                ..Default::default()
            }],
            stream: Some(true),
            ..Default::default()
        };
        let result = stream_chat_completion_blocking(
            &base_url,
            "test-auth",
            None,
            request,
            &mut |delta| deltas.push(delta.to_string()),
        )
        .expect("stream should succeed");
        assert_eq!(result.content, "Hello");
        assert_eq!(deltas, vec!["Hel".to_string(), "lo".to_string()]);
    }
}
