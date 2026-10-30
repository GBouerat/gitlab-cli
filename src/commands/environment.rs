//! Command handler for `gitlab env` / `gitlab environment`.

use crate::api::GitLabClient;
use crate::cli::environment::{
    EnvironmentArgs, EnvironmentCommands, EnvironmentCreateArgs, EnvironmentDeleteArgs,
    EnvironmentListArgs, EnvironmentStopArgs, EnvironmentViewArgs,
};
use crate::ui::open_in_browser;
use crate::ui::table::{colored_cell, create_table, print_json_or_table};
use colored::Colorize;
use comfy_table::{Cell, Color};

/// Dispatches environment subcommands.
///
/// # Arguments
///
/// * `args` - Environment command arguments.
/// * `client` - Authenticated GitLab API client.
/// * `project` - Target project path (e.g. `owner/repo`).
/// * `json_mode` - Whether to output raw JSON.
///
/// # Errors
///
/// Returns an error if API communication fails.
pub async fn execute_environment(
    args: EnvironmentArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    match args.command {
        EnvironmentCommands::List(list_args) => {
            handle_list(list_args, client, project, json_mode).await
        }
        EnvironmentCommands::View(view_args) => {
            handle_view(view_args, client, project, json_mode).await
        }
        EnvironmentCommands::Create(create_args) => {
            handle_create(create_args, client, project).await
        }
        EnvironmentCommands::Stop(stop_args) => handle_stop(stop_args, client, project).await,
        EnvironmentCommands::Delete(delete_args) => {
            handle_delete(delete_args, client, project).await
        }
    }
}

async fn handle_list(
    args: EnvironmentListArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let envs = client.list_environments(project, Some(args.limit)).await?;

    let mut table = create_table(&["ID", "Name", "State", "External URL"]);
    for e in &envs {
        let state_cell = match e.state.as_deref() {
            Some("available") => colored_cell("available", Color::Green, false),
            Some("stopped") => colored_cell("stopped", Color::DarkGrey, false),
            Some(other) => Cell::new(other),
            None => Cell::new("-"),
        };

        table.add_row(vec![
            colored_cell(e.id.to_string(), Color::Cyan, false),
            Cell::new(&e.name),
            state_cell,
            Cell::new(e.external_url.as_deref().unwrap_or("-")),
        ]);
    }

    print_json_or_table(&envs, &table, json_mode)
}

async fn handle_view(
    args: EnvironmentViewArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let env = client.get_environment(project, args.id).await?;

    if args.web {
        if let Some(url) = &env.external_url {
            open_in_browser(url);
            return Ok(());
        } else if let Some(web_url) = &env.web_url {
            open_in_browser(web_url);
            return Ok(());
        }
    }

    let mut table = create_table(&["Property", "Value"]);
    table.add_row(vec![
        Cell::new("Environment"),
        colored_cell(format!("#{} {}", env.id, env.name), Color::Cyan, true),
    ]);
    if let Some(slug) = &env.slug {
        table.add_row(vec![Cell::new("Slug"), Cell::new(slug)]);
    }
    table.add_row(vec![
        Cell::new("State"),
        colored_cell(
            env.state.as_deref().unwrap_or("-"),
            if env.state.as_deref() == Some("available") {
                Color::Green
            } else {
                Color::DarkGrey
            },
            true,
        ),
    ]);
    if let Some(url) = &env.external_url {
        table.add_row(vec![Cell::new("External URL"), Cell::new(url)]);
    }
    if let Some(created) = &env.created_at {
        table.add_row(vec![Cell::new("Created At"), Cell::new(created)]);
    }
    if let Some(web) = &env.web_url {
        table.add_row(vec![Cell::new("Web URL"), Cell::new(web)]);
    }

    print_json_or_table(&env, &table, json_mode)
}

async fn handle_create(
    args: EnvironmentCreateArgs,
    client: &GitLabClient,
    project: &str,
) -> anyhow::Result<()> {
    let env = client
        .create_environment(project, &args.name, args.external_url.as_deref())
        .await?;
    println!(
        "{} Created environment #{} ({})",
        "✓".green().bold(),
        env.id,
        env.name.bold()
    );
    Ok(())
}

async fn handle_stop(
    args: EnvironmentStopArgs,
    client: &GitLabClient,
    project: &str,
) -> anyhow::Result<()> {
    let env = client.stop_environment(project, args.id).await?;
    println!(
        "{} Stopped environment #{} ({})",
        "✓".green().bold(),
        env.id,
        env.name.bold()
    );
    Ok(())
}

async fn handle_delete(
    args: EnvironmentDeleteArgs,
    client: &GitLabClient,
    project: &str,
) -> anyhow::Result<()> {
    client.delete_environment(project, args.id).await?;
    println!("{} Deleted environment #{}", "✓".green().bold(), args.id);
    Ok(())
}
