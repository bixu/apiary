//! Comprehensive test coverage for all Honeycomb API resources
//! This file tests all API endpoints referenced by the tool

mod test_utils;

use apiary::client::HoneycombClient;
use test_utils::*;
use serde_json::json;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

/// Test API Keys endpoints
mod api_keys {
    use super::*;

    #[tokio::test]
    async fn test_list_api_keys() {
        let mock_server = create_mock_server().await;
        
        mock_successful_list(
            &mock_server,
            "/2/teams/test-team/api_keys",
            sample_api_key_data()
        ).await;

        let client = create_test_client(mock_server.uri());
        let response = client.get("/2/teams/test-team/api_keys", None).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_get_api_key() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/2/teams/test-team/api_keys/key-123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": {
                    "id": "key-123",
                    "type": "api_key",
                    "attributes": {
                        "name": "Test Key",
                        "key_type": "management",
                        "disabled": false
                    }
                }
            })))
            .mount(&mock_server)
            .await;

        let client = HoneycombClient::new(
            Some("test-mgmt-key".to_string()),
            None,
            Some(mock_server.uri()),
        );

        let response = client
            .get("/2/teams/test-team/api_keys/key-123", None)
            .await;
        assert!(response.is_ok());
    }
}

/// Test Environments endpoints
mod environments {
    use super::*;
    use apiary::common::OutputFormat;
    use apiary::environments::EnvironmentCommands;

    #[tokio::test]
    async fn test_list_environments() {
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
                            "color": "blue"
                        }
                    }
                ]
            })))
            .mount(&mock_server)
            .await;

        let client =
            HoneycombClient::new(Some("test-key".to_string()), None, Some(mock_server.uri()));

        let command = EnvironmentCommands::List {
            team: Some("test-team".to_string()),
            format: OutputFormat::Json,
        };

        let result = command.execute(&client, &None).await;
        assert!(result.is_ok());
    }
}

/// Test Datasets endpoints  
mod datasets {
    use super::*;

    #[tokio::test]
    async fn test_list_datasets() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/1/datasets"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "name": "test-dataset",
                    "slug": "test-dataset",
                    "created_at": "2023-01-01T00:00:00Z",
                    "last_written_at": "2023-01-01T00:00:00Z"
                }
            ])))
            .mount(&mock_server)
            .await;

        let client = HoneycombClient::new(
            None,
            Some("test-config-key".to_string()),
            Some(mock_server.uri()),
        );

        let response = client.get("/1/datasets", None).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_get_dataset() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/1/datasets/test-dataset"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "name": "test-dataset",
                "slug": "test-dataset",
                "created_at": "2023-01-01T00:00:00Z"
            })))
            .mount(&mock_server)
            .await;

        let client = HoneycombClient::new(
            None,
            Some("test-config-key".to_string()),
            Some(mock_server.uri()),
        );

        let response = client.get("/1/datasets/test-dataset", None).await;
        assert!(response.is_ok());
    }
}

/// Test Columns endpoints
mod columns {
    use super::*;
    use apiary::columns::ColumnCommands;
    use apiary::common::OutputFormat;

    #[tokio::test]
    async fn test_list_columns() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/1/columns/test-dataset"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "id": "col-123",
                    "key_name": "duration_ms",
                    "hidden": false,
                    "type": "float",
                    "created_at": "2023-01-01T00:00:00Z"
                }
            ])))
            .mount(&mock_server)
            .await;

        let client = HoneycombClient::new(
            None,
            Some("test-config-key".to_string()),
            Some(mock_server.uri()),
        );

        let command = ColumnCommands::List {
            dataset: "test-dataset".to_string(),
            format: OutputFormat::Json,
            environment: None,
        };

        let result = command.execute(&client).await;
        assert!(result.is_ok());
    }
}

/// Test Triggers endpoints
mod triggers {
    use super::*;
    use apiary::common::OutputFormat;
    use apiary::triggers::TriggerCommands;

