//! Checks that every registered command returns `Result<_, AtabeyError>` (R-5, AC-3).
//!
//! tauri-specta doesn't expose its collected command signatures directly, but it hands them to
//! every [`LanguageExt`] exporter. [`ErrorTypeAudit`] is an "exporter" that writes nothing and
//! inspects the signatures instead (ADR-0009). The `Result` detection mirrors tauri-specta's own
//! `extract_std_result` (`src/lang/js_ts.rs`, tag `v2.0.0-rc.25`).

use std::{path::Path, sync::Mutex};

use specta::{
    Types,
    datatype::{DataType, NamedReferenceType, Reference},
};
use tauri_specta::{BuilderConfiguration, LanguageExt};

/// Collects the commands whose error type isn't `atabey_types::AtabeyError`.
#[derive(Debug, Default)]
pub struct ErrorTypeAudit {
    violations: Mutex<Vec<String>>,
}

impl ErrorTypeAudit {
    /// The offending command names with the reason, sorted. Empty means every command is typed.
    #[must_use]
    pub fn violations(self) -> Vec<String> {
        let mut violations = self
            .violations
            .into_inner()
            .unwrap_or_else(|p| p.into_inner());
        violations.sort();
        violations
    }
}

impl LanguageExt for &ErrorTypeAudit {
    type Error = std::io::Error;

    fn export(self, cfg: &BuilderConfiguration, _path: &Path) -> Result<(), Self::Error> {
        let found: Vec<String> = cfg
            .commands
            .iter()
            .filter_map(|command| match command.result() {
                None => Some(format!("{}: returns nothing", command.name())),
                Some(result) => match std_result_error(result, &cfg.types) {
                    None => Some(format!("{}: doesn't return a Result", command.name())),
                    Some(error) if is_atabey_error(error, &cfg.types) => None,
                    Some(_) => Some(format!("{}: error type isn't AtabeyError", command.name())),
                },
            })
            .collect();
        self.violations
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .extend(found);
        Ok(())
    }
}

/// Runs the audit on a builder's commands and returns the violations (see
/// [`ErrorTypeAudit::violations`]).
///
/// # Errors
/// Never fails in practice: the audit writes nothing. The `io::Error` is required by
/// [`LanguageExt`].
pub fn audit_commands<R: tauri::Runtime>(
    builder: &tauri_specta::Builder<R>,
) -> std::io::Result<Vec<String>> {
    let audit = ErrorTypeAudit::default();
    // The path is unused: the audit writes nothing.
    builder.export(&audit, Path::new(""))?;
    Ok(audit.violations())
}

/// The `E` of a `std::result::Result<T, E>`, or `None` if `dt` isn't a std `Result`.
fn std_result_error<'a>(dt: &'a DataType, types: &Types) -> Option<&'a DataType> {
    let DataType::Reference(Reference::Named(reference)) = dt else {
        return None;
    };
    let named = types.get(reference)?;
    let is_std_result =
        named.name == "Result" && matches!(&*named.module_path, "std::result" | "core::result");
    match &reference.inner {
        NamedReferenceType::Reference { generics, .. } if is_std_result => {
            generics.get(1).map(|(_, error)| error)
        }
        _ => None,
    }
}

fn is_atabey_error(dt: &DataType, types: &Types) -> bool {
    let DataType::Reference(Reference::Named(reference)) = dt else {
        return false;
    };
    types.get(reference).is_some_and(|named| {
        named.name == "AtabeyError" && named.module_path.starts_with("atabey_types")
    })
}
