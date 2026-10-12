//! Retention (R-24, ADR-0012): session ledgers are kept raw for 90 days and then compacted into
//! one summary per project and day, kept forever; app logs are deleted after 14 days.

use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use atabey_types::{CorrelationId, ProjectId, SessionId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use specta::Type;
use time::{Date, Month, UtcOffset};
use uuid::Uuid;

use crate::clock::Clock;
use crate::entry::{Actor, EntryDraft, EntryKind, LedgerEntry, Outcome, Timestamp};
use crate::ledger::day_folder;
use crate::redact::Redactor;

/// The `schema_version` written into every [`DaySummary`].
const SUMMARY_SCHEMA_VERSION: u32 = 1;

/// How long things are kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetentionPolicy {
    /// A ledger day older than this many days is compacted. 90 by default (R-24).
    pub ledger_raw_days: u16,
    /// An app-log file older than this many days is deleted. 14 by default (R-24).
    pub app_log_days: u16,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            ledger_raw_days: 90,
            app_log_days: 14,
        }
    }
}

/// The on-disk form of `<defter root>/<project-id>/summaries/YYYY-MM-DD.json`: what a project's
/// ledger said on one UTC day, after the raw files are gone. Counts only; the entries' `summary`
/// text and `data` are never copied. Every list is sorted, so the file is stable.
#[derive(Serialize, Deserialize, Type, JsonSchema, Debug, Clone, PartialEq)]
pub struct DaySummary {
    /// Format version of this file. Currently 1.
    pub schema_version: u32,
    /// The project the entries belong to.
    pub project: ProjectId,
    /// The UTC day, `YYYY-MM-DD`.
    pub date: String,
    /// When the day was compacted (UTC, RFC 3339).
    #[serde(with = "time::serde::rfc3339")]
    #[specta(type = String)]
    #[schemars(with = "String")]
    pub compacted_at: Timestamp,
    /// How many entries could be read.
    pub entries: u64,
    /// How many distinct flows (correlation ids) those entries belong to.
    pub flows: u64,
    /// The earliest entry, if any entry could be read.
    #[serde(with = "time::serde::rfc3339::option")]
    #[specta(type = Option<String>)]
    #[schemars(with = "Option<String>")]
    pub first_ts: Option<Timestamp>,
    /// The latest entry, if any entry could be read.
    #[serde(with = "time::serde::rfc3339::option")]
    #[specta(type = Option<String>)]
    #[schemars(with = "Option<String>")]
    pub last_ts: Option<Timestamp>,
    /// One row per session file, sorted by session id.
    pub sessions: Vec<SessionSummary>,
    /// Entries per kind, sorted by the kind's JSON form.
    pub kinds: Vec<KindCount>,
    /// Entries per actor, sorted by the actor's JSON form.
    pub actors: Vec<ActorCount>,
    /// Entries per outcome.
    pub outcomes: OutcomeCounts,
    /// Lines that were not a valid ledger entry (cut off by a crash, or garbage).
    pub unreadable_lines: u64,
}

/// One session's share of a [`DaySummary`].
#[derive(Serialize, Deserialize, Type, JsonSchema, Debug, Clone, PartialEq)]
pub struct SessionSummary {
    /// The session.
    pub session: SessionId,
    /// How many entries could be read from its file.
    pub entries: u64,
    /// Its earliest entry.
    #[serde(with = "time::serde::rfc3339::option")]
    #[specta(type = Option<String>)]
    #[schemars(with = "Option<String>")]
    pub first_ts: Option<Timestamp>,
    /// Its latest entry.
    #[serde(with = "time::serde::rfc3339::option")]
    #[specta(type = Option<String>)]
    #[schemars(with = "Option<String>")]
    pub last_ts: Option<Timestamp>,
}

/// How many entries had one [`EntryKind`].
#[derive(Serialize, Deserialize, Type, JsonSchema, Debug, Clone, PartialEq)]
pub struct KindCount {
    /// The kind.
    pub kind: EntryKind,
    /// How many entries.
    pub count: u64,
}

/// How many entries one [`Actor`] caused.
#[derive(Serialize, Deserialize, Type, JsonSchema, Debug, Clone, PartialEq)]
pub struct ActorCount {
    /// The actor.
    pub actor: Actor,
    /// How many entries.
    pub count: u64,
}

