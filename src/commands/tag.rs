//! Command handler for `gitlab tag`.

use crate::api::GitLabClient;
use crate::cli::tag::{
    TagArgs, TagCommands, TagCreateArgs, TagDeleteArgs, TagListArgs, TagViewArgs,
};
use crate::ui::table::{colored_cell, create_table, print_json_or_table};
use colored::Colorize;
use comfy_table::{Cell, Color};

/// Dispatches tag subcommands.
///
/// # Arguments
///
/// * `args` - Tag command arguments.
/// * `client` - Authenticated GitLab API client.
/// * `project` - Target project path (e.g. `owner/repo`).
/// * `json_mode` - Whether to output raw JSON.
///
/// # Errors
///
/// Returns an error if API communication fails.
pub async fn execute_tag(
    args: TagArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    match args.command {
        TagCommands::List(list_args) => handle_list(list_args, client, project, json_mode).await,
        TagCommands::View(view_args) => handle_view(view_args, client, project, json_mode).await,
        TagCommands::Create(create_args) => handle_create(create_args, client, project).await,
        TagCommands::Delete(delete_args) => handle_delete(delete_args, client, project).await,
    }
}

async fn handle_list(
    args: TagListArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let tags = client.list_tags(project, Some(args.limit)).await?;

    let mut table = create_table(&["Tag", "Commit", "Protected", "Message"]);
    for t in &tags {
        let commit_sha = t
            .commit
            .as_ref()
            .and_then(|c| c.short_id.clone())
            .unwrap_or_else(|| "-".to_string());
        let is_protected = if t.protected.unwrap_or(false) {
            colored_cell("yes", Color::Yellow, true)
        } else {
            Cell::new("no")
        };
        let msg = t.message.as_deref().unwrap_or("-");

        table.add_row(vec![
            colored_cell(&t.name, Color::Cyan, true),
            Cell::new(commit_sha),
            is_protected,
            Cell::new(msg),
        ]);
    }

    print_json_or_table(&tags, &table, json_mode)
}

async fn handle_view(
    args: TagViewArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let tag = client.get_tag(project, &args.name).await?;

    let mut table = create_table(&["Property", "Value"]);
    table.add_row(vec![
        Cell::new("Tag Name"),
        colored_cell(&tag.name, Color::Cyan, true),
    ]);
    if let Some(msg) = &tag.message {
        table.add_row(vec![Cell::new("Message"), Cell::new(msg)]);
    }
    table.add_row(vec![
        Cell::new("Protected"),
        Cell::new(if tag.protected.unwrap_or(false) {
            "yes"
        } else {
            "no"
        }),
    ]);
    if let Some(c) = &tag.commit {
        table.add_row(vec![Cell::new("Commit SHA"), Cell::new(&c.id)]);
        if let Some(title) = &c.title {
            table.add_row(vec![Cell::new("Commit Title"), Cell::new(title)]);
        }
    }
    if let Some(r) = &tag.release {
        table.add_row(vec![
            Cell::new("Release"),
            colored_cell(&r.tag_name, Color::Green, true),
        ]);
    }

    print_json_or_table(&tag, &table, json_mode)
}

async fn handle_create(
    args: TagCreateArgs,
    client: &GitLabClient,
    project: &str,
) -> anyhow::Result<()> {
    let tag = client
        .create_tag(project, &args.name, &args.r#ref, args.message.as_deref())
        .await?;
    println!(
        "{} Created tag {} at {}",
        "✓".green().bold(),
        tag.name.bold(),
        args.r#ref
    );
    Ok(())
}

async fn handle_delete(
    args: TagDeleteArgs,
    client: &GitLabClient,
    project: &str,
) -> anyhow::Result<()> {
    client.delete_tag(project, &args.name).await?;
    println!("{} Deleted tag {}", "✓".green().bold(), args.name.bold());
    Ok(())
}
