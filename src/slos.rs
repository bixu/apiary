use crate::client::HoneycombClient;
use crate::common::{pretty_print_json, read_json_file, OutputFormat};
use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Subcommand)]
pub enum SloCommands {
    /// List all SLOs in a dataset
    List {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Output format
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,
    },
    /// Get a specific SLO
    Get {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// SLO ID
        #[arg(short, long)]
        id: String,
        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: OutputFormat,
    },
    /// Create a new SLO
    Create {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// SLO data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: OutputFormat,
    },
    /// Update an SLO
    Update {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// SLO ID
        #[arg(short, long)]
        id: String,
        /// SLO data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: OutputFormat,
    },
    /// Delete an SLO
    Delete {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// SLO ID
        #[arg(short, long)]
        id: String,
    },
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Slo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub sli: SloIndicator,
    pub target_percentage: f64,
    pub time_period: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SloIndicator {
    pub query: SloQuery,
    pub alias: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SloQuery {
    pub query_id: Option<String>,
    pub calculations: Vec<SloCalculation>,
    pub filters: Vec<SloFilter>,
    pub time_range: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SloCalculation {
    pub op: String,
    pub column: Option<String>,
    pub alias: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SloFilter {
    pub column: String,
    pub op: String,
    pub value: Value,
}

impl SloCommands {
    pub async fn execute(&self, client: &HoneycombClient) -> Result<()> {
        match self {
            SloCommands::List { dataset, format } => list_slos(client, dataset, format).await,
            SloCommands::Get {
                dataset,
                id,
                format,
            } => get_slo(client, dataset, id, format).await,
            SloCommands::Create {
                dataset,
                data,
                format,
            } => create_slo(client, dataset, data, format).await,
            SloCommands::Update {
                dataset,
                id,
                data,
                format,
            } => update_slo(client, dataset, id, data, format).await,
            SloCommands::Delete { dataset, id } => delete_slo(client, dataset, id).await,
        }
    }
}

async fn list_slos(client: &HoneycombClient, dataset: &str, format: &OutputFormat) -> Result<()> {
    let path = format!("/1/slos/{}", dataset);
    let response = client.get(&path, None).await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(&response)?);
        }
        OutputFormat::Pretty => {
            println!("{}", pretty_print_json(&response)?);
        }
        OutputFormat::Table => {
            if let Value::Array(slos) = response {
                println!(
                    "{:<15} {:<30} {:<15} {:<15} {}",
                    "ID", "Name", "Target %", "Time Period", "Created"
                );
                println!("{:-<85}", "");

                for slo in slos {
                    if let Ok(s) = serde_json::from_value::<Slo>(slo) {
                        println!(
                            "{:<15} {:<30} {:<15} {:<15} {}",
                            s.id,
                            s.name,
                            format!("{:.1}%", s.target_percentage),
                            format!("{}d", s.time_period),
                            s.created_at.format("%Y-%m-%d")
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

async fn get_slo(
    client: &HoneycombClient,
    dataset: &str,
    id: &str,
    format: &OutputFormat,
) -> Result<()> {
    let path = format!("/1/slos/{}/{}", dataset, id);
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

async fn create_slo(
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

    let path = format!("/1/slos/{}", dataset);
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

async fn update_slo(
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

    let path = format!("/1/slos/{}/{}", dataset, id);
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

async fn delete_slo(client: &HoneycombClient, dataset: &str, id: &str) -> Result<()> {
    let path = format!("/1/slos/{}/{}", dataset, id);
    client.delete(&path).await?;

    println!("SLO '{}' in dataset '{}' deleted successfully", id, dataset);

    Ok(())
}