    #[tokio::test]
    async fn test_list_triggers() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/1/triggers/test-dataset"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "id": "trigger-123",
                    "name": "High Error Rate",
                    "disabled": false,
                    "alert_type": "static_threshold",
                    "created_at": "2023-01-01T00:00:00Z",
                    "recipients": []
                }
            ])))
            .mount(&mock_server)
            .await;

        let client = HoneycombClient::new(
            None,
            Some("test-config-key".to_string()),
            Some(mock_server.uri()),
        );

        let command = TriggerCommands::List {
            dataset: "test-dataset".to_string(),
            format: OutputFormat::Json,
            environment: None,
        };

        let result = command.execute(&client).await;
        assert!(result.is_ok());
    }
}

/// Test SLOs endpoints
mod slos {
    use super::*;
    use apiary::common::OutputFormat;
    use apiary::slos::SloCommands;

    #[tokio::test]
    async fn test_list_slos() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/1/slos/test-dataset"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "id": "slo-123",
                    "name": "API Response Time",
                    "target_percentage": 99.9,
                    "time_period": "7d",
                    "created_at": "2023-01-01T00:00:00Z"
                }
            ])))
            .mount(&mock_server)
            .await;

        let client = HoneycombClient::new(
            None,
            Some("test-config-key".to_string()),
            Some(mock_server.uri()),
        );

        let command = SloCommands::List {
            dataset: "test-dataset".to_string(),
            format: OutputFormat::Json,
            environment: None,
        };

        let result = command.execute(&client).await;
        assert!(result.is_ok());
    }
}

/// Test Boards endpoints
mod boards {
    use super::*;
    use apiary::boards::BoardCommands;
    use apiary::common::OutputFormat;

    #[tokio::test]
    async fn test_list_boards() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/1/boards"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "id": "board-123",
                    "name": "Service Dashboard",
                    "style": "visual",
                    "created_at": "2023-01-01T00:00:00Z"
                }
            ])))
            .mount(&mock_server)
            .await;

        let client = HoneycombClient::new(
            None,
            Some("test-config-key".to_string()),
            Some(mock_server.uri()),
        );

        let command = BoardCommands::List {
            environment: None,
            format: OutputFormat::Json,
        };

        let result = command.execute(&client).await;
        assert!(result.is_ok());
    }
}

/// Test Markers endpoints
mod markers {
    use super::*;
    use apiary::common::OutputFormat;
    use apiary::markers::MarkerCommands;

    #[tokio::test]
    async fn test_list_markers() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/1/markers/test-dataset"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "id": "marker-123",
                    "message": "Deployment v1.2.3",
                    "start_time": 1609459200,
                    "color": "blue",
                    "url": "https://github.com/repo/commit/abc123"
                }
            ])))
            .mount(&mock_server)
            .await;

        let client = HoneycombClient::new(
            None,
            Some("test-config-key".to_string()),
            Some(mock_server.uri()),
        );

        let command = MarkerCommands::List {
            dataset: "test-dataset".to_string(),
            format: OutputFormat::Json,
            environment: None,
        };

        let result = command.execute(&client).await;
        assert!(result.is_ok());
    }
}

/// Test Recipients endpoints
mod recipients {
    use super::*;
    use apiary::common::OutputFormat;
    use apiary::recipients::RecipientCommands;

    #[tokio::test]
    async fn test_list_recipients() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/1/recipients"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "id": "recipient-123",
                    "name": "Engineering Team",
                    "type": "slack",
                    "target": "#engineering",
                    "created_at": "2023-01-01T00:00:00Z"
                }
            ])))
            .mount(&mock_server)
            .await;

        let client = HoneycombClient::new(
            None,
            Some("test-config-key".to_string()),
            Some(mock_server.uri()),
        );

        let command = RecipientCommands::List {
            format: OutputFormat::Json,
        };

        let result = command.execute(&client).await;
        assert!(result.is_ok());
    }
}

/// Test Burn Alerts endpoints
mod burn_alerts {
    use super::*;
    use apiary::burn_alerts::BurnAlertCommands;
    use apiary::common::OutputFormat;

