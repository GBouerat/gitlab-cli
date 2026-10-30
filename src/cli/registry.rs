//! CLI definitions for Container Registry commands.

use clap::{Args, Subcommand};

/// Top-level arguments for `gitlab registry`.
#[derive(Args, Debug)]
pub struct RegistryArgs {
    /// Registry subcommands.
    #[command(subcommand)]
    pub command: RegistryCommands,
}

/// Available container registry subcommands.
#[derive(Subcommand, Debug)]
pub enum RegistryCommands {
    /// List container repositories for the project.
    List(RegistryListArgs),
    /// List image tags in a container repository.
    Tags(RegistryTagsArgs),
    /// Delete a container image tag.
    DeleteTag(RegistryDeleteTagArgs),
    /// Delete an entire container repository.
    DeleteRepo(RegistryDeleteRepoArgs),
}

/// Arguments for `gitlab registry list`.
#[derive(Args, Debug)]
pub struct RegistryListArgs {
    /// Maximum number of repositories to list.
    #[arg(short = 'l', long, default_value = "30")]
    pub limit: usize,
}

/// Arguments for `gitlab registry tags`.
#[derive(Args, Debug)]
pub struct RegistryTagsArgs {
    /// Container repository numeric ID.
    pub repository_id: u64,

    /// Maximum number of tags to list.
    #[arg(short = 'l', long, default_value = "50")]
    pub limit: usize,
}

/// Arguments for `gitlab registry delete-tag`.
#[derive(Args, Debug)]
pub struct RegistryDeleteTagArgs {
    /// Container repository numeric ID.
    pub repository_id: u64,

    /// Tag name to delete (e.g. `latest` or `v1.0.0`).
    pub tag_name: String,
}

/// Arguments for `gitlab registry delete-repo`.
#[derive(Args, Debug)]
pub struct RegistryDeleteRepoArgs {
    /// Container repository numeric ID to delete.
    pub repository_id: u64,
}
