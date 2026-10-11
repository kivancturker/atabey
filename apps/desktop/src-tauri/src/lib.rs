//! Atabey desktop shell. Only this crate may depend on Tauri (specs/S0-foundations.md §5.2).

pub mod ext_scheme;
pub mod ipc;
pub mod isolation_probe;

/// Starts the Tauri application and blocks until the last window closes.
///
/// # Panics
/// Panics if the Tauri runtime fails to start, which is unrecoverable for a GUI app.
#[allow(clippy::expect_used)]
pub fn run() {
    let specta = ipc::specta_builder();
    #[allow(unused_mut)]
    let mut context = tauri::generate_context!();
    let main_handler = specta.invoke_handler();
    let builder = tauri::Builder::default();

    // The isolation probe (T-04, ADR-0007) needs both the cargo feature and the CLI flag.
    #[cfg(feature = "isolation-probe")]
    let builder = if isolation_probe::requested() {
        isolation_probe::install(builder, &mut context, main_handler)
    } else {
        builder.invoke_handler(main_handler)
    };
    #[cfg(not(feature = "isolation-probe"))]
    let builder = builder.invoke_handler(main_handler);

    builder
        .setup(move |app| {
            specta.mount_events(app);
            Ok(())
        })
        .run(context)
        .expect("error while running the Atabey desktop app");
}
