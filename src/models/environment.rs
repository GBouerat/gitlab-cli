//! Environment domain models for GitLab projects.

use serde::{Deserialize, Serialize};

/// Representation of a GitLab deployment environment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Environment {
    /// Numeric ID of the environment.
    pub id: u64,
    /// Environment name (e.g. `production`, `staging`, `review/feature`).
    pub name: String,
    /// Environment slug identifier.
    #[serde(default)]
    pub slug: Option<String>,
    /// External deployment URL.
    #[serde(default)]
    pub external_url: Option<String>,
    /// Environment lifecycle state (`available`, `stopped`).
    #[serde(default)]
    pub state: Option<String>,
    /// Creation timestamp in ISO 8601 format.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Update timestamp in ISO 8601 format.
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Auto-stop timestamp in ISO 8601 format.
    #[serde(default)]
    pub auto_stop_at: Option<String>,
    /// Web URL to view environment in browser.
    #[serde(default)]
    pub web_url: Option<String>,
}
