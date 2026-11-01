//! Command handler for `gitlab mr` / `gitlab merge-request`.

use crate::api::GitLabClient;
use crate::cli::mr::{
    MrApprovalsArgs, MrApproveArgs, MrArgs, MrCheckoutArgs, MrCloseArgs, MrCommands, MrCommentArgs,
    MrCreateArgs, MrDiffArgs, MrDiscussionsArgs, MrListArgs, MrMergeArgs, MrNoteArgs,
    MrNoteEditArgs, MrNotesArgs, MrRelinkArgs, MrReopenArgs, MrReplyArgs, MrResolveArgs,
    MrUpdateArgs, MrViewArgs,
};
use crate::commands::body::{read_body, read_required_body};
use crate::git::get_current_branch;
use crate::models::{Discussion, DiscussionPosition, MergeRequest, MergeRequestDiff, Note};
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
        MrCommands::Create(create_args) => {
            handle_create(create_args, client, project, json_mode).await
        }
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
        MrCommands::Diff(diff_args) => handle_diff(diff_args, client, project, json_mode).await,
        MrCommands::Discussions(disc_args) => {
            handle_discussions(disc_args, client, project, json_mode).await
        }
        MrCommands::Comment(comment_args) => {
            handle_comment(comment_args, client, project, json_mode).await
        }
        MrCommands::Reply(reply_args) => handle_reply(reply_args, client, project, json_mode).await,
        MrCommands::Resolve(resolve_args) => {
            handle_resolve(resolve_args, client, project, json_mode).await
        }
        MrCommands::NoteEdit(note_edit_args) => {
            handle_note_edit(note_edit_args, client, project, json_mode).await
        }
        MrCommands::Update(update_args) => {
            handle_update(update_args, client, project, json_mode).await
        }
        MrCommands::Relink(relink_args) => {
            handle_relink(relink_args, client, project, json_mode).await
        }
    }
}

