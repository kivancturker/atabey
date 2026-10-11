//! Shared domain types for Atabey.
//!
//! Every type that crosses a boundary (Tauri IPC, Ulak messages, files on disk) is defined here
//! once, and TypeScript bindings and JSON Schemas are generated from it (Constitution Art. 1).
//! See `specs/S0-foundations.md` §5.4 and ADR-0009.

mod budget;
mod error;
mod ids;
mod permission;

pub use budget::CapKind;
pub use error::{AtabeyError, ErrorKind, IpcResult};
pub use ids::{BudgetRequestId, CorrelationId, ExtensionId, ProjectId, ProviderId};
pub use permission::{HostPattern, PanelSlot, Permission};

/// Re-exported so dependents name the same decimal type the errors use.
pub use rust_decimal::Decimal;
