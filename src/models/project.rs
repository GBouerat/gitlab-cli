//! GitLab Project (repository) domain models.

use serde::{Deserialize, Serialize};

/// Detailed representation of a GitLab Project.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Project {
    /// Unique numeric project ID.
    pub id: u64,
    /// Short display name of the project.
    pub name: String,
    /// Full project name including parent namespaces (e.g. `GitLab Org / GitLab CLI`).
    #[serde(default)]
    pub name_with_namespace: Option<String>,
    /// Repository path slug (e.g. `gitlab-cli`).
    pub path: String,
    /// Full namespaced project path (e.g. `group/subgroup/gitlab-cli`).
    pub path_with_namespace: String,
    /// Project description.
    #[serde(default)]
    pub description: Option<String>,
    /// Default Git branch (typically `main` or `master`).
    #[serde(default)]
    pub default_branch: Option<String>,
    /// Project visibility level (`private`, `internal`, or `public`).
    #[serde(default)]
    pub visibility: Option<String>,
    /// SSH clone URL.
    #[serde(default)]
    pub ssh_url_to_repo: Option<String>,
    /// HTTP(S) clone URL.
    #[serde(default)]
    pub http_url_to_repo: Option<String>,
    /// Web interface URL for the project.
    #[serde(default)]
    pub web_url: Option<String>,
    /// Number of stars given to the project.
    #[serde(default)]
    pub star_count: Option<u64>,
    /// Number of project forks.
    #[serde(default)]
    pub forks_count: Option<u64>,
    /// Number of currently open issues.
    #[serde(default)]
    pub open_issues_count: Option<u64>,
    /// Creation timestamp in ISO 8601 format.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Last activity timestamp in ISO 8601 format.
    #[serde(default)]
    pub last_activity_at: Option<String>,
}
