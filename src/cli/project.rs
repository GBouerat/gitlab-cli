//! CLI definitions for project and repository commands.

use clap::{Args, Subcommand};

/// Top-level arguments for `gitlab project` (or alias `gitlab repo`).
#[derive(Args, Debug)]
pub struct ProjectArgs {
    /// Project subcommands.
    #[command(subcommand)]
    pub command: ProjectCommands,
}

/// Available project subcommands.
#[derive(Subcommand, Debug)]
pub enum ProjectCommands {
    /// View detailed information about a project.
    View(ProjectViewArgs),
    /// List projects accessible to the authenticated user.
    List(ProjectListArgs),
}

/// Arguments for `gitlab project view`.
#[derive(Args, Debug)]
pub struct ProjectViewArgs {
    /// Target project ID or namespaced path (defaults to current Git repository).
    pub project: Option<String>,

    /// Open the project in the default web browser.
    #[arg(short = 'w', long)]
    pub web: bool,
}

/// Arguments for `gitlab project list`.
#[derive(Args, Debug)]
pub struct ProjectListArgs {
    /// Limit to projects where you are an explicit member.
    #[arg(long, default_value = "true")]
    pub membership: bool,

    /// Maximum number of projects to display.
    #[arg(short = 'l', long, default_value = "30")]
    pub limit: usize,
}
