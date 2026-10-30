//! Command dispatcher and resolution orchestration.

pub mod api;
pub mod auth;
pub mod branch;
pub mod ci;
pub mod completion;
pub mod group;
pub mod issue;
pub mod mr;
pub mod project;
pub mod release;
pub mod snippet;
pub mod variable;

use crate::api::GitLabClient;
use crate::cli::{Cli, Commands};
use crate::config::Config;
use crate::git::resolve_git_context;
use std::env;

/// Dispatches the parsed CLI commands to their respective handlers.
///
/// # Arguments
///
/// * `cli` - Parsed command-line arguments.
/// * `config` - Loaded user configuration.
///
/// # Errors
///
/// Returns an error if context resolution, authentication, or command execution fails.
pub async fn run_command(cli: Cli, mut config: Config) -> anyhow::Result<()> {
    match cli.command {
        Commands::Auth(args) => auth::execute_auth(args, &mut config).await,
        Commands::Completion(args) => {
            completion::execute_completion(args);
            Ok(())
        }
        other => {
            let current_dir = env::current_dir().unwrap_or_default();
            let git_ctx = resolve_git_context(&current_dir).ok();

            let (host, token) = config.resolve_auth(
                cli.host.as_deref(),
                git_ctx.as_ref().map(|g| g.host.as_str()),
            )?;

            let client = GitLabClient::new(&host, &token)?;

            let default_project = cli
                .repo
                .as_deref()
                .or_else(|| git_ctx.as_ref().map(|g| g.project_path.as_str()));

            match other {
                Commands::Project(args) => {
                    project::execute_project(args, &client, default_project, cli.json).await
                }
                Commands::Mr(args) => {
                    let proj = default_project.ok_or_else(|| {
                        anyhow::anyhow!(
                            "No GitLab project specified. Run inside a GitLab repository or provide `--repo <owner/repo>`."
                        )
                    })?;
                    mr::execute_mr(args, &client, proj, cli.json).await
                }
                Commands::Issue(args) => {
                    let proj = default_project.ok_or_else(|| {
                        anyhow::anyhow!(
                            "No GitLab project specified. Run inside a GitLab repository or provide `--repo <owner/repo>`."
                        )
                    })?;
                    issue::execute_issue(args, &client, proj, cli.json).await
                }
                Commands::Ci(args) => {
                    let proj = default_project.ok_or_else(|| {
                        anyhow::anyhow!(
                            "No GitLab project specified. Run inside a GitLab repository or provide `--repo <owner/repo>`."
                        )
                    })?;
                    ci::execute_ci(args, &client, proj, cli.json).await
                }
                Commands::Release(args) => {
                    let proj = default_project.ok_or_else(|| {
                        anyhow::anyhow!(
                            "No GitLab project specified. Run inside a GitLab repository or provide `--repo <owner/repo>`."
                        )
                    })?;
                    release::execute_release(args, &client, proj, cli.json).await
                }
                Commands::Branch(args) => {
                    let proj = default_project.ok_or_else(|| {
                        anyhow::anyhow!(
                            "No GitLab project specified. Run inside a GitLab repository or provide `--repo <owner/repo>`."
                        )
                    })?;
                    branch::execute_branch(args, &client, proj, cli.json).await
                }
                Commands::Variable(args) => {
                    let proj = default_project.ok_or_else(|| {
                        anyhow::anyhow!(
                            "No GitLab project specified. Run inside a GitLab repository or provide `--repo <owner/repo>`."
                        )
                    })?;
                    variable::execute_variable(args, &client, proj, cli.json).await
                }
                Commands::Group(args) => group::execute_group(args, &client, cli.json).await,
                Commands::Snippet(args) => snippet::execute_snippet(args, &client, cli.json).await,
                Commands::Api(args) => api::execute_api(args, &client, default_project).await,
                Commands::Auth(_) | Commands::Completion(_) => unreachable!(),
            }
        }
    }
}
