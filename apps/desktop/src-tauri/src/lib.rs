//! Atabey desktop shell. Only this crate may depend on Tauri (specs/S0-foundations.md §5.2).

/// Starts the Tauri application and blocks until the last window closes.
///
/// # Panics
/// Panics if the Tauri runtime fails to start, which is unrecoverable for a GUI app.
#[allow(clippy::expect_used)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running the Atabey desktop app");
}
