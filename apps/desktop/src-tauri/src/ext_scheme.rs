//! The `atabey-ext://` scheme that serves extension UI pages into sandboxed iframes (R-34, AC-17).
//!
//! Pure logic only: parsing a request into an extension id and a file path, refusing anything
//! that could leave the extension's folder, and building the response with its security headers.
//! Where the files come from is behind [`UiAssets`]. The T-04 spike (ADR-0007) serves embedded
//! fixtures; T-22 serves installed extensions from disk. Registering the scheme with Tauri
//! happens in the caller, so this module is testable without a window.
//!
//! The host name is the security decision measured by the spike. Tauri classifies a frame by its
//! URL, and on Windows a custom scheme is `http://<scheme>.<host>/`. A host that is not
//! `localhost` makes that URL non-local, so Tauri denies app commands from it (ADR-0007).
//! [`respond`] therefore serves [`ISOLATED_HOST`] only.

use std::borrow::Cow;

use atabey_types::ExtensionId;
use tauri::http::{HeaderName, HeaderValue, Method, Request, Response, StatusCode, Uri, header};

/// The URI scheme.
pub const SCHEME: &str = "atabey-ext";

/// The host name extension pages are served from.
///
/// Never `localhost` and never ending in `.localhost`: on Windows Tauri treats
/// `http://atabey-ext.localhost/` as a local (trusted) frame (ADR-0007, `is_local_url`).
pub const ISOLATED_HOST: &str = "sandbox";

/// The host of the v0.5 design (`atabey-ext://localhost/…`), rejected by ADR-0007. [`respond`]
/// refuses it; only the probe serves it (through [`respond_on_hosts`]) to keep measuring it.
pub const LOCALHOST_HOST: &str = "localhost";

/// What S0 spec §5.12 prescribed for extension pages before amendment A-4. It does **not** work:
/// in a sandboxed (opaque-origin) document `'self'` matches nothing, so the page's own scripts,
/// styles and JSON are blocked (measured by the T-04 spike, ADR-0007). Kept only so the spike can
/// show the difference; [`extension_csp`] is what is served.
pub const SPEC_CSP: &str = "default-src 'self'";

/// `Access-Control-Allow-Origin` of every response. A sandboxed page has the origin `null`, and
/// module scripts and `fetch()` need CORS (measured: without it `<script type="module">` and
/// `fetch("./file.json")` fail while classic scripts and stylesheets load).
pub const ALLOW_ORIGIN: &str = "null";

/// The Content-Security-Policy of an extension page: its own folder and nothing else.
///
/// Scoped to `<origin>/<extension id>/`, so a page can load its own files but not the main
/// window's (`tauri://localhost`) and not another extension's (measured on macOS and Windows,
/// ADR-0007).
#[must_use]
pub fn extension_csp(windows: bool, host: &str, extension: &ExtensionId) -> String {
    let origin = if windows {
        format!("http://{SCHEME}.{host}")
    } else {
        format!("{SCHEME}://{host}")
    };
    format!("default-src {origin}/{extension}/")
}

/// Where extension UI files come from.
pub trait UiAssets {
    /// The file `path` (extension-relative, `/`-separated, already validated) of `extension`.
    fn get(&self, extension: &ExtensionId, path: &str) -> Option<Cow<'static, [u8]>>;
}

/// Why a request path was refused before touching any asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathError {
    /// The path is not `/<extension id>/<file path>`.
    Malformed,
    /// The extension id has characters outside `a-z 0-9 . -`.
    BadExtensionId,
    /// The file path tries to leave the extension folder or hides a separator.
    Traversal,
}

/// Splits the request path `/<extension id>/<file path>` and validates both parts.
///
/// # Errors
/// [`PathError`] for anything that is not a plain, relative path inside one extension.
pub fn parse(uri: &Uri) -> Result<(ExtensionId, String), PathError> {
    let path = uri.path().strip_prefix('/').ok_or(PathError::Malformed)?;
    let (extension, file) = path.split_once('/').ok_or(PathError::Malformed)?;
    let valid_id = !extension.is_empty()
        && extension
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'.' || b == b'-')
        && !extension.starts_with(['.', '-'])
        && !extension.ends_with(['.', '-'])
        && !extension.contains("..");
    if !valid_id {
        return Err(PathError::BadExtensionId);
    }
    if file.is_empty() {
        return Err(PathError::Malformed);
    }
    for segment in file.split('/') {
        if segment.is_empty() {
            return Err(PathError::Malformed);
        }
        if segment == "." || segment == ".." || has_hidden_separator(segment) {
            return Err(PathError::Traversal);
        }
    }
    Ok((ExtensionId::from(extension), file.to_owned()))
}

