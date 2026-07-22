//! Agent interface — MCP exposes read/write operations on the knowledge brain.
//!
//! **Read** (minimal tokens out): agents pull only what they need to reason.
//! **Write** (minimal tokens in): agents push distilled facts, not raw chat dumps.
//!
//! The brain itself is SQLite + vectors — not markdown. Markdown folders are
//! one ingest source among many.
//!
//! MCP tools (Phase 1 #11 + Phase 2 #6 search):
//!
//! | Tool | Op | Returns / accepts |
//! |------|----|-------------------|
//! | `search` | read | RRF-ranked `AgentAtomView` excerpts (optional rerank) |
//! | `cite` | read | Single citation slice |
//! | `ask` | read | Synthesize answer from retrieved atoms + citations |
//! | `remember` | write | Distilled atom: title, summary, tags |

pub mod brain;
pub mod init;
pub mod interface;
pub mod server;

pub use brain::BrainService;
pub use init::{ensure_default_config, wire_agent, AgentTarget};
pub use interface::{AgentRead, AgentWrite};
pub use server::run_stdio;
