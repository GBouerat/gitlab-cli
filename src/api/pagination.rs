//! HTTP header pagination extraction and RFC 5988 Link header parsing.

use reqwest::header::HeaderMap;
use std::collections::HashMap;

/// Pagination metadata extracted from GitLab API HTTP response headers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PaginationInfo {
    /// Current page index (from `x-page`).
    pub page: Option<u64>,
    /// Number of items requested per page (from `x-per-page`).
    pub per_page: Option<u64>,
    /// Total items available (from `x-total`). Note: may be omitted on large collections.
    pub total: Option<u64>,
    /// Total pages available (from `x-total-pages`).
    pub total_pages: Option<u64>,
    /// Next page number (from `x-next-page`).
    pub next_page: Option<u64>,
    /// Previous page number (from `x-prev-page`).
    pub prev_page: Option<u64>,
    /// RFC 5988 Link relations mapped from `rel` attributes (e.g. `next`, `prev`, `first`, `last`).
    pub links: HashMap<String, String>,
}

impl PaginationInfo {
    /// Extracts pagination information from HTTP response headers.
    ///
    /// # Arguments
    ///
    /// * `headers` - The HTTP headers map from a response.
    pub fn from_headers(headers: &HeaderMap) -> Self {
        let page = headers
            .get("x-page")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok());

        let per_page = headers
            .get("x-per-page")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok());

        let total = headers
            .get("x-total")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok());

        let total_pages = headers
            .get("x-total-pages")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok());

        let next_page = headers
            .get("x-next-page")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok());

        let prev_page = headers
            .get("x-prev-page")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok());

        let links = if let Some(link_header) = headers.get("link").and_then(|v| v.to_str().ok()) {
            parse_link_header(link_header)
        } else {
            HashMap::new()
        };

        Self {
            page,
            per_page,
            total,
            total_pages,
            next_page,
            prev_page,
            links,
        }
    }

    /// Returns the URL for the next page if specified in the RFC 5988 `Link` header.
    pub fn next_url(&self) -> Option<&str> {
        self.links.get("next").map(|s| s.as_str())
    }
}

/// Parses an RFC 5988 `Link` header string into a relation-to-URL map.
///
/// Example header value:
/// `<https://gitlab.com/api/v4/projects?page=2>; rel="next", <https://gitlab.com/api/v4/projects?page=1>; rel="first"`
///
/// # Arguments
///
/// * `header` - Raw string content of the `Link` header.
pub fn parse_link_header(header: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for part in header.split(',') {
        let trimmed = part.trim();
        let segments: Vec<&str> = trimmed.split(';').map(|s| s.trim()).collect();
        if segments.len() >= 2 {
            let url_part = segments[0];
            if url_part.starts_with('<') && url_part.ends_with('>') {
                let url = &url_part[1..url_part.len() - 1];

                for param in &segments[1..] {
                    if let Some((k, v)) = param.split_once('=') {
                        if k.trim() == "rel" {
                            let rel = v.trim().trim_matches('"').to_string();
                            map.insert(rel, url.to_string());
                        }
                    }
                }
            }
        }
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::{HeaderMap, HeaderValue};

    #[test]
    fn test_parse_link_header() {
        let link_hdr = r#"<https://gitlab.com/api/v4/projects?page=2&per_page=20>; rel="next", <https://gitlab.com/api/v4/projects?page=5&per_page=20>; rel="last""#;
        let map = parse_link_header(link_hdr);
        assert_eq!(
            map.get("next"),
            Some(&"https://gitlab.com/api/v4/projects?page=2&per_page=20".to_string())
        );
        assert_eq!(
            map.get("last"),
            Some(&"https://gitlab.com/api/v4/projects?page=5&per_page=20".to_string())
        );
    }

    #[test]
    fn test_pagination_from_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("x-page", HeaderValue::from_static("1"));
        headers.insert("x-per-page", HeaderValue::from_static("20"));
        headers.insert("x-total", HeaderValue::from_static("100"));
        headers.insert("x-total-pages", HeaderValue::from_static("5"));
        headers.insert("x-next-page", HeaderValue::from_static("2"));
        headers.insert(
            "link",
            HeaderValue::from_static(r#"<https://gitlab.com/api/v4/projects?page=2>; rel="next""#),
        );

        let info = PaginationInfo::from_headers(&headers);
        assert_eq!(info.page, Some(1));
        assert_eq!(info.per_page, Some(20));
        assert_eq!(info.total, Some(100));
        assert_eq!(info.total_pages, Some(5));
        assert_eq!(info.next_page, Some(2));
        assert_eq!(
            info.next_url(),
            Some("https://gitlab.com/api/v4/projects?page=2")
        );
    }
}
