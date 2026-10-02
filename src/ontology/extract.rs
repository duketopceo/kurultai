//! Zero-LLM edge extraction (competitive-sweep U3, gbrain-style).
//!
//! Structured references already present in atom content become typed graph
//! edges at index time — no LLM in the write path:
//!
//! - `[[Wiki Link]]` → `ent:<atom>` -references-> `ent:<slug>`
//! - `@person` → `ent:<atom>` -references-> `ent:<slug>` + `instance_of class:person`
//! - frontmatter `related:` / `depends_on:` lists → same `references` edges
//!
//! Target stubs are `instance` entities with no `atom_id`; promoting the
//! backing atom later (`ent:{atom_id}`) uses a different id, so stubs never
//! collide with real entities. All writes are upserts — re-index is safe.

use crate::error::Result;
use crate::store::Store;
use crate::types::{KnowledgeAtom, OntologyEntity, OntologyLink, OntologyLinkType};
use std::collections::BTreeMap;

/// Cap on extracted references per atom — a degenerate file can't blow up the graph.
const MAX_REFS_PER_ATOM: usize = 64;

/// Frontmatter keys whose comma/`[list]` values are entity references.
const REF_KEYS: &[&str] = &["related", "depends_on", "references", "links"];

/// A single extracted mention before it touches the store.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Mention {
    /// Slugified target (`vector-db`); used for the stub entity id.
    slug: String,
    /// Display name as written (`Vector DB`).
    name: String,
    /// True for `@name` mentions — target gets `instance_of class:person`.
    is_person: bool,
}

