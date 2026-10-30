//! Git branch domain models.

use serde::{Deserialize, Serialize};

/// Representation of a GitLab repository branch.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Branch {
    /// Branch name (e.g. `main` or `feature/xyz`).
    pub name: String,
    /// Whether this is the default repository branch.
    #[serde(default)]
    pub default: Option<bool>,
    /// Whether this branch has protection rules enabled.
    #[serde(default)]
    pub protected: Option<bool>,
    /// Whether this branch can be pushed to by the authenticated user.
    #[serde(default)]
    pub can_push: Option<bool>,
    /// Web URL to view the branch in a browser.
    #[serde(default)]
    pub web_url: Option<String>,
    /// The latest commit at HEAD of this branch.
    #[serde(default)]
    pub commit: Option<BranchCommit>,
}

/// Commit at HEAD of a branch.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BranchCommit {
    /// Full commit SHA.
    pub id: String,
    /// Short commit SHA.
    #[serde(default)]
    pub short_id: Option<String>,
    /// Commit title.
    #[serde(default)]
    pub title: Option<String>,
    /// Author name.
    #[serde(default)]
    pub author_name: Option<String>,
    /// Commit author timestamp.
    #[serde(default)]
    pub authored_date: Option<String>,
}
