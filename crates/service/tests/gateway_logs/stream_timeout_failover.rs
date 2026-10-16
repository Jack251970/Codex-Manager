use super::*;
use codexmanager_core::storage::UsageSnapshotRecord;

const STREAM_TIMEOUT_MS: u64 = 200;
const PRIMARY: &str = "acc_header_timeout_primary";
const SECONDARY: &str = "acc_header_timeout_secondary";
const PLATFORM_KEY: &str = "pk_header_timeout";
const KEY_ID: &str = "gk_header_timeout";

struct HeaderTimeoutUpstream {
    addr: String,
    requests: Receiver<CapturedUpstreamRequest>,
    stop: mpsc::Sender<()>,
    join: Option<thread::JoinHandle<()>>,
}

impl HeaderTimeoutUpstream {
    fn start(healthy_account: Option<&str>) -> Self {
        let listener = bind_test_listener("silent-header upstream");
        let addr = listener.local_addr().expect("upstream address").to_string();
        let server = tiny_http::Server::from_listener(listener, None).expect("mock upstream");
        let healthy_token = healthy_account.map(|id| format!("Bearer access_{id}"));
        let (request_tx, requests) = mpsc::channel();
        let (stop, stop_rx) = mpsc::channel();
        let join = thread::spawn(move || {
            // Retain these requests without sending even response headers. The
            // listener stays available for a retry and the next account.
            let mut silent_requests = Vec::new();
            let deadline = Instant::now() + Duration::from_secs(10);
            while Instant::now() < deadline && stop_rx.try_recv().is_err() {
                let Some(mut request) = server
                    .recv_timeout(Duration::from_millis(20))
                    .expect("receive upstream request")
                else {
                    continue;
                };
                let headers = request
                    .headers()
                    .iter()
                    .map(|header| {
                        (
                            header.field.as_str().to_string().to_ascii_lowercase(),
                            header.value.as_str().to_string(),
                        )
                    })
                    .collect::<HashMap<_, _>>();
                let healthy = healthy_token.as_ref().is_some_and(|token| {
                    headers
                        .get("authorization")
                        .is_some_and(|value| value == token)
                });
                let mut body = Vec::new();
                request
                    .as_reader()
                    .read_to_end(&mut body)
                    .expect("read upstream body");
                request_tx
                    .send(CapturedUpstreamRequest {
                        path: request.url().to_string(),
                        headers,
                        body,
                    })
                    .expect("capture upstream request");
                if healthy {
                    let body = concat!(
                        "data: {\"type\":\"response.output_text.delta\",\"delta\":\"healthy secondary answer\"}\n\n",
                        "data: {\"type\":\"response.completed\",\"response\":{\"id\":\"resp_header_timeout_ok\",\"status\":\"completed\",\"model\":\"gpt-5.3-codex\",\"output\":[{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"healthy secondary answer\"}]}],\"usage\":{\"input_tokens\":3,\"output_tokens\":2,\"total_tokens\":5}}}\n\n",
                        "data: [DONE]\n\n"
                    );
                    request
                        .respond(
                            tiny_http::Response::from_string(body).with_header(
                                tiny_http::Header::from_bytes("Content-Type", "text/event-stream")
                                    .expect("SSE header"),
                            ),
                        )
                        .expect("respond from healthy account");
                } else {
                    silent_requests.push(request);
                }
            }
        });
        Self {
            addr,
            requests,
            stop,
            join: Some(join),
        }
    }
}

impl Drop for HeaderTimeoutUpstream {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Some(join) = self.join.take() {
            join.join().expect("join silent-header upstream");
        }
    }
}