fn slugify(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

/// Scan atom content for `[[...]]`, `@name`, and frontmatter ref keys.
/// Deduped by slug (first surface form wins); deterministic order.
pub fn extract_references(content: &str) -> Vec<(String, String, bool)> {
    let mut mentions: BTreeMap<String, Mention> = BTreeMap::new();
    let mut add = |m: Mention| {
        mentions.entry(m.slug.clone()).or_insert(m);
    };

    for (i, ch) in content.char_indices() {
        if ch == '[' && content[i + 1..].starts_with('[') {
            if let Some(end) = content[i + 2..].find("]]") {
                let raw = content[i + 2..i + 2 + end]
                    .split('|') // [[target|alias]]
                    .next()
                    .unwrap_or("")
                    .split('#') // [[target#section]]
                    .next()
                    .unwrap_or("")
                    .trim();
                if !raw.is_empty() {
                    let slug = slugify(raw);
                    if !slug.is_empty() {
                        add(Mention {
                            slug,
                            name: raw.to_string(),
                            is_person: false,
                        });
                    }
                }
            }
        } else if ch == '@' {
            // @mention — word chars only, must not follow an alphanumeric
            // (email-ish `foo@bar` is not a mention).
            let prev_word = i > 0
                && content[..i]
                    .chars()
                    .last()
                    .is_some_and(|c| c.is_alphanumeric());
            if prev_word {
                continue;
            }
            let tail = &content[i + 1..];
            let name: String = tail
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                .collect();
            if name.len() >= 2 {
                let slug = slugify(&name);
                if !slug.is_empty() {
                    add(Mention {
                        slug,
                        name,
                        is_person: true,
                    });
                }
            }
        }
    }

    // Frontmatter `related:`/`depends_on:`/`references:`/`links:` — value is a
    // comma or `[a, b]` list of names.
    if let Some(rest) = content.trim_start_matches('\u{feff}').strip_prefix("---\n") {
        if let Some(end) = rest.find("\n---") {
            for line in rest[..end].lines() {
                if let Some((k, v)) = line.split_once(':') {
                    let key = k.trim().to_ascii_lowercase();
                    if REF_KEYS.contains(&key.as_str()) {
                        for item in v
                            .trim()
                            .trim_start_matches('[')
                            .trim_end_matches(']')
                            .split(',')
                        {
                            let name = item.trim().trim_matches('"').trim_matches('\'');
                            let slug = slugify(name);
                            if !slug.is_empty() {
                                add(Mention {
                                    slug,
                                    name: name.to_string(),
                                    is_person: false,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    mentions
        .into_values()
        .take(MAX_REFS_PER_ATOM)
        .map(|m| (m.slug, m.name, m.is_person))
        .collect()
}

/// Materialize extracted references for one atom as entities + `references`
/// links. Creates the atom's own `ent:{atom_id}` instance lazily and target
/// stubs `ent:<slug>` as needed. Returns (entities_created, links_upserted).
pub async fn apply_extracted_edges(
    store: &dyn Store,
    atom: &KnowledgeAtom,
    actor: &str,
) -> Result<(usize, usize)> {
    let mentions = extract_references(&atom.content);
    if mentions.is_empty() {
        return Ok((0, 0));
    }

    let from_id = format!("ent:{}", atom.id);
    if store.get_ontology_entity(&from_id).await?.is_none() {
        store
            .upsert_ontology_entity(&OntologyEntity {
                id: from_id.clone(),
                kind: "instance".into(),
                name: atom.title.clone(),
                atom_id: Some(atom.id.clone()),
                attributes: serde_json::json!({"auto": "extract"}),
            })
            .await?;
    }

    let mut entities = 0usize;
    let mut links = 0usize;
    for (slug, name, is_person) in mentions {
        if slug.is_empty() {
            continue;
        }
        let to_id = format!("ent:{slug}");
        if to_id == from_id {
            continue; // self-reference
        }
        if store.get_ontology_entity(&to_id).await?.is_none() {
            store
                .upsert_ontology_entity(&OntologyEntity {
                    id: to_id.clone(),
                    kind: "instance".into(),
                    name: name.clone(),
                    atom_id: None,
                    attributes: serde_json::json!({"auto": "extract"}),
                })
                .await?;
            entities += 1;
        }
        store
            .upsert_ontology_link(&OntologyLink {
                id: format!("link:{from_id}:references:{to_id}"),
                from_id: from_id.clone(),
                to_id: to_id.clone(),
                rel: OntologyLinkType::References,
                confidence: 0.9,
                status: "approved".into(),
                actor: actor.to_string(),
            })
            .await?;
        links += 1;

        if is_person {
            store
                .upsert_ontology_link(&OntologyLink {
                    id: format!("link:{to_id}:instance_of:{}", super::CLASS_PERSON),
                    from_id: to_id.clone(),
                    to_id: super::CLASS_PERSON.into(),
                    rel: OntologyLinkType::InstanceOf,
                    confidence: 0.7,
                    status: "approved".into(),
                    actor: actor.to_string(),
                })
                .await?;
        }
    }
    Ok((entities, links))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wiki_links_extract_deduped() {
        let refs =
            extract_references("See [[Vector Search]] and [[vector-search|the same thing]].");
        assert_eq!(
            refs,
            vec![("vector-search".into(), "Vector Search".into(), false)]
        );
    }

    #[test]
    fn mentions_skip_email_like() {
        let refs = extract_references("ping @alice or mail bob@example.com");
        assert_eq!(refs, vec![("alice".into(), "alice".into(), true)]);
    }

    #[test]
    fn frontmatter_related_extracted() {
        let content = "---\ntitle: X\nrelated: [Device Auth, Lumen Api]\ndepends_on: Embed Pipeline\n---\nbody";
        let refs = extract_references(content);
        let slugs: Vec<_> = refs.iter().map(|r| r.0.as_str()).collect();
        assert!(slugs.contains(&"device-auth"));
        assert!(slugs.contains(&"lumen-api"));
        assert!(slugs.contains(&"embed-pipeline"));
    }

    #[test]
    fn section_anchor_and_alias_stripped() {
        let refs = extract_references("[[Deploy Runbook#Rollback|rollback]]");
        assert_eq!(refs[0].0, "deploy-runbook");
    }
}
