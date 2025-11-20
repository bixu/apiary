use crate::client::HoneycombClient;
use crate::common::{OutputFormat, pretty_print_json, read_json_file};
use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Subcommand)]
pub enum BoardCommands {
    /// List all boards
    List {
        /// Output format
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,
    },
    /// Get a specific board
    Get {
        /// Board ID
        #[arg(short, long)]
        id: String,
        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: OutputFormat,
    },
    /// Create a new board
    Create {
        /// Board data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = "pretty")]
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
        #[arg(short, long, default_value = "pretty")]
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
    pub async fn execute(&self, client: &HoneycombClient) -> Result<()> {
        match self {
            BoardCommands::List { format } => {
                list_boards(client, format).await
            }
            BoardCommands::Get { id, format } => {
                get_board(client, id, format).await
            }
            BoardCommands::Create { data, format } => {
                create_board(client, data, format).await
            }
            BoardCommands::Update { id, data, format } => {
                update_board(client, id, data, format).await
            }
            BoardCommands::Delete { id } => {
                delete_board(client, id).await
            }
        }
    }
}

async fn list_boards(client: &HoneycombClient, format: &OutputFormat) -> Result<()> {
    let response = client.get("/1/boards", None).await?;
    
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(&response)?);
        }
        OutputFormat::Pretty => {
            println!("{}", pretty_print_json(&response)?);
        }
        OutputFormat::Table => {
            if let Value::Array(boards) = response {
                println!("{:<15} {:<30} {:<10} {:<20} {}", "ID", "Name", "Queries", "Style", "Created");
                println!("{:-<85}", "");
                
                for board in boards {
                    if let Ok(b) = serde_json::from_value::<Board>(board) {
                        println!("{:<15} {:<30} {:<10} {:<20} {}", 
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

async fn update_board(client: &HoneycombClient, id: &str, data: &str, format: &OutputFormat) -> Result<()> {
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