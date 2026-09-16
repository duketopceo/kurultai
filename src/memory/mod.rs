//! Three-tier memory: hot / warm / cold, driven by timestamps.

pub mod tier;

pub use tier::{classify, classify_atom, GraphNode, MemoryTier, TierPolicy, TierRule};
