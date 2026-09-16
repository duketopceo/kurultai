use crate::art::BannerMode;
use crate::config::file::FileConfig;
use crate::config::validate;
use crate::environment::Environment;
use crate::error::{KurultaiError, Result};
use crate::types::{Config, SourceConfig, SourceKind};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const DEFAULT_CONFIG_RELATIVE: &str = ".config/kurultai/config.toml";

/// Default config file path: `KURULTAI_CONFIG` or `~/.config/kurultai/config.toml`.
pub fn config_path() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("KURULTAI_CONFIG") {
        return Ok(PathBuf::from(path));
    }

    let home = dirs::home_dir()
        .ok_or_else(|| KurultaiError::config("could not resolve home directory"))?;
    Ok(home.join(DEFAULT_CONFIG_RELATIVE))
}

/// Load config from the default path.
pub fn load_config() -> Result<Config> {
    let path = config_path()?;
    load_config_from(&path)
}

/// Load config with optional path and `--env` / `KURULTAI_ENV` override.
pub fn load_config_with_env(path: Option<&Path>, env_override: Option<&str>) -> Result<Config> {
    let path = match path {
        Some(p) => p.to_path_buf(),
        None => config_path().map_err(|e| {
            KurultaiError::config(format!(
                "could not resolve config path: {e}. Set KURULTAI_CONFIG or run `kurultai init`"
            ))
        })?,
    };
    load_config_from_with_env(&path, env_override)
}

/// Load config from an explicit path.
pub fn load_config_from(path: &Path) -> Result<Config> {
    load_config_from_with_env(path, None)
}

fn load_config_from_with_env(path: &Path, env_override: Option<&str>) -> Result<Config> {
    if !path.exists() {
        tracing::warn!(path = %path.display(), "config file not found, using defaults");
        let env = Environment::resolve(env_override)?;
        let config = default_config(env)?;
        validate(&config)?;
        return Ok(config);
    }

    let raw = std::fs::read_to_string(path)?;
    let file: FileConfig = toml::from_str(&raw)?;
    let explicit_storage = file.storage.path.is_some();
    let env = Environment::resolve(env_override.or(file.environment.as_deref()))?;
    let config = file_to_runtime(file, env, explicit_storage)?;
    validate(&config)?;
    tracing::debug!(
        path = %path.display(),
        env = %config.environment,
        sources = config.sources.len(),
        "config loaded"
    );
    Ok(config)
}

fn default_config(env: Environment) -> Result<Config> {
    let home = dirs::home_dir()
        .ok_or_else(|| KurultaiError::config("could not resolve home directory"))?;
    Ok(Config {
        environment: env,
        sources: vec![],
        storage_path: home
            .join(env.storage_relative())
            .to_string_lossy()
            .into_owned(),
        embed_model: "openai/text-embedding-3-large".into(),
        embed_dim: 3072,
        embed_backend: None,
        reranker_model: None,
        poll_interval_secs: 300,
        nightly_full_sync_hour: None,
        inactivity_threshold_hours: None,
        mcp_http_secret: None,
        banner: BannerMode::Auto,
        tier_policy: crate::memory::TierPolicy::default(),
    })
}

