//! Per-device broker daemon (`kurultai broker`).
//!
//! One long-lived localhost service holds the machine's single upstream seat
//! session to a hosted instance (e.g. `knowledge.shippedit.dev`). Agents board
//! it over loopback with minted per-chat session keys — they never see the
//! upstream credential. Plan: `docs/plans/2026-09-28-001` (D1–D6).
//!
//! Security invariants (R6): the listener is strictly loopback (TCP
//! `127.0.0.1` and/or a unix socket) — never `0.0.0.0`; the upstream token is
//! resolved from the keyring/agent-key file at startup and held in memory
//! only.

pub mod board;
pub mod registry;
pub mod relay;
pub mod server;

pub use registry::{default_db_path, Registry};
pub use server::{run, BrokerOptions};
