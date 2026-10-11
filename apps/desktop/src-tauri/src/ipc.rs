//! The typed IPC surface between the Rust core and the React GUI (R-4, R-5, ADR-0009).
//!
//! Every Tauri command and event is registered in [`specta_builder`]. TypeScript bindings are
//! generated from it into `packages/bindings` by `pnpm gen`; never edit them by hand.
//!
//! Every command returns [`atabey_types::IpcResult`], so its error is always an
//! [`atabey_types::AtabeyError`]; `tests::every_command_returns_atabey_error` enforces it (AC-3).

mod audit;

use std::path::{Path, PathBuf};

use atabey_types::AtabeyError;
use tauri_specta::{Builder, ErrorHandlingMode, collect_commands, collect_events};

pub use audit::{ErrorTypeAudit, audit_commands};

/// Where the generated TypeScript bindings are committed, relative to this crate.
pub const BINDINGS_PATH: &str = "../../../packages/bindings/src/index.ts";

/// The single registry of commands, events and extra types exposed to the GUI.
#[must_use]
pub fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        // Later tasks add commands here, e.g. `project_open` (T-09), always returning `IpcResult<T>`.
        .commands(collect_commands![])
        .events(collect_events![])
        .typ::<AtabeyError>()
        .error_handling(ErrorHandlingMode::Result)
}

/// The absolute path of the committed bindings file.
#[must_use]
pub fn bindings_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(BINDINGS_PATH)
}

/// Writes the TypeScript bindings to `path`.
///
/// # Errors
/// Returns the exporter's error if the bindings can't be rendered or written.
pub fn export_bindings(path: &Path) -> Result<(), specta_typescript::Error> {
    specta_builder().export(specta_typescript::Typescript::default(), path)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// AC-3: every command's error type is `AtabeyError`.
    #[test]
    fn every_command_returns_atabey_error() -> std::io::Result<()> {
        assert_eq!(audit_commands(&specta_builder())?, Vec::<String>::new());
        Ok(())
    }

    /// A command from before R-5, with a bare string error and no `Result`.
    mod untyped {
        #[tauri::command]
        #[specta::specta]
        pub fn settings_get_legacy() -> Result<String, String> {
            Err("settings.toml is missing".to_owned())
        }

        #[tauri::command]
        #[specta::specta]
        pub fn theme_active_name() -> String {
            "atabey-monokai".to_owned()
        }

        #[tauri::command]
        #[specta::specta]
        pub fn project_recent_count() -> atabey_types::IpcResult<u32> {
            Ok(3)
        }
    }

    /// The guard must actually catch untyped commands, or AC-3 proves nothing.
    #[test]
    fn audit_flags_commands_without_atabey_error() -> std::io::Result<()> {
        let builder = Builder::<tauri::Wry>::new().commands(collect_commands![
            untyped::settings_get_legacy,
            untyped::theme_active_name,
            untyped::project_recent_count,
        ]);
        assert_eq!(
            audit_commands(&builder)?,
            vec![
                "settings_get_legacy: error type isn't AtabeyError".to_owned(),
                "theme_active_name: doesn't return a Result".to_owned(),
            ]
        );
        Ok(())
    }

    /// R-4: the committed bindings match what the Rust types generate today.
    #[test]
    fn bindings_are_up_to_date() -> Result<(), Box<dyn std::error::Error>> {
        let fresh = std::env::temp_dir().join(format!("atabey-bindings-{}.ts", std::process::id()));
        export_bindings(&fresh)?;
        let generated = std::fs::read_to_string(&fresh)?;
        std::fs::remove_file(&fresh)?;
        let committed = std::fs::read_to_string(bindings_path())?;
        assert!(
            generated == committed,
            "packages/bindings is out of date: run `pnpm gen` and commit the result"
        );
        Ok(())
    }
}