/// Entries per [`Outcome`] (the failure code and denial reason are dropped).
#[derive(Serialize, Deserialize, Type, JsonSchema, Debug, Clone, Default, PartialEq, Eq)]
pub struct OutcomeCounts {
    /// [`Outcome::Ok`].
    pub ok: u64,
    /// [`Outcome::Failed`].
    pub failed: u64,
    /// [`Outcome::Denied`].
    pub denied: u64,
    /// [`Outcome::Cancelled`].
    pub cancelled: u64,
}

/// One ledger day that was compacted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactedDay {
    /// The project.
    pub project: ProjectId,
    /// The UTC day, `YYYY-MM-DD`.
    pub date: String,
    /// How many entries the summary counts.
    pub entries: u64,
}

/// Something retention could not do. The run goes on and the next run tries again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetentionFailure {
    /// The file or folder involved.
    pub path: PathBuf,
    /// What went wrong.
    pub reason: String,
}

impl fmt::Display for RetentionFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path.display(), self.reason)
    }
}

impl RetentionFailure {
    fn new(path: &Path, reason: impl fmt::Display) -> Self {
        Self {
            path: path.to_path_buf(),
            reason: reason.to_string(),
        }
    }
}

/// What one retention pass did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RetentionReport {
    /// Ledger days turned into summaries, sorted by project and date.
    pub compacted: Vec<CompactedDay>,
    /// App-log files deleted, sorted.
    pub deleted_logs: Vec<PathBuf>,
    /// Steps that failed. Everything else in the pass still ran.
    pub failures: Vec<RetentionFailure>,
}

impl RetentionReport {
    /// Nothing was compacted, deleted or failed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.compacted.is_empty() && self.deleted_logs.is_empty() && self.failures.is_empty()
    }

    /// The ledger entry that records this run: `system` / `retention_run`, counts only.
    /// T-10 records it with one `append`.
    #[must_use]
    pub fn to_draft(&self) -> EntryDraft {
        let compacted = self.compacted.len();
        let deleted = self.deleted_logs.len();
        let failed = self.failures.len();
        EntryDraft::new(
            Actor::Core,
            EntryKind::System {
                event: "retention_run".into(),
            },
            format!(
                "Retention: compacted {compacted} ledger day(s), deleted {deleted} app log file(s), {failed} failure(s)"
            ),
        )
        .data(json!({
            "ledger_days_compacted": compacted,
            "app_logs_deleted": deleted,
            "failures": failed,
        }))
    }
}

/// The retention job. Cheap to build; does nothing until [`run`](Self::run),
/// [`run_if_due`](Self::run_if_due) or [`spawn`](Self::spawn) is called.
#[derive(Debug)]
pub struct Retention {
    defter_root: PathBuf,
    log_dir: PathBuf,
    clock: Arc<dyn Clock>,
    redactor: Redactor,
    policy: RetentionPolicy,
    last_run: Mutex<Option<Date>>,
}

impl Retention {
    /// A job over every project under `defter_root` and the app logs in `log_dir`, with the
    /// default [`RetentionPolicy`]. `clock` is the same seam the ledger uses.
    #[must_use]
    pub fn new(
        defter_root: impl Into<PathBuf>,
        log_dir: impl Into<PathBuf>,
        clock: Arc<dyn Clock>,
        redactor: Redactor,
    ) -> Self {
        Self {
            defter_root: defter_root.into(),
            log_dir: log_dir.into(),
            clock,
            redactor,
            policy: RetentionPolicy::default(),
            last_run: Mutex::new(None),
        }
    }

