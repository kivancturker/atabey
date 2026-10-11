//! The probe's report and its verdict. Pure: no Tauri types, unit-tested in every test run.
//!
//! Two sources of truth meet here. The JavaScript report says what each attack *experienced*;
//! the Rust observations ([`Observed`]) say what actually *arrived* (canary hits, breach events,
//! bridge decisions). The verdict trusts the Rust side for IPC and the bridge, and uses the
//! JavaScript side for what only the page can see (storage, parent access, functionality).

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

/// The variant name of the main frame running the same attacks (it must reach the canary).
pub const CONTROL: &str = "control";
/// `atabey-ext://localhost/…`, observed, never asserted.
pub const LOCALHOST_HOST: &str = "localhost-host";
/// `atabey-ext://sandbox/…`, asserted.
pub const ISOLATED_HOST: &str = "isolated-host";
/// The two fixture extensions.
pub const DEFTER_VIEWER: &str = "atabey.defter-viewer";
/// Declares no permissions at all.
pub const GIT_PULSE: &str = "atabey.git-pulse";

/// Process exit code: isolated.
pub const EXIT_ISOLATED: i32 = 0;
/// Process exit code: a breach, a missing functional check or a broken harness.
pub const EXIT_BREACH: i32 = 1;
/// Process exit code: the watchdog fired.
pub const EXIT_WATCHDOG: i32 = 2;

/// What a check tries to do, which decides how its result is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckKind {
    /// Recorded only (globals, origin, secure context).
    Info,
    /// An attempt to call the Rust core. Rust's own observations decide.
    Ipc,
    /// An attempt to leave the sandbox. `success` is a breach.
    Containment,
    /// Something an extension page needs to work. `success` is required.
    Functional,
    /// A message over the postMessage bridge. Expectation depends on the check id.
    Bridge,
}

/// One attempt and its outcome as the page saw it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckResult {
    /// Stable id, e.g. `storage.local`.
    pub id: String,
    /// How to read `success`.
    pub kind: CheckKind,
    /// Whether the attempt did what it tried to do (attack worked, feature worked, call allowed).
    pub success: bool,
    /// The error or measurement, for the ADR.
    pub detail: String,
}

/// All checks of one iframe (or of the main frame for the control).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunReport {
    /// `control`, `localhost-host` or `isolated-host`.
    pub variant: String,
    /// The extension id (`main-frame` for the control).
    pub extension: String,
    /// The iframe never reported back.
    pub silent: bool,
    /// The main frame's location changed while the iframe ran.
    pub top_navigated: bool,
    /// The results.
    pub checks: Vec<CheckResult>,
}

/// What the page sends to `probe_finish`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProbeReport {
    /// `location.origin` of the main frame.
    pub main_origin: String,
    /// One entry per frame that ran.
    pub runs: Vec<RunReport>,
    /// The orchestrator crashed.
    #[serde(default)]
    pub fatal: Option<String>,
}

/// A bridge message and what the core decided, taken from the sender the *host* identified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeDecision {
    /// The extension whose window sent the message (`event.source`).
    pub sender: String,
    /// The extension the message claimed to be from, if any.
    pub claimed: Option<String>,
    /// The Ulak command requested.
    pub command: String,
    /// Whether the core allowed it.
    pub allowed: bool,
}

/// What Rust itself saw while one variant ran.
#[derive(Debug, Clone, Default)]
pub struct Observed {
    /// Calls that reached the `probe_canary` command.
    pub canary_hits: u32,
    /// Emissions of the probe-only breach event.
    pub breach_events: u32,
    /// Every bridge decision.
    pub bridge: Vec<BridgeDecision>,
}

/// The outcome.
#[derive(Debug, Clone)]
pub struct Verdict {
    /// 0 isolated, 1 breach.
    pub exit_code: i32,
    /// One line per violated expectation.
    pub failures: Vec<String>,
    /// The Markdown table for stdout and the CI job summary.
    pub markdown: String,
}

