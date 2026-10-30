//! Code snippet domain models.

use super::user::User;
use serde::{Deserialize, Serialize};

/// Representation of a GitLab Code Snippet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Snippet {
    /// Unique numeric Snippet ID.
    pub id: u64,
    /// Snippet title.
    pub title: String,
    /// Primary filename.
    #[serde(default)]
    pub file_name: Option<String>,
    /// Detailed description.
    #[serde(default)]
    pub description: Option<String>,
    /// Visibility level (`public`, `internal`, `private`).
    pub visibility: String,
    /// User who authored the snippet.
    #[serde(default)]
    pub author: Option<User>,
    /// Creation timestamp in ISO 8601 format.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Last update timestamp in ISO 8601 format.
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Web URL to view the snippet in a browser.
    #[serde(default)]
    pub web_url: Option<String>,
    /// Raw download URL for snippet content.
    #[serde(default)]
    pub raw_url: Option<String>,
}
