//! Asynchronous GitLab REST API v4 Client.
//!
//! Provides a strongly-typed client for sending HTTP requests, handling authentication,
//! automatic percent-encoding for namespaced paths, pagination, rate-limit backoff,
//! and streaming job trace logs.

use super::error::{parse_gitlab_error_body, ApiError};
use super::pagination::PaginationInfo;
use crate::models::{
    Branch, CiVariable, Group, GroupMember, Job, MergeRequest, Pipeline, Project, Release, Snippet,
    User,
};
use futures_util::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, USER_AGENT};
use reqwest::{Client, Method, RequestBuilder, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::Write;
use std::time::Duration;
use tracing::{debug, warn};
use url::Url;

/// Custom User-Agent header value sent on all requests.
pub const USER_AGENT_STRING: &str = "gitlab-cli/0.1.0 (https://github.com/GBouerat/gitlab-cli)";

/// Maximum number of automatic retries on rate limit (HTTP 429) before returning an error.
const MAX_RATE_LIMIT_RETRIES: u32 = 3;

/// Client for communicating with the GitLab REST API v4.
#[derive(Clone, Debug)]
pub struct GitLabClient {
    host: String,
    token: String,
    base_url: Url,
    http: Client,
}

/// Parameters for creating a new Merge Request via [`GitLabClient::create_mr`].
#[derive(Debug, Clone, Default)]
pub struct CreateMrParams<'a> {
    /// Target project ID or namespaced path.
    pub project_path: &'a str,
    /// Source feature branch containing the changes.
    pub source_branch: &'a str,
    /// Target destination branch.
    pub target_branch: &'a str,
    /// Title of the merge request.
    pub title: &'a str,
    /// Optional description text (Markdown).
    pub description: Option<&'a str>,
    /// Whether to mark the MR as Draft.
    pub draft: bool,
    /// Whether to configure commits to squash on merge.
    pub squash: bool,
}

