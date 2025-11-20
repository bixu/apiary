use crate::client::HoneycombClient;
use crate::common::{OutputFormat, pretty_print_json, read_json_file};
use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};

#[derive(Subcommand)]
pub enum DatasetDefinitionCommands {
    /// Get dataset definitions
    Get {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: OutputFormat,
    },
    /// Update dataset definitions
    Update {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Dataset definition data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: OutputFormat,
    },
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DatasetDefinition {
    pub trace_id_field: Option<String>,
    pub parent_id_field: Option<String>,
    pub name_field: Option<String>,
    pub service_name_field: Option<String>,
    pub span_kind_field: Option<String>,
    pub status_code_field: Option<String>,
    pub duration_field: Option<String>,
    pub user_id_field: Option<String>,
    pub error_field: Option<String>,
    pub http_status_code_field: Option<String>,
    pub url_field: Option<String>,
    pub route_field: Option<String>,
}

impl DatasetDefinitionCommands {
    pub async fn execute(&self, client: &HoneycombClient) -> Result<()> {
        match self {
            DatasetDefinitionCommands::Get { dataset, format } => {
                get_dataset_definitions(client, dataset, format).await
            }
            DatasetDefinitionCommands::Update { dataset, data, format } => {
                update_dataset_definitions(client, dataset, data, format).await
            }
        }
    }
}

async fn get_dataset_definitions(client: &HoneycombClient, dataset: &str, format: &OutputFormat) -> Result<()> {
    let path = format!("/1/dataset_definitions/{}", dataset);
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

async fn update_dataset_definitions(client: &HoneycombClient, dataset: &str, data: &str, format: &OutputFormat) -> Result<()> {
    let json_data = if std::path::Path::new(data).exists() {
        read_json_file(data)?
    } else {
        serde_json::from_str(data)?
    };
    
    let path = format!("/1/dataset_definitions/{}", dataset);
    let response = client.patch(&path, &json_data).await?;
    
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