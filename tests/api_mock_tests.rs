//! Async HTTP mock server integration tests for GitLab REST API operations.

use gitlab_cli::api::GitLabClient;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

#[tokio::test]
async fn test_list_projects_pagination() {
    let mock_server = MockServer::start().await;

    let p1 = serde_json::json!([{
        "id": 1,
        "name": "Project One",
        "path": "p1",
        "path_with_namespace": "group/p1",
        "star_count": 5
    }]);

    let p2 = serde_json::json!([{
        "id": 2,
        "name": "Project Two",
        "path": "p2",
        "path_with_namespace": "group/p2",
        "star_count": 10
    }]);

    let next_link = format!(
        "<{}/api/v4/projects?page=2&per_page=1>; rel=\"next\"",
        mock_server.uri()
    );

    Mock::given(method("GET"))
        .and(path("/api/v4/projects"))
        .and(query_param("page", "1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(&p1)
                .append_header("link", next_link.as_str())
                .append_header("x-page", "1")
                .append_header("x-next-page", "2"),
        )
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects"))
        .and(query_param("page", "2"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(&p2)
                .append_header("x-page", "2"),
        )
        .mount(&mock_server)
        .await;

    let client = GitLabClient::new(mock_server.uri(), "test-token").unwrap();
    let projects = client.list_projects(true, Some(2)).await.unwrap();

    assert_eq!(projects.len(), 2);
    assert_eq!(projects[0].name, "Project One");
    assert_eq!(projects[1].name, "Project Two");
}

#[tokio::test]
async fn test_keyset_pagination_suppresses_page_param() {
    let mock_server = MockServer::start().await;

    let items = serde_json::json!([{
        "id": 1,
        "name": "Project One",
        "path": "p1",
        "path_with_namespace": "group/p1"
    }]);

    Mock::given(method("GET"))
        .and(path("/api/v4/projects"))
        .and(query_param("pagination", "keyset"))
        .and(wiremock::matchers::query_param_is_missing("page"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&items))
        .mount(&mock_server)
        .await;

    let client = GitLabClient::new(mock_server.uri(), "token").unwrap();
    let res: Vec<gitlab_cli::models::Project> = client
        .get_paged("projects", &[("pagination", "keyset")], Some(1))
        .await
        .unwrap();

    assert_eq!(res.len(), 1);
}

#[tokio::test]
async fn test_list_mrs_with_query_params() {
    let mock_server = MockServer::start().await;

    let mr_response = serde_json::json!([{
        "id": 101,
        "iid": 12,
        "project_id": 99,
        "title": "Add feature X",
        "state": "opened",
        "source_branch": "feature-x",
        "target_branch": "main"
    }]);

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/owner%2Frepo/merge_requests"))
        .and(query_param("state", "opened"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&mr_response))
        .mount(&mock_server)
        .await;

    let client = GitLabClient::new(mock_server.uri(), "token").unwrap();
    let mrs = client
        .list_mrs("owner/repo", Some("opened"), None, Some(10))
        .await
        .unwrap();

    assert_eq!(mrs.len(), 1);
    assert_eq!(mrs[0].iid, 12);
    assert_eq!(mrs[0].title, "Add feature X");
}

#[tokio::test]
async fn test_create_and_merge_mr() {
    let mock_server = MockServer::start().await;

    let created_mr = serde_json::json!({
        "id": 200,
        "iid": 5,
        "project_id": 42,
        "title": "Draft: New Feature",
        "state": "opened",
        "source_branch": "feat",
        "target_branch": "main",
        "draft": true,
        "squash": true
    });

    Mock::given(method("POST"))
        .and(path("/api/v4/projects/myorg%2Fproj/merge_requests"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&created_mr))
        .mount(&mock_server)
        .await;

    let merged_mr = serde_json::json!({
        "id": 200,
        "iid": 5,
        "project_id": 42,
        "title": "Draft: New Feature",
        "state": "merged",
        "source_branch": "feat",
        "target_branch": "main"
    });

    Mock::given(method("PUT"))
        .and(path("/api/v4/projects/myorg%2Fproj/merge_requests/5/merge"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&merged_mr))
        .mount(&mock_server)
        .await;

    let client = GitLabClient::new(mock_server.uri(), "token").unwrap();
    let mr = client
        .create_mr(&gitlab_cli::api::CreateMrParams {
            project_path: "myorg/proj",
            source_branch: "feat",
            target_branch: "main",
            title: "New Feature",
            description: Some("Description"),
            draft: true,
            squash: true,
        })
        .await
        .unwrap();

    assert_eq!(mr.iid, 5);
    assert!(mr.is_draft());

    let merged = client.merge_mr("myorg/proj", 5, true, true).await.unwrap();
    assert_eq!(merged.state, "merged");
}

#[tokio::test]
async fn test_list_and_create_issue() {
    let mock_server = MockServer::start().await;

    let issue_json = serde_json::json!({
        "id": 55,
        "iid": 3,
        "project_id": 1,
        "title": "Bug in parser",
        "state": "opened",
        "labels": ["bug", "v1"]
    });

    Mock::given(method("POST"))
        .and(path("/api/v4/projects/group%2Frepo/issues"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&issue_json))
        .mount(&mock_server)
        .await;

    let client = GitLabClient::new(mock_server.uri(), "token").unwrap();
    let created = client
        .create_issue("group/repo", "Bug in parser", None, Some("bug,v1"))
        .await
        .unwrap();

    assert_eq!(created.iid, 3);
    assert_eq!(created.title, "Bug in parser");
    assert_eq!(created.labels, vec!["bug", "v1"]);
}

#[tokio::test]
async fn test_pipeline_and_job_trace_stream() {
    let mock_server = MockServer::start().await;

    let pipeline_json = serde_json::json!({
        "id": 999,
        "project_id": 12,
        "sha": "abcdef1234567890",
        "ref": "main",
        "status": "success",
        "duration": 45
    });

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/group%2Frepo/pipelines/999"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&pipeline_json))
        .mount(&mock_server)
        .await;

    let raw_trace = "Running with gitlab-runner 16.5.0\nStep 1: cargo test... OK\nJob succeeded\n";

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/group%2Frepo/jobs/777/trace"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(raw_trace, "text/plain"))
        .mount(&mock_server)
        .await;

    let client = GitLabClient::new(mock_server.uri(), "token").unwrap();
    let pipe = client.get_pipeline("group/repo", 999).await.unwrap();
    assert_eq!(pipe.id, 999);
    assert_eq!(pipe.status, "success");

    let mut output_bytes = Vec::new();
    client
        .stream_job_trace("group/repo", 777, &mut output_bytes)
        .await
        .unwrap();

    let output_str = String::from_utf8(output_bytes).unwrap();
    assert!(output_str.contains("Job succeeded"));
}

#[tokio::test]
async fn test_rate_limit_backoff_and_retry() {
    let mock_server = MockServer::start().await;
    let request_count = Arc::new(AtomicU32::new(0));

    let counter_clone = Arc::clone(&request_count);
    Mock::given(method("GET"))
        .and(path("/api/v4/user"))
        .and(header("PRIVATE-TOKEN", "token-rate-limit"))
        .respond_with(move |_: &Request| {
            let count = counter_clone.fetch_add(1, Ordering::SeqCst);
            if count == 0 {
                // First attempt: return 429 with 1 second Retry-After
                ResponseTemplate::new(429)
                    .append_header("retry-after", "1")
                    .set_body_json(serde_json::json!({
                        "message": "429 Too Many Requests"
                    }))
            } else {
                // Second attempt: succeed
                ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "id": 1,
                    "username": "rate_limited_user",
                    "name": "Rate Limited User"
                }))
            }
        })
        .mount(&mock_server)
        .await;

    let client = GitLabClient::new(mock_server.uri(), "token-rate-limit").unwrap();
    let user = client.current_user().await.unwrap();

    assert_eq!(user.username, "rate_limited_user");
    assert_eq!(request_count.load(Ordering::SeqCst), 2);
}
