//! LLM distillation hooks (Phase 6). Soft-label emit is a stub until #12 lands.

use crate::types::{KnowledgeAtom, SoftLabel};

/// Propose soft labels for an atom (distillation path).
///
/// Thin stub for [#113](https://github.com/duketopceo/kurultai/issues/113) /
/// [#12](https://github.com/duketopceo/kurultai/issues/12): returns empty until
/// an LLM distillation pipeline is wired. Callers may still set soft labels
/// explicitly on [`KnowledgeAtom::soft_labels`] before upsert.
pub fn emit_soft_labels(_atom: &KnowledgeAtom) -> Vec<SoftLabel> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_returns_empty() {
        let atom = KnowledgeAtom::default();
        assert!(emit_soft_labels(&atom).is_empty());
    }
}
