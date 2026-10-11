//! Helpers shared by the integration tests.
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use atabey_defter::{Actor, EntryDraft, EntryKind, Ledger, LedgerEntry, ManualClock, Outcome};
use atabey_types::{ExtensionId, ProjectId, ProviderId, SessionId};
use serde_json::json;
use time::OffsetDateTime;
use time::macros::datetime;
use uuid::Uuid;

pub type TestResult = Result<(), Box<dyn std::error::Error>>;

pub const PROJECT: &str = "01920f4e-7b3a-7c41-9d2e-5a8b6c3d1e0f";
pub const SESSION: &str = "01920f4e-8c10-7a55-8f31-2b6d9e4a7c03";

pub fn project_id() -> ProjectId {
    ProjectId(Uuid::from_u128(0x01920f4e_7b3a_7c41_9d2e_5a8b6c3d1e0f))
}

pub fn session_id() -> SessionId {
    SessionId(Uuid::from_u128(0x01920f4e_8c10_7a55_8f31_2b6d9e4a7c03))
}

pub fn clock_at(instant: OffsetDateTime) -> Arc<ManualClock> {
    Arc::new(ManualClock::new(instant))
}

pub fn ledger_in(root: &Path, clock: Arc<ManualClock>) -> Ledger {
    Ledger::new(root, project_id(), session_id(), clock)
}

pub const MORNING: OffsetDateTime = datetime!(2026-10-12 09:30:15.250 UTC);

pub fn open_project_draft() -> EntryDraft {
    EntryDraft::new(
        Actor::User,
        EntryKind::IpcCommand {
            command: "project_open".into(),
        },
        "Open project /Users/kivancturker/Dev/harness",
    )
    .data(json!({ "path": "/Users/kivancturker/Dev/harness" }))
}

pub fn openrouter_draft() -> EntryDraft {
    EntryDraft::new(
        Actor::Provider(ProviderId::from("openrouter")),
        EntryKind::LlmCall {
            model: "anthropic/claude-sonnet-5".into(),
        },
        "OpenRouter chat completion for the spec review observer",
    )
    .outcome(Outcome::Failed {
        code: "timeout".into(),
    })
    .data(json!({ "prompt_tokens": 18432, "completion_tokens": 0 }))
}

pub fn extension_draft() -> EntryDraft {
    EntryDraft::new(
        Actor::Extension(ExtensionId::from("atabey.defter-viewer")),
        EntryKind::ExtensionMessage {
            topic: "defter.query".into(),
        },
        "Defter viewer asked for today's entries",
    )
}

/// Every ledger file under `root`, as `(day folder, file name)`, sorted.
pub fn ledger_files(root: &Path) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let ledger_dir = root.join(PROJECT).join("ledger");
    let mut found = Vec::new();
    for day in fs::read_dir(&ledger_dir)? {
        let day = day?;
        for file in fs::read_dir(day.path())? {
            found.push((
                day.file_name().to_string_lossy().into_owned(),
                file?.file_name().to_string_lossy().into_owned(),
            ));
        }
    }
    found.sort();
    Ok(found)
}

/// Parse a `.jsonl` file, requiring every line to be one complete entry.
pub fn read_entries(path: &PathBuf) -> Result<Vec<LedgerEntry>, Box<dyn std::error::Error>> {
    let text = fs::read_to_string(path)?;
    assert!(
        text.is_empty() || text.ends_with('\n'),
        "file must end with a newline"
    );
    text.lines()
        .map(|line| Ok(serde_json::from_str(line)?))
        .collect()
}