const VARIANTS: [&str; 3] = [CONTROL, LOCALHOST_HOST, ISOLATED_HOST];
const REQUIRED_FUNCTIONAL: [&str; 5] = [
    "fn.classic-script",
    "fn.module-script",
    "fn.stylesheet",
    "fn.fetch-json",
    "fn.postmessage-roundtrip",
];

fn runs_of<'a>(report: &'a ProbeReport, variant: &'a str) -> impl Iterator<Item = &'a RunReport> {
    report.runs.iter().filter(move |run| run.variant == variant)
}

/// Whether the variant showed any breach, by any observer (used for the table and the ADR).
fn breached(report: &ProbeReport, observed: &BTreeMap<String, Observed>, variant: &str) -> bool {
    let rust_side = observed
        .get(variant)
        .is_some_and(|o| o.canary_hits > 0 || o.breach_events > 0);
    rust_side
        || runs_of(report, variant).any(|run| {
            run.top_navigated
                || run
                    .checks
                    .iter()
                    .any(|c| c.kind == CheckKind::Containment && c.success)
        })
}

/// The expected `success` of a bridge check, by id.
fn expected_bridge(id: &str) -> Option<bool> {
    match id {
        "bridge.declared-command" => Some(true),
        "bridge.undeclared-command" | "bridge.claimed-identity" => Some(false),
        _ => None,
    }
}

fn assert_isolated(
    report: &ProbeReport,
    observed: &BTreeMap<String, Observed>,
    failures: &mut Vec<String>,
) {
    let seen = observed.get(ISOLATED_HOST).cloned().unwrap_or_default();
    if seen.canary_hits > 0 {
        failures.push(format!(
            "{ISOLATED_HOST}: {} call(s) reached the Rust canary command",
            seen.canary_hits
        ));
    }
    if seen.breach_events > 0 {
        failures.push(format!(
            "{ISOLATED_HOST}: {} breach event(s) were emitted",
            seen.breach_events
        ));
    }
    for extension in [DEFTER_VIEWER, GIT_PULSE] {
        let Some(run) = runs_of(report, ISOLATED_HOST).find(|run| run.extension == extension)
        else {
            failures.push(format!("{ISOLATED_HOST}/{extension}: no run in the report"));
            continue;
        };
        if run.silent {
            failures.push(format!(
                "{ISOLATED_HOST}/{extension}: the iframe never reported"
            ));
        }
        if run.top_navigated {
            failures.push(format!(
                "{ISOLATED_HOST}/{extension}: the top frame was navigated"
            ));
        }
        for check in &run.checks {
            let violated = match check.kind {
                CheckKind::Containment => check.success,
                CheckKind::Functional => !check.success,
                CheckKind::Bridge => expected_bridge(&check.id).is_some_and(|e| e != check.success),
                CheckKind::Info | CheckKind::Ipc => false,
            };
            if violated {
                failures.push(format!(
                    "{ISOLATED_HOST}/{extension}: {} went the wrong way ({})",
                    check.id, check.detail
                ));
            }
        }
        for id in REQUIRED_FUNCTIONAL {
            if !run.checks.iter().any(|c| c.id == id) {
                failures.push(format!(
                    "{ISOLATED_HOST}/{extension}: functional check {id} is missing"
                ));
            }
        }
    }
    let decided = |sender: &str, command: &str, claimed: Option<&str>, allowed: bool| {
        seen.bridge.iter().any(|d| {
            d.sender == sender
                && d.command == command
                && d.allowed == allowed
                && claimed.is_none_or(|c| d.claimed.as_deref() == Some(c))
        })
    };
    if !decided(DEFTER_VIEWER, "defter.query", None, true) {
        failures.push(format!(
            "{ISOLATED_HOST}: the core did not allow a declared command"
        ));
    }
    if !decided(DEFTER_VIEWER, "settings.set", None, false) {
        failures.push(format!(
            "{ISOLATED_HOST}: the core did not deny an undeclared command"
        ));
    }
    if !decided(GIT_PULSE, "defter.query", Some(DEFTER_VIEWER), false) {
        failures.push(format!(
            "{ISOLATED_HOST}: a forged identity claim was not denied against the real sender"
        ));
    }
    if seen
        .bridge
        .iter()
        .any(|d| d.sender == GIT_PULSE && d.allowed)
    {
        failures.push(format!(
            "{ISOLATED_HOST}: an extension without permissions was allowed"
        ));
    }
}

