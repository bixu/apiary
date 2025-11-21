# Testing Documentation

This file outlines the testing procedures and configurations for the project.

## Running Tests

### All Tests

```bash
cargo test
```

### With Coverage

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --all-features --workspace --timeout 120
```

## Mock Server Usage

The tests use WireMock to simulate Honeycomb API responses:

- Environment validation endpoints (`/2/teams/{team}/environments`)
- Dataset listing endpoints (`/1/datasets`)
- Error responses (404, validation failures)

## GitHub Actions Integration

The CI workflow (`.github/workflows/ci.yml`) includes:

- **Multi-Rust Version Testing**: Tests against stable, beta, and nightly Rust
- **Code Quality Checks**: Formatting, linting, and security auditing
- **Cross-Platform Builds**: Linux, macOS, and Windows
- **Coverage Reports**: Integration with Codecov for coverage tracking
- **CLI Validation**: Tests actual binary behavior in CI environment
