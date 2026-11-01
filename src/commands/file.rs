//! Command handler for `gitlab file`.

use crate::api::GitLabClient;
use crate::cli::file::FileArgs;
use crate::config::Config;
use url::Url;

/// Parsed components of a GitLab repository blob URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFileUrl {
    /// Hostname of the GitLab instance.
    pub host: String,
    /// Namespaced project path (e.g. `group/subgroup/project`).
    pub project_path: String,
    /// Git reference (branch name, tag, or commit SHA).
    pub ref_name: String,
    /// Relative file path within the repository.
    pub file_path: String,
    /// Raw path segment following the `/-/blob/` separator.
    pub after_blob: String,
}

impl ParsedFileUrl {
    /// Returns all candidate `(ref, file_path)` pairs for URLs where the branch name may contain slashes.
    pub fn candidate_splits(&self) -> Vec<(String, String)> {
        let parts: Vec<&str> = self.after_blob.split('/').collect();
        if parts.len() < 2 {
            return vec![(self.ref_name.clone(), self.file_path.clone())];
        }

        let mut splits = Vec::new();
        // First try the default split (single segment ref)
        splits.push((self.ref_name.clone(), self.file_path.clone()));

        // Then try multi-segment refs
        for i in 2..parts.len() {
            let ref_part = parts[..i].join("/");
            let path_part = parts[i..].join("/");
            if ref_part != self.ref_name {
                splits.push((ref_part, path_part));
            }
        }
        splits
    }
}

/// Parses a GitLab web URL into its host, project path, ref, and file path.
///
/// Expected format: `https://<host>/<project>/-/blob/<ref>/<path>`
///
/// # Arguments
///
/// * `raw_url` - The full web URL string.
///
/// # Errors
///
/// Returns an error if the URL is malformed or does not contain `/-/blob/`.
pub fn parse_file_url(raw_url: &str) -> anyhow::Result<ParsedFileUrl> {
    let parsed =
        Url::parse(raw_url.trim()).map_err(|e| anyhow::anyhow!("Invalid URL '{raw_url}': {e}"))?;

    let host = parsed
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("URL '{raw_url}' has no hostname"))?
        .to_string();

    let path = parsed.path().trim_start_matches('/');
    let marker = "/-/blob/";
    let blob_idx = path
        .find(marker)
        .ok_or_else(|| anyhow::anyhow!("URL does not contain '/-/blob/' separator: '{raw_url}'"))?;

    let project_path = path[..blob_idx].to_string();
    let after_blob = &path[blob_idx + marker.len()..];

    let (ref_name, file_path) = after_blob.split_once('/').ok_or_else(|| {
        anyhow::anyhow!("URL does not contain a file path after ref: '{raw_url}'")
    })?;

    if project_path.is_empty() || ref_name.is_empty() || file_path.is_empty() {
        anyhow::bail!("Invalid GitLab file URL: project, ref, or file path is empty");
    }

    Ok(ParsedFileUrl {
        host,
        project_path,
        ref_name: ref_name.to_string(),
        file_path: file_path.to_string(),
        after_blob: after_blob.to_string(),
    })
}

/// Executes the `gitlab file <URL>` command to fetch and print raw repository files.
///
/// # Arguments
///
/// * `args` - File arguments containing the web URL.
/// * `config` - Loaded configuration to resolve credentials for the URL's host.
/// * `json_mode` - Whether to output raw JSON.
///
/// # Errors
///
/// Returns an error if parsing or API communication fails.
pub async fn execute_file(args: FileArgs, config: &Config, json_mode: bool) -> anyhow::Result<()> {
    let parsed = parse_file_url(&args.url)?;

    let auth = config.resolve_auth(Some(&parsed.host), None)?;
    let client = GitLabClient::with_job_token(&auth.host, &auth.token, auth.is_job_token)?;

    let candidates = parsed.candidate_splits();
    let mut last_err = None;
    let mut resolved = None;

    for (cand_ref, cand_path) in candidates {
        match client
            .get_raw_file(&parsed.project_path, &cand_path, &cand_ref)
            .await
        {
            Ok(content) => {
                resolved = Some((cand_ref, cand_path, content));
                break;
            }
            Err(e) => {
                last_err = Some(e);
            }
        }
    }

    let (final_ref, final_path, content) = match resolved {
        Some(res) => res,
        None => match last_err {
            Some(err) => return Err(err.into()),
            None => anyhow::bail!("File not found in repository: '{}'", args.url),
        },
    };

    if json_mode {
        let output = serde_json::json!({
            "host": parsed.host,
            "project": parsed.project_path,
            "ref": final_ref,
            "path": final_path,
            "content": content,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    print!("{content}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_file_url_standard() {
        let url = "https://gitlab.com/gitlab-org/gitlab/-/blob/master/README.md";
        let parsed = parse_file_url(url).unwrap();
        assert_eq!(parsed.host, "gitlab.com");
        assert_eq!(parsed.project_path, "gitlab-org/gitlab");
        assert_eq!(parsed.ref_name, "master");
        assert_eq!(parsed.file_path, "README.md");
    }

    #[test]
    fn test_parse_file_url_nested_path_and_custom_host() {
        let url = "https://gitlab.corp.example.com/org/team/project/-/blob/feature-123/src/models/user.rs";
        let parsed = parse_file_url(url).unwrap();
        assert_eq!(parsed.host, "gitlab.corp.example.com");
        assert_eq!(parsed.project_path, "org/team/project");
        assert_eq!(parsed.ref_name, "feature-123");
        assert_eq!(parsed.file_path, "src/models/user.rs");
    }

    #[test]
    fn test_parse_file_url_invalid() {
        assert!(parse_file_url("not-a-url").is_err());
        assert!(parse_file_url("https://gitlab.com/group/project/blob/main/file.txt").is_err());
    }

    #[test]
    fn test_candidate_splits_multi_segment_branch() {
        let url = "https://gitlab.com/group/proj/-/blob/feature/sub/src/main.rs";
        let parsed = parse_file_url(url).unwrap();
        let splits = parsed.candidate_splits();
        assert_eq!(
            splits,
            vec![
                ("feature".to_string(), "sub/src/main.rs".to_string()),
                ("feature/sub".to_string(), "src/main.rs".to_string()),
                ("feature/sub/src".to_string(), "main.rs".to_string()),
            ]
        );
    }
}
