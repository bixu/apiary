//! Integration tests for Query Results API endpoints
//! Tests query execution, result polling, and data retrieval

use apiary::client::HoneycombClient;
use serde_json::json;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

#[tokio::test]
async fn test_create_query_result() {
    let mock_server = MockServer::start().await;

    // Mock query result creation
    Mock::given(method("POST"))
        .and(path("/1/query_results/test-dataset"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "query_result_id": "result-123",
            "query_url": "https://ui.honeycomb.io/test-team/datasets/test-dataset/result/result-123",
            "complete": false,
            "links": {
                "query_result": "/1/query_results/test-dataset/result-123"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = HoneycombClient::new(
        None,
        Some("test-config-key".to_string()),
        Some(mock_server.uri()),
    );

    let query_spec = json!({
        "calculations": [{"op": "COUNT"}],
        "filters": [],
        "breakdowns": ["service_name"],
        "orders": [{"op": "COUNT", "order": "descending"}],
        "limit": 100,
        "time_range": 3600
    });

    let response = client.post("/1/query_results/test-dataset", &query_spec).await;
    assert!(response.is_ok());
}

#[tokio::test]
async fn test_get_query_result_in_progress() {
    let mock_server = MockServer::start().await;

    // Mock query result still processing
    Mock::given(method("GET"))
        .and(path("/1/query_results/test-dataset/result-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "query_result_id": "result-123",
            "query_url": "https://ui.honeycomb.io/test-team/datasets/test-dataset/result/result-123", 
            "complete": false,
            "links": {
                "query_result": "/1/query_results/test-dataset/result-123"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = HoneycombClient::new(
        None,
        Some("test-config-key".to_string()),
        Some(mock_server.uri()),
    );

    let response = client.get("/1/query_results/test-dataset/result-123", None).await;
    assert!(response.is_ok());
}

#[tokio::test]
async fn test_get_query_result_complete_with_data() {
    let mock_server = MockServer::start().await;

    // Mock completed query result with data
    Mock::given(method("GET"))
        .and(path("/1/query_results/test-dataset/result-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "query_result_id": "result-123",
            "query_url": "https://ui.honeycomb.io/test-team/datasets/test-dataset/result/result-123",
            "complete": true,
            "links": {
                "query_result": "/1/query_results/test-dataset/result-123",
                "graph_image": "/1/query_results/test-dataset/result-123/graph_image"
            },
            "data": {
                "series": [
                    {
                        "time": "2023-01-01T00:00:00Z",
                        "data": [
                            {
                                "service_name": "api",
                                "COUNT": 1000
                            },
                            {
                                "service_name": "worker", 
                                "COUNT": 500
                            }
                        ]
                    }
                ],
                "is_time_series": true,
                "query": {
                    "calculations": [{"op": "COUNT"}],
                    "breakdowns": ["service_name"]
                }
            }
        })))
        .mount(&mock_server)
        .await;

    let client = HoneycombClient::new(
        None,
        Some("test-config-key".to_string()),
        Some(mock_server.uri()),
    );

    let response = client.get("/1/query_results/test-dataset/result-123", None).await;
    assert!(response.is_ok());
}

#[tokio::test] 
async fn test_query_timeout() {
    let mock_server = MockServer::start().await;

    // Mock query timeout error
    Mock::given(method("POST"))
        .and(path("/1/query_results/test-dataset"))
        .respond_with(ResponseTemplate::new(408).set_body_json(json!({
            "error": "Query timeout - query took too long to complete"
        })))
        .mount(&mock_server)
        .await;

    let client = HoneycombClient::new(
        None,
        Some("test-config-key".to_string()),
        Some(mock_server.uri()),
    );

    let complex_query = json!({
        "calculations": [
            {"op": "AVG", "column": "duration_ms"},
            {"op": "P95", "column": "duration_ms"},
            {"op": "COUNT"}
        ],
        "filters": [
            {"column": "service_name", "op": "=", "value": "api"}
        ],
        "breakdowns": ["endpoint", "status_code", "region"],
        "time_range": 86400  // 24 hours
    });

    let response = client.post("/1/query_results/test-dataset", &complex_query).await;
    assert!(response.is_err());
}

#[tokio::test]
async fn test_query_with_invalid_syntax() {
    let mock_server = MockServer::start().await;

    // Mock query syntax error
    Mock::given(method("POST"))
        .and(path("/1/query_results/test-dataset"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": "Invalid query: unknown column 'nonexistent_field'"
        })))
        .mount(&mock_server)
        .await;

    let client = HoneycombClient::new(
        None,
        Some("test-config-key".to_string()),
        Some(mock_server.uri()),
    );

    let invalid_query = json!({
        "calculations": [{"op": "AVG", "column": "nonexistent_field"}],
        "filters": [],
        "breakdowns": [],
        "time_range": 3600
    });

    let response = client.post("/1/query_results/test-dataset", &invalid_query).await;
    assert!(response.is_err());
}