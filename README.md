# Apiary - A (Honeycomb.io) API on the CLI

`apiary` attempts to provide coverage of most Honeycomb API resources.

The project also serves as an experiment around interactions between a
safety-centric language (Rust) and LLM coding (mostly Claude at this time).

## Installation

### Via Homebrew

```shell
brew tap bixu/apiary
brew install apiary
```

### From Source

```shell
git clone <repository-url>
cd apiary
cargo build --release
```

The binary will be available at `target/release/apiary`.

### Development
```shell
cargo run -- --help
```

## Configuration

### Environment Variables

```shell
export HONEYCOMB_API_ENDPOINT="api.eu1.honeycomb.io"
export HONEYCOMB_CONFIGURATION_API_KEY="****"
export HONEYCOMB_ENVIRONMENT="dev"
export HONEYCOMB_MANAGEMENT_API_KEY="****"
export HONEYCOMB_MANAGEMENT_API_KEY_ID="hc***_****"
export HONEYCOMB_TEAM="my_team"
```

### Command Line Options

```bash
apiary --api-key=<key> --team=<team> <command> <subcommand> [options]
```

## Usage Examples

### Authentication

```shell
# Validate API key and show permissions
apiary auth validate
```

### Dataset Management

```shell
# List all datasets
apiary datasets list

# Get specific dataset
apiary datasets get --dataset=myapp

# Create new dataset
apiary datasets create --data='{"name":"newapp","description":"New application"}'

# Create dataset from file
apiary datasets create --data=dataset.json
```

### Column Management

```shell
# List columns in a dataset
apiary columns list --dataset=myapp

# Get specific column
apiary columns get --dataset=myapp --id=column123

# Update column
apiary columns update --dataset=myapp --id=column123 --data='{"hidden":true}'
```

### Query Operations

```shell
# Create and run a query
apiary queries run --dataset=myapp --data='{
  "calculations": [{"op": "COUNT"}],
  "time_range": 3600,
  "granularity": 60
}'

# Create query without running
apiary queries create --dataset=myapp --data=query.json

# Get existing query
apiary queries get --dataset=myapp --id=query123
```

### Event Ingestion

```shell
# Send single event
apiary events --dataset=myapp --data='{
  "timestamp": "2023-11-20T10:00:00Z",
  "service": "web",
  "duration_ms": 150
}'

# Send batch events
apiary events --dataset=myapp --batch --data=events.json
```

### Trigger Management

```shell
# List triggers
apiary triggers list --dataset=myapp

# Create trigger
apiary triggers create --dataset=myapp --data='{
  "name": "High Error Rate",
  "query": {
    "calculations": [{"op": "COUNT"}],
    "filters": [{"column": "status_code", "op": ">=", "value": 400}]
  },
  "threshold": {"op": ">", "value": 10}
}'
```

### Board Operations

```shell
# List boards
apiary boards list

# Create board
apiary boards create --data='{
  "name": "My Dashboard",
  "queries": [{"query_id": "query123", "dataset": "myapp"}]
}'
```

### SLO Management

```shell
# List SLOs
apiary slos list --dataset=myapp

# Create SLO
apiary slos create --dataset=myapp --data=slo.json

# Get SLO details
apiary slos get --dataset=myapp --id=slo123
```

## Output Formats

### Table Format (default for lists)

```shell
apiary datasets list --format=table
```

### JSON Format

```shell
apiary datasets list --format=json
```

### Pretty JSON Format (default for single items)

```shell
apiary datasets get --dataset=myapp --format=pretty
```

## Contributing

Contributions are welcome! Please ensure:
- New features maintain API consistency
- Tests are added for new functionality
- Documentation is updated
- Error handling follows existing patterns

## License

Licensed under the Apache License, Version 2.0.
