//! Command handler for `gitlab issue`.

use crate::api::GitLabClient;
use crate::cli::issue::{
    IssueArgs, IssueCloseArgs, IssueCommands, IssueCreateArgs, IssueListArgs, IssueReopenArgs,
    IssueViewArgs,
};
use crate::ui::table::{colored_cell, create_table, print_json_or_table, status_cell};
use colored::Colorize;
use comfy_table::{Cell, Color};
use inquire::Text;

/// Dispatches issue subcommands.
///
/// # Arguments
///
/// * `args` - Issue command arguments.
/// * `client` - Authenticated GitLab API client.
/// * `project` - Target project path (e.g. `owner/repo`).
/// * `json_mode` - Whether to output raw JSON.
///
/// # Errors
///
/// Returns an error if API communication or display formatting fails.
pub async fn execute_issue(
    args: IssueArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    match args.command {
        IssueCommands::List(list_args) => {
            handle_list(list_args, client, project, json_mode).await
        }
        IssueCommands::View(view_args) => {
            handle_view(view_args, client, project, json_mode).await
        }
        IssueCommands::Create(create_args) => handle_create(create_args, client, project).await,
        IssueCommands::Close(close_args) => handle_close(close_args, client, project).await,
        IssueCommands::Reopen(reopen_args) => handle_reopen(reopen_args, client, project).await,
    }
}

async fn handle_list(
    args: IssueListArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let state_filter = if args.state == "all" {
        None
    } else {
        Some(args.state.as_str())
    };

    let issues = client
        .list_issues(
            project,
            state_filter,
            args.labels.as_deref(),
            Some(args.limit),
        )
        .await?;

    let mut table = create_table(&["#IID", "Title", "Author", "Status", "Labels"]);
    for issue in &issues {
        let author_name = issue
            .author
            .as_ref()
            .map(|a| a.username.as_str())
            .unwrap_or("-");
        let labels_display = if issue.labels.is_empty() {
            "-".to_string()
        } else {
            issue.labels.join(", ")
        };

        table.add_row(vec![
            colored_cell(format!("#{}", issue.iid), Color::Cyan, true),
            Cell::new(&issue.title),
            Cell::new(author_name),
            status_cell(&issue.state),
            Cell::new(labels_display),
        ]);
    }

    print_json_or_table(&issues, &table, json_mode)
}

async fn handle_view(
    args: IssueViewArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let issue = client.get_issue(project, args.iid).await?;

    if args.web {
        if let Some(url) = &issue.web_url {
            println!("Opening issue in browser: {url}");
            crate::ui::open_in_browser(url);
            return Ok(());
        }
    }

    if json_mode {
        let json = serde_json::to_string_pretty(&issue)?;
        println!("{json}");
        return Ok(());
    }

    let mut table = create_table(&["Property", "Value"]);
    table.add_row(vec![
        Cell::new("Issue"),
        colored_cell(format!("#{} {}", issue.iid, issue.title), Color::Cyan, true),
    ]);
    table.add_row(vec![Cell::new("Status"), status_cell(&issue.state)]);
    table.add_row(vec![
        Cell::new("Author"),
        Cell::new(
            issue
                .author
                .as_ref()
                .map(|a| a.username.as_str())
                .unwrap_or("-"),
        ),
    ]);
    if !issue.labels.is_empty() {
        table.add_row(vec![Cell::new("Labels"), Cell::new(issue.labels.join(", "))]);
    }
    if let Some(web) = &issue.web_url {
        table.add_row(vec![Cell::new("Web URL"), Cell::new(web)]);
    }
    if let Some(desc) = &issue.description {
        table.add_row(vec![Cell::new("Description"), Cell::new(desc)]);
    }

    println!("{table}");
    Ok(())
}

async fn handle_create(
    args: IssueCreateArgs,
    client: &GitLabClient,
    project: &str,
) -> anyhow::Result<()> {
    let title = match args.title {
        Some(t) => t,
        None => Text::new("Issue Title:").prompt()?,
    };

    let issue = client
        .create_issue(
            project,
            &title,
            args.description.as_deref(),
            args.labels.as_deref(),
        )
        .await?;

    println!(
        "{} Created issue #{} ({})",
        "✓".green().bold(),
        issue.iid,
        issue.title.bold()
    );
    if let Some(url) = &issue.web_url {
        println!("  URL: {url}");
    }

    Ok(())
}

async fn handle_close(
    args: IssueCloseArgs,
    client: &GitLabClient,
    project: &str,
) -> anyhow::Result<()> {
    let issue = client.close_issue(project, args.iid).await?;
    println!(
        "{} Closed issue #{} ({})",
        "✓".green().bold(),
        issue.iid,
        issue.title.bold()
    );
    Ok(())
}

async fn handle_reopen(
    args: IssueReopenArgs,
    client: &GitLabClient,
    project: &str,
) -> anyhow::Result<()> {
    let issue = client.reopen_issue(project, args.iid).await?;
    println!(
        "{} Reopened issue #{} ({})",
        "✓".green().bold(),
        issue.iid,
        issue.title.bold()
    );
    Ok(())
}
