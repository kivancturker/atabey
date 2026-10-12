//! R-24 and AC-12: ledger compaction and app-log retention, driven by an injected clock.

mod common;

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration as StdDuration;

use atabey_defter::{
    Actor, DaySummary, EntryDraft, EntryKind, Ledger, ManualClock, Outcome, Redactor, Retention,
    RetentionPolicy, RetentionReport,
};
use atabey_types::{ProjectId, SessionId};
use common::*;
use time::macros::datetime;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

const TODAY: OffsetDateTime = MORNING;

fn days_ago(days: i64) -> OffsetDateTime {
    TODAY - Duration::days(days)
}

fn date_text(instant: OffsetDateTime) -> String {
    let date = instant.date();
    format!(
        "{:04}-{:02}-{:02}",
        date.year(),
        u8::from(date.month()),
        date.day()
    )
}

fn other_project() -> ProjectId {
    ProjectId(Uuid::from_u128(0x01920f50_1111_7222_8333_444455556666))
}

fn other_session() -> SessionId {
    SessionId(Uuid::from_u128(0x01920f4e_9a00_7b10_8c20_3d4e5f607182))
}

struct Fixture {
    root: tempfile::TempDir,
    logs: tempfile::TempDir,
    clock: Arc<ManualClock>,
}

impl Fixture {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            root: tempfile::tempdir()?,
            logs: tempfile::tempdir()?,
            clock: clock_at(TODAY),
        })
    }

    fn retention(&self) -> Retention {
        self.retention_with(Redactor::new())
    }

    fn retention_with(&self, redactor: Redactor) -> Retention {
        Retention::new(
            self.root.path(),
            self.logs.path(),
            self.clock.clone(),
            redactor,
        )
    }

    /// Write the three example entries through a real ledger as if it were `instant`.
    fn write_day(&self, instant: OffsetDateTime) -> TestResult {
        self.clock.set(instant);
        let ledger = ledger_in(self.root.path(), self.clock.clone());
        let flow = ledger.begin_flow();
        flow.record(open_project_draft())?;
        flow.record(extension_draft())?;
        ledger.begin_flow().record(openrouter_draft())?;
        self.clock.set(TODAY);
        Ok(())
    }

    fn day_dir(&self, project: &str, instant: OffsetDateTime) -> PathBuf {
        self.root
            .path()
            .join(project)
            .join("ledger")
            .join(date_text(instant))
    }

    fn summary_path(&self, project: &str, instant: OffsetDateTime) -> PathBuf {
        self.root
            .path()
            .join(project)
            .join("summaries")
            .join(format!("{}.json", date_text(instant)))
    }

    fn log_path(&self, instant: OffsetDateTime) -> PathBuf {
        self.logs
            .path()
            .join(format!("atabey.{}.log", date_text(instant)))
    }

    fn write_log(&self, instant: OffsetDateTime) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let path = self.log_path(instant);
        fs::write(
            &path,
            "2026-07-13T10:04:11.218Z  INFO atabey_core: project opened\n",
        )?;
        Ok(path)
    }
}

fn read_summary(path: &Path) -> Result<DaySummary, Box<dyn std::error::Error>> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

/// AC-12: a 91-day-old ledger is compacted into a summary; a 15-day-old app log is deleted.
#[test]
fn ac_12_ninety_one_day_old_ledger_is_compacted_and_fifteen_day_old_log_is_deleted() -> TestResult {
    let fx = Fixture::new()?;
    fx.write_day(days_ago(91))?;
    let log = fx.write_log(days_ago(15))?;
    assert!(fx.day_dir(PROJECT, days_ago(91)).is_dir());
    assert!(log.is_file());

    let report = fx.retention().run();

    assert!(report.failures.is_empty(), "{:?}", report.failures);
    assert!(
        !fx.day_dir(PROJECT, days_ago(91)).exists(),
        "raw day folder must be gone"
    );
    assert!(!log.exists(), "15-day-old log must be gone");

    let summary = read_summary(&fx.summary_path(PROJECT, days_ago(91)))?;
    assert_eq!(summary.schema_version, 1);
    assert_eq!(summary.project, project_id());
    assert_eq!(summary.date, date_text(days_ago(91)));
    assert_eq!(summary.entries, 3);
    assert_eq!(summary.flows, 2);
    assert_eq!(summary.outcomes.ok, 2);
    assert_eq!(summary.outcomes.failed, 1);
    assert_eq!(summary.unreadable_lines, 0);
    assert_eq!(summary.sessions.len(), 1);
    assert_eq!(summary.sessions[0].session, session_id());
    assert_eq!(summary.sessions[0].entries, 3);
    assert_eq!(summary.compacted_at, TODAY);
    assert_eq!(report.compacted.len(), 1);
    assert_eq!(report.compacted[0].entries, 3);
    assert_eq!(report.deleted_logs, vec![log]);
    Ok(())
}

