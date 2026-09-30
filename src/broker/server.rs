//! Broker HTTP surface — loopback-only axum service.
//!
//! U1 scope: boot the service, resolve the upstream seat token, expose
//! `/health` + `/status`. Board/proxy/outbox land in U2–U5.

use crate::error::{KurultaiError, Result};
use crate::security::{read_agent_key, SecretString};
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

/// What the broker was configured/launched with.
#[derive(Debug, Clone)]
pub struct BrokerOptions {
    /// Upstream instance base URL (no trailing slash).
    pub upstream_url: String,
    /// Loopback port for the TCP listener.
    pub port: u16,
    /// Optional unix socket bound in addition to TCP.
    pub socket: Option<PathBuf>,
    /// Agent-key name override for the upstream seat token.
    pub key_name: Option<String>,
    /// Lane (`dev`/`prod`) for the default credential name.
    pub lane: String,
    /// Path to the broker-local registry db (`broker.db` beside the store).
    pub db_path: PathBuf,
}

#[derive(Clone)]
struct BrokerState {
    upstream_url: Arc<String>,
    /// Resolved at startup; the value never leaves the broker.
    upstream_token: SecretString,
    key_name: Arc<String>,
}

#[derive(Serialize)]
struct Status {
    service: &'static str,
    upstream: String,
    upstream_authenticated: bool,
    key_name: String,
}

pub(crate) fn hostname() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Credential name the broker resolves: `{lane}-broker-{hostname}-agent-token`.
/// Mirrors the `connect` naming so `kurultai connect --codename broker`
/// produces a loadable key.
fn default_key_name(lane: &str) -> String {
    format!(
        "{lane}-broker-{}-agent-token",
        crate::connect::seat_slug(&hostname())
    )
}

/// Resolve the upstream seat token: `KURULTAI_BROKER_KEY` env wins (dev), then
/// `key_name` override, then the lane+hostname default.
fn resolve_upstream_token(opts: &BrokerOptions) -> Result<(SecretString, String)> {
    if let Some(key) = crate::security::api_key_from_env_optional("KURULTAI_BROKER_KEY") {
        return Ok((key, "env:KURULTAI_BROKER_KEY".to_string()));
    }
    let name = opts
        .key_name
        .clone()
        .unwrap_or_else(|| default_key_name(&opts.lane));
    read_agent_key(&name)
        .map(|k| (k, name.clone()))
        .ok_or_else(|| {
            KurultaiError::config(format!(
                "no upstream seat token found for '{name}' — run \
                 `kurultai connect --codename broker` on this device first"
            ))
        })
}

fn router(state: BrokerState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/status", get(status))
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}

async fn status(State(s): State<BrokerState>) -> Json<Status> {
    Json(Status {
        service: "kurultai-broker",
        upstream: (*s.upstream_url).clone(),
        upstream_authenticated: !s.upstream_token.expose().is_empty(),
        key_name: (*s.key_name).clone(),
    })
}

/// Bind a loopback TCP listener — refuses non-loopback addresses outright (R6).
pub async fn bind_loopback(port: u16, host: Option<IpAddr>) -> Result<tokio::net::TcpListener> {
    let addr = match host {
        None => SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port),
        Some(ip) if ip.is_loopback() => SocketAddr::new(ip, port),
        Some(ip) => {
            return Err(KurultaiError::config(format!(
                "broker refuses non-loopback bind {ip} — agents board over loopback only"
            )))
        }
    };
    tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| KurultaiError::Other(anyhow::anyhow!("broker bind {addr}: {e}")))
}

/// Run the broker until killed. Resolves the upstream session first — a broker
/// without upstream auth is worse than none, so we fail fast.
pub async fn run(opts: BrokerOptions) -> Result<()> {
    let (token, key_name) = resolve_upstream_token(&opts)?;
    let registry = std::sync::Arc::new(crate::broker::Registry::open(&opts.db_path)?);
    let upstream = opts.upstream_url.trim_end_matches('/').to_string();
    let state = BrokerState {
        upstream_url: Arc::new(upstream.clone()),
        upstream_token: token,
        key_name: Arc::new(key_name.clone()),
    };
    let app = router(state.clone())
        .merge(crate::broker::board::routes(registry.clone()))
        .merge(crate::broker::relay::routes(
            crate::broker::relay::RelayState {
                registry,
                upstream_url: state.upstream_url,
                upstream_token: state.upstream_token,
                client: reqwest::Client::new(),
            },
        ));

    let listener = bind_loopback(opts.port, None).await?;
    println!(
        "Broker listening on http://127.0.0.1:{} → {upstream} (key: {key_name})",
        opts.port
    );
    tracing::info!(%upstream, port = opts.port, "kurultai broker listening");

    #[cfg(unix)]
    if let Some(sock) = &opts.socket {
        if let Some(dir) = sock.parent() {
            std::fs::create_dir_all(dir)
                .map_err(|e| KurultaiError::config(format!("broker socket dir: {e}")))?;
        }
        let _ = std::fs::remove_file(sock);
        let ul = tokio::net::UnixListener::bind(sock)
            .map_err(|e| KurultaiError::config(format!("broker socket {sock:?}: {e}")))?;
        tracing::info!(socket = %sock.display(), "kurultai broker socket listening");
        let app2 = app.clone();
        tokio::spawn(async move {
            let _ = axum::serve(ul, app2.into_make_service()).await;
        });
    }

    axum::serve(listener, app.into_make_service())
        .await
        .map_err(|e| KurultaiError::Other(anyhow::anyhow!("broker serve: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn refuses_non_loopback_bind() {
        let err = bind_loopback(0, Some("0.0.0.0".parse().unwrap()))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("non-loopback"));
    }

    #[tokio::test]
    async fn loopback_binds() {
        bind_loopback(0, None).await.unwrap();
        bind_loopback(0, Some("127.0.0.1".parse().unwrap()))
            .await
            .unwrap();
        bind_loopback(0, Some("::1".parse().unwrap()))
            .await
            .unwrap();
    }
}
