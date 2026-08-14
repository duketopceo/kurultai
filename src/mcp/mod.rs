//! Agent interface — MCP exposes read/write operations on the knowledge brain.
//!
//! **Read** (minimal tokens out): agents pull only what they need to reason.
//! **Write** (minimal tokens in): agents push distilled facts, not raw chat dumps.
//!
//! The brain itself is SQLite + vectors — not markdown. Markdown folders are
//! one ingest source among many.
//!
//! MCP tools (Phase 1–3):
//!
//! | Tool | Op | Returns / accepts |
//! |------|----|-------------------|
//! | `search` | read | RRF-ranked `AgentAtomView` excerpts (optional rerank) |
//! | `cite` | read | Single citation slice |
//! | `ask` | read | Synthesized answer + citations + confidence |
//! | `who_knows` | read | Distinct sources matching a topic |
//! | `promote` | write | Quarantine → trusted (never a side effect of remember) |
//! | `ontology_get` | read | Entities + typed links (seeded classes when empty) |
//! | `ontology_promote` | write | Atom → instance entity + `instance_of` (does not change trust_lane) |

pub mod brain;
pub mod init;
pub mod interface;
pub mod server;

pub use brain::BrainService;
pub use init::{
    ensure_default_config, init_walkthrough, provision_docs, wire_agent, AgentTarget, DocsProvision,
};
pub use interface::{AgentRead, AgentWrite};
pub use server::{handle_message, run_stdio, ToolSurface};
