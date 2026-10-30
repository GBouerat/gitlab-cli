//! Release domain models for GitLab projects.

use super::user::User;
use serde::{Deserialize, Serialize};

/// Representation of a GitLab Project Release.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Release {
    /// Human-friendly release name.
    pub name: String,
    /// Git tag name associated with the release.
    pub tag_name: String,
    /// Detailed Markdown release notes / description.
    #[serde(default)]
    pub description: Option<String>,
    /// Creation timestamp in ISO 8601 format.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Released timestamp in ISO 8601 format.
    #[serde(default)]
    pub released_at: Option<String>,
    /// User who created the release.
    #[serde(default)]
    pub author: Option<User>,
    /// Commit details linked to the tag.
    #[serde(default)]
    pub commit: Option<ReleaseCommit>,
    /// Release asset counts and links.
    #[serde(default)]
    pub assets: Option<ReleaseAssets>,
    /// Web URL to view the release in a browser.
    #[serde(default)]
    pub web_url: Option<String>,
}

/// Commit summary associated with a release.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReleaseCommit {
    /// Full commit SHA.
    pub id: String,
    /// Short commit SHA.
    #[serde(default)]
    pub short_id: Option<String>,
    /// Commit title / first line of message.
    #[serde(default)]
    pub title: Option<String>,
}

/// Assets attached to a release.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReleaseAssets {
    /// Count of attached links.
    #[serde(default)]
    pub count: Option<u64>,
    /// External or uploaded asset links.
    #[serde(default)]
    pub links: Vec<ReleaseLink>,
}

/// A specific link/asset in a release.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReleaseLink {
    /// Unique numeric ID of the asset link.
    pub id: u64,
    /// Display name of the asset.
    pub name: String,
    /// Download URL.
    pub url: String,
    /// Link type (e.g., `other`, `runbook`, `image`, `package`).
    #[serde(default)]
    pub link_type: Option<String>,
}
