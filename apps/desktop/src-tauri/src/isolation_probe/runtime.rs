//! The Tauri side of the isolation probe: scheme, commands, observers, watchdog.
//!
//! Compiled only with the `isolation-probe` feature and active only with `--isolation-probe`.
//! The probe commands are deliberately not part of [`crate::ipc::specta_builder`], so the
//! generated bindings and schemas do not change.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::io::Write as _;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use atabey_types::{AtabeyError, ErrorKind, ExtensionId, IpcResult, Permission};
use tauri::webview::{PageLoadEvent, PageLoadPayload};
use tauri::{Context, Listener, Manager, State, Webview, Wry, ipc::Invoke};

use super::report::{
    BridgeDecision, DEFTER_VIEWER, EXIT_WATCHDOG, GIT_PULSE, ISOLATED_HOST, LOCALHOST_HOST,
    Observed, ProbeReport, evaluate,
};
use crate::ext_scheme::{self, UiAssets};

const FLAG: &str = "--isolation-probe";
const BREACH_EVENT: &str = "atabey-probe-breach";
const WATCHDOG: Duration = Duration::from_secs(45);
const RUN_TIMEOUT_MS: u32 = 7_000;
const PROBE_COMMANDS: [&str; 5] = [
    "probe_canary",
    "probe_begin",
    "probe_bridge",
    "probe_finish",
    "probe_log",
];

const ATTACKS_JS: &str = include_str!("fixtures/shared/attacks.js");
const HOST_JS: &str = include_str!("fixtures/shared/host.js");

/// Whether this process was started with the probe flag.
#[must_use]
pub fn requested() -> bool {
    std::env::args().any(|arg| arg == FLAG)
}

/// The observations, shared between the commands, the event listener and the verdict.
#[derive(Default)]
struct ProbeState {
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    current: String,
    observed: BTreeMap<String, Observed>,
    started: bool,
}

impl ProbeState {
    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn with_current(&self, update: impl FnOnce(&mut Observed)) {
        let mut inner = self.lock();
        let variant = inner.current.clone();
        update(inner.observed.entry(variant).or_default());
    }
}

/// Fixture extensions, embedded so the probe needs no files at run time.
struct EmbeddedUi;

impl UiAssets for EmbeddedUi {
    fn get(&self, extension: &ExtensionId, path: &str) -> Option<Cow<'static, [u8]>> {
        let bytes: &'static [u8] = match (extension.as_str(), path) {
            (DEFTER_VIEWER, "ui/index.html") => {
                &include_bytes!("fixtures/atabey.defter-viewer/ui/index.html")[..]
            }
            (DEFTER_VIEWER, "ui/panel.css") => {
                &include_bytes!("fixtures/atabey.defter-viewer/ui/panel.css")[..]
            }
            (DEFTER_VIEWER, "ui/ledger-sample.json") => {
                &include_bytes!("fixtures/atabey.defter-viewer/ui/ledger-sample.json")[..]
            }
            (GIT_PULSE, "ui/index.html") => {
                &include_bytes!("fixtures/atabey.git-pulse/ui/index.html")[..]
            }
            (GIT_PULSE, "ui/panel.css") => {
                &include_bytes!("fixtures/atabey.git-pulse/ui/panel.css")[..]
            }
            (GIT_PULSE, "ui/ledger-sample.json") => {
                &include_bytes!("fixtures/atabey.git-pulse/ui/ledger-sample.json")[..]
            }
            (DEFTER_VIEWER | GIT_PULSE, "ui/panel.mjs") => {
                &include_bytes!("fixtures/shared/panel.mjs")[..]
            }
            (DEFTER_VIEWER | GIT_PULSE, "ui/attacks.js") => ATTACKS_JS.as_bytes(),
            _ => return None,
        };
        Some(Cow::Borrowed(bytes))
    }
}

/// What the fixture manifests declare (S0 spec §5.10).
fn declared(extension: &str) -> Vec<Permission> {
    match extension {
        DEFTER_VIEWER => vec![
            Permission::UlakCommand("defter.query".to_owned()),
            Permission::UlakSubscribe("defter.appended".to_owned()),
        ],
        _ => Vec::new(),
    }
}

