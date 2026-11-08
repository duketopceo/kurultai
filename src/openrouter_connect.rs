//! OpenRouter "Connect" — the documented OAuth PKCE flow, implemented directly
//! (OpenRouter only ships TypeScript/Python SDKs).
//!
//! Flow (<https://openrouter.ai/docs/guides/overview/auth/oauth>):
//! 1. send the user to `https://openrouter.ai/auth?callback_url=…&code_challenge=…&code_challenge_method=S256`
//! 2. OpenRouter redirects back with `?code=…` (and our `state`, unchanged)
//! 3. `POST /api/v1/auth/keys {code, code_verifier, code_challenge_method}` → `{key}`
//!
//! **Expiry:** as of 2026-11 the documented exchange body has NO expiry / name /
//! limit field (only `key_label` on the auth URL, which we send). So we cannot
//! request an expiring key. Accounts that require expiring keys answer 401
//! "API keys without an expiration date are not allowed"; [`explain_openrouter_error`]
//! maps that to an actionable message (here and in `embed` / `synthesize`).
//!
//! Secrets (code, verifier, key) are never logged; [`Pkce`] and [`ConnectedKey`]
//! redact in `Debug`.

use crate::error::{KurultaiError, Result};
use crate::security::SecretString;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const OPENROUTER_ORIGIN: &str = "https://openrouter.ai";
pub const APP_NAME: &str = "Kurultai";
/// Pending-flow lifetime. OpenRouter codes live 10 min; keep ours the same.
pub const STATE_TTL: Duration = Duration::from_secs(600);
const MAX_PENDING: usize = 64;

const EXPIRY_NEEDLE: &str = "without an expiration date";

/// Actionable text for OpenRouter's "no non-expiring keys" 401. Returns `None`
/// for any other status/body. Shared by connect, embed and synthesize.
pub fn explain_openrouter_error(status: u16, body: &str) -> Option<String> {
    if status != 401 || !body.to_ascii_lowercase().contains(EXPIRY_NEEDLE) {
        return None;
    }
    Some(
        "OpenRouter rejected the API key: this account requires keys with an expiration date. \
         Re-run `kurultai connect openrouter`, or create a key with an expiration at \
         https://openrouter.ai/settings/keys and set OPENROUTER_API_KEY (or the key file)."
            .to_string(),
    )
}

// ---------------------------------------------------------------- PKCE

/// RFC 7636 verifier + S256 challenge. Verifier is redacted in `Debug`.
#[derive(Clone)]
pub struct Pkce {
    verifier: String,
    pub challenge: String,
}

impl fmt::Debug for Pkce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pkce")
            .field("verifier", &"***")
            .field("challenge", &self.challenge)
            .finish()
    }
}

/// `base64url(sha256(verifier))`, no padding (RFC 7636 §4.2).
pub fn s256_challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

impl Pkce {
    /// Fresh 32 random bytes -> 43-char verifier.
    pub fn generate() -> Self {
        let bytes: [u8; 32] = rand::random();
        Self::from_verifier(URL_SAFE_NO_PAD.encode(bytes))
    }

    pub fn from_verifier(verifier: String) -> Self {
        let challenge = s256_challenge(&verifier);
        Self {
            verifier,
            challenge,
        }
    }

    pub fn verifier(&self) -> &str {
        &self.verifier
    }
}

