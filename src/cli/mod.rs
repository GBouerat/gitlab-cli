//! Top-level command-line argument structures and command dispatching types.

pub mod api;
pub mod auth;
pub mod branch;
pub mod ci;
pub mod completion;
pub mod deployment;
pub mod environment;
pub mod epic;
pub mod group;
pub mod issue;
pub mod milestone;
pub mod mr;
pub mod package;
pub mod project;
pub mod registry;
pub mod release;
pub mod snippet;
pub mod tag;
pub mod variable;

use clap::{Parser, Subcommand};

/// Top-level command-line interface specification for `gitlab`.
#[derive(Parser, Debug)]
#[command(
    name = "gitlab",
    author = "Guillaume Bouérat <contact@bouerat.fr>",
    version,
    about = "A fast, modular, and ergonomic command-line interface for GitLab written in Rust.",
    long_version = concat!(
        env!("CARGO_PKG_VERSION"), "\n",
        "Repository: https://github.com/GBouerat/gitlab-cli"
    ),
    after_help = "Repository: https://github.com/GBouerat/gitlab-cli\nReport issues at: https://github.com/GBouerat/gitlab-cli/issues"
)]
pub struct Cli {
    /// Target GitLab project (e.g., `owner/repo`). Defaults to the current Git repository remote.
    #[arg(short = 'R', long = "repo", global = true)]
    pub repo: Option<String>,

    /// Target GitLab host (e.g., `gitlab.com` or `gitlab.example.com`). Defaults to current Git remote or configuration.
    #[arg(short = 'H', long = "host", global = true)]
    pub host: Option<String>,

    /// Output raw JSON instead of formatted tables.
    #[arg(long = "json", global = true)]
    pub json: bool,

    /// Increase logging verbosity (-v for debug, -vv for trace).
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Action subcommand to execute.
    #[command(subcommand)]
    pub command: Commands,
}

/// Available top-level subcommands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Authenticate with a GitLab instance.
    Auth(auth::AuthArgs),
    /// View and manage GitLab projects and repositories.
    #[command(alias = "repo")]
    Project(project::ProjectArgs),
    /// Manage Merge Requests.
    #[command(alias = "merge-request")]
    Mr(mr::MrArgs),
    /// Track and manage GitLab issues.
    Issue(issue::IssueArgs),
    /// Manage CI/CD pipelines, jobs, and streaming traces.
    #[command(alias = "pipeline")]
    Ci(ci::CiArgs),
    /// Manage project releases.
    Release(release::ReleaseArgs),
    /// Manage repository branches.
    Branch(branch::BranchArgs),
    /// Manage repository tags.
    Tag(tag::TagArgs),
    /// Manage project environments.
    #[command(alias = "environment")]
    Env(environment::EnvironmentArgs),
    /// View project deployments.
    #[command(alias = "deployment")]
    Deploy(deployment::DeploymentArgs),
    /// Manage project and group milestones.
    Milestone(milestone::MilestoneArgs),
    /// Manage group epics.
    Epic(epic::EpicArgs),
    /// Manage project packages.
    Package(package::PackageArgs),
    /// Manage container registry images and repositories.
    Registry(registry::RegistryArgs),
    /// View and manage GitLab groups and members.
    Group(group::GroupArgs),
    /// Manage project CI/CD variables.
    Variable(variable::VariableArgs),
    /// Manage code snippets.
    Snippet(snippet::SnippetArgs),
    /// Execute direct authenticated REST API requests against GitLab.
    Api(api::ApiArgs),
    /// Generate shell completion scripts for Bash or Zsh.
    Completion(completion::CompletionArgs),
}