/// Marks the variant that the following observations belong to.
#[tauri::command]
fn probe_begin(state: State<'_, ProbeState>, variant: String) -> IpcResult<()> {
    state.lock().current = variant;
    Ok(())
}

/// Progress lines from the orchestrator, for CI logs.
#[tauri::command]
fn probe_log(message: String) -> IpcResult<()> {
    eprintln!("probe: {message}");
    Ok(())
}

/// The target of every IPC attack. Reaching it means an iframe can call app commands.
#[tauri::command]
fn probe_canary(state: State<'_, ProbeState>, who: String) -> IpcResult<()> {
    eprintln!("probe: CANARY reached by {who}");
    state.with_current(|o| o.canary_hits += 1);
    Ok(())
}

/// The core side of the postMessage bridge: the sender is whatever the host derived from
/// `event.source`; `claimed` is only logged.
#[tauri::command]
fn probe_bridge(
    state: State<'_, ProbeState>,
    sender: String,
    claimed: Option<String>,
    command: String,
) -> IpcResult<()> {
    let needed = Permission::UlakCommand(command.clone());
    let allowed = declared(&sender).contains(&needed);
    state.with_current(|o| {
        o.bridge.push(BridgeDecision {
            sender: sender.clone(),
            claimed,
            command,
            allowed,
        });
    });
    if allowed {
        Ok(())
    } else {
        Err(AtabeyError::from(ErrorKind::PermissionDenied {
            extension: ExtensionId::from(sender.as_str()),
            needed,
        }))
    }
}

/// Receives the page's report, decides, prints and exits.
#[tauri::command]
fn probe_finish(
    app: tauri::AppHandle,
    state: State<'_, ProbeState>,
    report: ProbeReport,
) -> IpcResult<()> {
    let observed = state.lock().observed.clone();
    let origins = [
        "tauri://localhost",
        "http://tauri.localhost",
        "https://tauri.localhost",
    ];
    let verdict = evaluate(&report, &observed, &origins);
    println!("{}", verdict.markdown);
    if let Some(path) = std::env::var_os("GITHUB_STEP_SUMMARY") {
        let appended = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .and_then(|mut file| writeln!(file, "{}", verdict.markdown));
        if let Err(error) = appended {
            eprintln!("probe: could not write the job summary: {error}");
        }
    }
    eprintln!("probe: exit code {}", verdict.exit_code);
    app.exit(verdict.exit_code);
    Ok(())
}

fn is_production_url(url: &tauri::Url) -> bool {
    matches!(
        (url.scheme(), url.host_str()),
        ("tauri", Some("localhost")) | ("http" | "https", Some("tauri.localhost"))
    )
}

/// The probe's plan, handed to the page as data.
fn plan_json() -> String {
    let extensions = [DEFTER_VIEWER, GIT_PULSE];
    let variants: Vec<serde_json::Value> = [
        (LOCALHOST_HOST, ext_scheme::LOCALHOST_HOST),
        (ISOLATED_HOST, ext_scheme::ISOLATED_HOST),
    ]
    .iter()
    .map(|(name, host)| {
        let urls: BTreeMap<&str, String> = extensions
            .iter()
            .map(|e| {
                (
                    *e,
                    ext_scheme::ui_url(host, &ExtensionId::from(*e), "ui/index.html"),
                )
            })
            .collect();
        serde_json::json!({ "name": name, "urls": urls })
    })
    .collect();
    serde_json::json!({
        "variants": variants,
        "extensions": extensions,
        "run_timeout_ms": RUN_TIMEOUT_MS,
        "app_asset_urls": ["tauri://localhost/index.html", "http://tauri.localhost/index.html"],
    })
    .to_string()
}

