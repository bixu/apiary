use crate::client::HoneycombClient;
use crate::common::{pretty_print_json, read_json_file, OutputFormat};
use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Subcommand)]
pub enum CalculatedFieldCommands {
    /// List all calculated fields in a dataset
    List {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Output format
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,
    },
    /// Get a specific calculated field
    Get {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Calculated Field ID
        #[arg(short, long)]
        id: String,
        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: OutputFormat,
    },
    /// Create a new calculated field
    Create {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Calculated field data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: OutputFormat,
    },
    /// Update a calculated field
    Update {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Calculated Field ID
        #[arg(short, long)]
        id: String,
        /// Calculated field data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: OutputFormat,
    },
    /// Delete a calculated field
    Delete {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Calculated Field ID
        #[arg(short, long)]
        id: String,
    },
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CalculatedField {
    pub id: String,
    pub alias: String,
    pub description: Option<String>,
    pub expression: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl CalculatedFieldCommands {
    pub async fn execute(&self, client: &HoneycombClient) -> Result<()> {
        match self {
            CalculatedFieldCommands::List { dataset, format } => {
                list_calculated_fields(client, dataset, format).await
            }
            CalculatedFieldCommands::Get {
                dataset,
                id,
                format,
            } => get_calculated_field(client, dataset, id, format).await,
            CalculatedFieldCommands::Create {
                dataset,
                data,
                format,
            } => create_calculated_field(client, dataset, data, format).await,
            CalculatedFieldCommands::Update {
                dataset,
                id,
                data,
                format,
            } => update_calculated_field(client, dataset, id, data, format).await,
            CalculatedFieldCommands::Delete { dataset, id } => {
                delete_calculated_field(client, dataset, id).await
            }
        }
    }
}

async fn list_calculated_fields(
    client: &HoneycombClient,
    dataset: &str,
    format: &OutputFormat,
) -> Result<()> {
    let path = format!("/1/derived_columns/{}", dataset);
    let response = client.get(&path, None).await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(&response)?);
        }
        OutputFormat::Pretty => {
            println!("{}", pretty_print_json(&response)?);
        }
        OutputFormat::Table => {
            if let Value::Array(fields) = response {
                println!(
                    "{:<15} {:<30} {:<50} Created",
                    "ID", "Alias", "Expression"
                );
                println!("{:-<110}", "");

                for field in fields {
                    if let Ok(cf) = serde_json::from_value::<CalculatedField>(field) {
                        println!(
                            "{:<15} {:<30} {:<50} {}",
                            cf.id,
                            cf.alias,
                            cf.expression,
                            cf.created_at.format("%Y-%m-%d")
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

async fn get_calculated_field(
    client: &HoneycombClient,
    dataset: &str,
    id: &str,
    format: &OutputFormat,
) -> Result<()> {
    let path = format!("/1/derived_columns/{}/{}", dataset, id);
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

async fn create_calculated_field(
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

    let path = format!("/1/derived_columns/{}", dataset);
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

async fn update_calculated_field(
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

    let path = format!("/1/derived_columns/{}/{}", dataset, id);
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

async fn delete_calculated_field(client: &HoneycombClient, dataset: &str, id: &str) -> Result<()> {
    let path = format!("/1/derived_columns/{}/{}", dataset, id);
    client.delete(&path).await?;

    println!(
        "Calculated field '{}' in dataset '{}' deleted successfully",
        id, dataset
    );

    Ok(())
}