async fn resolve_user_ids(client: &GitLabClient, usernames: &[String]) -> anyhow::Result<Vec<u64>> {
    let mut ids = Vec::new();
    for u in usernames {
        for item in u.split(',') {
            let clean = item.trim();
            if clean.is_empty() {
                continue;
            }
            if clean.eq_ignore_ascii_case("@me") {
                let me = client.current_user().await?;
                ids.push(me.id);
            } else {
                let lookup_name = clean.trim_start_matches('@');
                let user = client
                    .get_user_by_username(lookup_name)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("GitLab user '{clean}' not found"))?;
                ids.push(user.id);
            }
        }
    }
    Ok(ids)
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
            args.source_branch.as_deref(),
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
    let mr: MergeRequest = match &args.selector {
        Some(sel) => {
            if let Ok(iid) = sel.parse::<u64>() {
                client.get_mr(project, iid).await?
            } else {
                let mrs = client
                    .list_mrs(project, Some("opened"), None, Some(sel), Some(1))
                    .await?;
                mrs.into_iter().next().ok_or_else(|| {
                    anyhow::anyhow!(
                        "No open merge request found for branch '{sel}' in project '{project}'"
                    )
                })?
            }
        }
        None => {
            let current_dir = env::current_dir().unwrap_or_default();
            let branch = get_current_branch(&current_dir).map_err(|e| {
                anyhow::anyhow!(
                    "Could not determine current Git branch: {e}. Provide an IID or branch name explicitly."
                )
            })?;
            let mrs = client
                .list_mrs(project, Some("opened"), None, Some(&branch), Some(1))
                .await?;
            mrs.into_iter().next().ok_or_else(|| {
                anyhow::anyhow!(
                    "No open merge request found for current branch '{branch}' in project '{project}'"
                )
            })?
        }
    };

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
    json_mode: bool,
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

    let description = read_body(args.description.as_deref(), args.file.as_deref())?;

    let reviewer_ids = if !args.reviewers.is_empty() {
        Some(resolve_user_ids(client, &args.reviewers).await?)
    } else {
        None
    };

    let assignee_ids = if !args.assignees.is_empty() {
        Some(resolve_user_ids(client, &args.assignees).await?)
    } else {
        None
    };

    let labels = if !args.labels.is_empty() {
        Some(args.labels.join(","))
    } else {
        None
    };

    let mr = client
        .create_mr(&crate::api::CreateMrParams {
            project_path: project,
            source_branch: &source_branch,
            target_branch: &target_branch,
            title: &title,
            description: description.as_deref(),
            draft: args.draft,
            squash: args.squash,
            reviewer_ids,
            assignee_ids,
            labels,
            remove_source_branch: args.remove_source_branch,
        })
        .await?;

    if json_mode {
        println!("{}", serde_json::to_string_pretty(&mr)?);
        return Ok(());
    }

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

    let mr_head_ref = format!("refs/merge-requests/{}/head:{}", mr.iid, branch);
    let fetch_status = Command::new("git")
        .args(["fetch", "origin", &mr_head_ref])
        .status();

    let fetch_ok = match fetch_status {
        Ok(s) if s.success() => true,
        _ => {
            let refspec = format!("{branch}:refs/remotes/origin/{branch}");
            Command::new("git")
                .args(["fetch", "origin", &refspec])
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        }
    };

    if !fetch_ok {
        anyhow::bail!(
            "Failed to fetch merge request !{} branch '{branch}' from origin",
            mr.iid
        );
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
    let body = read_required_body(args.body.as_deref(), args.file.as_deref(), "Comment body:")?;

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

async fn handle_diff(
    args: MrDiffArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let diffs: Vec<MergeRequestDiff> = client.list_mr_diffs(project, args.iid, None).await?;

    if json_mode {
        println!("{}", serde_json::to_string_pretty(&diffs)?);
        return Ok(());
    }

    if args.patch {
        for d in &diffs {
            println!("diff --git a/{} b/{}", d.old_path, d.new_path);
            if d.new_file {
                println!("new file mode {}", d.b_mode.as_deref().unwrap_or("100644"));
            } else if d.deleted_file {
                println!(
                    "deleted file mode {}",
                    d.a_mode.as_deref().unwrap_or("100644")
                );
            } else if d.renamed_file {
                println!("rename from {}", d.old_path);
                println!("rename to {}", d.new_path);
            }
            println!("--- a/{}", d.old_path);
            println!("+++ b/{}", d.new_path);
            if !d.diff.is_empty() {
                println!("{}", d.diff);
            }
        }
        return Ok(());
    }

    let mut table = create_table(&["Old Path", "New Path", "Status"]);
    for d in &diffs {
        let status = if d.new_file {
            colored_cell("new", Color::Green, false)
        } else if d.deleted_file {
            colored_cell("deleted", Color::Red, false)
        } else if d.renamed_file {
            colored_cell("renamed", Color::Yellow, false)
        } else {
            colored_cell("modified", Color::Cyan, false)
        };

        table.add_row(vec![Cell::new(&d.old_path), Cell::new(&d.new_path), status]);
    }

    println!("{table}");
    Ok(())
}

async fn handle_discussions(
    args: MrDiscussionsArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let all_discussions: Vec<Discussion> =
        client.list_mr_discussions(project, args.iid, None).await?;

    let discussions: Vec<Discussion> = if args.unresolved {
        all_discussions
            .into_iter()
            .filter(|d| d.is_resolvable() && !d.is_resolved())
            .collect()
    } else {
        all_discussions
    };

    if json_mode {
        println!("{}", serde_json::to_string_pretty(&discussions)?);
        return Ok(());
    }

    if args.full {
        for d in &discussions {
            let status_str = if !d.is_resolvable() {
                "unresolvable"
            } else if d.is_resolved() {
                "resolved"
            } else {
                "unresolved"
            };

            println!("Thread {} [{}]", d.id.cyan().bold(), status_str);
            for note in &d.notes {
                let author = note
                    .author
                    .as_ref()
                    .map(|a| a.username.as_str())
                    .unwrap_or("unknown");
                let date = note.created_at.as_deref().unwrap_or("");
                println!(
                    "  Note #{} by @{} ({})",
                    note.id.to_string().yellow(),
                    author.bold(),
                    date
                );
                if let Some(pos) = &note.position {
                    let path = pos
                        .new_path
                        .as_deref()
                        .or(pos.old_path.as_deref())
                        .unwrap_or("-");
                    let line = pos
                        .new_line
                        .or(pos.old_line)
                        .map(|l| format!(":{l}"))
                        .unwrap_or_default();
                    println!("  Position: {path}{line}");
                }
                for line in note.body.lines() {
                    println!("    {line}");
                }
                println!();
            }
        }
        return Ok(());
    }

    let mut table = create_table(&["ID", "Resolved", "Position", "Author", "Comment"]);
    for d in &discussions {
        let resolved_cell = if !d.is_resolvable() {
            Cell::new("-")
        } else if d.is_resolved() {
            colored_cell("yes", Color::Green, false)
        } else {
            colored_cell("no", Color::Red, false)
        };

        let position = if let Some(pos) = d.position() {
            let path = pos
                .new_path
                .as_deref()
                .or(pos.old_path.as_deref())
                .unwrap_or("-");
            let line = pos
                .new_line
                .or(pos.old_line)
                .map(|l| format!(":{l}"))
                .unwrap_or_default();
            format!("{path}{line}")
        } else {
            "-".to_string()
        };

        let first_note = d.notes.first();
        let author = first_note
            .and_then(|n| n.author.as_ref())
            .map(|u| format!("@{}", u.username))
            .unwrap_or_else(|| "-".to_string());

        let comment = first_note
            .map(|n| n.body.lines().next().unwrap_or(""))
            .unwrap_or("");

        let short_id = if d.id.len() > 10 { &d.id[..10] } else { &d.id };

        table.add_row(vec![
            colored_cell(short_id, Color::Cyan, false),
            resolved_cell,
            Cell::new(position),
            Cell::new(author),
            Cell::new(comment),
        ]);
    }

    println!("{table}");
    Ok(())
}

async fn handle_comment(
    args: MrCommentArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let raw_body = read_required_body(args.body.as_deref(), args.file.as_deref(), "Comment body:")?;

    if (args.line.is_some() || args.old_line.is_some()) && args.path.is_none() {
        anyhow::bail!("--path is required when specifying --line or --old-line");
    }
    if args.path.is_some() && args.line.is_none() && args.old_line.is_none() {
        anyhow::bail!("At least one of --line or --old-line is required when specifying --path");
    }

    let body = if let Some(prefix) = &args.prefix {
        format!("{prefix}\n{raw_body}")
    } else {
        raw_body
    };

    let mr = client.get_mr(project, args.iid).await?;

    let position = if let Some(path) = &args.path {
        let diff_refs = mr.diff_refs.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Merge request !{} does not contain diff_refs", args.iid)
        })?;

        Some(DiscussionPosition {
            base_sha: diff_refs.base_sha.clone(),
            start_sha: diff_refs.start_sha.clone(),
            head_sha: diff_refs.head_sha.clone(),
            old_path: Some(path.clone()),
            new_path: Some(path.clone()),
            position_type: Some("text".to_string()),
            old_line: args.old_line,
            new_line: args.line,
        })
    } else {
        None
    };

    let disc = client
        .create_mr_discussion(project, args.iid, &body, position.as_ref())
        .await?;

    if json_mode {
        println!("{}", serde_json::to_string_pretty(&disc)?);
        return Ok(());
    }

    let first_note = disc.notes.first();
    let note_id = first_note.map(|n| n.id).unwrap_or(0);
    let note_url = first_note
        .and_then(|n| n.web_url.clone())
        .unwrap_or_else(|| {
            if let Some(mr_url) = &mr.web_url {
                format!("{mr_url}#note_{note_id}")
            } else {
                format!("note #{note_id}")
            }
        });

    println!(
        "{} Created discussion thread {}",
        "✓".green().bold(),
        disc.id.cyan().bold()
    );
    println!("  Note ID:  {note_id}");
    println!("  Note URL: {note_url}");

    Ok(())
}

