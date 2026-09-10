//! Integration tests for the client's request/response mechanics, driven against a local mock HTTP
//! server (no external network or real credentials). Deserialization of the shared model types is
//! covered in the `archivindex-truthsocial` crate, so these tests use lightweight JSON bodies.

use archivindex_truthsocial_api::auth::Credentials;
use archivindex_truthsocial_api::client::Client;
use archivindex_truthsocial_api::error::Error;
use archivindex_truthsocial_api::types::SearchKind;
use wiremock::matchers::{body_string_contains, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn login_stores_token_and_sends_password_grant() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .and(body_string_contains("grant_type=password"))
        .and(body_string_contains("username=alice"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "tok123",
            "token_type": "Bearer",
            "scope": "read"
        })))
        .mount(&server)
        .await;

    let mut client = Client::builder()
        .base_url(server.uri())
        .build()
        .expect("build client");
    client
        .login(&Credentials::new("cid", "csec", "alice", "secret"))
        .await
        .expect("login succeeds");

    assert_eq!(client.token(), Some("tok123"));
    assert!(client.is_authenticated());
}

#[tokio::test]
async fn login_failure_is_reported() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .respond_with(ResponseTemplate::new(401).set_body_string(r#"{"error":"invalid_grant"}"#))
        .mount(&server)
        .await;

    let mut client = Client::builder()
        .base_url(server.uri())
        .build()
        .expect("build client");
    let error = client
        .login(&Credentials::new("c", "s", "u", "p"))
        .await
        .expect_err("login should fail");

    assert!(matches!(error, Error::Auth(_)), "got {error:?}");
}

#[tokio::test]
async fn authenticated_request_sends_bearer_and_query() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2/search"))
        .and(header("authorization", "Bearer tok"))
        .and(query_param("q", "trump"))
        .and(query_param("type", "accounts"))
        .and(query_param("resolve", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "accounts": [], "statuses": [], "hashtags": []
        })))
        .mount(&server)
        .await;

    let client = Client::builder()
        .base_url(server.uri())
        .token("tok")
        .build()
        .expect("build client");
    let results = client
        .search("trump", Some(SearchKind::Accounts), true)
        .await
        .expect("search succeeds");

    assert!(results.accounts.is_empty());
    assert!(results.statuses.is_empty());
}

#[tokio::test]
async fn non_success_status_becomes_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/statuses/999"))
        .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
        .mount(&server)
        .await;

    let client = Client::builder()
        .base_url(server.uri())
        .build()
        .expect("build client");
    let error = client.status("999").await.expect_err("should 404");

    match error {
        Error::Api { status, body } => {
            assert_eq!(status.as_u16(), 404);
            assert_eq!(body, "not found");
        }
        other => panic!("expected Api error, got {other:?}"),
    }
}

#[tokio::test]
async fn verify_credentials_requires_a_token() {
    // No server contacted: the guard short-circuits before any request.
    let client = Client::new().expect("build client");
    let error = client
        .verify_credentials()
        .await
        .expect_err("unauthenticated");

    assert!(matches!(error, Error::Unauthenticated));
}

#[tokio::test]
async fn get_json_returns_raw_value() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/custom"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"hello": "world"})),
        )
        .mount(&server)
        .await;

    let client = Client::builder()
        .base_url(server.uri())
        .build()
        .expect("build client");
    let value: serde_json::Value = client
        .get_json("/api/v1/custom", &[])
        .await
        .expect("get_json succeeds");

    assert_eq!(value["hello"], "world");
}

#[tokio::test]
async fn cf_clearance_cookie_is_sent() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/custom"))
        .and(header("cookie", "cf_clearance=xyz"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})))
        .mount(&server)
        .await;

    let client = Client::builder()
        .base_url(server.uri())
        .cf_clearance("xyz")
        .build()
        .expect("build client");
    let value: serde_json::Value = client
        .get_json("/api/v1/custom", &[])
        .await
        .expect("request with cookie succeeds");

    assert_eq!(value["ok"], true);
}

