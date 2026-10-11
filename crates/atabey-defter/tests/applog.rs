//! R-22: app logs go to daily rolling files.

mod common;

use std::fs;

use atabey_defter::{LogFormat, Redactor, app_log, init_app_log};
use common::*;
use tracing_subscriber::EnvFilter;

/// `atabey.YYYY-MM-DD.log`
fn is_log_file_name(name: &str) -> bool {
    let Some(date) = name
        .strip_prefix("atabey.")
        .and_then(|rest| rest.strip_suffix(".log"))
    else {
        return false;
    };
    let parts: Vec<_> = date.split('-').collect();
    matches!(parts.as_slice(), [y, m, d] if y.len() == 4 && m.len() == 2 && d.len() == 2)
        && date.chars().all(|c| c.is_ascii_digit() || c == '-')
}

fn log_files(dir: &std::path::Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut names = Vec::new();
    for entry in fs::read_dir(dir)? {
        names.push(entry?.file_name().to_string_lossy().into_owned());
    }
    Ok(names)
}

fn read_log(dir: &std::path::Path) -> Result<String, Box<dyn std::error::Error>> {
    let names = log_files(dir)?;
    assert_eq!(names.len(), 1, "{names:?}");
    assert!(is_log_file_name(&names[0]), "{}", names[0]);
    Ok(fs::read_to_string(dir.join(&names[0]))?)
}

#[test]
fn text_log_lands_in_a_daily_named_file() -> TestResult {
    let dir = tempfile::tempdir()?;
    let logs = dir.path().join("logs"); // created on demand
    let log = app_log(
        &logs,
        LogFormat::Text,
        EnvFilter::new("info"),
        Redactor::new(),
    )?;

    tracing::dispatcher::with_default(&log.dispatch, || {
        tracing::info!(project = "harness", "project opened");
    });
    drop(log); // flushes

    let text = read_log(&logs)?;
    assert!(text.contains("project opened"), "{text}");
    assert!(text.contains("harness"), "{text}");
    Ok(())
}

#[test]
fn json_log_lines_parse() -> TestResult {
    let dir = tempfile::tempdir()?;
    let log = app_log(
        dir.path(),
        LogFormat::Json,
        EnvFilter::new("info"),
        Redactor::new(),
    )?;

    tracing::dispatcher::with_default(&log.dispatch, || {
        tracing::warn!(provider = "openrouter", "budget cap reached");
        tracing::info!("retry scheduled");
    });
    drop(log);

    let text = read_log(dir.path())?;
    let lines: Vec<serde_json::Value> = text
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()?;
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0]["fields"]["message"], "budget cap reached");
    assert_eq!(lines[0]["level"], "WARN");
    Ok(())
}

#[test]
fn log_lines_inside_a_flow_carry_the_ledger_correlation_id() -> TestResult {
    let ledger_root = tempfile::tempdir()?;
    let logs = tempfile::tempdir()?;
    let ledger = ledger_in(ledger_root.path(), clock_at(MORNING));
    let log = app_log(
        logs.path(),
        LogFormat::Json,
        EnvFilter::new("info"),
        Redactor::new(),
    )?;

    // A span binds to the subscriber that is active when it is created, so the flow starts inside
    // `with_default` here. In the app the global subscriber is installed first.
    let entry = tracing::dispatcher::with_default(&log.dispatch, || {
        let flow = ledger.begin_flow();
        flow.span().in_scope(|| {
            tracing::info!("opening project");
            flow.record(open_project_draft())
        })
    })?;
    drop(log);

    let text = read_log(logs.path())?;
    let line: serde_json::Value = serde_json::from_str(text.lines().next().ok_or("empty log")?)?;
    assert_eq!(
        line["span"]["correlation_id"],
        entry.correlation.to_string(),
        "{text}"
    );
    Ok(())
}

#[test]
fn init_installs_the_global_subscriber_once() -> TestResult {
    let dir = tempfile::tempdir()?;
    let guard = init_app_log(dir.path(), LogFormat::Text, Redactor::new())?;
    tracing::error!("global subscriber installed");
    // A second install is a typed error, not a panic.
    let again = init_app_log(dir.path(), LogFormat::Text, Redactor::new());
    assert!(matches!(
        again,
        Err(atabey_defter::LogError::AlreadyInstalled)
    ));
    drop(guard);
    assert!(read_log(dir.path())?.contains("global subscriber installed"));
    Ok(())
}
