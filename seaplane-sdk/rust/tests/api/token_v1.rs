use httpmock::prelude::*;
use seaplane::api::identity::v1::TokenRequest;
use serde_json::json;

use super::MOCK_SERVER;

fn build_req() -> TokenRequest {
    TokenRequest::builder()
        .api_key("abc123")
        .base_url(MOCK_SERVER.base_url())
        .build()
        .unwrap()
}

// POST /token
#[test]
fn access_token() {
    let mock = MOCK_SERVER.mock(|when, then| {
        when.method(POST)
            .path("/v1/token")
            .header("authorization", "Bearer abc123")
            .header("accept", "*/*")
            .header("host", format!("{}:{}", MOCK_SERVER.host(), MOCK_SERVER.port()));
        then.status(201).body("abc.123.def");
    });

    let req = build_req();
    let resp = req.access_token().unwrap();

    // Ensure the endpoint was hit
    mock.assert();

    assert_eq!(resp, "abc.123.def");
}

// Accept: application/json POST /token
#[test]
fn access_token_json() {
    let resp_json =
        json!({"token": "abc.123.def", "tenant": "tnt-abcdef1234567890", "subdomain": "pequod"});
    let mock = MOCK_SERVER.mock(|when, then| {
        when.method(POST)
            .path("/v1/token")
            .header("authorization", "Bearer abc123")
            .header("accept", "application/json")
            .header("host", format!("{}:{}", MOCK_SERVER.host(), MOCK_SERVER.port()));
        then.status(201).json_body(resp_json.clone());
    });

    let req = build_req();
    let resp = req.access_token_json().unwrap();

    // Ensure the endpoint was hit
    mock.assert();

    assert_eq!(resp, serde_json::from_value(resp_json).unwrap());
}
