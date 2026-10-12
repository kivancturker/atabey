//! Generates the JSON Schemas in `schemas/` from Rust types (Constitution Art. 1, ADR-0009).
//!
//! TypeScript bindings are generated separately by `apps/desktop/src-tauri` (only that crate may
//! depend on Tauri); `pnpm gen` runs both.

use std::path::{Path, PathBuf};

use atabey_defter::{DaySummary, LedgerEntry};
use atabey_types::AtabeyError;
use schemars::{Schema, schema_for};

/// Every generated schema: file name and schema. Later tasks add themes, manifests, blocks and
/// Ulak contracts here.
#[must_use]
pub fn schemas() -> Vec<(&'static str, Schema)> {
    vec![
        // The error contract Ulak extensions receive (T-21).
        ("atabey-error.schema.json", schema_for!(AtabeyError)),
        // One line of the Defter ledger (R-21); readers of `.jsonl` files validate against it.
        ("ledger-entry.schema.json", schema_for!(LedgerEntry)),
        // A compacted ledger day in `<project>/summaries/` (R-24, ADR-0012).
        ("day-summary.schema.json", schema_for!(DaySummary)),
    ]
}

/// The committed `schemas/` directory at the workspace root.
#[must_use]
pub fn schemas_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../schemas")
}

/// A schema as committed: pretty JSON with a trailing newline.
///
/// # Errors
/// Returns an error if the schema can't be serialized.
pub fn render(schema: &Schema) -> serde_json::Result<String> {
    let mut text = serde_json::to_string_pretty(schema)?;
    text.push('\n');
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// R-4: the committed schemas match what the Rust types generate today.
    #[test]
    fn schemas_are_up_to_date() -> Result<(), Box<dyn std::error::Error>> {
        for (file, schema) in schemas() {
            let committed = std::fs::read_to_string(schemas_dir().join(file))?;
            assert!(
                render(&schema)? == committed,
                "schemas/{file} is out of date: run `pnpm gen` and commit the result"
            );
        }
        Ok(())
    }
}
