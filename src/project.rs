//! Project namespacing for agent memory (#184).
//!
//! Nine Claude Code sessions can share one Kurultai store on one box as one unix
//! user. A `project_id` keeps one session's ingest out of another session's
//! recall. That is **noise control, not security** — every session runs as the
//! same user and can read the same files, `/proc` and the same SQLite database,
//! so any local process can pass any project string. Do not treat this as
//! isolation between agents.
//!
//! Stored in `KnowledgeAtom::metadata["project_id"]`; see
//! [`crate::types::KnowledgeAtom::project_id`]. A first-class column plus index
//! is the medium-term plan (issue #184 §1) and is deliberately deferred.

/// Namespace used when nothing sets `project_id`.
pub const DEFAULT_PROJECT: &str = "default";

/// Metadata key carrying the namespace.
pub const PROJECT_METADATA_KEY: &str = "project_id";

/// Env var read when a caller supplies no explicit project. Set this per session
/// (via the MCP client config `env` block) so each session tags its own writes.
pub const PROJECT_ENV: &str = "KURULTAI_PROJECT";

/// Longest accepted namespace — keeps metadata rows small and log lines readable.
const MAX_PROJECT_LEN: usize = 64;

/// Canonical form of a project namespace.
///
/// Trims, lowercases and truncates. Empty input becomes [`DEFAULT_PROJECT`].
/// Applied identically on write and on read so `Crew-YAM` and `crew-yam` cannot
/// silently become two namespaces.
pub fn normalize_project(project: &str) -> String {
    let trimmed = project.trim();
    if trimmed.is_empty() {
        return DEFAULT_PROJECT.to_string();
    }
    trimmed
        .chars()
        .take(MAX_PROJECT_LEN)
        .flat_map(char::to_lowercase)
        .collect()
}

/// Resolve the project for a call: explicit argument, else `KURULTAI_PROJECT`,
/// else [`DEFAULT_PROJECT`].
pub fn resolve_project(explicit: Option<&str>) -> String {
    let from_arg = explicit.map(str::trim).filter(|p| !p.is_empty());
    match from_arg {
        Some(p) => normalize_project(p),
        None => match std::env::var(PROJECT_ENV) {
            Ok(v) if !v.trim().is_empty() => normalize_project(&v),
            _ => DEFAULT_PROJECT.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_trims_lowercases_and_defaults() {
        assert_eq!(normalize_project("  Crew-YAM "), "crew-yam");
        assert_eq!(normalize_project(""), DEFAULT_PROJECT);
        assert_eq!(normalize_project("   "), DEFAULT_PROJECT);
        assert_eq!(normalize_project(&"x".repeat(200)).len(), MAX_PROJECT_LEN);
    }

    #[test]
    fn explicit_argument_wins_over_env() {
        // No env dependency: an explicit value is used verbatim (normalized).
        assert_eq!(resolve_project(Some("crew-itdash")), "crew-itdash");
        assert_eq!(resolve_project(Some("  ")), resolve_project(None));
    }
}
