mod api_keys;
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
mod marker_settings;
mod markers;
mod queries;
mod query_annotations;
mod recipients;
mod slos;
mod triggers;

use anyhow::Result;
use clap::{Parser, Subcommand};
use client::HoneycombClient;
use common::OutputFormat;

#[derive(Parser)]
#[command(name = "apiary")]
#[command(author = "Blake")]
#[command(version = "1.0.0")]
#[command(about = "A comprehensive CLI for Honeycomb API with dual authentication", long_about = None)]
struct Cli {
    /// Honeycomb Management API key ID for v2 endpoints (format: hcxmk_...)
    #[arg(
        long,
        env = "HONEYCOMB_MANAGEMENT_KEY_ID",
        help = "Management API key ID for v2 endpoints"
    )]
    management_key_id: Option<String>,

    /// Honeycomb Management API key secret for v2 endpoints
    #[arg(
        long,
        env = "HONEYCOMB_MANAGEMENT_KEY",
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
    /// Query operations
    Queries {
        #[command(subcommand)]
        command: queries::QueryCommands,
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
    /// API Key management (v2 API)
    ApiKeys {
        #[command(subcommand)]
        command: api_keys::ApiKeyCommands,
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
    /// Query Annotations management
    QueryAnnotations {
        #[command(subcommand)]
        command: query_annotations::QueryAnnotationCommands,
    },
    /// Send events to Honeycomb
    Events {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Event data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
        /// Use batch endpoint
        #[arg(long)]
        batch: bool,
    },
    /// Create and manage query results
    QueryResults {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        #[command(subcommand)]
        command: QueryResultCommands,
    },
    /// Service Maps operations
    ServiceMaps {
        #[command(subcommand)]
        command: ServiceMapCommands,
    },
    /// Reporting operations
    Reporting {
        #[command(subcommand)]
        command: ReportingCommands,
    },
}

#[derive(Subcommand)]
enum QueryResultCommands {
    /// Create a query result
    Create {
        /// Query ID
        #[arg(short, long)]
        query_id: String,
    },
    /// Get query result status and data
    Get {
        /// Query Result ID
        #[arg(short, long)]
        id: String,
    },
}

#[derive(Subcommand)]
enum ServiceMapCommands {
    /// Create dependency request
    CreateDependencyRequest {
        /// Request data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
    },
    /// Get dependency request
    GetDependencyRequest {
        /// Request ID
        #[arg(short, long)]
        id: String,
    },
}

#[derive(Subcommand)]
enum ReportingCommands {
    /// Get SLO historical data
    SloHistory {
        /// SLO historical data request (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
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

    match cli.command {
        Some(command) => execute_command(&client, command).await,
        None => {
            display_resource_usage();
            Ok(())
        }
    }
}

fn display_resource_usage() {
    println!("Apiary - Comprehensive Honeycomb API CLI");
    println!();
    println!("SUPPORTED HONEYCOMB API RESOURCES:");
    println!();

    println!("Authentication & Management:");
    println!("  auth              - Authentication operations and token validation");
    println!("  api-keys          - API key management (v2 Management API)");
    println!("  environments      - Environment management (v2 Management API)");
    println!();

    println!("Data & Schema:");
    println!("  datasets          - Dataset management and configuration");
    println!("  columns           - Column definitions and metadata");
    println!("  calculated-fields - Derived column calculations");
    println!("  dataset-definitions - Dataset schema definitions");
    println!();

    println!("Queries & Analysis:");
    println!("  queries           - Query creation, execution, and management");
    println!("  query-results     - Query result retrieval and status");
    println!("  query-annotations - Query annotation management");
    println!("  boards            - Dashboard and board management");
    println!();

    println!("Events & Data Ingestion:");
    println!("  events            - Send events to Honeycomb datasets");
    println!();

    println!("Alerting & Monitoring:");
    println!("  triggers          - Alert trigger configuration");
    println!("  slos              - Service Level Objective management");
    println!("  burn-alerts       - SLO burn alert configuration");
    println!("  recipients        - Notification recipient management");
    println!();

    println!("Visualization & Annotation:");
    println!("  markers           - Event marker management");
    println!("  marker-settings   - Marker display configuration");
    println!();

    println!("Service Management:");
    println!("  service-maps      - Service dependency mapping");
    println!("  reporting         - Historical SLO and metrics reporting");
    println!();

    println!("USAGE:");
    println!("  apiary <RESOURCE> --help           Show help for a specific resource");
    println!("  apiary <RESOURCE> <COMMAND> --help Show help for a specific command");
    println!();

    println!("EXAMPLES:");
    println!("  apiary datasets list               List all datasets");
    println!("  apiary queries create --help       Show query creation options");
    println!("  apiary events --dataset my-dataset --data event.json");
    println!("  apiary triggers list --dataset my-dataset");
    println!();

    println!("AUTHENTICATION:");
    println!(
        "  Set HONEYCOMB_MANAGEMENT_API_KEY_ID and HONEYCOMB_MANAGEMENT_API_KEY for v2 endpoints"
    );
    println!("  Set HONEYCOMB_CONFIGURATION_API_KEY for v1 endpoints");
    println!("  Or use --management-key-id / --management-key-secret / --config-key flags");
    println!();

    println!("ENDPOINT CONFIGURATION:");
    println!("  Set HONEYCOMB_API_ENDPOINT for custom endpoints (e.g., api.eu1.honeycomb.io)");
    println!("  Or use --api-endpoint / --api-url flags");
    println!();

    println!("For detailed help on any resource, use: apiary <resource> --help");
}

async fn execute_command(client: &HoneycombClient, command: Commands) -> Result<()> {
    match command {
        Commands::Auth { command } => command.execute(client).await,
        Commands::Datasets { command } => command.execute(client).await,
        Commands::Columns { command } => command.execute(client).await,
        Commands::Triggers { command } => command.execute(client).await,
        Commands::Queries { command } => command.execute(client).await,
        Commands::Boards { command } => command.execute(client).await,
        Commands::Markers { command } => command.execute(client).await,
        Commands::Recipients { command } => command.execute(client).await,
        Commands::Slos { command } => command.execute(client).await,
        Commands::BurnAlerts { command } => command.execute(client).await,
        Commands::Environments { command } => command.execute(client).await,
        Commands::ApiKeys { command } => command.execute(client).await,
        Commands::CalculatedFields { command } => command.execute(client).await,
        Commands::DatasetDefinitions { command } => command.execute(client).await,
        Commands::MarkerSettings { command } => command.execute(client).await,
        Commands::QueryAnnotations { command } => command.execute(client).await,
        Commands::Events {
            dataset,
            data,
            batch,
        } => send_events(client, &dataset, &data, batch).await,
        Commands::QueryResults { dataset, command } => {
            execute_query_result_command(client, &dataset, command).await
        }
        Commands::ServiceMaps { command } => execute_service_map_command(client, command).await,
        Commands::Reporting { command } => execute_reporting_command(client, command).await,
    }
}

async fn send_events(
    client: &HoneycombClient,
    dataset: &str,
    data: &str,
    batch: bool,
) -> Result<()> {
    let json_data = if std::path::Path::new(data).exists() {
        common::read_json_file(data)?
    } else {
        serde_json::from_str(data)?
    };

    let path = if batch {
        format!("/1/batch/{}", dataset)
    } else {
        format!("/1/events/{}", dataset)
    };

    let response = client.post(&path, &json_data).await?;
    println!("{}", common::pretty_print_json(&response)?);

    Ok(())
}

async fn execute_query_result_command(
    client: &HoneycombClient,
    dataset: &str,
    command: QueryResultCommands,
) -> Result<()> {
    match command {
        QueryResultCommands::Create { query_id } => {
            let data = serde_json::json!({ "query_id": query_id });
            let path = format!("/1/query_results/{}", dataset);
            let response = client.post(&path, &data).await?;
            println!("{}", common::pretty_print_json(&response)?);
        }
        QueryResultCommands::Get { id } => {
            let path = format!("/1/query_results/{}/{}", dataset, id);
            let response = client.get(&path, None).await?;
            println!("{}", common::pretty_print_json(&response)?);
        }
    }
    Ok(())
}

async fn execute_service_map_command(
    client: &HoneycombClient,
    command: ServiceMapCommands,
) -> Result<()> {
    match command {
        ServiceMapCommands::CreateDependencyRequest { data } => {
            let json_data = if std::path::Path::new(&data).exists() {
                common::read_json_file(&data)?
            } else {
                serde_json::from_str(&data)?
            };

            let response = client
                .post("/1/maps/dependencies/requests", &json_data)
                .await?;
            println!("{}", common::pretty_print_json(&response)?);
        }
        ServiceMapCommands::GetDependencyRequest { id } => {
            let path = format!("/1/maps/dependencies/requests/{}", id);
            let response = client.get(&path, None).await?;
            println!("{}", common::pretty_print_json(&response)?);
        }
    }
    Ok(())
}

async fn execute_reporting_command(
    client: &HoneycombClient,
    command: ReportingCommands,
) -> Result<()> {
    match command {
        ReportingCommands::SloHistory { data } => {
            let json_data = if std::path::Path::new(&data).exists() {
                common::read_json_file(&data)?
            } else {
                serde_json::from_str(&data)?
            };

            let response = client
                .post("/1/reporting/slos/historical", &json_data)
                .await?;
            println!("{}", common::pretty_print_json(&response)?);
        }
    }
    Ok(())
}
