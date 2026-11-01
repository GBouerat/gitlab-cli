//! Command handler for `gitlab ci` / `gitlab pipeline`.

use crate::api::GitLabClient;
use crate::cli::ci::{
    CiArgs, CiCancelArgs, CiCommands, CiListArgs, CiRetryArgs, CiRunArgs, CiTraceArgs, CiViewArgs,
};
use crate::git::{get_current_branch, get_head_sha};
use crate::models::{Job, PipelineWithFailedJobs};
use crate::ui::table::{colored_cell, create_table, status_cell};
use colored::Colorize;
use comfy_table::{Cell, Color};
use std::env;
use std::io::IsTerminal;

/// Dispatches CI/CD pipeline subcommands.
///
/// # Arguments
///
/// * `args` - CI command arguments.
/// * `client` - Authenticated GitLab API client.
/// * `project` - Target project path (e.g. `owner/repo`).
/// * `json_mode` - Whether to output raw JSON.
///
/// # Errors
///
/// Returns an error if API communication or streaming fails.
pub async fn execute_ci(
    args: CiArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    match args.command {
        CiCommands::List(list_args) => handle_list(list_args, client, project, json_mode).await,
        CiCommands::View(view_args) => handle_view(view_args, client, project, json_mode).await,
        CiCommands::Trace(trace_args) => handle_trace(trace_args, client, project, json_mode).await,
        CiCommands::Run(run_args) => handle_run(run_args, client, project).await,
        CiCommands::Retry(retry_args) => handle_retry(retry_args, client, project).await,
        CiCommands::Cancel(cancel_args) => handle_cancel(cancel_args, client, project).await,
    }
}

async fn handle_list(
    args: CiListArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let current_dir = env::current_dir().unwrap_or_default();
    let default_sha = get_head_sha(&current_dir).ok();

    let sha_filter = args.sha.as_deref().or(if args.ref_name.is_none() {
        default_sha.as_deref()
    } else {
        None
    });

    let pipelines = client
        .list_pipelines(
            project,
            args.ref_name.as_deref(),
            sha_filter,
            args.status.as_deref(),
            Some(args.limit),
        )
        .await?;

    let job_futures = pipelines
        .iter()
        .map(|p| client.list_pipeline_jobs(project, p.id, None));
    let job_results = futures_util::future::join_all(job_futures).await;

    let pipelines_with_failed: Vec<PipelineWithFailedJobs> = pipelines
        .into_iter()
        .zip(job_results)
        .map(|(p, res)| {
            let jobs = res.unwrap_or_default();
            let failed_jobs: Vec<Job> = jobs.into_iter().filter(|j| j.status == "failed").collect();
            PipelineWithFailedJobs {
                pipeline: p,
                failed_jobs,
            }
        })
        .collect();

    if json_mode {
        println!("{}", serde_json::to_string_pretty(&pipelines_with_failed)?);
        return Ok(());
    }

    let mut table = create_table(&["ID", "Status", "Ref", "Commit SHA", "Source"]);
    for item in &pipelines_with_failed {
        let p = &item.pipeline;
        let sha_short = if p.sha.len() >= 8 {
            &p.sha[..8]
        } else {
            &p.sha
        };

        table.add_row(vec![
            colored_cell(p.id.to_string(), Color::Cyan, true),
            status_cell(&p.status),
            Cell::new(&p.ref_name),
            Cell::new(sha_short),
            Cell::new(p.source.as_deref().unwrap_or("-")),
        ]);
    }

    println!("{table}");

    // List failed jobs for any pipeline that has them
    for item in &pipelines_with_failed {
        if !item.failed_jobs.is_empty() {
            println!(
                "\n{} Failed jobs for Pipeline #{}:",
                "✗".red().bold(),
                item.pipeline.id
            );
            let mut failed_table = create_table(&["Job ID", "Stage", "Name", "Web URL"]);
            for job in &item.failed_jobs {
                failed_table.add_row(vec![
                    colored_cell(job.id.to_string(), Color::Yellow, false),
                    Cell::new(&job.stage),
                    colored_cell(&job.name, Color::Red, false),
                    Cell::new(job.web_url.as_deref().unwrap_or("-")),
                ]);
            }
            println!("{failed_table}");
        }
    }

    Ok(())
}

