use crate::client::HoneycombClient;
use crate::common::{
    mask_identifier, CommandContext, ConfigurationKeyMaterial, ManagementKeyMaterial, OutputFormat,
    DEFAULT_TABLE_FORMAT,
};
use anyhow::Result;
use clap::Subcommand;
use serde::Serialize;

#[derive(Subcommand)]
pub enum ApiKeyCommands {
    /// Validate configured API keys without reaching into Honeycomb resources
    Validate {
        /// Output format
        #[arg(short, long, default_value = DEFAULT_TABLE_FORMAT)]
        format: OutputFormat,
    },
}

impl ApiKeyCommands {
    pub async fn execute(&self, client: &HoneycombClient, context: &CommandContext) -> Result<()> {
        match self {
            ApiKeyCommands::Validate { format } => validate_keys(client, context, format).await,
        }
    }
}

#[derive(Debug, Serialize)]
struct KeyValidationRow {
    #[serde(rename = "type")]
    key_type: &'static str,
    source: String,
    key_id: String,
    status: &'static str,
    details: String,
}

async fn validate_keys(
    client: &HoneycombClient,
    context: &CommandContext,
    format: &OutputFormat,
) -> Result<()> {
    let mut rows = Vec::new();
    rows.push(validate_management_key(client, context.key_material.management.as_ref()).await);
    rows.push(
        validate_configuration_key(client, context.key_material.configuration.as_ref()).await,
    );

    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&rows)?),
        OutputFormat::Pretty => print_pretty(&rows),
        OutputFormat::Table => print_table(&rows),
    }

    Ok(())
}

async fn validate_management_key(
    client: &HoneycombClient,
    material: Option<&ManagementKeyMaterial>,
) -> KeyValidationRow {
    match material {
        None => KeyValidationRow::not_configured("management"),
        Some(info) => {
            let source = info.source.to_string();
            let key_id = format_management_id(info);

            if !info.has_secret {
                return KeyValidationRow::invalid(
                    "management",
                    source,
                    key_id,
                    "Missing management key secret",
                );
            }

            if !client.has_management_key() {
                return KeyValidationRow::invalid(
                    "management",
                    source,
                    key_id,
                    "Management key not configured for client",
                );
            }

            match client.get("/2/auth", None).await {
                Ok(_) => {
                    KeyValidationRow::valid("management", source, key_id, "Validated via /2/auth")
                }
                Err(err) => {
                    KeyValidationRow::invalid("management", source, key_id, format!("{err}"))
                }
            }
        }
    }
}

async fn validate_configuration_key(
    client: &HoneycombClient,
    material: Option<&ConfigurationKeyMaterial>,
) -> KeyValidationRow {
    match material {
        None => KeyValidationRow::not_configured("configuration"),
        Some(info) => {
            let source = info.source.to_string();
            let key_id = mask_identifier(&info.id);

            if !client.has_config_key() {
                return KeyValidationRow::invalid(
                    "configuration",
                    source,
                    key_id,
                    "Configuration key not configured for client",
                );
            }

            match client.get("/1/auth", None).await {
                Ok(_) => KeyValidationRow::valid(
                    "configuration",
                    source,
                    key_id,
                    "Validated via /1/auth",
                ),
                Err(err) => {
                    KeyValidationRow::invalid("configuration", source, key_id, format!("{err}"))
                }
            }
        }
    }
}

impl KeyValidationRow {
    fn not_configured(key_type: &'static str) -> Self {
        Self {
            key_type,
            source: "-".to_string(),
            key_id: "-".to_string(),
            status: "not configured",
            details: "Set via env vars or flags to enable validation".to_string(),
        }
    }

    fn valid(
        key_type: &'static str,
        source: String,
        key_id: String,
        details: impl Into<String>,
    ) -> Self {
        Self {
            key_type,
            source,
            key_id,
            status: "valid",
            details: details.into(),
        }
    }

    fn invalid(
        key_type: &'static str,
        source: String,
        key_id: String,
        details: impl Into<String>,
    ) -> Self {
        Self {
            key_type,
            source,
            key_id,
            status: "invalid",
            details: details.into(),
        }
    }
}

fn print_table(rows: &[KeyValidationRow]) {
    println!(
        "{:<15} {:<30} {:<30} {:<15} {}",
        "Key Type", "Source", "Key ID", "Status", "Details"
    );
    println!("{:-<130}", "");
    for row in rows {
        println!(
            "{:<15} {:<30} {:<30} {:<15} {}",
            row.key_type, row.source, row.key_id, row.status, row.details
        );
    }
}

fn print_pretty(rows: &[KeyValidationRow]) {
    for row in rows {
        println!("{} key", row.key_type);
        println!("  Source : {}", row.source);
        println!("  Key ID : {}", row.key_id);
        println!("  Status : {}", row.status);
        println!("  Details: {}", row.details);
        println!();
    }
}

fn format_management_id(info: &ManagementKeyMaterial) -> String {
    if info.masked {
        mask_identifier(&info.id)
    } else {
        info.id.clone()
    }
}
