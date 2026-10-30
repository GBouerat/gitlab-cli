//! CLI definitions for Merge Request operations.

use clap::{Args, Subcommand};

/// Top-level arguments for `gitlab mr` (or alias `gitlab merge-request`).
#[derive(Args, Debug)]
pub struct MrArgs {
    /// Merge request subcommands.
    #[command(subcommand)]
    pub command: MrCommands,
}

/// Available merge request subcommands.
#[derive(Subcommand, Debug)]
pub enum MrCommands {
    /// List merge requests in the project.
    List(MrListArgs),
    /// View detailed information for a merge request.
    View(MrViewArgs),
    /// Create a new merge request.
    Create(MrCreateArgs),
    /// Check out a merge request source branch locally.
    Checkout(MrCheckoutArgs),
    /// Merge an open merge request.
    Merge(MrMergeArgs),
    /// Close an open merge request.
    Close(MrCloseArgs),
    /// Reopen a closed merge request.
    Reopen(MrReopenArgs),
    /// Approve an open merge request.
    Approve(MrApproveArgs),
    /// Revoke approval for a merge request.
    Unapprove(MrApproveArgs),
    /// View approval state and reviewers for a merge request.
    Approvals(MrApprovalsArgs),
    /// Add a comment/note to a merge request.
    Note(MrNoteArgs),
    /// List comments/notes on a merge request.
    Notes(MrNotesArgs),
}

/// Arguments for `gitlab mr list`.
#[derive(Args, Debug)]
pub struct MrListArgs {
    /// Filter by state (`opened`, `closed`, `merged`, `all`).
    #[arg(short = 's', long, default_value = "opened")]
    pub state: String,

    /// Filter by target branch name.
    #[arg(short = 't', long)]
    pub target_branch: Option<String>,

    /// Maximum number of merge requests to list.
    #[arg(short = 'l', long, default_value = "30")]
    pub limit: usize,
}

/// Arguments for `gitlab mr view`.
#[derive(Args, Debug)]
pub struct MrViewArgs {
    /// Project-scoped internal merge request IID (e.g. `12`).
    pub iid: u64,

    /// Open the merge request in the default web browser.
    #[arg(short = 'w', long)]
    pub web: bool,
}

/// Arguments for `gitlab mr create`.
#[derive(Args, Debug)]
pub struct MrCreateArgs {
    /// Title of the merge request.
    #[arg(short = 't', long)]
    pub title: Option<String>,

    /// Description / body text (Markdown).
    #[arg(short = 'd', long)]
    pub description: Option<String>,

    /// Source branch (defaults to currently checked out branch).
    #[arg(short = 's', long)]
    pub source_branch: Option<String>,

    /// Target destination branch (defaults to default branch or `main`).
    #[arg(short = 'b', long)]
    pub target_branch: Option<String>,

    /// Mark the merge request as Draft.
    #[arg(long)]
    pub draft: bool,

    /// Configure commits to squash upon merge.
    #[arg(long)]
    pub squash: bool,
}

/// Arguments for `gitlab mr checkout`.
#[derive(Args, Debug)]
pub struct MrCheckoutArgs {
    /// Internal project-scoped merge request IID to check out.
    pub iid: u64,
}

/// Arguments for `gitlab mr merge`.
#[derive(Args, Debug)]
pub struct MrMergeArgs {
    /// Internal project-scoped merge request IID to merge.
    pub iid: u64,

    /// Squash commits when merging.
    #[arg(long)]
    pub squash: bool,

    /// Delete source branch after merge.
    #[arg(long)]
    pub remove_source_branch: bool,
}

/// Arguments for `gitlab mr close`.
#[derive(Args, Debug)]
pub struct MrCloseArgs {
    /// Internal project-scoped merge request IID to close.
    pub iid: u64,
}

/// Arguments for `gitlab mr reopen`.
#[derive(Args, Debug)]
pub struct MrReopenArgs {
    /// Internal project-scoped merge request IID to reopen.
    pub iid: u64,
}

/// Arguments for `gitlab mr approve` and `gitlab mr unapprove`.
#[derive(Args, Debug)]
pub struct MrApproveArgs {
    /// Internal project-scoped merge request IID.
    pub iid: u64,
}

/// Arguments for `gitlab mr approvals`.
#[derive(Args, Debug)]
pub struct MrApprovalsArgs {
    /// Internal project-scoped merge request IID.
    pub iid: u64,
}

/// Arguments for `gitlab mr note`.
#[derive(Args, Debug)]
pub struct MrNoteArgs {
    /// Internal project-scoped merge request IID.
    pub iid: u64,

    /// Comment text body (Markdown). If omitted, an interactive editor/prompt will open.
    #[arg(short = 'b', long)]
    pub body: Option<String>,
}

/// Arguments for `gitlab mr notes`.
#[derive(Args, Debug)]
pub struct MrNotesArgs {
    /// Internal project-scoped merge request IID.
    pub iid: u64,

    /// Maximum number of notes to display.
    #[arg(short = 'l', long, default_value = "30")]
    pub limit: usize,
}
