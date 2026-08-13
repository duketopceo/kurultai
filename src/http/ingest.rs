//! Loopback dump ingest: `POST /ingest` (not under `/api/`).
//!
//! Requires `KURULTAI_INGEST_SECRET` (route disabled when unset). Peer must be
//! loopback. Shared-secret compare is constant-time.

use crate::embed::Embedder;
use crate::error::Result;
use crate::hashutil::sha256_hex;
use crate::ingest::dump::{self, DumpFormat};
use crate::quality::{apply_gate, evaluate};
use crate::store::Store;
use crate::types::TrustLane;
use axum::body::Bytes;
use axum::extract::{ConnectInfo, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::Arc;

/// Resolve ingest secret from env only (required — no config fallback).
pub fn resolve_ingest_secret() -> Option<String> {
    std::env::var("KURULTAI_INGEST_SECRET")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn secrets_equal(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.bytes()
        .zip(b.bytes())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

fn is_loopback(addr: &SocketAddr) -> bool {
    match addr.ip() {
        std::net::IpAddr::V4(v4) => v4.is_loopback(),
        std::net::IpAddr::V6(v6) => v6.is_loopback(),
    }
}

fn extract_secret(headers: &HeaderMap) -> Option<String> {
    if let Some(v) = headers.get("x-kurultai-ingest-secret") {
        let s = v.to_str().ok()?.trim();
        if !s.is_empty() {
            return Some(s.to_string());
        }
    }
    if let Some(v) = headers.get(axum::http::header::AUTHORIZATION) {
        let value = v.to_str().ok()?;
        let lower = value.to_ascii_lowercase();
        if lower.starts_with("bearer ") {
            let token = value["bearer ".len()..].trim();
            if !token.is_empty() {
                return Some(token.to_string());
            }
        }
    }
    None
}

#[derive(Clone)]
pub struct IngestState {
    pub store: Arc<dyn Store>,
    pub embedder: Arc<dyn Embedder>,
    pub secret: String,
}

#[derive(Debug, Deserialize)]
pub struct IngestQuery {
    /// Optional dump format override: markdown|json|ndjson|txt|text
    pub format: Option<String>,
    /// Optional logical path for stable source_id (default: ingest/body).
    pub name: Option<String>,
}

fn parse_format(raw: Option<&str>, content_type: Option<&str>) -> DumpFormat {
    if let Some(f) = raw {
        match f.trim().to_ascii_lowercase().as_str() {
            "md" | "markdown" => return DumpFormat::Markdown,
            "json" => return DumpFormat::Json,
            "jsonl" | "ndjson" => return DumpFormat::Ndjson,
            "txt" | "text" | "plain" => return DumpFormat::PlainText,
            _ => {}
        }
    }
    if let Some(ct) = content_type {
        let ct = ct.to_ascii_lowercase();
        if ct.contains("application/json") {
            return DumpFormat::Json;
        }
        if ct.contains("ndjson") || ct.contains("jsonl") {
            return DumpFormat::Ndjson;
        }
        if ct.contains("text/markdown") {
            return DumpFormat::Markdown;
        }
        if ct.contains("text/plain") {
            return DumpFormat::PlainText;
        }
    }
    DumpFormat::PlainText
}

/// Mount `POST /ingest` when a secret is configured.
pub fn routes(state: IngestState) -> Router {
    Router::new()
        .route("/ingest", post(ingest_post))
        .with_state(state)
}

async fn ingest_post(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<IngestState>,
    headers: HeaderMap,
    Query(q): Query<IngestQuery>,
    body: Bytes,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    if !is_loopback(&addr) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "ok": false, "error": "loopback only" })),
        ));
    }

    let provided = extract_secret(&headers).ok_or((
        StatusCode::UNAUTHORIZED,
        Json(json!({ "ok": false, "error": "missing ingest secret" })),
    ))?;
    if !secrets_equal(&provided, &state.secret) {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "ok": false, "error": "invalid ingest secret" })),
        ));
    }

    if body.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": "empty body" })),
        ));
    }

    let ct = headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok());
    let format = parse_format(q.format.as_deref(), ct);
    let rel_path = q
        .name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("ingest/body")
        .replace('\\', "/");

    let mut atoms = dump::atomize_bytes("ingest", &rel_path, &body, format, Utc::now()).map_err(
        |e| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({ "ok": false, "error": e.to_string() })),
            )
        },
    )?;

    if atoms.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": "no atoms in dump" })),
        ));
    }

    let mut batch_seen = std::collections::HashSet::new();
    for atom in &mut atoms {
        let hash = sha256_hex(&atom.content);
        let outcome = if batch_seen.contains(&hash) {
            crate::quality::GateOutcome::Quarantine {
                reason: format!("exact_duplicate:batch:{hash}"),
            }
        } else {
            evaluate(state.store.as_ref(), atom).await.map_err(|e| {
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(json!({ "ok": false, "error": e.to_string() })),
                )
            })?
        };
        batch_seen.insert(hash);
        apply_gate(atom, outcome);
        if atom.trust_lane == TrustLane::Quarantine {
            atom.embedding = None;
        } else if state.embedder.is_live() {
            let text = format!("{}\n{}", atom.title, atom.content);
            if let Ok(emb) = state.embedder.embed(&text).await {
                atom.embedding = Some(emb);
            }
        }
    }

    state.store.upsert_batch(&atoms).await.map_err(|e| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "ok": false, "error": e.to_string() })),
        )
    })?;

    let atom_ids: Vec<String> = atoms.iter().map(|a| a.id.clone()).collect();
    let all_trusted = atoms.iter().all(|a| a.trust_lane == TrustLane::Trusted);
    let lane = if all_trusted {
        "trusted"
    } else {
        "quarantine"
    };
    let quarantine_reason = atoms
        .iter()
        .find_map(|a| a.quarantine_reason.clone());

    // Minimal response — no brain dump.
    Ok((
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "atom_ids": atom_ids,
            "lane": lane,
            "quarantine_reason": quarantine_reason,
        })),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embed::NullEmbedder;
    use crate::store::SqliteVecStore;
    use axum::body::Body;
    use axum::http::{Request, StatusCode as Sc};
    use tower::ServiceExt;

    fn state(secret: &str) -> IngestState {
        let dir = tempfile::tempdir().unwrap();
        // Leak tempdir path for test process lifetime (test-only).
        let path = dir.path().join("store.db");
        let store = Arc::new(SqliteVecStore::open(&path, 4).unwrap());
        std::mem::forget(dir);
        IngestState {
            store,
            embedder: Arc::new(NullEmbedder::new(4)),
            secret: secret.into(),
        }
    }

    async fn call(
        app: Router,
        addr: SocketAddr,
        secret: Option<&str>,
        body: &str,
        format: &str,
    ) -> (Sc, Value) {
        let mut builder = Request::builder()
            .method("POST")
            .uri(format!("/ingest?format={format}&name=hook.md"))
            .header("content-type", "text/markdown");
        if let Some(s) = secret {
            builder = builder.header("x-kurultai-ingest-secret", s);
        }
        let req = builder.body(Body::from(body.to_string())).unwrap();
        // Inject ConnectInfo via extensions (tower/axum test pattern).
        let mut req = req;
        req.extensions_mut().insert(ConnectInfo(addr));
        let resp = app.oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let v: Value = serde_json::from_slice(&bytes).unwrap_or(json!({}));
        (status, v)
    }

    #[tokio::test]
    async fn rejects_missing_secret() {
        let app = routes(state("s3cret"));
        let (status, v) = call(
            app,
            "127.0.0.1:1".parse().unwrap(),
            None,
            "---\ntags: [ops]\n---\n\nBody with enough detail for ingest webhook quality gate pass.\n",
            "markdown",
        )
        .await;
        assert_eq!(status, Sc::UNAUTHORIZED);
        assert_eq!(v["ok"], false);
    }

    #[tokio::test]
    async fn rejects_non_loopback() {
        let app = routes(state("s3cret"));
        let (status, v) = call(
            app,
            "8.8.8.8:1".parse().unwrap(),
            Some("s3cret"),
            "---\ntags: [ops]\n---\n\nBody with enough detail for ingest webhook quality gate pass.\n",
            "markdown",
        )
        .await;
        assert_eq!(status, Sc::FORBIDDEN);
        assert_eq!(v["ok"], false);
    }

    #[tokio::test]
    async fn accepts_loopback_with_secret() {
        let app = routes(state("s3cret"));
        let body = "---\ntags: [ops]\n---\n\nBody with enough detail for ingest webhook quality gate pass.\n";
        let (status, v) = call(
            app,
            "127.0.0.1:9".parse().unwrap(),
            Some("s3cret"),
            body,
            "markdown",
        )
        .await;
        assert_eq!(status, Sc::OK, "{v}");
        assert_eq!(v["ok"], true);
        assert!(v["atom_ids"].as_array().unwrap().len() >= 1);
        assert!(v.get("atoms").is_none()); // no brain dump
        assert!(v["lane"].is_string());
    }

    #[test]
    fn resolve_ingest_secret_reads_env() {
        std::env::set_var("KURULTAI_INGEST_SECRET", "  abc  ");
        assert_eq!(resolve_ingest_secret().as_deref(), Some("abc"));
        std::env::remove_var("KURULTAI_INGEST_SECRET");
        assert_eq!(resolve_ingest_secret(), None);
    }
}
