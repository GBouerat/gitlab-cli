//! CLI definitions for Environment commands.

use clap::{Args, Subcommand};

/// Top-level arguments for `gitlab env` (or `gitlab environment`).
#[derive(Args, Debug)]
pub struct EnvironmentArgs {
    /// Environment subcommands.
    #[command(subcommand)]
    pub command: EnvironmentCommands,
}

/// Available environment subcommands.
#[derive(Subcommand, Debug)]
pub enum EnvironmentCommands {
    /// List project environments.
    List(EnvironmentListArgs),
    /// View detailed information for an environment.
    View(EnvironmentViewArgs),
    /// Create a new environment.
    Create(EnvironmentCreateArgs),
    /// Stop an active environment.
    Stop(EnvironmentStopArgs),
    /// Delete an environment.
    Delete(EnvironmentDeleteArgs),
}

/// Arguments for `gitlab env list`.
#[derive(Args, Debug)]
pub struct EnvironmentListArgs {
    /// Maximum number of environments to list.
    #[arg(short = 'l', long, default_value = "50")]
    pub limit: usize,
}

/// Arguments for `gitlab env view`.
#[derive(Args, Debug)]
pub struct EnvironmentViewArgs {
    /// Environment numeric ID.
    pub id: u64,

    /// Open external URL in the default web browser.
    #[arg(short = 'w', long)]
    pub web: bool,
}

/// Arguments for `gitlab env create`.
#[derive(Args, Debug)]
pub struct EnvironmentCreateArgs {
    /// Environment name (e.g. `production`, `staging`).
    pub name: String,

    /// External deployment URL.
    #[arg(short = 'u', long)]
    pub external_url: Option<String>,
}

/// Arguments for `gitlab env stop`.
#[derive(Args, Debug)]
pub struct EnvironmentStopArgs {
    /// Environment numeric ID to stop.
    pub id: u64,
}

/// Arguments for `gitlab env delete`.
#[derive(Args, Debug)]
pub struct EnvironmentDeleteArgs {
    /// Environment numeric ID to delete.
    pub id: u64,
}
