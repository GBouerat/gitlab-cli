//! Command handler for `gitlab registry`.

use crate::api::GitLabClient;
use crate::cli::registry::{
    RegistryArgs, RegistryCommands, RegistryDeleteRepoArgs, RegistryDeleteTagArgs,
    RegistryListArgs, RegistryTagsArgs,
};
use crate::ui::table::{colored_cell, create_table, print_json_or_table};
use colored::Colorize;
use comfy_table::{Cell, Color};

/// Dispatches container registry subcommands.
///
/// # Arguments
///
/// * `args` - Registry command arguments.
/// * `client` - Authenticated GitLab API client.
/// * `project` - Target project path (e.g. `owner/repo`).
/// * `json_mode` - Whether to output raw JSON.
///
/// # Errors
///
/// Returns an error if API communication fails.
pub async fn execute_registry(
    args: RegistryArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    match args.command {
        RegistryCommands::List(list_args) => {
            handle_list(list_args, client, project, json_mode).await
        }
        RegistryCommands::Tags(tags_args) => {
            handle_tags(tags_args, client, project, json_mode).await
        }
        RegistryCommands::DeleteTag(del_tag_args) => {
            handle_delete_tag(del_tag_args, client, project).await
        }
        RegistryCommands::DeleteRepo(del_repo_args) => {
            handle_delete_repo(del_repo_args, client, project).await
        }
    }
}

async fn handle_list(
    args: RegistryListArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let repos = client
        .list_registry_repositories(project, Some(args.limit))
        .await?;

    let mut table = create_table(&["ID", "Name", "Path", "Tags Count"]);
    for r in &repos {
        let tag_count = r
            .tag_count
            .map(|c| c.to_string())
            .unwrap_or_else(|| "-".to_string());

        table.add_row(vec![
            colored_cell(r.id.to_string(), Color::Cyan, false),
            Cell::new(&r.name),
            Cell::new(&r.path),
            Cell::new(tag_count),
        ]);
    }

    print_json_or_table(&repos, &table, json_mode)
}

async fn handle_tags(
    args: RegistryTagsArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let tags = client
        .list_registry_tags(project, args.repository_id, Some(args.limit))
        .await?;

    let mut table = create_table(&["Tag", "Location", "Short Digest", "Created At"]);
    for t in &tags {
        let digest = t.short_revision.as_deref().unwrap_or("-");

        table.add_row(vec![
            colored_cell(&t.name, Color::Cyan, true),
            Cell::new(&t.location),
            Cell::new(digest),
            Cell::new(t.created_at.as_deref().unwrap_or("-")),
        ]);
    }

    print_json_or_table(&tags, &table, json_mode)
}

async fn handle_delete_tag(
    args: RegistryDeleteTagArgs,
    client: &GitLabClient,
    project: &str,
) -> anyhow::Result<()> {
    client
        .delete_registry_tag(project, args.repository_id, &args.tag_name)
        .await?;
    println!(
        "{} Deleted tag {} from repository #{}",
        "✓".green().bold(),
        args.tag_name.bold(),
        args.repository_id
    );
    Ok(())
}

async fn handle_delete_repo(
    args: RegistryDeleteRepoArgs,
    client: &GitLabClient,
    project: &str,
) -> anyhow::Result<()> {
    client
        .delete_registry_repository(project, args.repository_id)
        .await?;
    println!(
        "{} Deleted container repository #{}",
        "✓".green().bold(),
        args.repository_id
    );
    Ok(())
}
