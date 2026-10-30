//! Deployment domain models for GitLab projects.

use super::environment::Environment;
use super::user::User;
use serde::{Deserialize, Serialize};

/// Representation of a GitLab deployment to an environment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Deployment {
    /// Numeric ID of the deployment.
    pub id: u64,
    /// Internal IID of the deployment.
    pub iid: u64,
    /// Git ref deployed.
    pub r#ref: String,
    /// Full commit SHA deployed.
    pub sha: String,
    /// Deployment status (`created`, `running`, `success`, `failed`, `canceled`).
    pub status: String,
    /// Creation timestamp in ISO 8601 format.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Update timestamp in ISO 8601 format.
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Target environment details.
    #[serde(default)]
    pub environment: Option<Environment>,
    /// User who triggered the deployment.
    #[serde(default)]
    pub user: Option<User>,
}
