//! Async HTTP mock server integration tests for GitLab REST API operations.

use gitlab_cli::api::GitLabClient;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use wiremock::matchers::{body_json, header, method, path, query_param};
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
        .list_mrs("owner/repo", Some("opened"), None, None, Some(10))
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
            ..Default::default()
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

#[tokio::test]
async fn test_mr_body_file_byte_for_byte() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mock_server = MockServer::start().await;

    // Body with backticks, $, quotes, and pipes
    let raw_content =
        "Here is `echo $FOO`, \"double quotes\", 'single quotes', and | pipes |.\nPreserved 100%.";

    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(raw_content.as_bytes()).unwrap();

    let note_json = serde_json::json!({
        "id": 888,
        "body": raw_content,
        "system": false,
        "resolvable": false,
        "resolved": false
    });

    Mock::given(method("POST"))
        .and(path("/api/v4/projects/team%2Frepo/merge_requests/42/notes"))
        .and(body_json(serde_json::json!({
            "body": raw_content
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(&note_json))
        .mount(&mock_server)
        .await;

    let client = GitLabClient::new(mock_server.uri(), "token").unwrap();

    // Read from file
    let body = gitlab_cli::commands::body::read_body(None, Some(tmp.path().to_str().unwrap()))
        .unwrap()
        .unwrap();

    assert_eq!(body, raw_content);

    let note = client.create_mr_note("team/repo", 42, &body).await.unwrap();
    assert_eq!(note.body, raw_content);
}

#[tokio::test]
async fn test_mr_diffs_and_patch() {
    let mock_server = MockServer::start().await;

    let diff_response = serde_json::json!([
        {
            "old_path": "src/main.rs",
            "new_path": "src/main.rs",
            "a_mode": "100644",
            "b_mode": "100644",
            "new_file": false,
            "renamed_file": false,
            "deleted_file": false,
            "diff": "@@ -1,3 +1,4 @@\n+use std::path::Path;\n fn main() {}"
        },
        {
            "old_path": "old.txt",
            "new_path": "new.txt",
            "a_mode": "100644",
            "b_mode": "100644",
            "new_file": false,
            "renamed_file": true,
            "deleted_file": false,
            "diff": ""
        }
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/team%2Frepo/merge_requests/12/diffs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&diff_response))
        .mount(&mock_server)
        .await;

    let client = GitLabClient::new(mock_server.uri(), "token").unwrap();
    let diffs = client.list_mr_diffs("team/repo", 12, None).await.unwrap();

    assert_eq!(diffs.len(), 2);
    assert_eq!(diffs[0].new_path, "src/main.rs");
    assert!(diffs[0].diff.contains("+use std::path::Path;"));
    assert!(diffs[1].renamed_file);
}

#[tokio::test]
async fn test_mr_discussions_and_inline_comment() {
    let mock_server = MockServer::start().await;

    let mr_json = serde_json::json!({
        "id": 100,
        "iid": 12,
        "project_id": 1,
        "title": "MR 12",
        "state": "opened",
        "source_branch": "feat",
        "target_branch": "main",
        "diff_refs": {
            "base_sha": "base123",
            "start_sha": "start456",
            "head_sha": "head789"
        }
    });

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/team%2Frepo/merge_requests/12"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&mr_json))
        .mount(&mock_server)
        .await;

    let disc_json = serde_json::json!({
        "id": "thread-abc-123",
        "individual_note": false,
        "notes": [
            {
                "id": 501,
                "body": "Inline review comment",
                "system": false,
                "resolvable": true,
                "resolved": false,
                "position": {
                    "base_sha": "base123",
                    "start_sha": "start456",
                    "head_sha": "head789",
                    "old_path": "src/lib.rs",
                    "new_path": "src/lib.rs",
                    "position_type": "text",
                    "new_line": 42
                }
            }
        ]
    });

    Mock::given(method("POST"))
        .and(path(
            "/api/v4/projects/team%2Frepo/merge_requests/12/discussions",
        ))
        .respond_with(ResponseTemplate::new(201).set_body_json(&disc_json))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path(
            "/api/v4/projects/team%2Frepo/merge_requests/12/discussions",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([disc_json])))
        .mount(&mock_server)
        .await;

    let reply_json = serde_json::json!({
        "id": 502,
        "body": "Looks good!",
        "system": false,
        "resolvable": true,
        "resolved": false
    });

    Mock::given(method("POST"))
        .and(path(
            "/api/v4/projects/team%2Frepo/merge_requests/12/discussions/thread-abc-123/notes",
        ))
        .respond_with(ResponseTemplate::new(201).set_body_json(&reply_json))
        .mount(&mock_server)
        .await;

    let resolved_disc = serde_json::json!({
        "id": "thread-abc-123",
        "individual_note": false,
        "notes": [
            {
                "id": 501,
                "body": "Inline review comment",
                "system": false,
                "resolvable": true,
                "resolved": true
            }
        ]
    });

    Mock::given(method("PUT"))
        .and(path(
            "/api/v4/projects/team%2Frepo/merge_requests/12/discussions/thread-abc-123",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(&resolved_disc))
        .mount(&mock_server)
        .await;

    let client = GitLabClient::new(mock_server.uri(), "token").unwrap();

    let mr = client.get_mr("team/repo", 12).await.unwrap();
    let diff_refs = mr.diff_refs.unwrap();
    assert_eq!(diff_refs.base_sha.as_deref(), Some("base123"));

    let pos = gitlab_cli::models::DiscussionPosition {
        base_sha: diff_refs.base_sha,
        start_sha: diff_refs.start_sha,
        head_sha: diff_refs.head_sha,
        old_path: Some("src/lib.rs".to_string()),
        new_path: Some("src/lib.rs".to_string()),
        position_type: Some("text".to_string()),
        new_line: Some(42),
        old_line: None,
    };

    let created_disc = client
        .create_mr_discussion("team/repo", 12, "Inline review comment", Some(&pos))
        .await
        .unwrap();
    assert_eq!(created_disc.id, "thread-abc-123");
    assert!(created_disc.is_resolvable());
    assert!(!created_disc.is_resolved());

    let discussions = client
        .list_mr_discussions("team/repo", 12, None)
        .await
        .unwrap();
    assert_eq!(discussions.len(), 1);

    let reply = client
        .create_discussion_note("team/repo", 12, "thread-abc-123", "Looks good!")
        .await
        .unwrap();
    assert_eq!(reply.id, 502);

    let resolved = client
        .resolve_mr_discussion("team/repo", 12, "thread-abc-123", true)
        .await
        .unwrap();
    assert!(resolved.is_resolved());
}

#[tokio::test]
async fn test_mr_note_edit_and_update() {
    let mock_server = MockServer::start().await;

    let updated_note = serde_json::json!({
        "id": 777,
        "body": "Edited body content",
        "system": false,
        "resolvable": false,
        "resolved": false
    });

    Mock::given(method("PUT"))
        .and(path(
            "/api/v4/projects/team%2Frepo/merge_requests/12/notes/777",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(&updated_note))
        .mount(&mock_server)
        .await;

    let user_alice = serde_json::json!([
        {
            "id": 333,
            "username": "alice",
            "name": "Alice Developer",
            "state": "active"
        }
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v4/users"))
        .and(query_param("username", "alice"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&user_alice))
        .mount(&mock_server)
        .await;

    let updated_mr = serde_json::json!({
        "id": 100,
        "iid": 12,
        "project_id": 1,
        "title": "Ready: New Title",
        "state": "opened",
        "source_branch": "feat",
        "target_branch": "main"
    });

    Mock::given(method("PUT"))
        .and(path("/api/v4/projects/team%2Frepo/merge_requests/12"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&updated_mr))
        .mount(&mock_server)
        .await;

    let client = GitLabClient::new(mock_server.uri(), "token").unwrap();

    let note = client
        .update_mr_note("team/repo", 12, 777, "Edited body content")
        .await
        .unwrap();
    assert_eq!(note.body, "Edited body content");

    let user = client.get_user_by_username("alice").await.unwrap().unwrap();
    assert_eq!(user.id, 333);

    let mr = client
        .update_mr(&gitlab_cli::api::UpdateMrParams {
            project_path: "team/repo",
            iid: 12,
            title: Some("Ready: New Title".to_string()),
            reviewer_ids: Some(vec![user.id]),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(mr.title, "Ready: New Title");
}

#[tokio::test]
async fn test_markdown_render_and_checkboxes() {
    let mock_server = MockServer::start().await;

    let render_response = serde_json::json!({
        "html": "<p>Rendered GFM with checkboxes</p>"
    });

    Mock::given(method("POST"))
        .and(path("/api/v4/markdown"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&render_response))
        .mount(&mock_server)
        .await;

    let client = GitLabClient::new(mock_server.uri(), "token").unwrap();
    let text = "- [x] Done\n- [ ] Todo\n| [x] Table item |";
    let res = client
        .render_markdown(text, true, Some("team/repo"))
        .await
        .unwrap();
    assert_eq!(res.html, "<p>Rendered GFM with checkboxes</p>");

    let count = gitlab_cli::commands::markdown::count_checkboxes(text);
    assert_eq!(count.checked, 2);
    assert_eq!(count.unchecked, 1);
    assert_eq!(count.total, 3);
}

#[tokio::test]
async fn test_ci_list_with_failed_jobs_and_trace_tail() {
    let mock_server = MockServer::start().await;

    let pipelines_response = serde_json::json!([
        {
            "id": 999,
            "project_id": 1,
            "sha": "abcdef0123456789",
            "ref": "main",
            "status": "failed"
        }
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/team%2Frepo/pipelines"))
        .and(query_param("sha", "abcdef0123456789"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&pipelines_response))
        .mount(&mock_server)
        .await;

    let jobs_response = serde_json::json!([
        {
            "id": 555,
            "name": "cargo test",
            "stage": "test",
            "status": "failed",
            "web_url": "https://gitlab.com/team/repo/-/jobs/555"
        },
        {
            "id": 556,
            "name": "build",
            "stage": "build",
            "status": "success",
            "web_url": "https://gitlab.com/team/repo/-/jobs/556"
        }
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/team%2Frepo/pipelines/999/jobs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&jobs_response))
        .mount(&mock_server)
        .await;

    let raw_trace = "section_start:1680000000:step\r\x1b[0KStarting step\n\x1b[32mSuccess line\x1b[0m\nsection_end:1680000000:step\r\x1b[0KFinal line";

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/team%2Frepo/jobs/555/trace"))
        .respond_with(ResponseTemplate::new(200).set_body_string(raw_trace))
        .mount(&mock_server)
        .await;

    let client = GitLabClient::new(mock_server.uri(), "token").unwrap();

    let pipelines = client
        .list_pipelines("team/repo", None, Some("abcdef0123456789"), None, Some(10))
        .await
        .unwrap();
    assert_eq!(pipelines.len(), 1);

    let jobs = client
        .list_pipeline_jobs("team/repo", 999, None)
        .await
        .unwrap();
    let failed: Vec<_> = jobs.into_iter().filter(|j| j.status == "failed").collect();
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].name, "cargo test");

    let trace = client.get_job_trace_text("team/repo", 555).await.unwrap();
    let stripped = gitlab_cli::commands::ci::strip_ansi_and_sections(&trace);
    assert!(!stripped.contains("section_start"));
    assert!(!stripped.contains("\x1b[32m"));
    assert!(stripped.contains("Starting step"));
    assert!(stripped.contains("Final line"));
}

#[tokio::test]
async fn test_raw_file_from_blob_url() {
    let mock_server = MockServer::start().await;

    let file_content = "# Welcome to GitLab CLI\nfn main() {}";

    Mock::given(method("GET"))
        .and(path(
            "/api/v4/projects/org%2Fproj/repository/files/src%2Fmain.rs/raw",
        ))
        .and(query_param("ref", "main"))
        .respond_with(ResponseTemplate::new(200).set_body_string(file_content))
        .mount(&mock_server)
        .await;

    let client = GitLabClient::new(mock_server.uri(), "token").unwrap();

    let content = client
        .get_raw_file("org/proj", "src/main.rs", "main")
        .await
        .unwrap();
    assert_eq!(content, file_content);
}

#[tokio::test]
async fn test_mr_commits_for_relink() {
    let mock_server = MockServer::start().await;

    let commits_response = serde_json::json!([
        {
            "id": "1111222233334444555566667777888899990000",
            "short_id": "11112222",
            "title": "Fix memory leak in parser",
            "author_name": "Bob",
            "created_at": "2026-10-30T12:00:00Z"
        }
    ]);

    Mock::given(method("GET"))
        .and(path(
            "/api/v4/projects/team%2Frepo/merge_requests/12/commits",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(&commits_response))
        .mount(&mock_server)
        .await;

    let client = GitLabClient::new(mock_server.uri(), "token").unwrap();
    let commits = client.list_mr_commits("team/repo", 12).await.unwrap();

    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].id, "1111222233334444555566667777888899990000");
    assert_eq!(commits[0].title, "Fix memory leak in parser");
}
