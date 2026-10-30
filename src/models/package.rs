//! Package domain models for GitLab projects.

use serde::{Deserialize, Serialize};

/// Representation of a GitLab project package (e.g., Maven, npm, PyPI, Generic, Conan).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Package {
    /// Numeric ID of the package.
    pub id: u64,
    /// Package name.
    pub name: String,
    /// Package version string.
    pub version: String,
    /// Package type/format (`npm`, `maven`, `pypi`, `generic`, etc.).
    pub package_type: String,
    /// Package status (`default`, `hidden`, `deprecated`, `processing`, `error`).
    #[serde(default)]
    pub status: Option<String>,
    /// Creation timestamp in ISO 8601 format.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Package files associated with this package.
    #[serde(default)]
    pub package_files: Option<Vec<PackageFile>>,
}

/// Representation of a file asset in a package.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageFile {
    /// Numeric ID of the file.
    pub id: u64,
    /// File name.
    pub file_name: String,
    /// File size in bytes.
    #[serde(default)]
    pub size: Option<u64>,
    /// SHA256 checksum.
    #[serde(default)]
    pub file_sha256: Option<String>,
    /// Creation timestamp in ISO 8601 format.
    #[serde(default)]
    pub created_at: Option<String>,
}
