# Apiary

`apiary` is a command-line interface to the Honeycomb.io API, written in Rust.
This is the first Rust program I've ever written.
Constructive criticism is most welcome!

## Usage

Getting help via the `--help` command:

```bash
$ cargo run -- --help
    Finished dev [unoptimized + debuginfo] target(s) in 0.05s
     Running `target/debug/apiary --help`
A command-line interface to the Honeycomb API

Usage: apiary --api-key <API_KEY> --dataset <DATASET> --resource <RESOURCE>

Options:
  -a, --api-key <API_KEY>
  -d, --dataset <DATASET>
  -r, --resource <RESOURCE>
  -h, --help                 Print help
  -V, --version              Print version
```

Try using piping output to `jq` like this:

```bash 
 $ apiary --api_key=$HONEYCOMB_API_KEY --dataset="<dataset name>" --resource="columns" \
         | jq --raw-output '.[] | "\(.key_name) \(.id)"';

```
