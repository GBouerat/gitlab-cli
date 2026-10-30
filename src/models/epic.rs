//! Epic domain models for GitLab groups.

use super::user::User;
use serde::{Deserialize, Serialize};

/// Representation of a GitLab Group Epic.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Epic {
    /// Numeric ID of the epic.
    pub id: u64,
    /// Internal IID of the epic within the group.
    pub iid: u64,
    /// Group ID containing the epic.
    pub group_id: u64,
    /// Epic title.
    pub title: String,
    /// Detailed Markdown description.
    #[serde(default)]
    pub description: Option<String>,
    /// State of the epic (`opened`, `closed`).
    #[serde(default)]
    pub state: Option<String>,
    /// Epic author.
    #[serde(default)]
    pub author: Option<User>,
    /// Target start date in `YYYY-MM-DD` format.
    #[serde(default)]
    pub start_date: Option<String>,
    /// Target due date in `YYYY-MM-DD` format.
    #[serde(default)]
    pub due_date: Option<String>,
    /// Creation timestamp in ISO 8601 format.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Update timestamp in ISO 8601 format.
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Web URL to view epic in browser.
    #[serde(default)]
    pub web_url: Option<String>,
}
