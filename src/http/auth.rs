//! Hub REST auth (public-mode API keys). Solo/loopback default is no auth.

use crate::hashutil::sha256_hex;
use axum::extract::{Request, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::Response;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HubAuth {
    #[default]
    None,
    ApiKey,
}

#[derive(Debug, Clone, Default)]
pub struct HubGate {
    pub auth: HubAuth,
    pub api_keys: Vec<String>,
}

pub fn parse_hub_auth(raw: Option<&str>) -> HubAuth {
    match raw.map(str::trim).unwrap_or("") {
        "api_key" | "apikey" => HubAuth::ApiKey,
        _ => HubAuth::None,
    }
}

pub fn parse_hub_bind_all(raw: Option<&str>) -> bool {
    matches!(
        raw.map(str::trim).unwrap_or(""),
        "all" | "0.0.0.0" | "public"
    )
}

pub fn keys_from_csv(raw: Option<&str>) -> Vec<String> {
    raw.unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

pub fn resolve_hub_gate_from_env() -> HubGate {
    HubGate {
        auth: parse_hub_auth(std::env::var("KURULTAI_HUB_AUTH").ok().as_deref()),
        api_keys: keys_from_csv(std::env::var("KURULTAI_HUB_API_KEYS").ok().as_deref()),
    }
}

pub fn resolve_bind_all_from_env() -> bool {
    parse_hub_bind_all(std::env::var("KURULTAI_HUB_BIND").ok().as_deref())
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

fn extract_bearer(headers: &HeaderMap) -> Option<String> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    if !value.to_ascii_lowercase().starts_with("bearer ") {
        return None;
    }
    let token = value["bearer ".len()..].trim();
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

pub fn token_accepted(token: &str, keys: &[String]) -> bool {
    let hashed = sha256_hex(token);
    keys.iter()
        .any(|k| secrets_equal(k, token) || secrets_equal(k, &hashed))
}

/// Paths exempt from hub API-key authentication (`/health` and embedded `/ui`).
pub fn path_requires_hub_auth(path: &str) -> bool {
    if path == "/health" || path.starts_with("/health/") {
        return false;
    }
    if path == "/ui" || path.starts_with("/ui/") {
        return false;
    }
    true
}

pub async fn hub_api_auth(
    State(gate): State<HubGate>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if gate.auth != HubAuth::ApiKey {
        return Ok(next.run(req).await);
    }
    let path = req.uri().path();
    if !path_requires_hub_auth(path) {
        return Ok(next.run(req).await);
    }
    match extract_bearer(req.headers()) {
        Some(token) if token_accepted(&token, &gate.api_keys) => Ok(next.run(req).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Env var holding the operator token required for daemon write routes under the
/// shared-store closed-write policy.
pub const ENV_ADMIN_TOKEN: &str = "KURULTAI_ADMIN_TOKEN";

/// POST routes that mutate durable state and must not be reachable unauthenticated.
const WRITE_ROUTES: &[&str] = &["/api/promote", "/api/touch"];

pub fn resolve_admin_token() -> Option<String> {
    std::env::var(ENV_ADMIN_TOKEN)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Fail-closed guard on daemon write routes.
///
/// `hub_api_auth` returns `next.run(req)` unconditionally whenever
/// `HubAuth != ApiKey`, and `HubAuth::None` is the default — so in the default solo
/// daemon configuration any local process could `POST /api/promote` and flip a
/// quarantined atom to trusted with no credential at all.
///
/// Under [`crate::write_policy::WriteMode::SharedClosed`] this guard requires a bearer
/// token matching `KURULTAI_ADMIN_TOKEN` on those routes, and returns 503 when no token
/// is configured (fail closed rather than fail open). Under `Solo` it is a no-op, so
/// the single-operator path is unchanged.
///
/// Caveat, stated plainly: on a shared box the admin token is readable from the
/// daemon's `/proc/<pid>/environ` by the same uid. This guard removes the
/// zero-credential hole and forces an explicit operator opt-in; it is not isolation.
pub async fn write_route_guard(req: Request, next: Next) -> Result<Response, StatusCode> {
    let decision = write_route_decision(
        req.method(),
        req.uri().path(),
        crate::write_policy::WriteMode::from_env(),
        resolve_admin_token().as_deref(),
        extract_bearer(req.headers()).as_deref(),
    );
    match decision {
        WriteRouteDecision::Allow => Ok(next.run(req).await),
        WriteRouteDecision::NoTokenConfigured => {
            tracing::warn!(
                path = %req.uri().path(),
                "write route refused: shared_write policy active but KURULTAI_ADMIN_TOKEN is unset"
            );
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
        WriteRouteDecision::Unauthorized => Err(StatusCode::UNAUTHORIZED),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteRouteDecision {
    Allow,
    /// Policy is active but no operator token exists — refuse rather than fail open.
    NoTokenConfigured,
    Unauthorized,
}

/// Pure policy decision for [`write_route_guard`] (env-free, directly testable).
pub fn write_route_decision(
    method: &axum::http::Method,
    path: &str,
    mode: crate::write_policy::WriteMode,
    admin_token: Option<&str>,
    bearer: Option<&str>,
) -> WriteRouteDecision {
    let is_write = method == axum::http::Method::POST && WRITE_ROUTES.contains(&path);
    if !is_write || mode != crate::write_policy::WriteMode::SharedClosed {
        return WriteRouteDecision::Allow;
    }
    let Some(expected) = admin_token.filter(|t| !t.is_empty()) else {
        return WriteRouteDecision::NoTokenConfigured;
    };
    match bearer {
        Some(token) if secrets_equal(token, expected) => WriteRouteDecision::Allow,
        _ => WriteRouteDecision::Unauthorized,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_auth_modes() {
        assert_eq!(parse_hub_auth(None), HubAuth::None);
        assert_eq!(parse_hub_auth(Some("api_key")), HubAuth::ApiKey);
        assert_eq!(parse_hub_auth(Some("none")), HubAuth::None);
    }

    #[test]
    fn token_matches_plaintext_or_sha256() {
        let plain = "secret-token";
        let hashed = sha256_hex(plain);
        assert!(token_accepted(plain, &[plain.to_string()]));
        assert!(token_accepted(plain, &[hashed]));
        assert!(!token_accepted("wrong", &[plain.to_string()]));
    }

    #[test]
    fn hub_auth_exempts_health_and_ui_only() {
        assert!(!path_requires_hub_auth("/health"));
        assert!(!path_requires_hub_auth("/health/ready"));
        assert!(!path_requires_hub_auth("/ui"));
        assert!(!path_requires_hub_auth("/ui/"));
        assert!(!path_requires_hub_auth("/ui/index.html"));
        assert!(!path_requires_hub_auth("/ui/assets/app.js"));

        assert!(path_requires_hub_auth("/api/status"));
        assert!(path_requires_hub_auth("/api/search"));
        assert!(path_requires_hub_auth("/api/ask"));
        assert!(path_requires_hub_auth("/search"));
        assert!(path_requires_hub_auth("/ask"));
        assert!(path_requires_hub_auth("/cite"));
        assert!(path_requires_hub_auth("/who_knows"));
    }
}
