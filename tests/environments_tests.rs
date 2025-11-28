use apiary::client::HoneycombClient;
use apiary::common::{require_valid_environment, validate_environment};
use serde_json::json;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

#[tokio::test]
async fn test_validate_environment_with_valid_slug() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/2/teams/test-team/environments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {
                    "id": "env-123",
                    "type": "environment",
                    "attributes": {
                        "name": "Production",
                        "slug": "production",
                        "description": "Production environment",
                        "timestamps": {
                            "created": "2023-01-01T00:00:00Z",
                            "updated": "2023-01-01T00:00:00Z"
                        }
                    }
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = HoneycombClient::new(
        Some("test-mgmt-key".to_string()),
        Some("test-config-key".to_string()),
        Some(mock_server.uri()),
    );

    let result = validate_environment(&client, "test-team", "production")
        .await
        .unwrap();
    assert!(result, "Should validate existing environment slug");
}

#[tokio::test]
async fn test_validate_environment_with_valid_name() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/2/teams/test-team/environments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {
                    "id": "env-123",
                    "type": "environment",
                    "attributes": {
                        "name": "Production",
                        "slug": "prod",
                        "timestamps": {
                            "created": "2023-01-01T00:00:00Z",
                            "updated": "2023-01-01T00:00:00Z"
                        }
                    }
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = HoneycombClient::new(
        Some("test-mgmt-key".to_string()),
        Some("test-config-key".to_string()),
        Some(mock_server.uri()),
    );

    let result = validate_environment(&client, "test-team", "Production")
        .await
        .unwrap();
    assert!(result, "Should validate existing environment name");
}

#[tokio::test]
async fn test_validate_environment_with_invalid_environment() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/2/teams/test-team/environments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {
                    "id": "env-123",
                    "type": "environment",
                    "attributes": {
                        "name": "Production",
                        "slug": "production",
                        "timestamps": {
                            "created": "2023-01-01T00:00:00Z",
                            "updated": "2023-01-01T00:00:00Z"
                        }
                    }
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = HoneycombClient::new(
        Some("test-mgmt-key".to_string()),
        Some("test-config-key".to_string()),
        Some(mock_server.uri()),
    );

    let result = validate_environment(&client, "test-team", "nonexistent")
        .await
        .unwrap();
    assert!(!result, "Should not validate nonexistent environment");
}

#[tokio::test]
async fn test_require_valid_environment_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/2/teams/test-team/environments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {
                    "id": "env-123",
                    "type": "environment",
                    "attributes": {
                        "name": "Production",
                        "slug": "production",
                        "timestamps": {
                            "created": "2023-01-01T00:00:00Z",
                            "updated": "2023-01-01T00:00:00Z"
                        }
                    }
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = HoneycombClient::new(
        Some("test-mgmt-key".to_string()),
        Some("test-config-key".to_string()),
        Some(mock_server.uri()),
    );

    let result = require_valid_environment(&client, "test-team", "production").await;
    assert!(result.is_ok(), "Should succeed for valid environment");
}
#[tokio::test]
async fn test_require_valid_environment_failure() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/2/teams/test-team/environments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": []
        })))
        .mount(&mock_server)
        .await;

    let client = HoneycombClient::new(
        Some("test-mgmt-key".to_string()),
        Some("test-config-key".to_string()),
        Some(mock_server.uri()),
    );

    let result = require_valid_environment(&client, "test-team", "invalid-env").await;
    assert!(result.is_err(), "Should fail for invalid environment");

    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("Environment 'invalid-env' not found in team 'test-team'"));
    assert!(error_message
        .contains("Use 'apiary environments list --team test-team' to see available environments"));
}

#[tokio::test]
async fn test_require_valid_environment_without_management_key() {
    let client = HoneycombClient::new(
        None,
        Some("test-config-key".to_string()),
        Some("http://127.0.0.1:9".to_string()),
    );

    let result = require_valid_environment(&client, "test-team", "production").await;
    assert!(
        result.is_ok(),
        "Environment validation should be skipped when no management key is present"
    );
}
