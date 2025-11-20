# Apiary - Honeycomb Management CLI

`apiary` is a comprehensive command-line interface to the Honeycomb API, written in Rust. It provides complete coverage of all Honeycomb management features using modern Management API Keys.

## Prerequisites

### Management API Key Required

This CLI requires a **Management API Key** from Honeycomb. Management Keys provide unified access to all Honeycomb APIs and use modern Bearer token authentication.

**To get a Management Key:**
1. Log into your Honeycomb UI
2. Go to **Team Settings → API Keys**
3. Click **"Create Management Key"**
4. Copy the key (format: `hcxmk_[id]:[secret]`)
5. Set it as an environment variable:
   ```bash
   export HONEYCOMB_API_KEY="hcxmk_your_management_key_here"
   ```

## Features

- **Management Key Authentication**: Modern Bearer token authentication
- **Multiple Output Formats**: JSON, pretty-printed JSON, and table formats
- **Environment Variables**: Support for API keys and configuration via environment variables
- **Comprehensive Management**: Full access to all Honeycomb resources

## Supported Resources

### Core Resources
- **Datasets** - Dataset management and configuration
- **Columns** - Column schema management
- **Events** - Event ingestion (single and batch)
- **Queries** - Query creation, execution, and result retrieval

### Observability & Alerting
- **Triggers** - Alert trigger management
- **Recipients** - Notification recipient configuration
- **SLOs** - Service Level Objective management
- **Burn Alerts** - SLO budget burn alerting
- **Markers** - Timeline markers for deployments and incidents

### Organization & Collaboration
- **Boards** - Dashboard and board management
- **Environments** - Environment configuration (v2 API)
- **Keys** - API key management (v2 API)
- **Auth** - Authentication validation and permissions

### Advanced Features
- **Service Maps** - Service dependency visualization
- **Reporting** - Historical data and reporting
- **Query Annotations** - Query metadata and collaboration

## Installation

### From Source
```bash
git clone <repository-url>
cd apiary
cargo build --release
```

The binary will be available at `target/release/apiary`.

### Development
```bash
cargo run -- --help
```

## Configuration

### Environment Variables
```bash
export HONEYCOMB_API_KEY="your-api-key-here"
export HONEYCOMB_API_URL="https://api.honeycomb.io"  # optional
export HONEYCOMB_TEAM="your-team-slug"              # for v2 APIs
```

### Command Line Options
```bash
apiary --api-key=<key> --team=<team> <command> <subcommand> [options]
```

## Usage Examples

### Authentication
```bash
# Validate API key and show permissions
apiary auth validate
```

### Dataset Management
```bash
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
```bash
# List columns in a dataset
apiary columns list --dataset=myapp

# Get specific column
apiary columns get --dataset=myapp --id=column123

# Update column
apiary columns update --dataset=myapp --id=column123 --data='{"hidden":true}'
```

### Query Operations
```bash
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
```bash
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
```bash
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
```bash
# List boards
apiary boards list

# Create board
apiary boards create --data='{
  "name": "My Dashboard",
  "queries": [{"query_id": "query123", "dataset": "myapp"}]
}'
```

### SLO Management
```bash
# List SLOs
apiary slos list --dataset=myapp

# Create SLO
apiary slos create --dataset=myapp --data=slo.json

# Get SLO details
apiary slos get --dataset=myapp --id=slo123
```

### Advanced Operations
```bash
# Service dependency mapping
apiary service-maps create-dependency-request --data='{
  "start_time": "2023-11-20T00:00:00Z",
  "end_time": "2023-11-20T23:59:59Z"
}'

# SLO historical reporting
apiary reporting slo-history --data='{
  "slo_ids": ["slo123"],
  "start_time": "2023-11-01T00:00:00Z",
  "end_time": "2023-11-20T23:59:59Z"
}'
```

## Output Formats

### Table Format (default for lists)
```bash
apiary datasets list --format=table
```

### JSON Format
```bash
apiary datasets list --format=json
```

### Pretty JSON Format (default for single items)
```bash
apiary datasets get --dataset=myapp --format=pretty
```

## API Consistency

The CLI is designed to maintain complete consistency with the Honeycomb API documentation:

- **Endpoint Mapping**: Each CLI command maps directly to an API endpoint
- **Parameter Names**: CLI flags use the same names as API parameters
- **Response Formats**: Raw API responses are preserved in JSON output modes
- **Error Handling**: API errors are passed through with original messages
- **Authentication**: Uses the same authentication headers as the API

This means:
- Anyone familiar with the API docs can use the CLI immediately
- CLI users gain understanding of the underlying API
- Switching between CLI and API calls is seamless

## Error Handling

The CLI provides detailed error messages and preserves API error responses:

```bash
$ apiary datasets get --dataset=nonexistent
Error: Request failed with status 404: {"error":"dataset not found"}
```

## Development

### Project Structure
```
src/
├── main.rs          # CLI interface and command routing
├── client.rs        # HTTP client and API communication
├── common.rs        # Shared utilities and types
├── auth.rs          # Authentication operations
├── datasets.rs      # Dataset management
├── columns.rs       # Column operations
├── triggers.rs      # Trigger management
├── queries.rs       # Query operations
├── boards.rs        # Board management
└── resources.rs     # Additional resources (markers, SLOs, etc.)
```

### Adding New Endpoints

When Honeycomb adds new API endpoints:
1. Add the endpoint to the appropriate module
2. Follow the existing pattern for CLI arguments
3. Maintain consistency with API documentation
4. Add examples to this README

## Contributing

Contributions are welcome! Please ensure:
- New features maintain API consistency
- Tests are added for new functionality
- Documentation is updated
- Error handling follows existing patterns

## License

Licensed under the Apache License, Version 2.0.
