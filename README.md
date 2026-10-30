# GitLab CLI (`gitlab-cli`)

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

A fast, modular, and ergonomic command-line interface for GitLab written in Rust, leveraging the GitLab REST API (`v4`).

---

## Features

- **Blazing Fast & Self-Contained**: Compiled to a native standalone binary with `rustls` (zero OpenSSL system library dependencies).
- **Context-Aware**: Automatically detects the active GitLab repository, host, and branch from your local `git remote origin`.
- **Multi-Instance Support**: Seamlessly manage credentials for `gitlab.com` alongside self-hosted/enterprise instances.
- **Merge Requests (`gitlab mr`)**: List, view, create, checkout, merge, close, and reopen merge requests directly from the terminal.
- **CI/CD Pipelines & Real-time Logs (`gitlab ci`)**: Inspect pipeline statuses, view job stages, trigger runs, and stream live job logs (`gitlab ci trace`).
- **Issue Tracking (`gitlab issue`)**: View and manage issues with label and state filtering.
- **Project Releases (`gitlab release`)**: List, view, create, and delete releases with release notes and tag associations.
- **Repository Branches (`gitlab branch`)**: Inspect branches, track default and protected states, create new branches, and delete branches.
- **Groups & Members (`gitlab group`)**: Browse namespaces, inspect group hierarchies, and view member permissions.
- **CI/CD Variables (`gitlab variable`)**: Manage project environment variables, with support for masked, protected, and scoped variables.
- **Code Snippets (`gitlab snippet`)**: Create, inspect, view raw contents, and delete personal and project snippets.
- **Direct API Access (`gitlab api`)**: Execute authenticated REST requests with automatic project path encoding (`:id` ➔ `%2F` safe slugs).
- **Adaptive Terminal UI**: Formatted with `comfy-table` with colored status indicators and optional `--json` export for scripting.
- **Shell Completions**: Native auto-completions for `bash` and `zsh`.

---

## Installation

### From Source

```bash
git clone https://github.com/GBouerat/gitlab-cli.git
cd gitlab-cli
cargo install --path .
```

---

## Quick Start

### 1. Authenticate

Log in to GitLab with a Personal Access Token (PAT):

```bash
# Authenticate with GitLab.com
gitlab auth login

# Authenticate with a self-managed instance
gitlab auth login --host gitlab.example.com
```

Check your authentication status:

```bash
gitlab auth status
```

Alternatively, pass tokens via environment variables:

```bash
export GITLAB_TOKEN="glpat-..."
export GITLAB_HOST="gitlab.example.com" # optional, defaults to gitlab.com
```

### 2. Projects & Repositories

```bash
# View current project details (auto-detected from git remote)
gitlab project view

# View a specific project
gitlab project view my-group/my-project

# Open project in default web browser
gitlab project view --web

# List projects accessible to you
gitlab project list
```

### 3. Merge Requests

```bash
# List open merge requests
gitlab mr list

# List merged merge requests
gitlab mr list --state merged

# View details of a merge request
gitlab mr view 42

# Create a merge request interactively (or from current branch)
gitlab mr create --title "Add new feature" --draft

# Check out a merge request branch locally
gitlab mr checkout 42

# Merge an open merge request
gitlab mr merge 42 --squash --remove-source-branch
```

### 4. CI/CD Pipelines & Job Logs

```bash
# List recent pipelines
gitlab ci list

# View jobs and status in a pipeline
gitlab ci view 123456

# Stream real-time logs from a CI job
gitlab ci trace 987654

# Trigger a pipeline on current branch
gitlab ci run

# Retry failed jobs in a pipeline
gitlab ci retry 123456
```

### 5. Issues

```bash
# List open issues
gitlab issue list

# Filter issues by label
gitlab issue list --labels "bug,critical"

# Create a new issue
gitlab issue create --title "Parser panic on invalid token"
```

### 6. Releases

```bash
# List project releases
gitlab release list

# View release details
gitlab release view v1.0.0

# Create a new release
gitlab release create v1.1.0 --name "v1.1.0 Feature Release" --ref main --description "Notes..."

# Delete a release
gitlab release delete v1.1.0
```

### 7. Branches

```bash
# List repository branches
gitlab branch list

# View branch details
gitlab branch view feature/awesome

# Create a new branch
gitlab branch create feature/awesome --ref main

# Delete a branch
gitlab branch delete feature/awesome
```

### 8. Groups & Members

```bash
# List accessible groups
gitlab group list

# Search for a specific group
gitlab group list --search platform

# View group metadata
gitlab group view core-platform

# List group members and their access levels
gitlab group members core-platform
```

### 9. CI/CD Variables

```bash
# List project variables
gitlab variable list

# Inspect a specific variable
gitlab variable get DATABASE_URL

# Set or update a variable
gitlab variable set DATABASE_URL --value "postgres://localhost/db" --masked --protected

# Delete a variable
gitlab variable delete DATABASE_URL
```

### 10. Snippets

```bash
# List snippets
gitlab snippet list

# View snippet metadata
gitlab snippet view 42

# View raw snippet contents
gitlab snippet view 42 --raw

# Create a snippet from file or interactive input
gitlab snippet create --title "Quick script" --file script.sh --content "echo hello"

# Delete a snippet
gitlab snippet delete 42
```

### 11. Raw API Querying

```bash
# Fetch current user details
gitlab api user

# Query a project endpoint (automatically substitutes :id from current repository)
gitlab api projects/:id/pipelines
```

### 12. Shell Completions

Generate completions for your shell:

```bash
# Bash
gitlab completion bash > ~/.local/share/bash-completion/completions/gitlab

# Zsh
gitlab completion zsh > ~/.zfunc/_gitlab
```

---

## Documentation

Full architectural documentation is available in [ARCHITECTURE.md](ARCHITECTURE.md).

To build and view local Rustdoc documentation:

```bash
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --open
```

---

## License

Dual-licensed under either:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

---

*Generated entirely by Gemini through Antigravity.*
