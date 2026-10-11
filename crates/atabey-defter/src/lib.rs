//! Defter: Atabey's append-only ledger and app-log setup.
//!
//! * The **ledger** records every user action, IPC command, extension message and LLM call as one
//!   JSON object per line (R-21, Constitution Art. 5). See [`Ledger`] and [`LedgerEntry`].
//! * A **flow** ([`Flow`]) carries one [`CorrelationId`](atabey_types::CorrelationId) through
//!   everything a user-initiated action causes, in the ledger and on `tracing` spans.
//! * The **app log** is `tracing` output in daily rolling files (R-22). See [`init_app_log`].
//!
//! Timestamps and day folders are UTC (ADR-0010). Redaction (T-06) and retention (T-07) are not
//! part of this crate yet.

mod applog;
mod clock;
mod entry;
mod error;
mod ledger;

pub use applog::{AppLog, LogFormat, app_log, init_app_log};
pub use clock::{Clock, ManualClock, SystemClock};
pub use entry::{Actor, EntryDraft, EntryKind, LedgerEntry, Outcome, Timestamp};
pub use error::{DefterError, LogError};
pub use ledger::{Flow, Ledger};
