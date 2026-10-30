//! Error types for the GitLab REST API client.

use thiserror::Error;

/// Specific errors returned from or encountered while communicating with the GitLab REST API.
#[derive(Debug, Error)]
pub enum ApiError {
    /// HTTP transport or connection failure.
    #[error("HTTP transport error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization or deserialization error.
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),

    /// URL parsing error.
    #[error("URL parsing error: {0}")]
    Url(#[from] url::ParseError),

    /// The API responded with a 401 Unauthorized status.
    #[error("Unauthorized (401): invalid or expired token")]
    Unauthorized,

    /// The API responded with a 403 Forbidden status.
    #[error("Forbidden (403): insufficient permissions: {message}")]
    Forbidden {
        /// Error message returned by GitLab.
        message: String,
    },

    /// The API responded with a 404 Not Found status.
    #[error("Not Found (404): resource '{resource}' not found")]
    NotFound {
        /// Target resource or path.
        resource: String,
    },

    /// The API rate limit was exceeded (429 Too Many Requests).
    #[error("Rate limit exceeded (429). Retry after {retry_after_secs}s")]
    RateLimited {
        /// Number of seconds to wait before retrying, if provided by the server.
        retry_after_secs: u64,
    },

    /// General GitLab API error response with status code and server message.
    #[error("GitLab API error ({status}): {message}")]
    GitLab {
        /// HTTP status code.
        status: reqwest::StatusCode,
        /// Message extracted from response body.
        message: String,
    },

    /// Other unexpected or internal errors.
    #[error("{0}")]
    Other(String),
}

/// Parses a GitLab error response body into a human-readable message.
///
/// GitLab returns error messages in different JSON schemas:
/// - `{"message": "Cannot modify locked merge request"}`
/// - `{"error": "invalid_token"}`
/// - `{"message": {"title": ["can't be blank"], "target_branch": ["is invalid"]}}`
pub fn parse_gitlab_error_body(raw_body: &str) -> String {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(raw_body) {
        if let Some(msg) = value.get("message") {
            if let Some(s) = msg.as_str() {
                return s.to_string();
            }
            if let Some(obj) = msg.as_object() {
                let parts: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| {
                        if let Some(arr) = v.as_array() {
                            let items: Vec<String> =
                                arr.iter().filter_map(|i| i.as_str()).map(String::from).collect();
                            format!("{k}: {}", items.join(", "))
                        } else {
                            format!("{k}: {v}")
                        }
                    })
                    .collect();
                return parts.join("; ");
            }
        }
        if let Some(err) = value.get("error").and_then(|e| e.as_str()) {
            return err.to_string();
        }
    }

    raw_body.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_message() {
        let json = r#"{"message": "Project not found"}"#;
        assert_eq!(parse_gitlab_error_body(json), "Project not found");
    }

    #[test]
    fn test_parse_error_field() {
        let json = r#"{"error": "invalid_token"}"#;
        assert_eq!(parse_gitlab_error_body(json), "invalid_token");
    }

    #[test]
    fn test_parse_nested_object_message() {
        let json = r#"{"message": {"title": ["can't be blank"], "branch": ["is required"]}}"#;
        let parsed = parse_gitlab_error_body(json);
        assert!(parsed.contains("title: can't be blank"));
        assert!(parsed.contains("branch: is required"));
    }

    #[test]
    fn test_parse_non_json_body() {
        let raw = "502 Bad Gateway from NGINX";
        assert_eq!(parse_gitlab_error_body(raw), "502 Bad Gateway from NGINX");
    }
}
