use httpmock::prelude::*;
use seaplane_cli::printer::printer;
use serde_json::json;

use super::{test_main, then, when, when_json, MOCK_SERVER};

fn multi_kv_resp() -> serde_json::Value {
    json!({
        "next_key": None::<String>,
        "kvs": [
            {"key": "Zm9v", "value": "YmFy"},
            {"key": "YmF6", "value": "YnV6"}
        ]
    })
}

#[test]
fn metadata_get() {
    let resp = json!({"key":"Zm9v", "value": "YmFy"});

    let mut mock = MOCK_SERVER.mock(|w, t| {
        when_json(w, GET, "/v1/config/base64:Zm9v");
        then(t, &resp);
    });

    let res = test_main(&cli!("metadata get Zm9v --base64"), MOCK_SERVER.base_url());
    assert!(res.is_ok());
    mock.assert_hits(1);
    assert_eq!(printer().as_string().trim(), "YmFy");
    printer().clear();

    let res = test_main(&cli!("metadata get foo"), MOCK_SERVER.base_url());
    assert!(res.is_ok());
    mock.assert_hits(2);
    assert_eq!(printer().as_string().trim(), "YmFy");
    printer().clear();

    let res = test_main(&cli!("metadata get foo --decode"), MOCK_SERVER.base_url());
    assert!(res.is_ok());
    mock.assert_hits(3);
    assert_eq!(printer().as_string().trim(), "bar");
    printer().clear();

    mock.delete();
}

#[test]
fn metadata_put() {
    let resp_json = json!({"status": 200_i32, "title": "Ok"});

    let mut mock = MOCK_SERVER.mock(|w, t| {
        when(w, PUT, "/v1/config/base64:Zm9v")
            .header("content-type", "application/octet-stream")
            .body("YmFy");
        then(t, &resp_json);
    });

    let res = test_main(&cli!("metadata set foo bar"), MOCK_SERVER.base_url());
    assert!(res.is_ok());
    mock.assert_hits(1);
    assert_eq!(printer().as_string().trim(), "Success");
    printer().clear();

    let res = test_main(&cli!("metadata set Zm9v YmFy --base64"), MOCK_SERVER.base_url());
    assert!(res.is_ok());
    mock.assert_hits(2);
    assert_eq!(printer().as_string().trim(), "Success");
    printer().clear();
    mock.delete();
}

#[test]
fn metadata_list_root() {
    let mut mock = MOCK_SERVER.mock(|w, t| {
        when_json(w, GET, "/v1/config/");
        then(t, &multi_kv_resp());
    });

    let res = test_main(&cli!("metadata list"), MOCK_SERVER.base_url());
    assert!(res.is_ok());
    mock.assert_hits(1);
    assert_eq!(
        printer().as_string().trim(),
        "KEY: Zm9v\nVALUE:\nYmFy\n---\nKEY: YmF6\nVALUE:\nYnV6"
    );
    printer().clear();

    let res = test_main(&cli!("metadata list -D"), MOCK_SERVER.base_url());
    assert!(res.is_ok());
    mock.assert_hits(2);
    assert_eq!(printer().as_string().trim(), "KEY: foo\nVALUE:\nbar\n---\nKEY: baz\nVALUE:\nbuz");
    printer().clear();
    mock.delete();
}

#[test]
fn metadata_list_dir() {
    let mut mock = MOCK_SERVER.mock(|w, t| {
        when_json(w, GET, "/v1/config/base64:UGVxdW9kIQ/");
        then(t, &multi_kv_resp());
    });

    let res = test_main(&cli!("metadata list Pequod!"), MOCK_SERVER.base_url());
    assert!(res.is_ok());
    mock.assert_hits(1);
    assert_eq!(
        printer().as_string().trim(),
        "KEY: Zm9v\nVALUE:\nYmFy\n---\nKEY: YmF6\nVALUE:\nYnV6"
    );
    printer().clear();

    let res = test_main(&cli!("metadata list UGVxdW9kIQ --base64 -D"), MOCK_SERVER.base_url());
    assert!(res.is_ok());
    mock.assert_hits(2);
    assert_eq!(printer().as_string().trim(), "KEY: foo\nVALUE:\nbar\n---\nKEY: baz\nVALUE:\nbuz");
    printer().clear();
    mock.delete();
}

