//! T-04 spike: does a sandboxed `atabey-ext://` iframe stay out of Atabey's internal API?
//!
//! The probe is a build of the real app that loads the real main window under the real CSP,
//! embeds extension iframes, lets them attack, and exits `0` (isolated), `1` (breach or a missing
//! functional check) or `2` (watchdog). Results and the decision: `adr/ADR-0007`.
//!
//! Only [`report`] (the verdict logic, pure) is always compiled so its tests run in the normal
//! test run. Everything that touches Tauri is behind the `isolation-probe` cargo feature **and**
//! the `--isolation-probe` command-line flag.

pub mod report;

#[cfg(feature = "isolation-probe")]
mod runtime;

#[cfg(feature = "isolation-probe")]
pub use runtime::{install, requested};