impl GitLabClient {
    /// Creates a new `GitLabClient` targeting the specified GitLab host.
    ///
    /// # Arguments
    ///
    /// * `host` - The GitLab hostname (e.g. `gitlab.com` or `https://gitlab.example.com`).
    /// * `token` - The personal, project, or group access token.
    ///
    /// # Errors
    ///
    /// Returns an [`ApiError`] if the constructed base URL is invalid.
    pub fn new(host: impl Into<String>, token: impl Into<String>) -> Result<Self, ApiError> {
        let host_str = host.into();
        let token_str = token.into();

        let base_str = if host_str.starts_with("http://") || host_str.starts_with("https://") {
            format!("{}/api/v4/", host_str.trim_end_matches('/'))
        } else {
            format!("https://{}/api/v4/", host_str.trim_end_matches('/'))
        };

        let base_url = Url::parse(&base_str)?;

        let mut default_headers = HeaderMap::new();
        default_headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_STRING));
        default_headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        let http = Client::builder()
            .default_headers(default_headers)
            .build()
            .map_err(ApiError::Http)?;

        Ok(Self {
            host: host_str,
            token: token_str,
            base_url,
            http,
        })
    }

    /// Returns the target hostname of this client.
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Percent-encodes project path separators (`/` -> `%2F`) for use in API URL paths.
    ///
    /// Example: `owner/my-project` becomes `owner%2Fmy-project`.
    pub fn encode_path(path: &str) -> String {
        path.replace('/', "%2F")
    }

    /// Attaches authentication header to a request builder.
    fn authenticate(&self, builder: RequestBuilder) -> RequestBuilder {
        builder.header("PRIVATE-TOKEN", &self.token)
    }

    /// Executes an HTTP request with automatic rate-limit backoff handling.
    async fn execute_with_retry(&self, req_builder: RequestBuilder) -> Result<Response, ApiError> {
        let mut retries = 0;

        loop {
            // Clone the request if possible for retrying
            let req = req_builder.try_clone().ok_or_else(|| {
                ApiError::Other("Request body stream cannot be cloned".to_string())
            })?;

            let resp = req.send().await.map_err(ApiError::Http)?;
            let status = resp.status();

            if status == StatusCode::TOO_MANY_REQUESTS {
                let retry_after_secs = resp
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(2);

                if retries < MAX_RATE_LIMIT_RETRIES && retry_after_secs <= 10 {
                    retries += 1;
                    warn!(
                        "Rate limit reached (429). Retrying in {}s (attempt {}/{})",
                        retry_after_secs, retries, MAX_RATE_LIMIT_RETRIES
                    );
                    tokio::time::sleep(Duration::from_secs(retry_after_secs)).await;
                    continue;
                }

                return Err(ApiError::RateLimited { retry_after_secs });
            }

            return Ok(resp);
        }
    }

    /// Handles error status codes by converting them to typed [`ApiError`].
    async fn check_response(
        &self,
        resp: Response,
        resource_hint: &str,
    ) -> Result<Response, ApiError> {
        let status = resp.status();
        if status.is_success() {
            return Ok(resp);
        }

        let body = resp.text().await.unwrap_or_default();
        let message = parse_gitlab_error_body(&body);

        match status {
            StatusCode::UNAUTHORIZED => Err(ApiError::Unauthorized),
            StatusCode::FORBIDDEN => Err(ApiError::Forbidden { message }),
            StatusCode::NOT_FOUND => Err(ApiError::NotFound {
                resource: resource_hint.to_string(),
            }),
            _ => Err(ApiError::GitLab { status, message }),
        }
    }

    /// Performs a single `GET` request and deserializes the response.
    pub async fn get<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        query: &[(&str, &str)],
    ) -> Result<T, ApiError> {
        let clean_endpoint = endpoint.trim_start_matches('/');
        let url = self.base_url.join(clean_endpoint)?;
        debug!("GET {}", url);

        let req = self.authenticate(self.http.get(url).query(query));
        let resp = self.execute_with_retry(req).await?;
        let resp = self.check_response(resp, endpoint).await?;

        let data = resp.json::<T>().await.map_err(ApiError::Http)?;
        Ok(data)
    }

    /// Fetches all pages of a paginated resource up to an optional limit.
    pub async fn get_paged<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        query: &[(&str, &str)],
        max_items: Option<usize>,
    ) -> Result<Vec<T>, ApiError> {
        let clean_endpoint = endpoint.trim_start_matches('/');
        let mut current_url = self.base_url.join(clean_endpoint)?;
        let mut all_items = Vec::new();
        let mut following_link = false;
        let mut page = 1;
        let is_keyset = query
            .iter()
            .any(|(k, v)| *k == "pagination" && *v == "keyset");

        loop {
            let mut req = self.authenticate(self.http.get(current_url.clone()));
            if !following_link {
                if !query.is_empty() {
                    req = req.query(query);
                }
                // Default per_page parameter if not explicitly specified
                if !query.iter().any(|(k, _)| *k == "per_page") {
                    req = req.query(&[("per_page", "50")]);
                }
                if !is_keyset {
                    req = req.query(&[("page", &page.to_string())]);
                }
            }

            let resp = self.execute_with_retry(req).await?;
            let resp = self.check_response(resp, endpoint).await?;

            let pagination = PaginationInfo::from_headers(resp.headers());
            let items: Vec<T> = resp.json().await.map_err(ApiError::Http)?;

            if items.is_empty() {
                break;
            }

            all_items.extend(items);

            if let Some(limit) = max_items {
                if all_items.len() >= limit {
                    all_items.truncate(limit);
                    break;
                }
            }

            // Check next page via RFC 5988 link or x-next-page
            if let Some(next_url) = pagination.next_url() {
                current_url = Url::parse(next_url)?;
                following_link = true;
            } else if let Some(next_p) = pagination.next_page {
                page = next_p;
                following_link = false;
            } else {
                break;
            }
        }

        Ok(all_items)
    }

    /// Performs a `POST` request with a JSON body and deserializes the response.
    pub async fn post<B: Serialize, T: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let clean_endpoint = endpoint.trim_start_matches('/');
        let url = self.base_url.join(clean_endpoint)?;
        debug!("POST {}", url);

        let req = self.authenticate(self.http.post(url).json(body));
        let resp = self.execute_with_retry(req).await?;
        let resp = self.check_response(resp, endpoint).await?;

        let data = resp.json::<T>().await.map_err(ApiError::Http)?;
        Ok(data)
    }

    /// Performs a `PUT` request with a JSON body and deserializes the response.
    pub async fn put<B: Serialize, T: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let clean_endpoint = endpoint.trim_start_matches('/');
        let url = self.base_url.join(clean_endpoint)?;
        debug!("PUT {}", url);

        let req = self.authenticate(self.http.put(url).json(body));
        let resp = self.execute_with_retry(req).await?;
        let resp = self.check_response(resp, endpoint).await?;

        let data = resp.json::<T>().await.map_err(ApiError::Http)?;
        Ok(data)
    }

    /// Performs a `DELETE` request.
    pub async fn delete(&self, endpoint: &str) -> Result<(), ApiError> {
        let clean_endpoint = endpoint.trim_start_matches('/');
        let url = self.base_url.join(clean_endpoint)?;
        debug!("DELETE {}", url);

        let req = self.authenticate(self.http.delete(url));
        let resp = self.execute_with_retry(req).await?;
        self.check_response(resp, endpoint).await?;
        Ok(())
    }

    /// Streams raw job trace log chunks directly into a writer (such as stdout).
    pub async fn stream_job_trace(
        &self,
        project_path: &str,
        job_id: u64,
        writer: &mut impl Write,
    ) -> Result<(), ApiError> {
        let encoded = Self::encode_path(project_path);
        let endpoint = format!("projects/{encoded}/jobs/{job_id}/trace");
        let url = self.base_url.join(&endpoint)?;

        let req = self.authenticate(self.http.get(url));
        let resp = self.execute_with_retry(req).await?;
        let resp = self.check_response(resp, &endpoint).await?;

        let mut stream = resp.bytes_stream();
        while let Some(chunk_res) = stream.next().await {
            let chunk = chunk_res.map_err(ApiError::Http)?;
            writer
                .write_all(&chunk)
                .map_err(|e| ApiError::Other(e.to_string()))?;
            writer.flush().map_err(|e| ApiError::Other(e.to_string()))?;
        }

        Ok(())
    }

    /// Performs a raw HTTP request against any GitLab endpoint (used by `gitlab api`).
    pub async fn raw_request(
        &self,
        method: Method,
        endpoint: &str,
        query: &[(&str, &str)],
        body: Option<serde_json::Value>,
    ) -> Result<Response, ApiError> {
        let clean_endpoint = endpoint.trim_start_matches('/');
        let url = self.base_url.join(clean_endpoint)?;

        let mut req = self.authenticate(self.http.request(method, url).query(query));
        if let Some(json_val) = body {
            req = req.json(&json_val);
        }

        let resp = self.execute_with_retry(req).await?;
        self.check_response(resp, endpoint).await
    }

    // --- High-Level Helper Endpoints ---

    /// Gets the authenticated user's profile (`GET /user`).
    pub async fn current_user(&self) -> Result<User, ApiError> {
        self.get("user", &[]).await
    }

    /// Gets project details by ID or namespaced path (`GET /projects/:id`).
    pub async fn get_project(&self, id_or_path: &str) -> Result<Project, ApiError> {
        let encoded = Self::encode_path(id_or_path);
        let endpoint = format!("projects/{encoded}");
        self.get(&endpoint, &[]).await
    }

    /// Lists projects accessible to the authenticated user (`GET /projects`).
    pub async fn list_projects(
        &self,
        membership: bool,
        max_items: Option<usize>,
    ) -> Result<Vec<Project>, ApiError> {
        let membership_str = if membership { "true" } else { "false" };
        self.get_paged(
            "projects",
            &[
                ("membership", membership_str),
                ("order_by", "last_activity_at"),
            ],
            max_items,
        )
        .await
    }

    /// Lists Merge Requests for a project (`GET /projects/:id/merge_requests`).
    pub async fn list_mrs(
        &self,
        project_path: &str,
        state: Option<&str>,
        target_branch: Option<&str>,
        max_items: Option<usize>,
    ) -> Result<Vec<MergeRequest>, ApiError> {
        let encoded = Self::encode_path(project_path);
        let endpoint = format!("projects/{encoded}/merge_requests");
        let mut query: Vec<(&str, &str)> = Vec::new();
        if let Some(s) = state {
            query.push(("state", s));
        }
        if let Some(tb) = target_branch {
            query.push(("target_branch", tb));
        }

        self.get_paged(&endpoint, &query, max_items).await
    }

    /// Gets a single Merge Request by internal IID (`GET /projects/:id/merge_requests/:mr_iid`).
    pub async fn get_mr(&self, project_path: &str, iid: u64) -> Result<MergeRequest, ApiError> {
        let encoded = Self::encode_path(project_path);
        let endpoint = format!("projects/{encoded}/merge_requests/{iid}");
        self.get(&endpoint, &[]).await
    }

    /// Creates a new Merge Request (`POST /projects/:id/merge_requests`).
    pub async fn create_mr(&self, params: &CreateMrParams<'_>) -> Result<MergeRequest, ApiError> {
        let encoded = Self::encode_path(params.project_path);
        let endpoint = format!("projects/{encoded}/merge_requests");

        let effective_title = if params.draft && !params.title.starts_with("Draft:") {
            format!("Draft: {}", params.title)
        } else {
            params.title.to_string()
        };

        #[derive(Serialize)]
        struct CreateMrBody<'a> {
            source_branch: &'a str,
            target_branch: &'a str,
            title: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<&'a str>,
            squash: bool,
        }

        let body = CreateMrBody {
            source_branch: params.source_branch,
            target_branch: params.target_branch,
            title: effective_title,
            description: params.description,
            squash: params.squash,
        };

        self.post(&endpoint, &body).await
    }

    /// Merges an open Merge Request (`PUT /projects/:id/merge_requests/:mr_iid/merge`).
    pub async fn merge_mr(
        &self,
        project_path: &str,
        iid: u64,
        squash: bool,
        remove_source_branch: bool,
    ) -> Result<MergeRequest, ApiError> {
        let encoded = Self::encode_path(project_path);
        let endpoint = format!("projects/{encoded}/merge_requests/{iid}/merge");

        #[derive(Serialize)]
        struct MergeMrBody {
            squash: bool,
            should_remove_source_branch: bool,
        }

        let body = MergeMrBody {
            squash,
            should_remove_source_branch: remove_source_branch,
        };

        self.put(&endpoint, &body).await
    }

    /// Closes an open Merge Request (`PUT /projects/:id/merge_requests/:mr_iid`).
    pub async fn close_mr(&self, project_path: &str, iid: u64) -> Result<MergeRequest, ApiError> {
        let encoded = Self::encode_path(project_path);
        let endpoint = format!("projects/{encoded}/merge_requests/{iid}");

        #[derive(Serialize)]
        struct UpdateMrState<'a> {
            state_event: &'a str,
        }

        self.put(
            &endpoint,
            &UpdateMrState {
                state_event: "close",
            },
        )
        .await
    }

    /// Reopens a closed Merge Request (`PUT /projects/:id/merge_requests/:mr_iid`).
    pub async fn reopen_mr(&self, project_path: &str, iid: u64) -> Result<MergeRequest, ApiError> {
        let encoded = Self::encode_path(project_path);
        let endpoint = format!("projects/{encoded}/merge_requests/{iid}");

        #[derive(Serialize)]
        struct UpdateMrState<'a> {
            state_event: &'a str,
        }

        self.put(
            &endpoint,
            &UpdateMrState {
                state_event: "reopen",
            },
        )
        .await
    }

    /// Lists issues for a project (`GET /projects/:id/issues`).
    pub async fn list_issues(
        &self,
        project_path: &str,
        state: Option<&str>,
        labels: Option<&str>,
        max_items: Option<usize>,
    ) -> Result<Vec<crate::models::Issue>, ApiError> {
        let encoded = Self::encode_path(project_path);
        let endpoint = format!("projects/{encoded}/issues");
        let mut query: Vec<(&str, &str)> = Vec::new();
        if let Some(s) = state {
            query.push(("state", s));
        }
        if let Some(l) = labels {
            query.push(("labels", l));
        }

        self.get_paged(&endpoint, &query, max_items).await
    }

    /// Gets a single issue by project-scoped IID (`GET /projects/:id/issues/:issue_iid`).
    pub async fn get_issue(
        &self,
        project_path: &str,
        iid: u64,
    ) -> Result<crate::models::Issue, ApiError> {
        let encoded = Self::encode_path(project_path);
        let endpoint = format!("projects/{encoded}/issues/{iid}");
        self.get(&endpoint, &[]).await
    }

    /// Creates a new issue (`POST /projects/:id/issues`).
    pub async fn create_issue(
        &self,
        project_path: &str,
        title: &str,
        description: Option<&str>,
        labels: Option<&str>,
    ) -> Result<crate::models::Issue, ApiError> {
        let encoded = Self::encode_path(project_path);
        let endpoint = format!("projects/{encoded}/issues");

        #[derive(Serialize)]
        struct CreateIssueBody<'a> {
            title: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            labels: Option<&'a str>,
        }

        let body = CreateIssueBody {
            title,
            description,
            labels,
        };

        self.post(&endpoint, &body).await
    }

    /// Closes an issue (`PUT /projects/:id/issues/:issue_iid`).
    pub async fn close_issue(
        &self,
        project_path: &str,
        iid: u64,
    ) -> Result<crate::models::Issue, ApiError> {
        let encoded = Self::encode_path(project_path);
        let endpoint = format!("projects/{encoded}/issues/{iid}");

        #[derive(Serialize)]
        struct UpdateIssueState<'a> {
            state_event: &'a str,
        }

        self.put(
            &endpoint,
            &UpdateIssueState {
                state_event: "close",
            },
        )
        .await
    }

    /// Reopens a closed issue (`PUT /projects/:id/issues/:issue_iid`).
    pub async fn reopen_issue(
        &self,
        project_path: &str,
        iid: u64,
    ) -> Result<crate::models::Issue, ApiError> {
        let encoded = Self::encode_path(project_path);
        let endpoint = format!("projects/{encoded}/issues/{iid}");

        #[derive(Serialize)]
        struct UpdateIssueState<'a> {
            state_event: &'a str,
        }

        self.put(
            &endpoint,
            &UpdateIssueState {
                state_event: "reopen",
            },
        )
        .await
    }

    /// Lists pipelines for a project (`GET /projects/:id/pipelines`).
    pub async fn list_pipelines(
        &self,
        project_path: &str,
        ref_name: Option<&str>,
        status: Option<&str>,
        max_items: Option<usize>,
    ) -> Result<Vec<Pipeline>, ApiError> {
        let encoded = Self::encode_path(project_path);
        let endpoint = format!("projects/{encoded}/pipelines");
        let mut query: Vec<(&str, &str)> = Vec::new();
        if let Some(r) = ref_name {
            query.push(("ref", r));
        }
        if let Some(s) = status {
            query.push(("status", s));
        }

        self.get_paged(&endpoint, &query, max_items).await
    }

    /// Gets a single pipeline by ID (`GET /projects/:id/pipelines/:pipeline_id`).
    pub async fn get_pipeline(&self, project_path: &str, id: u64) -> Result<Pipeline, ApiError> {
        let encoded = Self::encode_path(project_path);
        let endpoint = format!("projects/{encoded}/pipelines/{id}");
        self.get(&endpoint, &[]).await
    }

    /// Lists jobs for a pipeline (`GET /projects/:id/pipelines/:pipeline_id/jobs`).
    pub async fn list_pipeline_jobs(
        &self,
        project_path: &str,
        pipeline_id: u64,
    ) -> Result<Vec<Job>, ApiError> {
        let encoded = Self::encode_path(project_path);
        let endpoint = format!("projects/{encoded}/pipelines/{pipeline_id}/jobs");
        self.get_paged(&endpoint, &[], None).await
    }

    /// Creates / triggers a new pipeline on a given branch (`POST /projects/:id/pipeline`).
    pub async fn create_pipeline(
        &self,
        project_path: &str,
        ref_name: &str,
    ) -> Result<Pipeline, ApiError> {
        let encoded = Self::encode_path(project_path);
        let endpoint = format!("projects/{encoded}/pipeline");

        #[derive(Serialize)]
        struct CreatePipelineBody<'a> {
            #[serde(rename = "ref")]
            ref_name: &'a str,
        }

        self.post(&endpoint, &CreatePipelineBody { ref_name }).await
    }

    /// Retries failed jobs in a pipeline (`POST /projects/:id/pipelines/:pipeline_id/retry`).
    pub async fn retry_pipeline(&self, project_path: &str, id: u64) -> Result<Pipeline, ApiError> {
        let encoded = Self::encode_path(project_path);
        let endpoint = format!("projects/{encoded}/pipelines/{id}/retry");
        self.post(&endpoint, &serde_json::json!({})).await
    }

    /// Cancels a running pipeline (`POST /projects/:id/pipelines/:pipeline_id/cancel`).
    pub async fn cancel_pipeline(&self, project_path: &str, id: u64) -> Result<Pipeline, ApiError> {
        let encoded = Self::encode_path(project_path);
        let endpoint = format!("projects/{encoded}/pipelines/{id}/cancel");
        self.post(&endpoint, &serde_json::json!({})).await
    }

    // --- Release Endpoints ---

    /// Lists releases for a project (`GET /projects/:id/releases`).
    pub async fn list_releases(
        &self,
        project_path: &str,
        max_items: Option<usize>,
    ) -> Result<Vec<Release>, ApiError> {
        let encoded = Self::encode_path(project_path);
        let endpoint = format!("projects/{encoded}/releases");
        self.get_paged(&endpoint, &[], max_items).await
    }

    /// Gets a single release by tag name (`GET /projects/:id/releases/:tag_name`).
    pub async fn get_release(
        &self,
        project_path: &str,
        tag_name: &str,
    ) -> Result<Release, ApiError> {
        let encoded = Self::encode_path(project_path);
        let encoded_tag = Self::encode_path(tag_name);
        let endpoint = format!("projects/{encoded}/releases/{encoded_tag}");
        self.get(&endpoint, &[]).await
    }

    /// Creates a new release (`POST /projects/:id/releases`).
    pub async fn create_release(
        &self,
        project_path: &str,
        tag_name: &str,
        name: Option<&str>,
        description: Option<&str>,
        ref_name: Option<&str>,
    ) -> Result<Release, ApiError> {
        let encoded = Self::encode_path(project_path);
        let endpoint = format!("projects/{encoded}/releases");

        #[derive(Serialize)]
        struct CreateReleaseBody<'a> {
            tag_name: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            name: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<&'a str>,
            #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
            ref_name: Option<&'a str>,
        }

        let body = CreateReleaseBody {
            tag_name,
            name,
            description,
            ref_name,
        };

        self.post(&endpoint, &body).await
    }

    /// Deletes a release (`DELETE /projects/:id/releases/:tag_name`).
    pub async fn delete_release(&self, project_path: &str, tag_name: &str) -> Result<(), ApiError> {
        let encoded = Self::encode_path(project_path);
        let encoded_tag = Self::encode_path(tag_name);
        let endpoint = format!("projects/{encoded}/releases/{encoded_tag}");
        self.delete(&endpoint).await
    }

    // --- Branch Endpoints ---

    /// Lists repository branches (`GET /projects/:id/repository/branches`).
    pub async fn list_branches(
        &self,
        project_path: &str,
        max_items: Option<usize>,
    ) -> Result<Vec<Branch>, ApiError> {
        let encoded = Self::encode_path(project_path);
        let endpoint = format!("projects/{encoded}/repository/branches");
        self.get_paged(&endpoint, &[], max_items).await
    }

    /// Gets a single repository branch (`GET /projects/:id/repository/branches/:branch`).
    pub async fn get_branch(&self, project_path: &str, branch: &str) -> Result<Branch, ApiError> {
        let encoded = Self::encode_path(project_path);
        let encoded_branch = Self::encode_path(branch);
        let endpoint = format!("projects/{encoded}/repository/branches/{encoded_branch}");
        self.get(&endpoint, &[]).await
    }

    /// Creates a new branch from a ref (`POST /projects/:id/repository/branches`).
    pub async fn create_branch(
        &self,
        project_path: &str,
        branch: &str,
        ref_name: &str,
    ) -> Result<Branch, ApiError> {
        let encoded = Self::encode_path(project_path);
        let endpoint = format!("projects/{encoded}/repository/branches");

        #[derive(Serialize)]
        struct CreateBranchBody<'a> {
            branch: &'a str,
            #[serde(rename = "ref")]
            ref_name: &'a str,
        }

        let body = CreateBranchBody { branch, ref_name };

        self.post(&endpoint, &body).await
    }

    /// Deletes a branch (`DELETE /projects/:id/repository/branches/:branch`).
    pub async fn delete_branch(&self, project_path: &str, branch: &str) -> Result<(), ApiError> {
        let encoded = Self::encode_path(project_path);
        let encoded_branch = Self::encode_path(branch);
        let endpoint = format!("projects/{encoded}/repository/branches/{encoded_branch}");
        self.delete(&endpoint).await
    }

    // --- Group Endpoints ---

    /// Lists groups accessible to the user (`GET /groups`).
    pub async fn list_groups(
        &self,
        search: Option<&str>,
        max_items: Option<usize>,
    ) -> Result<Vec<Group>, ApiError> {
        let mut query = Vec::new();
        if let Some(s) = search {
            query.push(("search", s));
        }
        self.get_paged("groups", &query, max_items).await
    }

    /// Gets a single group by ID or path (`GET /groups/:id`).
    pub async fn get_group(&self, id_or_path: &str) -> Result<Group, ApiError> {
        let encoded = Self::encode_path(id_or_path);
        let endpoint = format!("groups/{encoded}");
        self.get(&endpoint, &[]).await
    }

    /// Lists members of a group (`GET /groups/:id/members`).
    pub async fn list_group_members(
        &self,
        id_or_path: &str,
        max_items: Option<usize>,
    ) -> Result<Vec<GroupMember>, ApiError> {
        let encoded = Self::encode_path(id_or_path);
        let endpoint = format!("groups/{encoded}/members");
        self.get_paged(&endpoint, &[], max_items).await
    }

    // --- CI/CD Variable Endpoints ---

    /// Lists CI/CD variables configured for a project (`GET /projects/:id/variables`).
    pub async fn list_variables(
        &self,
        project_path: &str,
        max_items: Option<usize>,
    ) -> Result<Vec<CiVariable>, ApiError> {
        let encoded = Self::encode_path(project_path);
        let endpoint = format!("projects/{encoded}/variables");
        self.get_paged(&endpoint, &[], max_items).await
    }

    /// Gets a single variable by key (`GET /projects/:id/variables/:key`).
    pub async fn get_variable(
        &self,
        project_path: &str,
        key: &str,
    ) -> Result<CiVariable, ApiError> {
        let encoded = Self::encode_path(project_path);
        let endpoint = format!("projects/{encoded}/variables/{key}");
        self.get(&endpoint, &[]).await
    }

    /// Sets or updates a project CI/CD variable (`POST /projects/:id/variables` or `PUT`).
    pub async fn set_variable(
        &self,
        project_path: &str,
        key: &str,
        value: &str,
        masked: bool,
        protected: bool,
        environment_scope: Option<&str>,
    ) -> Result<CiVariable, ApiError> {
        let encoded = Self::encode_path(project_path);
        let endpoint = format!("projects/{encoded}/variables");

        #[derive(Serialize)]
        struct SetVariableBody<'a> {
            key: &'a str,
            value: &'a str,
            masked: bool,
            protected: bool,
            #[serde(skip_serializing_if = "Option::is_none")]
            environment_scope: Option<&'a str>,
        }

        let body = SetVariableBody {
            key,
            value,
            masked,
            protected,
            environment_scope,
        };

        // Attempt POST first
        match self
            .post::<SetVariableBody, CiVariable>(&endpoint, &body)
            .await
        {
            Ok(v) => Ok(v),
            Err(ApiError::GitLab { status, .. }) if status == StatusCode::BAD_REQUEST => {
                // If it already exists, update via PUT
                let update_endpoint = format!("projects/{encoded}/variables/{key}");
                self.put(&update_endpoint, &body).await
            }
            Err(e) => Err(e),
        }
    }

    /// Deletes a project CI/CD variable (`DELETE /projects/:id/variables/:key`).
    pub async fn delete_variable(&self, project_path: &str, key: &str) -> Result<(), ApiError> {
        let encoded = Self::encode_path(project_path);
        let endpoint = format!("projects/{encoded}/variables/{key}");
        self.delete(&endpoint).await
    }

    // --- Snippet Endpoints ---

    /// Lists snippets accessible to the user (`GET /snippets`).
    pub async fn list_snippets(&self, max_items: Option<usize>) -> Result<Vec<Snippet>, ApiError> {
        self.get_paged("snippets", &[], max_items).await
    }

    /// Gets a single snippet by numeric ID (`GET /snippets/:id`).
    pub async fn get_snippet(&self, id: u64) -> Result<Snippet, ApiError> {
        let endpoint = format!("snippets/{id}");
        self.get(&endpoint, &[]).await
    }

    /// Gets the raw text content of a snippet (`GET /snippets/:id/raw`).
    pub async fn get_snippet_raw(&self, id: u64) -> Result<String, ApiError> {
        let endpoint = format!("snippets/{id}/raw");
        let url = self.base_url.join(&endpoint)?;
        let req = self.authenticate(self.http.get(url));
        let resp = self.execute_with_retry(req).await?;
        let resp = self.check_response(resp, &endpoint).await?;
        let text = resp.text().await.map_err(ApiError::Http)?;
        Ok(text)
    }

    /// Creates a new snippet (`POST /snippets`).
    pub async fn create_snippet(
        &self,
        title: &str,
        file_name: &str,
        content: &str,
        visibility: &str,
        description: Option<&str>,
    ) -> Result<Snippet, ApiError> {
        #[derive(Serialize)]
        struct CreateSnippetBody<'a> {
            title: &'a str,
            file_name: &'a str,
            content: &'a str,
            visibility: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<&'a str>,
        }

        let body = CreateSnippetBody {
            title,
            file_name,
            content,
            visibility,
            description,
        };

        self.post("snippets", &body).await
    }

    /// Deletes a snippet (`DELETE /snippets/:id`).
    pub async fn delete_snippet(&self, id: u64) -> Result<(), ApiError> {
        let endpoint = format!("snippets/{id}");
        self.delete(&endpoint).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn test_encode_path() {
        assert_eq!(GitLabClient::encode_path("owner/repo"), "owner%2Frepo");
        assert_eq!(
            GitLabClient::encode_path("group/subgroup/repo"),
            "group%2Fsubgroup%2Frepo"
        );
        assert_eq!(GitLabClient::encode_path("single"), "single");
    }

    #[tokio::test]
    async fn test_client_current_user_mock() {
        let mock_server = MockServer::start().await;

        let user_response = serde_json::json!({
            "id": 42,
            "username": "octocat",
            "name": "Octo Cat",
            "state": "active",
            "web_url": "https://gitlab.com/octocat"
        });

        Mock::given(method("GET"))
            .and(path("/api/v4/user"))
            .and(header("PRIVATE-TOKEN", "mock-token-xyz"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&user_response))
            .mount(&mock_server)
            .await;

        let client = GitLabClient::new(mock_server.uri(), "mock-token-xyz").unwrap();
        let user = client.current_user().await.unwrap();

        assert_eq!(user.id, 42);
        assert_eq!(user.username, "octocat");
        assert_eq!(user.name, "Octo Cat");
    }

    #[tokio::test]
    async fn test_client_unauthorized_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v4/user"))
            .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
                "message": "401 Unauthorized"
            })))
            .mount(&mock_server)
            .await;

        let client = GitLabClient::new(mock_server.uri(), "bad-token").unwrap();
        let err = client.current_user().await.unwrap_err();

        match err {
            ApiError::Unauthorized => (),
            other => panic!("Expected Unauthorized, got {:?}", other),
        }
    }
}
