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

    // environment alias for env
    let mut cmd_env = Command::cargo_bin("gitlab").unwrap();
    cmd_env.args(["environment", "--help"]);
    cmd_env.assert().success();

    // deployment alias for deploy
    let mut cmd_deploy = Command::cargo_bin("gitlab").unwrap();
    cmd_deploy.args(["deployment", "--help"]);
    cmd_deploy.assert().success();
}

#[test]
fn test_new_subcommands_help() {
    for subcommand in [
        "release",
        "branch",
        "group",
        "variable",
        "snippet",
        "tag",
        "env",
        "deploy",
        "milestone",
        "epic",
        "package",
        "registry",
    ] {
        let mut cmd = Command::cargo_bin("gitlab").unwrap();
        cmd.args([subcommand, "--help"]);
        cmd.assert().success();
    }
}

#[test]
fn test_mr_and_issue_extended_subcommands_help() {
    for sub in ["approve", "unapprove", "approvals", "note", "notes"] {
        let mut cmd = Command::cargo_bin("gitlab").unwrap();
        cmd.args(["mr", sub, "--help"]);
        cmd.assert().success();
    }

    for sub in ["note", "notes"] {
        let mut cmd = Command::cargo_bin("gitlab").unwrap();
        cmd.args(["issue", sub, "--help"]);
        cmd.assert().success();
    }
}

#[test]
fn test_project_scoped_command_fails_without_repo() {
    let mut cmd = Command::cargo_bin("gitlab").unwrap();
    // Run from root or temp directory with mock token to verify missing repo error
    cmd.args(["release", "list"])
        .env("GITLAB_TOKEN", "mock-token")
        .current_dir(std::env::temp_dir());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No GitLab project specified"));
}

#[test]
fn test_mr_review_subcommands_help() {
    for sub in [
        "diff",
        "discussions",
        "comment",
        "reply",
        "resolve",
        "note-edit",
        "update",
        "relink",
        "find",
    ] {
        let mut cmd = Command::cargo_bin("gitlab").unwrap();
        cmd.args(["mr", sub, "--help"]);
        cmd.assert().success();
    }
}

#[test]
fn test_markdown_and_file_commands_help() {
    let mut cmd_md = Command::cargo_bin("gitlab").unwrap();
    cmd_md.args(["markdown", "render", "--help"]);
    cmd_md.assert().success();

    let mut cmd_file = Command::cargo_bin("gitlab").unwrap();
    cmd_file.args(["file", "--help"]);
    cmd_file.assert().success();
}

#[test]
fn test_ci_list_and_trace_help() {
    let mut cmd_list = Command::cargo_bin("gitlab").unwrap();
    cmd_list.args(["ci", "list", "--help"]);
    cmd_list
        .assert()
        .success()
        .stdout(predicate::str::contains("--sha"))
        .stdout(predicate::str::contains("--ref"));

    let mut cmd_trace = Command::cargo_bin("gitlab").unwrap();
    cmd_trace.args(["ci", "trace", "--help"]);
    cmd_trace
        .assert()
        .success()
        .stdout(predicate::str::contains("--tail"))
        .stdout(predicate::str::contains("--no-color"));
}

#[test]
fn test_mr_update_alias_b_and_issue_file_flags() {
    let mut cmd_update = Command::cargo_bin("gitlab").unwrap();
    cmd_update.args(["mr", "update", "--help"]);
    cmd_update
        .assert()
        .success()
        .stdout(predicate::str::contains("-b"))
        .stdout(predicate::str::contains("-f, --file"));

    let mut cmd_issue_create = Command::cargo_bin("gitlab").unwrap();
    cmd_issue_create.args(["issue", "create", "--help"]);
    cmd_issue_create
        .assert()
        .success()
        .stdout(predicate::str::contains("-f, --file"));

    let mut cmd_issue_note = Command::cargo_bin("gitlab").unwrap();
    cmd_issue_note.args(["issue", "note", "--help"]);
    cmd_issue_note
        .assert()
        .success()
        .stdout(predicate::str::contains("-f, --file"));
}

#[test]
fn test_mr_comment_line_without_path_fails() {
    let mut cmd = Command::cargo_bin("gitlab").unwrap();
    cmd.args(["mr", "comment", "12", "-b", "Nice code", "--line", "42"])
        .env("GITLAB_TOKEN", "mock-token")
        .arg("--repo")
        .arg("owner/repo");
    cmd.assert().failure().stderr(predicate::str::contains(
        "--path is required when specifying --line or --old-line",
    ));
}