    /// Use `policy` instead of the defaults.
    #[must_use]
    pub fn with_policy(mut self, policy: RetentionPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// One pass over all projects and the app-log folder. Never fails as a whole: a problem with
    /// one day or one file goes into [`RetentionReport::failures`], the rest continues, and the
    /// next pass tries again. A missing folder (fresh install) is not a problem.
    #[must_use]
    pub fn run(&self) -> RetentionReport {
        let today = self.today();
        let mut report = RetentionReport::default();
        self.compact_ledgers(today, &mut report);
        self.delete_old_logs(today, &mut report);
        if !report.is_empty() {
            tracing::info!(
                ledger_days_compacted = report.compacted.len(),
                app_logs_deleted = report.deleted_logs.len(),
                failures = report.failures.len(),
                "retention run finished"
            );
        }
        report
    }

    /// [`run`](Self::run), but only on the first call (app startup) and afterwards only when the
    /// clock's UTC date differs from the date of the last run. That is R-24's "at startup and
    /// daily" rule. Returns `None` when nothing was due.
    pub fn run_if_due(&self) -> Option<RetentionReport> {
        let today = self.today();
        let mut last = self.last_run.lock().unwrap_or_else(PoisonError::into_inner);
        if *last == Some(today) {
            return None;
        }
        *last = Some(today);
        Some(self.run())
    }

    /// Start a thread that calls [`run_if_due`](Self::run_if_due) at once and then every `tick`.
    /// Production uses one hour: checking the date on a short tick, instead of sleeping 24 hours,
    /// survives laptop sleep.
    ///
    /// # Errors
    /// Returns the OS error if the thread can't be started.
    pub fn spawn(self, tick: Duration) -> io::Result<RetentionWorker> {
        self.spawn_with(tick, |_| {})
    }

    /// Like [`spawn`](Self::spawn), and calls `on_report` with every run that did something, so
    /// the composition root can record `report.to_draft()` in the ledger.
    ///
    /// # Errors
    /// Returns the OS error if the thread can't be started.
    pub fn spawn_with(
        self,
        tick: Duration,
        mut on_report: impl FnMut(RetentionReport) + Send + 'static,
    ) -> io::Result<RetentionWorker> {
        let (stop, wake) = mpsc::channel::<()>();
        let handle = thread::Builder::new()
            .name("atabey-retention".into())
            .spawn(move || {
                loop {
                    if let Some(report) = self.run_if_due()
                        && !report.is_empty()
                    {
                        on_report(report);
                    }
                    // Nothing is ever sent: the only wake-up besides the tick is the sender
                    // being dropped, which means "stop".
                    if let Err(RecvTimeoutError::Disconnected) = wake.recv_timeout(tick) {
                        break;
                    }
                }
            })?;
        Ok(RetentionWorker {
            stop: Some(stop),
            handle: Some(handle),
        })
    }

    fn today(&self) -> Date {
        self.clock.now().to_offset(UtcOffset::UTC).date()
    }

    fn compact_ledgers(&self, today: Date, report: &mut RetentionReport) {
        for (project, project_dir) in list_projects(&self.defter_root, report) {
            let ledger_dir = project_dir.join("ledger");
            let mut days = Vec::new();
            for_each_entry(&ledger_dir, report, |name, path, kind| {
                if kind.is_dir()
                    && let Some(date) = name.to_str().and_then(parse_day)
                {
                    days.push((date, path));
                }
            });
            days.sort();
            for (date, path) in days {
                if age_days(today, date) <= i64::from(self.policy.ledger_raw_days) {
                    continue;
                }
                match self.compact_day(project, &project_dir, date, &path) {
                    Ok(Some(done)) => report.compacted.push(done),
                    Ok(None) => {}
                    Err(failure) => report.failures.push(failure),
                }
            }
        }
        report
            .compacted
            .sort_by(|a, b| (&a.project, &a.date).cmp(&(&b.project, &b.date)));
    }

    /// Summarize one day folder, write the summary, then delete the raw files (ADR-0012).
    fn compact_day(
        &self,
        project: ProjectId,
        project_dir: &Path,
        date: Date,
        day_dir: &Path,
    ) -> Result<Option<CompactedDay>, RetentionFailure> {
        let files = session_files(day_dir).map_err(|e| RetentionFailure::new(day_dir, e))?;
        if files.is_empty() {
            return Ok(None);
        }
        let date_text = day_folder(date);
        let summaries_dir = project_dir.join("summaries");
        let summary_path = summaries_dir.join(format!("{date_text}.json"));

        let mut summary = summarize(
            project,
            &date_text,
            self.clock.now().to_offset(UtcOffset::UTC),
            &files,
        )
        .map_err(|e| RetentionFailure::new(day_dir, e))?;

        // A crash while deleting can leave only some raw files next to a complete summary.
        // Recomputing from those would undercount, so a bigger existing summary wins.
        let existing = read_summary(&summary_path);
        let covered = |s: &DaySummary| s.entries + s.unreadable_lines;
        match existing {
            Some(old) if covered(&old) > covered(&summary) => summary = old,
            _ => self
                .write_summary(&summaries_dir, &summary_path, &summary)
                .map_err(|e| RetentionFailure::new(&summary_path, e))?,
        }

        for (_, file) in &files {
            fs::remove_file(file).map_err(|e| RetentionFailure::new(file, e))?;
        }
        // Foreign files we left alone keep the folder; that is not a failure.
        let empty = fs::read_dir(day_dir)
            .map(|mut entries| entries.next().is_none())
            .map_err(|e| RetentionFailure::new(day_dir, e))?;
        if empty {
            fs::remove_dir(day_dir).map_err(|e| RetentionFailure::new(day_dir, e))?;
        }
        Ok(Some(CompactedDay {
            project,
            date: date_text,
            entries: summary.entries,
        }))
    }

    /// Redact, write to a temp file, flush and sync it, then rename it over the final name, so
    /// the summary on disk is always complete.
    fn write_summary(
        &self,
        summaries_dir: &Path,
        summary_path: &Path,
        summary: &DaySummary,
    ) -> io::Result<()> {
        let mut value = serde_json::to_value(summary)?;
        // R-23: a secret may have reached the keychain since the entries were written.
        self.redactor.redact_json(&mut value);
        let mut bytes = serde_json::to_vec_pretty(&value)?;
        bytes.push(b'\n');

        fs::create_dir_all(summaries_dir)?;
        let temp = summary_path.with_extension("json.tmp");
        let mut file = File::create(&temp)?;
        file.write_all(&bytes)?;
        file.flush()?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temp, summary_path)
    }

