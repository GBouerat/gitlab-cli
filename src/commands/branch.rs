//! Command handler for `gitlab branch`.

use crate::api::GitLabClient;
use crate::cli::branch::{
    BranchArgs, BranchCommands, BranchCreateArgs, BranchDeleteArgs, BranchListArgs, BranchViewArgs,
};
use crate::git::get_current_branch;
use crate::ui::table::{colored_cell, create_table, print_json_or_table};
use colored::Colorize;
use comfy_table::{Cell, Color};
use std::env;

/// Dispatches branch subcommands.
///
/// # Arguments
///
/// * `args` - Branch command arguments.
/// * `client` - Authenticated GitLab API client.
/// * `project` - Target project path (e.g. `owner/repo`).
/// * `json_mode` - Whether to output raw JSON.
///
/// # Errors
///
/// Returns an error if API communication fails.
pub async fn execute_branch(
    args: BranchArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    match args.command {
        BranchCommands::List(list_args) => handle_list(list_args, client, project, json_mode).await,
        BranchCommands::View(view_args) => handle_view(view_args, client, project, json_mode).await,
        BranchCommands::Create(create_args) => handle_create(create_args, client, project).await,
        BranchCommands::Delete(delete_args) => handle_delete(delete_args, client, project).await,
    }
}

async fn handle_list(
    args: BranchListArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let branches = client.list_branches(project, Some(args.limit)).await?;

    let mut table = create_table(&["Branch", "Default", "Protected", "Latest Commit"]);
    for b in &branches {
        let is_default = if b.default.unwrap_or(false) {
            colored_cell("yes", Color::Green, true)
        } else {
            Cell::new("no")
        };

        let is_protected = if b.protected.unwrap_or(false) {
            colored_cell("yes", Color::Yellow, true)
        } else {
            Cell::new("no")
        };

        let commit_summary = if let Some(c) = &b.commit {
            format!(
                "{} {}",
                c.short_id.as_deref().unwrap_or(&c.id),
                c.title.as_deref().unwrap_or("")
            )
        } else {
            "-".to_string()
        };

        table.add_row(vec![
            colored_cell(&b.name, Color::Cyan, true),
            is_default,
            is_protected,
            Cell::new(commit_summary),
        ]);
    }

    print_json_or_table(&branches, &table, json_mode)
}

async fn handle_view(
    args: BranchViewArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let current_dir = env::current_dir().unwrap_or_default();
    let branch_name = match args.branch {
        Some(b) => b,
        None => get_current_branch(&current_dir)?,
    };

    let branch = client.get_branch(project, &branch_name).await?;

    if args.web {
        if let Some(url) = &branch.web_url {
            println!("Opening branch in browser: {url}");
            crate::ui::open_in_browser(url);
            return Ok(());
        }
    }

    let mut table = create_table(&["Property", "Value"]);
    table.add_row(vec![
        Cell::new("Branch"),
        colored_cell(&branch.name, Color::Cyan, true),
    ]);
    table.add_row(vec![
        Cell::new("Default"),
        Cell::new(if branch.default.unwrap_or(false) {
            "yes"
        } else {
            "no"
        }),
    ]);
    table.add_row(vec![
        Cell::new("Protected"),
        Cell::new(if branch.protected.unwrap_or(false) {
            "yes"
        } else {
            "no"
        }),
    ]);
    if let Some(c) = &branch.commit {
        table.add_row(vec![Cell::new("Commit SHA"), Cell::new(&c.id)]);
        if let Some(t) = &c.title {
            table.add_row(vec![Cell::new("Commit Title"), Cell::new(t)]);
        }
        if let Some(author) = &c.author_name {
            table.add_row(vec![Cell::new("Commit Author"), Cell::new(author)]);
        }
        if let Some(date) = &c.authored_date {
            table.add_row(vec![Cell::new("Authored Date"), Cell::new(date)]);
        }
    }
    if let Some(web) = &branch.web_url {
        table.add_row(vec![Cell::new("Web URL"), Cell::new(web)]);
    }

    print_json_or_table(&branch, &table, json_mode)
}

async fn handle_create(
    args: BranchCreateArgs,
    client: &GitLabClient,
    project: &str,
) -> anyhow::Result<()> {
    let current_dir = env::current_dir().unwrap_or_default();
    let ref_name = match args.ref_name {
        Some(r) => r,
        None => get_current_branch(&current_dir).unwrap_or_else(|_| "main".to_string()),
    };

    let branch = client
        .create_branch(project, &args.branch, &ref_name)
        .await?;

    println!(
        "{} Created branch {} from '{}'",
        "✓".green().bold(),
        branch.name.cyan().bold(),
        ref_name.bold()
    );
    if let Some(url) = &branch.web_url {
        println!("  URL: {url}");
    }

    Ok(())
}

async fn handle_delete(
    args: BranchDeleteArgs,
    client: &GitLabClient,
    project: &str,
) -> anyhow::Result<()> {
    client.delete_branch(project, &args.branch).await?;
    println!(
        "{} Deleted branch {}",
        "✓".green().bold(),
        args.branch.bold()
    );
    Ok(())
}
