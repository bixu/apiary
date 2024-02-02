# Apiary

`apiary` is a command-line interface to the Honeycomb.io API, written in Rust.
This is the first Rust program I've ever written.
Constructive criticism is most welcome!

## Usage

Getting help via the `--help` command:

```bash
$ cargo run -- --help
```

Try piping output to `jq` like this:

```bash 
cargo run -- --api-key=$HONEYCOMB_API_KEY --dataset="<dataset>" --resource="columns" | jq --raw-output '.[] | "\(.key_name) \(.id)"'
```

## Building

```bash
$ cargo build
```
