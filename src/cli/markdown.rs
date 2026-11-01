//! CLI definitions for GitLab Markdown rendering commands.

use clap::{Args, Subcommand};

/// Top-level arguments for `gitlab markdown`.
#[derive(Args, Debug)]
pub struct MarkdownArgs {
    /// Markdown subcommands.
    #[command(subcommand)]
    pub command: MarkdownCommands,
}

/// Available Markdown subcommands.
#[derive(Subcommand, Debug)]
pub enum MarkdownCommands {
    /// Render Markdown via GitLab Flavored Markdown API.
    Render(MarkdownRenderArgs),
}

/// Arguments for `gitlab markdown render`.
#[derive(Args, Debug)]
#[command(
    after_help = "Examples:\n  gitlab markdown render -b \"# Hello World\"\n  gitlab markdown render -f README.md --project\n  cat doc.md | gitlab markdown render -f -"
)]
pub struct MarkdownRenderArgs {
    /// Inline Markdown text to render.
    #[arg(short = 'b', long)]
    pub body: Option<String>,

    /// Path to file containing Markdown (or `-` for stdin).
    #[arg(short = 'f', long, conflicts_with = "body")]
    pub file: Option<String>,

    /// Render within current project context for scoped references.
    #[arg(long)]
    pub project: bool,
}