async fn handle_reply(
    args: MrReplyArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let body = read_required_body(args.body.as_deref(), args.file.as_deref(), "Reply body:")?;

    let note: Note = client
        .create_discussion_note(project, args.iid, &args.discussion_id, &body)
        .await?;

    if json_mode {
        println!("{}", serde_json::to_string_pretty(&note)?);
        return Ok(());
    }

    println!(
        "{} Replied to discussion {} with note #{}",
        "✓".green().bold(),
        args.discussion_id.cyan(),
        note.id
    );
    if let Some(url) = &note.web_url {
        println!("  URL: {url}");
    }

    Ok(())
}

async fn handle_resolve(
    args: MrResolveArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let resolved = !args.undo;
    let disc: Discussion = client
        .resolve_mr_discussion(project, args.iid, &args.discussion_id, resolved)
        .await?;

    if json_mode {
        println!("{}", serde_json::to_string_pretty(&disc)?);
        return Ok(());
    }

    if resolved {
        println!(
            "{} Resolved discussion {}",
            "✓".green().bold(),
            args.discussion_id.cyan()
        );
    } else {
        println!(
            "{} Unresolved discussion {}",
            "✓".green().bold(),
            args.discussion_id.cyan()
        );
    }

    Ok(())
}

async fn handle_note_edit(
    args: MrNoteEditArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let raw_body = read_required_body(
        args.body.as_deref(),
        args.file.as_deref(),
        "Updated comment body:",
    )?;

    let body = if let Some(prefix) = &args.prefix {
        format!("{prefix}\n{raw_body}")
    } else {
        raw_body
    };

    let note: Note = client
        .update_mr_note(project, args.iid, args.note_id, &body)
        .await?;

    if json_mode {
        println!("{}", serde_json::to_string_pretty(&note)?);
        return Ok(());
    }

    println!(
        "{} Updated note #{} on MR !{}",
        "✓".green().bold(),
        note.id,
        args.iid
    );

    Ok(())
}

