use crate::client::HoneycombClient;
use crate::common::{pretty_print_json, read_json_file, OutputFormat, DEFAULT_TABLE_FORMAT, DEFAULT_PRETTY_FORMAT, CommandContext};
use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Subcommand)]
pub enum BurnAlertCommands {
    /// List all burn alerts in a dataset
    List {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Environment slug (optional, uses HONEYCOMB_ENVIRONMENT env var if not specified)
        #[arg(short, long, env = "HONEYCOMB_ENVIRONMENT")]
        environment: Option<String>,
        /// Output format
        #[arg(short, long, default_value = DEFAULT_TABLE_FORMAT)]
        format: OutputFormat,
    },
    /// Get a specific burn alert
    Get {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Burn Alert ID
        #[arg(short, long)]
        id: String,
        /// Output format
        #[arg(short, long, default_value = DEFAULT_PRETTY_FORMAT)]
        format: OutputFormat,
    },
    /// Create a new burn alert
    Create {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Burn alert data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = DEFAULT_PRETTY_FORMAT)]
        format: OutputFormat,
    },
    /// Update a burn alert
    Update {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Burn Alert ID
        #[arg(short, long)]
        id: String,
        /// Burn alert data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = DEFAULT_PRETTY_FORMAT)]
        format: OutputFormat,
    },
    /// Delete a burn alert
    Delete {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Burn Alert ID
        #[arg(short, long)]
        id: String,
    },
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BurnAlert {
    pub id: String,
    pub slo_id: String,
    pub exhaustion_minutes: i32,
    pub budget_rate_window_minutes: i32,
    pub budget_rate_decrease_threshold: f64,
    pub recipients: Vec<String>,
    pub disabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl BurnAlertCommands {
    pub async fn execute(&self, client: &HoneycombClient, context: &CommandContext) -> Result<()> {
        match self {
            BurnAlertCommands::List {
                dataset,
                environment,
                format,
            } => list_burn_alerts(client, dataset, environment.as_deref(), format).await,
            BurnAlertCommands::Get {
                dataset,
                id,
                format,
            } => get_burn_alert(client, dataset, id, format).await,
            BurnAlertCommands::Create {
                dataset,
                data,
                format,
            } => create_burn_alert(client, dataset, data, format).await,
            BurnAlertCommands::Update {
                dataset,
                id,
                data,
                format,
            } => update_burn_alert(client, dataset, id, data, format).await,
            BurnAlertCommands::Delete { dataset, id } => {
                delete_burn_alert(client, dataset, id).await
            }
        }
    }
}

async fn list_burn_alerts(
    client: &HoneycombClient,
    dataset: &str,
    environment: Option<&str>,
    format: &OutputFormat,
) -> Result<()> {
    use crate::common::require_valid_environment;
    use std::collections::HashMap;

    // If environment is provided, validate it exists
    if let Some(env) = environment {
        let team = std::env::var("HONEYCOMB_TEAM").unwrap_or_else(|_| "default".to_string());
        require_valid_environment(client, &team, env).await?;
    }

    let path = format!("/1/burn_alerts/{}", dataset);

    // Add environment as query parameter if provided
    let mut query_params = HashMap::new();
    if let Some(env) = environment {
        query_params.insert("environment".to_string(), env.to_string());
    }

    let response = client
        .get(
            &path,
            if query_params.is_empty() {
                None
            } else {
                Some(&query_params)
            },
        )
        .await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(&response)?);
        }
        OutputFormat::Pretty => {
            println!("{}", pretty_print_json(&response)?);
        }
        OutputFormat::Table => {
            if let Value::Array(burn_alerts) = response {
                println!(
                    "{:<15} {:<15} {:<15} {:<15} {:<10} Recipients",
                    "ID", "SLO ID", "Exhaustion", "Window", "Disabled"
                );
                println!("{:-<85}", "");

                for burn_alert in burn_alerts {
                    if let Ok(ba) = serde_json::from_value::<BurnAlert>(burn_alert) {
                        println!(
                            "{:<15} {:<15} {:<15} {:<15} {:<10} {}",
                            ba.id,
                            ba.slo_id,
                            format!("{}m", ba.exhaustion_minutes),
                            format!("{}m", ba.budget_rate_window_minutes),
                            ba.disabled,
                            ba.recipients.len()
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

async fn get_burn_alert(
    client: &HoneycombClient,
    dataset: &str,
    id: &str,
    format: &OutputFormat,
) -> Result<()> {
    let path = format!("/1/burn_alerts/{}/{}", dataset, id);
    let response = client.get(&path, None).await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(&response)?);
        }
        OutputFormat::Pretty | OutputFormat::Table => {
            println!("{}", pretty_print_json(&response)?);
        }
    }

    Ok(())
}

async fn create_burn_alert(
    client: &HoneycombClient,
    dataset: &str,
    data: &str,
    format: &OutputFormat,
) -> Result<()> {
    let json_data = if std::path::Path::new(data).exists() {
        read_json_file(data)?
    } else {
        serde_json::from_str(data)?
    };

    let path = format!("/1/burn_alerts/{}", dataset);
    let response = client.post(&path, &json_data).await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(&response)?);
        }
        OutputFormat::Pretty | OutputFormat::Table => {
            println!("{}", pretty_print_json(&response)?);
        }
    }

    Ok(())
}

async fn update_burn_alert(
    client: &HoneycombClient,
    dataset: &str,
    id: &str,
    data: &str,
    format: &OutputFormat,
) -> Result<()> {
    let json_data = if std::path::Path::new(data).exists() {
        read_json_file(data)?
    } else {
        serde_json::from_str(data)?
    };

    let path = format!("/1/burn_alerts/{}/{}", dataset, id);
    let response = client.put(&path, &json_data).await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(&response)?);
        }
        OutputFormat::Pretty | OutputFormat::Table => {
            println!("{}", pretty_print_json(&response)?);
        }
    }

    Ok(())
}

async fn delete_burn_alert(client: &HoneycombClient, dataset: &str, id: &str) -> Result<()> {
    let path = format!("/1/burn_alerts/{}/{}", dataset, id);
    client.delete(&path).await?;

    println!(
        "Burn Alert '{}' in dataset '{}' deleted successfully",
        id, dataset
    );

    Ok(())
}
