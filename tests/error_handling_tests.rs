//! Error handling and edge case tests for all API endpoints
//! Tests authentication, rate limiting, and various error conditions

use apiary::client::HoneycombClient;
use serde_json::json;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path, query_param},
};

/// Test authentication failures
mod authentication {
    use super::*;

    #[tokio::test]
    async fn test_invalid_management_key() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/2/teams/test-team/environments"))
            .respond_with(ResponseTemplate::new(401).set_body_json(json!({
                "error": "Unauthorized",
                "message": "Invalid API key"
            })))
            .mount(&mock_server)
            .await;

        let client = HoneycombClient::new(
            Some("invalid-key".to_string()),
            None,
            Some(mock_server.uri()),
        );

        let response = client.get("/2/teams/test-team/environments", None).await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_invalid_config_key() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/1/datasets"))
            .respond_with(ResponseTemplate::new(401).set_body_json(json!({
                "error": "Unauthorized"
            })))
            .mount(&mock_server)
            .await;

        let client = HoneycombClient::new(
            None,
            Some("invalid-key".to_string()),
            Some(mock_server.uri()),
        );

        let response = client.get("/1/datasets", None).await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_missing_required_key() {
        // Allow insecure HTTP URLs for this test
        std::env::set_var("ALLOW_INSECURE_HONEYCOMB_TEST_URLS", "true");
        let client = HoneycombClient::new(None, None, Some("http://api.test".to_string()));

        // Should fail without any API keys
        let response = client.get("/1/datasets", None).await;
        assert!(response.is_err());
    }
}

/// Test rate limiting scenarios
mod rate_limiting {
    use super::*;

    #[tokio::test]
    async fn test_rate_limit_exceeded() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/1/datasets"))
            .respond_with(ResponseTemplate::new(429).set_body_json(json!({
                "error": "Rate limit exceeded",
                "retry_after": 60
            })))
            .mount(&mock_server)
            .await;

        let client =
            HoneycombClient::new(None, Some("test-key".to_string()), Some(mock_server.uri()));

        let response = client.get("/1/datasets", None).await;
        assert!(response.is_err());
    }
}

/// Test resource not found scenarios
mod not_found {
    use super::*;

    #[tokio::test]
    async fn test_dataset_not_found() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/1/datasets/nonexistent"))
            .respond_with(ResponseTemplate::new(404).set_body_json(json!({
                "error": "Dataset not found"
            })))
            .mount(&mock_server)
            .await;

        let client =
            HoneycombClient::new(None, Some("test-key".to_string()), Some(mock_server.uri()));

        let response = client.get("/1/datasets/nonexistent", None).await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_trigger_not_found() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/1/triggers/test-dataset/nonexistent-trigger"))
            .respond_with(ResponseTemplate::new(404).set_body_json(json!({
                "error": "Trigger not found"
            })))
            .mount(&mock_server)
            .await;

        let client =
            HoneycombClient::new(None, Some("test-key".to_string()), Some(mock_server.uri()));

        let response = client
            .get("/1/triggers/test-dataset/nonexistent-trigger", None)
            .await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_api_key_not_found() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/2/teams/test-team/api-keys/nonexistent-key"))
            .respond_with(ResponseTemplate::new(404).set_body_json(json!({
                "error": "API key not found"
            })))
            .mount(&mock_server)
            .await;

        let client =
            HoneycombClient::new(Some("test-key".to_string()), None, Some(mock_server.uri()));

        let response = client
            .get("/2/teams/test-team/api-keys/nonexistent-key", None)
            .await;
        assert!(response.is_err());
    }
}

/// Test malformed request scenarios
mod validation_errors {
    use super::*;

    #[tokio::test]
    async fn test_invalid_json_in_post() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/1/triggers/test-dataset"))
            .respond_with(ResponseTemplate::new(400).set_body_json(json!({
                "error": "Invalid JSON in request body"
            })))
            .mount(&mock_server)
            .await;

        let client =
            HoneycombClient::new(None, Some("test-key".to_string()), Some(mock_server.uri()));

