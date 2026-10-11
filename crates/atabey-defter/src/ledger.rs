use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use atabey_types::{CorrelationId, ProjectId, SessionId};
use time::{Date, UtcOffset};
use tracing::Span;
use uuid::Uuid;

use crate::clock::Clock;
use crate::entry::{EntryDraft, LedgerEntry};
use crate::error::DefterError;

/// The file currently open for appends, and the day it belongs to.
#[derive(Debug)]
struct OpenDay {
    date: Date,
    path: PathBuf,
    file: File,
}

#[derive(Debug)]
struct Inner {
    root: PathBuf,
    project: ProjectId,
    session: SessionId,
    clock: Arc<dyn Clock>,
    open: Mutex<Option<OpenDay>>,
}

/// Appends ledger entries to
/// `<root>/<project-id>/ledger/YYYY-MM-DD/<session-id>.jsonl` (spec §5.8).
///
/// Cheap to clone; all clones share one open file. Appends are serialized by a lock and each is
/// one `write` of a complete line, so lines never interleave. The day folder follows the injected
/// clock, so it rolls over at UTC midnight while the app keeps running.
#[derive(Debug, Clone)]
pub struct Ledger {
    inner: Arc<Inner>,
}

impl Ledger {
    /// A ledger for one project and session. Nothing is created on disk until the first append.
    /// `root` is `~/.atabey/defter` in production and a temp dir in tests.
    #[must_use]
    pub fn new(
        root: impl Into<PathBuf>,
        project: ProjectId,
        session: SessionId,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                root: root.into(),
                project,
                session,
                clock,
                open: Mutex::new(None),
            }),
        }
    }

    /// The ledger file this session writes on `date`.
    #[must_use]
    pub fn path_for(&self, date: Date) -> PathBuf {
        let inner = &self.inner;
        inner
            .root
            .join(inner.project.to_string())
            .join("ledger")
            .join(day_folder(date))
            .join(format!("{}.jsonl", inner.session))
    }

    /// Start a new user-initiated flow with a fresh correlation id.
    #[must_use]
    pub fn begin_flow(&self) -> Flow {
        self.continue_flow(CorrelationId::new())
    }

    /// Continue a flow that already has a correlation id (e.g. one that came in with an IPC call).
    #[must_use]
    pub fn continue_flow(&self, correlation: CorrelationId) -> Flow {
        Flow {
            ledger: self.clone(),
            correlation,
            span: tracing::info_span!("flow", correlation_id = %correlation),
        }
    }

    /// Record an event under `correlation`. Returns the entry as written.
    ///
    /// # Errors
    /// Returns [`DefterError`] if the entry can't be encoded or the file can't be written.
    pub fn append(
        &self,
        correlation: CorrelationId,
        draft: EntryDraft,
    ) -> Result<LedgerEntry, DefterError> {
        let ts = self.inner.clock.now().to_offset(UtcOffset::UTC);
        let entry = LedgerEntry {
            ts,
            id: Uuid::now_v7(),
            correlation,
            actor: draft.actor,
            kind: draft.kind,
            summary: draft.summary,
            outcome: draft.outcome,
            data: draft.data,
        };
        self.commit(entry)
    }

    /// The single choke point every ledger write goes through.
    ///
    /// T-06 (R-23) inserts the redactor here, on `entry`, before it is serialized, so no caller
    /// has to change.
    fn commit(&self, entry: LedgerEntry) -> Result<LedgerEntry, DefterError> {
        let mut line = serde_json::to_vec(&entry)?;
        line.push(b'\n');

        let date = entry.ts.date();
        // The guarded state is only an open file handle, so a poisoned lock is safe to reuse.
        let mut open = self
            .inner
            .open
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let day = match open.take() {
            Some(day) if day.date == date => day,
            _ => self.open_day(date)?,
        };
        let day = open.insert(day);
        day.file
            .write_all(&line)
            .map_err(|source| DefterError::Io {
                path: day.path.clone(),
                source,
            })?;
        Ok(entry)
    }

    fn open_day(&self, date: Date) -> Result<OpenDay, DefterError> {
        let path = self.path_for(date);
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|source| DefterError::Io {
                path: dir.to_path_buf(),
                source,
            })?;
        }
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|source| DefterError::Io {
                path: path.clone(),
                source,
            })?;
        Ok(OpenDay { date, path, file })
    }

    /// The root folder this ledger writes under.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.inner.root
    }
}

/// `YYYY-MM-DD`, the folder name of a day.
pub(crate) fn day_folder(date: Date) -> String {
    format!(
        "{:04}-{:02}-{:02}",
        date.year(),
        u8::from(date.month()),
        date.day()
    )
}

/// One user-initiated flow: a correlation id plus the `tracing` span that carries it.
///
/// Every entry recorded through the flow shares its correlation id. Enter [`Flow::span`] (or
/// `.instrument(flow.span().clone())` in async code) so app-log lines carry the same id as a
/// `correlation_id` span field.
#[derive(Debug, Clone)]
pub struct Flow {
    ledger: Ledger,
    correlation: CorrelationId,
    span: Span,
}

impl Flow {
    /// This flow's correlation id.
    #[must_use]
    pub fn correlation(&self) -> CorrelationId {
        self.correlation
    }

    /// The span that records `correlation_id` on every log line emitted inside it.
    #[must_use]
    pub fn span(&self) -> &Span {
        &self.span
    }

    /// Record an event in this flow.
    ///
    /// # Errors
    /// Returns [`DefterError`] if the entry can't be encoded or the file can't be written.
    pub fn record(&self, draft: EntryDraft) -> Result<LedgerEntry, DefterError> {
        self.ledger.append(self.correlation, draft)
    }
}
