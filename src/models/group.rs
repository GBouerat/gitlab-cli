//! Group and subgroup domain models.

use serde::{Deserialize, Serialize};

/// Representation of a GitLab Group.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Group {
    /// Unique numeric Group ID.
    pub id: u64,
    /// Display name of the group.
    pub name: String,
    /// Path slug (e.g. `my-team`).
    pub path: String,
    /// Full namespaced group path (e.g. `parent-group/my-team`).
    pub full_path: String,
    /// Detailed description.
    #[serde(default)]
    pub description: Option<String>,
    /// Group visibility level (`private`, `internal`, or `public`).
    #[serde(default)]
    pub visibility: Option<String>,
    /// Web URL to view the group in the browser.
    #[serde(default)]
    pub web_url: Option<String>,
    /// Parent group ID if this is a subgroup.
    #[serde(default)]
    pub parent_id: Option<u64>,
}

/// Representation of a member inside a GitLab Group.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GroupMember {
    /// User ID.
    pub id: u64,
    /// Username handle.
    pub username: String,
    /// Display name.
    pub name: String,
    /// Current membership state (`active`, `blocked`).
    #[serde(default)]
    pub state: Option<String>,
    /// Numeric access level (e.g. 10 = Guest, 20 = Reporter, 30 = Developer, 40 = Maintainer, 50 = Owner).
    pub access_level: u32,
}

impl GroupMember {
    /// Returns human-readable role name corresponding to the GitLab numeric access level.
    pub fn role_name(&self) -> &'static str {
        match self.access_level {
            10 => "Guest",
            20 => "Reporter",
            30 => "Developer",
            40 => "Maintainer",
            50 => "Owner",
            _ => "Unknown",
        }
    }
}