        let invalid_data = json!({
            "name": "", // Empty name should be invalid
            "query_json": "invalid-json"
        });

        let response = client.post("/1/triggers/test-dataset", &invalid_data).await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_missing_required_fields() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/1/markers/test-dataset"))
            .respond_with(ResponseTemplate::new(422).set_body_json(json!({
                "error": "Validation failed",
                "details": ["message is required", "start_time is required"]
            })))
            .mount(&mock_server)
            .await;

        let client =
            HoneycombClient::new(None, Some("test-key".to_string()), Some(mock_server.uri()));

        let incomplete_marker = json!({
            "color": "blue"
            // Missing required fields: message, start_time
        });

        let response = client
            .post("/1/markers/test-dataset", &incomplete_marker)
            .await;
        assert!(response.is_err());
    }
}

/// Test server error scenarios  
mod server_errors {
    use super::*;

    #[tokio::test]
    async fn test_internal_server_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/1/datasets"))
            .respond_with(ResponseTemplate::new(500).set_body_json(json!({
                "error": "Internal server error"
            })))
            .mount(&mock_server)
            .await;

        let client =
            HoneycombClient::new(None, Some("test-key".to_string()), Some(mock_server.uri()));

        let response = client.get("/1/datasets", None).await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_service_unavailable() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/1/datasets"))
            .respond_with(ResponseTemplate::new(503).set_body_json(json!({
                "error": "Service temporarily unavailable"
            })))
            .mount(&mock_server)
            .await;

        let client =
            HoneycombClient::new(None, Some("test-key".to_string()), Some(mock_server.uri()));

        let response = client.get("/1/datasets", None).await;
        assert!(response.is_err());
    }
}

/// Test network and connectivity issues
mod connectivity {
    use super::*;

    #[tokio::test]
    async fn test_connection_timeout() {
        // Allow insecure HTTP URLs for this test
        std::env::set_var("ALLOW_INSECURE_HONEYCOMB_TEST_URLS", "true");
        // Use a non-routable IP to simulate connection timeout
        let client = HoneycombClient::new(
            None,
            Some("test-key".to_string()),
            Some("http://192.0.2.1".to_string()), // RFC5737 test IP
        );

        // Set a timeout to prevent the test from hanging
        let timeout_duration = std::time::Duration::from_secs(5);
        let response =
            tokio::time::timeout(timeout_duration, client.get("/1/datasets", None)).await;

        // The request should either timeout or fail with a connection error
        if let Ok(result) = response {
            assert!(result.is_err());
        }
        // If Err(_), timeout occurred, which is expected
    }

    #[tokio::test]
    async fn test_invalid_hostname() {
        // Allow insecure HTTP URLs for this test
        std::env::set_var("ALLOW_INSECURE_HONEYCOMB_TEST_URLS", "true");
        let client = HoneycombClient::new(
            None,
            Some("test-key".to_string()),
            Some("http://this-domain-does-not-exist-12345.com".to_string()),
        );

        let response = client.get("/1/datasets", None).await;
        assert!(response.is_err());
    }
}

/// Test edge cases with query parameters
mod query_parameters {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_special_characters_in_params() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/1/columns/test-dataset"))
            .and(query_param("key_name", "field with spaces & symbols"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .mount(&mock_server)
            .await;

        let client =
            HoneycombClient::new(None, Some("test-key".to_string()), Some(mock_server.uri()));

        let mut params = HashMap::new();
        params.insert(
            "key_name".to_string(),
            "field with spaces & symbols".to_string(),
        );

        let response = client.get("/1/columns/test-dataset", Some(&params)).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_unicode_in_params() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/1/markers/test-dataset"))
            .and(query_param("message", "🚀 Deployment émoji test 中文"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .mount(&mock_server)
            .await;

        let client =
            HoneycombClient::new(None, Some("test-key".to_string()), Some(mock_server.uri()));

        let mut params = HashMap::new();
        params.insert(
            "message".to_string(),
            "🚀 Deployment émoji test 中文".to_string(),
        );

        let response = client.get("/1/markers/test-dataset", Some(&params)).await;
        assert!(response.is_ok());
    }
}
