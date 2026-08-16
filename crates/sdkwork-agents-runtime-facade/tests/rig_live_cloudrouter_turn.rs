//! End-to-end regression test: the rig agent engine (default model backend)
//! must route live model calls through the cloudrouter SDK with the caller's
//! dual tokens.
//!
//! A previous defect left the rig runtime's in-process handler on the
//! bootstrap fail-closed stub after the live backend upgrade, so
//! `AgentEngineSlot::Rig.invoke_model` never reached `RigCloudRouterExecutor`
//! and every live turn failed with a provider error. This test bootstraps the
//! rig engine, invokes a model with caller dual tokens against a loopback
//! cloudrouter mock, and asserts the HTTP request carries both the
//! `Authorization: Bearer` and `Access-Token` headers.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

use sdkwork_agent_kernel::{EnvFileSecretHostProvider, ModelProvider, ModelRequest};
use sdkwork_agents_runtime_facade::{bootstrap_rig_agent_engine, AgentEngineSlot};
use sdkwork_agents_tool_cloudrouter::ENV_CLOUDROUTER_BASE_URL;

fn openai_completion_body() -> &'static str {
    r#"{"id":"chatcmpl-test","object":"chat.completion","created":1750000000,"model":"default","choices":[{"index":0,"message":{"role":"assistant","content":"pong"},"finish_reason":"stop"}],"usage":{"prompt_tokens":1,"completion_tokens":1,"total_tokens":2}}"#
}

/// Serves one OpenAI chat completion and records the request's
/// `Authorization` and `Access-Token` header values.
fn spawn_cloudrouter_mock() -> (String, Arc<Mutex<Option<(String, String)>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let addr = listener.local_addr().expect("local addr");
    let recorded: Arc<Mutex<Option<(String, String)>>> = Arc::new(Mutex::new(None));
    let recorder = Arc::clone(&recorded);
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut reader = BufReader::new(stream.try_clone().expect("clone stream"));
        let mut head = String::new();
        loop {
            let mut line = String::new();
            let read = reader.read_line(&mut line).expect("read line");
            if read == 0 || line == "\r\n" || line == "\n" {
                break;
            }
            head.push_str(&line);
        }
        let mut authorization = None;
        let mut access_token = None;
        for line in head.lines() {
            if let Some((name, value)) = line.split_once(':') {
                let name = name.trim().to_ascii_lowercase();
                let value = value.trim().to_string();
                if name == "authorization" {
                    authorization = Some(value);
                } else if name == "access-token" {
                    access_token = Some(value);
                }
            }
        }
        *recorder.lock().expect("recorder lock") = Some((
            authorization.unwrap_or_default(),
            access_token.unwrap_or_default(),
        ));
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            openai_completion_body().len(),
            openai_completion_body()
        );
        stream.write_all(response.as_bytes()).expect("write");
        stream.flush().expect("flush");
    });
    (format!("http://{addr}"), recorded)
}

/// Restores the cloudrouter base URL environment after the test, so parallel
/// tests never observe the mock address.
struct EnvGuard;

impl EnvGuard {
    fn with_mock_base_url(url: &str) -> Self {
        std::env::set_var(ENV_CLOUDROUTER_BASE_URL, url);
        EnvGuard
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        std::env::remove_var(ENV_CLOUDROUTER_BASE_URL);
    }
}

#[test]
fn rig_engine_live_turn_routes_through_cloudrouter_with_dual_tokens() {
    let (mock_url, recorded) = spawn_cloudrouter_mock();
    let _guard = EnvGuard::with_mock_base_url(&mock_url);

    let host = Arc::new(EnvFileSecretHostProvider::new());
    let slot = bootstrap_rig_agent_engine(None, host).expect("rig engine bootstrap");

    // The default rig configuration upgrades to the live cloud router backend.
    let AgentEngineSlot::Rig(integration) = &slot else {
        panic!("expected the rig engine slot");
    };
    let manifest = integration.model.provider_manifest();
    assert!(
        manifest.capabilities.contains(&"model.chat".to_string()),
        "rig engine must expose the model.chat capability"
    );

    let request = ModelRequest::new("request-1", vec!["user: hello".to_string()])
        .for_caller(
            Some("auth-token-abc".to_string()),
            Some("access-token-xyz".to_string()),
        );

    let response = slot.invoke_model(request).expect("live model invoke");

    assert_eq!(
        response.messages.join("\n"),
        "pong",
        "model call must succeed through the cloudrouter account pool"
    );

    let (authorization, access_token) = recorded
        .lock()
        .expect("recorded lock")
        .clone()
        .expect("headers recorded");
    assert_eq!(
        authorization, "Bearer auth-token-abc",
        "Authorization bearer must carry the caller auth token"
    );
    assert_eq!(
        access_token, "access-token-xyz",
        "Access-Token must carry the caller access token"
    );

    // Cancellation is best-effort on the rig engine: it must acknowledge with
    // a cancelled response instead of surfacing a hard provider error.
    let cancelled = slot
        .cancel_model("request-1")
        .expect("best-effort cancellation must not fail");
    assert_eq!(cancelled.finish_reason.as_deref(), Some("cancelled"));
}
