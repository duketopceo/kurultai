//! Token-budgeted markdown neighbor expansion after ranking.

use crate::brain::DEFAULT_EXCERPT_CAP;
use crate::error::Result;
use crate::store::Store;
use crate::types::{KnowledgeAtom, SearchResult};
use std::collections::HashSet;
use std::sync::Arc;

/// Expand markdown hits with same-file prev/next chunk context into `summary`
/// (excerpt source), hit-first, under `DEFAULT_EXCERPT_CAP`.
pub async fn expand_markdown_context(
    store: &Arc<dyn Store>,
    mut results: Vec<SearchResult>,
) -> Result<Vec<SearchResult>> {
    if results.is_empty() {
        return Ok(results);
    }

    let top_ids: HashSet<String> = results.iter().map(|r| r.atom.id.clone()).collect();

    for result in &mut results {
        let Some((rel_path, idx)) = chunk_coords(&result.atom) else {
            continue;
        };
        let source = result.atom.source.clone();

        let mut parts: Vec<String> = Vec::new();
        parts.push(hit_excerpt(&result.atom));

        if idx > 0 {
            if let Some(prev) = store
                .get_by_chunk_meta(&source, &rel_path, idx - 1)
                .await?
            {
                if !top_ids.contains(&prev.id) {
                    parts.push(format!("…prev: {}", neighbor_snippet(&prev)));
                }
            }
        }

        if let Some(next) = store
            .get_by_chunk_meta(&source, &rel_path, idx + 1)
            .await?
        {
            if !top_ids.contains(&next.id) {
                parts.push(format!("…next: {}", neighbor_snippet(&next)));
            }
        }

        let merged = merge_budgeted(&parts, DEFAULT_EXCERPT_CAP);
        result.atom.summary = merged;
    }

    Ok(results)
}

fn chunk_coords(atom: &KnowledgeAtom) -> Option<(String, u32)> {
    let rel = atom.metadata.get("rel_path")?.clone();
    let idx: u32 = atom.metadata.get("chunk_index")?.parse().ok()?;
    Some((rel, idx))
}

fn hit_excerpt(atom: &KnowledgeAtom) -> String {
    if !atom.summary.trim().is_empty() {
        atom.summary.clone()
    } else {
        atom.content.chars().take(DEFAULT_EXCERPT_CAP).collect()
    }
}

fn neighbor_snippet(atom: &KnowledgeAtom) -> String {
    let raw = if !atom.summary.trim().is_empty() {
        atom.summary.as_str()
    } else {
        atom.content.as_str()
    };
    raw.chars().take(120).collect()
}

fn merge_budgeted(parts: &[String], cap: usize) -> String {
    let mut out = String::new();
    for (i, part) in parts.iter().enumerate() {
        let sep = if i == 0 { "" } else { "\n" };
        let remaining = cap.saturating_sub(out.chars().count());
        if remaining == 0 {
            break;
        }
        let room = remaining.saturating_sub(sep.chars().count());
        if room == 0 {
            break;
        }
        out.push_str(sep);
        out.extend(part.chars().take(room));
    }
    out.chars().take(cap).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_hit_first_under_cap() {
        let merged = merge_budgeted(
            &[
                "HIT".into(),
                "…prev: PREVIOUS".into(),
                "…next: NEXT".into(),
            ],
            20,
        );
        assert!(merged.starts_with("HIT"));
        assert!(merged.chars().count() <= 20);
    }
}