    fn delete_old_logs(&self, today: Date, report: &mut RetentionReport) {
        let mut old = Vec::new();
        for_each_entry(&self.log_dir, report, |name, path, kind| {
            if kind.is_file()
                && let Some(date) = name.to_str().and_then(parse_log_name)
                && age_days(today, date) > i64::from(self.policy.app_log_days)
            {
                old.push(path);
            }
        });
        old.sort();
        for path in old {
            match fs::remove_file(&path) {
                Ok(()) => report.deleted_logs.push(path),
                Err(e) => report.failures.push(RetentionFailure::new(&path, e)),
            }
        }
    }
}

/// A running retention thread. Dropping it (or calling [`stop`](Self::stop)) wakes the thread
/// and waits for it to finish.
#[derive(Debug)]
pub struct RetentionWorker {
    stop: Option<mpsc::Sender<()>>,
    handle: Option<JoinHandle<()>>,
}

impl RetentionWorker {
    /// Stop the thread and wait for it. A pass that is already running finishes first.
    pub fn stop(self) {
        drop(self);
    }
}

impl Drop for RetentionWorker {
    fn drop(&mut self) {
        // Dropping the sender disconnects the channel, which wakes the thread out of its wait.
        drop(self.stop.take());
        if let Some(handle) = self.handle.take() {
            // A panic in the thread has nothing left to clean up.
            let _ = handle.join();
        }
    }
}

/// Whole days from `date` to `today`; negative for a future date.
fn age_days(today: Date, date: Date) -> i64 {
    (today - date).whole_days()
}

/// Parse exactly `YYYY-MM-DD` into a real calendar date.
fn parse_day(name: &str) -> Option<Date> {
    let mut parts = name.split('-');
    let year: i32 = parts.next()?.parse().ok()?;
    let month: u8 = parts.next()?.parse().ok()?;
    let day: u8 = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    let date = Date::from_calendar_date(year, Month::try_from(month).ok()?, day).ok()?;
    // Rejects "+2026-1-5" and similar spellings the numeric parse accepts.
    (day_folder(date) == name).then_some(date)
}

/// The date in `atabey.YYYY-MM-DD.log`.
fn parse_log_name(name: &str) -> Option<Date> {
    parse_day(name.strip_prefix("atabey.")?.strip_suffix(".log")?)
}

/// Visit the entries of `dir` that are not symlinks. A missing `dir` is fine; any other error is
/// recorded as a failure.
fn for_each_entry(
    dir: &Path,
    report: &mut RetentionReport,
    mut visit: impl FnMut(&std::ffi::OsStr, PathBuf, fs::FileType),
) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return,
        Err(e) => {
            report.failures.push(RetentionFailure::new(dir, e));
            return;
        }
    };
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                report.failures.push(RetentionFailure::new(dir, e));
                continue;
            }
        };
        // `DirEntry::file_type` does not follow symlinks, so a link reports as neither a file nor
        // a directory and is skipped.
        match entry.file_type() {
            Ok(kind) if !kind.is_symlink() => visit(&entry.file_name(), entry.path(), kind),
            Ok(_) => {}
            Err(e) => report
                .failures
                .push(RetentionFailure::new(&entry.path(), e)),
        }
    }
}

