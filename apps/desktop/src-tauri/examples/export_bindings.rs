//! Writes `packages/bindings/src/index.ts`. Run through `pnpm gen` (ADR-0009).

use std::process::ExitCode;

fn main() -> ExitCode {
    let path = atabey_desktop_lib::ipc::bindings_path();
    match atabey_desktop_lib::ipc::export_bindings(&path) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!(
                "failed to export TypeScript bindings to {}: {error}",
                path.display()
            );
            ExitCode::FAILURE
        }
    }
}
