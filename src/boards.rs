use crate::client::HoneycombClient;
use crate::common::{
    CommandContext, DEFAULT_PRETTY_FORMAT, DEFAULT_TABLE_FORMAT, OutputFormat, pretty_print_json,
    read_json_file,
};
use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Subcommand)]
pub enum BoardCommands {
    /// List all boards
    List {
        /// Environment slug (optional, uses HONEYCOMB_ENVIRONMENT env var if not specified)
        #[arg(short, long, env = "HONEYCOMB_ENVIRONMENT")]
        environment: Option<String>,
        /// Output format
        #[arg(short, long, default_value = DEFAULT_TABLE_FORMAT)]
        format: OutputFormat,
    },
    /// Get a specific board
    Get {
        /// Board ID
        #[arg(short, long)]
        id: String,
        /// Output format
        #[arg(short, long, default_value = DEFAULT_PRETTY_FORMAT)]
        format: OutputFormat,
    },
    /// Create a new board
    Create {
        /// Board data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = DEFAULT_PRETTY_FORMAT)]
        format: OutputFormat,
    },
    /// Update a board
    Update {
        /// Board ID
        #[arg(short, long)]
        id: String,
        /// Board data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = DEFAULT_PRETTY_FORMAT)]
        format: OutputFormat,
    },
    /// Delete a board
    Delete {
        /// Board ID
        #[arg(short, long)]
        id: String,
    },
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Board {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub style: String,
    pub queries: Vec<BoardQuery>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BoardQuery {
    pub query_id: String,
    pub dataset: String,
    pub query_style: String,
    pub graphic_settings: Option<Value>,
}

impl BoardCommands {
    pub async fn execute(&self, client: &HoneycombClient, _context: &CommandContext) -> Result<()> {
        match self {
            BoardCommands::List {
                environment,
                format,
            } => list_boards(client, environment, format).await,
            BoardCommands::Get { id, format } => get_board(client, id, format).await,
            BoardCommands::Create { data, format } => create_board(client, data, format).await,
            BoardCommands::Update { id, data, format } => {
                update_board(client, id, data, format).await
            }
            BoardCommands::Delete { id } => delete_board(client, id).await,
        }
    }
}

async fn list_boards(
    client: &HoneycombClient,
    environment: &Option<String>,
    format: &OutputFormat,
) -> Result<()> {
    use crate::common::require_valid_environment;
    use std::collections::HashMap;

    // If environment is provided, validate it exists
    if let Some(env) = environment {
        let team = std::env::var("HONEYCOMB_TEAM").unwrap_or_else(|_| "default".to_string());
        require_valid_environment(client, &team, env).await?;
    }

    let path = "/1/boards";

    // Add environment as query parameter if provided
    let mut query_params = HashMap::new();
    if let Some(env) = environment {
        query_params.insert("environment".to_string(), env.to_string());
    }

    let response = client
        .get(
            path,
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
            if let Value::Array(boards) = response {
                println!(
                    "{:<15} {:<30} {:<10} {:<20} Created",
                    "ID", "Name", "Queries", "Style"
                );
                println!("{:-<85}", "");

                for board in boards {
                    if let Ok(b) = serde_json::from_value::<Board>(board) {
                        println!(
                            "{:<15} {:<30} {:<10} {:<20} {}",
                            b.id,
                            b.name,
                            b.queries.len(),
                            b.style,
                            b.created_at.format("%Y-%m-%d")
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

async fn get_board(client: &HoneycombClient, id: &str, format: &OutputFormat) -> Result<()> {
    let path = format!("/1/boards/{}", id);
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

async fn create_board(client: &HoneycombClient, data: &str, format: &OutputFormat) -> Result<()> {
    let json_data = if std::path::Path::new(data).exists() {
        read_json_file(data)?
    } else {
        serde_json::from_str(data)?
    };

    let response = client.post("/1/boards", &json_data).await?;

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

async fn update_board(
    client: &HoneycombClient,
    id: &str,
    data: &str,
    format: &OutputFormat,
) -> Result<()> {
    let json_data = if std::path::Path::new(data).exists() {
        read_json_file(data)?
    } else {
        serde_json::from_str(data)?
    };

    let path = format!("/1/boards/{}", id);
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

async fn delete_board(client: &HoneycombClient, id: &str) -> Result<()> {
    let path = format!("/1/boards/{}", id);
    client.delete(&path).await?;

    println!("Board '{}' deleted successfully", id);

    Ok(())
}