fn seed_accounts(storage: &Storage, account_ids: &[&str], key_id: &str) {
    let now = now_ts();
    seed_model_catalog_models(storage, &["gpt-5.3-codex"]);
    for (sort, id) in account_ids.iter().enumerate() {
        storage
            .insert_account(&Account {
                id: (*id).to_string(),
                label: (*id).to_string(),
                issuer: "https://auth.openai.com".to_string(),
                chatgpt_account_id: Some(format!("chatgpt_{id}")),
                workspace_id: None,
                group_name: None,
                sort: sort as i64,
                status: "active".to_string(),
                created_at: now + sort as i64,
                updated_at: now + sort as i64,
            })
            .expect("insert account");
        storage
            .insert_token(&Token {
                account_id: (*id).to_string(),
                id_token: String::new(),
                access_token: format!("access_{id}"),
                refresh_token: String::new(),
                api_key_access_token: Some(format!("api_access_{id}")),
                last_refresh: now,
            })
            .expect("insert token");
        storage
            .insert_usage_snapshot(&UsageSnapshotRecord {
                account_id: (*id).to_string(),
                used_percent: Some(10.0),
                window_minutes: Some(300),
                resets_at: None,
                secondary_used_percent: None,
                secondary_window_minutes: None,
                secondary_resets_at: None,
                credits_json: None,
                captured_at: now,
            })
            .expect("insert usage snapshot");
    }
    storage
        .insert_api_key(&ApiKey {
            id: key_id.to_string(),
            name: Some(key_id.to_string()),
            model_slug: Some("gpt-5.3-codex".to_string()),
            reasoning_effort: None,
            service_tier: None,
            rotation_strategy: "account_rotation".to_string(),
            aggregate_api_id: None,
            account_plan_filter: None,
            aggregate_api_url: None,
            client_type: "codex".to_string(),
            protocol_type: "openai_compat".to_string(),
            auth_scheme: "authorization_bearer".to_string(),
            upstream_base_url: None,
            static_headers_json: None,
            key_hash: hash_platform_key_for_test(PLATFORM_KEY),
            status: "active".to_string(),
            created_at: now,
            last_used_at: None,
        })
        .expect("insert API key");
}

fn post_stream(addr: &str) -> (u16, String) {
    post_http_raw_with_read_timeout(
        addr,
        "/v1/responses",
        r#"{"model":"gpt-5.3-codex","input":"hello","stream":true}"#,
        &[
            ("Content-Type", "application/json"),
            ("Authorization", &format!("Bearer {PLATFORM_KEY}")),
        ],
        Duration::from_secs(5),
    )
}

fn assert_recorded_attempts_match_requests(
    storage: &Storage,
    trace_id: &str,
    requests: &[CapturedUpstreamRequest],
    max_attempts: usize,
) {
    assert!(!trace_id.is_empty());
    let attempts = wait_for_upstream_attempts(storage, trace_id, requests.len());
    // A send can reach its deadline before the mock HTTP parser presents it.
    // Every observed request must be captured, while unobserved timed-out sends
    // can legitimately add records to the same trace.
    assert!(attempts.len() >= requests.len());
    assert!(attempts.len() <= max_attempts, "{attempts:?}");
    for attempt in &attempts {
        assert_eq!(attempt.trace_id, trace_id);
        assert_eq!(attempt.method, "POST");
    }
    for request in requests {
        let wire_sha256 = format!("{:x}", Sha256::digest(&request.body));
        assert!(
            attempts
                .iter()
                .any(|attempt| attempt.wire_sha256 == wire_sha256),
            "persisted attempts must include the observed HTTP request bytes"
        );
    }
}

#[test]
fn gateway_silent_response_headers_fail_over_when_total_timeout_disabled() {
    let _lock = test_env_guard();
    let dir = new_test_dir("codexmanager-header-timeout-failover");
    let db_path = dir.join("codexmanager.db");
    let _db = EnvGuard::set("CODEXMANAGER_DB_PATH", db_path.to_string_lossy().as_ref());
    let _route = EnvGuard::set("CODEXMANAGER_ROUTE_STRATEGY", "ordered");
    // Ordered routing still permits health-based selection within its leading
    // window. Disable that choice so this test always reaches the silent primary.
    let _health = EnvGuard::set("CODEXMANAGER_ROUTE_HEALTH_P2C_ENABLED", "0");
    let _total = EnvGuard::set("CODEXMANAGER_UPSTREAM_TOTAL_TIMEOUT_MS", "0");
    let _stream = EnvGuard::set(
        "CODEXMANAGER_UPSTREAM_STREAM_TIMEOUT_MS",
        &STREAM_TIMEOUT_MS.to_string(),
    );
    let upstream = HeaderTimeoutUpstream::start(Some(SECONDARY));
    let _base = EnvGuard::set(
        "CODEXMANAGER_UPSTREAM_BASE_URL",
        &format!("http://{}/backend-api/codex", upstream.addr),
    );
    let storage = Storage::open(&db_path).expect("open db");
    seed_accounts(&storage, &[PRIMARY, SECONDARY], KEY_ID);
    let server = codexmanager_service::start_one_shot_server().expect("start gateway");
    let started = Instant::now();
    let (status, body) = post_stream(&server.addr);
    let elapsed = started.elapsed();
    server.join();

    assert_eq!(status, 200, "gateway response: {body}");
    assert!(body.contains("healthy secondary answer"), "{body}");
    assert!(body.contains("response.completed"), "{body}");
    assert!(elapsed < Duration::from_secs(5), "elapsed: {elapsed:?}");
    let requests = upstream.requests.try_iter().collect::<Vec<_>>();
    // The transport can retry with a fresh client, and the candidate layer can
    // retry that account once. Assert bounded failover rather than requiring a
    // particular number of recovery attempts before trying the healthy account.
    assert!(
        (2..=5).contains(&requests.len()),
        "silent-account recovery must stay bounded: {requests:?}"
    );
    let (secondary_request, primary_requests) = requests.split_last().expect("upstream requests");
    assert_eq!(
        secondary_request.headers.get("authorization"),
        Some(&format!("Bearer access_{SECONDARY}"))
    );
    for request in primary_requests {
        assert_eq!(
            request.headers.get("authorization"),
            Some(&format!("Bearer access_{PRIMARY}"))
        );
    }
    for request in &requests {
        assert_eq!(request.path, "/backend-api/codex/responses");
    }
    assert!(elapsed >= Duration::from_millis(STREAM_TIMEOUT_MS * primary_requests.len() as u64));
    let logs = storage
        .list_request_logs(Some(&format!("key:={KEY_ID}")), 10)
        .expect("read final request log");
    assert_eq!(logs.len(), 1, "{logs:?}");
    let log = &logs[0];
    assert_eq!(log.status_code, Some(200));
    assert_eq!(log.initial_account_id.as_deref(), Some(PRIMARY));
    assert_eq!(log.account_id.as_deref(), Some(SECONDARY));
    assert_eq!(
        serde_json::from_str::<Vec<String>>(
            log.attempted_account_ids_json
                .as_deref()
                .expect("attempted accounts")
        )
        .expect("decode attempted accounts"),
        vec![PRIMARY, SECONDARY]
    );
    let trace_id = log.trace_id.as_deref().expect("request trace id");
    assert_recorded_attempts_match_requests(&storage, trace_id, &requests, 5);
}

