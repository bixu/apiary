# Testing Documentation

This document describes the test coverage and testing approach for the environment validation functionality in the Apiary CLI.

## Test Structure

### Unit Tests (`tests/unit_tests.rs`)

These tests focus on testing the core validation functions in isolation using mock servers:

- **`test_validate_environment_with_valid_slug`**: Tests that the validation function correctly identifies a valid environment by its slug
- **`test_validate_environment_with_valid_name`**: Tests that the validation function correctly identifies a valid environment by its name  
- **`test_validate_environment_with_invalid_environment`**: Tests that the validation function correctly rejects an invalid environment
- **`test_require_valid_environment_success`**: Tests that the `require_valid_environment` function succeeds for valid environments
- **`test_require_valid_environment_failure`**: Tests that the `require_valid_environment` function fails with a descriptive error for invalid environments

### Integration Tests (`tests/integration_tests.rs`)

These tests verify the complete CLI behavior from the user perspective:

- **`test_datasets_list_requires_team_and_environment`**: Ensures both team and environment parameters are required
- **`test_datasets_list_requires_environment`**: Ensures environment parameter is required when team is provided
- **`test_datasets_list_requires_team`**: Ensures team parameter is required when environment is provided
- **`test_datasets_list_help_shows_required_params`**: Verifies that help output shows the correct required parameters
- **`test_datasets_list_short_flags`**: Tests that short flags (-t, -e) work correctly

## Test Coverage

The tests cover the following scenarios:

### ✅ Happy Path
- Valid environment slug validation
- Valid environment name validation  
- Successful command execution with valid parameters

### ✅ Error Handling
- Invalid environment names/slugs
- Missing required parameters
- Invalid team names
- Empty environment lists

### ✅ CLI Interface
- Required parameter validation
- Help text accuracy
- Short and long flag support
- Error message clarity

## Running Tests

### All Tests
```bash
cargo test
```

### Unit Tests Only
```bash
cargo test --test unit_tests
```

### Integration Tests Only
```bash
cargo test --test integration_tests
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

## Test Dependencies

- **`wiremock`**: Mock HTTP server for simulating API responses
- **`assert_cmd`**: CLI testing framework
- **`predicates`**: Assertion predicates for flexible test conditions
- **`serde_json`**: JSON manipulation for mock responses
- **`tokio-test`**: Async test utilities

## Adding New Tests

When adding new environment validation features:

1. **Add unit tests** for core validation logic in `tests/unit_tests.rs`
2. **Add integration tests** for CLI behavior in `tests/integration_tests.rs`
3. **Mock relevant API endpoints** using WireMock
4. **Test both success and failure cases**
5. **Verify error messages are user-friendly**

## Test Scenarios Covered

### Environment Validation
- [x] Valid environment slug
- [x] Valid environment name  
- [x] Invalid environment
- [x] Empty environment list
- [x] Network/API errors

### CLI Arguments
- [x] Missing team parameter
- [x] Missing environment parameter
- [x] Missing both parameters
- [x] Help text accuracy
- [x] Short flag support

### Error Messages
- [x] Descriptive validation errors
- [x] Helpful suggestions (list available environments)
- [x] Clear parameter requirement messages

The test suite ensures robust, reliable environment validation that provides clear feedback to users while maintaining backward compatibility where possible.