fn file_to_runtime(file: FileConfig, env: Environment, explicit_storage: bool) -> Result<Config> {
    let home = dirs::home_dir()
        .ok_or_else(|| KurultaiError::config("could not resolve home directory"))?;

    let storage_path = if explicit_storage {
        file.storage.path.unwrap_or_else(|| {
            home.join(env.storage_relative())
                .to_string_lossy()
                .into_owned()
        })
    } else {
        home.join(env.storage_relative())
            .to_string_lossy()
            .into_owned()
    };

    let sources = file
        .sources
        .into_iter()
        .map(|(name, source)| {
            let mut extra = HashMap::new();
            for (key, value) in source.extra {
                extra.insert(key, value_to_string(&value));
            }

            Ok(SourceConfig {
                name,
                kind: parse_source_kind(&source.kind),
                enabled: source.enabled,
                poll_interval_secs: source.poll_interval_secs,
                extra,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(Config {
        environment: env,
        sources,
        storage_path,
        embed_model: file
            .embed
            .model
            .unwrap_or_else(|| "openai/text-embedding-3-large".into()),
        embed_dim: file.embed.dimension.unwrap_or(3072),
        embed_backend: match &file.embed.backend {
            None => None,
            Some(b) => {
                let trimmed = b.trim();
                if trimmed.is_empty() {
                    return Err(KurultaiError::config(
                        "embed.backend is empty; use \"local\" or omit the key",
                    ));
                }
                Some(trimmed.to_ascii_lowercase())
            }
        },
        reranker_model: file.runtime.reranker_model,
        poll_interval_secs: file.runtime.poll_interval_secs.unwrap_or(300),
        nightly_full_sync_hour: match file.runtime.nightly_full_sync_hour {
            None => None,
            Some(h) if h <= 23 => Some(h),
            Some(h) => {
                return Err(KurultaiError::config(format!(
                    "nightly_full_sync_hour out of range (0-23), got {h}"
                )));
            }
        },
        inactivity_threshold_hours: file.runtime.inactivity_threshold_hours,
        mcp_http_secret: file.runtime.mcp_http_secret,
        banner: file.cli.banner,
        tier_policy: tiers_to_policy(file.tiers)?,
    })
}

/// `[tiers]` → [`TierPolicy`]: threshold overrides plus validated sequester
/// rules. `cap` must be `warm` or `cold` (rules cap, never promote); a bad
/// `trust_lane`/`cap` fails the load with a clear error (#325).
fn tiers_to_policy(t: crate::config::file::FileTiersConfig) -> Result<crate::memory::TierPolicy> {
    let mut policy = crate::memory::TierPolicy {
        hot_access_days: t.hot_access_days.unwrap_or(7),
        hot_index_hours: t.hot_index_hours.unwrap_or(48),
        cold_days: t.cold_days.unwrap_or(180),
        rules: Vec::new(),
    };
    for (i, r) in t.rule.iter().enumerate() {
        let cap_raw = r.cap.as_deref().unwrap_or("").trim();
        let cap = crate::memory::MemoryTier::parse(cap_raw)
            .filter(|c| *c != crate::memory::MemoryTier::Hot)
            .ok_or_else(|| {
                KurultaiError::config(format!(
                    "tiers.rule[{i}].cap must be \"warm\" or \"cold\", got {cap_raw:?}"
                ))
            })?;
        // TrustLane::parse fails closed to Quarantine — too loose for config;
        // validate strictly so a typo can't silently quarantine-cap everything.
        let trust_lane = match r.trust_lane.as_deref().map(str::trim) {
            None => None,
            Some("trusted") => Some(crate::types::TrustLane::Trusted),
            Some("quarantine") => Some(crate::types::TrustLane::Quarantine),
            Some(raw) => {
                return Err(KurultaiError::config(format!(
                    "tiers.rule[{i}].trust_lane must be \"trusted\" or \"quarantine\", got {raw:?}"
                )))
            }
        };
        policy.rules.push(crate::memory::TierRule {
            source: r.source.clone(),
            tag: r.tag.clone(),
            trust_lane,
            cap,
        });
    }
    Ok(policy)
}

fn value_to_string(value: &toml::Value) -> String {
    match value {
        toml::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn parse_source_kind(kind: &str) -> SourceKind {
    match kind.to_ascii_lowercase().as_str() {
        "appflowy" => SourceKind::AppFlowy,
        "markdown" | "filesystem" | "fs" => SourceKind::Markdown,
        "pond" => SourceKind::Pond,
        "dayflow" => SourceKind::Dayflow,
        "tech_tracker" | "techtracker" => SourceKind::TechTracker,
        "github" => SourceKind::GitHub,
        "json" => SourceKind::Json,
        "inbox" => SourceKind::Inbox,
        other => SourceKind::Custom(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::default_config_toml;
    use std::io::Write;

    #[test]
    fn loads_default_config_toml_shape() {
        let dir = tempfile_dir("cfg-valid");
        let path = dir.join("config.toml");
        std::fs::write(&path, default_config_toml()).unwrap();
        let cfg = load_config_from(&path).unwrap();
        assert_eq!(cfg.embed_dim, 3072);
        assert!(cfg.sources.is_empty());
        assert!(cfg.storage_path.contains("kurultai"));
    }

    #[test]
    fn loads_markdown_source_map() {
        let dir = tempfile_dir("cfg-src");
        let path = dir.join("config.toml");
        let toml = r#"
environment = "dev"
[storage]
path = "/tmp/kurultai-loader-test.db"
[embed]
model = "openai/text-embedding-3-large"
dimension = 4
[sources.notes]
kind = "markdown"
enabled = true
root_path = "/tmp/notes"
"#;
        std::fs::write(&path, toml).unwrap();
        let cfg = load_config_from(&path).unwrap();
        assert_eq!(cfg.sources.len(), 1);
        assert_eq!(cfg.sources[0].name, "notes");
        assert_eq!(cfg.sources[0].kind, SourceKind::Markdown);
        assert_eq!(
            cfg.sources[0].extra.get("root_path").map(String::as_str),
            Some("/tmp/notes")
        );
    }

    #[test]
    fn loads_json_source_kind() {
        let dir = tempfile_dir("cfg-json");
        let path = dir.join("config.toml");
        let toml = r#"
environment = "dev"
[storage]
path = "/tmp/kurultai-loader-json.db"
[embed]
model = "openai/text-embedding-3-large"
dimension = 4
[sources.data]
kind = "json"
enabled = true
root_path = "/tmp/data"
"#;
        std::fs::write(&path, toml).unwrap();
        let cfg = load_config_from(&path).unwrap();
        assert_eq!(cfg.sources.len(), 1);
        assert_eq!(cfg.sources[0].kind, SourceKind::Json);
    }

    #[test]
    fn rejects_invalid_toml() {
        let dir = tempfile_dir("cfg-bad");
        let path = dir.join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "[[[not valid").unwrap();
        assert!(load_config_from(&path).is_err());
    }

    #[test]
    fn rejects_nightly_hour_above_23() {
        let dir = tempfile_dir("cfg-hour");
        let path = dir.join("config.toml");
        let toml = r#"
environment = "dev"
[storage]
path = "/tmp/kurultai-hour-test.db"
[embed]
model = "openai/text-embedding-3-large"
dimension = 4
[runtime]
nightly_full_sync_hour = 24
"#;
        std::fs::write(&path, toml).unwrap();
        let err = load_config_from(&path).unwrap_err();
        assert!(
            err.to_string().contains("nightly_full_sync_hour"),
            "got {err}"
        );
    }

    #[test]
    fn accepts_valid_nightly_hour() {
        let dir = tempfile_dir("cfg-hour-ok");
        let path = dir.join("config.toml");
        let toml = r#"
environment = "dev"
[storage]
path = "/tmp/kurultai-hour-ok.db"
[embed]
model = "openai/text-embedding-3-large"
dimension = 4
[runtime]
nightly_full_sync_hour = 3
"#;
        std::fs::write(&path, toml).unwrap();
        let cfg = load_config_from(&path).unwrap();
        assert_eq!(cfg.nightly_full_sync_hour, Some(3));
    }

    #[test]
    fn loads_cli_banner_settings() {
        let dir = tempfile_dir("cfg-banner");
        let path = dir.join("config.toml");
        for (value, expected) in [
            ("true", BannerMode::Always),
            ("false", BannerMode::Never),
            ("\"auto\"", BannerMode::Auto),
        ] {
            let toml = format!(
                r#"
environment = "dev"
[storage]
path = "/tmp/kurultai-banner.db"
[embed]
model = "openai/text-embedding-3-large"
dimension = 4
[cli]
banner = {value}
"#
            );
            std::fs::write(&path, toml).unwrap();
            let cfg = load_config_from(&path).unwrap();
            assert_eq!(cfg.banner, expected, "banner = {value}");
        }
    }

    #[test]
    fn rejects_invalid_cli_banner() {
        let dir = tempfile_dir("cfg-banner-bad");
        let path = dir.join("config.toml");
        let toml = r#"
environment = "dev"
[storage]
path = "/tmp/kurultai-banner-bad.db"
[embed]
model = "openai/text-embedding-3-large"
dimension = 4
[cli]
banner = "sometimes"
"#;
        std::fs::write(&path, toml).unwrap();
        let err = load_config_from(&path).unwrap_err();
        assert!(
            err.to_string().contains("banner") || err.to_string().contains("cli"),
            "got {err}"
        );
    }

    #[test]
    fn default_config_toml_documents_cli_banner() {
        assert!(default_config_toml().contains("[cli]"));
        assert!(default_config_toml().contains("banner"));
    }

    #[test]
    fn loads_local_embed_backend() {
        let dir = tempfile_dir("cfg-local-embed");
        let path = dir.join("config.toml");
        let toml = r#"
environment = "dev"
[storage]
path = "/tmp/kurultai-local-embed.db"
[embed]
backend = "local"
model = "AllMiniLML6V2"
dimension = 384
"#;
        std::fs::write(&path, toml).unwrap();
        let cfg = load_config_from(&path).unwrap();
        assert_eq!(cfg.embed_backend.as_deref(), Some("local"));
        assert_eq!(cfg.embed_dim, 384);
        assert_eq!(cfg.embed_model, "AllMiniLML6V2");
    }

    #[test]
    fn rejects_empty_embed_backend() {
        let dir = tempfile_dir("cfg-empty-backend");
        let path = dir.join("config.toml");
        let toml = r#"
environment = "dev"
[storage]
path = "/tmp/kurultai-empty-backend.db"
[embed]
backend = ""
model = "openai/text-embedding-3-large"
dimension = 4
"#;
        std::fs::write(&path, toml).unwrap();
        let err = load_config_from(&path).unwrap_err().to_string();
        assert!(
            err.contains("embed.backend is empty"),
            "unexpected error: {err}"
        );
    }

    fn tempfile_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "kurultai-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_cfg(tag: &str, toml: &str) -> PathBuf {
        let dir = tempfile_dir(tag);
        let path = dir.join("config.toml");
        std::fs::write(&path, toml).unwrap();
        path
    }

    #[test]
    fn tiers_section_defaults_when_absent() {
        let path = write_cfg(
            "cfg-tiers-none",
            r#"environment = "dev"
[storage]
path = "/tmp/kurultai-tiers-none.db"
"#,
        );
        let cfg = load_config_from(&path).unwrap();
        assert_eq!(cfg.tier_policy.hot_access_days, 7);
        assert!(cfg.tier_policy.rules.is_empty());
    }

    #[test]
    fn tiers_rules_parse_and_override() {
        let path = write_cfg(
            "cfg-tiers-rules",
            r#"environment = "dev"
[storage]
path = "/tmp/kurultai-tiers-rules.db"
[tiers]
hot_access_days = 3
[[tiers.rule]]
source = "pond"
cap = "cold"
[[tiers.rule]]
tag = "session-transcript"
trust_lane = "quarantine"
cap = "warm"
"#,
        );
        let cfg = load_config_from(&path).unwrap();
        assert_eq!(cfg.tier_policy.hot_access_days, 3);
        assert_eq!(cfg.tier_policy.rules.len(), 2);
        let r0 = &cfg.tier_policy.rules[0];
        assert_eq!(r0.source.as_deref(), Some("pond"));
        assert_eq!(r0.cap, crate::memory::MemoryTier::Cold);
        let r1 = &cfg.tier_policy.rules[1];
        assert_eq!(r1.tag.as_deref(), Some("session-transcript"));
        assert_eq!(r1.trust_lane, Some(crate::types::TrustLane::Quarantine));
        assert_eq!(r1.cap, crate::memory::MemoryTier::Warm);
    }

    #[test]
    fn tiers_rule_rejects_hot_cap() {
        let path = write_cfg(
            "cfg-tiers-hotcap",
            r#"environment = "dev"
[storage]
path = "/tmp/kurultai-tiers-hot.db"
[[tiers.rule]]
source = "pond"
cap = "hot"
"#,
        );
        assert!(load_config_from(&path).is_err());
    }

    #[test]
    fn tiers_rule_rejects_bad_trust_lane() {
        let path = write_cfg(
            "cfg-tiers-badlane",
            r#"environment = "dev"
[storage]
path = "/tmp/kurultai-tiers-lane.db"
[[tiers.rule]]
trust_lane = "trusted-typo"
cap = "cold"
"#,
        );
        assert!(load_config_from(&path).is_err());
    }
}
