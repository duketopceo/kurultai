//! Optional trust of Cloudflare Access JWTs for human browser auth.
//!
//! When a hosted instance sits behind Cloudflare Access (e.g.
//! `knowledge.shippedit.dev`), the edge injects a signed JWT on every
//! authenticated request — the `CF_Authorization` cookie and the
//! `Cf-Access-Jwt-Assertion` header. Verifying that JWT lets humans sign in
//! with the Access IdP (Google/GitHub/OTP) instead of pasting a hub API key
//! into the Brain UI gate.
//!
//! Signature, issuer, audience, and expiry are all verified against the team
//! domain's public certs (`<team>/cdn-cgi/access/certs`), so a forged header
//! sent through a non-Access hostname (e.g. `api-*` agent endpoints that bypass
//! Access) cannot satisfy this path. Agent bearer auth is unchanged.

use axum::body::Body;
use axum::http::{header, HeaderMap, Request};
use jsonwebtoken::{decode, jwk::Jwk, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

pub const ENV_CF_ACCESS_TEAM: &str = "KURULTAI_CF_ACCESS_TEAM";
pub const ENV_CF_ACCESS_AUDS: &str = "KURULTAI_CF_ACCESS_AUDS";

const JWKS_TTL: Duration = Duration::from_secs(300);

/// Identity extracted from a verified Access JWT, attached to request extensions.
#[derive(Debug, Clone)]
pub struct CfAccessIdentity {
    pub sub: String,
    pub email: Option<String>,
}

pub struct CfAccess {
    /// Issuer, e.g. `https://team.cloudflareaccess.com` (no trailing slash).
    team_url: String,
    /// Allowed Application AUD tags (the Access app's `aud` claim).
    auds: Vec<String>,
    client: reqwest::Client,
    jwks: RwLock<Option<CachedJwks>>,
}

struct CachedJwks {
    keys: Vec<(Option<String>, DecodingKey)>,
    fetched_at: Instant,
}

#[derive(Debug, Deserialize)]
struct AccessClaims {
    sub: String,
    #[serde(default)]
    email: Option<String>,
}

/// Resolve config from env. Both vars are required to enable; either alone
/// logs a warning and disables the path (fail closed).
pub fn resolve_from_env() -> Option<Arc<CfAccess>> {
    let team = std::env::var(ENV_CF_ACCESS_TEAM)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let auds: Vec<String> = std::env::var(ENV_CF_ACCESS_AUDS)
        .ok()
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    match (team, auds.is_empty()) {
        (Some(team), false) => Some(Arc::new(CfAccess {
            team_url: normalize_team_url(&team),
            auds,
            client: reqwest::Client::new(),
            jwks: RwLock::new(None),
        })),
        (None, false) => {
            tracing::warn!(
                "{ENV_CF_ACCESS_AUDS} set without {ENV_CF_ACCESS_TEAM}; CF Access auth disabled"
            );
            None
        }
        (Some(_), true) => {
            tracing::warn!(
                "{ENV_CF_ACCESS_TEAM} set without {ENV_CF_ACCESS_AUDS}; CF Access auth disabled"
            );
            None
        }
        (None, true) => None,
    }
}

impl std::fmt::Debug for CfAccess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CfAccess")
            .field("team_url", &self.team_url)
            .field("auds_len", &self.auds.len())
            .finish()
    }
}

fn normalize_team_url(raw: &str) -> String {
    let trimmed = raw.trim().trim_end_matches('/');
    let with_scheme = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    };
    // Bare team name → <name>.cloudflareaccess.com
    let host = with_scheme
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    if !host.contains('.') {
        format!("https://{host}.cloudflareaccess.com")
    } else {
        with_scheme
    }
}

