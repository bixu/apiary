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
        /// Environment slug (uses HONEYCOMB_ENVIRONMENT env var by default, can be overridden with this flag)
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
    #[serde(rename = "type")]
    pub board_type: Option<String>,
    pub panels: Option<Vec<Value>>,
    pub preset_filters: Option<Vec<Value>>,
    pub links: Option<Value>,
    // Legacy fields (for backwards compatibility with older API responses)
    #[serde(default)]
    pub style: Option<String>,
    #[serde(default)]
    pub queries: Vec<BoardQuery>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
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

    // Get environment from flag or env var (clap handles env var automatically via env = "HONEYCOMB_ENVIRONMENT")
    let env = environment.as_ref().ok_or_else(|| {
        anyhow::anyhow!(
            "Environment is required. Set HONEYCOMB_ENVIRONMENT environment variable or use --environment flag."
        )
    })?;

    // Validate environment exists
    let team = std::env::var("HONEYCOMB_TEAM").unwrap_or_else(|_| "default".to_string());
    require_valid_environment(client, &team, env).await?;

    let path = "/1/boards";

    // Always add environment as query parameter
    let mut query_params = HashMap::new();
    query_params.insert("environment".to_string(), env.to_string());

    let response = client.get(path, Some(&query_params)).await?;

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
                    "{:<15} {:<40} {:<10} {:<15}",
                    "ID", "Name", "Panels", "Type"
                );
                println!("{:-<80}", "");

                for board in boards {
                    if let Ok(b) = serde_json::from_value::<Board>(board.clone()) {
                        let panel_count = b
                            .panels
                            .as_ref()
                            .map(|p| p.len())
                            .or(if !b.queries.is_empty() {
                                Some(b.queries.len())
                            } else {
                                None
                            })
                            .unwrap_or(0);
                        let board_type = b
                            .board_type
                            .as_deref()
                            .or(b.style.as_deref())
                            .unwrap_or("unknown");
                        let name = if let Some(desc) = &b.description {
                            format!("{} - {}", b.name, desc)
                        } else {
                            b.name
                        };
                        // Truncate name if too long
                        let display_name = if name.len() > 38 {
                            format!("{}...", &name[..35])
                        } else {
                            name
                        };
                        println!(
                            "{:<15} {:<40} {:<10} {:<15}",
                            b.id, display_name, panel_count, board_type
                        );
                    } else {
                        // Fallback: try to extract basic fields from raw JSON
                        if let Value::Object(obj) = board {
                            let id = obj.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
                            let name = obj
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown");
                            let panel_count = obj
                                .get("panels")
                                .and_then(|v| v.as_array())
                                .map(|a| a.len())
                                .unwrap_or(0);
                            let board_type = obj
                                .get("type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown");
                            println!(
                                "{:<15} {:<40} {:<10} {:<15}",
                                id, name, panel_count, board_type
                            );
                        }
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
