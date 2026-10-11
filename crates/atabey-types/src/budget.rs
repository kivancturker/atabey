use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use specta::Type;

/// Which spending cap a paused request would exceed (Constitution Art. 7).
#[derive(Serialize, Deserialize, Type, JsonSchema, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CapKind {
    /// The per-day cap (resets at local midnight).
    Daily,
    /// The per-calendar-month cap.
    Monthly,
}