    #[tokio::test]
    async fn test_list_burn_alerts() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/1/burn_alerts/test-dataset"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "id": "alert-123",
                    "slo_id": "slo-456",
                    "exhaustion_minutes": 60,
                    "alert_window_minutes": 5,
                    "disabled": false,
                    "recipients": []
                }
            ])))
            .mount(&mock_server)
            .await;

        let client = HoneycombClient::new(
            None,
            Some("test-config-key".to_string()),
            Some(mock_server.uri()),
        );

        let command = BurnAlertCommands::List {
            dataset: "test-dataset".to_string(),
            format: OutputFormat::Json,
            environment: None,
        };

        let result = command.execute(&client).await;
        assert!(result.is_ok());
    }
}

/// Test Calculated Fields endpoints
mod calculated_fields {
    use super::*;
    use apiary::calculated_fields::CalculatedFieldCommands;
    use apiary::common::OutputFormat;

    #[tokio::test]
    async fn test_list_calculated_fields() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/1/derived_columns/test-dataset"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "id": "field-123",
                    "alias": "error_rate",
                    "expression": "COUNT(*) WHERE status_code >= 400",
                    "created_at": "2023-01-01T00:00:00Z"
                }
            ])))
            .mount(&mock_server)
            .await;

        let client = HoneycombClient::new(
            None,
            Some("test-config-key".to_string()),
            Some(mock_server.uri()),
        );

        let command = CalculatedFieldCommands::List {
            dataset: "test-dataset".to_string(),
            format: OutputFormat::Json,
            environment: None,
        };

        let result = command.execute(&client).await;
        assert!(result.is_ok());
    }
}

/// Test Dataset Definitions endpoints
mod dataset_definitions {
    use super::*;
    use apiary::common::OutputFormat;
    use apiary::dataset_definitions::DatasetDefinitionCommands;

    #[tokio::test]
    async fn test_get_dataset_definitions() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/1/dataset_definitions/test-dataset"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "trace_id_field": "trace.trace_id",
                "parent_id_field": "trace.parent_id",
                "name_field": "name",
                "service_name_field": "service_name"
            })))
            .mount(&mock_server)
            .await;

        let client = HoneycombClient::new(
            None,
            Some("test-config-key".to_string()),
            Some(mock_server.uri()),
        );

        let command = DatasetDefinitionCommands::Get {
            dataset: "test-dataset".to_string(),
            team: None,
            environment: None,
            format: OutputFormat::Json,
        };

        let result = command.execute(&client, None).await;
        assert!(result.is_ok());
    }
}

/// Test Marker Settings endpoints
mod marker_settings {
    use super::*;
    use apiary::common::OutputFormat;
    use apiary::marker_settings::MarkerSettingCommands;

    #[tokio::test]
    async fn test_list_marker_settings() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/1/marker_settings/test-dataset"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "id": "setting-123",
                    "type": "deployment",
                    "color": "blue",
                    "created_at": "2023-01-01T00:00:00Z"
                }
            ])))
            .mount(&mock_server)
            .await;

        let client = HoneycombClient::new(
            None,
            Some("test-config-key".to_string()),
            Some(mock_server.uri()),
        );

        let command = MarkerSettingCommands::List {
            dataset: "test-dataset".to_string(),
            format: OutputFormat::Json,
            environment: None,
        };

        let result = command.execute(&client).await;
        assert!(result.is_ok());
    }
}

/// Test Auth endpoints
mod auth {
    use super::*;
    use apiary::auth::AuthCommands;

    #[tokio::test]
    async fn test_auth_whoami() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/1/auth"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "team": {
                    "name": "Test Team",
                    "slug": "test-team"
                },
                "environment": {
                    "name": "Production",
                    "slug": "production"
                }
            })))
            .mount(&mock_server)
            .await;

        let client = HoneycombClient::new(
            None,
            Some("test-config-key".to_string()),
            Some(mock_server.uri()),
        );

        let command = AuthCommands::Info;

        let result = command.execute(&client).await;
        assert!(result.is_ok());
    }
}
