use serde_json::Value;
use std::fmt;

/// Standard error types for the CLI
#[allow(dead_code)]
#[derive(Debug)]
pub enum ApiaryError {
    /// Authentication required but not provided
    AuthenticationRequired(String),
    /// Resource not found
    NotFound {
        resource: String,
        identifier: String,
    },
    /// Validation error
    ValidationError(String),
    /// API communication error
    ApiError { status: u16, message: String },
    /// Configuration error
    ConfigError(String),
}

impl fmt::Display for ApiaryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiaryError::AuthenticationRequired(endpoint) => {
                write!(
                    f,
                    "Authentication required for endpoint '{}'. Please set appropriate API keys.",
                    endpoint
                )
            }
            ApiaryError::NotFound {
                resource,
                identifier,
            } => {
                write!(f, "{} '{}' not found", resource, identifier)
            }
            ApiaryError::ValidationError(msg) => {
                write!(f, "Validation error: {}", msg)
            }
            ApiaryError::ApiError { status, message } => {
                write!(f, "API error ({}): {}", status, message)
            }
            ApiaryError::ConfigError(msg) => {
                write!(f, "Configuration error: {}", msg)
            }
        }
    }
}

impl std::error::Error for ApiaryError {}

/// Helper function to parse API error responses
#[allow(dead_code)]
pub fn parse_api_error(status: u16, body: &Value) -> ApiaryError {
    let message = body
        .get("error")
        .or_else(|| body.get("message"))
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown error")
        .to_string();

    ApiaryError::ApiError { status, message }
}

/// Standard error messages
pub mod messages {
    pub const TEAM_REQUIRED: &str =
        "Team is required. Use --team flag or set HONEYCOMB_TEAM environment variable.";
    #[allow(dead_code)]
    pub const ENVIRONMENT_REQUIRED: &str = "Environment is required. Use --environment flag or set HONEYCOMB_ENVIRONMENT environment variable.";
    pub const MANAGEMENT_KEY_REQUIRED: &str = "Management API key required for v2 endpoints. Set HONEYCOMB_MANAGEMENT_API_KEY_ID and HONEYCOMB_MANAGEMENT_API_KEY.";
    pub const CONFIG_KEY_REQUIRED: &str =
        "Configuration API key required for v1 endpoints. Set HONEYCOMB_CONFIGURATION_API_KEY.";

    pub fn environment_not_found(env: &str, team: &str) -> String {
        format!(
            "Environment '{}' not found in team '{}'. Use 'apiary environments list --team {}' to see available environments.",
            env, team, team
        )
    }
}
