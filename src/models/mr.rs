//! Merge Request (MR) domain models.

use super::user::User;
use serde::{Deserialize, Serialize};

/// Representation of a GitLab Merge Request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MergeRequest {
    /// Global unique MR ID.
    pub id: u64,
    /// Internal project-scoped ID (displayed in web UI as `!123`).
    pub iid: u64,
    /// ID of the target project.
    pub project_id: u64,
    /// MR title.
    pub title: String,
    /// Detailed description or body.
    #[serde(default)]
    pub description: Option<String>,
    /// State of the merge request (`opened`, `closed`, `merged`, `locked`).
    pub state: String,
    /// Creation timestamp in ISO 8601 format.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Last update timestamp in ISO 8601 format.
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Target destination branch (e.g. `main`).
    pub target_branch: String,
    /// Source feature branch containing the changes.
    pub source_branch: String,
    /// User who authored the merge request.
    #[serde(default)]
    pub author: Option<User>,
    /// Single assignee if configured.
    #[serde(default)]
    pub assignee: Option<User>,
    /// Multiple assignees if configured.
    #[serde(default)]
    pub assignees: Option<Vec<User>>,
    /// List of assigned labels.
    #[serde(default)]
    pub labels: Vec<String>,
    /// Web URL to view the MR in a browser.
    #[serde(default)]
    pub web_url: Option<String>,
    /// Whether merge conflicts exist between source and target branches.
    #[serde(default)]
    pub has_conflicts: Option<bool>,
    /// Whether this MR is marked as a draft.
    #[serde(default)]
    pub draft: Option<bool>,
    /// Legacy draft indicator (`work_in_progress`).
    #[serde(default)]
    pub work_in_progress: Option<bool>,
    /// General merge status (`can_be_merged`, `cannot_be_merged`, etc.).
    #[serde(default)]
    pub merge_status: Option<String>,
    /// Detailed merge status (e.g. `mergeable`, `conflict`, `ci_must_pass`).
    #[serde(default)]
    pub detailed_merge_status: Option<String>,
    /// Commit SHA at HEAD of source branch.
    #[serde(default)]
    pub sha: Option<String>,
    /// Whether source branch commits will be squashed upon merge.
    #[serde(default)]
    pub squash: Option<bool>,
    /// Number of comments / notes on the MR.
    #[serde(default)]
    pub user_notes_count: Option<u64>,
}

impl MergeRequest {
    /// Returns `true` if the merge request is marked as a Draft / WIP.
    pub fn is_draft(&self) -> bool {
        self.draft.unwrap_or(false)
            || self.work_in_progress.unwrap_or(false)
            || self.title.starts_with("Draft:")
            || self.title.starts_with("WIP:")
    }
}
