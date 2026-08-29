//! Pure start-fail policy for hub bind × auth (HUB-3). No sockets, no Postgres.

use super::auth::{parse_hub_bind_all, HubAuth, HubGate};
use crate::error::{KurultaiError, Result};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// How the daemon should bind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindKind {
    Loopback,
    /// `0.0.0.0` / `KURULTAI_HUB_BIND=all`
    All,
    /// Tailnet: `KURULTAI_HUB_BIND=tailscale` or a `100.64/10` address.
    Tailscale,
    /// Specific non-loopback, non-tailscale IP.
    Unicast,
}

/// Parsed bind request (kind + listen IP).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BindRequest {
    pub kind: BindKind,
    pub listen: IpAddr,
}

/// Allow or refuse listen before `TcpListener::bind`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HubListenDecision {
    Allow { listen: IpAddr, auth: HubAuth },
    Refuse { reason: String },
}

/// `100.64.0.0/10` (Tailscale CGNAT).
pub fn is_tailscale_cg_nat(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            o[0] == 100 && (64..=127).contains(&o[1])
        }
        IpAddr::V6(_) => false,
    }
}

/// Railway's default public hostname suffix.
pub fn is_public_railway_hostname(host: &str) -> bool {
    host.trim()
        .to_ascii_lowercase()
        .ends_with(".up.railway.app")
}

