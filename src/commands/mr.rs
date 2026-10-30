//! Command handler for `gitlab mr` / `gitlab merge-request`.

use crate::api::GitLabClient;
use crate::cli::mr::{
    MrApprovalsArgs, MrApproveArgs, MrArgs, MrCheckoutArgs, MrCloseArgs, MrCommands, MrCreateArgs,
    MrListArgs, MrMergeArgs, MrNoteArgs, MrNotesArgs, MrReopenArgs, MrViewArgs,
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
        MrCommands::Approve(approve_args) => handle_approve(approve_args, client, project).await,
        MrCommands::Unapprove(unapprove_args) => {
            handle_unapprove(unapprove_args, client, project).await
        }
        MrCommands::Approvals(approvals_args) => {
            handle_approvals(approvals_args, client, project, json_mode).await
        }
        MrCommands::Note(note_args) => handle_note(note_args, client, project).await,
        MrCommands::Notes(notes_args) => handle_notes(notes_args, client, project, json_mode).await,
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

async fn handle_approve(
    args: MrApproveArgs,
    client: &GitLabClient,
    project: &str,
) -> anyhow::Result<()> {
    let approvals = client.approve_mr(project, args.iid).await?;
    println!(
        "{} Approved merge request !{} (approvals left: {})",
        "✓".green().bold(),
        args.iid,
        approvals.approvals_left
    );
    Ok(())
}

async fn handle_unapprove(
    args: MrApproveArgs,
    client: &GitLabClient,
    project: &str,
) -> anyhow::Result<()> {
    client.unapprove_mr(project, args.iid).await?;
    println!(
        "{} Revoked approval for merge request !{}",
        "✓".green().bold(),
        args.iid
    );
    Ok(())
}

async fn handle_approvals(
    args: MrApprovalsArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let approvals = client.get_mr_approvals(project, args.iid).await?;

    let mut table = create_table(&["Property", "Value"]);
    table.add_row(vec![
        Cell::new("Merge Request"),
        colored_cell(format!("!{}", args.iid), Color::Cyan, true),
    ]);
    table.add_row(vec![
        Cell::new("Approvals Required"),
        Cell::new(approvals.approvals_required.to_string()),
    ]);
    table.add_row(vec![
        Cell::new("Approvals Left"),
        colored_cell(
            approvals.approvals_left.to_string(),
            if approvals.approvals_left == 0 {
                Color::Green
            } else {
                Color::Yellow
            },
            true,
        ),
    ]);
    let approvers: Vec<String> = approvals
        .approved_by
        .iter()
        .map(|a| format!("{} (@{})", a.user.name, a.user.username))
        .collect();
    table.add_row(vec![
        Cell::new("Approved By"),
        Cell::new(if approvers.is_empty() {
            "None".to_string()
        } else {
            approvers.join(", ")
        }),
    ]);

    print_json_or_table(&approvals, &table, json_mode)
}

async fn handle_note(args: MrNoteArgs, client: &GitLabClient, project: &str) -> anyhow::Result<()> {
    let body = match args.body {
        Some(b) => b,
        None => Text::new("Comment body:").prompt()?,
    };

    let note = client.create_mr_note(project, args.iid, &body).await?;
    println!(
        "{} Added comment #{} to merge request !{}",
        "✓".green().bold(),
        note.id,
        args.iid
    );
    Ok(())
}

async fn handle_notes(
    args: MrNotesArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let notes = client
        .list_mr_notes(project, args.iid, Some(args.limit))
        .await?;

    let mut table = create_table(&["ID", "Author", "Type", "Created At", "Comment"]);
    for n in &notes {
        let author = n
            .author
            .as_ref()
            .map(|u| format!("@{}", u.username))
            .unwrap_or_else(|| "-".to_string());
        let note_type = if n.system { "system" } else { "user" };
        let snippet = if n.body.len() > 60 {
            format!("{}...", &n.body[..57])
        } else {
            n.body.clone()
        };

        table.add_row(vec![
            colored_cell(n.id.to_string(), Color::Cyan, false),
            Cell::new(author),
            colored_cell(
                note_type,
                if n.system {
                    Color::DarkGrey
                } else {
                    Color::Green
                },
                false,
            ),
            Cell::new(n.created_at.as_deref().unwrap_or("-")),
            Cell::new(snippet),
        ]);
    }

    print_json_or_table(&notes, &table, json_mode)
}
