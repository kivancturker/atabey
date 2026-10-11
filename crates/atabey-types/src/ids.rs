use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

macro_rules! uuid_id {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(
            Serialize, Deserialize, Type, JsonSchema, Debug, Clone, Copy, PartialEq, Eq, Hash,
            PartialOrd, Ord,
        )]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            /// A new UUIDv7 id. v7 ids sort by creation time.
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

macro_rules! string_id {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(
            Serialize, Deserialize, Type, JsonSchema, Debug, Clone, PartialEq, Eq, Hash,
            PartialOrd, Ord,
        )]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl $name {
            /// The id as a string slice.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }
    };
}

uuid_id!(
    /// Identifies an opened project. Stored in `.atabey/project.toml`.
    ProjectId
);
uuid_id!(
    /// Ties together everything that happens in one user-initiated flow (Defter, logs, IPC).
    CorrelationId
);
uuid_id!(
    /// Identifies one app run. The Defter ledger writes one `<session-id>.jsonl` file per session.
    SessionId
);
uuid_id!(
    /// Identifies one paused model request waiting for the user's budget decision.
    BudgetRequestId
);
string_id!(
    /// An extension's reverse-DNS id, e.g. `atabey.defter-viewer`.
    ExtensionId
);
string_id!(
    /// A model provider's id, e.g. `openrouter`.
    ProviderId
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_ids_are_v7_and_sort_by_creation() {
        let first = CorrelationId::new();
        let second = CorrelationId::new();
        assert_eq!(first.0.get_version_num(), 7);
        assert!(first < second);
    }

    #[test]
    fn ids_are_plain_strings_on_the_wire() -> Result<(), serde_json::Error> {
        let extension = ExtensionId::from("atabey.defter-viewer");
        assert_eq!(
            serde_json::to_string(&extension)?,
            "\"atabey.defter-viewer\""
        );
        let project: ProjectId = serde_json::from_str("\"01920f4e-7b3a-7c41-9d2e-5a8b6c3d1e0f\"")?;
        assert_eq!(project.to_string(), "01920f4e-7b3a-7c41-9d2e-5a8b6c3d1e0f");
        Ok(())
    }
}
