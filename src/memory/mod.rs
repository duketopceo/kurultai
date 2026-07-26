//! Three-tier memory: hot / warm / cold, driven by timestamps.

pub mod tier;

pub use tier::{classify, GraphNode, MemoryTier, TierPolicy};