async fn handle_update(
    args: MrUpdateArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let description = read_body(args.description.as_deref(), args.file.as_deref())?;

    let reviewer_ids = if !args.reviewers.is_empty() {
        Some(resolve_user_ids(client, &args.reviewers).await?)
    } else {
        None
    };

    let assignee_ids = if !args.assignees.is_empty() {
        Some(resolve_user_ids(client, &args.assignees).await?)
    } else {
        None
    };

    let add_labels = if !args.add_labels.is_empty() {
        Some(args.add_labels.join(","))
    } else {
        None
    };

    let remove_labels = if !args.remove_labels.is_empty() {
        Some(args.remove_labels.join(","))
    } else {
        None
    };

    // Handle title and ready/draft transformation
    let title = if args.ready || args.draft {
        let current_mr = client.get_mr(project, args.iid).await?;
        let base_title = args.title.unwrap_or(current_mr.title);
        if args.ready {
            let stripped = base_title
                .trim_start_matches("Draft:")
                .trim_start_matches("WIP:")
                .trim()
                .to_string();
            Some(stripped)
        } else if args.draft && !base_title.starts_with("Draft:") {
            Some(format!("Draft: {base_title}"))
        } else {
            Some(base_title)
        }
    } else {
        args.title
    };

    let updated = client
        .update_mr(&crate::api::UpdateMrParams {
            project_path: project,
            iid: args.iid,
            title,
            description,
            add_labels,
            remove_labels,
            reviewer_ids,
            assignee_ids,
            state_event: None,
        })
        .await?;

    if json_mode {
        println!("{}", serde_json::to_string_pretty(&updated)?);
        return Ok(());
    }

    println!(
        "{} Updated merge request !{} ({})",
        "✓".green().bold(),
        updated.iid,
        updated.title.bold()
    );
    if let Some(url) = &updated.web_url {
        println!("  URL: {url}");
    }

    Ok(())
}

