//! R-21 and AC-10 (T-05 part): the session ledger.

mod common;

use std::thread;

use atabey_defter::{Clock, Ledger};
use common::*;
use time::Duration;
use time::macros::datetime;

#[test]
fn entry_round_trips_at_the_spec_path() -> TestResult {
    let root = tempfile::tempdir()?;
    let clock = clock_at(MORNING);
    let ledger = ledger_in(root.path(), clock.clone());

    let written = ledger.begin_flow().record(open_project_draft())?;

    let path = root
        .path()
        .join(PROJECT)
        .join("ledger")
        .join("2026-10-12")
        .join(format!("{SESSION}.jsonl"));
    assert!(path.is_file(), "expected {}", path.display());
    assert_eq!(read_entries(&path)?, vec![written.clone()]);
    assert_eq!(written.ts, clock.now());
    assert_eq!(written.id.get_version_num(), 7);
    Ok(())
}

#[test]
fn entries_in_one_flow_share_a_correlation_id() -> TestResult {
    let root = tempfile::tempdir()?;
    let ledger = ledger_in(root.path(), clock_at(MORNING));

    let opening = ledger.begin_flow();
    let first = opening.record(open_project_draft())?;
    let second = opening.record(extension_draft())?;
    let reviewing = ledger.begin_flow();
    let third = reviewing.record(openrouter_draft())?;

    assert_eq!(first.correlation, second.correlation);
    assert_eq!(first.correlation, opening.correlation());
    assert_ne!(first.correlation, third.correlation);
    assert_ne!(first.id, second.id);

    let on_disk = read_entries(&ledger.path_for(MORNING.date()))?;
    assert_eq!(on_disk, vec![first, second, third]);
    Ok(())
}

#[test]
fn a_continued_flow_keeps_the_given_correlation_id() -> TestResult {
    let root = tempfile::tempdir()?;
    let ledger = ledger_in(root.path(), clock_at(MORNING));
    let origin = ledger.begin_flow();

    let entry = ledger
        .continue_flow(origin.correlation())
        .record(extension_draft())?;

    assert_eq!(entry.correlation, origin.correlation());
    Ok(())
}

#[test]
fn concurrent_appends_write_only_complete_lines() -> TestResult {
    const THREADS: usize = 8;
    const PER_THREAD: usize = 60;
    let root = tempfile::tempdir()?;
    let ledger = ledger_in(root.path(), clock_at(MORNING));

    let handles: Vec<_> = (0..THREADS)
        .map(|_| {
            let ledger: Ledger = ledger.clone();
            thread::spawn(move || {
                let flow = ledger.begin_flow();
                for _ in 0..PER_THREAD {
                    flow.record(open_project_draft().data(serde_json::json!({
                        "padding": "x".repeat(2048),
                    })))?;
                }
                Ok::<_, atabey_defter::DefterError>(flow.correlation())
            })
        })
        .collect();
    let mut flows = Vec::new();
    for handle in handles {
        flows.push(handle.join().map_err(|_| "writer thread panicked")??);
    }

    let entries = read_entries(&ledger.path_for(MORNING.date()))?;
    assert_eq!(entries.len(), THREADS * PER_THREAD);
    let mut ids: Vec<_> = entries.iter().map(|entry| entry.id).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(
        ids.len(),
        THREADS * PER_THREAD,
        "no entry lost or duplicated"
    );
    for flow in flows {
        let count = entries.iter().filter(|e| e.correlation == flow).count();
        assert_eq!(count, PER_THREAD);
    }
    Ok(())
}

#[test]
fn the_day_folder_rolls_over_at_midnight_while_running() -> TestResult {
    let root = tempfile::tempdir()?;
    let clock = clock_at(datetime!(2026-10-12 23:59:58 UTC));
    let ledger = ledger_in(root.path(), clock.clone());
    let flow = ledger.begin_flow();

    let before = flow.record(open_project_draft())?;
    clock.advance(Duration::seconds(4));
    let after = flow.record(extension_draft())?;

    assert_eq!(
        ledger_files(root.path())?,
        vec![
            ("2026-10-12".to_owned(), format!("{SESSION}.jsonl")),
            ("2026-10-13".to_owned(), format!("{SESSION}.jsonl")),
        ]
    );
    assert_eq!(
        read_entries(&ledger.path_for(datetime!(2026-10-12 0:00 UTC).date()))?,
        vec![before]
    );
    assert_eq!(
        read_entries(&ledger.path_for(datetime!(2026-10-13 0:00 UTC).date()))?,
        vec![after]
    );
    Ok(())
}

#[test]
fn a_second_session_appends_to_its_own_file_only() -> TestResult {
    let root = tempfile::tempdir()?;
    let first = ledger_in(root.path(), clock_at(MORNING));
    let second = Ledger::new(
        root.path(),
        project_id(),
        atabey_types::SessionId::new(),
        clock_at(MORNING),
    );

    first.begin_flow().record(open_project_draft())?;
    second.begin_flow().record(openrouter_draft())?;

    assert_eq!(ledger_files(root.path())?.len(), 2);
    Ok(())
}

#[test]
fn write_errors_are_typed_not_panics() -> TestResult {
    let root = tempfile::tempdir()?;
    // A file where the project folder should be makes `create_dir_all` fail on every OS.
    std::fs::write(root.path().join(PROJECT), b"not a folder")?;
    let ledger = ledger_in(root.path(), clock_at(MORNING));

    let result = ledger.begin_flow().record(open_project_draft());

    assert!(matches!(result, Err(atabey_defter::DefterError::Io { .. })));
    Ok(())
}
