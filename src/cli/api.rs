//! CLI definitions for direct REST API requests (`gitlab api`).

use clap::Args;

/// Arguments for `gitlab api`.
#[derive(Args, Debug)]
pub struct ApiArgs {
    /// Target API endpoint path (e.g., `projects/:id/pipelines` or `user`).
    pub endpoint: String,

    /// HTTP method to use (defaults to `GET`).
    #[arg(short = 'X', long = "method", default_value = "GET")]
    pub method: String,

    /// Request fields as `key=value` pairs added to body or query parameters.
    #[arg(short = 'F', long = "field")]
    pub fields: Vec<String>,
}