#[test]
fn metadata_list_dir_json() {
    let mut mock = MOCK_SERVER.mock(|w, t| {
        when_json(w, GET, "/v1/config/base64:UGVxdW9kIQ/");
        then(t, &multi_kv_resp());
    });

    let res = test_main(&cli!("metadata list --format json Pequod!"), MOCK_SERVER.base_url());
    assert!(res.is_ok());
    mock.assert_hits(1);
    assert_eq!(
        printer().as_string().trim(),
        json!([{"key":"Zm9v","value":"YmFy"},{"key":"YmF6", "value":"YnV6"}]).to_string()
    );

    printer().clear();

    let res = test_main(&cli!("metadata list -D --format json Pequod!"), MOCK_SERVER.base_url());
    assert!(!res.is_ok());
    printer().clear();
    mock.delete();
}

#[test]
fn metadata_list_root_from() {
    let mut mock = MOCK_SERVER.mock(|w, t| {
        when_json(w, GET, "/v1/config/").query_param("from", "base64:UGVxdW9kIQ");
        then(t, &multi_kv_resp());
    });

    let res = test_main(&cli!("metadata list --from Pequod!"), MOCK_SERVER.base_url());
    assert!(res.is_ok());
    mock.assert_hits(1);
    assert_eq!(
        printer().as_string().trim(),
        "KEY: Zm9v\nVALUE:\nYmFy\n---\nKEY: YmF6\nVALUE:\nYnV6"
    );
    printer().clear();

    let res = test_main(&cli!("metadata list -f UGVxdW9kIQ --base64"), MOCK_SERVER.base_url());
    assert!(res.is_ok());
    mock.assert_hits(2);
    assert_eq!(
        printer().as_string().trim(),
        "KEY: Zm9v\nVALUE:\nYmFy\n---\nKEY: YmF6\nVALUE:\nYnV6"
    );
    printer().clear();
    mock.delete();
}

#[test]
fn metadata_list_dir_from() {
    let mut mock = MOCK_SERVER.mock(|w, t| {
        when_json(w, GET, "/v1/config/base64:UXVlZXF1ZWc/")
            .query_param("from", "base64:UGVxdW9kIQ");
        then(t, &multi_kv_resp());
    });

    let res = test_main(&cli!("metadata list Queequeg --from Pequod!"), MOCK_SERVER.base_url());
    assert!(res.is_ok());
    mock.assert_hits(1);
    assert_eq!(
        printer().as_string().trim(),
        "KEY: Zm9v\nVALUE:\nYmFy\n---\nKEY: YmF6\nVALUE:\nYnV6"
    );
    printer().clear();

    let res = test_main(
        &cli!("metadata list UXVlZXF1ZWc -f UGVxdW9kIQ --base64"),
        MOCK_SERVER.base_url(),
    );
    assert!(res.is_ok());
    mock.assert_hits(2);
    assert_eq!(
        printer().as_string().trim(),
        "KEY: Zm9v\nVALUE:\nYmFy\n---\nKEY: YmF6\nVALUE:\nYnV6"
    );
    printer().clear();
    mock.delete();
}

#[test]
fn metadata_delete() {
    let resp_json = json!({"status": 200u32, "title": "Ok"});

    let mock = MOCK_SERVER.mock(|w, t| {
        when_json(w, DELETE, "/v1/config/base64:Zm9v");
        then(t, &resp_json);
    });

    let res = test_main(&cli!("metadata delete foo"), MOCK_SERVER.base_url());
    assert!(res.is_ok());
    mock.assert_hits(1);
    assert_eq!(printer().as_string().trim(), "Removed Zm9v\n\nSuccessfully removed 1 item");
    printer().clear();
}
