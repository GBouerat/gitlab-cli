//! Command handler for `gitlab mr` / `gitlab merge-request`.

use crate::api::GitLabClient;
use crate::cli::mr::{
    MrArgs, MrCheckoutArgs, MrCloseArgs, MrCommands, MrCreateArgs, MrListArgs, MrMergeArgs,
    MrReopenArgs, MrViewArgs,
};
use crate::git::get_current_branch;
use crate::ui::table::{colored_cell, create_table, print_json_or_table, status_cell};
use colored::Colorize;
use comfy_table::{Cell, Color};
use inquire::Text;
use std::env;
use std::process::Command;

/// Dispatches merge request subcommands.
///
/// # Arguments
///
/// * `args` - Merge request command arguments.
/// * `client` - Authenticated GitLab API client.
/// * `project` - Target project path (e.g. `owner/repo`).
/// * `json_mode` - Whether to output raw JSON.
///
/// # Errors
///
/// Returns an error if API communication or Git branch checkout fails.
pub async fn execute_mr(
    args: MrArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    match args.command {
        MrCommands::List(list_args) => handle_list(list_args, client, project, json_mode).await,
        MrCommands::View(view_args) => handle_view(view_args, client, project, json_mode).await,
        MrCommands::Create(create_args) => handle_create(create_args, client, project).await,
        MrCommands::Checkout(checkout_args) => {
            handle_checkout(checkout_args, client, project).await
        }
        MrCommands::Merge(merge_args) => handle_merge(merge_args, client, project).await,
        MrCommands::Close(close_args) => handle_close(close_args, client, project).await,
        MrCommands::Reopen(reopen_args) => handle_reopen(reopen_args, client, project).await,
    }
}

async fn handle_list(
    args: MrListArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let state_filter = if args.state == "all" {
        None
    } else {
        Some(args.state.as_str())
    };

    let mrs = client
        .list_mrs(
            project,
            state_filter,
            args.target_branch.as_deref(),
            Some(args.limit),
        )
        .await?;

    let mut table = create_table(&["!IID", "Title", "Branch", "Author", "Status"]);
    for mr in &mrs {
        let branch_display = format!("{} → {}", mr.source_branch, mr.target_branch);
        let author_name = mr
            .author
            .as_ref()
            .map(|a| a.username.as_str())
            .unwrap_or("-");

        table.add_row(vec![
            colored_cell(format!("!{}", mr.iid), Color::Cyan, true),
            Cell::new(&mr.title),
            Cell::new(branch_display),
            Cell::new(author_name),
            status_cell(&mr.state),
        ]);
    }

    print_json_or_table(&mrs, &table, json_mode)
}

async fn handle_view(
    args: MrViewArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let mr = client.get_mr(project, args.iid).await?;

    if args.web {
        if let Some(url) = &mr.web_url {
            println!("Opening MR in browser: {url}");
            crate::ui::open_in_browser(url);
            return Ok(());
        }
    }

    if json_mode {
        let json = serde_json::to_string_pretty(&mr)?;
        println!("{json}");
        return Ok(());
    }

    let mut table = create_table(&["Property", "Value"]);
    table.add_row(vec![
        Cell::new("Merge Request"),
        colored_cell(format!("!{} {}", mr.iid, mr.title), Color::Cyan, true),
    ]);
    table.add_row(vec![Cell::new("Status"), status_cell(&mr.state)]);
    table.add_row(vec![
        Cell::new("Author"),
        Cell::new(
            mr.author
                .as_ref()
                .map(|a| a.username.as_str())
                .unwrap_or("-"),
        ),
    ]);
    table.add_row(vec![
        Cell::new("Branches"),
        Cell::new(format!("{} → {}", mr.source_branch, mr.target_branch)),
    ]);
    if let Some(draft) = mr.draft {
        table.add_row(vec![
            Cell::new("Draft"),
            Cell::new(if draft { "yes" } else { "no" }),
        ]);
    }
    if let Some(conflicts) = mr.has_conflicts {
        table.add_row(vec![
            Cell::new("Conflicts"),
            Cell::new(if conflicts { "yes" } else { "none" }),
        ]);
    }
    if let Some(web) = &mr.web_url {
        table.add_row(vec![Cell::new("Web URL"), Cell::new(web)]);
    }
    if let Some(desc) = &mr.description {
        table.add_row(vec![Cell::new("Description"), Cell::new(desc)]);
    }

    println!("{table}");
    Ok(())
}

