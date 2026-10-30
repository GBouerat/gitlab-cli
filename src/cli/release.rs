//! CLI definitions for Release commands.

use clap::{Args, Subcommand};

/// Top-level arguments for `gitlab release`.
#[derive(Args, Debug)]
pub struct ReleaseArgs {
    /// Release subcommands.
    #[command(subcommand)]
    pub command: ReleaseCommands,
}

/// Available release subcommands.
#[derive(Subcommand, Debug)]
pub enum ReleaseCommands {
    /// List releases for the project.
    List(ReleaseListArgs),
    /// View detailed information for a release.
    View(ReleaseViewArgs),
    /// Create a new release.
    Create(ReleaseCreateArgs),
    /// Delete a release.
    Delete(ReleaseDeleteArgs),
}

/// Arguments for `gitlab release list`.
#[derive(Args, Debug)]
pub struct ReleaseListArgs {
    /// Maximum number of releases to list.
    #[arg(short = 'l', long, default_value = "30")]
    pub limit: usize,
}

/// Arguments for `gitlab release view`.
#[derive(Args, Debug)]
pub struct ReleaseViewArgs {
    /// Tag name of the release (e.g. `v1.0.0`).
    pub tag: String,

    /// Open release in web browser.
    #[arg(short = 'w', long)]
    pub web: bool,
}

/// Arguments for `gitlab release create`.
#[derive(Args, Debug)]
pub struct ReleaseCreateArgs {
    /// Tag name for the release (e.g. `v1.2.0`).
    #[arg(short = 't', long)]
    pub tag: String,

    /// Release display name (defaults to tag name).
    #[arg(short = 'n', long)]
    pub name: Option<String>,

    /// Detailed release notes / description (Markdown).
    #[arg(short = 'd', long)]
    pub description: Option<String>,

    /// Target commitish ref (commit SHA, branch name, or tag).
    #[arg(short = 'r', long)]
    pub ref_name: Option<String>,
}

/// Arguments for `gitlab release delete`.
#[derive(Args, Debug)]
pub struct ReleaseDeleteArgs {
    /// Tag name of the release to delete.
    pub tag: String,
}
