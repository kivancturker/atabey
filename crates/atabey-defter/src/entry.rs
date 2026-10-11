use atabey_types::{CorrelationId, ExtensionId, ProviderId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use specta::Type;
use time::OffsetDateTime;
use uuid::Uuid;

/// A UTC instant, written as an RFC 3339 string (`2026-10-12T09:30:15.250Z`).
pub type Timestamp = OffsetDateTime;

/// One line of the ledger (spec §5.4, §5.8).
#[derive(Serialize, Deserialize, Type, JsonSchema, Debug, Clone, PartialEq)]
pub struct LedgerEntry {
    /// When the entry was written (UTC, RFC 3339).
    #[serde(with = "time::serde::rfc3339")]
    #[specta(type = String)]
    #[schemars(with = "String")]
    pub ts: Timestamp,
    /// This entry's own id (UUIDv7).
    pub id: Uuid,
    /// The user-initiated flow this entry belongs to.
    pub correlation: CorrelationId,
    /// Who did it.
    pub actor: Actor,
    /// What kind of event it was.
    pub kind: EntryKind,
    /// A short human-readable description. Never the full payload.
    pub summary: String,
    /// How it ended.
    pub outcome: Outcome,
    /// Structured detail about the event. Redacted before it is written (T-06).
    #[specta(type = specta_typescript::Unknown)]
    pub data: serde_json::Value,
}

/// Who caused an entry.
#[derive(Serialize, Deserialize, Type, JsonSchema, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Actor {
    /// The person using the app.
    User,
    /// Atabey itself.
    Core,
    /// An extension.
    Extension(ExtensionId),
    /// A coding agent or an observer, by name.
    Agent(String),
    /// A model provider.
    Provider(ProviderId),
}

/// What happened. The four sources R-21 lists, plus `system` for events of Atabey's own.
/// Adding a variant later is a compatible change for readers that ignore unknown kinds.
#[derive(Serialize, Deserialize, Type, JsonSchema, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EntryKind {
    /// Something the user did in the GUI, e.g. `open_project`.
    UserAction {
        /// The action's name.
        action: String,
    },
    /// A Tauri command, e.g. `project_open`.
    IpcCommand {
        /// The command's name.
        command: String,
    },
    /// A message an extension sent or received over Ulak.
    ExtensionMessage {
        /// The Ulak topic or command, e.g. `defter.query`.
        topic: String,
    },
    /// A call to a model provider.
    LlmCall {
        /// The model id, e.g. `anthropic/claude-sonnet-5`.
        model: String,
    },
    /// An event inside Atabey that no user action triggered, e.g. a retention run.
    System {
        /// The event's name.
        event: String,
    },
}

/// How an event ended.
#[derive(Serialize, Deserialize, Type, JsonSchema, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Outcome {
    /// It worked.
    Ok,
    /// It failed.
    Failed {
        /// The `AtabeyError` code, e.g. `timeout`.
        code: String,
    },
    /// A permission or budget check refused it.
    Denied {
        /// Why.
        reason: String,
    },
    /// It was stopped before it finished.
    Cancelled,
}

/// What a caller supplies to record an event. The ledger adds the timestamp, the entry id and the
/// correlation id.
#[derive(Debug, Clone, PartialEq)]
pub struct EntryDraft {
    /// Who did it.
    pub actor: Actor,
    /// What kind of event it was.
    pub kind: EntryKind,
    /// A short human-readable description.
    pub summary: String,
    /// How it ended. [`Outcome::Ok`] unless set.
    pub outcome: Outcome,
    /// Structured detail. `null` unless set.
    pub data: serde_json::Value,
}

impl EntryDraft {
    /// A successful event with no extra data.
    #[must_use]
    pub fn new(actor: Actor, kind: EntryKind, summary: impl Into<String>) -> Self {
        Self {
            actor,
            kind,
            summary: summary.into(),
            outcome: Outcome::Ok,
            data: serde_json::Value::Null,
        }
    }

    /// Set the outcome.
    #[must_use]
    pub fn outcome(mut self, outcome: Outcome) -> Self {
        self.outcome = outcome;
        self
    }

    /// Set the structured detail.
    #[must_use]
    pub fn data(mut self, data: serde_json::Value) -> Self {
        self.data = data;
        self
    }
}
