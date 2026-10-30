//! Command handler for `gitlab api`.

use crate::api::GitLabClient;
use crate::cli::api::ApiArgs;
use reqwest::Method;
use std::str::FromStr;

/// Dispatches direct REST API requests.
///
/// # Arguments
///
/// * `args` - API command arguments.
/// * `client` - Authenticated GitLab API client.
/// * `default_project` - Optional default project path detected from Git context.
///
/// # Errors
///
/// Returns an error if the HTTP method is invalid or the API request fails.
pub async fn execute_api(
    args: ApiArgs,
    client: &GitLabClient,
    default_project: Option<&str>,
) -> anyhow::Result<()> {
    let method = Method::from_str(&args.method.to_uppercase())?;

    // Replace :id with encoded default project path if available
    let endpoint = if let Some(proj) = default_project {
        let encoded = GitLabClient::encode_path(proj);
        args.endpoint.replace(":id", &encoded)
    } else {
        args.endpoint
    };

    let mut query_params = Vec::new();
    let mut body_map = serde_json::Map::new();

    for field in &args.fields {
        if let Some((k, v)) = field.split_once('=') {
            let key = k.trim().to_string();
            let val = v.trim();

            if method == Method::GET || method == Method::DELETE {
                query_params.push((key, val.to_string()));
            } else {
                // Auto-cast booleans and numbers where applicable
                let json_val = if let Ok(b) = val.parse::<bool>() {
                    serde_json::Value::Bool(b)
                } else if let Ok(n) = val.parse::<i64>() {
                    serde_json::Value::Number(n.into())
                } else {
                    serde_json::Value::String(val.to_string())
                };
                body_map.insert(key, json_val);
            }
        }
    }

    let query_refs: Vec<(&str, &str)> = query_params
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    let body_opt = if body_map.is_empty() {
        None
    } else {
        Some(serde_json::Value::Object(body_map))
    };

    let resp = client
        .raw_request(method, &endpoint, &query_refs, body_opt)
        .await?;

    let text = resp.text().await?;
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
        println!("{}", serde_json::to_string_pretty(&json)?);
    } else {
        println!("{text}");
    }

    Ok(())
}
