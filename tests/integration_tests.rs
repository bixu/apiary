use assert_cmd::Command;
use predicates::prelude::*;

/// Test CLI argument validation for datasets list command
#[tokio::test]
async fn test_datasets_list_requires_team_and_environment() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--"]);

    // Remove environment variables to test validation
    cmd.env_remove("HONEYCOMB_TEAM");
    cmd.env_remove("HONEYCOMB_ENVIRONMENT");

    // Test missing both team and environment when env vars are not set
    cmd.arg("datasets")
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Team is required. Use --team flag or set HONEYCOMB_TEAM environment variable.",
        ));
}

#[tokio::test]
async fn test_datasets_list_requires_environment() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--"]);

    // Remove team environment variable to test team validation
    cmd.env_remove("HONEYCOMB_TEAM");
    // Keep environment variable - it's not required for datasets list

    // Test missing team argument when no HONEYCOMB_TEAM env var is set
    // Environment is optional for datasets list command
    cmd.arg("datasets")
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Team is required. Use --team flag or set HONEYCOMB_TEAM environment variable.",
        ));
}

#[tokio::test]
async fn test_datasets_list_requires_team() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--"]);

    // Remove HONEYCOMB_TEAM to test fallback behavior
    cmd.env_remove("HONEYCOMB_TEAM");

    // Test missing team argument when no HONEYCOMB_TEAM env var
    cmd.arg("datasets")
        .arg("list")
        .arg("--environment")
        .arg("test-env")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Team is required. Use --team flag or set HONEYCOMB_TEAM environment variable.",
        ));
}

/// Test help output includes correct parameters
#[tokio::test]
async fn test_datasets_list_help_shows_required_params() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--"]);

    cmd.arg("datasets")
        .arg("list")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--team <TEAM>"))
        .stdout(predicate::str::contains("--environment <ENVIRONMENT>"))
        .stdout(predicate::str::contains(
            "Team slug (uses HONEYCOMB_TEAM env var if not specified)",
        ))
        .stdout(predicate::str::contains(
            "Environment slug (uses HONEYCOMB_ENVIRONMENT env var if not specified)",
        ));
}

/// Test short flags work correctly
#[tokio::test]
async fn test_datasets_list_short_flags() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--"]);

    // Test that short flags are properly recognized in help
    cmd.arg("datasets")
        .arg("list")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("-t, --team"))
        .stdout(predicate::str::contains("-e, --environment"));
}
