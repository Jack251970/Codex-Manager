use super::{should_skip_codex_v1_alt_for_responses, UpstreamRequestContext};

#[test]
fn api_client_responses_request_skips_codex_v1_alt_retry() {
    let request_ctx = UpstreamRequestContext {
        request_path: "/v1/responses",
        protocol_type: crate::apikey_profile::PROTOCOL_OPENAI_COMPAT,
    };
    assert!(should_skip_codex_v1_alt_for_responses(
        request_ctx,
        "https://chatgpt.com/backend-api/codex/v1/responses"
    ));
}

#[test]
fn native_codex_responses_request_skips_codex_v1_alt_retry() {
    let request_ctx = UpstreamRequestContext {
        request_path: "/v1/responses",
        protocol_type: crate::apikey_profile::PROTOCOL_OPENAI_COMPAT,
    };
    assert!(should_skip_codex_v1_alt_for_responses(
        request_ctx,
        "https://chatgpt.com/backend-api/codex/v1/responses"
    ));
}

#[test]
fn non_responses_request_keeps_alternate_path_available() {
    let request_ctx = UpstreamRequestContext {
        request_path: "/v1/chat/completions",
        protocol_type: crate::apikey_profile::PROTOCOL_OPENAI_COMPAT,
    };
    assert!(!should_skip_codex_v1_alt_for_responses(
        request_ctx,
        "https://chatgpt.com/backend-api/codex/v1/chat/completions"
    ));
}
