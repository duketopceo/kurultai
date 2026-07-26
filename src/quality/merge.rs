//! Safe auto-merge rules for near-duplicate atoms.

use crate::hashutil::sha256_hex;
use crate::types::KnowledgeAtom;

/// Normalize body for Jaccard: lowercase, strip punctuation, collapse whitespace.
pub fn normalize_body(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let mut prev_space = false;
    for ch in content.chars() {
        if ch.is_alphanumeric() {
            out.extend(ch.to_lowercase());
            prev_space = false;
        } else if !prev_space {
            out.push(' ');
            prev_space = true;
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn jaccard(a: &str, b: &str) -> f64 {
    use std::collections::HashSet;
    let ta: HashSet<&str> = a.split_whitespace().collect();
    let tb: HashSet<&str> = b.split_whitespace().collect();
    if ta.is_empty() && tb.is_empty() {
        return 1.0;
    }
    if ta.is_empty() || tb.is_empty() {
        return 0.0;
    }
    let inter = ta.intersection(&tb).count() as f64;
    let union = ta.union(&tb).count() as f64;
    if union == 0.0 {
        0.0
    } else {
        inter / union
    }
}

fn field_conflict(a: &str, b: &str) -> bool {
    let a = a.trim();
    let b = b.trim();
    if a.is_empty() || b.is_empty() {
        return false;
    }
    !a.eq_ignore_ascii_case(b)
}

/// True when titles/summaries/resolutions conflict (both non-empty and disagree).
pub fn has_merge_conflict(a: &KnowledgeAtom, b: &KnowledgeAtom) -> bool {
    if field_conflict(&a.title, &b.title) {
        return true;
    }
    if field_conflict(&a.summary, &b.summary) {
        return true;
    }
    match (&a.resolution, &b.resolution) {
        (Some(ra), Some(rb)) if !ra.trim().is_empty() && !rb.trim().is_empty() => {
            !ra.trim().eq_ignore_ascii_case(rb.trim())
        }
        _ => false,
    }
}

/// Bodies are merge-similar when same hash or normalized Jaccard ≥ threshold.
pub fn bodies_similar(a: &KnowledgeAtom, b: &KnowledgeAtom, jaccard_threshold: f64) -> bool {
    if sha256_hex(&a.content) == sha256_hex(&b.content) {
        return true;
    }
    let na = normalize_body(&a.content);
    let nb = normalize_body(&b.content);
    jaccard(&na, &nb) >= jaccard_threshold
}

/// Choose survivor: older `indexed_at`, then lower id.
pub fn survivor_loser<'a>(
    a: &'a KnowledgeAtom,
    b: &'a KnowledgeAtom,
) -> (&'a KnowledgeAtom, &'a KnowledgeAtom) {
    if a.indexed_at < b.indexed_at {
        (a, b)
    } else if b.indexed_at < a.indexed_at {
        (b, a)
    } else if a.id <= b.id {
        (a, b)
    } else {
        (b, a)
    }
}

/// Additive merge of loser into survivor fields (returns updated survivor clone).
pub fn merge_additive(survivor: &KnowledgeAtom, loser: &KnowledgeAtom) -> KnowledgeAtom {
    let mut out = survivor.clone();
    for t in &loser.tags {
        if !out.tags.iter().any(|x| x == t) {
            out.tags.push(t.clone());
        }
    }
    for (k, v) in &loser.metadata {
        out.metadata.entry(k.clone()).or_insert_with(|| v.clone());
    }
    if out
        .resolution
        .as_ref()
        .map(|s| s.trim().is_empty())
        .unwrap_or(true)
    {
        if let Some(r) = &loser.resolution {
            if !r.trim().is_empty() {
                out.resolution = Some(r.clone());
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;

    fn atom(id: &str, title: &str, content: &str) -> KnowledgeAtom {
        KnowledgeAtom {
            id: id.into(),
            source: "markdown".into(),
            source_id: format!("/{id}"),
            title: title.into(),
            summary: content.into(),
            content: content.into(),
            tags: vec!["a".into()],
            source_updated_at: Utc::now(),
            indexed_at: Utc::now(),
            metadata: HashMap::new(),
            ..Default::default()
        }
    }

    #[test]
    fn jaccard_identical() {
        let n = normalize_body("Hello, World!");
        assert!((jaccard(&n, &n) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn conflict_on_titles() {
        let a = atom("1", "Alpha", "body");
        let b = atom("2", "Beta", "body");
        assert!(has_merge_conflict(&a, &b));
    }

    #[test]
    fn no_conflict_empty_title() {
        let mut a = atom("1", "Alpha", "body");
        let b = atom("2", "", "body");
        a.title = "Alpha".into();
        assert!(!has_merge_conflict(&a, &b));
    }
}
