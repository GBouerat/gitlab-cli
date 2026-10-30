//! Command handler for `gitlab epic`.

use crate::api::GitLabClient;
use crate::cli::epic::{
    EpicArgs, EpicCloseArgs, EpicCommands, EpicCreateArgs, EpicListArgs, EpicViewArgs,
};
use crate::ui::open_in_browser;
use crate::ui::table::{colored_cell, create_table, print_json_or_table};
use colored::Colorize;
use comfy_table::{Cell, Color};

/// Dispatches epic subcommands.
///
/// # Arguments
///
/// * `args` - Epic command arguments.
/// * `client` - Authenticated GitLab API client.
/// * `json_mode` - Whether to output raw JSON.
///
/// # Errors
///
/// Returns an error if API communication fails.
pub async fn execute_epic(
    args: EpicArgs,
    client: &GitLabClient,
    json_mode: bool,
) -> anyhow::Result<()> {
    match args.command {
        EpicCommands::List(list_args) => handle_list(list_args, client, json_mode).await,
        EpicCommands::View(view_args) => handle_view(view_args, client, json_mode).await,
        EpicCommands::Create(create_args) => handle_create(create_args, client).await,
        EpicCommands::Close(close_args) => handle_close(close_args, client).await,
    }
}

async fn handle_list(
    args: EpicListArgs,
    client: &GitLabClient,
    json_mode: bool,
) -> anyhow::Result<()> {
    let epics = client
        .list_epics(&args.group, args.state.as_deref(), Some(args.limit))
        .await?;

    let mut table = create_table(&["IID", "Title", "State", "Author"]);
    for e in &epics {
        let state_cell = match e.state.as_deref() {
            Some("opened") => colored_cell("opened", Color::Green, false),
            Some("closed") => colored_cell("closed", Color::DarkGrey, false),
            Some(other) => Cell::new(other),
            None => Cell::new("-"),
        };
        let author_str = e
            .author
            .as_ref()
            .map(|u| format!("@{}", u.username))
            .unwrap_or_else(|| "-".to_string());

        table.add_row(vec![
            colored_cell(format!("&{}", e.iid), Color::Cyan, true),
            Cell::new(&e.title),
            state_cell,
            Cell::new(author_str),
        ]);
    }

    print_json_or_table(&epics, &table, json_mode)
}

async fn handle_view(
    args: EpicViewArgs,
    client: &GitLabClient,
    json_mode: bool,
) -> anyhow::Result<()> {
    let epic = client.get_epic(&args.group, args.iid).await?;

    if args.web {
        if let Some(web_url) = &epic.web_url {
            open_in_browser(web_url);
            return Ok(());
        }
    }

    let mut table = create_table(&["Property", "Value"]);
    table.add_row(vec![
        Cell::new("Epic"),
        colored_cell(format!("&{} {}", epic.iid, epic.title), Color::Cyan, true),
    ]);
    table.add_row(vec![
        Cell::new("State"),
        colored_cell(
            epic.state.as_deref().unwrap_or("-"),
            if epic.state.as_deref() == Some("opened") {
                Color::Green
            } else {
                Color::DarkGrey
            },
            true,
        ),
    ]);
    if let Some(a) = &epic.author {
        table.add_row(vec![
            Cell::new("Author"),
            Cell::new(format!("{} (@{})", a.name, a.username)),
        ]);
    }
    if let Some(desc) = &epic.description {
        table.add_row(vec![Cell::new("Description"), Cell::new(desc)]);
    }

    print_json_or_table(&epic, &table, json_mode)
}

async fn handle_create(args: EpicCreateArgs, client: &GitLabClient) -> anyhow::Result<()> {
    let epic = client
        .create_epic(&args.group, &args.title, args.description.as_deref())
        .await?;

    println!(
        "{} Created epic &{} ({})",
        "✓".green().bold(),
        epic.iid,
        epic.title.bold()
    );
    Ok(())
}

async fn handle_close(args: EpicCloseArgs, client: &GitLabClient) -> anyhow::Result<()> {
    let epic = client.close_epic(&args.group, args.iid).await?;
    println!(
        "{} Closed epic &{} ({})",
        "✓".green().bold(),
        epic.iid,
        epic.title.bold()
    );
    Ok(())
}
