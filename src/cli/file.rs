//! CLI definitions for printing repository files via web URLs.

use clap::Args;

/// Arguments for `gitlab file <URL>`.
#[derive(Args, Debug)]
#[command(
    after_help = "Examples:\n  gitlab file https://gitlab.com/gitlab-org/gitlab/-/blob/master/README.md\n  gitlab file https://gitlab.example.com/org/proj/-/blob/main/src/main.rs"
)]
pub struct FileArgs {
    /// Full GitLab web URL to the file blob (e.g. `https://gitlab.example.com/group/project/-/blob/main/path/to/file`).
    pub url: String,
}
