use httpmock::{prelude::*, Method, Then, When};
use seaplane::api::compute::v2::{response::*, Flight, Formation, FormationsRequest};

use super::MOCK_SERVER;

fn when(when: When, m: Method, p: &str) -> When {
    when.method(m)
        .path(p)
        .header("authorization", "Bearer abc123")
        .header("accept", "*/*")
        .header("host", format!("{}:{}", MOCK_SERVER.host(), MOCK_SERVER.port()))
}

fn then(then: Then, resp_body: serde_json::Value) -> Then {
    then.status(200)
        .header("content-type", "application/json")
        .json_body(resp_body)
}

fn build_req(incl_id: bool) -> FormationsRequest {
    let mut bdr = FormationsRequest::builder()
        .token("abc123")
        .base_url(MOCK_SERVER.base_url());
    if incl_id {
        bdr = bdr.formation_id("frm-agc6amh7z527vijkv2cutplwaa".parse().unwrap());
    }
    bdr.build().unwrap()
}

fn build_formation() -> Formation {
    Formation::builder()
        .name("stubb")
        .add_flight(
            Flight::builder()
                .name("pequod")
                .image("registry.hub.docker.com/stubb/alpine:latest")
                .build()
                .unwrap(),
        )
        .add_flight(
            Flight::builder()
                .name("flask")
                .image("registry.hub.docker.com/stubb/alpine:latest")
                .build()
                .unwrap(),
        )
        .gateway_flight("pequod")
        .build()
        .unwrap()
}

// GET /formations
#[test]
fn get_all_formations() {
    let resp_json = r#"{
        "objects":[{
          "name": "example-formation",
          "url": "https://example-formation.tenant.on.cplane.cloud",
          "oid": "frm-agc6amh7z527vijkv2cutplwaa",
          "flights": [{
              "name": "example-flight",
              "oid": "flt-agc6amh7z527vijkv2cutplwaa",
              "image": "registry.cplane.cloud/seaplane-demo/nginxdemos/hello:latest",
              "status": "healthy"
          }],
          "gateway-flight": "example-flight"
        }],
        "meta":{
            "total":1,
            "next":null,
            "prev":null
        }
    }"#;
    let resp_t: GetFormationsResponse = serde_json::from_str(resp_json).unwrap();

    let mock = MOCK_SERVER.mock(|w, t| {
        when(w, GET, "/v2beta/formations");
        then(t, serde_json::to_value(resp_t.clone()).unwrap());
    });

    let req = build_req(false);
    let resp = req.get_all().unwrap();

    // Ensure the endpoint was hit
    mock.assert();

    assert_eq!(resp, resp_t);
}

// GET /formations/OID
#[test]
fn get_formation() {
    let mut frm = build_formation();
    frm.oid = Some("frm-agc6amh7z527vijkv2cutplwaa".parse().unwrap());
    frm.url = Some(
        "https://example-formation.tenant.on.cplane.cloud"
            .parse()
            .unwrap(),
    );
    let resp_body = serde_json::to_value(&frm).unwrap();

    let mock = MOCK_SERVER.mock(|w, t| {
        when(w, GET, "/v2beta/formations/frm-agc6amh7z527vijkv2cutplwaa")
            .header("content-type", "application/json");
        then(t, resp_body);
    });

    let req = build_req(true);
    let resp = req.get().unwrap();

    // Ensure the endpoint was hit
    mock.assert();

    assert_eq!(resp, frm);
}

// POST /formations
#[test]
fn create_formation() {
    let mut frm = build_formation();
    frm.oid = Some("frm-agc6amh7z527vijkv2cutplwaa".parse().unwrap());
    frm.url = Some(
        "https://example-formation.tenant.on.cplane.cloud"
            .parse()
            .unwrap(),
    );
    let resp_body = serde_json::to_value(frm).unwrap();

    let mock = MOCK_SERVER.mock(|w, then| {
        when(w, POST, "/v2beta/formations")
            .header("content-type", "application/json")
            .json_body_obj(&build_formation());
        then.status(201)
            .header("content-type", "application/json")
            .header("Location", "https://stubb.tenant.on.cplane.cloud")
            .json_body(resp_body);
    });

    let req = build_req(false);
    assert!(req.create(&build_formation()).is_ok());

    // Ensure the endpoint was hit
    mock.assert();
}

// DELETE /formations/ID
#[test]
fn delete_formation() {
    let mock = MOCK_SERVER.mock(|w, t| {
        when(w, DELETE, "/v2beta/formations/frm-agc6amh7z527vijkv2cutplwaa")
            .header("content-type", "application/json");
        t.status(200);
    });

    let req = build_req(true);
    assert!(req.delete().is_ok());

    // Ensure the endpoint was hit
    mock.assert();
}
