//! Merge Request (MR) domain models.

use super::note::{DiscussionPosition, Note};
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
    /// Base, start, and head commit references for diff calculations.
    #[serde(default)]
    pub diff_refs: Option<DiffRefs>,
}

/// Commit references for calculating diffs on a Merge Request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DiffRefs {
    /// Base commit SHA.
    #[serde(default)]
    pub base_sha: Option<String>,
    /// Start commit SHA.
    #[serde(default)]
    pub start_sha: Option<String>,
    /// Head commit SHA.
    #[serde(default)]
    pub head_sha: Option<String>,
}

/// Represents a changed file diff entry in a Merge Request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MergeRequestDiff {
    /// Old path of the changed file.
    pub old_path: String,
    /// New path of the changed file.
    pub new_path: String,
    /// File mode before changes.
    #[serde(default)]
    pub a_mode: Option<String>,
    /// File mode after changes.
    #[serde(default)]
    pub b_mode: Option<String>,
    /// Whether the file is newly added.
    #[serde(default)]
    pub new_file: bool,
    /// Whether the file was renamed.
    #[serde(default)]
    pub renamed_file: bool,
    /// Whether the file was deleted.
    #[serde(default)]
    pub deleted_file: bool,
    /// Unified diff string.
    #[serde(default)]
    pub diff: String,
}

/// A discussion thread on a merge request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Discussion {
    /// Discussion thread unique identifier.
    pub id: String,
    /// Indicates whether this discussion represents an individual note.
    #[serde(default)]
    pub individual_note: Option<bool>,
    /// Notes belonging to this discussion thread.
    #[serde(default)]
    pub notes: Vec<Note>,
}

impl Discussion {
    /// Returns `true` if any note in the discussion is resolvable.
    pub fn is_resolvable(&self) -> bool {
        self.notes.iter().any(|n| n.resolvable)
    }

    /// Returns `true` if the discussion is resolvable and all resolvable notes are resolved.
    pub fn is_resolved(&self) -> bool {
        let resolvable: Vec<&Note> = self.notes.iter().filter(|n| n.resolvable).collect();
        !resolvable.is_empty() && resolvable.iter().all(|n| n.resolved)
    }

    /// Returns the inline code position if this discussion is attached to a file diff.
    pub fn position(&self) -> Option<&DiscussionPosition> {
        self.notes.iter().find_map(|n| n.position.as_ref())
    }
}

/// Commit entry associated with a merge request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MrCommit {
    /// Full 40-character commit SHA.
    pub id: String,
    /// Short SHA prefix (typically 8 characters).
    #[serde(default)]
    pub short_id: Option<String>,
    /// Commit subject/title.
    pub title: String,
    /// Author name.
    #[serde(default)]
    pub author_name: Option<String>,
    /// Commit timestamp in ISO 8601 format.
    #[serde(default)]
    pub created_at: Option<String>,
}

/// Result of rendering GitLab Flavored Markdown.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarkdownRenderResult {
    /// Rendered HTML string.
    pub html: String,
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
