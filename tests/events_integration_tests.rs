//! Integration tests for Events API endpoints
//! Tests both single event and batch event submission

use apiary::client::HoneycombClient;
use serde_json::json;
use wiremock::{
    matchers::{method, path, header},
    Mock, MockServer, ResponseTemplate,
};

#[tokio::test]
async fn test_send_single_event() {
    let mock_server = MockServer::start().await;

    // Mock single event endpoint
    Mock::given(method("POST"))
        .and(path("/1/events/test-dataset"))
        .and(header("content-type", "application/json"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let client = HoneycombClient::new(
        None,
        Some("test-config-key".to_string()),
        Some(mock_server.uri()),
    );

    let event_data = json!({
        "timestamp": "2023-01-01T00:00:00Z",
        "service_name": "api",
        "duration_ms": 150,
        "status_code": 200
    });

    let response = client.post("/1/events/test-dataset", &event_data).await;
    assert!(response.is_ok());
}

#[tokio::test]
async fn test_send_batch_events() {
    let mock_server = MockServer::start().await;

    // Mock batch events endpoint
    Mock::given(method("POST"))
        .and(path("/1/batch/test-dataset"))
        .and(header("content-type", "application/json"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let client = HoneycombClient::new(
        None,
        Some("test-config-key".to_string()),
        Some(mock_server.uri()),
    );

    let batch_data = json!([
        {
            "timestamp": "2023-01-01T00:00:00Z",
            "service_name": "api",
            "duration_ms": 150
        },
        {
            "timestamp": "2023-01-01T00:00:01Z", 
            "service_name": "api",
            "duration_ms": 200
        }
    ]);

    let response = client.post("/1/batch/test-dataset", &batch_data).await;
    assert!(response.is_ok());
}

#[tokio::test]
async fn test_event_validation_error() {
    let mock_server = MockServer::start().await;

    // Mock validation error response
    Mock::given(method("POST"))
        .and(path("/1/events/test-dataset"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": "Invalid timestamp format"
        })))
        .mount(&mock_server)
        .await;

    let client = HoneycombClient::new(
        None,
        Some("test-config-key".to_string()),
        Some(mock_server.uri()),
    );

    let invalid_event = json!({
        "timestamp": "invalid-timestamp",
        "service_name": "api"
    });

    let response = client.post("/1/events/test-dataset", &invalid_event).await;
    assert!(response.is_err());
}