async fn handle_relink(
    args: MrRelinkArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let cur_dir = env::current_dir()?;
    let original_body = if let Some(nid) = args.note_id {
        let note = client.get_mr_note(project, args.iid, nid).await?;
        note.body
    } else {
        let mr = client.get_mr(project, args.iid).await?;
        mr.description.unwrap_or_default()
    };

    let mr_commits = client.list_mr_commits(project, args.iid).await?;

    struct Replacement {
        start: usize,
        end: usize,
        old_sha: String,
        new_sha: String,
        subject: String,
    }

    let mut candidates: Vec<(usize, usize, String)> = Vec::new();
    let bytes = original_body.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_hexdigit() {
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_hexdigit() {
                i += 1;
            }
            let len = i - start;
            if len == 40 || len == 8 {
                let prev_alphanumeric = if start > 0 {
                    bytes[start - 1].is_ascii_alphanumeric()
                } else {
                    false
                };
                let next_alphanumeric = if i < bytes.len() {
                    bytes[i].is_ascii_alphanumeric()
                } else {
                    false
                };

                if !prev_alphanumeric && !next_alphanumeric {
                    let token = original_body[start..i].to_string();
                    candidates.push((start, i, token));
                }
            }
        } else {
            i += 1;
        }
    }

    let mut replacements: Vec<Replacement> = Vec::new();
    for (start, end, sha) in candidates {
        // Look up old commit's subject in local git repository
        let subject = match crate::git::get_commit_subject(&cur_dir, &sha) {
            Ok(s) => s,
            Err(_) => continue, // Not a known commit locally, skip
        };

        // Match subject among current MR commits
        let matches: Vec<&crate::models::MrCommit> = mr_commits
            .iter()
            .filter(|c| c.title.trim() == subject.trim())
            .collect();

        if matches.len() == 1 {
            let new_full_id = &matches[0].id;
            let target_len = std::cmp::min(sha.len(), new_full_id.len());
            let replacement_sha = new_full_id[..target_len].to_string();

            if replacement_sha.to_lowercase() != sha.to_lowercase() {
                replacements.push(Replacement {
                    start,
                    end,
                    old_sha: sha,
                    new_sha: replacement_sha,
                    subject,
                });
            }
        } else if matches.is_empty() {
            eprintln!(
                "{} SHA {} has no matching commit in MR (subject: '{}')",
                "!".yellow(),
                sha,
                subject
            );
        } else {
            eprintln!(
                "{} SHA {} has ambiguous match ({} commits match subject: '{}')",
                "!".yellow(),
                sha,
                matches.len(),
                subject
            );
        }
    }

    // Apply replacements from back to front
    let mut rewritten = original_body.clone();
    for rep in replacements.iter().rev() {
        rewritten.replace_range(rep.start..rep.end, &rep.new_sha);
    }

    if json_mode {
        #[derive(serde::Serialize)]
        struct RelinkResult<'a> {
            iid: u64,
            note_id: Option<u64>,
            modified: bool,
            replacements_count: usize,
            body: &'a str,
        }
        let res = RelinkResult {
            iid: args.iid,
            note_id: args.note_id,
            modified: rewritten != original_body,
            replacements_count: replacements.len(),
            body: &rewritten,
        };
        println!("{}", serde_json::to_string_pretty(&res)?);
        return Ok(());
    }

    if args.dry_run {
        println!("{}", "Dry run relink diff:".bold());
        if rewritten == original_body {
            println!("No commit SHAs to rewrite.");
        } else {
            for rep in &replacements {
                println!(
                    "  {} -> {} ({})",
                    rep.old_sha.red(),
                    rep.new_sha.green(),
                    rep.subject
                );
            }
            println!();
            print!("{}", generate_line_diff(&original_body, &rewritten));
        }
        return Ok(());
    }

    if rewritten == original_body {
        println!("No commit SHAs needed updating.");
        return Ok(());
    }

    if let Some(nid) = args.note_id {
        client
            .update_mr_note(project, args.iid, nid, &rewritten)
            .await?;
        println!(
            "{} Updated note #{} with {} rewritten SHA(s).",
            "✓".green().bold(),
            nid,
            replacements.len()
        );
    } else {
        client
            .update_mr(&crate::api::UpdateMrParams {
                project_path: project,
                iid: args.iid,
                description: Some(rewritten),
                ..Default::default()
            })
            .await?;
        println!(
            "{} Updated MR !{} description with {} rewritten SHA(s).",
            "✓".green().bold(),
            args.iid,
            replacements.len()
        );
    }

    Ok(())
}

/// Generates a simple unified diff between original and rewritten text.
fn generate_line_diff(original: &str, rewritten: &str) -> String {
    let orig_lines: Vec<&str> = original.lines().collect();
    let new_lines: Vec<&str> = rewritten.lines().collect();

    let mut out = String::new();
    out.push_str("--- original\n+++ rewritten\n");

    let max_len = std::cmp::max(orig_lines.len(), new_lines.len());
    for i in 0..max_len {
        let orig = orig_lines.get(i).copied();
        let new = new_lines.get(i).copied();

        match (orig, new) {
            (Some(o), Some(n)) if o == n => {
                out.push_str(&format!("  {o}\n"));
            }
            (Some(o), Some(n)) => {
                out.push_str(&format!("- {o}\n"));
                out.push_str(&format!("+ {n}\n"));
            }
            (Some(o), None) => {
                out.push_str(&format!("- {o}\n"));
            }
            (None, Some(n)) => {
                out.push_str(&format!("+ {n}\n"));
            }
            (None, None) => {}
        }
    }
    out
}
