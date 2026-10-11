use std::fmt;

use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{BudgetRequestId, CapKind, ExtensionId, Permission, ProviderId};

/// The result type of every Tauri command (R-5, AC-3).
pub type IpcResult<T> = Result<T, AtabeyError>;

/// Every error that crosses a boundary (Tauri IPC, Ulak).
///
/// On the wire it is one flat object: the `code` of its [`ErrorKind`], that kind's fields, and a
/// human-readable `message` written once in Rust (R-5, amendment A-2), e.g.
/// `{ "code": "not_found", "what": "README.md", "message": "Not found: README.md" }`.
#[derive(Serialize, Deserialize, Type, JsonSchema, Debug, Clone, PartialEq)]
pub struct AtabeyError {
    /// What went wrong, as data the GUI can match on.
    #[serde(flatten)]
    pub kind: ErrorKind,
    /// What went wrong, as text the GUI can show as is.
    pub message: String,
}

impl AtabeyError {
    /// What went wrong, as data.
    #[must_use]
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl From<ErrorKind> for AtabeyError {
    fn from(kind: ErrorKind) -> Self {
        let message = kind.to_string();
        Self { kind, message }
    }
}

impl fmt::Display for AtabeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for AtabeyError {}

/// The kinds of [`AtabeyError`], tagged by `code` on the wire.
///
/// Field names must never be `code` or `message`: both are taken by the flattened error.
#[derive(Serialize, Deserialize, Type, JsonSchema, Debug, Clone, PartialEq)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum ErrorKind {
    /// A file, project, theme or setting doesn't exist.
    NotFound {
        /// What was looked up, e.g. a project-relative path.
        what: String,
    },
    /// The caller sent a value the core can't accept.
    InvalidInput {
        /// The input field, e.g. `path`.
        field: String,
        /// Why it was rejected.
        reason: String,
    },
    /// An extension tried something its manifest doesn't allow.
    PermissionDenied {
        /// The extension that was refused.
        extension: ExtensionId,
        /// The permission it would need.
        needed: Permission,
    },
    /// A model request is paused because it would exceed a budget cap (Constitution Art. 7).
    BudgetPaused {
        /// The paused request, resolved with `budget_resolve`.
        request: BudgetRequestId,
        /// The amount in USD over the cap.
        needed_usd: Decimal,
        /// The cap it would exceed.
        cap: CapKind,
    },
    /// A model provider returned an error.
    Provider {
        /// The provider that failed.
        provider: ProviderId,
        /// The HTTP status, if the request got a response.
        status: Option<u16>,
        /// The provider's error, with secrets already redacted.
        detail: String,
    },
    /// An operation took longer than allowed.
    Timeout {
        /// The limit that was hit, in milliseconds (`u32`: TypeScript numbers can't hold every `u64`).
        after_ms: u32,
    },
    /// A bug or an unexpected failure. Full detail goes to Defter; only a short summary crosses
    /// the boundary.
    Internal {
        /// A short, GUI-safe summary.
        summary: String,
    },
}

/// The human-readable message. Written for the GUI, so no Rust type names or debug output.
impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound { what } => write!(f, "Not found: {what}"),
            Self::InvalidInput { field, reason } => write!(f, "Invalid {field}: {reason}"),
            Self::PermissionDenied { extension, needed } => {
                write!(f, "Extension {extension} needs the permission {needed}")
            }
            Self::BudgetPaused {
                needed_usd, cap, ..
            } => write!(
                f,
                "Paused: this request needs ${needed_usd} more than the {cap} budget allows"
            ),
            Self::Provider {
                provider,
                status: Some(status),
                detail,
            } => {
                write!(f, "{provider} failed (HTTP {status}): {detail}")
            }
            Self::Provider {
                provider,
                status: None,
                detail,
            } => {
                write!(f, "{provider} failed: {detail}")
            }
            Self::Timeout { after_ms } => write!(f, "Timed out after {after_ms} ms"),
            Self::Internal { summary } => write!(f, "Internal error: {summary}"),
        }
    }
}

impl std::error::Error for ErrorKind {}

impl fmt::Display for CapKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Daily => "daily",
            Self::Monthly => "monthly",
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::PanelSlot;

    fn one_of_each_kind() -> Result<Vec<(&'static str, ErrorKind)>, rust_decimal::Error> {
        Ok(vec![
            (
                "not_found",
                ErrorKind::NotFound {
                    what: "docs/brainstorm/README.md".into(),
                },
            ),
            (
                "invalid_input",
                ErrorKind::InvalidInput {
                    field: "path".into(),
                    reason: "must stay inside the project root".into(),
                },
            ),
            (
                "permission_denied",
                ErrorKind::PermissionDenied {
                    extension: "atabey.defter-viewer".into(),
                    needed: Permission::UiPanel(PanelSlot::MainArea),
                },
            ),
            (
                "budget_paused",
                ErrorKind::BudgetPaused {
                    request: BudgetRequestId(uuid::Uuid::from_u128(
                        0x0192_0f4e_7b3a_7c41_9d2e_5a8b_6c3d_1e0f,
                    )),
                    needed_usd: Decimal::from_str("0.42")?,
                    cap: CapKind::Daily,
                },
            ),
            (
                "provider",
                ErrorKind::Provider {
                    provider: "openrouter".into(),
                    status: Some(429),
                    detail: "Rate limit exceeded; retry after 20 s".into(),
                },
            ),
            ("timeout", ErrorKind::Timeout { after_ms: 30_000 }),
            (
                "internal",
                ErrorKind::Internal {
                    summary: "the project index is unavailable".into(),
                },
            ),
        ])
    }

    #[test]
    fn wire_shape_is_code_fields_and_message() -> Result<(), Box<dyn std::error::Error>> {
        for (code, kind) in one_of_each_kind()? {
            let error = AtabeyError::from(kind);
            let json = serde_json::to_value(&error)?;
            assert_eq!(json["code"], code);
            assert_eq!(json["message"], error.to_string());
            insta::assert_json_snapshot!(code, json);
        }
        Ok(())
    }

    #[test]
    fn round_trips_through_json() -> Result<(), Box<dyn std::error::Error>> {
        for (_, kind) in one_of_each_kind()? {
            let error = AtabeyError::from(kind);
            let back: AtabeyError = serde_json::from_str(&serde_json::to_string(&error)?)?;
            assert_eq!(back, error);
        }
        Ok(())
    }

    #[test]
    fn money_is_a_string_never_a_float() -> Result<(), Box<dyn std::error::Error>> {
        let error = AtabeyError::from(ErrorKind::BudgetPaused {
            request: BudgetRequestId::new(),
            needed_usd: Decimal::from_str("0.10")?,
            cap: CapKind::Monthly,
        });
        assert_eq!(serde_json::to_value(&error)?["needed_usd"], "0.10");
        Ok(())
    }

    #[test]
    fn provider_message_includes_status_only_when_present() {
        let with_status = AtabeyError::from(ErrorKind::Provider {
            provider: "openrouter".into(),
            status: Some(503),
            detail: "upstream unavailable".into(),
        });
        let without_status = AtabeyError::from(ErrorKind::Provider {
            provider: "openrouter".into(),
            status: None,
            detail: "connection refused".into(),
        });
        assert_eq!(
            with_status.message,
            "openrouter failed (HTTP 503): upstream unavailable"
        );
        assert_eq!(
            without_status.message,
            "openrouter failed: connection refused"
        );
    }
}
