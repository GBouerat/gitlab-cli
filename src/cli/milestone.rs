//! CLI definitions for Milestone commands.

use clap::{Args, Subcommand};

/// Top-level arguments for `gitlab milestone`.
#[derive(Args, Debug)]
pub struct MilestoneArgs {
    /// Milestone subcommands.
    #[command(subcommand)]
    pub command: MilestoneCommands,
}

/// Available milestone subcommands.
#[derive(Subcommand, Debug)]
pub enum MilestoneCommands {
    /// List project or group milestones.
    List(MilestoneListArgs),
    /// View detailed information for a milestone.
    View(MilestoneViewArgs),
    /// Create a new milestone.
    Create(MilestoneCreateArgs),
    /// Close an active milestone.
    Close(MilestoneCloseArgs),
    /// Delete a milestone.
    Delete(MilestoneDeleteArgs),
}

/// Arguments for `gitlab milestone list`.
#[derive(Args, Debug)]
pub struct MilestoneListArgs {
    /// Filter by state (`active`, `closed`).
    #[arg(short = 's', long)]
    pub state: Option<String>,

    /// Target a group instead of project (e.g. `my-group`).
    #[arg(short = 'g', long)]
    pub group: Option<String>,

    /// Maximum number of milestones to list.
    #[arg(short = 'l', long, default_value = "30")]
    pub limit: usize,
}

/// Arguments for `gitlab milestone view`.
#[derive(Args, Debug)]
pub struct MilestoneViewArgs {
    /// Milestone numeric ID.
    pub id: u64,

    /// Open milestone in the default web browser.
    #[arg(short = 'w', long)]
    pub web: bool,
}

/// Arguments for `gitlab milestone create`.
#[derive(Args, Debug)]
pub struct MilestoneCreateArgs {
    /// Milestone title.
    pub title: String,

    /// Detailed description.
    #[arg(short = 'd', long)]
    pub description: Option<String>,

    /// Target due date in `YYYY-MM-DD` format.
    #[arg(long)]
    pub due_date: Option<String>,

    /// Target start date in `YYYY-MM-DD` format.
    #[arg(long)]
    pub start_date: Option<String>,
}

/// Arguments for `gitlab milestone close`.
#[derive(Args, Debug)]
pub struct MilestoneCloseArgs {
    /// Milestone numeric ID to close.
    pub id: u64,
}

/// Arguments for `gitlab milestone delete`.
#[derive(Args, Debug)]
pub struct MilestoneDeleteArgs {
    /// Milestone numeric ID to delete.
    pub id: u64,
}