#[test]
fn boundaries_ninety_days_and_fourteen_days_are_kept() -> TestResult {
    let fx = Fixture::new()?;
    fx.write_day(days_ago(90))?;
    fx.write_day(TODAY)?;
    let log_14 = fx.write_log(days_ago(14))?;
    let log_today = fx.write_log(TODAY)?;

    let report = fx.retention().run();

    assert!(report.is_empty(), "{report:?}");
    assert!(fx.day_dir(PROJECT, days_ago(90)).is_dir());
    assert!(fx.day_dir(PROJECT, TODAY).is_dir());
    assert!(!fx.summary_path(PROJECT, days_ago(90)).exists());
    assert!(log_14.is_file());
    assert!(log_today.is_file());
    Ok(())
}

#[test]
fn a_custom_policy_moves_the_boundaries() -> TestResult {
    let fx = Fixture::new()?;
    fx.write_day(days_ago(8))?;
    let log = fx.write_log(days_ago(3))?;

    let report = fx
        .retention()
        .with_policy(RetentionPolicy {
            ledger_raw_days: 7,
            app_log_days: 2,
        })
        .run();

    assert_eq!(report.compacted.len(), 1);
    assert_eq!(report.deleted_logs, vec![log]);
    Ok(())
}

#[test]
fn summary_wire_form_is_stable() -> TestResult {
    let fx = Fixture::new()?;
    let day = datetime!(2026-07-13 08:00:00 UTC);
    fx.clock.set(day);
    let morning = ledger_in(fx.root.path(), fx.clock.clone());
    let flow = morning.begin_flow();
    flow.record(open_project_draft())?;
    fx.clock.set(datetime!(2026-07-13 08:00:05.500 UTC));
    flow.record(extension_draft())?;
    fx.clock.set(datetime!(2026-07-13 08:12:41 UTC));
    morning.begin_flow().record(openrouter_draft())?;

    fx.clock.set(datetime!(2026-07-13 17:45:09 UTC));
    let evening = Ledger::new(
        fx.root.path(),
        project_id(),
        other_session(),
        fx.clock.clone(),
        Redactor::new(),
    );
    let flow = evening.begin_flow();
    flow.record(
        EntryDraft::new(
            Actor::Agent("claude-code".into()),
            EntryKind::UserAction {
                action: "approve_spec".into(),
            },
            "Approve the S0 spec",
        )
        .outcome(Outcome::Denied {
            reason: "budget exhausted".into(),
        }),
    )?;
    flow.record(
        EntryDraft::new(
            Actor::User,
            EntryKind::UserAction {
                action: "approve_spec".into(),
            },
            "Cancel",
        )
        .outcome(Outcome::Cancelled),
    )?;
    evening.begin_flow().record(open_project_draft())?;
    // A write cut short by a crash leaves a partial last line.
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(evening.path_for(day.date()))?;
    file.write_all(br#"{"ts":"2026-07-13T17:50:00Z","id":"01920f4f-"#)?;
    drop(file);

    fx.clock.set(TODAY);
    let report = fx.retention().run();
    assert!(report.failures.is_empty(), "{:?}", report.failures);

    let summary = read_summary(&fx.summary_path(PROJECT, day))?;
    assert_eq!(summary.unreadable_lines, 1);
    insta::with_settings!({ omit_expression => true }, {
        insta::assert_json_snapshot!(summary);
    });
    Ok(())
}

#[test]
fn a_second_run_does_nothing() -> TestResult {
    let fx = Fixture::new()?;
    fx.write_day(days_ago(120))?;
    fx.write_log(days_ago(30))?;
    let retention = fx.retention();

    assert!(!retention.run().is_empty());
    let summary_before = fs::read(fx.summary_path(PROJECT, days_ago(120)))?;
    assert!(retention.run().is_empty());
    assert_eq!(
        fs::read(fx.summary_path(PROJECT, days_ago(120)))?,
        summary_before
    );
    Ok(())
}

#[test]
fn a_summary_next_to_its_raw_folder_is_recomputed_and_the_folder_removed() -> TestResult {
    let fx = Fixture::new()?;
    fx.write_day(days_ago(100))?;
    // Keep a copy of the raw file to put back, as if the process died before deleting the folder.
    let raw = fx
        .day_dir(PROJECT, days_ago(100))
        .join(format!("{SESSION}.jsonl"));
    let raw_copy = fs::read(&raw)?;
    let _ = fx.retention().run();
    let first = read_summary(&fx.summary_path(PROJECT, days_ago(100)))?;

    fs::create_dir_all(raw.parent().ok_or("no parent")?)?;
    fs::write(&raw, &raw_copy)?;
    let report = fx.retention().run();

    assert!(report.failures.is_empty());
    assert_eq!(report.compacted.len(), 1);
    assert!(!fx.day_dir(PROJECT, days_ago(100)).exists());
    assert_eq!(
        read_summary(&fx.summary_path(PROJECT, days_ago(100)))?,
        first
    );
    Ok(())
}

#[test]
fn a_summary_is_not_replaced_by_a_smaller_partial_leftover() -> TestResult {
    let fx = Fixture::new()?;
    fx.write_day(days_ago(100))?;
    let raw = fx
        .day_dir(PROJECT, days_ago(100))
        .join(format!("{SESSION}.jsonl"));
    let full = fs::read_to_string(&raw)?;
    let _ = fx.retention().run();
    let first = read_summary(&fx.summary_path(PROJECT, days_ago(100)))?;
    assert_eq!(first.entries, 3);

    // Only the first line survives, as after a crash halfway through deleting.
    let first_line = full.lines().next().ok_or("empty ledger")?;
    fs::create_dir_all(raw.parent().ok_or("no parent")?)?;
    fs::write(&raw, format!("{first_line}\n"))?;
    let _ = fx.retention().run();

    assert!(!fx.day_dir(PROJECT, days_ago(100)).exists());
    assert_eq!(
        read_summary(&fx.summary_path(PROJECT, days_ago(100)))?,
        first
    );
    Ok(())
}

#[test]
fn unrecognized_names_are_left_alone() -> TestResult {
    let fx = Fixture::new()?;
    fx.write_day(days_ago(5))?;
    let ledger_dir = fx.root.path().join(PROJECT).join("ledger");
    let notes = ledger_dir.join("notes");
    fs::create_dir_all(&notes)?;
    fs::write(notes.join("todo.md"), "Rotate the OpenRouter key\n")?;
    let impossible = ledger_dir.join("2026-13-45");
    fs::create_dir_all(&impossible)?;
    fs::write(impossible.join(format!("{SESSION}.jsonl")), "")?;
    let stray_file = ledger_dir.join(date_text(days_ago(200)));
    fs::write(&stray_file, "a file where a day folder should be\n")?;
    let future = ledger_dir.join("2027-01-01");
    fs::create_dir_all(&future)?;
    fs::write(future.join(format!("{SESSION}.jsonl")), "")?;
    let not_ours = fx.root.path().join("scratch");
    fs::create_dir_all(not_ours.join("ledger").join("2025-01-01"))?;
    fs::write(
        not_ours.join("ledger").join("2025-01-01").join("a.jsonl"),
        "",
    )?;

    let plain = fx.logs.path().join("atabey.log");
    fs::write(&plain, "x\n")?;
    let text = fx.logs.path().join("atabey.2026-01-01.txt");
    fs::write(&text, "x\n")?;
    let bad_date = fx.logs.path().join("atabey.2026-13-45.log");
    fs::write(&bad_date, "x\n")?;
    let future_log = fx.logs.path().join("atabey.2027-01-01.log");
    fs::write(&future_log, "x\n")?;
    let sub = fx.logs.path().join("atabey.2025-01-01.log");
    fs::create_dir_all(&sub)?;

    let report = fx.retention().run();

    assert!(report.is_empty(), "{report:?}");
    for kept in [
        &notes,
        &impossible,
        &stray_file,
        &future,
        &not_ours,
        &plain,
        &text,
        &bad_date,
        &future_log,
        &sub,
    ] {
        assert!(kept.exists(), "{} must be untouched", kept.display());
    }
    Ok(())
}

#[test]
fn foreign_files_in_a_day_folder_are_kept_and_not_summarized_again() -> TestResult {
    let fx = Fixture::new()?;
    fx.write_day(days_ago(100))?;
    let keep = fx.day_dir(PROJECT, days_ago(100)).join("README.txt");
    fs::write(&keep, "Exported for the audit\n")?;

    let report = fx.retention().run();
    assert_eq!(report.compacted.len(), 1);
    assert!(keep.is_file());
    let summary = fs::read(fx.summary_path(PROJECT, days_ago(100)))?;

    assert!(fx.retention().run().is_empty());
    assert_eq!(fs::read(fx.summary_path(PROJECT, days_ago(100)))?, summary);
    Ok(())
}

#[test]
fn two_projects_under_one_root_are_both_compacted() -> TestResult {
    let fx = Fixture::new()?;
    fx.write_day(days_ago(95))?;
    fx.clock.set(days_ago(95));
    let other = Ledger::new(
        fx.root.path(),
        other_project(),
        other_session(),
        fx.clock.clone(),
        Redactor::new(),
    );
    other.begin_flow().record(extension_draft())?;
    fx.clock.set(TODAY);

    let report = fx.retention().run();

    assert_eq!(report.compacted.len(), 2);
    let other_dir = other_project().to_string();
    assert_eq!(
        read_summary(&fx.summary_path(&other_dir, days_ago(95)))?.entries,
        1
    );
    assert_eq!(
        read_summary(&fx.summary_path(PROJECT, days_ago(95)))?.entries,
        3
    );
    Ok(())
}

#[test]
fn missing_roots_give_an_empty_report() -> TestResult {
    let base = tempfile::tempdir()?;
    let retention = Retention::new(
        base.path().join("defter"),
        base.path().join("logs"),
        clock_at(TODAY),
        Redactor::new(),
    );
    assert_eq!(retention.run(), RetentionReport::default());
    Ok(())
}

#[test]
fn run_if_due_runs_at_startup_then_once_per_utc_day() -> TestResult {
    let fx = Fixture::new()?;
    fx.clock.set(datetime!(2026-10-12 23:59:00 UTC));
    let retention = fx.retention();

    assert!(retention.run_if_due().is_some(), "startup run");
    assert!(retention.run_if_due().is_none(), "same UTC day");
    fx.clock.set(datetime!(2026-10-12 23:59:59.999 UTC));
    assert!(retention.run_if_due().is_none(), "still the same UTC day");
    fx.clock.set(datetime!(2026-10-13 00:00:01 UTC));
    assert!(retention.run_if_due().is_some(), "after UTC midnight");
    assert!(retention.run_if_due().is_none());
    Ok(())
}

#[test]
fn a_secret_stored_after_the_entries_were_written_is_not_in_the_summary() -> TestResult {
    let fx = Fixture::new()?;
    let keychain = KeychainMock::new();
    let runner = "hetzner-build-runner-7f3a91c2";
    fx.clock.set(days_ago(100));
    let ledger = Ledger::new(
        fx.root.path(),
        project_id(),
        session_id(),
        fx.clock.clone(),
        keychain.redactor(),
    );
    ledger.begin_flow().record(EntryDraft::new(
        Actor::Agent(runner.into()),
        EntryKind::UserAction {
            action: "start_build".into(),
        },
        "Start a build",
    ))?;
    fx.clock.set(TODAY);
    assert!(fs::read_to_string(ledger.path_for(days_ago(100).date()))?.contains(runner));

    keychain.set("hetzner", runner)?;
    let report = fx.retention_with(keychain.redactor()).run();

    assert!(report.failures.is_empty());
    let text = fs::read_to_string(fx.summary_path(PROJECT, days_ago(100)))?;
    assert!(!text.contains(runner), "{text}");
    assert!(text.contains("«redacted:keychain»"), "{text}");
    Ok(())
}

#[test]
fn the_report_becomes_a_core_system_entry_with_counts_only() -> TestResult {
    let fx = Fixture::new()?;
    fx.write_day(days_ago(91))?;
    fx.write_log(days_ago(20))?;
    fx.write_log(days_ago(21))?;

    let draft = fx.retention().run().to_draft();

    assert_eq!(draft.actor, Actor::Core);
    assert_eq!(
        draft.kind,
        EntryKind::System {
            event: "retention_run".into()
        }
    );
    assert_eq!(draft.outcome, Outcome::Ok);
    assert_eq!(
        draft.data,
        serde_json::json!({ "ledger_days_compacted": 1, "app_logs_deleted": 2, "failures": 0 })
    );
    Ok(())
}

#[test]
fn a_failure_is_reported_and_the_rest_of_the_pass_continues() -> TestResult {
    let fx = Fixture::new()?;
    fx.write_day(days_ago(100))?;
    fx.write_day(days_ago(101))?;
    // A folder where the temp file must go makes writing that day's summary fail.
    let blocker = fx
        .summary_path(PROJECT, days_ago(101))
        .with_extension("json.tmp");
    fs::create_dir_all(&blocker)?;
    let log = fx.write_log(days_ago(40))?;

    let report = fx.retention().run();

    assert_eq!(report.failures.len(), 1, "{:?}", report.failures);
    assert_eq!(
        report.failures[0].path,
        fx.summary_path(PROJECT, days_ago(101))
    );
    assert_eq!(report.compacted.len(), 1);
    assert!(fx.summary_path(PROJECT, days_ago(100)).is_file());
    assert!(!log.exists(), "the log pass still ran");
    assert!(
        fx.day_dir(PROJECT, days_ago(101)).is_dir(),
        "a day whose summary failed keeps its raw folder"
    );

    // The next run finishes it once the obstacle is gone.
    fs::remove_dir(&blocker)?;
    let retry = fx.retention().run();
    assert!(retry.failures.is_empty());
    assert_eq!(retry.compacted.len(), 1);
    assert!(!fx.day_dir(PROJECT, days_ago(101)).exists());
    Ok(())
}

#[cfg(unix)]
#[test]
fn symlinks_are_never_followed() -> TestResult {
    use std::os::unix::fs::symlink;

    let fx = Fixture::new()?;
    let outside = tempfile::tempdir()?;
    let target_day = outside.path().join("day");
    fs::create_dir_all(&target_day)?;
    fs::write(target_day.join(format!("{SESSION}.jsonl")), "")?;
    let target_log = outside.path().join("atabey.log");
    fs::write(&target_log, "x\n")?;

    let ledger_dir = fx.root.path().join(PROJECT).join("ledger");
    fs::create_dir_all(&ledger_dir)?;
    symlink(&target_day, ledger_dir.join(date_text(days_ago(200))))?;
    symlink(&target_log, fx.log_path(days_ago(200)))?;

    let report = fx.retention().run();

    assert!(report.is_empty(), "{report:?}");
    assert!(target_day.join(format!("{SESSION}.jsonl")).is_file());
    assert!(target_log.is_file());
    Ok(())
}

#[test]
fn the_worker_runs_at_startup_and_stops_cleanly() -> TestResult {
    let fx = Fixture::new()?;
    fx.write_day(days_ago(91))?;
    let (sender, receiver) = mpsc::channel();

    let worker = fx
        .retention()
        .spawn_with(StdDuration::from_millis(5), move |report| {
            let _ = sender.send(report);
        })?;

    let report = receiver.recv_timeout(StdDuration::from_secs(30))?;
    assert_eq!(report.compacted.len(), 1);
    assert!(fx.summary_path(PROJECT, days_ago(91)).is_file());
    worker.stop();
    // The sender died with the thread: nothing more arrives and the channel is closed.
    assert!(matches!(
        receiver.recv_timeout(StdDuration::from_secs(30)),
        Err(mpsc::RecvTimeoutError::Disconnected)
    ));
    Ok(())
}
