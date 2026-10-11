//! The wire form of a ledger entry is pinned so accidental format changes are caught.

mod common;

use atabey_defter::{Actor, EntryKind, LedgerEntry, Outcome};
use atabey_types::{CorrelationId, ExtensionId, ProviderId};
use common::*;
use serde_json::json;
use uuid::Uuid;

fn pinned_entry() -> Result<LedgerEntry, Box<dyn std::error::Error>> {
    Ok(LedgerEntry {
        ts: MORNING,
        id: Uuid::parse_str("01920f4f-1a2b-7c3d-8e4f-5a6b7c8d9e0f")?,
        correlation: CorrelationId(Uuid::parse_str("01920f4e-9d20-7b66-a1c4-3e5f7a9b1d28")?),
        actor: Actor::Provider(ProviderId::from("openrouter")),
        kind: EntryKind::LlmCall {
            model: "anthropic/claude-sonnet-5".into(),
        },
        summary: "OpenRouter chat completion for the spec review observer".into(),
        outcome: Outcome::Failed {
            code: "timeout".into(),
        },
        data: json!({ "prompt_tokens": 18432, "completion_tokens": 0 }),
    })
}

#[test]
fn ledger_entry_wire_form_is_stable() -> TestResult {
    let entry = pinned_entry()?;
    insta::with_settings!({ omit_expression => true }, {
        insta::assert_json_snapshot!(entry);
    });
    Ok(())
}

#[test]
fn actors_kinds_and_outcomes_have_readable_wire_forms() -> TestResult {
    insta::with_settings!({ omit_expression => true }, {
    insta::assert_json_snapshot!(json!({
        "actors": [
            Actor::User,
            Actor::Core,
            Actor::Extension(ExtensionId::from("atabey.defter-viewer")),
            Actor::Agent("claude-code".into()),
            Actor::Provider(ProviderId::from("openrouter")),
        ],
        "kinds": [
            EntryKind::UserAction { action: "open_project".into() },
            EntryKind::IpcCommand { command: "project_open".into() },
            EntryKind::ExtensionMessage { topic: "defter.query".into() },
            EntryKind::LlmCall { model: "anthropic/claude-sonnet-5".into() },
            EntryKind::System { event: "retention_run".into() },
        ],
        "outcomes": [
            Outcome::Ok,
            Outcome::Failed { code: "timeout".into() },
            Outcome::Denied { reason: "daily cap reached".into() },
            Outcome::Cancelled,
        ],
    }));
    });
    Ok(())
}

#[test]
fn types_export_to_typescript() -> TestResult {
    let types = specta::Types::default().register::<LedgerEntry>();
    let ts = specta_typescript::Typescript::default().export(&types, specta_serde::Format)?;
    assert!(ts.contains("LedgerEntry"), "{ts}");
    assert!(ts.contains("EntryKind"), "{ts}");
    assert!(ts.contains("data: unknown"), "{ts}");
    Ok(())
}
