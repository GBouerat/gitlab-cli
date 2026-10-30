//! Command handler for `gitlab milestone`.

use crate::api::GitLabClient;
use crate::cli::milestone::{
    MilestoneArgs, MilestoneCloseArgs, MilestoneCommands, MilestoneCreateArgs, MilestoneDeleteArgs,
    MilestoneListArgs, MilestoneViewArgs,
};
use crate::ui::open_in_browser;
use crate::ui::table::{colored_cell, create_table, print_json_or_table};
use colored::Colorize;
use comfy_table::{Cell, Color};

/// Dispatches milestone subcommands.
///
/// # Arguments
///
/// * `args` - Milestone command arguments.
/// * `client` - Authenticated GitLab API client.
/// * `project` - Target project path (e.g. `owner/repo`).
/// * `json_mode` - Whether to output raw JSON.
///
/// # Errors
///
/// Returns an error if API communication fails.
pub async fn execute_milestone(
    args: MilestoneArgs,
    client: &GitLabClient,
    project: Option<&str>,
    json_mode: bool,
) -> anyhow::Result<()> {
    match args.command {
        MilestoneCommands::List(list_args) => {
            handle_list(list_args, client, project, json_mode).await
        }
        MilestoneCommands::View(view_args) => {
            let proj = project.ok_or_else(|| {
                anyhow::anyhow!("No project specified. Provide `--repo <owner/repo>`.")
            })?;
            handle_view(view_args, client, proj, json_mode).await
        }
        MilestoneCommands::Create(create_args) => {
            let proj = project.ok_or_else(|| {
                anyhow::anyhow!("No project specified. Provide `--repo <owner/repo>`.")
            })?;
            handle_create(create_args, client, proj).await
        }
        MilestoneCommands::Close(close_args) => {
            let proj = project.ok_or_else(|| {
                anyhow::anyhow!("No project specified. Provide `--repo <owner/repo>`.")
            })?;
            handle_close(close_args, client, proj).await
        }
        MilestoneCommands::Delete(delete_args) => {
            let proj = project.ok_or_else(|| {
                anyhow::anyhow!("No project specified. Provide `--repo <owner/repo>`.")
            })?;
            handle_delete(delete_args, client, proj).await
        }
    }
}

async fn handle_list(
    args: MilestoneListArgs,
    client: &GitLabClient,
    project: Option<&str>,
    json_mode: bool,
) -> anyhow::Result<()> {
    let milestones = if let Some(grp) = &args.group {
        client
            .list_group_milestones(grp, args.state.as_deref(), Some(args.limit))
            .await?
    } else {
        let proj = project.ok_or_else(|| {
            anyhow::anyhow!(
                "No project or group specified. Use `--repo <owner/repo>` or `--group <group>`."
            )
        })?;
        client
            .list_project_milestones(proj, args.state.as_deref(), Some(args.limit))
            .await?
    };

    let mut table = create_table(&["ID", "Title", "State", "Due Date"]);
    for m in &milestones {
        let state_cell = match m.state.as_deref() {
            Some("active") => colored_cell("active", Color::Green, false),
            Some("closed") => colored_cell("closed", Color::DarkGrey, false),
            Some(other) => Cell::new(other),
            None => Cell::new("-"),
        };

        table.add_row(vec![
            colored_cell(m.id.to_string(), Color::Cyan, false),
            Cell::new(&m.title),
            state_cell,
            Cell::new(m.due_date.as_deref().unwrap_or("-")),
        ]);
    }

    print_json_or_table(&milestones, &table, json_mode)
}

async fn handle_view(
    args: MilestoneViewArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let milestone = client.get_milestone(project, args.id).await?;

    if args.web {
        if let Some(web_url) = &milestone.web_url {
            open_in_browser(web_url);
            return Ok(());
        }
    }

    let mut table = create_table(&["Property", "Value"]);
    table.add_row(vec![
        Cell::new("Milestone"),
        colored_cell(
            format!("#{} {}", milestone.id, milestone.title),
            Color::Cyan,
            true,
        ),
    ]);
    table.add_row(vec![
        Cell::new("State"),
        colored_cell(
            milestone.state.as_deref().unwrap_or("-"),
            if milestone.state.as_deref() == Some("active") {
                Color::Green
            } else {
                Color::DarkGrey
            },
            true,
        ),
    ]);
    if let Some(due) = &milestone.due_date {
        table.add_row(vec![Cell::new("Due Date"), Cell::new(due)]);
    }
    if let Some(start) = &milestone.start_date {
        table.add_row(vec![Cell::new("Start Date"), Cell::new(start)]);
    }
    if let Some(desc) = &milestone.description {
        table.add_row(vec![Cell::new("Description"), Cell::new(desc)]);
    }

    print_json_or_table(&milestone, &table, json_mode)
}

async fn handle_create(
    args: MilestoneCreateArgs,
    client: &GitLabClient,
    project: &str,
) -> anyhow::Result<()> {
    let milestone = client
        .create_milestone(
            project,
            &args.title,
            args.description.as_deref(),
            args.due_date.as_deref(),
            args.start_date.as_deref(),
        )
        .await?;

    println!(
        "{} Created milestone #{} ({})",
        "✓".green().bold(),
        milestone.id,
        milestone.title.bold()
    );
    Ok(())
}

async fn handle_close(
    args: MilestoneCloseArgs,
    client: &GitLabClient,
    project: &str,
) -> anyhow::Result<()> {
    let milestone = client.close_milestone(project, args.id).await?;
    println!(
        "{} Closed milestone #{} ({})",
        "✓".green().bold(),
        milestone.id,
        milestone.title.bold()
    );
    Ok(())
}

async fn handle_delete(
    args: MilestoneDeleteArgs,
    client: &GitLabClient,
    project: &str,
) -> anyhow::Result<()> {
    client.delete_milestone(project, args.id).await?;
    println!("{} Deleted milestone #{}", "✓".green().bold(), args.id);
    Ok(())
}