/// Random opaque `state` (CSRF + flow lookup).
pub fn random_state() -> String {
    let bytes: [u8; 24] = rand::random();
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Build the authorization URL against `origin` (injectable for tests).
pub fn build_auth_url(
    origin: &str,
    callback_url: &str,
    challenge: &str,
    state: &str,
) -> Result<String> {
    let mut url = reqwest::Url::parse(&format!("{}/auth", origin.trim_end_matches('/')))
        .map_err(|e| KurultaiError::config(format!("bad OpenRouter origin: {e}")))?;
    url.query_pairs_mut()
        .append_pair("callback_url", callback_url)
        .append_pair("code_challenge", challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("key_label", APP_NAME)
        .append_pair("state", state);
    Ok(url.to_string())
}

// ---------------------------------------------------------------- errors

#[derive(Debug, thiserror::Error)]
pub enum ConnectError {
    #[error("{0}")]
    ExpiryRequired(String),
    #[error("OpenRouter rejected the code exchange ({status})")]
    Rejected { status: u16 },
    #[error("OpenRouter exchange failed: {0}")]
    Transport(String),
    #[error("OpenRouter response did not contain a key")]
    NoKey,
    #[error("unknown, expired or already-used state")]
    BadState,
    #[error("authorization denied or missing code")]
    NoCode,
}

impl From<ConnectError> for KurultaiError {
    fn from(e: ConnectError) -> Self {
        KurultaiError::Security(e.to_string())
    }
}

// ---------------------------------------------------------------- exchange

/// Minted key; `Debug` is redacted.
pub struct ConnectedKey(SecretString);

impl ConnectedKey {
    pub fn expose(&self) -> &str {
        self.0.expose()
    }
}

impl fmt::Debug for ConnectedKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ConnectedKey(***)")
    }
}

/// `POST {api_base}/api/v1/auth/keys`. `api_base` defaults to [`OPENROUTER_ORIGIN`].
pub async fn exchange_code(
    client: &reqwest::Client,
    api_base: &str,
    code: &str,
    verifier: &str,
) -> std::result::Result<ConnectedKey, ConnectError> {
    let resp = client
        .post(format!(
            "{}/api/v1/auth/keys",
            api_base.trim_end_matches('/')
        ))
        .header("HTTP-Referer", "https://github.com/duketopceo/kurultai")
        .header("X-Title", APP_NAME)
        .json(&serde_json::json!({
            "code": code,
            "code_verifier": verifier,
            "code_challenge_method": "S256",
        }))
        .send()
        .await
        // reqwest errors can embed the URL only; no body/secret. Strip it anyway.
        .map_err(|e| ConnectError::Transport(e.without_url().to_string()))?;
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        if let Some(msg) = explain_openrouter_error(status.as_u16(), &body) {
            return Err(ConnectError::ExpiryRequired(msg));
        }
        return Err(ConnectError::Rejected {
            status: status.as_u16(),
        });
    }
    let v: serde_json::Value = serde_json::from_str(&body).map_err(|_| ConnectError::NoKey)?;
    match v.get("key").and_then(|k| k.as_str()).map(str::trim) {
        Some(k) if !k.is_empty() => Ok(ConnectedKey(SecretString::new(k.to_string()))),
        _ => Err(ConnectError::NoKey),
    }
}

// ---------------------------------------------------------------- pending flows

struct Pending {
    pkce: Pkce,
    created: Instant,
}

/// In-memory, single-use, TTL-bound map `state -> PKCE verifier`.
#[derive(Clone)]
pub struct PendingFlows {
    inner: Arc<Mutex<HashMap<String, Pending>>>,
    ttl: Duration,
}

impl Default for PendingFlows {
    fn default() -> Self {
        Self::with_ttl(STATE_TTL)
    }
}

impl fmt::Debug for PendingFlows {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PendingFlows(***)")
    }
}

impl PendingFlows {
    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            inner: Arc::default(),
            ttl,
        }
    }

    /// Register a new flow; returns `(state, challenge)`.
    pub fn begin(&self) -> (String, String) {
        let pkce = Pkce::generate();
        let state = random_state();
        let challenge = pkce.challenge.clone();
        let mut map = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let ttl = self.ttl;
        map.retain(|_, p| p.created.elapsed() < ttl);
        if map.len() >= MAX_PENDING {
            // Drop the oldest rather than grow without bound.
            if let Some(oldest) = map
                .iter()
                .min_by_key(|(_, p)| p.created)
                .map(|(k, _)| k.clone())
            {
                map.remove(&oldest);
            }
        }
        map.insert(
            state.clone(),
            Pending {
                pkce,
                created: Instant::now(),
            },
        );
        (state, challenge)
    }

    /// Consume `state` (single use). Expired / unknown -> `BadState`.
    pub fn take(&self, state: &str) -> std::result::Result<Pkce, ConnectError> {
        let mut map = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        match map.remove(state) {
            Some(p) if p.created.elapsed() < self.ttl => Ok(p.pkce),
            _ => Err(ConnectError::BadState),
        }
    }
}

