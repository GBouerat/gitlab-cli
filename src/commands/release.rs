//! Command handler for `gitlab release`.

use crate::api::GitLabClient;
use crate::cli::release::{
    ReleaseArgs, ReleaseCommands, ReleaseCreateArgs, ReleaseDeleteArgs, ReleaseListArgs,
    ReleaseViewArgs,
};
use crate::ui::table::{colored_cell, create_table, print_json_or_table};
use colored::Colorize;
use comfy_table::{Cell, Color};

/// Dispatches release subcommands.
///
/// # Arguments
///
/// * `args` - Release command arguments.
/// * `client` - Authenticated GitLab API client.
/// * `project` - Target project path (e.g. `owner/repo`).
/// * `json_mode` - Whether to output raw JSON.
///
/// # Errors
///
/// Returns an error if API communication fails.
pub async fn execute_release(
    args: ReleaseArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    match args.command {
        ReleaseCommands::List(list_args) => {
            handle_list(list_args, client, project, json_mode).await
        }
        ReleaseCommands::View(view_args) => {
            handle_view(view_args, client, project, json_mode).await
        }
        ReleaseCommands::Create(create_args) => handle_create(create_args, client, project).await,
        ReleaseCommands::Delete(delete_args) => handle_delete(delete_args, client, project).await,
    }
}

async fn handle_list(
    args: ReleaseListArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let releases = client.list_releases(project, Some(args.limit)).await?;

    let mut table = create_table(&["Tag", "Name", "Released At", "Author"]);
    for r in &releases {
        let author_name = r
            .author
            .as_ref()
            .map(|a| a.username.as_str())
            .unwrap_or("-");
        let date_str = r.released_at.as_deref().unwrap_or("-");

        table.add_row(vec![
            colored_cell(&r.tag_name, Color::Cyan, true),
            Cell::new(&r.name),
            Cell::new(date_str),
            Cell::new(author_name),
        ]);
    }

    print_json_or_table(&releases, &table, json_mode)
}

async fn handle_view(
    args: ReleaseViewArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let release = client.get_release(project, &args.tag).await?;

    if args.web {
        if let Some(url) = &release.web_url {
            println!("Opening release in browser: {url}");
            crate::ui::open_in_browser(url);
            return Ok(());
        }
    }

    if json_mode {
        let json = serde_json::to_string_pretty(&release)?;
        println!("{json}");
        return Ok(());
    }

    let mut table = create_table(&["Property", "Value"]);
    table.add_row(vec![
        Cell::new("Release"),
        colored_cell(
            format!("{} ({})", release.name, release.tag_name),
            Color::Cyan,
            true,
        ),
    ]);
    table.add_row(vec![Cell::new("Tag"), Cell::new(&release.tag_name)]);
    if let Some(rel_date) = &release.released_at {
        table.add_row(vec![Cell::new("Released At"), Cell::new(rel_date)]);
    }
    if let Some(author) = &release.author {
        table.add_row(vec![Cell::new("Author"), Cell::new(&author.username)]);
    }
    if let Some(commit) = &release.commit {
        table.add_row(vec![
            Cell::new("Commit"),
            Cell::new(format!(
                "{} {}",
                commit.short_id.as_deref().unwrap_or(&commit.id),
                commit.title.as_deref().unwrap_or("")
            )),
        ]);
    }
    if let Some(assets) = &release.assets {
        let links: Vec<String> = assets
            .links
            .iter()
            .map(|l| format!("{}: {}", l.name, l.url))
            .collect();
        if !links.is_empty() {
            table.add_row(vec![Cell::new("Assets"), Cell::new(links.join("\n"))]);
        }
    }
    if let Some(web) = &release.web_url {
        table.add_row(vec![Cell::new("Web URL"), Cell::new(web)]);
    }
    if let Some(desc) = &release.description {
        table.add_row(vec![Cell::new("Description"), Cell::new(desc)]);
    }

    println!("{table}");
    Ok(())
}

async fn handle_create(
    args: ReleaseCreateArgs,
    client: &GitLabClient,
    project: &str,
) -> anyhow::Result<()> {
    let release = client
        .create_release(
            project,
            &args.tag,
            args.name.as_deref(),
            args.description.as_deref(),
            args.ref_name.as_deref(),
        )
        .await?;

    println!(
        "{} Created release {} ({})",
        "✓".green().bold(),
        release.tag_name.cyan().bold(),
        release.name
    );
    if let Some(url) = &release.web_url {
        println!("  URL: {url}");
    }

    Ok(())
}

async fn handle_delete(
    args: ReleaseDeleteArgs,
    client: &GitLabClient,
    project: &str,
) -> anyhow::Result<()> {
    client.delete_release(project, &args.tag).await?;
    println!("{} Deleted release {}", "✓".green().bold(), args.tag.bold());
    Ok(())
}
