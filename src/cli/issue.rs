//! CLI definitions for Issue tracking commands.

use clap::{Args, Subcommand};

/// Top-level arguments for `gitlab issue`.
#[derive(Args, Debug)]
pub struct IssueArgs {
    /// Issue subcommands.
    #[command(subcommand)]
    pub command: IssueCommands,
}

/// Available issue subcommands.
#[derive(Subcommand, Debug)]
pub enum IssueCommands {
    /// List issues in the project.
    List(IssueListArgs),
    /// View detailed information for an issue.
    View(IssueViewArgs),
    /// Create a new issue.
    Create(IssueCreateArgs),
    /// Close an open issue.
    Close(IssueCloseArgs),
    /// Reopen a closed issue.
    Reopen(IssueReopenArgs),
    /// Add a comment/note to an issue.
    Note(IssueNoteArgs),
    /// List comments/notes on an issue.
    Notes(IssueNotesArgs),
}

/// Arguments for `gitlab issue list`.
#[derive(Args, Debug)]
pub struct IssueListArgs {
    /// Filter by state (`opened`, `closed`, `all`).
    #[arg(short = 's', long, default_value = "opened")]
    pub state: String,

    /// Filter by comma-separated label names.
    #[arg(short = 'l', long)]
    pub labels: Option<String>,

    /// Maximum number of issues to list.
    #[arg(short = 'n', long, default_value = "30")]
    pub limit: usize,
}

/// Arguments for `gitlab issue view`.
#[derive(Args, Debug)]
pub struct IssueViewArgs {
    /// Internal project-scoped issue IID (e.g. `42`).
    pub iid: u64,

    /// Open the issue in the default web browser.
    #[arg(short = 'w', long)]
    pub web: bool,
}

/// Arguments for `gitlab issue create`.
#[derive(Args, Debug)]
#[command(
    after_help = "Examples:\n  gitlab issue create -t \"Bug title\" -d \"Details\"\n  gitlab issue create -t \"Bug title\" -f details.md\n  cat bug.md | gitlab issue create -t \"Bug\" -f -"
)]
pub struct IssueCreateArgs {
    /// Issue title.
    #[arg(short = 't', long)]
    pub title: Option<String>,

    /// Issue description (Markdown).
    #[arg(short = 'd', long)]
    pub description: Option<String>,

    /// Path to file containing description (or `-` for stdin).
    #[arg(short = 'f', long, conflicts_with = "description")]
    pub file: Option<String>,

    /// Comma-separated labels to attach.
    #[arg(short = 'l', long)]
    pub labels: Option<String>,
}

/// Arguments for `gitlab issue close`.
#[derive(Args, Debug)]
pub struct IssueCloseArgs {
    /// Internal project-scoped issue IID to close.
    pub iid: u64,
}

/// Arguments for `gitlab issue reopen`.
#[derive(Args, Debug)]
pub struct IssueReopenArgs {
    /// Internal project-scoped issue IID to reopen.
    pub iid: u64,
}

/// Arguments for `gitlab issue note`.
#[derive(Args, Debug)]
#[command(
    after_help = "Examples:\n  gitlab issue note 42 -b \"Reproduced on macOS\"\n  gitlab issue note 42 -f comment.md\n  cat comment.md | gitlab issue note 42 -f -"
)]
pub struct IssueNoteArgs {
    /// Internal project-scoped issue IID.
    pub iid: u64,

    /// Comment text body (Markdown). If omitted, an interactive editor/prompt will open.
    #[arg(short = 'b', long)]
    pub body: Option<String>,

    /// Path to file containing note body (or `-` for stdin).
    #[arg(short = 'f', long, conflicts_with = "body")]
    pub file: Option<String>,
}

/// Arguments for `gitlab issue notes`.
#[derive(Args, Debug)]
pub struct IssueNotesArgs {
    /// Internal project-scoped issue IID.
    pub iid: u64,

    /// Maximum number of notes to display.
    #[arg(short = 'n', long, default_value = "30")]
    pub limit: usize,
}
