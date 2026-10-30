//! CLI definitions for Snippet commands.

use clap::{Args, Subcommand};

/// Top-level arguments for `gitlab snippet`.
#[derive(Args, Debug)]
pub struct SnippetArgs {
    /// Snippet subcommands.
    #[command(subcommand)]
    pub command: SnippetCommands,
}

/// Available snippet subcommands.
#[derive(Subcommand, Debug)]
pub enum SnippetCommands {
    /// List personal and accessible snippets.
    List(SnippetListArgs),
    /// View detailed information or content of a snippet.
    View(SnippetViewArgs),
    /// Create a new snippet.
    Create(SnippetCreateArgs),
    /// Delete a snippet.
    Delete(SnippetDeleteArgs),
}

/// Arguments for `gitlab snippet list`.
#[derive(Args, Debug)]
pub struct SnippetListArgs {
    /// Maximum number of snippets to list.
    #[arg(short = 'l', long, default_value = "30")]
    pub limit: usize,
}

/// Arguments for `gitlab snippet view`.
#[derive(Args, Debug)]
pub struct SnippetViewArgs {
    /// Snippet numeric ID.
    pub id: u64,

    /// Print raw snippet code/content directly to stdout.
    #[arg(short = 'r', long)]
    pub raw: bool,

    /// Open snippet in web browser.
    #[arg(short = 'w', long)]
    pub web: bool,
}

/// Arguments for `gitlab snippet create`.
#[derive(Args, Debug)]
pub struct SnippetCreateArgs {
    /// Title of the snippet.
    #[arg(short = 't', long)]
    pub title: Option<String>,

    /// Filename of the snippet (e.g. `main.rs`, `script.sh`).
    #[arg(short = 'f', long)]
    pub file_name: Option<String>,

    /// Path to file containing content (or pass `-` to read from stdin).
    #[arg(short = 'c', long)]
    pub file: Option<String>,

    /// Raw snippet content string directly provided via argument.
    #[arg(long)]
    pub content: Option<String>,

    /// Visibility level (`public`, `internal`, or `private`, default: `private`).
    #[arg(long, default_value = "private")]
    pub visibility: String,

    /// Description of the snippet.
    #[arg(short = 'd', long)]
    pub description: Option<String>,
}

/// Arguments for `gitlab snippet delete`.
#[derive(Args, Debug)]
pub struct SnippetDeleteArgs {
    /// Snippet numeric ID to delete.
    pub id: u64,
}
