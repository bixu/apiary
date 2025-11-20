use anyhow::{Context, Result};
use reqwest::{Client, Method, Response};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HoneycombClient {
    client: Client,
    management_key: Option<String>,
    config_key: Option<String>,
    base_url: String,
}

impl HoneycombClient {
    pub fn new(
        management_key: Option<String>,
        config_key: Option<String>,
        base_url: Option<String>,
    ) -> Self {
        Self {
            client: Client::new(),
            management_key,
            config_key,
            base_url: base_url.unwrap_or_else(|| "https://api.honeycomb.io".to_string()),
        }
    }

    /// Get the appropriate API key for debugging/info purposes
    pub fn get_key_for_endpoint(&self, path: &str) -> Option<&str> {
        if self.is_v2_endpoint(path) {
            self.management_key.as_deref()
        } else {
            self.config_key.as_deref()
        }
    }

    pub async fn request(
        &self,
        method: Method,
        path: &str,
        query_params: Option<&HashMap<String, String>>,
        body: Option<&Value>,
    ) -> Result<Response> {
        let url = format!("{}{}", self.base_url, path);

        let mut request = self.client.request(method, &url);

        // Use appropriate authentication based on endpoint
        if self.is_v2_endpoint(path) {
            // v2 endpoints use Management Key with Bearer token
            if let Some(management_key) = &self.management_key {
                request = request.header("Authorization", format!("Bearer {}", management_key));
            } else {
                return Err(anyhow::anyhow!(
                    "v2 endpoint '{}' requires a Management Key.\n\
                     Set HONEYCOMB_MANAGEMENT_API_KEY_ID and HONEYCOMB_MANAGEMENT_API_KEY environment variables.",
                    path
                ));
            }
        } else {
            // v1 endpoints use Configuration Key with X-Honeycomb-Team header
            if let Some(config_key) = &self.config_key {
                request = request.header("X-Honeycomb-Team", config_key);
            } else {
                return Err(anyhow::anyhow!(
                    "v1 endpoint '{}' requires a Configuration Key.\n\
                     Set HONEYCOMB_CONFIGURATION_API_KEY environment variable.",
                    path
                ));
            }
        }

        request = request.header("Content-Type", "application/json");

        // Add query parameters
        if let Some(params) = query_params {
            for (key, value) in params {
                request = request.query(&[(key, value)]);
            }
        }

        // Add body for POST/PUT/PATCH requests
        if let Some(body) = body {
            request = request.json(body);
        }

        request
            .send()
            .await
            .with_context(|| format!("Failed to send request to {}", url))
    }

    pub async fn get(
        &self,
        path: &str,
        query_params: Option<&HashMap<String, String>>,
    ) -> Result<Value> {
        let response = self.request(Method::GET, path, query_params, None).await?;
        self.handle_response(response).await
    }

    pub async fn post(&self, path: &str, body: &Value) -> Result<Value> {
        let response = self.request(Method::POST, path, None, Some(body)).await?;
        self.handle_response(response).await
    }

    pub async fn put(&self, path: &str, body: &Value) -> Result<Value> {
        let response = self.request(Method::PUT, path, None, Some(body)).await?;
        self.handle_response(response).await
    }

    pub async fn patch(&self, path: &str, body: &Value) -> Result<Value> {
        let response = self.request(Method::PATCH, path, None, Some(body)).await?;
        self.handle_response(response).await
    }

    pub async fn delete(&self, path: &str) -> Result<()> {
        let response = self.request(Method::DELETE, path, None, None).await?;
        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Request failed with status {}: {}", status, text);
        }
    }

    async fn handle_response(&self, response: Response) -> Result<Value> {
        let status = response.status();
        let text = response.text().await?;

        if status.is_success() {
            if text.is_empty() {
                Ok(Value::Null)
            } else {
                serde_json::from_str(&text)
                    .with_context(|| format!("Failed to parse JSON response: {}", text))
            }
        } else {
            anyhow::bail!("Request failed with status {}: {}", status, text);
        }
    }

    /// Check if an endpoint is a v2 endpoint (uses Management Key)
    pub fn is_v2_endpoint(&self, path: &str) -> bool {
        path.starts_with("/2/")
    }

    /// Check if we have a valid Management Key
    pub fn has_management_key(&self) -> bool {
        self.management_key.is_some()
    }

    /// Check if we have a valid Configuration Key
    pub fn has_config_key(&self) -> bool {
        self.config_key.is_some()
    }
}
