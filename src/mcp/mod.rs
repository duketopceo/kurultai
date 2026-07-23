//! Agent interface — MCP exposes two operations on the knowledge brain.
//!
//! **Read** (minimal tokens out): agents pull only what they need to reason.
//! **Write** (minimal tokens in): agents push distilled facts, not raw chat dumps.
//!
//! The brain itself is SQLite + vectors — not markdown. Markdown folders are
//! one ingest source among many.
//!
//! Planned MCP tools (Phase 3, #7):
//!
//! | Tool | Op | Returns / accepts |
//! |------|----|-------------------|
//! | `search` | read | Ranked atom excerpts + scores (not full vault) |
//! | `cite` | read | Single atom: title, excerpt, `source_uri` |
//! | `ask` | read | Synthesized answer + citation list |
//! | `remember` | write | Distilled atom: decision, resolution, tags |
//!
//! HTTP daemon mirrors the same contract for non-MCP agents.

pub mod interface;
pub mod stdio;

pub use interface::{AgentRead, AgentWrite};
pub use stdio::run;
