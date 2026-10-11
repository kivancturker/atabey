//! Atabey desktop shell. Only this crate may depend on Tauri (specs/S0-foundations.md §5.2).

pub mod ipc;

/// Starts the Tauri application and blocks until the last window closes.
///
/// # Panics
/// Panics if the Tauri runtime fails to start, which is unrecoverable for a GUI app.
#[allow(clippy::expect_used)]
pub fn run() {
    let builder = ipc::specta_builder();
    tauri::Builder::default()
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the Atabey desktop app");
}