/// Backslashes, NUL and the percent-encoded forms of `.`, `/`, `\` and NUL.
fn has_hidden_separator(segment: &str) -> bool {
    if segment.contains(['\\', '\0']) {
        return true;
    }
    let lower = segment.to_ascii_lowercase();
    ["%2e", "%2f", "%5c", "%00"]
        .iter()
        .any(|encoded| lower.contains(encoded))
}

/// The URL a sandboxed iframe loads for `file` of `extension`, in this OS's form.
#[must_use]
pub fn ui_url(host: &str, extension: &ExtensionId, file: &str) -> String {
    ui_url_for(cfg!(windows), host, extension, file)
}

/// [`ui_url`] for an explicit OS form, so both can be tested anywhere.
///
/// macOS and Linux: `atabey-ext://HOST/ext/file`. Windows (WebView2 has no custom schemes):
/// `http://atabey-ext.HOST/ext/file`, which wry maps back before calling the handler.
#[must_use]
pub fn ui_url_for(windows: bool, host: &str, extension: &ExtensionId, file: &str) -> String {
    if windows {
        format!("http://{SCHEME}.{host}/{extension}/{file}")
    } else {
        format!("{SCHEME}://{host}/{extension}/{file}")
    }
}

/// The `Content-Type` for a file name. Module scripts need a JavaScript type.
#[must_use]
pub fn mime_for(file: &str) -> &'static str {
    let extension = file.rsplit_once('.').map_or("", |(_, ext)| ext);
    match extension.to_ascii_lowercase().as_str() {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "woff2" => "font/woff2",
        _ => "application/octet-stream",
    }
}

fn reply(
    status: StatusCode,
    content_type: &'static str,
    csp: &str,
    body: Cow<'static, [u8]>,
) -> Response<Cow<'static, [u8]>> {
    let mut response = Response::new(body);
    *response.status_mut() = status;
    let headers = response.headers_mut();
    let fixed: [(HeaderName, &'static str); 5] = [
        (header::CONTENT_TYPE, content_type),
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        (header::CACHE_CONTROL, "no-store"),
        (header::ACCESS_CONTROL_ALLOW_ORIGIN, ALLOW_ORIGIN),
        (header::CONTENT_SECURITY_POLICY, "default-src 'none'"),
    ];
    for (name, value) in fixed {
        headers.insert(name, HeaderValue::from_static(value));
    }
    // The extension id and host are validated, so this cannot fail; if it ever did, the
    // restrictive `default-src 'none'` above stays.
    if let Ok(value) = HeaderValue::from_str(csp) {
        headers.insert(header::CONTENT_SECURITY_POLICY, value);
    }
    response
}

fn refuse(status: StatusCode, reason: &'static str) -> Response<Cow<'static, [u8]>> {
    reply(
        status,
        "text/plain; charset=utf-8",
        "default-src 'none'",
        Cow::Borrowed(reason.as_bytes()),
    )
}

/// Answers one request: the file with its security headers, or an error status.
///
/// 405 for anything but GET, 400 for a host other than [`ISOLATED_HOST`] or a path that is not
/// plain and relative, 404 for an unknown extension or file.
pub fn respond<A: UiAssets>(
    assets: &A,
    request: &Request<Vec<u8>>,
) -> Response<Cow<'static, [u8]>> {
    respond_on_hosts(assets, request, &[ISOLATED_HOST])
}

/// [`respond`] for an explicit list of hosts. The isolation probe passes [`LOCALHOST_HOST`] as
/// well, so the rejected URL shape stays measured on every Tauri/wry upgrade (ADR-0007).
pub fn respond_on_hosts<A: UiAssets>(
    assets: &A,
    request: &Request<Vec<u8>>,
    hosts: &[&str],
) -> Response<Cow<'static, [u8]>> {
    if request.method() != Method::GET {
        let mut response = refuse(StatusCode::METHOD_NOT_ALLOWED, "method not allowed");
        response
            .headers_mut()
            .insert(header::ALLOW, HeaderValue::from_static("GET"));
        return response;
    }
    let (extension, file) = match parse(request.uri()) {
        Ok(parsed) => parsed,
        Err(_) => return refuse(StatusCode::BAD_REQUEST, "bad path"),
    };
    let host = request.uri().host().unwrap_or_default();
    if !hosts.contains(&host) {
        return refuse(StatusCode::BAD_REQUEST, "bad host");
    }
    match assets.get(&extension, &file) {
        Some(bytes) => {
            let csp = extension_csp(cfg!(windows), host, &extension);
            reply(StatusCode::OK, mime_for(&file), &csp, bytes)
        }
        None => refuse(StatusCode::NOT_FOUND, "not found"),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)] // fixed, valid test inputs
