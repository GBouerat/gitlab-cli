# GitLab CLI (`gitlab-cli`)

## Project Overview
A fast, modular, and ergonomic command-line interface for GitLab written in Rust, leveraging the GitLab REST API (`v4`).

## Repository Architecture & Guidelines
- **Binary Name**: `gitlab` (package name: `gitlab-cli`)
- **Repository URL**: `https://github.com/GBouerat/gitlab-cli`
- **Current Branch**: `dev`
- **Documentation**:
  - Full Rustdoc coverage (`//!` module docstrings and `///` item docstrings across all files).
  - Strict documentation checks: `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps`.
  - Maintain `ARCHITECTURE.md` alongside `README.md`.
  - Append `*Generated entirely by Gemini through Antigravity.*` at the end of `README.md`.
- **Terminal Tables**:
  - Format tables using `comfy-table`.
  - Do not embed raw ANSI escape sequences inside cell strings; use `Cell::new(...).fg(Color::...)` and `Attribute::Bold`.
- **Shell Completions**:
  - Support `bash` and `zsh` completions.
- **Testing Standards**:
  - Unit and integration tests covering edge cases and CLI argument combinations.
  - Async HTTP mock tests via `wiremock` for GitLab API client verification.
  - Mutex-guarded `ENV_LOCK` for tests interacting with environment variables.