fn on_page_load(webview: &Webview<Wry>, payload: &PageLoadPayload<'_>) {
    if payload.event() != PageLoadEvent::Finished || webview.label() != "main" {
        return;
    }
    let app = webview.app_handle();
    if !is_production_url(payload.url()) {
        eprintln!(
            "probe: the main frame loaded {} (dev mode). The probe must run in production mode: \
             build with the tauri/custom-protocol feature (mise run probe).",
            payload.url()
        );
        app.exit(1);
        return;
    }
    let state = app.state::<ProbeState>();
    {
        let mut inner = state.lock();
        if inner.started {
            return;
        }
        inner.started = true;
    }
    let observer = app.clone();
    app.listen_any(BREACH_EVENT, move |_event| {
        eprintln!("probe: BREACH EVENT received");
        observer
            .state::<ProbeState>()
            .with_current(|o| o.breach_events += 1);
    });
    let script = format!(
        "const PROBE_PLAN = {};\n{ATTACKS_JS}\n{HOST_JS}",
        plan_json()
    );
    if let Err(error) = webview.eval(script) {
        eprintln!("probe: could not inject the orchestrator: {error}");
        app.exit(1);
    }
}

/// Experiment hooks, probe builds only: replace the extension page CSP or add an
/// `Access-Control-Allow-Origin` header without recompiling, to find the minimal header set
/// that lets an opaque-origin page load its own files (ADR-0007, possible amendment A-4).
fn apply_experiment_headers(response: &mut tauri::http::Response<Cow<'static, [u8]>>) {
    use tauri::http::{HeaderValue, header};
    let headers = response.headers_mut();
    let overrides = [
        ("ATABEY_PROBE_CSP", header::CONTENT_SECURITY_POLICY),
        ("ATABEY_PROBE_ACAO", header::ACCESS_CONTROL_ALLOW_ORIGIN),
    ];
    for (variable, name) in overrides {
        if let Some(value) = std::env::var(variable)
            .ok()
            .map(|v| {
                if v == "spec" {
                    ext_scheme::SPEC_CSP.to_owned()
                } else {
                    v
                }
            })
            .and_then(|v| HeaderValue::from_str(&v).ok())
        {
            headers.insert(name, value);
        }
    }
}

fn probe_handler() -> impl Fn(Invoke<Wry>) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        probe_canary,
        probe_begin,
        probe_bridge,
        probe_finish,
        probe_log
    ]
}

/// Registers the scheme, the probe commands and the observers on `builder`.
///
/// `main_handler` keeps serving every command that is not a probe command, so the app behaves as
/// usual. The frame-src of the runtime CSP is widened for both URL shapes under test; the
/// committed `tauri.conf.json` is not touched (the selected shape is decided by ADR-0007).
pub fn install<H>(
    builder: tauri::Builder<Wry>,
    context: &mut Context<Wry>,
    main_handler: H,
) -> tauri::Builder<Wry>
where
    H: Fn(Invoke<Wry>) -> bool + Send + Sync + 'static,
{
    let frame_src = format!(
        "frame-src {s}: http://{s}.{l} http://{s}.{i}",
        s = ext_scheme::SCHEME,
        l = ext_scheme::LOCALHOST_HOST,
        i = ext_scheme::ISOLATED_HOST,
    );
    let security = &mut context.config_mut().app.security;
    for csp in [&mut security.csp, &mut security.dev_csp] {
        if let Some(existing) = csp.as_ref() {
            *csp = Some(tauri::utils::config::Csp::Policy(format!(
                "{existing}; {frame_src}"
            )));
        }
    }

    std::thread::spawn(|| {
        std::thread::sleep(WATCHDOG);
        eprintln!("probe: watchdog fired after {} s", WATCHDOG.as_secs());
        std::process::exit(EXIT_WATCHDOG);
    });

    let probe_handler = probe_handler();
    builder
        .manage(ProbeState::default())
        .register_uri_scheme_protocol(ext_scheme::SCHEME, |_ctx, request| {
            let mut response = ext_scheme::respond(&EmbeddedUi, &request);
            apply_experiment_headers(&mut response);
            eprintln!(
                "probe: served {} {} -> {}",
                request.method(),
                request.uri(),
                response.status()
            );
            response
        })
        .on_page_load(on_page_load)
        .invoke_handler(move |invoke: Invoke<Wry>| {
            if PROBE_COMMANDS.contains(&invoke.message.command()) {
                probe_handler(invoke)
            } else {
                main_handler(invoke)
            }
        })
}
