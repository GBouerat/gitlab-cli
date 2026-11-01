//! Helper utilities for reading markdown bodies from inline flags, files, or standard input.

use std::fs;
use std::io::{self, Read};

/// Resolves a body string from either an inline string or a file path (where `"-"` indicates stdin).
///
/// Preserves all characters, backticks, dollar signs, quotes, and pipes byte for byte.
///
/// # Arguments
///
/// * `inline` - Optional inline string passed via CLI flag (e.g. `-b` or `-d`).
/// * `file` - Optional file path or `"-"` for stdin (e.g. `-f` or `--file`).
///
/// # Errors
///
/// Returns an error if reading from the file or standard input fails.
pub fn read_body(inline: Option<&str>, file: Option<&str>) -> anyhow::Result<Option<String>> {
    if let Some(path) = file {
        if path == "-" {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok(Some(buffer))
        } else {
            let content = fs::read_to_string(path)
                .map_err(|e| anyhow::anyhow!("Failed to read file '{path}': {e}"))?;
            Ok(Some(content))
        }
    } else {
        Ok(inline.map(|s| s.to_string()))
    }
}

/// Reads a required body string from either an inline string or a file path.
///
/// If neither is supplied, prompts interactively if a TTY is available, or returns an error.
///
/// # Arguments
///
/// * `inline` - Optional inline string passed via CLI flag.
/// * `file` - Optional file path or `"-"` for stdin.
/// * `prompt_message` - Prompt to show in an interactive editor.
///
/// # Errors
///
/// Returns an error if no body is provided in non-interactive environments or reading fails.
pub fn read_required_body(
    inline: Option<&str>,
    file: Option<&str>,
    prompt_message: &str,
) -> anyhow::Result<String> {
    if let Some(body) = read_body(inline, file)? {
        if !body.is_empty() {
            return Ok(body);
        }
    }

    use std::io::IsTerminal;
    if std::io::stdin().is_terminal() {
        let text = inquire::Text::new(prompt_message).prompt()?;
        if text.trim().is_empty() {
            anyhow::bail!("Body cannot be empty");
        }
        Ok(text)
    } else {
        anyhow::bail!(
            "A body is required. Use `-f <file>` (or `-f -` for stdin) or inline `-b <text>`."
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_body_inline() {
        let inline = Some("inline `$VAR` 'quotes' and |pipes|");
        let res = read_body(inline, None).unwrap();
        assert_eq!(res.as_deref(), inline);
    }

    #[test]
    fn test_read_body_file() {
        let mut tmp = NamedTempFile::new().unwrap();
        let content = "Text with `code`, $FOO, \"double\" and |pipes|.\nMulti-line.";
        tmp.write_all(content.as_bytes()).unwrap();

        let res = read_body(None, Some(tmp.path().to_str().unwrap())).unwrap();
        assert_eq!(res.as_deref(), Some(content));
    }
}
