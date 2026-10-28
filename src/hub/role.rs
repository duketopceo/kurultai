//! Hub RBAC role model (#115): `viewer < contributor < admin`.
//!
//! Roles are stored per issued hub key (`hub_api_keys.role`). Principals that
//! do not come from an issued key (static `KURULTAI_HUB_API_KEYS`, agent keys,
//! Cloudflare Access, the solo/no-auth path) carry no role and are not subject
//! to [`required_role`] checks.

use axum::http::Method;

/// Team-level role of an issued hub key. Ordering is least → most privileged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HubRole {
    /// Read-only: search, ask, who_knows, recall, cite, list.
    Viewer,
    /// Viewer + durable writes (message board, touch, proposal submit).
    Contributor,
    /// Contributor + promote, ontology human-lane writes, proposal decide,
    /// hub activity log.
    Admin,
}

impl HubRole {
    pub fn as_str(self) -> &'static str {
        match self {
            HubRole::Viewer => "viewer",
            HubRole::Contributor => "contributor",
            HubRole::Admin => "admin",
        }
    }

    /// Strict parse for operator input (`--role`).
    pub fn parse(raw: &str) -> Option<HubRole> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "viewer" => Some(HubRole::Viewer),
            "contributor" => Some(HubRole::Contributor),
            "admin" => Some(HubRole::Admin),
            _ => None,
        }
    }

    /// Lenient parse for stored values: anything unrecognised is the least
    /// privileged role (fail closed).
    pub fn from_stored(raw: &str) -> HubRole {
        HubRole::parse(raw).unwrap_or(HubRole::Viewer)
    }
}

impl std::fmt::Display for HubRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// POST endpoints that only read. Everything else that mutates needs at least
/// [`HubRole::Contributor`].
const READ_ONLY_POSTS: &[&str] = &[
    "/api/search",
    "/search",
    "/api/recall",
    "/api/ask",
    "/ask",
    "/cite",
    "/who_knows",
    "/api/metrics/client",
];

/// Routes that always need [`HubRole::Admin`], whatever the method.
fn is_admin_route(path: &str) -> bool {
    path == "/api/hub/activity"
        || path == "/api/promote"
        || path == "/api/ontology/promote"
        || path == "/api/ontology/entity"
        || path == "/api/ontology/link"
        || path.starts_with("/api/ontology/entity/")
        || path.starts_with("/api/ontology/link/")
        || (path.starts_with("/api/ontology/proposals/") && path.ends_with("/decide"))
}

/// Minimum role needed for `method path`. Unknown mutating routes default to
/// contributor so a newly added write route is never viewer-reachable.
pub fn required_role(method: &Method, path: &str) -> HubRole {
    if is_admin_route(path) {
        return HubRole::Admin;
    }
    let mutating = !matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS);
    if !mutating || READ_ONLY_POSTS.contains(&path) {
        HubRole::Viewer
    } else {
        HubRole::Contributor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roles_order_least_to_most_privileged() {
        assert!(HubRole::Viewer < HubRole::Contributor);
        assert!(HubRole::Contributor < HubRole::Admin);
    }

    #[test]
    fn parse_round_trips_and_unknown_fails_closed() {
        for r in [HubRole::Viewer, HubRole::Contributor, HubRole::Admin] {
            assert_eq!(HubRole::parse(r.as_str()), Some(r));
            assert_eq!(HubRole::from_stored(r.as_str()), r);
        }
        assert_eq!(HubRole::parse(" Admin "), Some(HubRole::Admin));
        assert_eq!(HubRole::parse("root"), None);
        assert_eq!(HubRole::from_stored("root"), HubRole::Viewer);
        assert_eq!(HubRole::from_stored(""), HubRole::Viewer);
    }

    #[test]
    fn reads_and_read_only_posts_need_viewer() {
        for p in [
            "/api/search",
            "/search",
            "/api/recall",
            "/api/ask",
            "/ask",
            "/cite",
            "/who_knows",
        ] {
            assert_eq!(required_role(&Method::POST, p), HubRole::Viewer, "{p}");
        }
        assert_eq!(required_role(&Method::GET, "/api/atoms"), HubRole::Viewer);
        assert_eq!(required_role(&Method::GET, "/api/search"), HubRole::Viewer);
        assert_eq!(required_role(&Method::GET, "/api/status"), HubRole::Viewer);
    }

    #[test]
    fn board_and_touch_writes_need_contributor() {
        assert_eq!(
            required_role(&Method::POST, "/api/hey/threads"),
            HubRole::Contributor
        );
        assert_eq!(
            required_role(&Method::POST, "/api/touch"),
            HubRole::Contributor
        );
        assert_eq!(
            required_role(&Method::POST, "/api/ontology/proposals"),
            HubRole::Contributor
        );
        // Unknown mutating route: fail safe.
        assert_eq!(
            required_role(&Method::POST, "/api/some/new/write"),
            HubRole::Contributor
        );
        assert_eq!(
            required_role(&Method::DELETE, "/api/some/new/write"),
            HubRole::Contributor
        );
    }

    #[test]
    fn promote_ontology_writes_decide_and_activity_need_admin() {
        for (m, p) in [
            (Method::POST, "/api/promote"),
            (Method::POST, "/api/ontology/promote"),
            (Method::POST, "/api/ontology/entity"),
            (Method::POST, "/api/ontology/link"),
            (Method::DELETE, "/api/ontology/entity/e1"),
            (Method::DELETE, "/api/ontology/link/l1"),
            (Method::POST, "/api/ontology/proposals/p1/decide"),
            (Method::GET, "/api/hub/activity"),
        ] {
            assert_eq!(required_role(&m, p), HubRole::Admin, "{m} {p}");
        }
    }
}
