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
        "duration": 45.8
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
    assert_eq!(pipe.duration, Some(45.8));

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

#[tokio::test]
async fn test_mr_approvals_and_notes() {
    let mock_server = MockServer::start().await;

    let approvals_json = serde_json::json!({
        "id": 1,
        "iid": 12,
        "project_id": 100,
        "approvals_required": 2,
        "approvals_left": 1,
        "approved_by": [{
            "user": {
                "id": 5,
                "username": "reviewer",
                "name": "Code Reviewer"
            }
        }]
    });

    let note_json = serde_json::json!({
        "id": 1001,
        "body": "LGTM!",
        "author": {
            "id": 5,
            "username": "reviewer",
            "name": "Code Reviewer"
        },
        "created_at": "2026-01-01T12:00:00Z",
        "system": false
    });

    // Approve
    Mock::given(method("POST"))
        .and(path(
            "/api/v4/projects/group%2Frepo/merge_requests/12/approve",
        ))
        .respond_with(ResponseTemplate::new(201).set_body_json(&approvals_json))
        .mount(&mock_server)
        .await;

    // Unapprove
    Mock::given(method("POST"))
        .and(path(
            "/api/v4/projects/group%2Frepo/merge_requests/12/unapprove",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&mock_server)
        .await;

    // Approvals
    Mock::given(method("GET"))
        .and(path(
            "/api/v4/projects/group%2Frepo/merge_requests/12/approvals",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(&approvals_json))
        .mount(&mock_server)
        .await;

    // Notes list
    Mock::given(method("GET"))
        .and(path(
            "/api/v4/projects/group%2Frepo/merge_requests/12/notes",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([note_json])))
        .mount(&mock_server)
        .await;

    // Create note
    Mock::given(method("POST"))
        .and(path(
            "/api/v4/projects/group%2Frepo/merge_requests/12/notes",
        ))
        .respond_with(ResponseTemplate::new(201).set_body_json(&note_json))
        .mount(&mock_server)
        .await;

    let client = GitLabClient::new(mock_server.uri(), "test-token").unwrap();

    let approved = client.approve_mr("group/repo", 12).await.unwrap();
    assert_eq!(approved.approvals_left, 1);
    assert_eq!(approved.approved_by.len(), 1);

    client.unapprove_mr("group/repo", 12).await.unwrap();

    let state = client.get_mr_approvals("group/repo", 12).await.unwrap();
    assert_eq!(state.approvals_required, 2);

    let notes = client
        .list_mr_notes("group/repo", 12, Some(5))
        .await
        .unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].body, "LGTM!");

    let created_note = client
        .create_mr_note("group/repo", 12, "LGTM!")
        .await
        .unwrap();
    assert_eq!(created_note.id, 1001);
}

#[tokio::test]
async fn test_issue_notes() {
    let mock_server = MockServer::start().await;

    let note_json = serde_json::json!({
        "id": 2002,
        "body": "Looking into this issue.",
        "author": {
            "id": 1,
            "username": "developer",
            "name": "Dev"
        },
        "created_at": "2026-01-01T12:00:00Z",
        "system": false
    });

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/group%2Frepo/issues/5/notes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([note_json])))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/v4/projects/group%2Frepo/issues/5/notes"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&note_json))
        .mount(&mock_server)
        .await;

    let client = GitLabClient::new(mock_server.uri(), "test-token").unwrap();

    let notes = client
        .list_issue_notes("group/repo", 5, Some(5))
        .await
        .unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].id, 2002);

    let note = client
        .create_issue_note("group/repo", 5, "Looking into this issue.")
        .await
        .unwrap();
    assert_eq!(note.body, "Looking into this issue.");
}

#[tokio::test]
async fn test_tags_operations() {
    let mock_server = MockServer::start().await;

    let tag_json = serde_json::json!({
        "name": "v2.0.0",
        "message": "Major release v2.0",
        "protected": false,
        "commit": {
            "id": "abcdef1234567890",
            "short_id": "abcdef12",
            "title": "Release 2.0 commit"
        }
    });

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/group%2Frepo/repository/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([tag_json])))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/group%2Frepo/repository/tags/v2.0.0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&tag_json))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/v4/projects/group%2Frepo/repository/tags"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&tag_json))
        .mount(&mock_server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/api/v4/projects/group%2Frepo/repository/tags/v2.0.0"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let client = GitLabClient::new(mock_server.uri(), "test-token").unwrap();

    let list = client.list_tags("group/repo", Some(5)).await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "v2.0.0");

    let single = client.get_tag("group/repo", "v2.0.0").await.unwrap();
    assert_eq!(single.name, "v2.0.0");
    assert_eq!(single.message.as_deref(), Some("Major release v2.0"));

    let created = client
        .create_tag("group/repo", "v2.0.0", "main", Some("Major release v2.0"))
        .await
        .unwrap();
    assert_eq!(created.name, "v2.0.0");

    client.delete_tag("group/repo", "v2.0.0").await.unwrap();
}

