//! Container Registry domain models for GitLab projects.

use serde::{Deserialize, Serialize};

/// Representation of a GitLab container registry repository.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryRepository {
    /// Numeric ID of the repository.
    pub id: u64,
    /// Repository name.
    pub name: String,
    /// Full image path (e.g. `registry.gitlab.com/group/project/image`).
    pub path: String,
    /// Location URL.
    pub location: String,
    /// Creation timestamp in ISO 8601 format.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Tag count if requested.
    #[serde(default)]
    pub tag_count: Option<usize>,
}

/// Representation of a tag in a container registry repository.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryTag {
    /// Tag name (e.g. `latest`, `v1.0.0`).
    pub name: String,
    /// Full image path with tag.
    pub path: String,
    /// Tag location URL.
    pub location: String,
    /// Revision SHA digest.
    #[serde(default)]
    pub revision: Option<String>,
    /// Short revision SHA digest.
    #[serde(default)]
    pub short_revision: Option<String>,
    /// Total tag size in bytes.
    #[serde(default)]
    pub total_size: Option<u64>,
    /// Creation timestamp in ISO 8601 format.
    #[serde(default)]
    pub created_at: Option<String>,
}
