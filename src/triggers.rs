use crate::client::HoneycombClient;
use crate::common::{pretty_print_json, read_json_file, OutputFormat, DEFAULT_TABLE_FORMAT, DEFAULT_PRETTY_FORMAT, CommandContext};
use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Subcommand)]
pub enum TriggerCommands {
    /// List all triggers in a dataset
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
    /// Get a specific trigger
    Get {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Trigger ID
        #[arg(short, long)]
        id: String,
        /// Output format
        #[arg(short, long, default_value = DEFAULT_PRETTY_FORMAT)]
        format: OutputFormat,
    },
    /// Create a new trigger
    Create {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Trigger data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = DEFAULT_PRETTY_FORMAT)]
        format: OutputFormat,
    },
    /// Update a trigger
    Update {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Trigger ID
        #[arg(short, long)]
        id: String,
        /// Trigger data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = DEFAULT_PRETTY_FORMAT)]
        format: OutputFormat,
    },
    /// Delete a trigger
    Delete {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Trigger ID
        #[arg(short, long)]
        id: String,
    },
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Trigger {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub query: TriggerQuery,
    pub disabled: bool,
    pub alert_type: String,
    pub threshold: TriggerThreshold,
    pub recipients: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TriggerQuery {
    pub query_id: Option<String>,
    pub calculations: Vec<Calculation>,
    pub filters: Vec<Filter>,
    pub time_range: i64,
    pub granularity: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Calculation {
    pub op: String,
    pub column: Option<String>,
    pub alias: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Filter {
    pub column: String,
    pub op: String,
    pub value: Value,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TriggerThreshold {
    pub op: String,
    pub value: f64,
    pub frequency: i32,
}

impl TriggerCommands {
    pub async fn execute(&self, client: &HoneycombClient, context: &CommandContext) -> Result<()> {
        match self {
            TriggerCommands::List {
                dataset,
                environment,
                format,
            } => list_triggers(client, dataset, environment.as_deref(), format).await,
            TriggerCommands::Get {
                dataset,
                id,
                format,
            } => get_trigger(client, dataset, id, format).await,
            TriggerCommands::Create {
                dataset,
                data,
                format,
            } => create_trigger(client, dataset, data, format).await,
            TriggerCommands::Update {
                dataset,
                id,
                data,
                format,
            } => update_trigger(client, dataset, id, data, format).await,
            TriggerCommands::Delete { dataset, id } => delete_trigger(client, dataset, id).await,
        }
    }
}

async fn list_triggers(
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

    let path = format!("/1/triggers/{}", dataset);

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
            if let Value::Array(triggers) = response {
                println!(
                    "{:<15} {:<30} {:<10} {:<15} {:<20} Recipients",
                    "ID", "Name", "Disabled", "Alert Type", "Created"
                );
                println!("{:-<100}", "");

                for trigger in triggers {
                    if let Ok(trig) = serde_json::from_value::<Trigger>(trigger) {
                        println!(
                            "{:<15} {:<30} {:<10} {:<15} {:<20} {}",
                            trig.id,
                            trig.name,
                            trig.disabled,
                            trig.alert_type,
                            trig.created_at.format("%Y-%m-%d"),
                            trig.recipients.len()
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

async fn get_trigger(
    client: &HoneycombClient,
    dataset: &str,
    id: &str,
    format: &OutputFormat,
) -> Result<()> {
    let path = format!("/1/triggers/{}/{}", dataset, id);
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

async fn create_trigger(
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

    let path = format!("/1/triggers/{}", dataset);
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

async fn update_trigger(
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

    let path = format!("/1/triggers/{}/{}", dataset, id);
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

async fn delete_trigger(client: &HoneycombClient, dataset: &str, id: &str) -> Result<()> {
    let path = format!("/1/triggers/{}/{}", dataset, id);
    client.delete(&path).await?;

    println!(
        "Trigger '{}' in dataset '{}' deleted successfully",
        id, dataset
    );

    Ok(())
}
