//! Write provenance and containment policy for shared-store, multi-session deployments.
//!
//! # Threat model (same box, same uid)
//!
//! The target deployment is N Claude Code sessions on one machine, running as one
//! unix user, against one SQLite file. Under that constraint:
//!
//! - `agent_id` is **self-asserted**. Any session can pass any value, read another
//!   session's `/proc/<pid>/environ`, or open the DB file directly. An ACL keyed on
//!   `agent_id` would be theatre and this module does not pretend otherwise.
//! - Env-var secrets (`KURULTAI_INGEST_SECRET`, `KURULTAI_MCP_HTTP_SECRET`) are not
//!   access controls here — every session can read them.
//!
//! What *is* enforceable, and what this module provides:
//!
//! 1. **Provenance** — every agent-reachable write is stamped with the agent id and
//!    namespace it claimed, so a poisoned session's atoms can be attributed and
//!    bulk-revoked after the fact.
//! 2. **Containment** — under [`WriteMode::SharedClosed`], no agent-reachable write
//!    path can land in the globally-searchable trusted lane. Agent writes are forced
//!    to quarantine regardless of the quality gate outcome, and the
//!    quarantine -> trusted transition is restricted to the operator-run CLI.
//! 3. **Namespacing** — writes carry a `project_id` so sessions stop polluting each
//!    other's recall.
//!
//! Containment is the only real control. Provenance and namespacing are hygiene.

use std::collections::HashMap;

/// Feature flag id gating the closed-write policy. Default off: the solo path is unchanged.
pub const FEATURE_SHARED_WRITE: &str = "shared_write";

/// Env var for the self-asserted agent identity (CLI `--agent-id` takes precedence).
pub const ENV_AGENT_ID: &str = "KURULTAI_AGENT_ID";

/// Env var for the write/recall namespace (CLI `--namespace` takes precedence).
pub const ENV_NAMESPACE: &str = "KURULTAI_NAMESPACE";

/// Metadata key holding the self-asserted writer identity.
pub const META_AGENT_ID: &str = "agent_id";

/// Metadata key holding the namespace. Matches [`crate::types::KnowledgeAtom::project_id`].
pub const META_PROJECT_ID: &str = "project_id";

/// Metadata key recording which transport performed the write.
pub const META_WRITE_TRANSPORT: &str = "write_transport";

/// Quarantine reason stamped on writes contained by the closed policy.
pub const CONTAINED_REASON: &str = "agent_write_containment";

/// How agent-reachable writes are treated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WriteMode {
    /// Single-operator box. Gate outcome decides the lane; unchanged legacy behaviour.
    #[default]
    Solo,
    /// Shared store, multiple sessions. Agent writes never reach the trusted lane.
    SharedClosed,
}

impl WriteMode {
    /// Resolve from the `shared_write` feature flag.
    pub fn from_env() -> Self {
        if crate::features::enabled(FEATURE_SHARED_WRITE) {
            WriteMode::SharedClosed
        } else {
            WriteMode::Solo
        }
    }
}

/// Which surface a write arrived on. Used for audit actor strings and promote gating.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteTransport {
    /// MCP stdio child spawned by an agent session.
    Mcp,
    /// Daemon HTTP API.
    Http,
    /// Loopback `POST /ingest`.
    Ingest,
    /// Operator-run `kurultai` CLI. The only transport trusted to promote.
    Cli,
}

impl WriteTransport {
    pub fn as_str(self) -> &'static str {
        match self {
            WriteTransport::Mcp => "mcp",
            WriteTransport::Http => "http",
            WriteTransport::Ingest => "ingest",
            WriteTransport::Cli => "cli",
        }
    }

    /// Whether this transport is reachable by an agent session (i.e. untrusted for promote).
    pub fn is_agent_reachable(self) -> bool {
        !matches!(self, WriteTransport::Cli)
    }
}

/// Identity + policy carried down every write path.
#[derive(Debug, Clone)]
pub struct WriteContext {
    /// Self-asserted. Never treat as an authorization claim.
    pub agent_id: Option<String>,
    /// Namespace for this writer's atoms (`project_id` in atom metadata).
    pub namespace: Option<String>,
    pub transport: WriteTransport,
    pub mode: WriteMode,
}

impl WriteContext {
    /// Legacy/solo context: no identity, gate decides the lane.
    pub fn solo(transport: WriteTransport) -> Self {
        Self {
            agent_id: None,
            namespace: None,
            transport,
            mode: WriteMode::Solo,
        }
    }

    /// Resolve identity from explicit flags, falling back to env, and mode from the feature flag.
    pub fn resolve(
        transport: WriteTransport,
        agent_id: Option<&str>,
        namespace: Option<&str>,
    ) -> Self {
        Self {
            agent_id: pick(agent_id, ENV_AGENT_ID),
            namespace: pick(namespace, ENV_NAMESPACE),
            transport,
            mode: WriteMode::from_env(),
        }
    }

