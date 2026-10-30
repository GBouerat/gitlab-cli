//! Command handler for `gitlab deploy` / `gitlab deployment`.

use crate::api::GitLabClient;
use crate::cli::deployment::{
    DeploymentArgs, DeploymentCommands, DeploymentListArgs, DeploymentViewArgs,
};
use crate::ui::table::{colored_cell, create_table, print_json_or_table, status_cell};
use comfy_table::{Cell, Color};

/// Dispatches deployment subcommands.
///
/// # Arguments
///
/// * `args` - Deployment command arguments.
/// * `client` - Authenticated GitLab API client.
/// * `project` - Target project path (e.g. `owner/repo`).
/// * `json_mode` - Whether to output raw JSON.
///
/// # Errors
///
/// Returns an error if API communication fails.
pub async fn execute_deployment(
    args: DeploymentArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    match args.command {
        DeploymentCommands::List(list_args) => {
            handle_list(list_args, client, project, json_mode).await
        }
        DeploymentCommands::View(view_args) => {
            handle_view(view_args, client, project, json_mode).await
        }
    }
}

async fn handle_list(
    args: DeploymentListArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let deployments = client
        .list_deployments(project, args.status.as_deref(), Some(args.limit))
        .await?;

    let mut table = create_table(&["ID", "IID", "Environment", "Status", "Ref", "Created At"]);
    for d in &deployments {
        let env_name = d
            .environment
            .as_ref()
            .map(|e| e.name.as_str())
            .unwrap_or("-");

        table.add_row(vec![
            colored_cell(d.id.to_string(), Color::Cyan, false),
            Cell::new(d.iid.to_string()),
            Cell::new(env_name),
            status_cell(&d.status),
            Cell::new(&d.r#ref),
            Cell::new(d.created_at.as_deref().unwrap_or("-")),
        ]);
    }

    print_json_or_table(&deployments, &table, json_mode)
}

async fn handle_view(
    args: DeploymentViewArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let deployment = client.get_deployment(project, args.id).await?;

    let mut table = create_table(&["Property", "Value"]);
    table.add_row(vec![
        Cell::new("Deployment"),
        colored_cell(
            format!("#{} (IID: {})", deployment.id, deployment.iid),
            Color::Cyan,
            true,
        ),
    ]);
    table.add_row(vec![Cell::new("Status"), status_cell(&deployment.status)]);
    table.add_row(vec![Cell::new("Ref"), Cell::new(&deployment.r#ref)]);
    table.add_row(vec![Cell::new("SHA"), Cell::new(&deployment.sha)]);
    if let Some(e) = &deployment.environment {
        table.add_row(vec![
            Cell::new("Environment"),
            colored_cell(format!("#{} {}", e.id, e.name), Color::Green, true),
        ]);
    }
    if let Some(u) = &deployment.user {
        table.add_row(vec![
            Cell::new("User"),
            Cell::new(format!("{} (@{})", u.name, u.username)),
        ]);
    }
    if let Some(created) = &deployment.created_at {
        table.add_row(vec![Cell::new("Created At"), Cell::new(created)]);
    }

    print_json_or_table(&deployment, &table, json_mode)
}
