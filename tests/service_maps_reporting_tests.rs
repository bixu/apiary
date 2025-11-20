//! Tests for Service Maps and Reporting functionality
//! These are more complex API endpoints that provide aggregated data

use apiary::client::HoneycombClient;
use serde_json::json;
use wiremock::{
    matchers::{method, path, query_param},
    Mock, MockServer, ResponseTemplate,
};

/// Test Service Maps endpoints
mod service_maps {
    use super::*;

    #[tokio::test]
    async fn test_get_service_map() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/1/service_map/test-dataset"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "services": [
                    {
                        "name": "api-gateway",
                        "request_rate": 1500,
                        "error_rate": 0.02,
                        "avg_duration": 45,
                        "p95_duration": 120
                    },
                    {
                        "name": "user-service", 
                        "request_rate": 800,
                        "error_rate": 0.01,
                        "avg_duration": 25,
                        "p95_duration": 65
                    }
                ],
                "connections": [
                    {
                        "from": "api-gateway",
                        "to": "user-service",
                        "request_rate": 400,
                        "error_rate": 0.005
                    }
                ],
                "time_range": {
                    "start": "2023-01-01T00:00:00Z",
                    "end": "2023-01-01T01:00:00Z"
                }
            })))
            .mount(&mock_server)
            .await;

        let client = HoneycombClient::new(
            None,
            Some("test-config-key".to_string()),
            Some(mock_server.uri()),
        );

        let response = client.get("/1/service_map/test-dataset", None).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_service_map_with_time_range() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/1/service_map/test-dataset"))
            .and(query_param("start_time", "2023-01-01T00:00:00Z"))
            .and(query_param("end_time", "2023-01-01T01:00:00Z"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "services": [],
                "connections": [],
                "time_range": {
                    "start": "2023-01-01T00:00:00Z",
                    "end": "2023-01-01T01:00:00Z"
                }
            })))
            .mount(&mock_server)
            .await;

        let client = HoneycombClient::new(
            None,
            Some("test-config-key".to_string()),
            Some(mock_server.uri()),
        );

        let mut params = std::collections::HashMap::new();
        params.insert("start_time".to_string(), "2023-01-01T00:00:00Z".to_string());
        params.insert("end_time".to_string(), "2023-01-01T01:00:00Z".to_string());

        let response = client.get("/1/service_map/test-dataset", Some(&params)).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_service_map_filtered_by_service() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/1/service_map/test-dataset"))
            .and(query_param("service", "api-gateway"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "services": [
                    {
                        "name": "api-gateway",
                        "request_rate": 1500,
                        "error_rate": 0.02,
                        "avg_duration": 45,
                        "p95_duration": 120
                    }
                ],
                "connections": [],
                "focus_service": "api-gateway"
            })))
            .mount(&mock_server)
            .await;

        let client = HoneycombClient::new(
            None,
            Some("test-config-key".to_string()),
            Some(mock_server.uri()),
        );

        let mut params = std::collections::HashMap::new();
        params.insert("service".to_string(), "api-gateway".to_string());

        let response = client.get("/1/service_map/test-dataset", Some(&params)).await;
        assert!(response.is_ok());
    }
}

/// Test Reporting endpoints
mod reporting {
    use super::*;

