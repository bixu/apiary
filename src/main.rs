use clap::Parser;
use reqwest;
use tokio;

#[derive(Parser)]
#[command(name = "apiary")]
#[command(author = "Blake")]
#[command(version = "1.0")]
#[command(about = "A command-line interface to the Honeycomb API", long_about = None)]
struct Cli {
    #[arg(short, long)]
    api_key: String,
    #[arg(short, long)]
    dataset: String,
    #[arg(short, long)]
    resource: String,
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let cli = Cli::parse();

    let url = format!("https://api.honeycomb.io/1/{}/{}", cli.resource, cli.dataset);

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("x-honeycomb-team", cli.api_key)
        .send()
        .await
        .expect("failed to get response")
        .text()
        .await
        .expect("failed to get payload");

        println!("{}", response);
    Ok(())
}