#[test]
fn gateway_single_silent_account_still_obeys_stream_header_timeout() {
    let _lock = test_env_guard();
    let dir = new_test_dir("codexmanager-single-header-timeout");
    let db_path = dir.join("codexmanager.db");
    let _db = EnvGuard::set("CODEXMANAGER_DB_PATH", db_path.to_string_lossy().as_ref());
    let _route = EnvGuard::set("CODEXMANAGER_ROUTE_STRATEGY", "ordered");
    let _health = EnvGuard::set("CODEXMANAGER_ROUTE_HEALTH_P2C_ENABLED", "0");
    let _total = EnvGuard::set("CODEXMANAGER_UPSTREAM_TOTAL_TIMEOUT_MS", "0");
    let _stream = EnvGuard::set(
        "CODEXMANAGER_UPSTREAM_STREAM_TIMEOUT_MS",
        &STREAM_TIMEOUT_MS.to_string(),
    );
    let upstream = HeaderTimeoutUpstream::start(None);
    let _base = EnvGuard::set(
        "CODEXMANAGER_UPSTREAM_BASE_URL",
        &format!("http://{}/backend-api/codex", upstream.addr),
    );
    let storage = Storage::open(&db_path).expect("open db");
    seed_accounts(
        &storage,
        &["acc_header_timeout_only"],
        "gk_single_header_timeout",
    );
    let server = codexmanager_service::start_one_shot_server().expect("start gateway");
    let started = Instant::now();
    let (status, body) = post_stream(&server.addr);
    let elapsed = started.elapsed();
    server.join();

    assert_eq!(status, 502, "gateway response: {body}");
    assert!(
        body.contains("upstream response headers timed out after 200 ms"),
        "{body}"
    );
    assert!(elapsed < Duration::from_secs(5), "elapsed: {elapsed:?}");
    let requests = upstream.requests.try_iter().collect::<Vec<_>>();
    assert!(
        (1..=2).contains(&requests.len()),
        "single-account transport recovery must stay bounded: {requests:?}"
    );
    assert!(elapsed >= Duration::from_millis(STREAM_TIMEOUT_MS * requests.len() as u64));
    assert!(requests.iter().all(|request| {
        request.headers.get("authorization").map(String::as_str)
            == Some("Bearer access_acc_header_timeout_only")
    }));
    let logs = storage
        .list_request_logs(Some("key:=gk_single_header_timeout"), 10)
        .expect("read timeout request log");
    assert_eq!(logs.len(), 1, "{logs:?}");
    assert_eq!(logs[0].status_code, Some(502));
    let trace_id = logs[0]
        .trace_id
        .as_deref()
        .expect("timeout request trace id");
    assert_recorded_attempts_match_requests(&storage, trace_id, &requests, 2);
}