#[tokio::test]
async fn test_environments_and_deployments() {
    let mock_server = MockServer::start().await;

    let env_json = serde_json::json!({
        "id": 1,
        "name": "production",
        "slug": "production",
        "external_url": "https://example.com",
        "state": "available"
    });

    let deploy_json = serde_json::json!({
        "id": 50,
        "iid": 1,
        "ref": "main",
        "sha": "1234567890abcdef",
        "status": "success",
        "created_at": "2026-01-01T12:00:00Z"
    });

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/group%2Frepo/environments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([env_json])))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/group%2Frepo/environments/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&env_json))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/v4/projects/group%2Frepo/environments"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&env_json))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/v4/projects/group%2Frepo/environments/1/stop"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "name": "production",
            "state": "stopped"
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/api/v4/projects/group%2Frepo/environments/1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/group%2Frepo/deployments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([deploy_json])))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/group%2Frepo/deployments/50"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&deploy_json))
        .mount(&mock_server)
        .await;

    let client = GitLabClient::new(mock_server.uri(), "test-token").unwrap();

    let envs = client
        .list_environments("group/repo", Some(5))
        .await
        .unwrap();
    assert_eq!(envs.len(), 1);
    assert_eq!(envs[0].name, "production");

    let single_env = client.get_environment("group/repo", 1).await.unwrap();
    assert_eq!(single_env.id, 1);

    let created_env = client
        .create_environment("group/repo", "production", Some("https://example.com"))
        .await
        .unwrap();
    assert_eq!(created_env.name, "production");

    let stopped = client.stop_environment("group/repo", 1).await.unwrap();
    assert_eq!(stopped.state.as_deref(), Some("stopped"));

    client.delete_environment("group/repo", 1).await.unwrap();

    let deploys = client
        .list_deployments("group/repo", None, Some(5))
        .await
        .unwrap();
    assert_eq!(deploys.len(), 1);
    assert_eq!(deploys[0].status, "success");

    let deploy = client.get_deployment("group/repo", 50).await.unwrap();
    assert_eq!(deploy.id, 50);
}

#[tokio::test]
async fn test_milestones_and_epics() {
    let mock_server = MockServer::start().await;

    let milestone_json = serde_json::json!({
        "id": 10,
        "title": "v1.0 Milestone",
        "description": "Initial stable milestone",
        "state": "active",
        "due_date": "2026-12-31"
    });

    let epic_json = serde_json::json!({
        "id": 20,
        "iid": 1,
        "group_id": 100,
        "title": "Core Modernization",
        "description": "Modernize the platform",
        "state": "opened"
    });

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/group%2Frepo/milestones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([milestone_json])))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v4/groups/group/milestones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([milestone_json])))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/group%2Frepo/milestones/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&milestone_json))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/v4/projects/group%2Frepo/milestones"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&milestone_json))
        .mount(&mock_server)
        .await;

    Mock::given(method("PUT"))
        .and(path("/api/v4/projects/group%2Frepo/milestones/10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 10,
            "title": "v1.0 Milestone",
            "state": "closed"
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/api/v4/projects/group%2Frepo/milestones/10"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v4/groups/group/epics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([epic_json])))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v4/groups/group/epics/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&epic_json))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/v4/groups/group/epics"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&epic_json))
        .mount(&mock_server)
        .await;

    Mock::given(method("PUT"))
        .and(path("/api/v4/groups/group/epics/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 20,
            "iid": 1,
            "group_id": 100,
            "title": "Core Modernization",
            "state": "closed"
        })))
        .mount(&mock_server)
        .await;

    let client = GitLabClient::new(mock_server.uri(), "test-token").unwrap();

    let p_ms = client
        .list_project_milestones("group/repo", None, Some(5))
        .await
        .unwrap();
    assert_eq!(p_ms.len(), 1);

    let g_ms = client
        .list_group_milestones("group", None, Some(5))
        .await
        .unwrap();
    assert_eq!(g_ms.len(), 1);

    let ms = client.get_milestone("group/repo", 10).await.unwrap();
    assert_eq!(ms.title, "v1.0 Milestone");

    let created_ms = client
        .create_milestone(
            "group/repo",
            "v1.0 Milestone",
            Some("Initial stable milestone"),
            Some("2026-12-31"),
            None,
        )
        .await
        .unwrap();
    assert_eq!(created_ms.id, 10);

    let closed_ms = client.close_milestone("group/repo", 10).await.unwrap();
    assert_eq!(closed_ms.state.as_deref(), Some("closed"));

    client.delete_milestone("group/repo", 10).await.unwrap();

    let epics = client.list_epics("group", None, Some(5)).await.unwrap();
    assert_eq!(epics.len(), 1);

    let epic = client.get_epic("group", 1).await.unwrap();
    assert_eq!(epic.iid, 1);

    let created_epic = client
        .create_epic(
            "group",
            "Core Modernization",
            Some("Modernize the platform"),
        )
        .await
        .unwrap();
    assert_eq!(created_epic.iid, 1);

    let closed_epic = client.close_epic("group", 1).await.unwrap();
    assert_eq!(closed_epic.state.as_deref(), Some("closed"));
}

