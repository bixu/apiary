use assert_cmd::Command;
use predicates::prelude::*;

/// Test CLI argument validation for datasets list command
#[tokio::test]
async fn test_datasets_list_requires_team_and_environment() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--"]);

    // Test missing environment argument (team is optional due to HONEYCOMB_TEAM env var)
    cmd.arg("datasets")
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "the following required arguments were not provided",
        ))
        .stderr(predicate::str::contains("--environment <ENVIRONMENT>"));
}

#[tokio::test]
async fn test_datasets_list_requires_environment() {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--"]);

    // Test missing environment argument
    cmd.arg("datasets")
        .arg("list")
        .arg("--team")
        .arg("test-team")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "the following required arguments were not provided",
        ))
        .stderr(predicate::str::contains("--environment <ENVIRONMENT>"));
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
        .stdout(predicate::str::contains("Environment slug (required)"));
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
