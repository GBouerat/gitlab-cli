//! # GitLab CLI (`gitlab-cli`)
//!
//! A fast, modular, and ergonomic command-line interface for GitLab written in Rust,
//! leveraging the GitLab REST API (`v4`).
//!
//! ## Modules
//! - [`api`]: Strongly-typed asynchronous HTTP client for GitLab REST API v4.
//! - [`cli`]: Clap-based CLI structure and command-line arguments.
//! - [`commands`]: Subcommand implementations for auth, projects, merge requests, issues, CI/CD, releases, branches, groups, variables, snippets, and completions.
//! - [`config`]: Multi-host configuration and credential management (`~/.config/gitlab-cli/config.json`).
//! - [`git`]: Local repository inspection, branch detection, and remote URL parsing.
//! - [`models`]: Data models for GitLab entities (Projects, MRs, Issues, Pipelines, Jobs, Releases, Branches, Groups, Variables, Snippets, Users).
//! - [`ui`]: Terminal styling, table formatting, and cross-platform browser helpers.

pub mod api;
pub mod cli;
pub mod commands;
pub mod config;
pub mod git;
pub mod models;
pub mod ui;

pub use api::client::GitLabClient;
pub use config::Config;
