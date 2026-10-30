//! Integration tests for CLI command-line interface execution.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_version_output_contains_repository_url() {
    let mut cmd = Command::cargo_bin("gitlab").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"))
        .stdout(predicate::str::contains(
            "https://github.com/GBouerat/gitlab-cli",
        ));
}

#[test]
fn test_help_output_contains_repository_url() {
    let mut cmd = Command::cargo_bin("gitlab").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "https://github.com/GBouerat/gitlab-cli",
        ))
        .stdout(predicate::str::contains("Repository:"));
}

#[test]
fn test_completion_bash() {
    let mut cmd = Command::cargo_bin("gitlab").unwrap();
    cmd.args(["completion", "bash"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("_gitlab()"));
}

#[test]
fn test_completion_zsh() {
    let mut cmd = Command::cargo_bin("gitlab").unwrap();
    cmd.args(["completion", "zsh"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("#compdef gitlab"));
}

#[test]
fn test_completion_unsupported_shell() {
    let mut cmd = Command::cargo_bin("gitlab").unwrap();
    cmd.args(["completion", "fish"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid value 'fish'"));
}

#[test]
fn test_subcommand_aliases() {
    // repo alias for project
    let mut cmd_repo = Command::cargo_bin("gitlab").unwrap();
    cmd_repo.args(["repo", "--help"]);
    cmd_repo.assert().success();

    // merge-request alias for mr
    let mut cmd_mr = Command::cargo_bin("gitlab").unwrap();
    cmd_mr.args(["merge-request", "--help"]);
    cmd_mr.assert().success();

    // pipeline alias for ci
    let mut cmd_ci = Command::cargo_bin("gitlab").unwrap();
    cmd_ci.args(["pipeline", "--help"]);
    cmd_ci.assert().success();
}
