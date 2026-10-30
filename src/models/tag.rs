//! Git Tag domain models.

use super::release::ReleaseCommit;
use serde::{Deserialize, Serialize};

/// Representation of a Git repository tag.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tag {
    /// Tag name.
    pub name: String,
    /// Optional annotation message.
    #[serde(default)]
    pub message: Option<String>,
    /// Target commit details.
    #[serde(default)]
    pub commit: Option<ReleaseCommit>,
    /// Associated release summary if any.
    #[serde(default)]
    pub release: Option<TagRelease>,
    /// Whether the tag is protected.
    #[serde(default)]
    pub protected: Option<bool>,
}

/// Release summary associated with a tag.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TagRelease {
    /// Tag name.
    pub tag_name: String,
    /// Release description.
    #[serde(default)]
    pub description: Option<String>,
}
