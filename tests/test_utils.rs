use serde_json::json;
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::{method, path}};
use crate::client::HoneycombClient;

/// Create a mock server for testing
pub async fn create_mock_server() -> MockServer {
    MockServer::start().await
}

/// Create a test client with mock server
pub fn create_test_client(mock_server_uri: String) -> HoneycombClient {
    HoneycombClient::new(
        Some("test-mgmt-key".to_string()),
        Some("test-config-key".to_string()),
        Some(mock_server_uri),
    )
}

/// Setup standard mock for successful list responses
pub async fn mock_successful_list(server: &MockServer, endpoint: &str, response_data: serde_json::Value) {
    Mock::given(method("GET"))
        .and(path(endpoint))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_data))
        .mount(server)
        .await;
}

/// Setup standard mock for successful get responses  
pub async fn mock_successful_get(server: &MockServer, endpoint: &str, response_data: serde_json::Value) {
    Mock::given(method("GET"))
        .and(path(endpoint))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_data))
        .mount(server)
        .await;
}

/// Setup standard mock for 404 responses
pub async fn mock_not_found(server: &MockServer, endpoint: &str) {
    Mock::given(method("GET"))
        .and(path(endpoint))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Not found"
        })))
        .mount(server)
        .await;
}

/// Standard test data for environments
pub fn sample_environment_data() -> serde_json::Value {
    json!({
        "data": [
            {
                "id": "env-123",
                "type": "environment",
                "attributes": {
                    "name": "Production",
                    "slug": "production",
                    "description": "Production environment"
                }
            }
        ]
    })
}

/// Standard test data for datasets
pub fn sample_dataset_data() -> serde_json::Value {
    json!({
        "data": [
            {
                "name": "test-dataset",
                "slug": "test-dataset",
                "created_at": "2023-01-01T00:00:00Z",
                "last_written_at": "2023-01-02T00:00:00Z"
            }
        ]
    })
}

/// Standard test data for API keys
pub fn sample_api_key_data() -> serde_json::Value {
    json!({
        "data": [
            {
                "id": "key-123",
                "type": "api_key",
                "attributes": {
                    "name": "Test Key",
                    "key_type": "management",
                    "disabled": false,
                    "environment_id": "env-456"
                }
            }
        ]
    })
}