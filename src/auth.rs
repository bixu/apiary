use crate::client::HoneycombClient;
use crate::common::{CommandContext, DEFAULT_PRETTY_FORMAT, OutputFormat};
use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};

#[derive(Subcommand)]
pub enum AuthCommands {
    /// Validate an API key and get authentication information
    Validate {
        /// Output format
        #[arg(short, long, default_value = DEFAULT_PRETTY_FORMAT)]
        format: OutputFormat,
    },
    /// Show information about the current API key type and requirements
    Info,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthResponse {
    pub data: AuthData,
    pub included: Vec<TeamInfo>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthData {
    pub attributes: AuthAttributes,
    pub relationships: AuthRelationships,
    #[serde(rename = "type")]
    pub data_type: String,
    pub id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthAttributes {
    pub name: String,
    pub key_type: String,
    pub scopes: Vec<String>,
    pub disabled: bool,
    pub timestamps: AuthTimestamps,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthTimestamps {
    pub created: String,
    pub updated: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthRelationships {
    pub team: TeamRelation,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TeamRelation {
    pub data: TeamData,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TeamData {
    #[serde(rename = "type")]
    pub team_type: String,
    pub id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TeamInfo {
    pub attributes: TeamAttributes,
    #[serde(rename = "type")]
    pub team_type: String,
    pub id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TeamAttributes {
    pub name: String,
    pub slug: String,
}

impl AuthCommands {
    pub async fn execute(&self, client: &HoneycombClient, _context: &CommandContext) -> Result<()> {
        match self {
            AuthCommands::Validate { format } => validate_auth(client, format).await,
            AuthCommands::Info => show_key_info(client).await,
        }
    }
}

async fn validate_auth(client: &HoneycombClient, format: &OutputFormat) -> Result<()> {
    let response = client.get("/2/auth", None).await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(&response)?);
        }
        OutputFormat::Pretty => {
            println!("{}", serde_json::to_string_pretty(&response)?);
        }
        OutputFormat::Table => {
            // Parse the v2 auth response
            if let Ok(auth_response) = serde_json::from_value::<AuthResponse>(response.clone()) {
                println!("API Key Information:");
                println!("==================");
                println!("Name: {}", auth_response.data.attributes.name);
                println!("Type: {}", auth_response.data.attributes.key_type);
                println!("ID: {}", auth_response.data.id);
                println!(
                    "Status: {}",
                    if auth_response.data.attributes.disabled {
                        "Disabled"
                    } else {
                        "Active"
                    }
                );
                println!(
                    "Created: {}",
                    auth_response.data.attributes.timestamps.created
                );
                println!(
                    "Updated: {}",
                    auth_response.data.attributes.timestamps.updated
                );
                println!();

                if let Some(team) = auth_response.included.first() {
                    println!("Team Information:");
                    println!("=================");
                    println!("Name: {}", team.attributes.name);
                    println!("Slug: {}", team.attributes.slug);
                    println!("ID: {}", team.id);
                    println!();
                }

                println!("Scopes:");
                println!("=======");
                for scope in &auth_response.data.attributes.scopes {
                    println!("  • {}", scope);
                }
            } else {
                println!("{}", serde_json::to_string_pretty(&response)?);
            }
        }
    }

    Ok(())
}

async fn show_key_info(client: &HoneycombClient) -> Result<()> {
    println!("🔑 API Key Information");
    println!("======================");

    if client.has_management_key() {
        if let Some(mgmt_key) = client.get_key_for_endpoint("/2/auth") {
            println!(
                "Management Key: {} (for v2 endpoints)",
                &mgmt_key[..8.min(mgmt_key.len())]
            );
            println!("✅ Can access v2 APIs (Bearer authentication)");
        }
    } else {
        println!("❌ No Management Key - cannot access v2 APIs");
    }

    println!();

    if client.has_config_key() {
        if let Some(config_key) = client.get_key_for_endpoint("/1/datasets") {
            println!(
                "Configuration Key: {} (for v1 endpoints)",
                &config_key[..8.min(config_key.len())]
            );
            println!("✅ Can access v1 APIs (X-Honeycomb-Team authentication)");
        }
    } else {
        println!("❌ No Configuration Key - cannot access v1 APIs");
    }

    println!();
    println!("💡 Set environment variables:");
    println!("   export HONEYCOMB_MANAGEMENT_API_KEY_ID=\"hc***\"");
    println!("   export HONEYCOMB_MANAGEMENT_API_KEY=\"***\"");
    println!("   export HONEYCOMB_CONFIGURATION_API_KEY=\"***\"");

    Ok(())
}