#[tokio::test]
async fn test_packages_and_registry() {
    let mock_server = MockServer::start().await;

    let pkg_json = serde_json::json!({
        "id": 1,
        "name": "my-pkg",
        "version": "1.0.0",
        "package_type": "npm",
        "status": "default"
    });

    let reg_repo_json = serde_json::json!({
        "id": 2,
        "name": "my-app",
        "path": "group/repo/my-app",
        "location": "registry.gitlab.com/group/repo/my-app",
        "tag_count": 3
    });

    let reg_tag_json = serde_json::json!({
        "name": "latest",
        "path": "group/repo/my-app:latest",
        "location": "registry.gitlab.com/group/repo/my-app:latest",
        "short_revision": "abcd1234"
    });

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/group%2Frepo/packages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([pkg_json])))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/group%2Frepo/packages/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&pkg_json))
        .mount(&mock_server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/api/v4/projects/group%2Frepo/packages/1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/group%2Frepo/registry/repositories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([reg_repo_json])))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path(
            "/api/v4/projects/group%2Frepo/registry/repositories/2/tags",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([reg_tag_json])))
        .mount(&mock_server)
        .await;

    Mock::given(method("DELETE"))
        .and(path(
            "/api/v4/projects/group%2Frepo/registry/repositories/2/tags/latest",
        ))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    Mock::given(method("DELETE"))
        .and(path(
            "/api/v4/projects/group%2Frepo/registry/repositories/2",
        ))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let client = GitLabClient::new(mock_server.uri(), "test-token").unwrap();

    let pkgs = client
        .list_packages("group/repo", None, Some(5))
        .await
        .unwrap();
    assert_eq!(pkgs.len(), 1);
    assert_eq!(pkgs[0].name, "my-pkg");

    let pkg = client.get_package("group/repo", 1).await.unwrap();
    assert_eq!(pkg.version, "1.0.0");

    client.delete_package("group/repo", 1).await.unwrap();

    let repos = client
        .list_registry_repositories("group/repo", Some(5))
        .await
        .unwrap();
    assert_eq!(repos.len(), 1);
    assert_eq!(repos[0].name, "my-app");

    let tags = client
        .list_registry_tags("group/repo", 2, Some(5))
        .await
        .unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name, "latest");

    client
        .delete_registry_tag("group/repo", 2, "latest")
        .await
        .unwrap();
    client
        .delete_registry_repository("group/repo", 2)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_job_token_header_authentication() {
    let mock_server = MockServer::start().await;

    let user_json = serde_json::json!({
        "id": 99,
        "username": "ci-runner",
        "name": "CI Runner",
        "state": "active"
    });

    Mock::given(method("GET"))
        .and(path("/api/v4/user"))
        .and(header("JOB-TOKEN", "jwt-job-token-xyz"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&user_json))
        .mount(&mock_server)
        .await;

    let client =
        GitLabClient::with_job_token(mock_server.uri(), "jwt-job-token-xyz", true).unwrap();
    let user = client.current_user().await.unwrap();
    assert_eq!(user.username, "ci-runner");
}

#[tokio::test]
async fn test_set_variable_validation_error_does_not_put() {
    let mock_server = MockServer::start().await;

    // Simulate POST returning 400 for invalid masked value (not 'already taken')
    Mock::given(method("POST"))
        .and(path("/api/v4/projects/group%2Frepo/variables"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "message": {"value": ["is invalid - must be at least 8 characters"]}
        })))
        .mount(&mock_server)
        .await;

    let client = GitLabClient::new(mock_server.uri(), "test-token").unwrap();

    let err = client
        .set_variable("group/repo", "SECRET", "short", true, false, None)
        .await
        .unwrap_err();

    match err {
        gitlab_cli::api::error::ApiError::GitLab { status, message } => {
            assert_eq!(status, reqwest::StatusCode::BAD_REQUEST);
            assert!(message.contains("is invalid"));
        }
        other => panic!("Expected GitLab ApiError, got {:?}", other),
    }
}
