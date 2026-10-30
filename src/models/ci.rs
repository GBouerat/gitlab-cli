//! CI/CD Pipeline and Job domain models.

use serde::{Deserialize, Serialize};

/// Representation of a GitLab CI Pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Pipeline {
    /// Unique pipeline ID.
    pub id: u64,
    /// Internal project pipeline IID if available.
    #[serde(default)]
    pub iid: Option<u64>,
    /// Owning project ID.
    pub project_id: u64,
    /// Commit SHA triggered by the pipeline.
    pub sha: String,
    /// Target Git branch or tag name.
    #[serde(rename = "ref")]
    pub ref_name: String,
    /// Execution status (`success`, `failed`, `running`, `pending`, `canceled`, `skipped`, `manual`).
    pub status: String,
    /// Trigger source (e.g., `push`, `web`, `merge_request_event`, `schedule`).
    #[serde(default)]
    pub source: Option<String>,
    /// Creation timestamp in ISO 8601 format.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Last update timestamp in ISO 8601 format.
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Web interface URL for the pipeline.
    #[serde(default)]
    pub web_url: Option<String>,
    /// Execution duration in seconds.
    #[serde(default)]
    pub duration: Option<f64>,
    /// Time queued waiting for a runner in seconds.
    #[serde(default)]
    pub queued_duration: Option<f64>,
}

/// Representation of a GitLab CI Job inside a Pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Job {
    /// Unique numeric Job ID.
    pub id: u64,
    /// Name of the job as defined in `.gitlab-ci.yml`.
    pub name: String,
    /// Stage name (e.g., `build`, `test`, `deploy`).
    pub stage: String,
    /// Job status (`success`, `failed`, `running`, `pending`, `canceled`, `skipped`, `manual`).
    pub status: String,
    /// Branch or tag ref.
    #[serde(rename = "ref", default)]
    pub ref_name: Option<String>,
    /// Creation timestamp in ISO 8601 format.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Execution start timestamp in ISO 8601 format.
    #[serde(default)]
    pub started_at: Option<String>,
    /// Execution finish timestamp in ISO 8601 format.
    #[serde(default)]
    pub finished_at: Option<String>,
    /// Duration of job execution in seconds.
    #[serde(default)]
    pub duration: Option<f64>,
    /// Web URL to view the job in the browser.
    #[serde(default)]
    pub web_url: Option<String>,
    /// Whether job failure is allowed without failing the overall pipeline.
    #[serde(default)]
    pub allow_failure: Option<bool>,
}
