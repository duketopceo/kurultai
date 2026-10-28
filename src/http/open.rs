//! `GET /api/open?file=…` — "reveal this source in the OS" for the Brain UI.
//!
//! This route launches a desktop opener on the daemon's host, so it is only
//! mounted for the solo, loopback, unauthenticated daemon and every request is
//! re-checked:
//!
//! * peer is loopback, `Host` is a loopback name, and no proxy-forwarding
//!   headers are present (a tunnel on the same box would otherwise look local);
//! * browser cross-site requests are refused (`Sec-Fetch-Site`, `Origin`) — a
//!   plain `GET` must not be fireable from another web page;
//! * the target canonicalizes inside a configured, indexed source root (so `..`
//!   and symlink escapes are out) and is a document-like file or a plain folder
//!   — never an app bundle or script the OS would execute;
//! * the opener is spawned with the path as a single argv entry (no shell).

use crate::security::resolve_allowed_path;
use axum::extract::{ConnectInfo, Query};
use axum::http::{header, request::Parts, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// File types the UI may reveal. Anything else (scripts, `.command`, `.app`,
/// `.desktop`, installers, …) is refused because the OS opener would run it.
const OPENABLE_EXTENSIONS: &[&str] = &[
    "md", "markdown", "txt", "rst", "log", "json", "jsonl", "csv", "tsv", "toml", "yaml", "yml",
    "pdf", "html", "htm", "png", "jpg", "jpeg", "gif", "webp", "svg", "rs", "ts", "tsx", "js",
    "jsx", "py", "go",
];

type Launcher = dyn Fn(&Path) -> std::io::Result<()> + Send + Sync;

#[derive(Clone)]
pub(crate) struct OpenState {
    roots: Arc<Vec<PathBuf>>,
    launch: Arc<Launcher>,
}

impl OpenState {
    pub(crate) fn new(roots: Vec<PathBuf>) -> Self {
        Self::with_launcher(roots, Arc::new(spawn_os_opener))
    }

    pub(crate) fn with_launcher(roots: Vec<PathBuf>, launch: Arc<Launcher>) -> Self {
        // Canonicalize once so containment compares like with like.
        let roots = roots
            .into_iter()
            .filter_map(|r| r.canonicalize().ok())
            .collect();
        Self {
            roots: Arc::new(roots),
            launch,
        }
    }
}

pub(crate) fn routes(state: OpenState) -> Router {
    Router::new().route(
        "/api/open",
        get(move |parts: Parts, q: Query<HashMap<String, String>>| {
            let state = state.clone();
            async move { api_open(state, parts, q).await }
        }),
    )
}

/// Single argv entry, no shell. Platforms without a known opener refuse.
fn spawn_os_opener(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    let program = "open";
    #[cfg(all(unix, not(target_os = "macos")))]
    let program = "xdg-open";
    #[cfg(not(unix))]
    {
        let _ = path;
        return Err(std::io::Error::other("no opener on this platform"));
    }
    #[cfg(unix)]
    {
        std::process::Command::new(program)
            .arg(path)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map(|_| ())
    }
}

async fn api_open(
    state: OpenState,
    parts: Parts,
    Query(params): Query<HashMap<String, String>>,
) -> axum::response::Response {
    let peer = parts
        .extensions
        .get::<ConnectInfo<SocketAddr>>()
        .map(|c| c.0.ip());
    if let Err(why) = check_caller(peer, &parts.headers) {
        tracing::warn!(reason = why, "api/open refused");
        return refuse(StatusCode::FORBIDDEN, why);
    }
    let Some(file) = params.get("file").filter(|f| !f.trim().is_empty()) else {
        return refuse(StatusCode::BAD_REQUEST, "file is required");
    };
    let target = match resolve_target(&state.roots, file) {
        Ok(t) => t,
        Err(why) => {
            tracing::warn!(reason = why, "api/open refused");
            return refuse(StatusCode::NOT_FOUND, why);
        }
    };
    let launch = Arc::clone(&state.launch);
    match tokio::task::spawn_blocking(move || launch(&target)).await {
        Ok(Ok(())) => (StatusCode::OK, Json(serde_json::json!({ "ok": true }))).into_response(),
        _ => refuse(StatusCode::INTERNAL_SERVER_ERROR, "could not launch opener"),
    }
}

fn refuse(status: StatusCode, why: &str) -> axum::response::Response {
    (
        status,
        Json(serde_json::json!({ "ok": false, "error": why })),
    )
        .into_response()
}

fn is_loopback_host(host: &str) -> bool {
    let h = host.trim();
    let name = if let Some(rest) = h.strip_prefix('[') {
        rest.split(']').next().unwrap_or("")
    } else {
        h.rsplit_once(':').map_or(h, |(n, _)| n)
    };
    name.eq_ignore_ascii_case("localhost") || name == "127.0.0.1" || name == "::1"
}

/// Origin of the caller: same machine, same browser origin, not proxied.
fn check_caller(peer: Option<IpAddr>, headers: &HeaderMap) -> Result<(), &'static str> {
    match peer {
        Some(ip) if ip.is_loopback() => {}
        _ => return Err("only available to local callers"),
    }
    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !is_loopback_host(host) {
        return Err("only available on a loopback host");
    }
    for forwarded in [
        "forwarded",
        "x-forwarded-for",
        "x-forwarded-host",
        "cf-connecting-ip",
        "cf-access-jwt-assertion",
    ] {
        if headers.contains_key(forwarded) {
            return Err("not available through a proxy");
        }
    }
    if let Some(site) = headers.get("sec-fetch-site").and_then(|v| v.to_str().ok()) {
        if !matches!(site, "same-origin" | "none") {
            return Err("cross-site requests are refused");
        }
    }
    if let Some(origin) = headers.get(header::ORIGIN).and_then(|v| v.to_str().ok()) {
        let host = origin.split_once("://").map_or(origin, |(_, h)| h);
        if !is_loopback_host(host) {
            return Err("cross-site requests are refused");
        }
    }
    Ok(())
}