/// Reads the report against the expectations and renders the table.
///
/// `control` must reach the canary, `isolated-host` must be isolated and functional,
/// `localhost-host` is reported but never fails the run.
#[must_use]
pub fn evaluate(
    report: &ProbeReport,
    observed: &BTreeMap<String, Observed>,
    production_origins: &[&str],
) -> Verdict {
    let mut failures = Vec::new();
    if let Some(fatal) = &report.fatal {
        failures.push(format!("the orchestrator crashed: {fatal}"));
    }
    if !production_origins.contains(&report.main_origin.as_str()) {
        failures.push(format!(
            "the main frame ran at {}, not at the production origin ({}); build with tauri/custom-protocol",
            report.main_origin,
            production_origins.join(" or ")
        ));
    }
    for variant in VARIANTS {
        if runs_of(report, variant).next().is_none() {
            failures.push(format!("{variant}: no run in the report"));
        }
    }
    if observed.get(CONTROL).is_none_or(|o| o.canary_hits == 0) {
        failures.push(format!(
            "{CONTROL}: the main frame did not reach the canary, so the probe could not detect a breach"
        ));
    }
    assert_isolated(report, observed, &mut failures);

    let markdown = render(report, observed, &failures);
    Verdict {
        exit_code: if failures.is_empty() {
            EXIT_ISOLATED
        } else {
            EXIT_BREACH
        },
        failures,
        markdown,
    }
}

fn cell(result: &CheckResult) -> String {
    let text = match result.kind {
        CheckKind::Info if result.id.starts_with("env.") => format!("`{}`", result.detail),
        CheckKind::Info if result.id.starts_with("global.") => {
            if result.success { "present" } else { "absent" }.to_owned()
        }
        CheckKind::Info => if result.success { "fetched" } else { "blocked" }.to_owned(),
        CheckKind::Ipc if result.detail.starts_with("unavailable") => "unavailable".to_owned(),
        CheckKind::Ipc => if result.success {
            "**resolved**"
        } else {
            "rejected"
        }
        .to_owned(),
        CheckKind::Containment => if result.success {
            "**reached**"
        } else {
            "blocked"
        }
        .to_owned(),
        CheckKind::Functional => if result.success { "ok" } else { "**FAIL**" }.to_owned(),
        CheckKind::Bridge => if result.success { "allowed" } else { "denied" }.to_owned(),
    };
    text.replace('|', "/")
}

