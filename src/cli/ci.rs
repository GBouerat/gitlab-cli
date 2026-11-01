//! CLI definitions for CI/CD Pipeline and Job commands.

use clap::{Args, Subcommand};

/// Top-level arguments for `gitlab ci` (or alias `gitlab pipeline`).
#[derive(Args, Debug)]
pub struct CiArgs {
    /// CI/CD subcommands.
    #[command(subcommand)]
    pub command: CiCommands,
}

/// Available CI/CD subcommands.
#[derive(Subcommand, Debug)]
pub enum CiCommands {
    /// List recent pipelines for the project.
    List(CiListArgs),
    /// View detailed stages and jobs for a pipeline.
    View(CiViewArgs),
    /// Print execution log trace of a specific job.
    Trace(CiTraceArgs),
    /// Trigger a new pipeline on a branch or tag.
    Run(CiRunArgs),
    /// Retry failed jobs in a pipeline.
    Retry(CiRetryArgs),
    /// Cancel a running or pending pipeline.
    Cancel(CiCancelArgs),
}

/// Arguments for `gitlab ci list`.
#[derive(Args, Debug)]
#[command(
    after_help = "Examples:\n  gitlab ci list\n  gitlab ci list --ref main\n  gitlab ci list --sha 11223344\n  gitlab ci list --status failed"
)]
pub struct CiListArgs {
    /// Target branch or tag ref.
    #[arg(short = 'r', long = "ref", conflicts_with = "sha")]
    pub ref_name: Option<String>,

    /// Target commit SHA (defaults to HEAD of current repository if --ref is omitted).
    #[arg(long, conflicts_with = "ref_name")]
    pub sha: Option<String>,

    /// Filter by execution status (`running`, `success`, `failed`, `pending`, `canceled`).
    #[arg(short = 's', long)]
    pub status: Option<String>,

    /// Maximum number of pipelines to list.
    #[arg(short = 'l', long, default_value = "20")]
    pub limit: usize,
}

/// Arguments for `gitlab ci view`.
#[derive(Args, Debug)]
pub struct CiViewArgs {
    /// Pipeline numeric ID (e.g. `987654`).
    pub id: u64,

    /// Open the pipeline in the default web browser.
    #[arg(short = 'w', long)]
    pub web: bool,
}

/// Arguments for `gitlab ci trace`.
#[derive(Args, Debug)]
#[command(
    after_help = "Examples:\n  gitlab ci trace 98765\n  gitlab ci trace 98765 --tail 50\n  gitlab ci trace 98765 --no-color"
)]
pub struct CiTraceArgs {
    /// Numeric Job ID whose logs to inspect.
    pub job_id: u64,

    /// Number of lines to display from the end of the log.
    #[arg(short = 'n', long)]
    pub tail: Option<usize>,

    /// Strip ANSI color codes and GitLab section markers.
    #[arg(long)]
    pub no_color: bool,
}

/// Arguments for `gitlab ci run`.
#[derive(Args, Debug)]
pub struct CiRunArgs {
    /// Target Git ref (branch or tag name, defaults to current branch).
    #[arg(short = 'r', long)]
    pub ref_name: Option<String>,
}

/// Arguments for `gitlab ci retry`.
#[derive(Args, Debug)]
pub struct CiRetryArgs {
    /// Pipeline ID to retry.
    pub id: u64,
}

/// Arguments for `gitlab ci cancel`.
#[derive(Args, Debug)]
pub struct CiCancelArgs {
    /// Pipeline ID to cancel.
    pub id: u64,
}
