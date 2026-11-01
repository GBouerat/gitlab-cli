//! Models for notes (comments) and merge request approvals.

use super::user::User;
use serde::{Deserialize, Serialize};

/// Representation of a comment or system note on an Issue or Merge Request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Note {
    /// Numeric ID of the note.
    pub id: u64,
    /// Markdown body content of the note.
    pub body: String,
    /// Author of the note.
    #[serde(default)]
    pub author: Option<User>,
    /// Creation timestamp in ISO 8601 format.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Update timestamp in ISO 8601 format.
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Whether this is an automated system note rather than a user comment.
    #[serde(default)]
    pub system: bool,
    /// Whether this note is resolvable.
    #[serde(default)]
    pub resolvable: bool,
    /// Whether this note has been resolved.
    #[serde(default)]
    pub resolved: bool,
    /// Diff position details if this note is an inline code discussion.
    #[serde(default)]
    pub position: Option<DiscussionPosition>,
    /// Type of note (e.g. `DiffNote` or `DiscussionNote`).
    #[serde(rename = "type", default)]
    pub note_type: Option<String>,
    /// Web URL to view the note in the GitLab UI.
    #[serde(default)]
    pub web_url: Option<String>,
}

/// Position of a diff note within a merge request source file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DiscussionPosition {
    /// Base commit SHA.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_sha: Option<String>,
    /// Start commit SHA.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_sha: Option<String>,
    /// Head commit SHA.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_sha: Option<String>,
    /// Old file path before change.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub old_path: Option<String>,
    /// New file path after change.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_path: Option<String>,
    /// Position type (typically "text").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position_type: Option<String>,
    /// Line number in old file version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub old_line: Option<u64>,
    /// Line number in new file version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_line: Option<u64>,
}

/// Representation of the approval state for a Merge Request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MrApprovals {
    /// Merge Request internal ID.
    pub id: u64,
    /// Project ID containing the Merge Request.
    #[serde(default)]
    pub project_id: Option<u64>,
    /// Merge Request internal IID.
    #[serde(default)]
    pub iid: Option<u64>,
    /// Required approvals count.
    #[serde(default)]
    pub approvals_required: u32,
    /// Approvals left before merge.
    #[serde(default)]
    pub approvals_left: u32,
    /// List of users who have approved the Merge Request.
    #[serde(default)]
    pub approved_by: Vec<ApprovedByUser>,
}

/// User entry in the approval list of a Merge Request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApprovedByUser {
    /// User details.
    pub user: User,
}
