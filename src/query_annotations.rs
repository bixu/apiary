use crate::client::HoneycombClient;
use crate::common::{OutputFormat, pretty_print_json, read_json_file};
use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Subcommand)]
pub enum QueryAnnotationCommands {
    /// List all query annotations in a dataset
    List {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Output format
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,
    },
    /// Get a specific query annotation
    Get {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Query Annotation ID
        #[arg(short, long)]
        id: String,
        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: OutputFormat,
    },
    /// Create a new query annotation
    Create {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Query annotation data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: OutputFormat,
    },
    /// Update a query annotation
    Update {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Query Annotation ID
        #[arg(short, long)]
        id: String,
        /// Query annotation data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: OutputFormat,
    },
    /// Delete a query annotation
    Delete {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Query Annotation ID
        #[arg(short, long)]
        id: String,
    },
}

#[derive(Deserialize, Serialize, Debug)]
pub struct QueryAnnotation {
    pub id: String,
    pub query_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl QueryAnnotationCommands {
    pub async fn execute(&self, client: &HoneycombClient) -> Result<()> {
        match self {
            QueryAnnotationCommands::List { dataset, format } => {
                list_query_annotations(client, dataset, format).await
            }
            QueryAnnotationCommands::Get { dataset, id, format } => {
                get_query_annotation(client, dataset, id, format).await
            }
            QueryAnnotationCommands::Create { dataset, data, format } => {
                create_query_annotation(client, dataset, data, format).await
            }
            QueryAnnotationCommands::Update { dataset, id, data, format } => {
                update_query_annotation(client, dataset, id, data, format).await
            }
            QueryAnnotationCommands::Delete { dataset, id } => {
                delete_query_annotation(client, dataset, id).await
            }
        }
    }
}

async fn list_query_annotations(client: &HoneycombClient, dataset: &str, format: &OutputFormat) -> Result<()> {
    let path = format!("/1/query_annotations/{}", dataset);
    let response = client.get(&path, None).await?;
    
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(&response)?);
        }
        OutputFormat::Pretty => {
            println!("{}", pretty_print_json(&response)?);
        }
        OutputFormat::Table => {
            if let Value::Array(annotations) = response {
                println!("{:<15} {:<15} {:<30} {}", "ID", "Query ID", "Name", "Created");
                println!("{:-<75}", "");
                
                for annotation in annotations {
                    if let Ok(qa) = serde_json::from_value::<QueryAnnotation>(annotation) {
                        println!("{:<15} {:<15} {:<30} {}", 
                            qa.id,
                            qa.query_id,
                            qa.name,
                            qa.created_at.format("%Y-%m-%d")
                        );
                    }
                }
            }
        }
    }
    
    Ok(())
}

async fn get_query_annotation(client: &HoneycombClient, dataset: &str, id: &str, format: &OutputFormat) -> Result<()> {
    let path = format!("/1/query_annotations/{}/{}", dataset, id);
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

async fn create_query_annotation(client: &HoneycombClient, dataset: &str, data: &str, format: &OutputFormat) -> Result<()> {
    let json_data = if std::path::Path::new(data).exists() {
        read_json_file(data)?
    } else {
        serde_json::from_str(data)?
    };
    
    let path = format!("/1/query_annotations/{}", dataset);
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

async fn update_query_annotation(client: &HoneycombClient, dataset: &str, id: &str, data: &str, format: &OutputFormat) -> Result<()> {
    let json_data = if std::path::Path::new(data).exists() {
        read_json_file(data)?
    } else {
        serde_json::from_str(data)?
    };
    
    let path = format!("/1/query_annotations/{}/{}", dataset, id);
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

async fn delete_query_annotation(client: &HoneycombClient, dataset: &str, id: &str) -> Result<()> {
    let path = format!("/1/query_annotations/{}/{}", dataset, id);
    client.delete(&path).await?;
    
    println!("Query annotation '{}' in dataset '{}' deleted successfully", id, dataset);
    
    Ok(())
}