//! Command handler for generating shell completions.

use crate::cli::completion::{CompletionArgs, ShellType};
use crate::cli::Cli;
use clap::CommandFactory;
use clap_complete::Shell;
use std::io;

/// Generates shell completion script for Bash or Zsh to stdout.
///
/// # Arguments
///
/// * `args` - Completion arguments indicating target shell.
pub fn execute_completion(args: CompletionArgs) {
    let mut cmd = Cli::command();
    let shell = match args.shell {
        ShellType::Bash => Shell::Bash,
        ShellType::Zsh => Shell::Zsh,
    };

    clap_complete::generate(shell, &mut cmd, "gitlab", &mut io::stdout());
}
