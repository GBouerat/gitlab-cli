//! Git context discovery and repository remote parsing.
//!
//! This module provides functions to inspect the current local Git repository,
//! extracting remote URLs, identifying GitLab hostnames and project paths (namespaces),
//! and discovering the active branch name.

use std::path::Path;
use std::process::Command;
use url::Url;

/// Information extracted from a Git remote pointing to a GitLab repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitContext {
    /// The GitLab instance hostname (e.g., `gitlab.com` or `gitlab.example.com`).
    pub host: String,
    /// The namespaced project path without trailing `.git` (e.g., `group/subgroup/project`).
    pub project_path: String,
}

impl GitContext {
    /// Creates a new `GitContext` from a hostname and a project path.
    ///
    /// # Arguments
    ///
    /// * `host` - The GitLab instance hostname.
    /// * `project_path` - The namespaced project path (e.g., `owner/repo`).
    pub fn new(host: impl Into<String>, project_path: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            project_path: project_path.into(),
        }
    }
}

/// Parses a Git remote URL into a [`GitContext`] containing the host and namespaced project path.
///
/// Supports:
/// - Standard SSH URLs: `git@gitlab.com:owner/repo.git`
/// - SSH URLs with protocol: `ssh://git@gitlab.example.com:2222/owner/subgroup/repo.git`
/// - HTTPS / HTTP URLs: `https://gitlab.com/owner/repo.git` or `http://host/owner/repo`
///
/// # Arguments
///
/// * `remote_url` - The remote URL string from Git configuration.
///
/// # Errors
///
/// Returns an error if the URL format cannot be recognized or does not contain a valid project path.
pub fn parse_remote_url(remote_url: &str) -> anyhow::Result<GitContext> {
    let trimmed = remote_url.trim();

    // Check for SSH scp-like syntax: [user@]host:owner/repo.git
    if !trimmed.contains("://") {
        if let Some((user_host, path_with_git)) = trimmed.split_once(':') {
            let host = if let Some((_user, h)) = user_host.split_once('@') {
                h
            } else {
                user_host
            };

            // Strip any leading slashes and trailing .git
            let clean_path = path_with_git
                .trim_start_matches('/')
                .trim_end_matches(".git");

            if !host.is_empty() && !clean_path.is_empty() && !host.contains('/') {
                return Ok(GitContext::new(host.to_string(), clean_path.to_string()));
            }
        }
    }

    // Try standard URL parsing (HTTP, HTTPS, ssh://)
    if let Ok(parsed) = Url::parse(trimmed) {
        if let Some(host) = parsed.host_str() {
            let path = parsed
                .path()
                .trim_start_matches('/')
                .trim_end_matches(".git");
            if !path.is_empty() {
                return Ok(GitContext::new(host.to_string(), path.to_string()));
            }
        }
    }

    anyhow::bail!("Could not parse GitLab remote URL: '{remote_url}'")
}

/// Resolves the current GitLab context from the local Git repository in the given directory.
///
/// Queries `git remote get-url origin` using the `git` binary.
///
/// # Arguments
///
/// * `dir` - The directory containing the Git repository (or a subdirectory).
///
/// # Errors
///
/// Returns an error if `git` is not installed, the directory is not inside a Git repo,
/// or the remote `origin` cannot be parsed.
pub fn resolve_git_context(dir: &Path) -> anyhow::Result<GitContext> {
    let output = Command::new("git")
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .current_dir(dir)
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to get git remote 'origin': {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    let raw_url = String::from_utf8(output.stdout)?;
    parse_remote_url(&raw_url)
}

/// Retrieves the current checked out Git branch in the given directory.
///
/// Queries `git rev-parse --abbrev-ref HEAD`.
///
/// # Arguments
///
/// * `dir` - The directory containing the Git repository.
///
/// # Errors
///
/// Returns an error if the Git command fails or the branch name cannot be determined.
pub fn get_current_branch(dir: &Path) -> anyhow::Result<String> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .current_dir(dir)
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to determine current branch: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    let branch = String::from_utf8(output.stdout)?.trim().to_string();
    if branch.is_empty() || branch == "HEAD" {
        anyhow::bail!("Detached HEAD or empty branch name");
    }

    Ok(branch)
}