async fn handle_view(
    args: CiViewArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let pipeline = client.get_pipeline(project, args.id).await?;

    if args.web {
        if let Some(url) = &pipeline.web_url {
            println!("Opening pipeline in browser: {url}");
            crate::ui::open_in_browser(url);
            return Ok(());
        }
    }

    let jobs = client.list_pipeline_jobs(project, args.id, None).await?;

    if json_mode {
        let combined = serde_json::json!({
            "pipeline": pipeline,
            "jobs": jobs,
        });
        println!("{}", serde_json::to_string_pretty(&combined)?);
        return Ok(());
    }

    let mut pipe_table = create_table(&["Property", "Value"]);
    pipe_table.add_row(vec![
        Cell::new("Pipeline"),
        colored_cell(pipeline.id.to_string(), Color::Cyan, true),
    ]);
    pipe_table.add_row(vec![Cell::new("Status"), status_cell(&pipeline.status)]);
    pipe_table.add_row(vec![Cell::new("Ref"), Cell::new(&pipeline.ref_name)]);
    pipe_table.add_row(vec![Cell::new("Commit SHA"), Cell::new(&pipeline.sha)]);
    if let Some(dur) = pipeline.duration {
        pipe_table.add_row(vec![Cell::new("Duration"), Cell::new(format!("{dur:.0}s"))]);
    }
    if let Some(web) = &pipeline.web_url {
        pipe_table.add_row(vec![Cell::new("Web URL"), Cell::new(web)]);
    }
    println!("{pipe_table}\n");

    println!("{}", "Jobs:".bold());
    let mut jobs_table = create_table(&["Job ID", "Stage", "Name", "Status", "Duration"]);
    for job in &jobs {
        let dur_str = job
            .duration
            .map(|d| format!("{d:.0}s"))
            .unwrap_or_else(|| "-".to_string());

        jobs_table.add_row(vec![
            colored_cell(job.id.to_string(), Color::Cyan, false),
            Cell::new(&job.stage),
            Cell::new(&job.name),
            status_cell(&job.status),
            Cell::new(dur_str),
        ]);
    }
    println!("{jobs_table}");

    Ok(())
}

/// Strips ANSI color escape sequences and GitLab section markers from runner trace logs.
pub fn strip_ansi_and_sections(raw: &str) -> String {
    static SECTION_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"section_(?:start|end):\d+:[^\r\n]*").unwrap()
    });
    static ANSI_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])").unwrap()
    });

    let cleaned_sections = SECTION_RE.replace_all(raw, "");
    let cleaned_ansi = ANSI_RE.replace_all(&cleaned_sections, "");

    cleaned_ansi.replace('\r', "")
}

async fn handle_trace(
    args: CiTraceArgs,
    client: &GitLabClient,
    project: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let raw_trace = client.get_job_trace_text(project, args.job_id).await?;

    let should_strip = !std::io::stdout().is_terminal() || args.no_color;
    let processed = if should_strip {
        strip_ansi_and_sections(&raw_trace)
    } else {
        raw_trace
    };

    let final_output = if let Some(tail_count) = args.tail {
        let lines: Vec<&str> = processed.lines().collect();
        let start = lines.len().saturating_sub(tail_count);
        lines[start..].join("\n")
    } else {
        processed
    };

    if json_mode {
        let obj = serde_json::json!({
            "job_id": args.job_id,
            "trace": final_output,
        });
        println!("{}", serde_json::to_string_pretty(&obj)?);
        return Ok(());
    }

    if !final_output.is_empty() {
        println!("{final_output}");
    }

    Ok(())
}

async fn handle_run(args: CiRunArgs, client: &GitLabClient, project: &str) -> anyhow::Result<()> {
    let current_dir = env::current_dir()?;
    let ref_name = match args.ref_name {
        Some(r) => r,
        None => get_current_branch(&current_dir)?,
    };

    let pipeline = client.create_pipeline(project, &ref_name).await?;

    println!(
        "{} Triggered pipeline #{} on ref '{}'",
        "✓".green().bold(),
        pipeline.id,
        ref_name.cyan().bold()
    );
    if let Some(url) = &pipeline.web_url {
        println!("  URL: {url}");
    }

    Ok(())
}

async fn handle_retry(
    args: CiRetryArgs,
    client: &GitLabClient,
    project: &str,
) -> anyhow::Result<()> {
    let pipeline = client.retry_pipeline(project, args.id).await?;
    println!(
        "{} Retried pipeline #{} (Status: {})",
        "✓".green().bold(),
        pipeline.id,
        pipeline.status
    );
    Ok(())
}

async fn handle_cancel(
    args: CiCancelArgs,
    client: &GitLabClient,
    project: &str,
) -> anyhow::Result<()> {
    let pipeline = client.cancel_pipeline(project, args.id).await?;
    println!(
        "{} Canceled pipeline #{} (Status: {})",
        "✓".green().bold(),
        pipeline.id,
        pipeline.status
    );
    Ok(())
}
