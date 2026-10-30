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
