//! CLI definitions for Branch commands.

use clap::{Args, Subcommand};

/// Top-level arguments for `gitlab branch`.
#[derive(Args, Debug)]
pub struct BranchArgs {
    /// Branch subcommands.
    #[command(subcommand)]
    pub command: BranchCommands,
}

/// Available branch subcommands.
#[derive(Subcommand, Debug)]
pub enum BranchCommands {
    /// List repository branches.
    List(BranchListArgs),
    /// View detailed information about a branch.
    View(BranchViewArgs),
    /// Create a new branch.
    Create(BranchCreateArgs),
    /// Delete a branch.
    Delete(BranchDeleteArgs),
}

/// Arguments for `gitlab branch list`.
#[derive(Args, Debug)]
pub struct BranchListArgs {
    /// Maximum number of branches to list.
    #[arg(short = 'l', long, default_value = "30")]
    pub limit: usize,
}

/// Arguments for `gitlab branch view`.
#[derive(Args, Debug)]
pub struct BranchViewArgs {
    /// Branch name (defaults to currently checked out branch).
    pub branch: Option<String>,

    /// Open branch in web browser.
    #[arg(short = 'w', long)]
    pub web: bool,
}

/// Arguments for `gitlab branch create`.
#[derive(Args, Debug)]
pub struct BranchCreateArgs {
    /// Name of the new branch.
    pub branch: String,

    /// Target ref (branch, commit SHA, or tag) from which to create the branch.
    #[arg(short = 'r', long)]
    pub ref_name: Option<String>,
}

/// Arguments for `gitlab branch delete`.
#[derive(Args, Debug)]
pub struct BranchDeleteArgs {
    /// Branch name to delete.
    pub branch: String,
}
