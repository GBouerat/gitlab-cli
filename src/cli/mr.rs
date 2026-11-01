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
    /// View detailed information for a merge request (or find by branch).
    #[command(alias = "find")]
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
    /// List changed files or print the unified diff patch.
    Diff(MrDiffArgs),
    /// List discussions and inline review threads on a merge request.
    Discussions(MrDiscussionsArgs),
    /// Start a new discussion thread or inline code review comment.
    Comment(MrCommentArgs),
    /// Reply to an existing discussion thread.
    Reply(MrReplyArgs),
    /// Resolve or unresolve a discussion thread.
    Resolve(MrResolveArgs),
    /// Edit an existing note or comment in place.
    #[command(name = "note-edit")]
    NoteEdit(MrNoteEditArgs),
    /// Update metadata, labels, reviewers, or description of a merge request.
    Update(MrUpdateArgs),
    /// Relink rebased commit SHAs in note or description to current commits.
    Relink(MrRelinkArgs),
}

/// Arguments for `gitlab mr list`.
#[derive(Args, Debug)]
#[command(
    after_help = "Examples:\n  gitlab mr list\n  gitlab mr list --state merged\n  gitlab mr list --source-branch feature-x"
)]
pub struct MrListArgs {
    /// Filter by state (`opened`, `closed`, `merged`, `all`).
    #[arg(short = 's', long, default_value = "opened")]
    pub state: String,

    /// Filter by target branch name.
    #[arg(short = 't', long)]
    pub target_branch: Option<String>,

    /// Filter by source branch name.
    #[arg(long)]
    pub source_branch: Option<String>,

    /// Maximum number of merge requests to list.
    #[arg(short = 'l', long, default_value = "30")]
    pub limit: usize,
}

/// Arguments for `gitlab mr view` and alias `gitlab mr find`.
#[derive(Args, Debug)]
#[command(
    after_help = "Examples:\n  gitlab mr view\n  gitlab mr view 12\n  gitlab mr view feature-branch\n  gitlab mr find 12 --web"
)]
pub struct MrViewArgs {
    /// Internal merge request IID (e.g. `12`) or source branch name (defaults to current branch).
    pub selector: Option<String>,

    /// Open the merge request in the default web browser.
    #[arg(short = 'w', long)]
    pub web: bool,
}

/// Arguments for `gitlab mr create`.
#[derive(Args, Debug)]
#[command(
    after_help = "Examples:\n  gitlab mr create -t \"Feature\" -f body.md --reviewer alice --assignee @me\n  gitlab mr create -t \"Bugfix\" --label bug --remove-source-branch"
)]
pub struct MrCreateArgs {
    /// Title of the merge request.
    #[arg(short = 't', long)]
    pub title: Option<String>,

    /// Description / body text (Markdown).
    #[arg(short = 'd', long)]
    pub description: Option<String>,

    /// Path to file containing description (or `-` for stdin).
    #[arg(short = 'f', long, conflicts_with = "description")]
    pub file: Option<String>,

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

    /// Assignee usernames (or `@me` for the authenticated user).
    #[arg(long = "assignee", value_name = "USERNAME")]
    pub assignees: Vec<String>,

    /// Reviewer usernames (or `@me` for the authenticated user).
    #[arg(long = "reviewer", value_name = "USERNAME")]
    pub reviewers: Vec<String>,

    /// Label names to apply to the merge request.
    #[arg(long = "label", value_name = "LABEL")]
    pub labels: Vec<String>,

    /// Delete source branch after merge.
    #[arg(long)]
    pub remove_source_branch: bool,
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
#[command(
    after_help = "Examples:\n  gitlab mr note 12 -b \"Looks good!\"\n  gitlab mr note 12 -f notes.md\n  cat notes.md | gitlab mr note 12 -f -"
)]
pub struct MrNoteArgs {
    /// Internal project-scoped merge request IID.
    pub iid: u64,

    /// Comment text body (Markdown). If omitted, an interactive editor/prompt will open.
    #[arg(short = 'b', long)]
    pub body: Option<String>,

    /// Path to file containing note body (or `-` for stdin).
    #[arg(short = 'f', long, conflicts_with = "body")]
    pub file: Option<String>,
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

/// Arguments for `gitlab mr diff`.
#[derive(Args, Debug)]
#[command(after_help = "Examples:\n  gitlab mr diff 12\n  gitlab mr diff 12 --patch")]
pub struct MrDiffArgs {
    /// Internal project-scoped merge request IID.
    pub iid: u64,

    /// Print the unified patch diff for changed files.
    #[arg(short = 'p', long)]
    pub patch: bool,
}

/// Arguments for `gitlab mr discussions`.
#[derive(Args, Debug)]
#[command(
    after_help = "Examples:\n  gitlab mr discussions 12\n  gitlab mr discussions 12 --unresolved\n  gitlab mr discussions 12 --full"
)]
pub struct MrDiscussionsArgs {
    /// Internal project-scoped merge request IID.
    pub iid: u64,

    /// Only display unresolved discussion threads.
    #[arg(short = 'u', long)]
    pub unresolved: bool,

    /// Print every note of each thread in full with note IDs.
    #[arg(long)]
    pub full: bool,
}

