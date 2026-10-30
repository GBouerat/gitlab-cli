//! Command handler for `gitlab group`.

use crate::api::GitLabClient;
use crate::cli::group::{GroupArgs, GroupCommands, GroupListArgs, GroupMembersArgs, GroupViewArgs};
use crate::ui::table::{colored_cell, create_table, print_json_or_table};
use comfy_table::{Cell, Color};

/// Dispatches group subcommands.
///
/// # Arguments
///
/// * `args` - Group command arguments.
/// * `client` - Authenticated GitLab API client.
/// * `json_mode` - Whether to output raw JSON.
///
/// # Errors
///
/// Returns an error if API communication fails.
pub async fn execute_group(
    args: GroupArgs,
    client: &GitLabClient,
    json_mode: bool,
) -> anyhow::Result<()> {
    match args.command {
        GroupCommands::List(list_args) => handle_list(list_args, client, json_mode).await,
        GroupCommands::View(view_args) => handle_view(view_args, client, json_mode).await,
        GroupCommands::Members(members_args) => {
            handle_members(members_args, client, json_mode).await
        }
    }
}

async fn handle_list(
    args: GroupListArgs,
    client: &GitLabClient,
    json_mode: bool,
) -> anyhow::Result<()> {
    let groups = client
        .list_groups(args.search.as_deref(), Some(args.limit))
        .await?;

    let mut table = create_table(&["ID", "Name", "Full Path", "Visibility"]);
    for g in &groups {
        table.add_row(vec![
            colored_cell(g.id.to_string(), Color::Cyan, false),
            Cell::new(&g.name),
            Cell::new(&g.full_path),
            colored_cell(g.visibility.as_deref().unwrap_or("-"), Color::Yellow, false),
        ]);
    }

    print_json_or_table(&groups, &table, json_mode)
}

async fn handle_view(
    args: GroupViewArgs,
    client: &GitLabClient,
    json_mode: bool,
) -> anyhow::Result<()> {
    let group = client.get_group(&args.group).await?;

    if args.web {
        if let Some(url) = &group.web_url {
            println!("Opening group in browser: {url}");
            crate::ui::open_in_browser(url);
            return Ok(());
        }
    }

    if json_mode {
        let json = serde_json::to_string_pretty(&group)?;
        println!("{json}");
        return Ok(());
    }

    let mut table = create_table(&["Property", "Value"]);
    table.add_row(vec![
        Cell::new("Group"),
        colored_cell(&group.name, Color::Cyan, true),
    ]);
    table.add_row(vec![
        Cell::new("ID"),
        colored_cell(group.id.to_string(), Color::Cyan, false),
    ]);
    table.add_row(vec![Cell::new("Path"), Cell::new(&group.full_path)]);
    if let Some(vis) = &group.visibility {
        table.add_row(vec![
            Cell::new("Visibility"),
            colored_cell(vis, Color::Yellow, true),
        ]);
    }
    if let Some(desc) = &group.description {
        table.add_row(vec![Cell::new("Description"), Cell::new(desc)]);
    }
    if let Some(web) = &group.web_url {
        table.add_row(vec![Cell::new("Web URL"), Cell::new(web)]);
    }

    println!("{table}");
    Ok(())
}

async fn handle_members(
    args: GroupMembersArgs,
    client: &GitLabClient,
    json_mode: bool,
) -> anyhow::Result<()> {
    let members = client
        .list_group_members(&args.group, Some(args.limit))
        .await?;

    let mut table = create_table(&["User ID", "Username", "Name", "Role"]);
    for m in &members {
        table.add_row(vec![
            colored_cell(m.id.to_string(), Color::Cyan, false),
            Cell::new(&m.username),
            Cell::new(&m.name),
            colored_cell(m.role_name(), Color::Green, true),
        ]);
    }

    print_json_or_table(&members, &table, json_mode)
}
