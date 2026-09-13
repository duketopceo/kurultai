//! GlitchTip / Sentry-compatible error tracking for the Kurultai daemon and CLI.
//!
//! Disabled when no DSN is configured. Never sends default PII; sensitive headers
//! are redacted in [`scrub_event`] before events leave the process.

use crate::environment::Environment;
use sentry::integrations::panic::PanicIntegration;
use sentry::{ClientInitGuard, Level};
use std::sync::OnceLock;

pub const DSN_ENV: &str = "KURULTAI_GLITCHTIP_DSN";
pub const DSN_ALIAS_ENV: &str = "SENTRY_DSN";
pub const TEST_ENV: &str = "KURULTAI_GLITCHTIP_TEST";
pub const VERSION_ENV: &str = "KURULTAI_VERSION";
pub const TRACES_ENV: &str = "KURULTAI_GLITCHTIP_TRACES";

static ACTIVE: OnceLock<bool> = OnceLock::new();

const SENSITIVE_HEADERS: &[&str] = &["authorization", "cookie", "x-api-key"];

/// Resolve DSN from `KURULTAI_GLITCHTIP_DSN`, then `SENTRY_DSN`.
pub fn resolve_dsn() -> Option<String> {
    std::env::var(DSN_ENV)
        .ok()
        .or_else(|| std::env::var(DSN_ALIAS_ENV).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// True after a successful [`init`].
pub fn is_enabled() -> bool {
    *ACTIVE.get().unwrap_or(&false)
}

/// Initialize GlitchTip when a DSN is set. Returns the client guard (keep alive).
pub fn init(env: Environment) -> Option<ClientInitGuard> {
    let dsn = resolve_dsn()?;
    let traces = std::env::var(TRACES_ENV)
        .ok()
        .and_then(|s| s.parse::<f32>().ok())
        .map(|n| n.clamp(0.0, 1.0))
        .unwrap_or(0.0);

    let release = std::env::var(VERSION_ENV)
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| format!("kurultai@{}", env!("CARGO_PKG_VERSION")));

    let opts = sentry::ClientOptions::new()
        .dsn(dsn.as_str())
        .environment(env.as_str())
        .release(release)
        .send_default_pii(false)
        .traces_sample_rate(traces)
        .traces_sampler(move |ctx| {
            let name = ctx.name().to_lowercase();
            if name.contains("/health") || name.contains("/metrics") {
                return 0.0;
            }
            traces
        })
        .before_send(scrub_event)
        .add_integration(PanicIntegration::new());

    let guard = sentry::init(opts);
    sentry::configure_scope(|scope| {
        scope.set_tag("service", "kurultai");
    });

    if env_truthy(TEST_ENV) {
        sentry::capture_message("GlitchTip test event", Level::Info);
    }

    let _ = ACTIVE.set(true);
    tracing::info!(environment = env.as_str(), "GlitchTip initialized");
    Some(guard)
}

/// Redact sensitive request headers and drop obvious secret substrings from messages.
pub fn scrub_event(
    mut event: sentry::protocol::Event<'static>,
) -> Option<sentry::protocol::Event<'static>> {
    if let Some(ref mut req) = event.request {
        for (key, value) in req.headers.iter_mut() {
            if SENSITIVE_HEADERS.contains(&key.to_lowercase().as_str()) {
                *value = "[redacted]".into();
            }
        }
    }

    if let Some(ref mut msg) = event.message {
        *msg = redact_inline_secrets(msg);
    }

    for ex in &mut event.exception {
        if let Some(ref mut val) = ex.value {
            *val = redact_inline_secrets(val);
        }
    }

    Some(event)
}

fn redact_inline_secrets(input: &str) -> String {
    let lower = input.to_lowercase();
    if lower.contains("bearer ")
        || lower.contains("api_key=")
        || lower.contains("token=")
        || lower.contains("secret=")
    {
        "[redacted]".to_string()
    } else {
        input.to_string()
    }
}

fn env_truthy(name: &str) -> bool {
    std::env::var(name)
        .ok()
        .map(|v| {
            matches!(
                v.trim().to_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_is_noop_without_dsn() {
        let _lock = test_env_lock();
        std::env::remove_var(DSN_ENV);
        std::env::remove_var(DSN_ALIAS_ENV);
        assert!(resolve_dsn().is_none());
        // Cannot call init() twice with global sentry client; resolve_dsn covers the gate.
    }

    #[test]
    fn scrub_redacts_auth_headers() {
        let mut headers = std::collections::BTreeMap::new();
        headers.insert("Authorization".into(), "Bearer secret".into());
        headers.insert("X-Other".into(), "ok".into());
        let event = sentry::protocol::Event {
            request: Some(sentry::protocol::Request {
                headers,
                ..Default::default()
            }),
            ..Default::default()
        };
        let out = scrub_event(event).expect("event");
        let req = out.request.expect("request");
        assert_eq!(
            req.headers.get("Authorization").map(String::as_str),
            Some("[redacted]")
        );
        assert_eq!(req.headers.get("X-Other").map(String::as_str), Some("ok"));
    }

    #[test]
    fn resolve_dsn_prefers_primary_env() {
        let _lock = test_env_lock();
        std::env::set_var(DSN_ENV, "https://key@errors.example/1");
        std::env::set_var(DSN_ALIAS_ENV, "https://alias@errors.example/2");
        assert_eq!(
            resolve_dsn().as_deref(),
            Some("https://key@errors.example/1")
        );
        std::env::remove_var(DSN_ENV);
        std::env::remove_var(DSN_ALIAS_ENV);
    }

    fn test_env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }
}
