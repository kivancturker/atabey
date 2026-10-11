use std::{env, path::Path, process::ExitCode};

/// The Common Controls v6 manifest every Tauri executable needs on Windows.
/// Same content as tauri-build's default (`tauri-build/src/windows-app-manifest.xml`).
const WINDOWS_MANIFEST_FILE: &str = "windows-app-manifest.xml";

fn main() -> ExitCode {
    let windows = if targets_windows_msvc() {
        embed_windows_manifest_in_every_executable();
        // The linker embeds the manifest now; a second copy in the resource file would clash.
        tauri_build::WindowsAttributes::new_without_app_manifest()
    } else {
        tauri_build::WindowsAttributes::new()
    };

    match tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("tauri-build failed: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn targets_windows_msvc() -> bool {
    let is = |var: &str, value: &str| env::var(var).is_ok_and(|actual| actual == value);
    is("CARGO_CFG_TARGET_OS", "windows") && is("CARGO_CFG_TARGET_ENV", "msvc")
}

/// tauri-build puts the manifest in a resource file that is linked into `[[bin]]` targets only,
/// so test and example executables start without it and abort with `STATUS_ENTRYPOINT_NOT_FOUND`
/// (0xc0000139). Passing it to the linker covers every executable. This is the workaround Tauri
/// uses for its own tests (`tauri/build.rs`, `embed_manifest_for_tests`).
fn embed_windows_manifest_in_every_executable() {
    let manifest = env::var("CARGO_MANIFEST_DIR")
        .map(|dir| Path::new(&dir).join(WINDOWS_MANIFEST_FILE))
        .unwrap_or_else(|_| WINDOWS_MANIFEST_FILE.into());
    println!("cargo:rerun-if-changed={}", manifest.display());
    println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", manifest.display());
}
