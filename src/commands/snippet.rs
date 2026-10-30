//! Command handler for `gitlab snippet`.

use crate::api::GitLabClient;
use crate::cli::snippet::{
    SnippetArgs, SnippetCommands, SnippetCreateArgs, SnippetDeleteArgs, SnippetListArgs,
    SnippetViewArgs,
};
use crate::ui::table::{colored_cell, create_table, print_json_or_table};
use colored::Colorize;
use comfy_table::{Cell, Color};
use inquire::Text;
use std::fs;
use std::io;

/// Dispatches snippet subcommands.
///
/// # Arguments
///
/// * `args` - Snippet command arguments.
/// * `client` - Authenticated GitLab API client.
/// * `json_mode` - Whether to output raw JSON.
///
/// # Errors
///
/// Returns an error if API communication fails.
pub async fn execute_snippet(
    args: SnippetArgs,
    client: &GitLabClient,
    json_mode: bool,
) -> anyhow::Result<()> {
    match args.command {
        SnippetCommands::List(list_args) => handle_list(list_args, client, json_mode).await,
        SnippetCommands::View(view_args) => handle_view(view_args, client, json_mode).await,
        SnippetCommands::Create(create_args) => handle_create(create_args, client).await,
        SnippetCommands::Delete(delete_args) => handle_delete(delete_args, client).await,
    }
}

async fn handle_list(
    args: SnippetListArgs,
    client: &GitLabClient,
    json_mode: bool,
) -> anyhow::Result<()> {
    let snippets = client.list_snippets(Some(args.limit)).await?;

    let mut table = create_table(&["ID", "Title", "File", "Visibility", "Updated At"]);
    for s in &snippets {
        table.add_row(vec![
            colored_cell(s.id.to_string(), Color::Cyan, false),
            Cell::new(&s.title),
            Cell::new(s.file_name.as_deref().unwrap_or("-")),
            colored_cell(&s.visibility, Color::Yellow, false),
            Cell::new(s.updated_at.as_deref().unwrap_or("-")),
        ]);
    }

    print_json_or_table(&snippets, &table, json_mode)
}

async fn handle_view(
    args: SnippetViewArgs,
    client: &GitLabClient,
    json_mode: bool,
) -> anyhow::Result<()> {
    if args.raw {
        let raw = client.get_snippet_raw(args.id).await?;
        print!("{raw}");
        return Ok(());
    }

    let snippet = client.get_snippet(args.id).await?;

    if args.web {
        if let Some(url) = &snippet.web_url {
            println!("Opening snippet in browser: {url}");
            crate::ui::open_in_browser(url);
            return Ok(());
        }
    }

    if json_mode {
        let json = serde_json::to_string_pretty(&snippet)?;
        println!("{json}");
        return Ok(());
    }

    let mut table = create_table(&["Property", "Value"]);
    table.add_row(vec![
        Cell::new("Snippet"),
        colored_cell(
            format!("#{} {}", snippet.id, snippet.title),
            Color::Cyan,
            true,
        ),
    ]);
    if let Some(f) = &snippet.file_name {
        table.add_row(vec![Cell::new("File"), Cell::new(f)]);
    }
    table.add_row(vec![
        Cell::new("Visibility"),
        colored_cell(&snippet.visibility, Color::Yellow, true),
    ]);
    if let Some(author) = &snippet.author {
        table.add_row(vec![Cell::new("Author"), Cell::new(&author.username)]);
    }
    if let Some(web) = &snippet.web_url {
        table.add_row(vec![Cell::new("Web URL"), Cell::new(web)]);
    }
    if let Some(desc) = &snippet.description {
        table.add_row(vec![Cell::new("Description"), Cell::new(desc)]);
    }

    println!("{table}");
    Ok(())
}

async fn handle_create(args: SnippetCreateArgs, client: &GitLabClient) -> anyhow::Result<()> {
    let title = match args.title {
        Some(t) => t,
        None => Text::new("Snippet Title:").prompt()?,
    };

    let file_name = match args.file_name {
        Some(f) => f,
        None => Text::new("File Name (e.g. snippet.txt):").prompt()?,
    };

    let content = match (args.content, args.file) {
        (Some(c), _) => c,
        (None, Some(ref f)) if f == "-" => io::read_to_string(io::stdin())?,
        (None, Some(ref f)) => fs::read_to_string(f)?,
        (None, None) => Text::new("Snippet Content:").prompt()?,
    };

    let snippet = client
        .create_snippet(
            &title,
            &file_name,
            &content,
            &args.visibility,
            args.description.as_deref(),
        )
        .await?;

    println!(
        "{} Created snippet #{} ({})",
        "✓".green().bold(),
        snippet.id,
        snippet.title.bold()
    );
    if let Some(url) = &snippet.web_url {
        println!("  URL: {url}");
    }

    Ok(())
}

async fn handle_delete(args: SnippetDeleteArgs, client: &GitLabClient) -> anyhow::Result<()> {
    client.delete_snippet(args.id).await?;
    println!("{} Deleted snippet #{}", "✓".green().bold(), args.id);
    Ok(())
}
