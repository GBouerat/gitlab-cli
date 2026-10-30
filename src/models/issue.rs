//! GitLab Issue domain models.

use super::user::User;
use serde::{Deserialize, Serialize};

/// Detailed representation of a GitLab Issue.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Issue {
    /// Global unique issue ID.
    pub id: u64,
    /// Internal project-scoped issue number (displayed in UI as `#123`).
    pub iid: u64,
    /// ID of the owning project.
    pub project_id: u64,
    /// Issue title.
    pub title: String,
    /// Detailed issue description (in Markdown).
    #[serde(default)]
    pub description: Option<String>,
    /// Current issue state (`opened` or `closed`).
    pub state: String,
    /// Creation timestamp in ISO 8601 format.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Last update timestamp in ISO 8601 format.
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Closed timestamp in ISO 8601 format if applicable.
    #[serde(default)]
    pub closed_at: Option<String>,
    /// User who reported the issue.
    #[serde(default)]
    pub author: Option<User>,
    /// Single assignee if configured.
    #[serde(default)]
    pub assignee: Option<User>,
    /// Multiple assignees if configured.
    #[serde(default)]
    pub assignees: Option<Vec<User>>,
    /// List of attached label names.
    #[serde(default)]
    pub labels: Vec<String>,
    /// Web URL to view the issue in a browser.
    #[serde(default)]
    pub web_url: Option<String>,
    /// Whether the issue is confidential.
    #[serde(default)]
    pub confidential: Option<bool>,
    /// Number of user comments/notes on the issue.
    #[serde(default)]
    pub user_notes_count: Option<u64>,
}
