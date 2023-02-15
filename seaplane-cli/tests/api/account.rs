use httpmock::prelude::*;
use once_cell::sync::Lazy;
use seaplane_cli::printer::printer;
use serde_json::json;

// _NOT_ using crate::api::MOCK_SERVER because that one throw away the `Mock` object for the token
// since none of the other APIs care about it. Here, however, we are testing that specific mock so
// we use a totally separate one
static ACCOUNT_MOCK_SERVER: Lazy<MockServer> = Lazy::new(MockServer::start);

#[test]
fn account_token() {
    let mut mock = ACCOUNT_MOCK_SERVER.mock(|when, then| {
        when.method(POST)
            .path("/identity/token")
            .header("authorization", "Bearer abc123");
        then.status(201).body("abc.123.def");
    });

    let res = seaplane_cli::test_main_exec_with_ctx(
        &argv!("account token"),
        crate::api::ctx_from_url(ACCOUNT_MOCK_SERVER.base_url()),
    );
    assert!(res.is_ok(), "{res:?}");
    mock.assert();

    assert_eq!(printer().as_string().trim(), "abc.123.def");

    // Prep for next test to not conflict
    mock.delete();
    printer().clear();
}

#[test]
fn account_token_json() {
    let resp_json =
        json!({"token": "abc.123.def", "tenant": "tnt-abcdef1234567890", "subdomain": "pequod"});
    let mock = ACCOUNT_MOCK_SERVER.mock(|when, then| {
        when.method(POST)
            .path("/identity/token")
            .header("authorization", "Bearer abc123")
            .header("accept", "application/json");
        then.status(201).json_body(resp_json.clone());
    });

    let res = seaplane_cli::test_main_exec_with_ctx(
        &argv!("account token --json"),
        crate::api::ctx_from_url(ACCOUNT_MOCK_SERVER.base_url()),
    );
    assert!(res.is_ok(), "{res:?}");
    mock.assert();
    assert_eq!(
        printer().as_string().trim(),
        r#"{"token":"abc.123.def","tenant":"tnt-abcdef1234567890","subdomain":"pequod"}"#
    );

    printer().clear();
}
