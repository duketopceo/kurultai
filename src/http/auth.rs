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

pub async fn hub_api_auth(
    State(gate): State<HubGate>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if gate.auth != HubAuth::ApiKey {
        return Ok(next.run(req).await);
    }
    let path = req.uri().path();
    if !path.starts_with("/api/") {
        return Ok(next.run(req).await);
    }
    match extract_bearer(req.headers()) {
        Some(token) if token_accepted(&token, &gate.api_keys) => Ok(next.run(req).await),
        _ => Err(StatusCode::UNAUTHORIZED),
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
}