fn render(
    report: &ProbeReport,
    observed: &BTreeMap<String, Observed>,
    failures: &[String],
) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "### Extension iframe isolation probe ({})\n",
        std::env::consts::OS
    );
    let _ = writeln!(out, "Main frame origin: `{}`\n", report.main_origin);
    let _ = writeln!(
        out,
        "| Check | control | localhost-host | isolated-host |\n|---|---|---|---|"
    );

    let mut ids: Vec<&str> = Vec::new();
    for run in &report.runs {
        for check in &run.checks {
            if !ids.contains(&check.id.as_str()) {
                ids.push(&check.id);
            }
        }
    }
    for id in ids {
        let _ = write!(out, "| `{id}` |");
        for variant in VARIANTS {
            let results: Vec<&CheckResult> = runs_of(report, variant)
                .flat_map(|run| run.checks.iter())
                .filter(|c| c.id == id)
                .collect();
            let text = match results.split_first() {
                None => "-".to_owned(),
                Some((first, rest)) if rest.iter().any(|r| r.success != first.success) => {
                    format!("{} (differs per extension)", cell(first))
                }
                Some((first, _)) => cell(first),
            };
            let _ = write!(out, " {text} |");
        }
        out.push('\n');
    }
    let row = |label: &str, value: &dyn Fn(&str) -> String| {
        let mut line = format!("| **{label}** |");
        for variant in VARIANTS {
            let _ = write!(line, " {} |", value(variant));
        }
        line.push('\n');
        line
    };
    let count = |variant: &str| observed.get(variant).cloned().unwrap_or_default();
    out.push_str(&row("Rust: canary hits", &|v| {
        count(v).canary_hits.to_string()
    }));
    out.push_str(&row("Rust: breach events", &|v| {
        count(v).breach_events.to_string()
    }));
    out.push_str(&row("Rust: bridge allowed / denied", &|v| {
        let o = count(v);
        let allowed = o.bridge.iter().filter(|d| d.allowed).count();
        format!("{allowed} / {}", o.bridge.len() - allowed)
    }));
    out.push_str(&row("Iframe never reported", &|v| {
        runs_of(report, v).filter(|r| r.silent).count().to_string()
    }));
    out.push_str(&row("Top frame navigated", &|v| {
        runs_of(report, v)
            .filter(|r| r.top_navigated)
            .count()
            .to_string()
    }));
    out.push_str(&row(
        "Result",
        &|v| match (v, breached(report, observed, v)) {
            (CONTROL, true) => "reaches the core (as required)".to_owned(),
            (CONTROL, false) => "**never reached**".to_owned(),
            (_, true) => "**BREACH**".to_owned(),
            (_, false) => "isolated".to_owned(),
        },
    ));

    out.push('\n');
    if failures.is_empty() {
        out.push_str("**Verdict: isolated.** The asserted variant (`isolated-host`) held; `localhost-host` is reported, not asserted.\n");
    } else {
        out.push_str("**Verdict: FAILED**\n\n");
        for failure in failures {
            let _ = writeln!(out, "- {failure}");
        }
    }
    out.push_str("\n<details><summary>Details per check</summary>\n\n| Variant | Extension | Check | Detail |\n|---|---|---|---|\n");
    for run in &report.runs {
        for check in &run.checks {
            let _ = writeln!(
                out,
                "| {} | {} | `{}` | {} |",
                run.variant,
                run.extension,
                check.id,
                check.detail.replace('|', "/").replace('\n', " ")
            );
        }
    }
    out.push_str("\n</details>\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const ORIGINS: [&str; 2] = ["tauri://localhost", "http://tauri.localhost"];

    fn check(id: &str, kind: CheckKind, success: bool) -> CheckResult {
        CheckResult {
            id: id.to_owned(),
            kind,
            success,
            detail: "measured".to_owned(),
        }
    }

    fn functional() -> Vec<CheckResult> {
        REQUIRED_FUNCTIONAL
            .iter()
            .map(|id| check(id, CheckKind::Functional, true))
            .collect()
    }

    fn run(variant: &str, extension: &str, mut checks: Vec<CheckResult>) -> RunReport {
        checks.extend(functional());
        checks.push(check("storage.local", CheckKind::Containment, false));
        checks.push(check("bridge.undeclared-command", CheckKind::Bridge, false));
        RunReport {
            variant: variant.to_owned(),
            extension: extension.to_owned(),
            silent: false,
            top_navigated: false,
            checks,
        }
    }

    fn decision(
        sender: &str,
        command: &str,
        claimed: Option<&str>,
        allowed: bool,
    ) -> BridgeDecision {
        BridgeDecision {
            sender: sender.to_owned(),
            claimed: claimed.map(str::to_owned),
            command: command.to_owned(),
            allowed,
        }
    }

    fn healthy() -> (ProbeReport, BTreeMap<String, Observed>) {
        let mut runs = vec![RunReport {
            variant: CONTROL.to_owned(),
            extension: "main-frame".to_owned(),
            silent: false,
            top_navigated: false,
            checks: vec![check("ipc.internals-invoke", CheckKind::Ipc, true)],
        }];
        for variant in [LOCALHOST_HOST, ISOLATED_HOST] {
            runs.push(run(
                variant,
                DEFTER_VIEWER,
                vec![check("bridge.declared-command", CheckKind::Bridge, true)],
            ));
            runs.push(run(
                variant,
                GIT_PULSE,
                vec![check("bridge.claimed-identity", CheckKind::Bridge, false)],
            ));
        }
        let report = ProbeReport {
            main_origin: "tauri://localhost".to_owned(),
            runs,
            fatal: None,
        };
        let bridge = vec![
            decision(DEFTER_VIEWER, "defter.query", None, true),
            decision(DEFTER_VIEWER, "settings.set", None, false),
            decision(GIT_PULSE, "defter.query", Some(DEFTER_VIEWER), false),
        ];
        let observed = BTreeMap::from([
            (
                CONTROL.to_owned(),
                Observed {
                    canary_hits: 2,
                    breach_events: 1,
                    bridge: vec![],
                },
            ),
            (
                LOCALHOST_HOST.to_owned(),
                Observed {
                    bridge: bridge.clone(),
                    ..Observed::default()
                },
            ),
            (
                ISOLATED_HOST.to_owned(),
                Observed {
                    bridge,
                    ..Observed::default()
                },
            ),
        ]);
        (report, observed)
    }

    fn failures_of(report: &ProbeReport, observed: &BTreeMap<String, Observed>) -> Vec<String> {
        evaluate(report, observed, &ORIGINS).failures
    }

    #[test]
    fn a_healthy_run_is_isolated() {
        let (report, observed) = healthy();
        let verdict = evaluate(&report, &observed, &ORIGINS);
        assert_eq!(verdict.failures, Vec::<String>::new());
        assert_eq!(verdict.exit_code, EXIT_ISOLATED);
        assert!(verdict.markdown.contains("| `storage.local` |"));
        assert!(verdict.markdown.contains("**Verdict: isolated.**"));
    }

    #[test]
    fn a_canary_hit_from_the_isolated_variant_is_a_breach() {
        let (report, mut observed) = healthy();
        observed
            .entry(ISOLATED_HOST.to_owned())
            .or_default()
            .canary_hits = 1;
        let verdict = evaluate(&report, &observed, &ORIGINS);
        assert_eq!(verdict.exit_code, EXIT_BREACH);
        assert!(verdict.failures[0].contains("reached the Rust canary"));
        assert!(verdict.markdown.contains("**BREACH**"));
    }

    #[test]
    fn a_breach_event_from_the_isolated_variant_is_a_breach() {
        let (report, mut observed) = healthy();
        observed
            .entry(ISOLATED_HOST.to_owned())
            .or_default()
            .breach_events = 1;
        assert!(failures_of(&report, &observed)[0].contains("breach event"));
    }

    #[test]
    fn a_control_that_cannot_reach_the_canary_fails_the_probe() {
        let (report, mut observed) = healthy();
        observed.entry(CONTROL.to_owned()).or_default().canary_hits = 0;
        assert!(failures_of(&report, &observed)[0].contains("could not detect a breach"));
    }

    #[test]
    fn the_localhost_variant_is_reported_but_never_fails() {
        let (mut report, mut observed) = healthy();
        observed
            .entry(LOCALHOST_HOST.to_owned())
            .or_default()
            .canary_hits = 3;
        for run in report
            .runs
            .iter_mut()
            .filter(|r| r.variant == LOCALHOST_HOST)
        {
            run.checks
                .push(check("parent.document", CheckKind::Containment, true));
        }
        let verdict = evaluate(&report, &observed, &ORIGINS);
        assert_eq!(verdict.exit_code, EXIT_ISOLATED);
        assert!(verdict.markdown.contains("**BREACH**"));
    }

    #[test]
    fn a_reached_containment_check_or_a_navigated_top_frame_fails() {
        let (mut report, observed) = healthy();
        report.runs[3]
            .checks
            .push(check("parent.document", CheckKind::Containment, true));
        report.runs[4].top_navigated = true;
        let failures = failures_of(&report, &observed);
        assert!(failures.iter().any(|f| f.contains("parent.document")));
        assert!(
            failures
                .iter()
                .any(|f| f.contains("top frame was navigated"))
        );
    }

    #[test]
    fn a_missing_or_failing_functional_check_fails() {
        let (mut report, observed) = healthy();
        report.runs[3].checks.retain(|c| c.id != "fn.module-script");
        report.runs[4]
            .checks
            .iter_mut()
            .filter(|c| c.id == "fn.fetch-json")
            .for_each(|c| c.success = false);
        let failures = failures_of(&report, &observed);
        assert!(
            failures
                .iter()
                .any(|f| f.contains("fn.module-script is missing"))
        );
        assert!(
            failures
                .iter()
                .any(|f| f.contains("fn.fetch-json went the wrong way"))
        );
    }

    #[test]
    fn a_silent_iframe_fails() {
        let (mut report, observed) = healthy();
        report.runs[3].silent = true;
        assert!(failures_of(&report, &observed)[0].contains("never reported"));
    }

    #[test]
    fn the_bridge_must_deny_forgeries_using_the_real_sender() {
        let (report, mut observed) = healthy();
        let isolated = observed.entry(ISOLATED_HOST.to_owned()).or_default();
        // The core believed the claim: it allowed the query on behalf of git-pulse.
        isolated.bridge[2] = decision(GIT_PULSE, "defter.query", Some(DEFTER_VIEWER), true);
        let failures = failures_of(&report, &observed);
        assert!(failures.iter().any(|f| f.contains("forged identity claim")));
        assert!(
            failures
                .iter()
                .any(|f| f.contains("without permissions was allowed"))
        );
    }

    #[test]
    fn the_javascript_claim_of_a_bridge_forgery_succeeding_fails() {
        let (mut report, observed) = healthy();
        report.runs[4]
            .checks
            .iter_mut()
            .filter(|c| c.id == "bridge.claimed-identity")
            .for_each(|c| c.success = true);
        assert!(failures_of(&report, &observed)[0].contains("bridge.claimed-identity"));
    }

    #[test]
    fn running_outside_the_production_origin_fails() {
        let (mut report, observed) = healthy();
        report.main_origin = "http://127.0.0.1:1420".to_owned();
        assert!(failures_of(&report, &observed)[0].contains("production origin"));
    }

    #[test]
    fn a_crashed_orchestrator_and_missing_variants_fail() {
        let (mut report, observed) = healthy();
        report.fatal = Some("TypeError: invoke is not a function".to_owned());
        report.runs.retain(|r| r.variant != ISOLATED_HOST);
        let failures = failures_of(&report, &observed);
        assert!(failures[0].contains("crashed"));
        assert!(failures.iter().any(|f| f.contains("isolated-host: no run")));
    }

    #[test]
    fn the_wire_format_rejects_unknown_fields() {
        let ok = r#"{"main_origin":"tauri://localhost","runs":[]}"#;
        assert!(serde_json::from_str::<ProbeReport>(ok).is_ok());
        let extra = r#"{"main_origin":"tauri://localhost","runs":[],"verdict":"fine"}"#;
        assert!(serde_json::from_str::<ProbeReport>(extra).is_err());
        let bad_kind = r#"{"id":"x","kind":"magic","success":true,"detail":""}"#;
        assert!(serde_json::from_str::<CheckResult>(bad_kind).is_err());
    }
}
