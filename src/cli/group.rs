//! CLI definitions for Group and Subgroup commands.

use clap::{Args, Subcommand};

/// Top-level arguments for `gitlab group`.
#[derive(Args, Debug)]
pub struct GroupArgs {
    /// Group subcommands.
    #[command(subcommand)]
    pub command: GroupCommands,
}

/// Available group subcommands.
#[derive(Subcommand, Debug)]
pub enum GroupCommands {
    /// List groups accessible to the user.
    List(GroupListArgs),
    /// View detailed information for a group.
    View(GroupViewArgs),
    /// List members and roles for a group.
    Members(GroupMembersArgs),
}

/// Arguments for `gitlab group list`.
#[derive(Args, Debug)]
pub struct GroupListArgs {
    /// Filter groups by search query.
    #[arg(short = 's', long)]
    pub search: Option<String>,

    /// Maximum number of groups to list.
    #[arg(short = 'l', long, default_value = "30")]
    pub limit: usize,
}

/// Arguments for `gitlab group view`.
#[derive(Args, Debug)]
pub struct GroupViewArgs {
    /// Group numeric ID or full path (e.g. `my-org` or `parent/subgroup`).
    pub group: String,

    /// Open group in web browser.
    #[arg(short = 'w', long)]
    pub web: bool,
}

/// Arguments for `gitlab group members`.
#[derive(Args, Debug)]
pub struct GroupMembersArgs {
    /// Group numeric ID or full path.
    pub group: String,

    /// Maximum number of members to list.
    #[arg(short = 'l', long, default_value = "50")]
    pub limit: usize,
}
