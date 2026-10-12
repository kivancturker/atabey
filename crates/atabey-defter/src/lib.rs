//! Defter: Atabey's append-only ledger and app-log setup.
//!
//! * The **ledger** records every user action, IPC command, extension message and LLM call as one
//!   JSON object per line (R-21, Constitution Art. 5). See [`Ledger`] and [`LedgerEntry`].
//! * A **flow** ([`Flow`]) carries one [`CorrelationId`](atabey_types::CorrelationId) through
//!   everything a user-initiated action causes, in the ledger and on `tracing` spans.
//! * The **app log** is `tracing` output in daily rolling files (R-22). See [`init_app_log`].
//!
//! * The **redactor** ([`Redactor`]) replaces every secret in the keychain, and anything that
//!   matches a secret pattern, with `«redacted:<kind>»` before a ledger entry, a log line or an
//!   IPC error is written (R-23, ADR-0011).
//!
//! * **Retention** ([`Retention`]) compacts ledger days older than 90 days into per-day
//!   [`DaySummary`] files kept forever and deletes app logs older than 14 days, at startup and
//!   daily (R-24, ADR-0012).
//!
//! Timestamps and day folders are UTC (ADR-0010).

mod applog;
mod clock;
mod entry;
mod error;
mod ledger;
mod redact;
mod retention;

pub use applog::{AppLog, LogFormat, app_log, init_app_log};
pub use clock::{Clock, ManualClock, SystemClock};
pub use entry::{Actor, EntryDraft, EntryKind, LedgerEntry, Outcome, Timestamp};
pub use error::{DefterError, LogError};
pub use ledger::{Flow, Ledger};
pub use redact::{KnownSecret, RedactError, RedactionKind, Redactor};
pub use retention::{
    ActorCount, CompactedDay, DaySummary, KindCount, OutcomeCounts, Retention, RetentionFailure,
    RetentionPolicy, RetentionReport, RetentionWorker, SessionSummary,
};
