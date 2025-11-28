mod auth;
mod boards;
mod burn_alerts;
mod calculated_fields;
mod client;
mod columns;
mod common;
mod dataset_definitions;
mod datasets;
mod environments;
mod errors;
mod marker_settings;
mod markers;

mod recipients;
mod slos;
mod triggers;

use anyhow::Result;
use clap::{Parser, Subcommand};
use client::HoneycombClient;
use common::OutputFormat;
use std::env;

#[derive(Parser)]
#[command(name = "apiary")]
#[command(author = "Blake")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "A comprehensive CLI for Honeycomb API with dual authentication", long_about = None)]
struct Cli {
    /// Honeycomb Management API key ID for v2 endpoints (format: hcxmk_...)
    #[arg(
        long,
        env = "HONEYCOMB_MANAGEMENT_API_KEY_ID",
        help = "Management API key ID for v2 endpoints"
    )]
    management_key_id: Option<String>,

    /// Honeycomb Management API key secret for v2 endpoints
    #[arg(
        long,
        env = "HONEYCOMB_MANAGEMENT_API_KEY",
        help = "Management API key secret for v2 endpoints"
    )]
    management_key_secret: Option<String>,

    /// Honeycomb Configuration API key for v1 endpoints (64 chars, starts with hcaik_)
    #[arg(
        long,
        env = "HONEYCOMB_CONFIGURATION_API_KEY",
        help = "Configuration API key for v1 endpoints"
    )]
    config_key: Option<String>,

    /// Legacy: Honeycomb API key (will use as Management key if others not provided)
    #[arg(
        short,
        long,
        env = "HONEYCOMB_API_KEY",
        help = "Legacy API key (use management_key_id/management_key_secret/config_key instead)"
    )]
    api_key: Option<String>,

    /// Honeycomb API base URL
    #[arg(long, env = "HONEYCOMB_API_URL")]
    api_url: Option<String>,

    /// Honeycomb API endpoint (e.g., eu1.api.honeycomb.io)
    #[arg(long, env = "HONEYCOMB_API_ENDPOINT")]
    api_endpoint: Option<String>,

    /// Team slug (for v2 API endpoints)
    #[arg(long, env = "HONEYCOMB_TEAM")]
    team: Option<String>,

    /// Global output format override
    #[arg(long, global = true)]
    format: Option<OutputFormat>,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Authentication operations
    Auth {
        #[command(subcommand)]
        command: auth::AuthCommands,
    },
    /// Dataset management
    Datasets {
        #[command(subcommand)]
        command: datasets::DatasetCommands,
    },
    /// Column management
    Columns {
        #[command(subcommand)]
        command: columns::ColumnCommands,
    },
    /// Trigger management
    Triggers {
        #[command(subcommand)]
        command: triggers::TriggerCommands,
    },

    /// Board management
    Boards {
        #[command(subcommand)]
        command: boards::BoardCommands,
    },
    /// Marker management
    Markers {
        #[command(subcommand)]
        command: markers::MarkerCommands,
    },
    /// Recipient management
    Recipients {
        #[command(subcommand)]
        command: recipients::RecipientCommands,
    },
    /// SLO management
    Slos {
        #[command(subcommand)]
        command: slos::SloCommands,
    },
    /// Burn Alert management
    BurnAlerts {
        #[command(subcommand)]
        command: burn_alerts::BurnAlertCommands,
    },
    /// Environment management (v2 API)
    Environments {
        #[command(subcommand)]
        command: environments::EnvironmentCommands,
    },
    /// Calculated Fields (Derived Columns) management
    CalculatedFields {
        #[command(subcommand)]
        command: calculated_fields::CalculatedFieldCommands,
    },
    /// Dataset Definitions management
    DatasetDefinitions {
        #[command(subcommand)]
        command: dataset_definitions::DatasetDefinitionCommands,
    },
    /// Marker Settings management
    MarkerSettings {
        #[command(subcommand)]
        command: marker_settings::MarkerSettingCommands,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Determine which keys to use
    let management_key =
        if let (Some(id), Some(secret)) = (&cli.management_key_id, &cli.management_key_secret) {
            Some(format!("{}:{}", id, secret))
        } else {
            // Fall back to api_key if it looks like a management key
            cli.api_key.as_ref().and_then(|key| {
                if key.starts_with("hcxmk_") || key.starts_with("hcamk_") {
                    Some(key.clone())
                } else {
                    None
                }
            })
        };

    let config_key = cli.config_key.or_else(|| {
        // Fall back to api_key if it looks like a config key
        cli.api_key.as_ref().and_then(|key| {
            if key.starts_with("hcaik_") || (key.len() == 64 && !key.contains(":")) {
                Some(key.clone())
            } else {
                None
            }
        })
    });

    // Construct the API URL - prioritize api_url, then construct from api_endpoint
    let api_url = cli.api_url.or_else(|| {
        cli.api_endpoint.map(|endpoint| {
            if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
                endpoint
            } else {
                format!("https://{}", endpoint)
            }
        })
    });

    if cli.verbose {
        if let Some(ref mgmt_key) = management_key {
            eprintln!("Management Key: {}...", &mgmt_key[..8]);
        }
        if let Some(ref conf_key) = config_key {
            eprintln!("Configuration Key: {}...", &conf_key[..8]);
        }
        if let Some(ref url) = api_url {
            eprintln!("API URL: {}", url);
        }
        if let Some(ref team) = cli.team {
            eprintln!("Team: {}", team);
        }
    }

    let client = HoneycombClient::new(management_key, config_key, api_url);

    let context = common::CommandContext { team: cli.team };

    match cli.command {
        Some(command) => execute_command(&client, command, &context).await,
        None => {
            display_resource_usage();
            Ok(())
        }
    }
}