    /// Resolve entirely from env for call sites that have no flags of their own.
    pub fn from_env(transport: WriteTransport) -> Self {
        Self::resolve(transport, None, None)
    }

    /// Whether agent-reachable writes must be forced into quarantine.
    pub fn contains_writes(&self) -> bool {
        self.mode == WriteMode::SharedClosed && self.transport.is_agent_reachable()
    }

    /// Audit actor string. Carries the real (self-asserted) agent id instead of a
    /// hardcoded per-transport constant, so `quality_audit.actor` can attribute a write.
    pub fn actor(&self) -> String {
        match &self.agent_id {
            Some(id) => format!("{}:{}", self.transport.as_str(), id),
            None => self.transport.as_str().to_string(),
        }
    }

    /// Namespace to stamp on writes, if any was claimed.
    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }

    /// Stamp provenance onto atom metadata. Caller-supplied values are overwritten:
    /// an agent must not be able to forge a different writer's provenance in its own atoms.
    pub fn stamp(&self, meta: &mut HashMap<String, String>) {
        meta.insert(
            META_WRITE_TRANSPORT.to_string(),
            self.transport.as_str().to_string(),
        );
        if let Some(id) = &self.agent_id {
            meta.insert(META_AGENT_ID.to_string(), id.clone());
        }
        if let Some(ns) = &self.namespace {
            meta.insert(META_PROJECT_ID.to_string(), ns.clone());
        }
    }
}

/// Parse an actor string produced by [`WriteContext::actor`] back into its transport.
///
/// Unknown/legacy strings are treated as agent-reachable (fail closed).
pub fn actor_transport(actor: &str) -> WriteTransport {
    let head = actor.split(':').next().unwrap_or("").trim();
    match head {
        "cli" => WriteTransport::Cli,
        "http" => WriteTransport::Http,
        "ingest" => WriteTransport::Ingest,
        _ => WriteTransport::Mcp,
    }
}

/// Whether `actor` may perform quarantine -> trusted promotion under the current mode.
///
/// Under [`WriteMode::SharedClosed`] only the operator-run CLI may promote: otherwise a
/// session could self-promote the atom it just wrote and defeat containment entirely.
pub fn promote_allowed(actor: &str, mode: WriteMode) -> bool {
    mode != WriteMode::SharedClosed || !actor_transport(actor).is_agent_reachable()
}

fn pick(explicit: Option<&str>, env_key: &str) -> Option<String> {
    explicit
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .or_else(|| {
            std::env::var(env_key)
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solo_context_does_not_contain_writes() {
        let ctx = WriteContext::solo(WriteTransport::Mcp);
        assert!(!ctx.contains_writes());
        assert_eq!(ctx.actor(), "mcp");
    }

    #[test]
    fn shared_closed_contains_agent_transports_only() {
        for (transport, contained) in [
            (WriteTransport::Mcp, true),
            (WriteTransport::Http, true),
            (WriteTransport::Ingest, true),
            (WriteTransport::Cli, false),
        ] {
            let ctx = WriteContext {
                agent_id: None,
                namespace: None,
                transport,
                mode: WriteMode::SharedClosed,
            };
            assert_eq!(ctx.contains_writes(), contained, "{transport:?}");
        }
    }

    #[test]
    fn actor_carries_agent_id() {
        let ctx = WriteContext {
            agent_id: Some("session-3".into()),
            namespace: Some("kurultai".into()),
            transport: WriteTransport::Mcp,
            mode: WriteMode::SharedClosed,
        };
        assert_eq!(ctx.actor(), "mcp:session-3");
    }

    #[test]
    fn stamp_overwrites_caller_supplied_provenance() {
        let ctx = WriteContext {
            agent_id: Some("real".into()),
            namespace: Some("ns-a".into()),
            transport: WriteTransport::Mcp,
            mode: WriteMode::SharedClosed,
        };
        let mut meta = HashMap::new();
        meta.insert(META_AGENT_ID.to_string(), "forged".to_string());
        meta.insert(META_PROJECT_ID.to_string(), "ns-victim".to_string());
        ctx.stamp(&mut meta);
        assert_eq!(meta[META_AGENT_ID], "real");
        assert_eq!(meta[META_PROJECT_ID], "ns-a");
        assert_eq!(meta[META_WRITE_TRANSPORT], "mcp");
    }

    #[test]
    fn only_cli_may_promote_under_closed_policy() {
        assert!(promote_allowed("cli", WriteMode::SharedClosed));
        assert!(!promote_allowed("mcp:session-1", WriteMode::SharedClosed));
        assert!(!promote_allowed("http", WriteMode::SharedClosed));
        assert!(!promote_allowed("ingest", WriteMode::SharedClosed));
        // Legacy / unknown actor strings fail closed.
        assert!(!promote_allowed("whatever", WriteMode::SharedClosed));
        // Solo mode is unchanged.
        assert!(promote_allowed("mcp", WriteMode::Solo));
    }
}
