//! Entry point for the `gitlab` command-line application.

use clap::Parser;
use colored::Colorize;
use gitlab_cli::cli::Cli;
use gitlab_cli::commands::run_command;
use gitlab_cli::config::Config;
use std::process::ExitCode;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    // Configure logging verbosity based on -v flags
    let filter = match cli.verbose {
        0 => LevelFilter::WARN,
        1 => LevelFilter::DEBUG,
        _ => LevelFilter::TRACE,
    };

    let env_filter = EnvFilter::builder()
        .with_default_directive(filter.into())
        .from_env_lossy();

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .without_time()
        .init();

    let config = match Config::load() {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("{}: Failed to load configuration: {err}", "error".red().bold());
            return ExitCode::FAILURE;
        }
    };

    if let Err(err) = run_command(cli, config).await {
        eprintln!("{}: {err}", "error".red().bold());
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