mod tests {
    use super::*;

    struct Fixed;

    impl UiAssets for Fixed {
        fn get(&self, extension: &ExtensionId, path: &str) -> Option<Cow<'static, [u8]>> {
            match (extension.as_str(), path) {
                ("atabey.defter-viewer", "ui/index.html") => {
                    Some(Cow::Borrowed(b"<!doctype html><title>Defter</title>"))
                }
                ("atabey.defter-viewer", "ui/panel.mjs") => Some(Cow::Borrowed(b"export {};")),
                _ => None,
            }
        }
    }

    fn get(uri: &str) -> Response<Cow<'static, [u8]>> {
        let request = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .body(Vec::new())
            .expect("valid test request");
        respond(&Fixed, &request)
    }

    fn header_of<'a>(response: &'a Response<Cow<'static, [u8]>>, name: &HeaderName) -> &'a str {
        response
            .headers()
            .get(name)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
    }

    #[test]
    fn serves_a_known_file_with_security_headers() {
        let response = get("atabey-ext://sandbox/atabey.defter-viewer/ui/index.html");
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.body().as_ref(),
            b"<!doctype html><title>Defter</title>"
        );
        assert_eq!(
            header_of(&response, &header::CONTENT_TYPE),
            "text/html; charset=utf-8"
        );
        assert_eq!(
            header_of(&response, &header::CONTENT_SECURITY_POLICY),
            extension_csp(
                cfg!(windows),
                ISOLATED_HOST,
                &ExtensionId::from("atabey.defter-viewer")
            )
        );
        assert_eq!(
            header_of(&response, &header::ACCESS_CONTROL_ALLOW_ORIGIN),
            "null"
        );
        assert_eq!(
            header_of(&response, &header::X_CONTENT_TYPE_OPTIONS),
            "nosniff"
        );
        assert_eq!(header_of(&response, &header::CACHE_CONTROL), "no-store");
    }

    #[test]
    fn a_host_that_is_not_ours_is_a_400() {
        for uri in [
            "atabey-ext://evil/atabey.defter-viewer/ui/index.html",
            "atabey-ext://sandbox.evil/atabey.defter-viewer/ui/index.html",
            "/atabey.defter-viewer/ui/index.html",
        ] {
            assert_eq!(get(uri).status(), StatusCode::BAD_REQUEST, "{uri}");
        }
    }

    /// ADR-0007 selected the `sandbox` host; the `localhost` shape is served to the probe only.
    #[test]
    fn the_rejected_localhost_host_is_served_only_on_request() {
        let uri = "atabey-ext://localhost/atabey.defter-viewer/ui/index.html";
        assert_eq!(get(uri).status(), StatusCode::BAD_REQUEST);

        let request = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .body(Vec::new())
            .expect("valid test request");
        let response = respond_on_hosts(&Fixed, &request, &[ISOLATED_HOST, LOCALHOST_HOST]);
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            header_of(&response, &header::CONTENT_SECURITY_POLICY),
            extension_csp(
                cfg!(windows),
                LOCALHOST_HOST,
                &ExtensionId::from("atabey.defter-viewer")
            )
        );
    }

    #[test]
    fn the_csp_is_scoped_to_the_extension_folder_in_both_url_forms() {
        let ext = ExtensionId::from("atabey.git-pulse");
        assert_eq!(
            extension_csp(false, ISOLATED_HOST, &ext),
            "default-src atabey-ext://sandbox/atabey.git-pulse/"
        );
        assert_eq!(
            extension_csp(true, ISOLATED_HOST, &ext),
            "default-src http://atabey-ext.sandbox/atabey.git-pulse/"
        );
    }

    #[test]
    fn errors_get_the_most_restrictive_csp_and_no_file_headers_leak() {
        let response = get("atabey-ext://sandbox/atabey.git-pulse/ui/index.html");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            header_of(&response, &header::CONTENT_SECURITY_POLICY),
            "default-src 'none'"
        );
    }

    #[test]
    fn unknown_extension_or_file_is_404() {
        for uri in [
            "atabey-ext://sandbox/atabey.git-pulse/ui/index.html",
            "atabey-ext://sandbox/atabey.defter-viewer/ui/missing.css",
        ] {
            assert_eq!(get(uri).status(), StatusCode::NOT_FOUND, "{uri}");
        }
    }

    #[test]
    fn only_get_is_allowed() {
        for method in [
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::HEAD,
            Method::OPTIONS,
        ] {
            let request = Request::builder()
                .method(method.clone())
                .uri("atabey-ext://sandbox/atabey.defter-viewer/ui/index.html")
                .body(Vec::new())
                .expect("valid test request");
            let response = respond(&Fixed, &request);
            assert_eq!(
                response.status(),
                StatusCode::METHOD_NOT_ALLOWED,
                "{method}"
            );
            assert_eq!(header_of(&response, &header::ALLOW), "GET");
        }
    }

    #[test]
    fn traversal_in_every_spelling_is_a_400() {
        for path in [
            "/atabey.defter-viewer/../atabey.git-pulse/ui/index.html",
            "/atabey.defter-viewer/ui/../../secret.json",
            "/atabey.defter-viewer/ui/%2e%2e/secret.json",
            "/atabey.defter-viewer/ui/%2E%2E/secret.json",
            "/atabey.defter-viewer/ui%2f..%2fsecret.json",
            "/atabey.defter-viewer/ui/..%5csecret.json",
            "/atabey.defter-viewer/ui\\..\\secret.json",
            "/atabey.defter-viewer/ui/index.html%00.png",
            "/atabey.defter-viewer//etc/passwd",
            "/atabey.defter-viewer/",
            "/atabey.defter-viewer",
            "/",
            "/Atabey.Defter-Viewer/ui/index.html",
            "/../ui/index.html",
            "/atabey..defter/ui/index.html",
        ] {
            let uri = format!("atabey-ext://sandbox{path}");
            assert_eq!(get(&uri).status(), StatusCode::BAD_REQUEST, "{uri}");
        }
    }

    #[test]
    fn parse_returns_extension_and_relative_file() {
        let uri: Uri = "atabey-ext://sandbox/atabey.git-pulse/ui/assets/panel.css?v=2"
            .parse()
            .expect("valid uri");
        assert_eq!(
            parse(&uri),
            Ok((
                ExtensionId::from("atabey.git-pulse"),
                "ui/assets/panel.css".to_owned()
            ))
        );
    }

    #[test]
    fn mime_types_cover_what_a_panel_loads() {
        for (file, mime) in [
            ("ui/index.html", "text/html; charset=utf-8"),
            ("ui/panel.mjs", "text/javascript; charset=utf-8"),
            ("ui/attacks.js", "text/javascript; charset=utf-8"),
            ("ui/panel.css", "text/css; charset=utf-8"),
            ("ui/ledger-sample.json", "application/json"),
            ("ui/icon.svg", "image/svg+xml"),
            ("ui/blob.bin", "application/octet-stream"),
            ("ui/no-extension", "application/octet-stream"),
        ] {
            assert_eq!(mime_for(file), mime, "{file}");
        }
    }

    #[test]
    fn url_form_per_os() {
        let ext = ExtensionId::from("atabey.defter-viewer");
        assert_eq!(
            ui_url_for(false, ISOLATED_HOST, &ext, "ui/index.html"),
            "atabey-ext://sandbox/atabey.defter-viewer/ui/index.html"
        );
        assert_eq!(
            ui_url_for(true, ISOLATED_HOST, &ext, "ui/index.html"),
            "http://atabey-ext.sandbox/atabey.defter-viewer/ui/index.html"
        );
        assert_eq!(
            ui_url_for(true, LOCALHOST_HOST, &ext, "ui/index.html"),
            "http://atabey-ext.localhost/atabey.defter-viewer/ui/index.html"
        );
    }

    /// On Windows Tauri trusts `http://<registered scheme>.localhost`; the isolated host must
    /// never fall into that shape (ADR-0007).
    #[test]
    fn isolated_host_is_never_localhost() {
        assert_ne!(ISOLATED_HOST, "localhost");
        assert!(!ISOLATED_HOST.ends_with(".localhost"));
        assert!(!ISOLATED_HOST.is_empty());
    }
}
