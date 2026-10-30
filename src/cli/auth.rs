//! CLI definitions for authentication and token management.

use clap::{Args, Subcommand};

/// Top-level arguments for `gitlab auth`.
#[derive(Args, Debug)]
pub struct AuthArgs {
    /// Authentication subcommands.
    #[command(subcommand)]
    pub command: AuthCommands,
}

/// Available authentication subcommands.
#[derive(Subcommand, Debug)]
pub enum AuthCommands {
    /// Authenticate with a GitLab instance and store credentials.
    Login(LoginArgs),
    /// Display current authentication status and active user.
    Status(StatusArgs),
    /// Remove stored credentials for a GitLab host.
    Logout(LogoutArgs),
}

/// Arguments for `gitlab auth login`.
#[derive(Args, Debug)]
pub struct LoginArgs {
    /// Target GitLab host (e.g. `gitlab.com` or `gitlab.example.com`).
    #[arg(short = 'H', long)]
    pub host: Option<String>,

    /// Personal access token (prompted securely if omitted).
    #[arg(short = 't', long)]
    pub token: Option<String>,
}

/// Arguments for `gitlab auth status`.
#[derive(Args, Debug)]
pub struct StatusArgs {
    /// Target GitLab host to check (defaults to active host).
    #[arg(short = 'H', long)]
    pub host: Option<String>,
}

/// Arguments for `gitlab auth logout`.
#[derive(Args, Debug)]
pub struct LogoutArgs {
    /// Target GitLab host to remove credentials for.
    #[arg(short = 'H', long)]
    pub host: Option<String>,
}
