//! CLI definitions for Deployment commands.

use clap::{Args, Subcommand};

/// Top-level arguments for `gitlab deploy` (or `gitlab deployment`).
#[derive(Args, Debug)]
pub struct DeploymentArgs {
    /// Deployment subcommands.
    #[command(subcommand)]
    pub command: DeploymentCommands,
}

/// Available deployment subcommands.
#[derive(Subcommand, Debug)]
pub enum DeploymentCommands {
    /// List project deployments.
    List(DeploymentListArgs),
    /// View detailed information for a deployment.
    View(DeploymentViewArgs),
}

/// Arguments for `gitlab deploy list`.
#[derive(Args, Debug)]
pub struct DeploymentListArgs {
    /// Filter deployments by status (`created`, `running`, `success`, `failed`, `canceled`).
    #[arg(short = 's', long)]
    pub status: Option<String>,

    /// Maximum number of deployments to list.
    #[arg(short = 'l', long, default_value = "30")]
    pub limit: usize,
}

/// Arguments for `gitlab deploy view`.
#[derive(Args, Debug)]
pub struct DeploymentViewArgs {
    /// Deployment numeric ID.
    pub id: u64,
}
