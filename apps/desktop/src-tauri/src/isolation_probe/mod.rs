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

/// Adds `sources` to the `frame-src` directive of `policy`.
///
/// A second `frame-src` directive would be ignored (the first one wins in CSP), so the existing
/// directive is extended; one is added when the policy has none.
#[cfg(any(test, feature = "isolation-probe"))]
fn widen_frame_src(policy: &str, sources: &str) -> String {
    let mut found = false;
    let mut directives: Vec<String> = policy
        .split(';')
        .map(str::trim)
        .filter(|directive| !directive.is_empty())
        .map(|directive| {
            if directive.split_whitespace().next() == Some("frame-src") {
                found = true;
                format!("{directive} {sources}")
            } else {
                directive.to_owned()
            }
        })
        .collect();
    if !found {
        directives.push(format!("frame-src {sources}"));
    }
    directives.join("; ")
}

#[cfg(test)]
mod tests {
    use super::widen_frame_src;

    const OBSERVED: &str = "atabey-ext://localhost http://atabey-ext.localhost";

    #[test]
    fn an_existing_frame_src_directive_is_extended_not_duplicated() {
        let policy = "default-src 'self' customprotocol: asset:; connect-src 'self' ipc: http://ipc.localhost; frame-src atabey-ext://sandbox http://atabey-ext.sandbox";
        assert_eq!(
            widen_frame_src(policy, OBSERVED),
            "default-src 'self' customprotocol: asset:; connect-src 'self' ipc: http://ipc.localhost; frame-src atabey-ext://sandbox http://atabey-ext.sandbox atabey-ext://localhost http://atabey-ext.localhost"
        );
    }

    #[test]
    fn a_policy_without_frame_src_gains_the_directive() {
        assert_eq!(
            widen_frame_src(
                "default-src 'self' ; style-src 'self' 'unsafe-inline'",
                OBSERVED
            ),
            "default-src 'self'; style-src 'self' 'unsafe-inline'; frame-src atabey-ext://localhost http://atabey-ext.localhost"
        );
    }
}
