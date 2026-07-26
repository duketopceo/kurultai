//! Quality gate, promote, and near-duplicate merge for trust lanes.

pub mod gate;
pub mod merge;
pub mod near_dupe;
pub mod promote;

pub use gate::{apply_gate, evaluate, GateOutcome};
pub use promote::{promote_atom, PromoteResult};