fn extract_assertion(headers: &HeaderMap) -> Option<String> {
    if let Some(v) = headers.get("cf-access-jwt-assertion") {
        if let Ok(s) = v.to_str() {
            if !s.trim().is_empty() {
                return Some(s.trim().to_string());
            }
        }
    }
    let cookie = headers.get(header::COOKIE)?.to_str().ok()?;
    for pair in cookie.split(';') {
        let pair = pair.trim();
        if let Some(value) = pair.strip_prefix("CF_Authorization=") {
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

impl CfAccess {
    /// Verify any Access assertion on the request. On success attaches a
    /// [`CfAccessIdentity`] extension and returns true; otherwise false.
    pub async fn authorize(&self, req: &mut Request<Body>) -> bool {
        let Some(token) = extract_assertion(req.headers()) else {
            return false;
        };
        match self.verify(&token).await {
            Ok(identity) => {
                req.extensions_mut().insert(identity);
                true
            }
            Err(e) => {
                tracing::warn!(error = %e, "CF Access JWT rejected");
                false
            }
        }
    }

    async fn verify(&self, token: &str) -> Result<CfAccessIdentity, String> {
        let header = jsonwebtoken::decode_header(token).map_err(|e| format!("header: {e}"))?;
        if header.alg != Algorithm::RS256 {
            return Err(format!("unexpected alg {:?}", header.alg));
        }
        let key = self.decoding_key(header.kid.as_deref()).await?;
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(std::slice::from_ref(&self.team_url));
        validation.set_audience(&self.auds);
        let data =
            decode::<AccessClaims>(token, &key, &validation).map_err(|e| format!("verify: {e}"))?;
        Ok(CfAccessIdentity {
            sub: data.claims.sub,
            email: data.claims.email,
        })
    }

    async fn decoding_key(&self, kid: Option<&str>) -> Result<DecodingKey, String> {
        {
            let cache = self.jwks.read().await;
            if let Some(c) = cache.as_ref() {
                if let Some(k) = pick_key(c, kid) {
                    return Ok(k);
                }
            }
        }
        // Miss (empty cache, unknown kid, or stale): refetch the team certs once.
        let fetched = self.fetch_jwks().await?;
        let mut cache = self.jwks.write().await;
        *cache = Some(fetched);
        pick_key(cache.as_ref().unwrap(), kid)
            .ok_or_else(|| format!("no cert for kid {}", kid.unwrap_or("<none>")))
    }

    async fn fetch_jwks(&self) -> Result<CachedJwks, String> {
        let url = format!("{}/cdn-cgi/access/certs", self.team_url);
        let body: serde_json::Value = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("certs fetch: {e}"))?
            .error_for_status()
            .map_err(|e| format!("certs status: {e}"))?
            .json()
            .await
            .map_err(|e| format!("certs parse: {e}"))?;
        let keys = body
            .get("keys")
            .and_then(|k| k.as_array())
            .ok_or_else(|| "certs response missing keys".to_string())?;
        let mut out = Vec::new();
        for raw in keys {
            let jwk: Jwk =
                serde_json::from_value(raw.clone()).map_err(|e| format!("jwk parse: {e}"))?;
            let kid = jwk.common.key_id.clone();
            let key = DecodingKey::from_jwk(&jwk).map_err(|e| format!("jwk key: {e}"))?;
            out.push((kid, key));
        }
        if out.is_empty() {
            return Err("certs response had no usable keys".to_string());
        }
        Ok(CachedJwks {
            keys: out,
            fetched_at: Instant::now(),
        })
    }
}

fn pick_key(cache: &CachedJwks, kid: Option<&str>) -> Option<DecodingKey> {
    if cache.fetched_at.elapsed() > JWKS_TTL {
        return None;
    }
    cache
        .keys
        .iter()
        .find(|(k, _)| k.as_deref() == kid)
        .map(|(_, k)| k.clone())
}

#[cfg(test)]
impl CfAccess {
    fn seeded_for_test(
        team_url: &str,
        auds: Vec<String>,
        keys: Vec<(Option<String>, DecodingKey)>,
    ) -> Self {
        Self {
            team_url: team_url.to_string(),
            auds,
            client: reqwest::Client::new(),
            jwks: RwLock::new(Some(CachedJwks {
                keys,
                fetched_at: Instant::now(),
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header as JwtHeader};

    const TEAM: &str = "https://duketopceo.cloudflareaccess.com";
    const AUD: &str = "681b5166-test-aud";

    struct TestGate {
        cf: CfAccess,
        enc: EncodingKey,
    }

    /// Generate an ephemeral RSA pair via the openssl CLI — no key material in
    /// git and no RSA keygen dependency (cargo-audit flags `rsa`). Both OpenSSL
    /// and LibreSSL provide `genrsa`/`rsa`.
    fn test_gate() -> TestGate {
        let dir = std::env::temp_dir().join(format!(
            "kurultai-cf-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let priv_path = dir.join("k.pem");
        let pub_path = dir.join("k.pub.pem");
        let run = |args: &[&str]| {
            let out = std::process::Command::new("openssl")
                .args(args)
                .output()
                .expect("openssl not found");
            assert!(out.status.success(), "openssl {:?} failed", args);
        };
        run(&["genrsa", "-out", priv_path.to_str().unwrap(), "2048"]);
        run(&[
            "rsa",
            "-in",
            priv_path.to_str().unwrap(),
            "-pubout",
            "-out",
            pub_path.to_str().unwrap(),
        ]);
        let priv_pem = std::fs::read(&priv_path).unwrap();
        let pub_pem = std::fs::read(&pub_path).unwrap();
        std::fs::remove_dir_all(&dir).ok();
        TestGate {
            cf: CfAccess::seeded_for_test(
                TEAM,
                vec![AUD.to_string()],
                vec![(
                    Some("test-kid".to_string()),
                    DecodingKey::from_rsa_pem(&pub_pem).unwrap(),
                )],
            ),
            enc: EncodingKey::from_rsa_pem(&priv_pem).unwrap(),
        }
    }

    fn sign(enc: &EncodingKey, aud: &str, iss: &str) -> String {
        #[derive(serde::Serialize)]
        struct Claims<'a> {
            sub: &'a str,
            email: &'a str,
            aud: &'a str,
            iss: &'a str,
            exp: usize,
        }
        let mut header = JwtHeader::new(Algorithm::RS256);
        header.kid = Some("test-kid".to_string());
        encode(
            &header,
            &Claims {
                sub: "u1",
                email: "duketopceo@gmail.com",
                aud,
                iss,
                exp: (chrono::Utc::now().timestamp() + 3600) as usize,
            },
            enc,
        )
        .unwrap()
    }

    fn request_with_cookie(token: &str) -> Request<Body> {
        Request::builder()
            .uri("/api/status")
            .header(header::COOKIE, format!("CF_Authorization={token}"))
            .body(Body::empty())
            .unwrap()
    }

    #[tokio::test]
    async fn valid_access_jwt_authorizes() {
        let gate = test_gate();
        let mut req = request_with_cookie(&sign(&gate.enc, AUD, TEAM));
        assert!(gate.cf.authorize(&mut req).await);
        let id = req.extensions().get::<CfAccessIdentity>().unwrap();
        assert_eq!(id.email.as_deref(), Some("duketopceo@gmail.com"));
    }

    #[tokio::test]
    async fn wrong_aud_or_iss_rejected() {
        let gate = test_gate();
        let mut req = request_with_cookie(&sign(&gate.enc, "other-aud", TEAM));
        assert!(!gate.cf.authorize(&mut req).await);
        let mut req = request_with_cookie(&sign(&gate.enc, AUD, "https://evil.example.com"));
        assert!(!gate.cf.authorize(&mut req).await);
    }

    #[tokio::test]
    async fn no_assertion_rejected() {
        let gate = test_gate();
        let mut req = Request::builder()
            .uri("/api/status")
            .body(Body::empty())
            .unwrap();
        assert!(!gate.cf.authorize(&mut req).await);
    }

    #[test]
    fn team_url_normalization() {
        assert_eq!(
            normalize_team_url("duketopceo"),
            "https://duketopceo.cloudflareaccess.com"
        );
        assert_eq!(
            normalize_team_url("duketopceo.cloudflareaccess.com/"),
            "https://duketopceo.cloudflareaccess.com"
        );
        assert_eq!(
            normalize_team_url("https://duketopceo.cloudflareaccess.com"),
            "https://duketopceo.cloudflareaccess.com"
        );
    }

    #[test]
    fn assertion_prefers_header_then_cookie() {
        let mut h = HeaderMap::new();
        h.insert("cf-access-jwt-assertion", "hdr.jwt".parse().unwrap());
        h.insert(
            header::COOKIE,
            "a=1; CF_Authorization=ck.jwt; b=2".parse().unwrap(),
        );
        assert_eq!(extract_assertion(&h).as_deref(), Some("hdr.jwt"));

        let mut h = HeaderMap::new();
        h.insert(
            header::COOKIE,
            "a=1; CF_Authorization=ck.jwt; b=2".parse().unwrap(),
        );
        assert_eq!(extract_assertion(&h).as_deref(), Some("ck.jwt"));

        assert_eq!(extract_assertion(&HeaderMap::new()), None);
    }
}