/// Arguments for `gitlab mr comment`.
#[derive(Args, Debug)]
#[command(
    after_help = "Examples:\n  gitlab mr comment 12 -b \"LGTM\"\n  gitlab mr comment 12 -f review.md\n  gitlab mr comment 12 -f note.md --path src/main.rs --line 42\n  cat review.md | gitlab mr comment 12 -f - --prefix \"Automated Check:\""
)]
pub struct MrCommentArgs {
    /// Internal project-scoped merge request IID.
    pub iid: u64,

    /// Comment text body (Markdown).
    #[arg(short = 'b', long)]
    pub body: Option<String>,

    /// Path to file containing comment body (or `-` for stdin).
    #[arg(short = 'f', long, conflicts_with = "body")]
    pub file: Option<String>,

    /// Target file path for an inline diff comment.
    #[arg(long)]
    pub path: Option<String>,

    /// New line number for an inline diff comment.
    #[arg(long)]
    pub line: Option<u64>,

    /// Old line number for context or removed lines.
    #[arg(long)]
    pub old_line: Option<u64>,

    /// Line prefix to prepend to the comment body.
    #[arg(long)]
    pub prefix: Option<String>,
}

/// Arguments for `gitlab mr reply`.
#[derive(Args, Debug)]
#[command(
    after_help = "Examples:\n  gitlab mr reply 12 disc-id-123 -b \"Fixed in latest push\"\n  gitlab mr reply 12 disc-id-123 -f reply.md"
)]
pub struct MrReplyArgs {
    /// Internal project-scoped merge request IID.
    pub iid: u64,

    /// Discussion thread ID to reply to.
    pub discussion_id: String,

    /// Reply text body (Markdown).
    #[arg(short = 'b', long)]
    pub body: Option<String>,

    /// Path to file containing reply body (or `-` for stdin).
    #[arg(short = 'f', long, conflicts_with = "body")]
    pub file: Option<String>,
}

/// Arguments for `gitlab mr resolve`.
#[derive(Args, Debug)]
#[command(
    after_help = "Examples:\n  gitlab mr resolve 12 disc-id-123\n  gitlab mr resolve 12 disc-id-123 --undo"
)]
pub struct MrResolveArgs {
    /// Internal project-scoped merge request IID.
    pub iid: u64,

    /// Discussion thread ID to resolve or unresolve.
    pub discussion_id: String,

    /// Unresolve the thread instead of resolving it.
    #[arg(long)]
    pub undo: bool,
}

/// Arguments for `gitlab mr note-edit`.
#[derive(Args, Debug)]
#[command(
    after_help = "Examples:\n  gitlab mr note-edit 12 456 -b \"Updated comment body\"\n  gitlab mr note-edit 12 456 -f updated.md --prefix \"[EDITED]\""
)]
pub struct MrNoteEditArgs {
    /// Internal project-scoped merge request IID.
    pub iid: u64,

    /// Numeric ID of the note to edit.
    pub note_id: u64,

    /// Updated comment text body (Markdown).
    #[arg(short = 'b', long)]
    pub body: Option<String>,

    /// Path to file containing updated body (or `-` for stdin).
    #[arg(short = 'f', long, conflicts_with = "body")]
    pub file: Option<String>,

    /// Line prefix to prepend to the note body.
    #[arg(long)]
    pub prefix: Option<String>,
}

/// Arguments for `gitlab mr update`.
#[derive(Args, Debug)]
#[command(
    after_help = "Examples:\n  gitlab mr update 12 --title \"New title\" --ready\n  gitlab mr update 12 -f description.md --add-label bug --reviewer alice\n  gitlab mr update 12 --draft --remove-label ready"
)]
pub struct MrUpdateArgs {
    /// Internal project-scoped merge request IID.
    pub iid: u64,

    /// Updated title for the merge request.
    #[arg(short = 't', long)]
    pub title: Option<String>,

    /// Updated description text body (Markdown).
    #[arg(short = 'd', long, visible_short_alias = 'b', visible_alias = "body")]
    pub description: Option<String>,

    /// Path to file containing updated description (or `-` for stdin).
    #[arg(short = 'f', long, conflicts_with = "description")]
    pub file: Option<String>,

    /// Labels to add to the merge request.
    #[arg(long = "add-label", value_name = "LABEL")]
    pub add_labels: Vec<String>,

    /// Labels to remove from the merge request.
    #[arg(long = "remove-label", value_name = "LABEL")]
    pub remove_labels: Vec<String>,

    /// Reviewer usernames (or `@me`).
    #[arg(long = "reviewer", value_name = "USERNAME")]
    pub reviewers: Vec<String>,

    /// Assignee usernames (or `@me`).
    #[arg(long = "assignee", value_name = "USERNAME")]
    pub assignees: Vec<String>,

    /// Mark the merge request as ready (removes Draft prefix).
    #[arg(long, conflicts_with = "draft")]
    pub ready: bool,

    /// Mark the merge request as Draft.
    #[arg(long, conflicts_with = "ready")]
    pub draft: bool,
}

/// Arguments for `gitlab mr relink`.
#[derive(Args, Debug)]
#[command(
    after_help = "Examples:\n  gitlab mr relink 12 --dry-run\n  gitlab mr relink 12\n  gitlab mr relink 12 456"
)]
pub struct MrRelinkArgs {
    /// Internal project-scoped merge request IID.
    pub iid: u64,

    /// Optional Note ID to rewrite (if omitted, rewrites the MR description).
    pub note_id: Option<u64>,

    /// Print a unified diff of rewrites without applying changes to GitLab.
    #[arg(long)]
    pub dry_run: bool,
}