// ---------------------------------------------------------------- storage

/// Where a connected key lands. Tests use a temp-dir [`FileKeyStore`].
pub trait KeyStore: Send + Sync {
    /// Persist; returns a human description of the location (never the key).
    fn store(&self, key: &str) -> Result<String>;
}

/// 0600 key file, written atomically (temp file in the same dir + rename).
/// Default path is the one `api_key_from_keyfile` reads, so `build_embedder`
/// and the synthesizer pick it up (env still wins).
#[derive(Debug, Clone)]
pub struct FileKeyStore {
    path: PathBuf,
}

impl FileKeyStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// `KURULTAI_API_KEY_FILE` or `<config-dir>/openrouter.key`.
    pub fn default_location() -> Result<Self> {
        let path = std::env::var("KURULTAI_API_KEY_FILE")
            .ok()
            .filter(|p| !p.trim().is_empty())
            .map(PathBuf::from)
            .or_else(crate::security::key_file_path)
            .ok_or_else(|| KurultaiError::config("could not resolve key file path"))?;
        Ok(Self::new(path))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl KeyStore for FileKeyStore {
    fn store(&self, key: &str) -> Result<String> {
        use std::io::Write;
        let dir = self
            .path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        std::fs::create_dir_all(dir)?;
        let mut tmp = tempfile::NamedTempFile::new_in(dir)?; // created 0600 on unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            tmp.as_file()
                .set_permissions(std::fs::Permissions::from_mode(0o600))?;
        }
        tmp.write_all(key.trim().as_bytes())?;
        tmp.write_all(b"\n")?;
        tmp.as_file().sync_all()?;
        tmp.persist(&self.path)
            .map_err(|e| KurultaiError::Io(e.error))?;
        Ok(format!("key file {}", self.path.display()))
    }
}

/// Best-effort extra copy in the local `omaseal` keyring (`kurultai/openrouter`).
/// Returns true when stored. The key goes over stdin, never argv.
pub fn store_in_omaseal(key: &str) -> bool {
    use std::io::Write;
    use std::process::{Command, Stdio};
    if Command::new("omaseal")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_err()
    {
        return false;
    }
    let Ok(mut child) = Command::new("omaseal")
        .args(["set", "kurultai", "openrouter"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    else {
        return false;
    };
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(key.as_bytes());
    }
    child.wait().map(|s| s.success()).unwrap_or(false)
}

/// Finish a flow: consume state, exchange, store. Shared by CLI and daemon.
pub async fn complete_flow(
    flows: &PendingFlows,
    client: &reqwest::Client,
    api_base: &str,
    store: &dyn KeyStore,
    state: &str,
    code: &str,
) -> std::result::Result<String, KurultaiError> {
    let pkce = flows.take(state)?;
    let key = exchange_code(client, api_base, code, pkce.verifier()).await?;
    store.store(key.expose())
}

// ---------------------------------------------------------------- CLI

pub const SUCCESS_HTML: &str = "<!doctype html><meta charset=utf-8><title>Kurultai</title>\
<body style=\"font-family:system-ui;max-width:32rem;margin:4rem auto\">\
<h1>OpenRouter connected</h1><p>You can close this tab and return to Kurultai.</p></body>";

pub fn failure_html(msg: &str) -> String {
    let safe: String = msg
        .chars()
        .map(|c| match c {
            '<' => '\u{2039}',
            '>' => '\u{203a}',
            '&' => '+',
            c => c,
        })
        .collect();
    format!(
        "<!doctype html><meta charset=utf-8><title>Kurultai</title>\
<body style=\"font-family:system-ui;max-width:32rem;margin:4rem auto\">\
<h1>OpenRouter connect failed</h1><p>{safe}</p></body>"
    )
}

#[derive(Debug)]
pub struct CliOptions {
    pub port: u16,
    pub no_open: bool,
    /// Origin of the OpenRouter site/API (tests inject a fake).
    pub origin: String,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            port: 0,
            no_open: false,
            origin: OPENROUTER_ORIGIN.to_string(),
        }
    }
}

