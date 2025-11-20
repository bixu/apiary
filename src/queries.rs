use crate::client::HoneycombClient;
use crate::common::{OutputFormat, pretty_print_json, read_json_file};
use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Subcommand)]
pub enum QueryCommands {
    /// Create a new query
    Create {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Query data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: OutputFormat,
    },
    /// Get a specific query
    Get {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Query ID
        #[arg(short, long)]
        id: String,
        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: OutputFormat,
    },
    /// Run a query and get results
    Run {
        /// Dataset slug
        #[arg(short, long)]
        dataset: String,
        /// Query data (JSON file path or inline JSON)
        #[arg(long)]
        data: String,
        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: OutputFormat,
        /// Poll for results until complete
        #[arg(long, default_value = "true")]
        wait: bool,
        /// Maximum time to wait for results (seconds)
        #[arg(long, default_value = "30")]
        timeout: u64,
    },
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Query {
    pub id: String,
    pub query_url: String,
    pub query_style: String,
    pub calculations: Vec<Calculation>,
    pub filters: Vec<Filter>,
    pub breakdowns: Vec<String>,
    pub orders: Vec<Order>,
    pub limit: Option<i32>,
    pub time_range: Option<i64>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub granularity: Option<i64>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct QueryResult {
    pub query_result_id: String,
    pub query_url: String,
    pub complete: bool,
    pub links: Option<QueryResultLinks>,
    pub data: Option<QueryData>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct QueryResultLinks {
    pub query_result: String,
    pub graph_image: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct QueryData {
    pub series: Vec<QuerySeries>,
    pub is_time_series: bool,
    pub query: Query,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct QuerySeries {
    pub time: Option<chrono::DateTime<chrono::Utc>>,
    pub data: Vec<QueryDataPoint>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct QueryDataPoint {
    #[serde(flatten)]
    pub values: std::collections::HashMap<String, Value>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Calculation {
    pub op: String,
    pub column: Option<String>,
    pub alias: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Filter {
    pub column: String,
    pub op: String,
    pub value: Value,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Order {
    pub op: Option<String>,
    pub column: Option<String>,
    pub order: String,
}

impl QueryCommands {
    pub async fn execute(&self, client: &HoneycombClient) -> Result<()> {
        match self {
            QueryCommands::Create { dataset, data, format } => {
                create_query(client, dataset, data, format).await
            }
            QueryCommands::Get { dataset, id, format } => {
                get_query(client, dataset, id, format).await
            }
            QueryCommands::Run { dataset, data, format, wait, timeout } => {
                run_query(client, dataset, data, format, *wait, *timeout).await
            }
        }
    }
}

async fn create_query(client: &HoneycombClient, dataset: &str, data: &str, format: &OutputFormat) -> Result<()> {
    let json_data = if std::path::Path::new(data).exists() {
        read_json_file(data)?
    } else {
        serde_json::from_str(data)?
    };
    
    let path = format!("/1/queries/{}", dataset);
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

async fn get_query(client: &HoneycombClient, dataset: &str, id: &str, format: &OutputFormat) -> Result<()> {
    let path = format!("/1/queries/{}/{}", dataset, id);
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

async fn run_query(client: &HoneycombClient, dataset: &str, data: &str, format: &OutputFormat, wait: bool, timeout: u64) -> Result<()> {
    // Step 1: Create the query
    let json_data = if std::path::Path::new(data).exists() {
        read_json_file(data)?
    } else {
        serde_json::from_str(data)?
    };
    
    let query_path = format!("/1/queries/{}", dataset);
    let query_response = client.post(&query_path, &json_data).await?;
    
    let query_id = query_response.get("id")
        .and_then(|id| id.as_str())
        .ok_or_else(|| anyhow::anyhow!("Failed to get query ID from response"))?;
    
    // Step 2: Create query result
    let result_path = format!("/1/query_results/{}", dataset);
    let result_data = serde_json::json!({
        "query_id": query_id
    });
    
    let result_response = client.post(&result_path, &result_data).await?;
    
    let result_id = result_response.get("query_result_id")
        .and_then(|id| id.as_str())
        .ok_or_else(|| anyhow::anyhow!("Failed to get query result ID from response"))?;
    
    if !wait {
        println!("Query result ID: {}", result_id);
        println!("Use 'apiary query-results get --dataset {} --id {}' to check status", dataset, result_id);
        return Ok(());
    }
    
    // Step 3: Poll for results
    let poll_path = format!("/1/query_results/{}/{}", dataset, result_id);
    let mut elapsed = 0;
    
    loop {
        if elapsed >= timeout {
            anyhow::bail!("Query timed out after {} seconds", timeout);
        }
        
        let poll_response = client.get(&poll_path, None).await?;
        
        if let Some(complete) = poll_response.get("complete").and_then(|c| c.as_bool()) {
            if complete {
                match format {
                    OutputFormat::Json => {
                        println!("{}", serde_json::to_string(&poll_response)?);
                    }
                    OutputFormat::Pretty | OutputFormat::Table => {
                        println!("{}", pretty_print_json(&poll_response)?);
                    }
                }
                return Ok(());
            }
        }
        
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        elapsed += 1;
    }
}