---
id: ADR-0007
title: Extension UI isolation — sandboxed iframes on atabey-ext://
status: accepted
date: 2026-10-12
decisions: [D-100]
spec: specs/S0-foundations.md (R-34, AC-17, §5.10, §5.12)
---

# ADR-0007: Extension UI isolation — sandboxed iframes on `atabey-ext://`

> **Status: accepted (2026-10-12).** Both systems are measured: macOS locally and in CI, Windows in CI ([run 38178105506](https://github.com/kivancturker/atabey/actions/runs/38178105506), commit `c126dc3`, `windows-latest`, 2026-10-11 21:58 UTC). The decision gate at the end selected the `sandbox` host. Spec amendment A-3 is applied (v1.0.3); A-4 is proposed and waits for the user.

## Context

R-34 puts extension UIs in `<iframe sandbox="allow-scripts">` served from `atabey-ext://`, and AC-17 requires that such a frame cannot call Atabey's internal API on macOS and Windows. The v0.5 design assumed that an opaque origin makes the frame "not the app". Reading Tauri 2.12.3 and wry 0.57.0 shows that this is not automatically true:

| Finding | Source (vendored crates) |
|---|---|
| Tauri classifies a frame by its **URL**, not by its origin. Every app-registered custom scheme counts as **local**. On macOS the test is the scheme; on Windows the URL must be `http(s)://<scheme>.localhost`, so a host that does not end in `.localhost` is **remote** | `tauri-2.12.3/src/webview/mod.rs:1961` (`is_local_url`) |
| App commands from a local frame skip the ACL; remote frames are denied unless a capability lists them. Plugin commands from a local frame use the capability of the frame's window | `tauri-2.12.3/src/webview/mod.rs:2079` |
| Init scripts (with `__TAURI_INTERNALS__` and the invoke key) are injected into **all sub-frames on Windows**, main frame only on macOS | `wry-0.57.0/src/lib.rs:1029`, `src/webview2/mod.rs:507`, `src/wkwebview/mod.rs:780` |
| The custom-protocol IPC needs a parseable `Origin` header; an opaque origin sends `null`, so the call is refused and the JS falls back to `postMessage`, which is attributed to the **frame URL** | `tauri-2.12.3/src/ipc/protocol.rs:487`, `scripts/ipc-protocol.js`; `wry-0.57.0/src/wkwebview/class/wry_web_view_delegate.rs:50`, `src/webview2/mod.rs:964` |
| Windows has no custom schemes: `atabey-ext://HOST/path` is served as `http://atabey-ext.HOST/path` and mapped back before the handler runs | `wry-0.57.0/src/custom_protocol_workaround.rs` |
| The CSP is read from the runtime config, so a probe build can widen `frame-src` with `Context::config_mut()` | `tauri-2.12.3/src/manager/mod.rs:361` |

So the spec-literal URL `atabey-ext://localhost/…` (on Windows `http://atabey-ext.localhost/…`) is classified as **local** by Tauri on both systems. Tauri's own guidance for untrusted content is a dedicated webview whose capability is bound to its label (GHSA-57fm-592m-34r7).

## Decision

Serve extension pages from `atabey-ext://sandbox/<extension id>/…` (Windows: `http://atabey-ext.sandbox/…`) inside `<iframe sandbox="allow-scripts">`, with three header rules (measured on both systems, see the findings below):

| Header | Value | Why |
|---|---|---|
| `Content-Security-Policy` | `default-src <origin>/<extension id>/` | **Amends §5.12** (`default-src 'self'` does not work, see below; proposed amendment A-4). Page can load only its own folder |
| `Access-Control-Allow-Origin` | `null` | Module scripts and `fetch()` from an opaque-origin page are CORS requests |
| `X-Content-Type-Options` / `Cache-Control` | `nosniff` / `no-store` | Module scripts need a JavaScript MIME type; no stale UI |

The host `sandbox` is never `localhost` and never ends in `.localhost` (unit test in `ext_scheme.rs`). By Tauri's source, that makes the frame **remote** on Windows, so its commands need a capability that lists the origin, and none does. Both hosts measured as safe, so this extra layer was **not exercised** by the run (Windows finding 4); it is chosen because it still holds if a later wry or WebView2 version starts delivering iframe messages. `ext_scheme::respond` serves the `sandbox` host only, and the main window's `frame-src` lists only that host (A-3): `frame-src atabey-ext://sandbox http://atabey-ext.sandbox`.

The probe stays in CI as the regression guard for every Tauri/wry upgrade (`mise run probe`, T-04; T-24 extends it to the real extension host).

## How the probe works

A build of the real app (production mode, `tauri/custom-protocol`, real CSP, real main window) started with `--isolation-probe`. It embeds the two fixture extensions (`atabey.defter-viewer` declares `defter.query` and `defter.appended`; `atabey.git-pulse` declares nothing), and runs each of these as a sandboxed iframe in turn:

| Variant | iframe URL (macOS · Windows) | Role |
|---|---|---|
| `control` | the main frame itself runs the same attack code | Must **reach** the canary command, which proves the probe can see a breach |
| `localhost-host` | `atabey-ext://localhost/…` · `http://atabey-ext.localhost/…` | Spec-literal, observed, never fails the run |
| `isolated-host` | `atabey-ext://sandbox/…` · `http://atabey-ext.sandbox/…` | Candidate, asserted |

Rust observes independently of what the page claims: calls to the canary command, emissions of a probe-only breach event, and every bridge decision (with the sender taken from `event.source`). Exit code 0 = isolated, 1 = breach or missing functional check, 2 = watchdog (120 s). A sabotage run with `allow-same-origin` added made it exit 1 (storage reachable).

## Results

Cell values: **blocked / rejected / absent** are the safe outcomes; **reached / resolved** means the attack worked. "No answer" means the call neither resolved nor rejected within the probe's 1.5 s and nothing arrived in Rust.

| Check | control (main frame, macOS) | macOS `localhost-host` | macOS `isolated-host` | Windows `localhost-host` | Windows `isolated-host` |
|---|---|---|---|---|---|
| `location.origin` / `window.origin` | `tauri://localhost` | `atabey-ext://localhost` / `null` | `atabey-ext://sandbox` / `null` | `http://atabey-ext.localhost` / `null` | `http://atabey-ext.sandbox` / `null` |
| `isSecureContext` | true | true | true | true | **false** |
| `__TAURI__` | absent | absent | absent | absent | absent |
| `__TAURI_INTERNALS__`, `isTauri`, `window.ipc` | present | absent | absent | **present** | **present** |
| `webkit.messageHandlers.ipc` | present | **present** | **present** | n/a | n/a |
| `chrome.webview` | absent | absent | absent | **present** | **present** |
| `invoke("probe_canary")` | **resolved** | unavailable | unavailable | rejected (no answer) | rejected (no answer) |
| `plugin:event\|emit` | **resolved** | unavailable | unavailable | rejected (no answer) | rejected (no answer) |
| forged `window.ipc` / `chrome.webview` / `webkit` post | rejected (key) | rejected (key) | rejected (key) | rejected (never arrived) | rejected (never arrived) |
| forged `fetch` to `ipc://` and `http://ipc.localhost` | rejected | rejected | rejected | rejected | rejected |
| `parent.document`, `parent.__TAURI_INTERNALS__` | reached (same window) | blocked | blocked | blocked | blocked |
| `top.location` navigation, `window.open` | n/a | blocked | blocked | blocked | blocked |
| `localStorage`, `indexedDB`, `document.cookie` | reached | blocked | blocked | blocked | blocked |
| fetch `tauri://localhost/index.html` (Windows: `http://tauri.localhost/index.html`) | reached | blocked | blocked | blocked | blocked |
| fetch another extension's file | n/a | blocked | blocked | blocked | blocked |
| classic script, ES module, stylesheet, `fetch("./file.json")`, postMessage round trip | n/a | ok | ok | ok | ok |
| bridge: declared command / undeclared command / forged identity | n/a | allowed / denied / denied | allowed / denied / denied | allowed / denied / denied | allowed / denied / denied |
| **Rust: canary hits / breach events** | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |

(macOS: probe runs of 2026-10-12, locally three times and once in CI, identical results. Windows: the CI run linked at the top. The Windows control ran at `http://tauri.localhost` and reached the canary and the breach event once each, like the macOS control. The full per-check detail strings are in each job summary.)

### What the macOS run found

| # | Finding | Consequence |
|---|---|---|
| 1 | **`default-src 'self'` (spec §5.12) breaks the page.** In the sandboxed document WebKit does not match `'self'`: the HTML loads but its stylesheet, scripts and JSON are never requested | A-4 needed: use a source that names the origin, scoped to the extension folder |
| 2 | `default-src atabey-ext:` works but lets the page read other extensions' files and was replaced by `default-src atabey-ext://sandbox/<id>/`; with the folder-scoped form, fetching another extension's file is blocked | Folder-scoped CSP, generated per request |
| 3 | **CORS is required** for `<script type="module">` and `fetch("./file.json")`: without `Access-Control-Allow-Origin` both fail while classic scripts and stylesheets load. `null` and `*` both worked | `Access-Control-Allow-Origin: null` on every response |
| 4 | `location.origin` reports `atabey-ext://sandbox` even though the document is sandboxed; `window.origin` reports `null` and storage is blocked | Do not use `location.origin` to decide isolation |
| 5 | `isSecureContext` is `true` in the sandboxed frame | Do not rely on secure-context gating |
| 6 | On macOS `webkit.messageHandlers.ipc` **is present in the iframe** (WKWebView registers script message handlers for all frames) but a post is rejected because the iframe has no invoke key (init scripts are main-frame only) | Isolation on macOS rests on the missing key. Keep the forged-post checks in CI |
| 7 | `localhost-host` and `isolated-host` behave identically on macOS | Expected: the scheme, not the host, decides on macOS |
| 8 | A fetch to `ipc://localhost/…` from the main frame is aborted (does not answer) when the key is forged | None; recorded |
| 9 | `document.cookie = …` is silently ignored in the opaque origin instead of throwing; the probe reads the value back | Probe detail |

### What the Windows run found

| # | Finding | Consequence |
|---|---|---|
| 1 | **Both URL shapes are isolated on Windows too**: 0 canary hits, 0 breach events, every containment check blocked, every functional check ok | AC-17 holds on both systems; iframes stay (no D-100 / R-34 amendment) |
| 2 | **The IPC script is injected into the iframe** (`__TAURI_INTERNALS__`, `isTauri`, `window.ipc` and `chrome.webview` are present), as the source reading predicted. The iframe therefore holds the **real invoke key** | Unlike macOS, isolation on Windows does not rest on a missing key |
| 3 | A call with the real key (`invoke("probe_canary")`, `plugin:event\|emit`) gets no answer and never reaches Rust. The custom-protocol path fails inside the page (`fetch` to `http://ipc.localhost` → `Failed to fetch`: the extension-page CSP has no `connect-src` for it, and Tauri would refuse the `Origin: null` header anyway). The `postMessage` fallback does not arrive either: Tauri logged an invoke-key mismatch for each of the control's three forged posts and **none** during the iframe variants | Which layer drops the iframe's `postMessage` was not isolated. Most likely WebView2 hands frame messages only to frame-level handlers and wry registers only the top-level one (`wry-0.57.0/src/webview2/mod.rs:956`). That is library behaviour, not a documented guarantee: the probe must run on every Tauri/wry upgrade |
| 4 | `localhost-host` and `isolated-host` gave **identical security results**. The predicted difference (Tauri denying the non-local URL) was not exercised, because no message got far enough to be classified | `isolated-host` is selected for a layer known from source, not from this measurement |
| 5 | **`isSecureContext` is `false`** for `http://atabey-ext.sandbox` (it is `true` for `http://atabey-ext.localhost`, and `true` for both hosts on macOS) | On Windows an extension UI has no secure-context-only APIs (`crypto.subtle`, `crypto.randomUUID`, `navigator.clipboard`). Such features go through the bridge. Serving over `https://` (`use_https_scheme`) would restore them but also moves the main window's origin: parked as P-005 |
| 6 | The folder-scoped CSP and `Access-Control-Allow-Origin: null` work on WebView2: module script, stylesheet and `fetch("./file.json")` load, another extension's file is blocked | A-4 is confirmed on both systems |

## Decision gate (applied 2026-10-12)

```mermaid
flowchart TD
  R["Results from macOS and Windows"] --> B{"isolated-host safe and functional on both?"}
  B -- yes --> S["Select isolated-host; accepted"]
  B -- no --> A{"localhost-host safe and functional on both?"}
  A -- yes --> L["Select localhost-host; note the single layer"]
  A -- no --> X["Iframes rejected; propose a D-100 / R-34 amendment (dedicated webviews)"]
  S --> C{"Folder-scoped CSP and CORS null work on Windows?"}
  L --> C
  C -- yes --> D["Amendments A-3 (frame-src) and A-4 (extension CSP) go into the spec"]
  C -- no --> P["Re-measure with the ATABEY_PROBE_CSP / ATABEY_PROBE_ACAO hooks, then amend"]
```

| Gate question | Answer | Outcome |
|---|---|---|
| `isolated-host` safe and functional on both? | Yes (macOS and Windows: 0 canary hits, 0 breach events, all functional checks ok) | **`isolated-host` selected; ADR accepted** |
| Folder-scoped CSP and CORS `null` work on Windows? | Yes (Windows finding 6) | A-3 applied (spec v1.0.3, `tauri.conf.json`); A-4 proposed in spec §8 |

## Alternatives considered

| Option | Pros | Cons |
|---|---|---|
| Sandboxed iframe, host `sandbox` (**selected**) | Cheap; DOM overlays (menus, palette) work over panels; Tauri's own remote-origin denial on Windows is a further layer (from source); regression-guarded in CI | Depends on per-platform Tauri behaviour; macOS isolation rests on the missing invoke key plus the sandbox; not a secure context on Windows |
| Sandboxed iframe, host `localhost` (v0.5 design) | Matches v0.5 text; measured safe on both systems; a secure context on Windows | Tauri treats it as a local frame on both systems, so on Windows nothing in Tauri would stop a call that got through with the injected key |
| Dedicated webview per extension (Tauri guidance) | Capabilities bind to webview labels | Needs the `unstable` multi-webview feature; native view sits above the DOM (z-order, layout sync). Fallback if both iframe variants fail |
| Process-level isolation (Electron `<webview>`) | Strongest | Not available in Tauri; same z-order cost |
| No UI isolation (Obsidian) | Simple | No security boundary, contradicts R-34 |

Other tools for context: VS Code serves webviews from a unique `vscode-webview://` origin with `postMessage`; Figma runs plugin UI in a null-origin iframe and plugin logic in a separate JS sandbox.

## Consequences

| Area | Effect |
|---|---|
| T-22 (extension host) | Reuses `ext_scheme.rs` (parse, traversal rejection, headers, URL forms; `respond` serves the `sandbox` host only). Serves installed extensions instead of the embedded fixtures. **Never** put `ipc:`, `http://ipc.localhost` or the app origin into an extension's `connect-src`: on Windows the iframe holds the invoke key, and the page CSP is one of the layers in front of it. No secure-context APIs in extension UIs on Windows (finding 5). `PanelSlot` has no `Bottom` although §5.10's manifest and R-37 use a bottom panel: fix there |
| T-24 (security E2E) | Replaces the fixtures with the real extension host and asserts AC-17 "and logged" through Defter. Worth adding there: a Windows run with `connect-src http://ipc.localhost` forced through `ATABEY_PROBE_CSP`, to measure the `Origin: null` refusal on its own |
| T-25 | Add the "probe feature is off in release builds" check |
| Spec | Amendment **A-3** (applied, v1.0.3): main-window CSP gains `frame-src atabey-ext://sandbox http://atabey-ext.sandbox`. With only this list the `localhost` frame is blocked and the `sandbox` frame loads (measured on macOS, 2026-10-12; the probe widens the list at run time to keep measuring `localhost`). Amendment **A-4** (proposed, waits for the user): the §5.12 extension-page CSP row changes from `default-src 'self'` to the folder-scoped form, plus the CORS header |
| Probe in release | Double gate: cargo feature `isolation-probe` and the `--isolation-probe` flag. `ATABEY_PROBE_CSP` and `ATABEY_PROBE_ACAO` (experiment hooks that replace response headers) exist only in probe builds |
| Reversal | Replace the iframe host with dedicated webviews; the bridge protocol (`postMessage` + `event.source` authentication) carries over |
