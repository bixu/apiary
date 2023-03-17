use clap::{Arg, App};
use reqwest;
use tokio;

struct ProgramArgs {
    api_key: String,
    dataset: String,
    resource: String,
}

impl ProgramArgs {
    fn new() -> Self {
        
        let about_text = r###"A command-line interface to the Honeycomb API

Try using piping output to `jq` like this:

    apiary --api_key=$HONEYCOMB_API_KEY --dataset="<dataset name>" --resource="columns" \
        | jq --raw-output '.[] | "\(.key_name) \(.id)"'    

"###;

        // basic app information
        let app = App::new("apiary")
            .version("0.1.0")
            .about(about_text);

        let api_key_option = Arg::with_name("api_key")
            .long("api_key") // allow --name
            .short("k") // allow -n
            .takes_value(true)
            .help("A valid Honeycomb API key.")
            .required(true);

        let dataset_option = Arg::with_name("dataset")
            .long("dataset")
            .short("d")
            .takes_value(true)
            .help("The Honeycomb Dataset to query.")
            .required(true);

        let resource_option = Arg::with_name("resource")
            .long("resource")
            .short("r")
            .takes_value(true)
            .help("The Honeycomb Dataset resource to interact with.")
            .required(true);

        let app = app.arg(api_key_option);
        let app = app.arg(dataset_option);
        let app = app.arg(resource_option);

        let matches = app.get_matches();

        let api_key = matches.value_of("api_key")
            .expect("You must pass in a valid API key with `--api_key`.");

        let dataset = matches.value_of("dataset")
            .expect("You must pass in a Dataset with `--dataset`.");

        let resource = matches.value_of("resource")
            .expect("You must pass in a Dataset resource with `--resource`.");

        ProgramArgs { 
            api_key: api_key.to_string(),
            dataset: dataset.to_string(),
            resource: resource.to_string(),
        }
    }
}


#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args = ProgramArgs::new();

    let url = format!("https://api.honeycomb.io/1/{}/{}", args.resource, args.dataset);

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("x-honeycomb-team", args.api_key)
        .send()
        .await
        .expect("failed to get response")
        .text()
        .await
        .expect("failed to get payload");

        println!("{}", response);
    Ok(())
}
