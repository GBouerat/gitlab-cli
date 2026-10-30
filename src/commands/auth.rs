//! Command handler for `gitlab auth`.

use crate::api::GitLabClient;
use crate::cli::auth::{AuthArgs, AuthCommands, LoginArgs, LogoutArgs, StatusArgs};
use crate::config::{Config, DEFAULT_GITLAB_HOST};
use colored::Colorize;
use inquire::Password;

/// Dispatches authentication subcommands.
///
/// # Arguments
///
/// * `args` - Authentication command arguments.
/// * `config` - Stored configuration reference.
///
/// # Errors
///
/// Returns an error if login, status check, or logout operations fail.
pub async fn execute_auth(args: AuthArgs, config: &mut Config) -> anyhow::Result<()> {
    match args.command {
        AuthCommands::Login(login_args) => handle_login(login_args, config).await,
        AuthCommands::Status(status_args) => handle_status(status_args, config).await,
        AuthCommands::Logout(logout_args) => handle_logout(logout_args, config).await,
    }
}

async fn handle_login(args: LoginArgs, config: &mut Config) -> anyhow::Result<()> {
    let host = args.host.unwrap_or_else(|| {
        if config.default_host.is_empty() {
            DEFAULT_GITLAB_HOST.to_string()
        } else {
            config.default_host.clone()
        }
    });

    let token = match args.token {
        Some(t) => t,
        None => {
            println!(
                "{}",
                format!("Authenticating with GitLab instance: {host}").cyan()
            );
            Password::new("Enter your GitLab Personal Access Token:")
                .without_confirmation()
                .prompt()?
        }
    };

    let trimmed_token = token.trim().to_string();
    if trimmed_token.is_empty() {
        anyhow::bail!("Access token cannot be empty");
    }

    // Verify token by querying current user
    let client = GitLabClient::new(&host, &trimmed_token)?;
    let user = client.current_user().await?;

    config.set_host_token(&host, &trimmed_token, Some(user.username.clone()));
    config.save()?;

    println!(
        "{} Authenticated with {} as {} ({})",
        "✓".green().bold(),
        host.bold(),
        user.username.cyan().bold(),
        user.name
    );

    Ok(())
}

async fn handle_status(args: StatusArgs, config: &Config) -> anyhow::Result<()> {
    let host = args.host.unwrap_or_else(|| {
        if config.default_host.is_empty() {
            DEFAULT_GITLAB_HOST.to_string()
        } else {
            config.default_host.clone()
        }
    });

    match config.resolve_auth(Some(&host), None) {
        Ok((resolved_host, token)) => {
            let client = GitLabClient::new(&resolved_host, &token)?;
            match client.current_user().await {
                Ok(user) => {
                    let masked_token = if token.len() > 8 {
                        format!("{}...{}", &token[..4], &token[token.len() - 4..])
                    } else {
                        "***".to_string()
                    };

                    println!(
                        "{} Authenticated with GitLab host: {}",
                        "✓".green().bold(),
                        resolved_host.bold()
                    );
                    println!("  Username: {}", user.username.cyan().bold());
                    println!("  Name:     {}", user.name);
                    println!("  Token:    {}", masked_token);
                }
                Err(err) => {
                    println!(
                        "{} Token configured for host '{}' failed validation: {}",
                        "✗".red().bold(),
                        resolved_host,
                        err
                    );
                }
            }
        }
        Err(_) => {
            println!(
                "{} Not logged in to GitLab host: {}",
                "✗".yellow().bold(),
                host
            );
            println!("  Run `gitlab auth login --host {}` to authenticate.", host);
        }
    }

    Ok(())
}

async fn handle_logout(args: LogoutArgs, config: &mut Config) -> anyhow::Result<()> {
    let host = args.host.unwrap_or_else(|| {
        if config.default_host.is_empty() {
            DEFAULT_GITLAB_HOST.to_string()
        } else {
            config.default_host.clone()
        }
    });

    if config.remove_host(&host) {
        config.save()?;
        println!(
            "{} Logged out from host: {}",
            "✓".green().bold(),
            host.bold()
        );
    } else {
        println!(
            "{} No stored credentials found for host: {}",
            "!".yellow(),
            host
        );
    }

    Ok(())
}