pub fn parse_allow_public_hub(raw: Option<&str>) -> bool {
    matches!(
        raw.map(str::trim)
            .unwrap_or("")
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

pub fn parse_bind_request(raw: Option<&str>, listen_override: Option<&str>) -> BindRequest {
    let s = raw.map(str::trim).unwrap_or("");
    let override_ip = listen_override.and_then(|v| v.trim().parse::<IpAddr>().ok());

    if s.is_empty()
        || s.eq_ignore_ascii_case("loopback")
        || s.eq_ignore_ascii_case("localhost")
        || s == "127.0.0.1"
        || s == "::1"
    {
        return BindRequest {
            kind: BindKind::Loopback,
            listen: IpAddr::V4(Ipv4Addr::LOCALHOST),
        };
    }

    if parse_hub_bind_all(Some(s)) || s == "::" {
        return BindRequest {
            kind: BindKind::All,
            listen: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        };
    }

    if s.eq_ignore_ascii_case("tailscale") {
        return BindRequest {
            kind: BindKind::Tailscale,
            listen: override_ip.unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
        };
    }

    if let Ok(ip) = s.parse::<IpAddr>() {
        if ip.is_loopback() {
            return BindRequest {
                kind: BindKind::Loopback,
                listen: ip,
            };
        }
        if is_tailscale_cg_nat(ip) {
            return BindRequest {
                kind: BindKind::Tailscale,
                listen: ip,
            };
        }
        return BindRequest {
            kind: BindKind::Unicast,
            listen: ip,
        };
    }

    BindRequest {
        kind: BindKind::Loopback,
        listen: IpAddr::V4(Ipv4Addr::LOCALHOST),
    }
}

/// Deterministic allow/refuse (AE2, AE11, R17, R18).
pub fn hub_listen_decision(
    req: BindRequest,
    auth: HubAuth,
    key_count: usize,
    allow_public_hub: bool,
    public_hostname: Option<&str>,
) -> HubListenDecision {
    if req.kind == BindKind::Loopback {
        return HubListenDecision::Allow {
            listen: req.listen,
            auth,
        };
    }

    let railway_public = public_hostname
        .map(str::trim)
        .filter(|h| !h.is_empty())
        .is_some_and(is_public_railway_hostname);

    if railway_public && !allow_public_hub {
        return HubListenDecision::Refuse {
            reason:
                "refusing public Railway hostname (*.up.railway.app) without ALLOW_PUBLIC_HUB=1"
                    .into(),
        };
    }

    match (req.kind, auth, key_count > 0) {
        (BindKind::Tailscale, _, _) => HubListenDecision::Allow {
            listen: req.listen,
            auth,
        },
        (_, HubAuth::ApiKey, true) => HubListenDecision::Allow {
            listen: req.listen,
            auth,
        },
        (_, HubAuth::ApiKey, false) => HubListenDecision::Refuse {
            reason: "non-loopback bind with hub.auth=api_key requires at least one KURULTAI_HUB_API_KEYS entry"
                .into(),
        },
        (_, HubAuth::None, _) => HubListenDecision::Refuse {
            reason: "non-loopback bind with hub.auth=none is a hard start error (set KURULTAI_HUB_AUTH=api_key and keys, or KURULTAI_HUB_BIND=tailscale)"
                .into(),
        },
    }
}

pub fn allow_public_hub_from_env() -> bool {
    parse_allow_public_hub(std::env::var("ALLOW_PUBLIC_HUB").ok().as_deref())
}

pub fn detect_public_hostname() -> Option<String> {
    for key in ["KURULTAI_PUBLIC_HOSTNAME", "RAILWAY_PUBLIC_DOMAIN"] {
        if let Ok(v) = std::env::var(key) {
            let t = v.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
    }
    if let Ok(url) = std::env::var("RAILWAY_STATIC_URL") {
        let t = url.trim();
        if let Some(host) = host_from_url(t) {
            return Some(host);
        }
    }
    None
}

fn host_from_url(raw: &str) -> Option<String> {
    let rest = raw
        .strip_prefix("https://")
        .or_else(|| raw.strip_prefix("http://"))
        .unwrap_or(raw);
    let host = rest.split('/').next().unwrap_or("").split(':').next()?;
    if host.is_empty() {
        None
    } else {
        Some(host.to_string())
    }
}

/// Bind target from env, with `bind_all` forcing `0.0.0.0`.
pub fn bind_request_from_env(bind_all: bool) -> BindRequest {
    if bind_all {
        return BindRequest {
            kind: BindKind::All,
            listen: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        };
    }
    parse_bind_request(
        std::env::var("KURULTAI_HUB_BIND").ok().as_deref(),
        std::env::var("KURULTAI_HUB_LISTEN").ok().as_deref(),
    )
}

/// Decide listen address or return a start-fail config error.
pub fn resolve_listen_socket(port: u16, bind_all: bool, hub: &HubGate) -> Result<SocketAddr> {
    let req = bind_request_from_env(bind_all);
    match hub_listen_decision(
        req,
        hub.auth,
        hub.api_keys.len(),
        allow_public_hub_from_env(),
        detect_public_hostname().as_deref(),
    ) {
        HubListenDecision::Allow { listen, .. } => Ok(SocketAddr::new(listen, port)),
        HubListenDecision::Refuse { reason } => Err(KurultaiError::config(reason)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(kind: BindKind, listen: IpAddr) -> BindRequest {
        BindRequest { kind, listen }
    }

    #[test]
    fn ae11_bind_all_auth_none_refuses() {
        let d = hub_listen_decision(
            req(BindKind::All, IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
            HubAuth::None,
            0,
            false,
            None,
        );
        match d {
            HubListenDecision::Refuse { reason } => {
                assert!(reason.contains("none"), "{reason}");
            }
            other => panic!("expected refuse, got {other:?}"),
        }
    }

    #[test]
    fn bind_all_api_key_with_key_allows() {
        let d = hub_listen_decision(
            req(BindKind::All, IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
            HubAuth::ApiKey,
            1,
            false,
            None,
        );
        assert!(matches!(
            d,
            HubListenDecision::Allow {
                auth: HubAuth::ApiKey,
                ..
            }
        ));
    }

    #[test]
    fn loopback_auth_none_allows() {
        let d = hub_listen_decision(
            req(BindKind::Loopback, IpAddr::V4(Ipv4Addr::LOCALHOST)),
            HubAuth::None,
            0,
            false,
            None,
        );
        assert!(matches!(d, HubListenDecision::Allow { .. }));
    }

    #[test]
    fn railway_public_hostname_refuses_without_flag() {
        let d = hub_listen_decision(
            req(BindKind::All, IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
            HubAuth::ApiKey,
            1,
            false,
            Some("kurultai.up.railway.app"),
        );
        match d {
            HubListenDecision::Refuse { reason } => {
                assert!(reason.contains("ALLOW_PUBLIC_HUB"), "{reason}");
            }
            other => panic!("expected refuse, got {other:?}"),
        }
    }

    #[test]
    fn railway_public_hostname_allows_with_flag() {
        let d = hub_listen_decision(
            req(BindKind::All, IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
            HubAuth::ApiKey,
            1,
            true,
            Some("kurultai.up.railway.app"),
        );
        assert!(matches!(d, HubListenDecision::Allow { .. }));
    }

    #[test]
    fn tailscale_auth_none_allows() {
        let d = hub_listen_decision(
            req(
                BindKind::Tailscale,
                IpAddr::V4(Ipv4Addr::new(100, 64, 1, 2)),
            ),
            HubAuth::None,
            0,
            false,
            None,
        );
        assert!(matches!(d, HubListenDecision::Allow { .. }));
    }

    #[test]
    fn parse_tailscale_token_and_cgnat() {
        let t = parse_bind_request(Some("tailscale"), None);
        assert_eq!(t.kind, BindKind::Tailscale);
        let ip = parse_bind_request(Some("100.64.1.8"), None);
        assert_eq!(ip.kind, BindKind::Tailscale);
        assert_eq!(ip.listen, IpAddr::V4(Ipv4Addr::new(100, 64, 1, 8)));
    }

    #[test]
    fn parse_all_and_loopback() {
        assert_eq!(parse_bind_request(Some("all"), None).kind, BindKind::All);
        assert_eq!(parse_bind_request(None, None).kind, BindKind::Loopback);
        assert!(parse_allow_public_hub(Some("1")));
        assert!(!parse_allow_public_hub(Some("0")));
        assert!(is_public_railway_hostname("foo.up.railway.app"));
        assert!(!is_public_railway_hostname("foo.railway.internal"));
    }

    #[test]
    fn bind_all_api_key_without_keys_refuses() {
        let d = hub_listen_decision(
            req(BindKind::All, IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
            HubAuth::ApiKey,
            0,
            false,
            None,
        );
        assert!(matches!(d, HubListenDecision::Refuse { .. }));
    }

    fn restore_var(key: &str, prev: Option<String>) {
        match prev {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
    }

    #[test]
    fn resolve_listen_socket_loopback_allows_auth_none() {
        let keys = [
            "KURULTAI_HUB_BIND",
            "KURULTAI_HUB_LISTEN",
            "ALLOW_PUBLIC_HUB",
            "KURULTAI_PUBLIC_HOSTNAME",
            "RAILWAY_PUBLIC_DOMAIN",
            "RAILWAY_STATIC_URL",
        ];
        let prev: Vec<_> = keys.iter().map(|k| (*k, std::env::var(k).ok())).collect();
        for k in keys {
            std::env::remove_var(k);
        }
        let hub = HubGate::default();
        let addr = resolve_listen_socket(8421, false, &hub).unwrap();
        assert_eq!(addr.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
        assert_eq!(addr.port(), 8421);
        for (k, v) in prev {
            restore_var(k, v);
        }
    }

    #[test]
    fn resolve_listen_socket_bind_all_auth_none_is_config_error() {
        let keys = [
            "ALLOW_PUBLIC_HUB",
            "KURULTAI_PUBLIC_HOSTNAME",
            "RAILWAY_PUBLIC_DOMAIN",
            "RAILWAY_STATIC_URL",
        ];
        let prev: Vec<_> = keys.iter().map(|k| (*k, std::env::var(k).ok())).collect();
        for k in keys {
            std::env::remove_var(k);
        }
        let hub = HubGate::default();
        let err = resolve_listen_socket(8421, true, &hub).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("none"), "{msg}");
        for (k, v) in prev {
            restore_var(k, v);
        }
    }

    #[test]
    fn host_from_url_strips_scheme_and_path() {
        assert_eq!(
            host_from_url("https://foo.up.railway.app/path"),
            Some("foo.up.railway.app".into())
        );
        assert_eq!(host_from_url("foo.example"), Some("foo.example".into()));
        assert_eq!(host_from_url(""), None);
    }
}