/// Retrieves the current HEAD commit SHA (40 hexadecimal characters) in the given directory.
///
/// Queries `git rev-parse HEAD`.
///
/// # Arguments
///
/// * `dir` - The directory containing the Git repository.
///
/// # Errors
///
/// Returns an error if the Git command fails or HEAD cannot be resolved.
pub fn get_head_sha(dir: &Path) -> anyhow::Result<String> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .current_dir(dir)
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to get HEAD commit SHA: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    let sha = String::from_utf8(output.stdout)?.trim().to_string();
    if sha.is_empty() {
        anyhow::bail!("Empty HEAD SHA returned from git");
    }

    Ok(sha)
}

/// Retrieves the subject line of a commit from the local Git repository.
///
/// Queries `git log -1 --format=%s <sha>`.
///
/// # Arguments
///
/// * `dir` - The directory containing the Git repository.
/// * `sha` - Full commit SHA or commit prefix.
///
/// # Errors
///
/// Returns an error if the commit does not exist locally or cannot be queried.
pub fn get_commit_subject(dir: &Path, sha: &str) -> anyhow::Result<String> {
    let output = Command::new("git")
        .arg("log")
        .arg("-1")
        .arg("--format=%s")
        .arg(sha)
        .current_dir(dir)
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "Commit '{}' not found in local git repository: {}",
            sha,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    let subject = String::from_utf8(output.stdout)?.trim().to_string();
    if subject.is_empty() {
        anyhow::bail!("Commit '{}' has an empty subject line", sha);
    }

    Ok(subject)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ssh_remote() {
        let ctx = parse_remote_url("git@gitlab.com:owner/repo.git").unwrap();
        assert_eq!(ctx.host, "gitlab.com");
        assert_eq!(ctx.project_path, "owner/repo");
    }

    #[test]
    fn test_parse_ssh_custom_user() {
        let ctx =
            parse_remote_url("custom-user@gitlab.mycorp.org:team/deep/sub/project.git").unwrap();
        assert_eq!(ctx.host, "gitlab.mycorp.org");
        assert_eq!(ctx.project_path, "team/deep/sub/project");
    }

    #[test]
    fn test_parse_ssh_remote_subgroups() {
        let ctx = parse_remote_url("git@gitlab.example.com:group/subgroup/deep/repo.git").unwrap();
        assert_eq!(ctx.host, "gitlab.example.com");
        assert_eq!(ctx.project_path, "group/subgroup/deep/repo");
    }

    #[test]
    fn test_parse_https_remote() {
        let ctx = parse_remote_url("https://gitlab.com/owner/project.git").unwrap();
        assert_eq!(ctx.host, "gitlab.com");
        assert_eq!(ctx.project_path, "owner/project");
    }

    #[test]
    fn test_parse_https_without_git_suffix() {
        let ctx = parse_remote_url("https://gitlab.mycompany.org/team/project").unwrap();
        assert_eq!(ctx.host, "gitlab.mycompany.org");
        assert_eq!(ctx.project_path, "team/project");
    }

    #[test]
    fn test_parse_ssh_protocol_url() {
        let ctx = parse_remote_url("ssh://git@gitlab.example.com:2222/org/project.git").unwrap();
        assert_eq!(ctx.host, "gitlab.example.com");
        assert_eq!(ctx.project_path, "org/project");
    }

    #[test]
    fn test_invalid_urls() {
        assert!(parse_remote_url("not-a-valid-url").is_err());
        assert!(parse_remote_url("git@gitlab.com:").is_err());
    }
}
