use crate::client::HoneycombClient;
use crate::common::{pretty_print_json, read_json_file, OutputFormat, DEFAULT_PRETTY_FORMAT};
use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum DatasetDefinitionCommands {
    /// Get dataset definitions
    Get {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Team slug (uses HONEYCOMB_TEAM env var if not specified)
        #[arg(short, long, env = "HONEYCOMB_TEAM")]
        team: Option<String>,
        /// Environment slug (optional, uses HONEYCOMB_ENVIRONMENT env var if not specified)
        #[arg(short, long, env = "HONEYCOMB_ENVIRONMENT")]
        environment: Option<String>,
        /// Output format
        #[arg(short, long, default_value = DEFAULT_PRETTY_FORMAT)]
        format: OutputFormat,
    },
    /// Update dataset definitions
    Update {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Team slug (uses HONEYCOMB_TEAM env var if not specified)
        #[arg(short, long, env = "HONEYCOMB_TEAM")]
        team: Option<String>,
        /// Environment slug (optional, uses HONEYCOMB_ENVIRONMENT env var if not specified)
        #[arg(short, long, env = "HONEYCOMB_ENVIRONMENT")]
        environment: Option<String>,
        /// Dataset definition data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = DEFAULT_PRETTY_FORMAT)]
        format: OutputFormat,
    },
}

impl DatasetDefinitionCommands {
    pub async fn execute(&self, client: &HoneycombClient, global_team: Option<&str>) -> Result<()> {
        match self {
            DatasetDefinitionCommands::Get {
                dataset,
                team,
                environment,
                format,
            } => {
                let team_str = global_team.or(team.as_deref()).unwrap_or("default");
                get_dataset_definitions(client, dataset, team_str, environment.as_deref(), format)
                    .await
            }
            DatasetDefinitionCommands::Update {
                dataset,
                team,
                environment,
                data,
                format,
            } => {
                let team_str = global_team.or(team.as_deref()).unwrap_or("default");
                update_dataset_definitions(
                    client,
                    dataset,
                    team_str,
                    environment.as_deref(),
                    data,
                    format,
                )
                .await
            }
        }
    }
}

async fn get_dataset_definitions(
    client: &HoneycombClient,
    dataset: &str,
    team: &str,
    environment: Option<&str>,
    format: &OutputFormat,
) -> Result<()> {
    use crate::common::require_valid_environment;
    use std::collections::HashMap;

    // If environment is provided, validate it exists
    if let Some(env) = environment {
        require_valid_environment(client, team, env).await?;
    }

    // Build query parameters
    let mut query_params = HashMap::new();
    if let Some(env) = environment {
        query_params.insert("environment".to_string(), env.to_string());
    }

    let path = format!("/1/dataset_definitions/{}", dataset);
    let response = client
        .get(
            &path,
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
        OutputFormat::Pretty | OutputFormat::Table => {
            println!("{}", pretty_print_json(&response)?);
        }
    }

    Ok(())
}

async fn update_dataset_definitions(
    client: &HoneycombClient,
    dataset: &str,
    team: &str,
    environment: Option<&str>,
    data: &str,
    format: &OutputFormat,
) -> Result<()> {
    use crate::common::require_valid_environment;
    use std::collections::HashMap;

    // If environment is provided, validate it exists
    if let Some(env) = environment {
        require_valid_environment(client, team, env).await?;
    }

    let json_data = if std::path::Path::new(data).exists() {
        read_json_file(data)?
    } else {
        serde_json::from_str(data)?
    };

    // Build query parameters for the request if needed
    let mut query_params = HashMap::new();
    if let Some(env) = environment {
        query_params.insert("environment".to_string(), env.to_string());
    }

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