fn display_resource_usage() {
    println!();
    println!("Apiary - The Honeycomb API CLI");
    println!();

    println!("  auth                - Authentication operations and token validation");
    println!("  boards              - Dashboard and board management");
    println!("  burn-alerts         - SLO burn alert configuration");
    println!("  calculated-fields   - Derived column calculations");
    println!("  columns             - Column definitions and metadata");
    println!("  dataset-definitions - Dataset schema definitions");
    println!("  datasets            - Dataset management and configuration");
    println!("  environments        - Environment management (v2 Management API)");
    println!("  marker-settings     - Marker display configuration");
    println!("  markers             - Event marker management");
    println!("  recipients          - Notification recipient management");
    println!("  slos                - Service Level Objective management");
    println!("  triggers            - Alert trigger configuration");
    println!();

    println!("Usage:");
    println!("  apiary <resource> --help            Show help for a specific resource");
    println!("  apiary <resource> <COMMAND> --help  Show help for a specific command");
    println!();

    println!("Examples:");
    println!("  apiary datasets list                        List all datasets");
    println!("  apiary boards list --help                   Show board listing options");
    println!("  apiary triggers list --dataset my-dataset   List triggers for a specific dataset");
    println!();

    println!("Authentication:");
    println!(
        "  Set HONEYCOMB_MANAGEMENT_API_KEY_ID and HONEYCOMB_MANAGEMENT_API_KEY (v2 endpoints)"
    );
    println!("  Set HONEYCOMB_CONFIGURATION_API_KEY (v1 endpoints)");
    println!();
    println!(
        "  Or use flags:
          --management-key-id
          --management-key-secret
          --config-key flags"
    );
    println!();

    println!("Endpoint Configuration:");
    println!("  Set HONEYCOMB_API_ENDPOINT for custom endpoints (e.g., api.eu1.honeycomb.io)");
    println!(
        "  Or use flags:
          --api-endpoint
          --api-url flags"
    );
    println!();

    println!("For detailed help on any resource, use: apiary <resource> --help");
}

async fn execute_command(
    client: &HoneycombClient,
    command: Commands,
    context: &common::CommandContext,
) -> Result<()> {
    match command {
        Commands::Auth { command } => command.execute(client, context).await,
        Commands::Datasets { command } => command.execute(client, context).await,
        Commands::Columns { command } => command.execute(client, context).await,
        Commands::Triggers { command } => command.execute(client, context).await,
        Commands::Boards { command } => command.execute(client, context).await,
        Commands::Markers { command } => command.execute(client, context).await,
        Commands::Recipients { command } => command.execute(client, context).await,
        Commands::Slos { command } => command.execute(client, context).await,
        Commands::BurnAlerts { command } => command.execute(client, context).await,
        Commands::Environments { command } => command.execute(client, context).await,
        Commands::CalculatedFields { command } => command.execute(client, context).await,
        Commands::DatasetDefinitions { command } => command.execute(client, context).await,
        Commands::MarkerSettings { command } => command.execute(client, context).await,
    }
}
