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

#[tokio::test]
async fn test_releases_operations() {
    let mock_server = MockServer::start().await;

    let release_json = serde_json::json!({
        "tag_name": "v1.0.0",
        "name": "Initial Release",
        "description": "First stable release",
        "created_at": "2026-01-01T00:00:00Z",
        "released_at": "2026-01-01T00:00:00Z",
        "_links": {
            "self": "https://gitlab.example.com/api/v4/projects/1/releases/v1.0.0"
        }
    });

    // List releases
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/group%2Frepo/releases"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([release_json])))
        .mount(&mock_server)
        .await;

    // Get release
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/group%2Frepo/releases/v1.0.0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&release_json))
        .mount(&mock_server)
        .await;

    // Create release
    Mock::given(method("POST"))
        .and(path("/api/v4/projects/group%2Frepo/releases"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&release_json))
        .mount(&mock_server)
        .await;

    // Delete release
    Mock::given(method("DELETE"))
        .and(path("/api/v4/projects/group%2Frepo/releases/v1.0.0"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let client = GitLabClient::new(mock_server.uri(), "test-token").unwrap();

    let list = client.list_releases("group/repo", Some(10)).await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].tag_name, "v1.0.0");

    let single = client.get_release("group/repo", "v1.0.0").await.unwrap();
    assert_eq!(single.tag_name, "v1.0.0");
    assert_eq!(single.name, "Initial Release");

    let created = client
        .create_release(
            "group/repo",
            "v1.0.0",
            Some("Initial Release"),
            Some("First stable release"),
            Some("main"),
        )
        .await
        .unwrap();
    assert_eq!(created.tag_name, "v1.0.0");

    client.delete_release("group/repo", "v1.0.0").await.unwrap();
}

#[tokio::test]
async fn test_branches_operations() {
    let mock_server = MockServer::start().await;

    let branch_json = serde_json::json!({
        "name": "feature/awesome",
        "merged": false,
        "protected": false,
        "default": false,
        "web_url": "https://gitlab.example.com/group/repo/-/tree/feature/awesome",
        "commit": {
            "id": "1234567890abcdef",
            "short_id": "12345678",
            "title": "Add feature",
            "author_name": "Developer"
        }
    });

    // List branches
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/group%2Frepo/repository/branches"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([branch_json])))
        .mount(&mock_server)
        .await;

    // Get branch
    Mock::given(method("GET"))
        .and(path(
            "/api/v4/projects/group%2Frepo/repository/branches/feature%2Fawesome",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(&branch_json))
        .mount(&mock_server)
        .await;

    // Create branch
    Mock::given(method("POST"))
        .and(path("/api/v4/projects/group%2Frepo/repository/branches"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&branch_json))
        .mount(&mock_server)
        .await;

    // Delete branch
    Mock::given(method("DELETE"))
        .and(path(
            "/api/v4/projects/group%2Frepo/repository/branches/feature%2Fawesome",
        ))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let client = GitLabClient::new(mock_server.uri(), "test-token").unwrap();

    let list = client.list_branches("group/repo", Some(10)).await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "feature/awesome");

    let single = client
        .get_branch("group/repo", "feature/awesome")
        .await
        .unwrap();
    assert_eq!(single.name, "feature/awesome");

    let created = client
        .create_branch("group/repo", "feature/awesome", "main")
        .await
        .unwrap();
    assert_eq!(created.name, "feature/awesome");

    client
        .delete_branch("group/repo", "feature/awesome")
        .await
        .unwrap();
}

