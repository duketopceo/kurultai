//! Versioned product feature flags.
//!
//! Defaults are the shipped-on version. Override with `KURULTAI_FEATURE_<ID>=0|1`
//! (`true`/`false`/`on`/`off` also work). Unknown ids are ignored (stay default).
//! These flags document the v# roadmap; they do not turn off kernel FTS/search.

/// One named flag and the release that introduced it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeatureSpec {
    pub id: &'static str,
    /// Semver without a leading `v` (e.g. `0.4.0`).
    pub since: &'static str,
    pub default_on: bool,
    pub summary: &'static str,
}

/// Catalog for `status` and next work orders. Keep ids stable.
pub const ALL: &[FeatureSpec] = &[
    FeatureSpec {
        id: "fts",
        since: "0.3.0",
        default_on: true,
        summary: "FTS5 search / who-knows / extractive ask (no API key)",
    },
    FeatureSpec {
        id: "brain_ui",
        since: "0.4.0",
        default_on: true,
        summary: "Embedded Brain UI at GET /ui/",
    },
    FeatureSpec {
        id: "mcp_http",
        since: "0.4.0",
        default_on: true,
        summary: "Daemon MCP HTTP/SSE (needs KURULTAI_MCP_HTTP_SECRET)",
    },
    FeatureSpec {
        id: "local_embed",
        since: "0.3.0",
        default_on: cfg!(feature = "local-embed"),
        summary: "On-device ONNX embeddings (cargo --features local-embed)",
    },
    FeatureSpec {
        id: "shared_write",
        since: "0.5.1",
        default_on: false,
        summary: "Shared-store write containment (agent writes quarantined + namespaced)",
    },
    FeatureSpec {
        id: "hub",
        since: "0.5.0",
        default_on: false,
        summary: "Shared team/company hub (Wave G — not shipped)",
    },
];

/// Whether `id` is on after env overlay.
pub fn enabled(id: &str) -> bool {
    let spec = ALL.iter().find(|f| f.id.eq_ignore_ascii_case(id));
    let default = spec.map(|s| s.default_on).unwrap_or(false);
    let key = format!("KURULTAI_FEATURE_{}", id.to_ascii_uppercase());
    match std::env::var(&key) {
        Ok(raw) => parse_bool(&raw).unwrap_or(default),
        Err(_) => default,
    }
}

fn parse_bool(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "on" | "yes" => Some(true),
        "0" | "false" | "off" | "no" => Some(false),
        _ => None,
    }
}

/// Lines for `kurultai status` (no trailing newline on last line).
pub fn status_lines() -> Vec<String> {
    ALL.iter()
        .map(|f| {
            let on = if enabled(f.id) { "on " } else { "off" };
            format!(
                "    {id:<12} {on}  v{since}  {summary}",
                id = f.id,
                since = f.since,
                summary = f.summary
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_ids_are_unique_and_stable() {
        let mut ids: Vec<_> = ALL.iter().map(|f| f.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), ALL.len());
        assert!(ALL.iter().any(|f| f.id == "fts" && f.default_on));
        assert!(ALL
            .iter()
            .any(|f| f.id == "hub" && !f.default_on && f.since == "0.5.0"));
        assert!(ALL.iter().any(|f| f.id == "brain_ui" && f.since == "0.4.0"));
    }

    #[test]
    fn parse_bool_accepts_common_tokens() {
        assert_eq!(parse_bool("1"), Some(true));
        assert_eq!(parse_bool("off"), Some(false));
        assert_eq!(parse_bool("maybe"), None);
    }

    #[test]
    fn env_override_flips_hub() {
        let key = "KURULTAI_FEATURE_HUB";
        let prev = std::env::var(key).ok();
        std::env::set_var(key, "1");
        assert!(enabled("hub"));
        std::env::set_var(key, "0");
        assert!(!enabled("hub"));
        match prev {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
    }
}