/// Canonical path inside an indexed root that is safe to hand to the opener.
fn resolve_target(roots: &[PathBuf], file: &str) -> Result<PathBuf, &'static str> {
    let path = resolve_allowed_path(file).map_err(|_| "path rejected")?;
    if !roots.iter().any(|root| path.starts_with(root)) {
        return Err("path is outside the indexed source roots");
    }
    let meta = std::fs::metadata(&path).map_err(|_| "file not found")?;
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    if meta.is_dir() {
        // `Foo.app`, `Foo.workflow`, … are directories the OS would run.
        if ext.is_some() {
            return Err("this folder type cannot be opened");
        }
    } else if meta.is_file() {
        if !ext
            .as_deref()
            .is_some_and(|e| OPENABLE_EXTENSIONS.contains(&e))
        {
            return Err("this file type cannot be opened");
        }
    } else {
        return Err("not a file or folder");
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use std::sync::Mutex;
    use tower::ServiceExt;

    struct Fixture {
        _dir: tempfile::TempDir,
        root: PathBuf,
        outside: PathBuf,
        launched: Arc<Mutex<Vec<PathBuf>>>,
        app: Router,
    }

    fn fixture() -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("notes");
        std::fs::create_dir_all(root.join("sub")).unwrap();
        std::fs::write(root.join("a.md"), "hi").unwrap();
        std::fs::write(root.join("run.command"), "#!/bin/sh\n").unwrap();
        std::fs::write(root.join("noext"), "x").unwrap();
        std::fs::create_dir_all(root.join("Evil.app")).unwrap();
        let outside = dir.path().join("secret.md");
        std::fs::write(&outside, "secret").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, root.join("link.md")).unwrap();
        let launched = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&launched);
        let state = OpenState::with_launcher(
            vec![root.clone()],
            Arc::new(move |p: &Path| {
                sink.lock().unwrap().push(p.to_path_buf());
                Ok(())
            }),
        );
        Fixture {
            root: root.canonicalize().unwrap(),
            outside,
            launched,
            app: routes(state),
            _dir: dir,
        }
    }

    async fn get_open(
        f: &Fixture,
        file: &str,
        peer: Option<&str>,
        headers: &[(&str, &str)],
    ) -> StatusCode {
        let uri = format!("/api/open?file={}", urlencode(file));
        let mut req = Request::builder().uri(uri).header("host", "127.0.0.1:7777");
        for (k, v) in headers {
            req = req.header(*k, *v);
        }
        let mut req = req.body(Body::empty()).unwrap();
        if let Some(p) = peer {
            let addr = SocketAddr::new(p.parse().unwrap(), 50000);
            req.extensions_mut().insert(ConnectInfo(addr));
        }
        f.app.clone().oneshot(req).await.unwrap().status()
    }

    fn urlencode(s: &str) -> String {
        s.bytes()
            .map(|b| match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'/' => {
                    (b as char).to_string()
                }
                _ => format!("%{b:02X}"),
            })
            .collect()
    }

    fn p(f: &Fixture, rel: &str) -> String {
        f.root.join(rel).to_string_lossy().into_owned()
    }

    #[tokio::test]
    async fn loopback_opens_indexed_document_and_folder() {
        let f = fixture();
        assert_eq!(
            get_open(&f, &p(&f, "a.md"), Some("127.0.0.1"), &[]).await,
            StatusCode::OK
        );
        assert_eq!(
            get_open(
                &f,
                &p(&f, "sub"),
                Some("::1"),
                &[("sec-fetch-site", "same-origin")]
            )
            .await,
            StatusCode::OK
        );
        let got = f.launched.lock().unwrap().clone();
        assert_eq!(got, vec![f.root.join("a.md"), f.root.join("sub")]);
    }

    #[tokio::test]
    async fn refuses_non_loopback_missing_peer_and_proxied_callers() {
        let f = fixture();
        let file = p(&f, "a.md");
        assert_eq!(
            get_open(&f, &file, Some("10.0.0.5"), &[]).await,
            StatusCode::FORBIDDEN
        );
        assert_eq!(get_open(&f, &file, None, &[]).await, StatusCode::FORBIDDEN);
        for h in [
            "x-forwarded-for",
            "cf-connecting-ip",
            "cf-access-jwt-assertion",
            "forwarded",
        ] {
            assert_eq!(
                get_open(&f, &file, Some("127.0.0.1"), &[(h, "1.2.3.4")]).await,
                StatusCode::FORBIDDEN,
                "{h}"
            );
        }
        assert!(f.launched.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn refuses_cross_site_and_rebound_host() {
        let f = fixture();
        let file = p(&f, "a.md");
        for h in [
            ("sec-fetch-site", "cross-site"),
            ("sec-fetch-site", "same-site"),
        ] {
            assert_eq!(
                get_open(&f, &file, Some("127.0.0.1"), &[h]).await,
                StatusCode::FORBIDDEN,
                "{h:?}"
            );
        }
        assert_eq!(
            get_open(
                &f,
                &file,
                Some("127.0.0.1"),
                &[("origin", "https://evil.example")]
            )
            .await,
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            get_open(
                &f,
                &file,
                Some("127.0.0.1"),
                &[("origin", "http://localhost:7777")]
            )
            .await,
            StatusCode::OK
        );
        // DNS rebinding: attacker hostname resolving to 127.0.0.1.
        let uri = format!("/api/open?file={}", urlencode(&file));
        let mut req = Request::builder()
            .uri(uri)
            .header("host", "rebind.evil.example:7777")
            .body(Body::empty())
            .unwrap();
        req.extensions_mut().insert(ConnectInfo(
            "127.0.0.1:50000".parse::<SocketAddr>().unwrap(),
        ));
        let status = f.app.clone().oneshot(req).await.unwrap().status();
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn refuses_paths_outside_roots_and_traversal() {
        let f = fixture();
        let peer = Some("127.0.0.1");
        for bad in [
            f.outside.to_string_lossy().into_owned(),
            "/etc/passwd".to_string(),
            format!("{}/../secret.md", f.root.display()),
            p(&f, "does-not-exist.md"),
        ] {
            assert_eq!(
                get_open(&f, &bad, peer, &[]).await,
                StatusCode::NOT_FOUND,
                "{bad}"
            );
        }
        #[cfg(unix)]
        assert_eq!(
            get_open(&f, &p(&f, "link.md"), peer, &[]).await,
            StatusCode::NOT_FOUND,
            "symlink escaping the root"
        );
        assert_eq!(get_open(&f, "", peer, &[]).await, StatusCode::BAD_REQUEST);
        assert!(f.launched.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn refuses_executable_types_and_app_bundles() {
        let f = fixture();
        let peer = Some("127.0.0.1");
        for bad in ["run.command", "noext", "Evil.app"] {
            assert_eq!(
                get_open(&f, &p(&f, bad), peer, &[]).await,
                StatusCode::NOT_FOUND,
                "{bad}"
            );
        }
        assert!(f.launched.lock().unwrap().is_empty());
    }

    #[test]
    fn loopback_host_parsing() {
        for ok in [
            "localhost",
            "localhost:7777",
            "127.0.0.1:80",
            "[::1]:7777",
            "LOCALHOST",
        ] {
            assert!(is_loopback_host(ok), "{ok}");
        }
        for bad in [
            "",
            "evil.example",
            "127.0.0.1.evil.example",
            "localhost.evil.example:1",
            "10.0.0.1",
        ] {
            assert!(!is_loopback_host(bad), "{bad}");
        }
    }
}
