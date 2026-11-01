//! Command handler for `gitlab markdown`.

use crate::api::GitLabClient;
use crate::cli::markdown::{MarkdownArgs, MarkdownCommands, MarkdownRenderArgs};
use crate::commands::body::read_required_body;
use colored::Colorize;
use serde::Serialize;

/// Summary of task-list checkbox occurrences.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckboxCount {
    /// Number of checked task-list items.
    pub checked: usize,
    /// Number of unchecked task-list items.
    pub unchecked: usize,
    /// Total number of task-list items.
    pub total: usize,
}

/// Counts task-list checkboxes (`[ ]`, `[x]`, `[X]`) in Markdown text.
///
/// Accurately parses task-list items and markdown table checkboxes while skipping
/// code blocks (fenced with ``` or ~~~), inline code spans (`...`), and normal
/// bracketed identifiers like `arr[x]`.
pub fn count_checkboxes(text: &str) -> CheckboxCount {
    let mut checked = 0;
    let mut unchecked = 0;
    let mut in_code_block = false;

    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }

        let bytes = line.as_bytes();
        let mut i = 0;
        let mut in_inline_code = false;

        while i < bytes.len() {
            if bytes[i] == b'`' {
                in_inline_code = !in_inline_code;
                i += 1;
                continue;
            }

            if !in_inline_code && i + 2 < bytes.len() && bytes[i] == b'[' {
                let is_unchecked = bytes[i + 1] == b' ' && bytes[i + 2] == b']';
                let is_checked =
                    (bytes[i + 1] == b'x' || bytes[i + 1] == b'X') && bytes[i + 2] == b']';

                if is_unchecked || is_checked {
                    // Preceding character must be start of line, whitespace, or list/table marker
                    let valid_prev = if i == 0 {
                        true
                    } else {
                        let prev = bytes[i - 1];
                        prev.is_ascii_whitespace()
                            || prev == b'-'
                            || prev == b'*'
                            || prev == b'+'
                            || prev == b'|'
                    };

                    // Following character must be end of line, whitespace, or punctuation/table separator
                    let valid_next = if i + 3 >= bytes.len() {
                        true
                    } else {
                        let next = bytes[i + 3];
                        next.is_ascii_whitespace()
                            || next == b'|'
                            || next == b':'
                            || next == b','
                            || next == b'.'
                    };

                    if valid_prev && valid_next {
                        if is_unchecked {
                            unchecked += 1;
                        } else {
                            checked += 1;
                        }
                        i += 3;
                        continue;
                    }
                }
            }
            i += 1;
        }
    }

    CheckboxCount {
        checked,
        unchecked,
        total: checked + unchecked,
    }
}

/// Dispatches markdown subcommands.
///
/// # Arguments
///
/// * `args` - Markdown command arguments.
/// * `client` - Authenticated GitLab API client.
/// * `project` - Optional project path for contextual rendering.
/// * `json_mode` - Whether to output raw JSON.
///
/// # Errors
///
/// Returns an error if reading body or API communication fails.
pub async fn execute_markdown(
    args: MarkdownArgs,
    client: &GitLabClient,
    project: Option<&str>,
    json_mode: bool,
) -> anyhow::Result<()> {
    match args.command {
        MarkdownCommands::Render(render_args) => {
            handle_render(render_args, client, project, json_mode).await
        }
    }
}

async fn handle_render(
    args: MarkdownRenderArgs,
    client: &GitLabClient,
    project: Option<&str>,
    json_mode: bool,
) -> anyhow::Result<()> {
    let body = read_required_body(
        args.body.as_deref(),
        args.file.as_deref(),
        "Enter Markdown to render:",
    )?;

    let checkbox_count = count_checkboxes(&body);

    let project_context = if args.project { project } else { None };

    let rendered = client.render_markdown(&body, true, project_context).await?;

    if json_mode {
        let output = serde_json::json!({
            "html": rendered.html,
            "task_list": checkbox_count,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    // Print HTML to stdout
    println!("{}", rendered.html);

    // Report checkbox counts to stderr
    eprintln!(
        "Task-list checkboxes: {}/{} completed ({} checked, {} unchecked)",
        checkbox_count.checked.to_string().green(),
        checkbox_count.total.to_string().cyan(),
        checkbox_count.checked,
        checkbox_count.unchecked
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_checkboxes_markdown() {
        let text = r#"
# Tasks
- [ ] Task 1
- [x] Task 2
- [X] Task 3
- Regular bullet

| Name | Status |
|---|---|
| Item A | [ ] Pending |
| Item B | [x] Completed |
"#;
        let counts = count_checkboxes(text);
        assert_eq!(counts.checked, 3);
        assert_eq!(counts.unchecked, 2);
        assert_eq!(counts.total, 5);
    }

    #[test]
    fn test_count_checkboxes_ignores_code_and_brackets() {
        let text = r#"
# Documentation
Here is an inline code snippet: `[x]` and `[ ]`.
Here is array indexing: array[x] and matrix[ ].

```rust
fn example() {
    let list = [x];
    let empty = [ ];
}
```

Real task list:
* [x] Documented
* [ ] Tested
"#;
        let counts = count_checkboxes(text);
        assert_eq!(counts.checked, 1);
        assert_eq!(counts.unchecked, 1);
        assert_eq!(counts.total, 2);
    }
}
