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
pub enum RecipientCommands {
    /// List all recipients
    List {
        /// Output format
        #[arg(short, long, default_value = DEFAULT_TABLE_FORMAT)]
        format: OutputFormat,
    },
    /// Get a specific recipient
    Get {
        /// Recipient ID
        #[arg(short, long)]
        id: String,
        /// Output format
        #[arg(short, long, default_value = DEFAULT_PRETTY_FORMAT)]
        format: OutputFormat,
    },
    /// Create a new recipient
    Create {
        /// Recipient data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = DEFAULT_PRETTY_FORMAT)]
        format: OutputFormat,
    },
    /// Update a recipient
    Update {
        /// Recipient ID
        #[arg(short, long)]
        id: String,
        /// Recipient data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = DEFAULT_PRETTY_FORMAT)]
        format: OutputFormat,
    },
    /// Delete a recipient
    Delete {
        /// Recipient ID
        #[arg(short, long)]
        id: String,
    },
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Recipient {
    pub id: String,
    pub name: String,
    pub target: String,
    #[serde(rename = "type")]
    pub recipient_type: String,
    pub details: Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl RecipientCommands {
    pub async fn execute(&self, client: &HoneycombClient, _context: &CommandContext) -> Result<()> {
        match self {
            RecipientCommands::List { format } => list_recipients(client, format).await,
            RecipientCommands::Get { id, format } => get_recipient(client, id, format).await,
            RecipientCommands::Create { data, format } => {
                create_recipient(client, data, format).await
            }
            RecipientCommands::Update { id, data, format } => {
                update_recipient(client, id, data, format).await
            }
            RecipientCommands::Delete { id } => delete_recipient(client, id).await,
        }
    }
}

async fn list_recipients(client: &HoneycombClient, format: &OutputFormat) -> Result<()> {
    let response = client.get("/1/recipients", None).await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(&response)?);
        }
        OutputFormat::Pretty => {
            println!("{}", pretty_print_json(&response)?);
        }
        OutputFormat::Table => {
            if let Value::Array(recipients) = response {
                println!(
                    "{:<15} {:<30} {:<15} {:<30} Created",
                    "ID", "Name", "Type", "Target"
                );
                println!("{:-<95}", "");

                for recipient in recipients {
                    if let Ok(r) = serde_json::from_value::<Recipient>(recipient) {
                        println!(
                            "{:<15} {:<30} {:<15} {:<30} {}",
                            r.id,
                            r.name,
                            r.recipient_type,
                            r.target,
                            r.created_at.format("%Y-%m-%d")
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

async fn get_recipient(client: &HoneycombClient, id: &str, format: &OutputFormat) -> Result<()> {
    let path = format!("/1/recipients/{}", id);
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

async fn create_recipient(
    client: &HoneycombClient,
    data: &str,
    format: &OutputFormat,
) -> Result<()> {
    let json_data = if std::path::Path::new(data).exists() {
        read_json_file(data)?
    } else {
        serde_json::from_str(data)?
    };

    let response = client.post("/1/recipients", &json_data).await?;

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

async fn update_recipient(
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

    let path = format!("/1/recipients/{}", id);
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

async fn delete_recipient(client: &HoneycombClient, id: &str) -> Result<()> {
    let path = format!("/1/recipients/{}", id);
    client.delete(&path).await?;

    println!("Recipient '{}' deleted successfully", id);

    Ok(())
}
