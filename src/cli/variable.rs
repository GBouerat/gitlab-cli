//! CLI definitions for CI/CD Variable commands.

use clap::{Args, Subcommand};

/// Top-level arguments for `gitlab variable`.
#[derive(Args, Debug)]
pub struct VariableArgs {
    /// Variable subcommands.
    #[command(subcommand)]
    pub command: VariableCommands,
}

/// Available variable subcommands.
#[derive(Subcommand, Debug)]
pub enum VariableCommands {
    /// List CI/CD variables configured for the project.
    List(VariableListArgs),
    /// Get details of a specific variable.
    Get(VariableGetArgs),
    /// Create or update a CI/CD variable.
    Set(VariableSetArgs),
    /// Delete a CI/CD variable.
    Delete(VariableDeleteArgs),
}

/// Arguments for `gitlab variable list`.
#[derive(Args, Debug)]
pub struct VariableListArgs {
    /// Maximum number of variables to list.
    #[arg(short = 'l', long, default_value = "50")]
    pub limit: usize,
}

/// Arguments for `gitlab variable get`.
#[derive(Args, Debug)]
pub struct VariableGetArgs {
    /// Variable key name.
    pub key: String,
}

/// Arguments for `gitlab variable set`.
#[derive(Args, Debug)]
pub struct VariableSetArgs {
    /// Variable key name.
    pub key: String,

    /// Variable value.
    #[arg(short = 'V', long)]
    pub value: Option<String>,

    /// Mask the variable in job logs.
    #[arg(short = 'm', long)]
    pub masked: bool,

    /// Restrict variable to protected branches and tags.
    #[arg(short = 'p', long)]
    pub protected: bool,

    /// Environment scope filter (defaults to `*`).
    #[arg(short = 'e', long, default_value = "*")]
    pub environment_scope: String,
}

/// Arguments for `gitlab variable delete`.
#[derive(Args, Debug)]
pub struct VariableDeleteArgs {
    /// Variable key name to delete.
    pub key: String,
}
