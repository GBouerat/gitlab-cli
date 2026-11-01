# GitLab CLI (`gitlab-cli`)

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

A fast, modular, and ergonomic command-line interface for GitLab written in Rust, leveraging the GitLab REST API (`v4`).

---

## Features

- **Blazing Fast & Self-Contained**: Compiled to a native standalone binary with `rustls` (zero OpenSSL system library dependencies).
- **Context-Aware**: Automatically detects the active GitLab repository, host, and branch from your local `git remote origin`.
- **Multi-Instance Support**: Seamlessly manage credentials for `gitlab.com` alongside self-hosted/enterprise instances.
- **Merge Requests (`gitlab mr`)**: List, view, create, checkout, merge, close, reopen, approve, and comment on merge requests.
- **CI/CD Pipelines & Real-time Logs (`gitlab ci`)**: Inspect pipeline statuses, view job stages, trigger runs, and stream live job logs (`gitlab ci trace`).
- **Issue Tracking (`gitlab issue`)**: View and manage issues with label and state filtering, and add discussion comments.
- **Project Releases (`gitlab release`)**: List, view, create, and delete releases with release notes and tag associations.
- **Repository Branches & Tags (`gitlab branch`, `gitlab tag`)**: Manage branches and Git tags, protection rules, and commit refs.
- **Environments & Deployments (`gitlab env`, `gitlab deploy`)**: Manage deployment environments and inspect deployment statuses.
- **Milestones & Epics (`gitlab milestone`, `gitlab epic`)**: Track delivery roadmap progress across projects and group epics.
- **Packages & Container Registry (`gitlab package`, `gitlab registry`)**: Inspect packages and container images/tags.
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

### 3. Merge Requests & Review Workflow

```bash
# List open merge requests
gitlab mr list

# List merge requests for a specific source branch
gitlab mr list --source-branch feature-branch

# View open merge request of current branch (or find by branch name/IID)
gitlab mr view
gitlab mr find feature-branch
gitlab mr view 42

# Create a merge request with reviewers, assignees, and labels
gitlab mr create --title "Add new feature" -f description.md --reviewer alice --assignee @me --label feature --remove-source-branch

# View changed files or unified diff
gitlab mr diff 42
gitlab mr diff 42 --patch

# Inspect review discussions and threads
gitlab mr discussions 42
gitlab mr discussions 42 --unresolved
gitlab mr discussions 42 --full

# Start a new discussion thread or inline diff review comment
gitlab mr comment 42 -f review.md
gitlab mr comment 42 -f review.md --path src/main.rs --line 42 --prefix "Suggestion:"
cat review.md | gitlab mr comment 42 -f -

# Reply to or resolve a discussion thread
gitlab mr reply 42 thread-abc-123 -b "Addressed in latest commit"
gitlab mr resolve 42 thread-abc-123
gitlab mr resolve 42 thread-abc-123 --undo

# Edit an existing note in place
gitlab mr note-edit 42 12345 -f updated-note.md --prefix "[EDIT]"

# Update merge request metadata, labels, reviewers, or draft status
gitlab mr update 42 --ready --add-label approved --reviewer bob
gitlab mr update 42 --draft --remove-label approved

# Relink rebased commit SHAs in note or description to current commits
gitlab mr relink 42 --dry-run
gitlab mr relink 42
gitlab mr relink 42 12345

# Check out a merge request branch locally
gitlab mr checkout 42

# Approve a merge request
gitlab mr approve 42

# View approval status and reviewers
gitlab mr approvals 42

# Merge an open merge request
gitlab mr merge 42 --squash --remove-source-branch
```

### 4. CI/CD Pipelines & Job Traces

```bash
# List pipelines for HEAD commit of current repo (includes failed jobs breakdown)
gitlab ci list

# List pipelines for a specific ref or commit SHA
gitlab ci list --ref main
gitlab ci list --sha 11223344

# View jobs and status in a pipeline
gitlab ci view 123456

# Print job trace log with optional tailing and ANSI/section stripping
gitlab ci trace 987654
gitlab ci trace 987654 --tail 50 --no-color

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

# Create a new issue (inline description or from file/stdin)
gitlab issue create --title "Parser panic on invalid token" -f bug-report.md
cat report.md | gitlab issue create --title "Regression" -f -

# Add a comment to an issue (inline body or from file/stdin)
gitlab issue note 15 --body "Looking into this."
gitlab issue note 15 -f investigation.md

# List comments on an issue
gitlab issue notes 15
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

### 7. Branches & Tags

```bash
# List repository branches
gitlab branch list

# View branch details
gitlab branch view feature/awesome

# Create a new branch
gitlab branch create feature/awesome --ref main

# Delete a branch
gitlab branch delete feature/awesome

# List repository tags
gitlab tag list

# View tag details
gitlab tag view v1.0.0

# Create a new tag
gitlab tag create v1.0.0 --ref main --message "Release v1.0"

# Delete a tag
gitlab tag delete v1.0.0
```

### 8. Environments & Deployments

```bash
# List project environments
gitlab env list

# View environment details
gitlab env view 1

# Create an environment
gitlab env create staging --url https://staging.example.com

# Stop an environment
gitlab env stop 1

# List deployments
gitlab deploy list

# View deployment details
gitlab deploy view 100
```

### 9. Milestones & Epics

```bash
# List project milestones
gitlab milestone list

# List group milestones
gitlab milestone list --group my-group

# View milestone details
gitlab milestone view 10

# Create a milestone
gitlab milestone create "v1.0 Release" --due-date "2026-12-31"

# Close a milestone
gitlab milestone close 10

# List group epics
gitlab epic list --group my-group

# View epic details
gitlab epic view 1 --group my-group

# Create a group epic
gitlab epic create "Core Modernization" --group my-group
```

### 10. Packages & Container Registry

```bash
# List project packages
gitlab package list

# View package details
gitlab package view 42

# Delete a package
gitlab package delete 42

# List container repositories
gitlab registry list

# List tags for a container repository
gitlab registry tags 2

# Delete a container image tag
gitlab registry delete-tag 2 latest
```

### 11. Groups & Members

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

### 12. CI/CD Variables

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

### 13. Snippets

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

### 14. Raw API Querying

```bash
# Fetch current user details
gitlab api user

# Query a project endpoint (automatically substitutes :id from current repository)
gitlab api projects/:id/pipelines
```

### 15. Markdown Rendering (`gitlab markdown`)

Render GitLab Flavored Markdown (GFM) using GitLab's backend rendering engine and inspect task-list checkbox completion:

```bash
# Render inline markdown
gitlab markdown render -b "# Title with task\n- [x] Done\n- [ ] Pending"

# Render from a file with project context
gitlab markdown render -f README.md --project

# Render from stdin
cat notes.md | gitlab markdown render -f -
```

### 16. Repository File Web URL (`gitlab file`)

Fetch and print the raw content of any file directly using its web URL:

```bash
# Print a repository file from its GitLab web URL
gitlab file https://gitlab.example.com/group/subgroup/project/-/blob/main/src/main.rs
```

### 17. Shell Completions

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
