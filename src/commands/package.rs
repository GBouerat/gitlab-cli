//! Command handler for `gitlab package`.

use crate::api::GitLabClient;
use crate::cli::package::{
    PackageArgs, PackageCommands, PackageDeleteArgs, PackageListArgs, PackageViewArgs,
};
use crate::ui::table::{colored_cell, create_table, print_json_or_table};
use colored::Colorize;
use comfy_table::{Cell, Color};

/// Dispatches package subcommands.
///
/// # Arguments
///
/// * `args` - Package command arguments.
/// * `client` - Authenticated GitLab API client.
/// * `project` - Target project path (e.g. `owner/repo`).
/// * `json_mode` - Whether to output raw JSON.
///
/// # Errors
///
/// Returns an error if API communication fails.
pub async fn execute_package(
    args: PackageArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    match args.command {
        PackageCommands::List(list_args) => {
            handle_list(list_args, client, project, json_mode).await
        }
        PackageCommands::View(view_args) => {
            handle_view(view_args, client, project, json_mode).await
        }
        PackageCommands::Delete(delete_args) => handle_delete(delete_args, client, project).await,
    }
}

async fn handle_list(
    args: PackageListArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let packages = client
        .list_packages(project, args.package_type.as_deref(), Some(args.limit))
        .await?;

    let mut table = create_table(&["ID", "Name", "Version", "Type", "Status"]);
    for p in &packages {
        table.add_row(vec![
            colored_cell(p.id.to_string(), Color::Cyan, false),
            Cell::new(&p.name),
            Cell::new(&p.version),
            colored_cell(&p.package_type, Color::Yellow, false),
            Cell::new(p.status.as_deref().unwrap_or("default")),
        ]);
    }

    print_json_or_table(&packages, &table, json_mode)
}

async fn handle_view(
    args: PackageViewArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let package = client.get_package(project, args.id).await?;

    let mut table = create_table(&["Property", "Value"]);
    table.add_row(vec![
        Cell::new("Package"),
        colored_cell(
            format!("#{} {}", package.id, package.name),
            Color::Cyan,
            true,
        ),
    ]);
    table.add_row(vec![Cell::new("Version"), Cell::new(&package.version)]);
    table.add_row(vec![Cell::new("Type"), Cell::new(&package.package_type)]);
    if let Some(status) = &package.status {
        table.add_row(vec![Cell::new("Status"), Cell::new(status)]);
    }
    if let Some(created) = &package.created_at {
        table.add_row(vec![Cell::new("Created At"), Cell::new(created)]);
    }
    if let Some(files) = &package.package_files {
        let file_names: Vec<String> = files.iter().map(|f| f.file_name.clone()).collect();
        table.add_row(vec![Cell::new("Files"), Cell::new(file_names.join(", "))]);
    }

    print_json_or_table(&package, &table, json_mode)
}

async fn handle_delete(
    args: PackageDeleteArgs,
    client: &GitLabClient,
    project: &str,
) -> anyhow::Result<()> {
    client.delete_package(project, args.id).await?;
    println!("{} Deleted package #{}", "✓".green().bold(), args.id);
    Ok(())
}
