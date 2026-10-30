//! User and membership domain models.

use serde::{Deserialize, Serialize};

/// Basic GitLab user representation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct User {
    /// Global unique user ID.
    pub id: u64,
    /// GitLab username handle.
    pub username: String,
    /// Full display name of the user.
    pub name: String,
    /// Current account state (e.g., `active`, `blocked`, `deactivated`).
    #[serde(default)]
    pub state: Option<String>,
    /// URL to user's avatar image.
    #[serde(default)]
    pub avatar_url: Option<String>,
    /// Web interface URL for the user profile.
    #[serde(default)]
    pub web_url: Option<String>,
}
