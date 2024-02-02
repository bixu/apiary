use clap::Parser;

#[derive(Parser)]
#[command(name = "apiary")]
#[command(author = "Blake")]
#[command(version = "0.3.0")]
#[command(about = "A command-line interface to the Honeycomb API", long_about = None)]
struct Cli {
    #[arg(short, long)]
    api_key: String,
    #[arg(short, long)]
    dataset: String,
    #[arg(short, long)]
    endpoint: String,
    #[arg(short, long)]
    id: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let cli = Cli::parse();

    let url = format!("https://api.honeycomb.io/1/{}/{}/{}?detailed=true",
                      cli.endpoint,
                      cli.dataset,
                      cli.id.unwrap_or_else(|| "".to_string())
                     );

    let client = reqwest::Client::new();
    // https://docs.honeycomb.io/api/tag/SLOs#operation/getSlo
    let url_clone = url.clone(); // avoid borrow-checker errors
    let response = client
        .get(url_clone) // Use the cloned `url` variable
        .header("x-honeycomb-team", cli.api_key)
        .send()
        .await
        .expect("failed to get response")
        .text()
        .await
        .expect("failed to get payload");

    eprintln!("Requesting: {}", url);
    println!("{}", response);
    Ok(())
}