/// The project folders under `root`: real directories named by a UUID, sorted.
fn list_projects(root: &Path, report: &mut RetentionReport) -> Vec<(ProjectId, PathBuf)> {
    let mut projects = Vec::new();
    for_each_entry(root, report, |name, path, kind| {
        if kind.is_dir()
            && let Some(id) = name.to_str().and_then(|n| Uuid::parse_str(n).ok())
        {
            projects.push((ProjectId(id), path));
        }
    });
    projects.sort();
    projects
}

/// The `<session-id>.jsonl` files directly in a day folder, sorted by session. Symlinks and
/// differently named files are not ours and are skipped.
fn session_files(day_dir: &Path) -> io::Result<Vec<(SessionId, PathBuf)>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(day_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name();
        if let Some(name) = name.to_str()
            && let Some(stem) = name.strip_suffix(".jsonl")
            && let Ok(id) = Uuid::parse_str(stem)
        {
            files.push((SessionId(id), entry.path()));
        }
    }
    files.sort();
    Ok(files)
}

fn read_summary(path: &Path) -> Option<DaySummary> {
    serde_json::from_slice(&fs::read(path).ok()?).ok()
}

/// A running count of one session file, or of the whole day.
#[derive(Default)]
struct Tally {
    entries: u64,
    first: Option<Timestamp>,
    last: Option<Timestamp>,
}

impl Tally {
    fn add(&mut self, ts: Timestamp) {
        self.entries += 1;
        self.first = Some(self.first.map_or(ts, |first| first.min(ts)));
        self.last = Some(self.last.map_or(ts, |last| last.max(ts)));
    }
}

/// Read the files one line at a time and count. A line that isn't a ledger entry is counted in
/// `unreadable_lines`; an I/O error aborts, so nothing is deleted on a half-read day.
fn summarize(
    project: ProjectId,
    date: &str,
    compacted_at: Timestamp,
    files: &[(SessionId, PathBuf)],
) -> io::Result<DaySummary> {
    let mut day = Tally::default();
    let mut flows: HashSet<CorrelationId> = HashSet::new();
    let mut sessions = Vec::with_capacity(files.len());
    let mut kinds: BTreeMap<String, (EntryKind, u64)> = BTreeMap::new();
    let mut actors: BTreeMap<String, (Actor, u64)> = BTreeMap::new();
    let mut outcomes = OutcomeCounts::default();
    let mut unreadable_lines = 0;

    for (session, path) in files {
        let mut reader = BufReader::new(File::open(path)?);
        let mut line = Vec::new();
        let mut span = Tally::default();
        loop {
            line.clear();
            if reader.read_until(b'\n', &mut line)? == 0 {
                break;
            }
            let text = line.trim_ascii();
            if text.is_empty() {
                continue;
            }
            let Ok(entry) = serde_json::from_slice::<LedgerEntry>(text) else {
                unreadable_lines += 1;
                continue;
            };
            span.add(entry.ts);
            day.add(entry.ts);
            flows.insert(entry.correlation);
            // The JSON form is the sort key: Actor and EntryKind have no total order of their own.
            kinds
                .entry(serde_json::to_string(&entry.kind)?)
                .or_insert_with(|| (entry.kind.clone(), 0))
                .1 += 1;
            actors
                .entry(serde_json::to_string(&entry.actor)?)
                .or_insert_with(|| (entry.actor.clone(), 0))
                .1 += 1;
            match entry.outcome {
                Outcome::Ok => outcomes.ok += 1,
                Outcome::Failed { .. } => outcomes.failed += 1,
                Outcome::Denied { .. } => outcomes.denied += 1,
                Outcome::Cancelled => outcomes.cancelled += 1,
            }
        }
        sessions.push(SessionSummary {
            session: *session,
            entries: span.entries,
            first_ts: span.first,
            last_ts: span.last,
        });
    }

    Ok(DaySummary {
        schema_version: SUMMARY_SCHEMA_VERSION,
        project,
        date: date.to_owned(),
        compacted_at,
        entries: day.entries,
        flows: flows.len() as u64,
        first_ts: day.first,
        last_ts: day.last,
        sessions,
        kinds: kinds
            .into_values()
            .map(|(kind, count)| KindCount { kind, count })
            .collect(),
        actors: actors
            .into_values()
            .map(|(actor, count)| ActorCount { actor, count })
            .collect(),
        outcomes,
        unreadable_lines,
    })
}
