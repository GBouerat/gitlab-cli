//! CLI definitions for Project Package commands.

use clap::{Args, Subcommand};

/// Top-level arguments for `gitlab package`.
#[derive(Args, Debug)]
pub struct PackageArgs {
    /// Package subcommands.
    #[command(subcommand)]
    pub command: PackageCommands,
}

/// Available package subcommands.
#[derive(Subcommand, Debug)]
pub enum PackageCommands {
    /// List packages published to the project.
    List(PackageListArgs),
    /// View detailed information for a package.
    View(PackageViewArgs),
    /// Delete a package.
    Delete(PackageDeleteArgs),
}

/// Arguments for `gitlab package list`.
#[derive(Args, Debug)]
pub struct PackageListArgs {
    /// Filter by package format/type (e.g. `npm`, `maven`, `pypi`, `generic`).
    #[arg(short = 't', long)]
    pub package_type: Option<String>,

    /// Maximum number of packages to list.
    #[arg(short = 'l', long, default_value = "30")]
    pub limit: usize,
}

/// Arguments for `gitlab package view`.
#[derive(Args, Debug)]
pub struct PackageViewArgs {
    /// Package numeric ID.
    pub id: u64,
}

/// Arguments for `gitlab package delete`.
#[derive(Args, Debug)]
pub struct PackageDeleteArgs {
    /// Package numeric ID to delete.
    pub id: u64,
}