    #[tokio::test]
    async fn test_daily_usage_report() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/1/usage/daily"))
            .and(query_param("date", "2023-01-01"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "date": "2023-01-01",
                "total_events": 1250000,
                "billable_events": 1200000,
                "datasets": [
                    {
                        "name": "production",
                        "events": 800000,
                        "billable_events": 780000
                    },
                    {
                        "name": "staging", 
                        "events": 450000,
                        "billable_events": 420000
                    }
                ],
                "cost_estimate": 125.50
            })))
            .mount(&mock_server)
            .await;

        let client = HoneycombClient::new(
            None,
            Some("test-config-key".to_string()),
            Some(mock_server.uri()),
        );

        let mut params = std::collections::HashMap::new();
        params.insert("date".to_string(), "2023-01-01".to_string());

        let response = client.get("/1/usage/daily", Some(&params)).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_monthly_usage_report() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/1/usage/monthly"))
            .and(query_param("month", "2023-01"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "month": "2023-01",
                "total_events": 38750000,
                "billable_events": 37200000,
                "daily_breakdown": [
                    {
                        "date": "2023-01-01",
                        "events": 1250000,
                        "billable_events": 1200000
                    },
                    {
                        "date": "2023-01-02",
                        "events": 1300000,
                        "billable_events": 1250000
                    }
                ],
                "projected_cost": 3720.00,
                "billing_period": {
                    "start": "2023-01-01",
                    "end": "2023-01-31"
                }
            })))
            .mount(&mock_server)
            .await;

        let client = HoneycombClient::new(
            None,
            Some("test-config-key".to_string()),
            Some(mock_server.uri()),
        );

        let mut params = std::collections::HashMap::new();
        params.insert("month".to_string(), "2023-01".to_string());

        let response = client.get("/1/usage/monthly", Some(&params)).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_slo_performance_report() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/1/reports/slo_performance"))
            .and(query_param("dataset", "test-dataset"))
            .and(query_param("time_range", "7d"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "dataset": "test-dataset",
                "time_range": "7d",
                "slos": [
                    {
                        "id": "slo-123",
                        "name": "API Response Time",
                        "target_percentage": 99.9,
                        "actual_percentage": 99.95,
                        "status": "healthy",
                        "budget_remaining": 95.2,
                        "error_budget_consumed": 4.8
                    },
                    {
                        "id": "slo-456", 
                        "name": "Service Availability",
                        "target_percentage": 99.5,
                        "actual_percentage": 99.2,
                        "status": "warning",
                        "budget_remaining": 40.0,
                        "error_budget_consumed": 60.0
                    }
                ],
                "overall_health": "warning",
                "generated_at": "2023-01-08T00:00:00Z"
            })))
            .mount(&mock_server)
            .await;

        let client = HoneycombClient::new(
            None,
            Some("test-config-key".to_string()),
            Some(mock_server.uri()),
        );

        let mut params = std::collections::HashMap::new();
        params.insert("dataset".to_string(), "test-dataset".to_string());
        params.insert("time_range".to_string(), "7d".to_string());

        let response = client.get("/1/reports/slo_performance", Some(&params)).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_alert_summary_report() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/1/reports/alerts"))
            .and(query_param("time_range", "24h"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "time_range": "24h",
                "total_alerts": 15,
                "alerts_by_severity": {
                    "critical": 2,
                    "warning": 8,
                    "info": 5
                },
                "alerts_by_dataset": {
                    "production": 12,
                    "staging": 3
                },
                "top_triggers": [
                    {
                        "trigger_id": "trigger-123",
                        "name": "High Error Rate",
                        "count": 5,
                        "last_fired": "2023-01-01T23:45:00Z"
                    },
                    {
                        "trigger_id": "trigger-456",
                        "name": "Slow Response Time",
                        "count": 3,
                        "last_fired": "2023-01-01T22:30:00Z"
                    }
                ],
                "mttr_minutes": 18.5,
                "generated_at": "2023-01-02T00:00:00Z"
            })))
            .mount(&mock_server)
            .await;

        let client = HoneycombClient::new(
            None,
            Some("test-config-key".to_string()),
            Some(mock_server.uri()),
        );

        let mut params = std::collections::HashMap::new();
        params.insert("time_range".to_string(), "24h".to_string());

        let response = client.get("/1/reports/alerts", Some(&params)).await;
        assert!(response.is_ok());
    }
}

/// Test complex data aggregation scenarios
mod aggregation_tests {
    use super::*;

    #[tokio::test]
    async fn test_cross_dataset_analysis() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/1/analysis/cross_dataset"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "analysis_id": "analysis-789",
                "datasets": ["frontend", "backend", "database"],
                "correlations": [
                    {
                        "datasets": ["frontend", "backend"],
                        "correlation": 0.85,
                        "metric": "response_time"
                    }
                ],
                "insights": [
                    {
                        "type": "anomaly",
                        "description": "Unusual spike in frontend errors correlates with backend latency increase",
                        "confidence": 0.92,
                        "time_window": "2023-01-01T14:00:00Z to 2023-01-01T15:00:00Z"
                    }
                ],
                "status": "completed"
            })))
            .mount(&mock_server)
            .await;

        let client = HoneycombClient::new(
            None,
            Some("test-config-key".to_string()),
            Some(mock_server.uri()),
        );

        let analysis_request = json!({
            "datasets": ["frontend", "backend", "database"],
            "time_range": "1h",
            "metrics": ["response_time", "error_rate", "throughput"],
            "analysis_type": "correlation"
        });

        let response = client.post("/1/analysis/cross_dataset", &analysis_request).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_performance_baseline() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/1/baseline/test-dataset"))
            .and(query_param("metric", "response_time"))
            .and(query_param("baseline_period", "7d"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "dataset": "test-dataset",
                "metric": "response_time", 
                "baseline_period": "7d",
                "current_period": "1h",
                "baseline_stats": {
                    "mean": 85.5,
                    "p50": 75.0,
                    "p95": 150.0,
                    "p99": 250.0,
                    "std_dev": 32.1
                },
                "current_stats": {
                    "mean": 92.3,
                    "p50": 80.0,
                    "p95": 165.0,
                    "p99": 280.0,
                    "std_dev": 38.7
                },
                "deviation_score": 1.2,
                "anomaly_detected": false,
                "trend": "slight_increase"
            })))
            .mount(&mock_server)
            .await;

        let client = HoneycombClient::new(
            None,
            Some("test-config-key".to_string()),
            Some(mock_server.uri()),
        );

        let mut params = std::collections::HashMap::new();
        params.insert("metric".to_string(), "response_time".to_string());
        params.insert("baseline_period".to_string(), "7d".to_string());

        let response = client.get("/1/baseline/test-dataset", Some(&params)).await;
        assert!(response.is_ok());
    }
}