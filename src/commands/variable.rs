//! Command handler for `gitlab variable`.

use crate::api::GitLabClient;
use crate::cli::variable::{
    VariableArgs, VariableCommands, VariableDeleteArgs, VariableGetArgs, VariableListArgs,
    VariableSetArgs,
};
use crate::ui::table::{colored_cell, create_table, print_json_or_table};
use colored::Colorize;
use comfy_table::{Cell, Color};
use inquire::Password;

/// Dispatches CI/CD variable subcommands.
///
/// # Arguments
///
/// * `args` - Variable command arguments.
/// * `client` - Authenticated GitLab API client.
/// * `project` - Target project path (e.g. `owner/repo`).
/// * `json_mode` - Whether to output raw JSON.
///
/// # Errors
///
/// Returns an error if API communication fails.
pub async fn execute_variable(
    args: VariableArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    match args.command {
        VariableCommands::List(list_args) => {
            handle_list(list_args, client, project, json_mode).await
        }
        VariableCommands::Get(get_args) => handle_get(get_args, client, project, json_mode).await,
        VariableCommands::Set(set_args) => handle_set(set_args, client, project).await,
        VariableCommands::Delete(delete_args) => handle_delete(delete_args, client, project).await,
    }
}

async fn handle_list(
    args: VariableListArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let variables = client.list_variables(project, Some(args.limit)).await?;

    let mut table = create_table(&["Key", "Type", "Masked", "Protected", "Scope"]);
    for v in &variables {
        let is_masked = if v.masked.unwrap_or(false) {
            colored_cell("yes", Color::Yellow, false)
        } else {
            Cell::new("no")
        };

        let is_protected = if v.protected.unwrap_or(false) {
            colored_cell("yes", Color::Green, false)
        } else {
            Cell::new("no")
        };

        table.add_row(vec![
            colored_cell(&v.key, Color::Cyan, true),
            Cell::new(&v.variable_type),
            is_masked,
            is_protected,
            Cell::new(&v.environment_scope),
        ]);
    }

    print_json_or_table(&variables, &table, json_mode)
}

async fn handle_get(
    args: VariableGetArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let variable = client.get_variable(project, &args.key).await?;

    if json_mode {
        let json = serde_json::to_string_pretty(&variable)?;
        println!("{json}");
        return Ok(());
    }

    let mut table = create_table(&["Property", "Value"]);
    table.add_row(vec![
        Cell::new("Key"),
        colored_cell(&variable.key, Color::Cyan, true),
    ]);
    table.add_row(vec![
        Cell::new("Value"),
        if variable.masked.unwrap_or(false) {
            Cell::new("****** (masked)")
        } else {
            Cell::new(&variable.value)
        },
    ]);
    table.add_row(vec![Cell::new("Type"), Cell::new(&variable.variable_type)]);
    table.add_row(vec![
        Cell::new("Masked"),
        Cell::new(if variable.masked.unwrap_or(false) {
            "yes"
        } else {
            "no"
        }),
    ]);
    table.add_row(vec![
        Cell::new("Protected"),
        Cell::new(if variable.protected.unwrap_or(false) {
            "yes"
        } else {
            "no"
        }),
    ]);
    table.add_row(vec![
        Cell::new("Environment Scope"),
        Cell::new(&variable.environment_scope),
    ]);
    if let Some(desc) = &variable.description {
        table.add_row(vec![Cell::new("Description"), Cell::new(desc)]);
    }

    println!("{table}");
    Ok(())
}

async fn handle_set(
    args: VariableSetArgs,
    client: &GitLabClient,
    project: &str,
) -> anyhow::Result<()> {
    let value = match args.value {
        Some(v) => v,
        None => Password::new(&format!("Enter value for variable '{}':", args.key))
            .without_confirmation()
            .prompt()?,
    };

    let variable = client
        .set_variable(
            project,
            &args.key,
            &value,
            args.masked,
            args.protected,
            Some(&args.environment_scope),
        )
        .await?;

    println!(
        "{} Set variable {} (Scope: '{}')",
        "✓".green().bold(),
        variable.key.cyan().bold(),
        variable.environment_scope
    );
    Ok(())
}

async fn handle_delete(
    args: VariableDeleteArgs,
    client: &GitLabClient,
    project: &str,
) -> anyhow::Result<()> {
    client.delete_variable(project, &args.key).await?;
    println!(
        "{} Deleted variable {}",
        "✓".green().bold(),
        args.key.bold()
    );
    Ok(())
}