/// Loopback flow: bind 127.0.0.1, print/open the auth URL, wait for
/// `GET /callback`, exchange, store. Returns the storage description.
pub async fn run_cli(opts: CliOptions, store: &dyn KeyStore) -> Result<String> {
    use axum::extract::{Query, State};
    use axum::response::Html;
    use axum::routing::get;

    let listener = tokio::net::TcpListener::bind(("127.0.0.1", opts.port)).await?;
    let port = listener.local_addr()?.port();
    let callback = format!("http://127.0.0.1:{port}/callback");
    let flows = PendingFlows::default();
    let (state, challenge) = flows.begin();
    let url = build_auth_url(&opts.origin, &callback, &challenge, &state)?;

    type Done = Arc<Mutex<Option<tokio::sync::oneshot::Sender<(String, String)>>>>;
    let (tx, rx) = tokio::sync::oneshot::channel::<(String, String)>();
    let done: Done = Arc::new(Mutex::new(Some(tx)));
    async fn cb(
        State(done): State<Done>,
        Query(q): Query<HashMap<String, String>>,
    ) -> Html<String> {
        let (Some(code), Some(state)) = (q.get("code"), q.get("state")) else {
            return Html(failure_html("Authorization was denied or incomplete."));
        };
        if let Some(tx) = done.lock().unwrap_or_else(|e| e.into_inner()).take() {
            let _ = tx.send((code.clone(), state.clone()));
        }
        Html(
            "<!doctype html><meta charset=utf-8><title>Kurultai</title>\
<body style=\"font-family:system-ui;max-width:32rem;margin:4rem auto\">\
<h1>Almost there</h1><p>Return to your terminal.</p></body>"
                .to_string(),
        )
    }
    let app = axum::Router::new()
        .route("/callback", get(cb))
        .with_state(done);
    let server = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    println!("Authorize Kurultai on OpenRouter:\n  {url}");
    if !opts.no_open && std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        crate::webui::open_browser(&url);
    }

    let got = tokio::time::timeout(STATE_TTL, rx).await;
    let result = match got {
        Ok(Ok((code, st))) => {
            let client = reqwest::Client::new();
            complete_flow(&flows, &client, &opts.origin, store, &st, &code).await
        }
        _ => Err(KurultaiError::from(ConnectError::NoCode)),
    };
    server.abort();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::routing::post;
    use axum::{http::StatusCode, Json, Router};

    #[test]
    fn rfc7636_appendix_b_vector() {
        assert_eq!(
            s256_challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn generated_verifier_is_valid_rfc7636() {
        let p = Pkce::generate();
        assert!((43..=128).contains(&p.verifier().len()));
        assert!(p
            .verifier()
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
        assert_eq!(p.challenge, s256_challenge(p.verifier()));
        assert_ne!(Pkce::generate().verifier(), p.verifier());
    }

    #[test]
    fn auth_url_carries_s256_params() {
        let u = build_auth_url(
            "https://openrouter.ai/",
            "http://127.0.0.1:1/callback",
            "CH",
            "ST",
        )
        .unwrap();
        let url = reqwest::Url::parse(&u).unwrap();
        assert_eq!(url.path(), "/auth");
        let q: HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(q["callback_url"], "http://127.0.0.1:1/callback");
        assert_eq!(q["code_challenge"], "CH");
        assert_eq!(q["code_challenge_method"], "S256");
        assert_eq!(q["state"], "ST");
        assert_eq!(q["key_label"], "Kurultai");
        assert!(!q.contains_key("expires_at"));
    }

    #[test]
    fn expiry_message_mapping() {
        let body = r#"{"error":{"message":"API keys without an expiration date are not allowed"}}"#;
        let m = explain_openrouter_error(401, body).unwrap();
        assert!(m.contains("kurultai connect openrouter"));
        assert!(m.contains("openrouter.ai/settings/keys"));
        assert!(explain_openrouter_error(403, body).is_none());
        assert!(explain_openrouter_error(401, "bad key").is_none());
    }

    #[test]
    fn secrets_absent_from_debug() {
        let p = Pkce::from_verifier("VERIFIER-SECRET".into());
        assert!(!format!("{p:?}").contains("VERIFIER-SECRET"));
        let k = ConnectedKey(SecretString::new("sk-or-secret".into()));
        assert!(!format!("{k:?}").contains("sk-or-secret"));
        let flows = PendingFlows::default();
        flows.begin();
        assert!(!format!("{flows:?}").contains("Pkce"));
    }

    #[test]
    fn state_single_use_and_ttl() {
        let flows = PendingFlows::default();
        let (st, _) = flows.begin();
        assert!(flows.take("nope").is_err());
        assert!(flows.take(&st).is_ok());
        assert!(matches!(flows.take(&st), Err(ConnectError::BadState)));

        let short = PendingFlows::with_ttl(Duration::from_millis(1));
        let (st, _) = short.begin();
        std::thread::sleep(Duration::from_millis(10));
        assert!(matches!(short.take(&st), Err(ConnectError::BadState)));
    }

    #[test]
    fn file_store_is_0600_and_atomic_overwrite() {
        let dir = tempfile::tempdir().unwrap();
        let store = FileKeyStore::new(dir.path().join("sub/openrouter.key"));
        store.store("sk-one").unwrap();
        store.store("sk-two").unwrap();
        let raw = std::fs::read_to_string(store.path()).unwrap();
        assert_eq!(raw.trim(), "sk-two");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(store.path())
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o600);
        }
        assert_eq!(
            std::fs::read_dir(store.path().parent().unwrap())
                .unwrap()
                .count(),
            1
        );
    }

    /// Fake OpenRouter: asserts the PKCE pair and returns a key, or 401 expiry.
    async fn fake_server(expiry_401: bool) -> String {
        let app = Router::new().route(
            "/api/v1/auth/keys",
            post(move |Json(b): Json<serde_json::Value>| async move {
                if expiry_401 {
                    return (
                        StatusCode::UNAUTHORIZED,
                        Json(serde_json::json!({"error":{"message":
                            "API keys without an expiration date are not allowed"}})),
                    );
                }
                let ok = b["code"] == "the-code"
                    && b["code_challenge_method"] == "S256"
                    && b["code_verifier"]
                        .as_str()
                        .map(|v| s256_challenge(v).len() == 43)
                        .unwrap_or(false);
                if ok {
                    (
                        StatusCode::OK,
                        Json(serde_json::json!({"key":"sk-or-v1-fake"})),
                    )
                } else {
                    (StatusCode::BAD_REQUEST, Json(serde_json::json!({})))
                }
            }),
        );
        let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = l.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(l, app).await.unwrap() });
        format!("http://{addr}")
    }

    #[tokio::test]
    async fn exchange_stores_key_to_tempfile() {
        let base = fake_server(false).await;
        let dir = tempfile::tempdir().unwrap();
        let store = FileKeyStore::new(dir.path().join("openrouter.key"));
        let flows = PendingFlows::default();
        let (st, _) = flows.begin();
        let c = reqwest::Client::new();
        complete_flow(&flows, &c, &base, &store, &st, "the-code")
            .await
            .unwrap();
        assert_eq!(
            std::fs::read_to_string(store.path()).unwrap().trim(),
            "sk-or-v1-fake"
        );
        // replay with the same state is refused
        let again = complete_flow(&flows, &c, &base, &store, &st, "the-code").await;
        assert!(again.is_err());
    }

    #[tokio::test]
    async fn state_mismatch_never_hits_network() {
        let dir = tempfile::tempdir().unwrap();
        let store = FileKeyStore::new(dir.path().join("k"));
        let flows = PendingFlows::default();
        let c = reqwest::Client::new();
        // unroutable base: would error as Transport if contacted
        let e = complete_flow(&flows, &c, "http://127.0.0.1:1", &store, "bogus", "x")
            .await
            .unwrap_err();
        assert!(e.to_string().contains("state"));
        assert!(!store.path().exists());
    }

    #[tokio::test]
    async fn exchange_maps_expiry_401() {
        let base = fake_server(true).await;
        let c = reqwest::Client::new();
        let e = exchange_code(&c, &base, "the-code", "v").await.unwrap_err();
        assert!(matches!(e, ConnectError::ExpiryRequired(_)));
        assert!(e.to_string().contains("expiration"));
    }
}
