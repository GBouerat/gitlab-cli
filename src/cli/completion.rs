//! CLI definitions for generating shell completions.

use clap::{Args, ValueEnum};

/// Supported shell targets for completion script generation.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellType {
    /// Bourne-Again SHell completions.
    Bash,
    /// Z SHell completions.
    Zsh,
}

/// Arguments for the `completion` command.
#[derive(Args, Debug)]
pub struct CompletionArgs {
    /// Shell for which to generate completions (`bash` or `zsh`).
    #[arg(value_enum)]
    pub shell: ShellType,
}
