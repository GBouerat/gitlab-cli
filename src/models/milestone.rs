//! Milestone domain models for GitLab projects and groups.

use serde::{Deserialize, Serialize};

/// Representation of a GitLab Milestone.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Milestone {
    /// Numeric ID of the milestone.
    pub id: u64,
    /// Internal IID of the milestone within the project or group.
    #[serde(default)]
    pub iid: Option<u64>,
    /// Project ID containing the milestone (if project-level).
    #[serde(default)]
    pub project_id: Option<u64>,
    /// Group ID containing the milestone (if group-level).
    #[serde(default)]
    pub group_id: Option<u64>,
    /// Milestone title.
    pub title: String,
    /// Detailed Markdown description.
    #[serde(default)]
    pub description: Option<String>,
    /// State of the milestone (`active`, `closed`).
    #[serde(default)]
    pub state: Option<String>,
    /// Target due date in `YYYY-MM-DD` format.
    #[serde(default)]
    pub due_date: Option<String>,
    /// Start date in `YYYY-MM-DD` format.
    #[serde(default)]
    pub start_date: Option<String>,
    /// Creation timestamp in ISO 8601 format.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Update timestamp in ISO 8601 format.
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Web URL to view milestone in browser.
    #[serde(default)]
    pub web_url: Option<String>,
}