async fn handle_create(
    args: MrCreateArgs,
    client: &GitLabClient,
    project: &str,
) -> anyhow::Result<()> {
    let current_dir = env::current_dir()?;
    let source_branch = match args.source_branch {
        Some(s) => s,
        None => get_current_branch(&current_dir)?,
    };

    let target_branch = match args.target_branch {
        Some(t) => t,
        None => {
            let proj_info = client.get_project(project).await?;
            proj_info
                .default_branch
                .unwrap_or_else(|| "main".to_string())
        }
    };

    let title = match args.title {
        Some(t) => t,
        None => Text::new("Merge Request Title:")
            .with_default(&source_branch)
            .prompt()?,
    };

    let mr = client
        .create_mr(&crate::api::CreateMrParams {
            project_path: project,
            source_branch: &source_branch,
            target_branch: &target_branch,
            title: &title,
            description: args.description.as_deref(),
            draft: args.draft,
            squash: args.squash,
        })
        .await?;

    println!(
        "{} Created merge request !{} ({})",
        "✓".green().bold(),
        mr.iid,
        mr.title.bold()
    );
    if let Some(url) = &mr.web_url {
        println!("  URL: {url}");
    }

    Ok(())
}

async fn handle_checkout(
    args: MrCheckoutArgs,
    client: &GitLabClient,
    project: &str,
) -> anyhow::Result<()> {
    let mr = client.get_mr(project, args.iid).await?;
    let branch = &mr.source_branch;

    println!(
        "Checking out branch '{}' from merge request !{}",
        branch.cyan(),
        mr.iid
    );

    let refspec = format!("{branch}:refs/remotes/origin/{branch}");
    let fetch_status = Command::new("git")
        .args(["fetch", "origin", &refspec])
        .status()?;

    if !fetch_status.success() {
        anyhow::bail!("Failed to fetch branch '{branch}' from origin");
    }

    let checkout_status = Command::new("git").args(["checkout", branch]).status()?;

    if !checkout_status.success() {
        let checkout_track = Command::new("git")
            .args(["checkout", "-b", branch, &format!("origin/{branch}")])
            .status()?;
        if !checkout_track.success() {
            anyhow::bail!("Failed to switch to branch '{branch}'");
        }
    }

    println!(
        "{} Switched to branch '{}'",
        "✓".green().bold(),
        branch.bold()
    );
    Ok(())
}

async fn handle_merge(
    args: MrMergeArgs,
    client: &GitLabClient,
    project: &str,
) -> anyhow::Result<()> {
    let mr = client
        .merge_mr(project, args.iid, args.squash, args.remove_source_branch)
        .await?;

    println!(
        "{} Merged merge request !{} ({})",
        "✓".green().bold(),
        mr.iid,
        mr.title.bold()
    );
    Ok(())
}

async fn handle_close(
    args: MrCloseArgs,
    client: &GitLabClient,
    project: &str,
) -> anyhow::Result<()> {
    let mr = client.close_mr(project, args.iid).await?;
    println!(
        "{} Closed merge request !{} ({})",
        "✓".green().bold(),
        mr.iid,
        mr.title.bold()
    );
    Ok(())
}

async fn handle_reopen(
    args: MrReopenArgs,
    client: &GitLabClient,
    project: &str,
) -> anyhow::Result<()> {
    let mr = client.reopen_mr(project, args.iid).await?;
    println!(
        "{} Reopened merge request !{} ({})",
        "✓".green().bold(),
        mr.iid,
        mr.title.bold()
    );
    Ok(())
}