#[tokio::test]
async fn test_groups_and_members() {
    let mock_server = MockServer::start().await;

    let group_json = serde_json::json!({
        "id": 10,
        "name": "Core Platform",
        "path": "core",
        "full_name": "Core Platform",
        "full_path": "core",
        "visibility": "public",
        "web_url": "https://gitlab.example.com/groups/core"
    });

    let member_json = serde_json::json!({
        "id": 101,
        "username": "alice",
        "name": "Alice Smith",
        "state": "active",
        "access_level": 40
    });

    Mock::given(method("GET"))
        .and(path("/api/v4/groups"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([group_json])))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v4/groups/core"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&group_json))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v4/groups/core/members"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([member_json])))
        .mount(&mock_server)
        .await;

    let client = GitLabClient::new(mock_server.uri(), "test-token").unwrap();

    let groups = client.list_groups(None, Some(5)).await.unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].name, "Core Platform");

    let group = client.get_group("core").await.unwrap();
    assert_eq!(group.id, 10);
    assert_eq!(group.full_path, "core");

    let members = client.list_group_members("core", Some(5)).await.unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].username, "alice");
    assert_eq!(members[0].access_level, 40);
}

#[tokio::test]
async fn test_variables_operations_with_put_fallback() {
    let mock_server = MockServer::start().await;

    let var_json = serde_json::json!({
        "variable_type": "env_var",
        "key": "DATABASE_URL",
        "value": "postgres://localhost/db",
        "protected": true,
        "masked": true,
        "environment_scope": "*"
    });

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/group%2Frepo/variables"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([var_json])))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/group%2Frepo/variables/DATABASE_URL"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&var_json))
        .mount(&mock_server)
        .await;

    // Simulate POST returning 400 (already exists)
    Mock::given(method("POST"))
        .and(path("/api/v4/projects/group%2Frepo/variables"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "message": {"key": ["has already been taken"]}
        })))
        .mount(&mock_server)
        .await;

    // PUT fallback succeeds
    Mock::given(method("PUT"))
        .and(path("/api/v4/projects/group%2Frepo/variables/DATABASE_URL"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&var_json))
        .mount(&mock_server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/api/v4/projects/group%2Frepo/variables/DATABASE_URL"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let client = GitLabClient::new(mock_server.uri(), "test-token").unwrap();

    let list = client.list_variables("group/repo", Some(5)).await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].key, "DATABASE_URL");

    let var = client
        .get_variable("group/repo", "DATABASE_URL")
        .await
        .unwrap();
    assert_eq!(var.key, "DATABASE_URL");
    assert_eq!(var.masked, Some(true));

    // Set variable falls back to PUT on 400
    let updated = client
        .set_variable(
            "group/repo",
            "DATABASE_URL",
            "postgres://localhost/db",
            true,
            true,
            Some("*"),
        )
        .await
        .unwrap();
    assert_eq!(updated.key, "DATABASE_URL");

    client
        .delete_variable("group/repo", "DATABASE_URL")
        .await
        .unwrap();
}

#[tokio::test]
async fn test_snippets_operations() {
    let mock_server = MockServer::start().await;

    let snippet_json = serde_json::json!({
        "id": 42,
        "title": "Example Script",
        "file_name": "test.sh",
        "description": "Useful test helper",
        "visibility": "private",
        "web_url": "https://gitlab.example.com/snippets/42",
        "raw_url": "https://gitlab.example.com/snippets/42/raw",
        "updated_at": "2026-01-01T12:00:00Z"
    });

    Mock::given(method("GET"))
        .and(path("/api/v4/snippets"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([snippet_json])))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v4/snippets/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&snippet_json))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v4/snippets/42/raw"))
        .respond_with(ResponseTemplate::new(200).set_body_string("echo 'hello world'"))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/v4/snippets"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&snippet_json))
        .mount(&mock_server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/api/v4/snippets/42"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let client = GitLabClient::new(mock_server.uri(), "test-token").unwrap();

    let list = client.list_snippets(Some(5)).await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, 42);

    let snippet = client.get_snippet(42).await.unwrap();
    assert_eq!(snippet.title, "Example Script");

    let raw = client.get_snippet_raw(42).await.unwrap();
    assert_eq!(raw, "echo 'hello world'");

    let created = client
        .create_snippet(
            "Example Script",
            "test.sh",
            "echo 'hello world'",
            "private",
            Some("Useful test helper"),
        )
        .await
        .unwrap();
    assert_eq!(created.id, 42);

    client.delete_snippet(42).await.unwrap();
}
