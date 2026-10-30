//! CI/CD Variable domain models.

use serde::{Deserialize, Serialize};

/// Representation of a GitLab CI/CD Project Variable.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CiVariable {
    /// Variable name/key.
    pub key: String,
    /// Variable value.
    pub value: String,
    /// Variable type (`env_var` or `file`).
    #[serde(rename = "variable_type", default = "default_variable_type")]
    pub variable_type: String,
    /// Whether the variable value is masked in job logs.
    #[serde(default)]
    pub masked: Option<bool>,
    /// Whether the variable is only passed to pipelines running on protected branches/tags.
    #[serde(default)]
    pub protected: Option<bool>,
    /// Environment scope filter (e.g. `*`, `production`, `staging`).
    #[serde(default = "default_environment_scope")]
    pub environment_scope: String,
    /// Detailed description of the variable purpose.
    #[serde(default)]
    pub description: Option<String>,
}

fn default_variable_type() -> String {
    "env_var".to_string()
}

fn default_environment_scope() -> String {
    "*".to_string()
}
