//! CLI definitions for Group Epic commands.

use clap::{Args, Subcommand};

/// Top-level arguments for `gitlab epic`.
#[derive(Args, Debug)]
pub struct EpicArgs {
    /// Epic subcommands.
    #[command(subcommand)]
    pub command: EpicCommands,
}

/// Available epic subcommands.
#[derive(Subcommand, Debug)]
pub enum EpicCommands {
    /// List epics in a group.
    List(EpicListArgs),
    /// View detailed information for an epic.
    View(EpicViewArgs),
    /// Create a new epic in a group.
    Create(EpicCreateArgs),
    /// Close an open epic.
    Close(EpicCloseArgs),
}

/// Arguments for `gitlab epic list`.
#[derive(Args, Debug)]
pub struct EpicListArgs {
    /// Target group path (e.g. `my-group`).
    #[arg(short = 'g', long)]
    pub group: String,

    /// Filter by state (`opened`, `closed`).
    #[arg(short = 's', long)]
    pub state: Option<String>,

    /// Maximum number of epics to list.
    #[arg(short = 'l', long, default_value = "30")]
    pub limit: usize,
}

/// Arguments for `gitlab epic view`.
#[derive(Args, Debug)]
pub struct EpicViewArgs {
    /// Target group path (e.g. `my-group`).
    #[arg(short = 'g', long)]
    pub group: String,

    /// Internal group-scoped epic IID (e.g. `1`).
    pub iid: u64,

    /// Open epic in the default web browser.
    #[arg(short = 'w', long)]
    pub web: bool,
}

/// Arguments for `gitlab epic create`.
#[derive(Args, Debug)]
pub struct EpicCreateArgs {
    /// Target group path (e.g. `my-group`).
    #[arg(short = 'g', long)]
    pub group: String,

    /// Epic title.
    pub title: String,

    /// Detailed description.
    #[arg(short = 'd', long)]
    pub description: Option<String>,
}

/// Arguments for `gitlab epic close`.
#[derive(Args, Debug)]
pub struct EpicCloseArgs {
    /// Target group path (e.g. `my-group`).
    #[arg(short = 'g', long)]
    pub group: String,

    /// Internal group-scoped epic IID to close.
    pub iid: u64,
}
