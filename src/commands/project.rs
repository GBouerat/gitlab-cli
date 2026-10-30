//! Command handler for `gitlab project` / `gitlab repo`.

use crate::api::GitLabClient;
use crate::cli::project::{ProjectArgs, ProjectCommands, ProjectListArgs, ProjectViewArgs};
use crate::ui::table::{colored_cell, create_table, print_json_or_table};
use comfy_table::{Cell, Color};

/// Dispatches project subcommands.
///
/// # Arguments
///
/// * `args` - Project command arguments.
/// * `client` - Authenticated GitLab API client.
/// * `default_project` - Default project path detected from the current Git context.
/// * `json_mode` - Whether to output raw JSON.
///
/// # Errors
///
/// Returns an error if API communication or display formatting fails.
pub async fn execute_project(
    args: ProjectArgs,
    client: &GitLabClient,
    default_project: Option<&str>,
    json_mode: bool,
) -> anyhow::Result<()> {
    match args.command {
        ProjectCommands::View(view_args) => {
            handle_view(view_args, client, default_project, json_mode).await
        }
        ProjectCommands::List(list_args) => handle_list(list_args, client, json_mode).await,
    }
}

async fn handle_view(
    args: ProjectViewArgs,
    client: &GitLabClient,
    default_project: Option<&str>,
    json_mode: bool,
) -> anyhow::Result<()> {
    let target = args.project.as_deref().or(default_project).ok_or_else(|| {
        anyhow::anyhow!(
            "No project specified. Provide a project path or run inside a GitLab repository."
        )
    })?;

    let project = client.get_project(target).await?;

    if args.web {
        if let Some(url) = &project.web_url {
            println!("Opening project in browser: {url}");
            crate::ui::open_in_browser(url);
            return Ok(());
        }
    }

    let mut table = create_table(&["Property", "Value"]);
    table.add_row(vec![
        Cell::new("Name"),
        Cell::new(
            project
                .name_with_namespace
                .as_deref()
                .unwrap_or(&project.name),
        ),
    ]);
    table.add_row(vec![
        Cell::new("Path"),
        Cell::new(&project.path_with_namespace),
    ]);
    table.add_row(vec![
        Cell::new("ID"),
        colored_cell(project.id.to_string(), Color::Cyan, false),
    ]);
    if let Some(desc) = &project.description {
        table.add_row(vec![Cell::new("Description"), Cell::new(desc)]);
    }
    if let Some(vis) = &project.visibility {
        table.add_row(vec![
            Cell::new("Visibility"),
            colored_cell(vis, Color::Yellow, true),
        ]);
    }
    if let Some(branch) = &project.default_branch {
        table.add_row(vec![Cell::new("Default Branch"), Cell::new(branch)]);
    }
    if let Some(ssh) = &project.ssh_url_to_repo {
        table.add_row(vec![Cell::new("SSH Clone URL"), Cell::new(ssh)]);
    }
    if let Some(http) = &project.http_url_to_repo {
        table.add_row(vec![Cell::new("HTTP Clone URL"), Cell::new(http)]);
    }
    if let Some(web) = &project.web_url {
        table.add_row(vec![Cell::new("Web URL"), Cell::new(web)]);
    }
    table.add_row(vec![
        Cell::new("Stats"),
        Cell::new(format!(
            "★ {} | ⑂ {} | ◉ {} open issues",
            project.star_count.unwrap_or(0),
            project.forks_count.unwrap_or(0),
            project.open_issues_count.unwrap_or(0)
        )),
    ]);

    print_json_or_table(&project, &table, json_mode)
}

async fn handle_list(
    args: ProjectListArgs,
    client: &GitLabClient,
    json_mode: bool,
) -> anyhow::Result<()> {
    let projects = client
        .list_projects(args.membership, Some(args.limit))
        .await?;

    let mut table = create_table(&["ID", "Path", "Visibility", "Stars", "Default Branch"]);
    for p in &projects {
        table.add_row(vec![
            colored_cell(p.id.to_string(), Color::Cyan, false),
            Cell::new(&p.path_with_namespace),
            colored_cell(
                p.visibility.as_deref().unwrap_or("unknown"),
                Color::Yellow,
                false,
            ),
            Cell::new(p.star_count.unwrap_or(0).to_string()),
            Cell::new(p.default_branch.as_deref().unwrap_or("-")),
        ]);
    }

    print_json_or_table(&projects, &table, json_mode)
}