#[tokio::test]
async fn credentials_are_not_sent_to_another_origin() {
    let origin = MockServer::start().await;
    let other = MockServer::start().await;
    let client = Client::builder()
        .base_url(origin.uri())
        .token("secret")
        .cf_clearance("clearance")
        .build()
        .unwrap();
    for path in [
        format!("{}/api/v1/custom", other.uri()),
        other.uri().replacen("http:", "", 1),
    ] {
        assert!(matches!(
            client.get_text(&path, &[]).await,
            Err(Error::ExternalUrl)
        ));
    }
    assert!(other.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn login_does_not_forward_its_password_grant_on_redirect() {
    let origin = MockServer::start().await;
    let other = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .respond_with(ResponseTemplate::new(307).insert_header("location", other.uri()))
        .expect(1)
        .mount(&origin)
        .await;
    let mut client = Client::builder().base_url(origin.uri()).build().unwrap();
    assert!(matches!(
        client
            .login(&Credentials::new(
                "client",
                "client-secret",
                "alice",
                "password"
            ))
            .await,
        Err(Error::Auth(_))
    ));
    assert!(other.received_requests().await.unwrap().is_empty());
}

#[test]
fn base_urls_require_http_without_embedded_credentials() {
    for base in [
        "file:///tmp/api",
        "ftp://example.com/",
        "https://user:secret@example.com/",
    ] {
        assert!(matches!(
            Client::builder().base_url(base).build(),
            Err(Error::InvalidBaseUrl)
        ));
    }
}

#[tokio::test]
async fn endpoint_identifiers_cannot_change_the_request_path_or_query() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(404))
        .expect(6)
        .mount(&server)
        .await;
    let client = Client::builder()
        .base_url(format!("{}/base?discard=this#fragment", server.uri()))
        .build()
        .unwrap();
    let identifier = "a/b?c#d%2Fé";
    let params = archivindex_truthsocial_api::types::TimelineParams::default();
    assert!(matches!(
        client.account(identifier).await,
        Err(Error::Api { .. })
    ));
    assert!(matches!(
        client.account_statuses(identifier, &params).await,
        Err(Error::Api { .. })
    ));
    assert!(matches!(
        client.status(identifier).await,
        Err(Error::Api { .. })
    ));
    assert!(matches!(
        client.status_context(identifier).await,
        Err(Error::Api { .. })
    ));
    assert!(matches!(
        client.hashtag_timeline(identifier, &params).await,
        Err(Error::Api { .. })
    ));
    assert!(matches!(
        client.group_timeline(identifier, &params).await,
        Err(Error::Api { .. })
    ));

    let encoded = "a%2Fb%3Fc%23d%252F%C3%A9";
    let paths = [
        format!("/api/v1/accounts/{encoded}"),
        format!("/api/v1/accounts/{encoded}/statuses"),
        format!("/api/v1/statuses/{encoded}"),
        format!("/api/v1/statuses/{encoded}/context"),
        format!("/api/v1/timelines/tag/{encoded}"),
        format!("/api/v1/timelines/group/{encoded}"),
    ];
    let requests = server.received_requests().await.unwrap();
    for (request, path) in requests.iter().zip(paths) {
        assert_eq!(request.url.path(), path);
        assert!(request.url.query().is_none());
        assert!(request.url.fragment().is_none());
    }
}

#[tokio::test]
async fn empty_and_dot_identifiers_are_rejected_before_sending() {
    let server = MockServer::start().await;
    let client = Client::builder().base_url(server.uri()).build().unwrap();
    for identifier in ["", ".", ".."] {
        assert!(matches!(
            client.status(identifier).await,
            Err(Error::InvalidPathSegment)
        ));
    }
    assert!(server.received_requests().await.unwrap().is_empty());
}
