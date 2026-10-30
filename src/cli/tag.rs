//! CLI definitions for Git Tag commands.

use clap::{Args, Subcommand};

/// Top-level arguments for `gitlab tag`.
#[derive(Args, Debug)]
pub struct TagArgs {
    /// Tag subcommands.
    #[command(subcommand)]
    pub command: TagCommands,
}

/// Available tag subcommands.
#[derive(Subcommand, Debug)]
pub enum TagCommands {
    /// List repository tags.
    List(TagListArgs),
    /// View detailed information for a tag.
    View(TagViewArgs),
    /// Create a new tag.
    Create(TagCreateArgs),
    /// Delete a tag.
    Delete(TagDeleteArgs),
}

/// Arguments for `gitlab tag list`.
#[derive(Args, Debug)]
pub struct TagListArgs {
    /// Maximum number of tags to list.
    #[arg(short = 'l', long, default_value = "50")]
    pub limit: usize,
}

/// Arguments for `gitlab tag view`.
#[derive(Args, Debug)]
pub struct TagViewArgs {
    /// Tag name (e.g. `v1.0.0`).
    pub name: String,
}

/// Arguments for `gitlab tag create`.
#[derive(Args, Debug)]
pub struct TagCreateArgs {
    /// Tag name to create.
    pub name: String,

    /// Target ref/commit to tag (e.g. `main` or SHA).
    #[arg(short = 'r', long, default_value = "main")]
    pub r#ref: String,

    /// Optional annotation message.
    #[arg(short = 'm', long)]
    pub message: Option<String>,
}

/// Arguments for `gitlab tag delete`.
#[derive(Args, Debug)]
pub struct TagDeleteArgs {
    /// Tag name to delete.
    pub name: String,
}
