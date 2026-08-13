//! Brain facade — AgentRead / AgentWrite over the SQLite store.

use crate::activity::ActivityLog;
use crate::brain::{AgentAtomView, DEFAULT_EXCERPT_CAP};
use crate::embed::Embedder;
use crate::error::{KurultaiError, Result};
use crate::hashutil::atom_id;
use crate::mcp::interface::{AgentRead, AgentWrite};
use crate::memory::{GraphNode, MemoryTier, TierPolicy};
use crate::quality::{apply_gate, evaluate, promote_atom, PromoteResult};
use crate::query::{expand_markdown_context, hybrid_search_filtered};
use crate::rerank::Reranker;
use crate::store::{SearchFilter, Store};
use crate::synthesize::{who_knows_from_hits, Synthesizer, WhoKnowsEntry};
use crate::types::{Answer, Citation, KnowledgeAtom, SearchResult};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

static REMEMBER_SEQ: AtomicU64 = AtomicU64::new(1);

/// MCP-facing brain bound to the app store + embedder.
#[derive(Clone)]
pub struct BrainService {
    store: Arc<dyn Store>,
    embedder: Arc<dyn Embedder>,
    reranker: Arc<dyn Reranker>,
    synthesizer: Arc<dyn Synthesizer>,
    activity: Arc<ActivityLog>,
}

/// Second-hop expansion: search shared tags from primary hits, merge unique atoms (#74).
async fn multi_hop_expand(
    brain: &BrainService,
    primary: Vec<SearchResult>,
    limit: usize,
) -> Result<Vec<SearchResult>> {
    if primary.is_empty() {
        return Ok(primary);
    }
    let mut tags: Vec<String> = primary
        .iter()
        .flat_map(|r| r.atom.tags.iter().cloned())
        .filter(|t| t.len() >= 2)
        .collect();
    tags.sort();
    tags.dedup();
    if tags.is_empty() {
        // Fall back: use distinctive title tokens as hop queries
        tags = primary
            .iter()
            .flat_map(|r| {
                r.atom
                    .title
                    .split(|c: char| !c.is_alphanumeric())
                    .filter(|w| w.len() > 3)
                    .map(|w| w.to_ascii_lowercase())
            })
            .take(4)
            .collect();
        tags.sort();
        tags.dedup();
    }

    let mut by_id: HashMap<String, SearchResult> = HashMap::new();
    let primary_ids: std::collections::HashSet<String> =
        primary.iter().map(|r| r.atom.id.clone()).collect();
    for r in primary {
        by_id.insert(r.atom.id.clone(), r);
    }

    // Bounded second-hop searches — concurrent fan-out (≤4).
    let tags: Vec<String> = tags.into_iter().take(4).collect();
    let mut set = tokio::task::JoinSet::new();
    for tag in tags {
        let brain = brain.clone();
        set.spawn(async move {
            let hits = brain.hybrid_hits(&tag, 4).await.unwrap_or_default();
            let ids: Vec<String> = hits.iter().map(|r| r.atom.id.clone()).collect();
            brain.activity.record("search_hop", &tag, ids, None);
            hits
        });
    }
    while let Some(joined) = set.join_next().await {
        let hop = match joined {
            Ok(v) => v,
            Err(_) => continue,
        };
        for mut r in hop {
            if primary_ids.contains(&r.atom.id) {
                continue;
            }
            if !r.matched_by.iter().any(|m| m == "multi_hop") {
                r.matched_by.push("multi_hop".into());
            }
            by_id.entry(r.atom.id.clone()).or_insert(r);
        }
    }

    let mut merged: Vec<SearchResult> = by_id.into_values().collect();
    merged.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    // re-rank indices
    for (i, r) in merged.iter_mut().enumerate() {
        r.rank = i + 1;
    }
    merged.truncate(limit.max(1));
    Ok(merged)
}

impl BrainService {
    pub fn new(
        store: Arc<dyn Store>,
        embedder: Arc<dyn Embedder>,
        reranker: Arc<dyn Reranker>,
        synthesizer: Arc<dyn Synthesizer>,
    ) -> Self {
        Self::with_activity(
            store,
            embedder,
            reranker,
            synthesizer,
            Arc::new(ActivityLog::new()),
        )
    }

    pub fn with_activity(
        store: Arc<dyn Store>,
        embedder: Arc<dyn Embedder>,
        reranker: Arc<dyn Reranker>,
        synthesizer: Arc<dyn Synthesizer>,
        activity: Arc<ActivityLog>,
    ) -> Self {
        Self {
            store,
            embedder,
            reranker,
            synthesizer,
            activity,
        }
    }

    pub fn activity(&self) -> Arc<ActivityLog> {
        Arc::clone(&self.activity)
    }

    /// Hybrid search + markdown context expand (no activity record).
    async fn hybrid_hits(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        self.hybrid_hits_filtered(query, limit, SearchFilter::default())
            .await
    }

    async fn hybrid_hits_filtered(
        &self,
        query: &str,
        limit: usize,
        filter: SearchFilter,
    ) -> Result<Vec<SearchResult>> {
        let results = hybrid_search_filtered(
            &self.store,
            &self.embedder,
            &self.reranker,
            query,
            limit,
            filter,
        )
        .await?;
        expand_markdown_context(&self.store, results).await
    }

    /// Search returning token-capped views (primary MCP payload).
    pub async fn search_views(&self, query: &str, limit: usize) -> Result<Vec<AgentAtomView>> {
        self.search_views_filtered(query, limit, false).await
    }

    pub async fn search_views_filtered(
        &self,
        query: &str,
        limit: usize,
        include_quarantine: bool,
    ) -> Result<Vec<AgentAtomView>> {
        let results = self
            .search_filtered(query, limit, include_quarantine)
            .await?;
        Ok(results
            .into_iter()
            .map(|r| AgentAtomView::from_atom(&r.atom, r.score, DEFAULT_EXCERPT_CAP))
            .collect())
    }

    /// Total atoms in the store (dashboard / status).
    pub async fn atom_count(&self) -> Result<u64> {
        self.store.count().await
    }

    pub async fn lane_counts(&self) -> Result<(u64, u64, u64)> {
        let trusted = self
            .store
            .count_by_lane(crate::types::TrustLane::Trusted)
            .await?;
        let quarantine = self
            .store
            .count_by_lane(crate::types::TrustLane::Quarantine)
            .await?;
        let merge_pending = self.store.count_merge_candidates_pending().await?;
        Ok((trusted, quarantine, merge_pending))
    }

    /// Return up to `limit` atoms ordered newest-first (dashboard default view).
    pub async fn list_atoms(&self, limit: usize) -> Result<Vec<KnowledgeAtom>> {
        self.list_atoms_filtered(limit, false).await
    }

    pub async fn list_atoms_filtered(
        &self,
        limit: usize,
        include_quarantine: bool,
    ) -> Result<Vec<KnowledgeAtom>> {
        self.store
            .list_atoms(
                limit,
                SearchFilter {
                    trusted_only: !include_quarantine,
                },
            )
            .await
    }

    /// Explicit promote (never called from remember).
    pub async fn promote(
        &self,
        atom_id: &str,
        actor: &str,
        reason: Option<&str>,
    ) -> Result<PromoteResult> {
        let res = promote_atom(self.store.as_ref(), atom_id, actor, reason).await?;
        self.activity
            .record("promote", atom_id, vec![atom_id.to_string()], None);
        Ok(res)
    }

    pub fn store(&self) -> Arc<dyn Store> {
        Arc::clone(&self.store)
    }

    /// Embedder handle for loopback ingest (skip-on-quarantine applied by caller).
    pub fn embedder(&self) -> Arc<dyn Embedder> {
        Arc::clone(&self.embedder)
    }

    /// Search with optional quarantine inclusion (HTTP / MCP).
    pub async fn search_filtered(
        &self,
        query: &str,
        limit: usize,
        include_quarantine: bool,
    ) -> Result<Vec<SearchResult>> {
        let filter = SearchFilter {
            trusted_only: !include_quarantine,
        };
        let results = self.hybrid_hits_filtered(query, limit, filter).await?;
        let ids: Vec<String> = results.iter().map(|r| r.atom.id.clone()).collect();
        self.touch_access_many(&ids).await;
        self.activity.record("search", query, ids, None);
        Ok(results)
    }

    /// Hot / warm / cold counts under default [`TierPolicy`].
    pub async fn tier_counts(&self) -> Result<(u64, u64, u64)> {
        self.store.count_by_tier(TierPolicy::default()).await
    }

    /// Graph stubs for the Brain UI (foveated whole-brain load).
    pub async fn list_graph_nodes(
        &self,
        tier: Option<MemoryTier>,
        limit: usize,
        include_quarantine: bool,
    ) -> Result<Vec<GraphNode>> {
        self.store
            .list_graph_nodes(
                tier,
                limit,
                SearchFilter {
                    trusted_only: !include_quarantine,
                },
                TierPolicy::default(),
            )
            .await
    }

    /// Bump access timestamp (UI focus / explicit touch).
    pub async fn touch_access(&self, id: &str) -> Result<Option<KnowledgeAtom>> {
        self.store.touch_access(id).await?;
        self.store.get(id).await
    }

    async fn touch_access_many(&self, ids: &[String]) {
        for id in ids {
            let _ = self.store.touch_access(id).await;
        }
    }
}

fn citation_from_atom(atom: &KnowledgeAtom, score: f64, include_url: bool) -> Citation {
    let view = AgentAtomView::from_atom(atom, score, DEFAULT_EXCERPT_CAP);
    let mut c = Citation::from_atom(atom, view.excerpt);
    if !include_url {
        c.url = None;
    }
    c
}

#[async_trait::async_trait]
impl AgentRead for BrainService {
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        self.search_filtered(query, limit, false).await
    }

    async fn cite(&self, source: &str, source_id: &str) -> Result<Option<Citation>> {
        let Some(atom) = self.store.get_by_source_id(source, source_id).await? else {
            self.activity.record(
                "cite",
                &format!("{source}/{source_id}"),
                vec![],
                Some("miss".into()),
            );
            return Ok(None);
        };
        let id = atom.id.clone();
        let _ = self.store.touch_access(&id).await;
        self.activity
            .record("cite", &format!("{source}/{source_id}"), vec![id], None);
        Ok(Some(citation_from_atom(&atom, 1.0, true)))
    }

    async fn ask(&self, question: &str) -> Result<Answer> {
        let primary = self.hybrid_hits(question, 8).await?;
        let hits = multi_hop_expand(self, primary, 8).await?;
        let mut answer = self.synthesizer.synthesize(question, &hits).await?;
        if answer.graph_chain.is_empty() {
            answer.graph_chain = crate::synthesize::graph_chain_from_hits(&hits);
        }
        let ids: Vec<String> = hits.iter().map(|r| r.atom.id.clone()).collect();
        let detail: Option<String> = {
            let t: String = answer.answer.chars().take(160).collect();
            if t.is_empty() {
                None
            } else {
                Some(t)
            }
        };
        self.activity.record("ask", question, ids, detail);
        Ok(answer)
    }

    async fn who_knows(&self, topic: &str, limit: usize) -> Result<Vec<WhoKnowsEntry>> {
        let hits = self.hybrid_hits(topic, limit.max(1)).await?;
        let ids: Vec<String> = hits.iter().map(|r| r.atom.id.clone()).collect();
        self.activity.record("who_knows", topic, ids, None);
        Ok(who_knows_from_hits(&hits))
    }
}

#[async_trait::async_trait]
impl AgentWrite for BrainService {
    async fn remember(
        &self,
        title: &str,
        summary: &str,
        tags: &[String],
        metadata: &[(&str, &str)],
    ) -> Result<String> {
        if title.trim().is_empty() || summary.trim().is_empty() {
            return Err(KurultaiError::config(
                "remember requires non-empty title and summary",
            ));
        }

        // Clamp write payload — agents must distill, not dump chat.
        let title: String = title.chars().take(200).collect();
        let summary: String = summary.chars().take(4_000).collect();

        let mut meta = HashMap::new();
        for (k, v) in metadata {
            meta.insert((*k).to_string(), (*v).to_string());
        }

        let source = "agent";
        let source_id = format!(
            "remember/{}_{}",
            Utc::now().timestamp_nanos_opt().unwrap_or(0),
            REMEMBER_SEQ.fetch_add(1, Ordering::Relaxed)
        );
        let content = summary.clone();
        let id = atom_id(source, &source_id, &content);

        let mut atom = KnowledgeAtom {
            id: id.clone(),
            source: source.into(),
            source_id,
            title: title.clone(),
            summary: summary.chars().take(280).collect(),
            content,
            question: None,
            resolution: None,
            tags: tags.to_vec(),
            soft_labels: vec![],
            source_updated_at: Utc::now(),
            indexed_at: Utc::now(),
            embedding: None,
            metadata: meta,
            ..Default::default()
        };

        let outcome = evaluate(self.store.as_ref(), &atom).await?;
        apply_gate(&mut atom, outcome);

        // KTD7: skip embed on quarantine (don't pay / pollute atoms_vec).
        if atom.trust_lane == crate::types::TrustLane::Trusted && self.embedder.is_live() {
            let text = format!("{}\n{}", atom.title, atom.content);
            if let Ok(emb) = self.embedder.embed(&text).await {
                atom.embedding = Some(emb);
            }
        } else {
            atom.embedding = None;
        }

        let lane = atom.trust_lane.as_str().to_string();
        let q_reason = atom.quarantine_reason.clone();
        self.store.upsert(&atom).await?;
        let detail = match q_reason {
            Some(r) => Some(format!("{lane}:{r}")),
            None => Some(lane),
        };
        self.activity
            .record("remember", &title, vec![id.clone()], detail);
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connectors::markdown::MarkdownConnector;
    use crate::connectors::Connector;
    use crate::embed::NullEmbedder;
    use crate::pipeline::IndexPipeline;
    use crate::rerank::NullReranker;
    use crate::store::SqliteVecStore;
    use crate::synthesize::ExtractiveSynthesizer;
    use crate::types::{SourceConfig, SourceKind};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    async fn brain_with_fixture() -> BrainService {
        static N: AtomicU64 = AtomicU64::new(0);
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vault");
        let db_dir = std::env::temp_dir().join(format!(
            "kurultai-mcp-{}-{}-{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or(0),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&db_dir).unwrap();
        let store = Arc::new(SqliteVecStore::open(db_dir.join("store.db"), 4).unwrap());
        let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
        let pipeline =
            IndexPipeline::new(Arc::clone(&store) as Arc<dyn Store>, Arc::clone(&embedder));

        let mut connector = MarkdownConnector::new();
        let mut extra = HashMap::new();
        extra.insert("root_path".into(), fixture.to_string_lossy().into_owned());
        connector
            .init(&SourceConfig {
                name: "notes".into(),
                kind: SourceKind::Markdown,
                enabled: true,
                poll_interval_secs: 60,
                extra,
            })
            .await
            .unwrap();
        pipeline
            .index_connector("notes", &connector, true)
            .await
            .unwrap();

        BrainService::new(
            store,
            embedder,
            Arc::new(NullReranker::new()),
            Arc::new(ExtractiveSynthesizer::new()),
        )
    }

    #[tokio::test]
    async fn search_returns_capped_views() {
        let brain = brain_with_fixture().await;
        let views = brain
            .search_views("KNOWN_PHRASE_KURULTAI_42", 5)
            .await
            .unwrap();
        assert!(!views.is_empty());
        assert!(views[0].excerpt.chars().count() <= DEFAULT_EXCERPT_CAP);
        // Full vault content must not appear as unbounded dump
        assert!(!views[0].excerpt.contains(&"x".repeat(500)));
    }

    #[tokio::test]
    async fn blank_query_returns_empty() {
        let brain = brain_with_fixture().await;
        let hits = brain.search("   ", 5).await.unwrap();
        assert!(hits.is_empty());
    }

    #[tokio::test]
    async fn fts_only_marks_matched_by_fts() {
        let brain = brain_with_fixture().await;
        let hits = brain.search("KNOWN_PHRASE_KURULTAI_42", 5).await.unwrap();
        assert!(!hits.is_empty());
        assert!(hits[0].matched_by.iter().any(|m| m == "fts"));
        assert!(!hits[0].matched_by.iter().any(|m| m == "vector"));
    }

    #[tokio::test]
    async fn ask_extractive_from_fixture() {
        let brain = brain_with_fixture().await;
        let answer = brain.ask("KNOWN_PHRASE_KURULTAI_42").await.unwrap();
        assert!(answer.confidence > 0.0);
        assert!(!answer.citations.is_empty());
        assert!(
            answer.answer.contains("KNOWN_PHRASE_KURULTAI_42")
                || answer
                    .citations
                    .iter()
                    .any(|c| c.excerpt.contains("KNOWN_PHRASE_KURULTAI_42"))
        );
    }

    #[tokio::test]
    async fn who_knows_returns_markdown_source() {
        let brain = brain_with_fixture().await;
        let entries = brain
            .who_knows("KNOWN_PHRASE_KURULTAI_42", 10)
            .await
            .unwrap();
        assert!(!entries.is_empty());
        assert!(entries.iter().any(|e| e.source == "notes"));
    }

    #[tokio::test]
    async fn remember_creates_agent_atom() {
        let brain = brain_with_fixture().await;
        let id = brain
            .remember(
                "Decision",
                "Use FTS-first boot without API keys so local search works offline for operators and agents.",
                &["architecture".into()],
                &[("via", "test")],
            )
            .await
            .unwrap();
        assert!(!id.is_empty());
        let hits = brain.search("FTS-first boot", 5).await.unwrap();
        let hit = hits
            .iter()
            .find(|h| h.atom.source == "agent" && h.atom.id == id)
            .expect("remembered atom searchable");
        assert_eq!(hit.atom.title, "Decision");
        assert!(hit.atom.tags.iter().any(|t| t == "architecture"));
        assert_eq!(
            hit.atom.metadata.get("via").map(String::as_str),
            Some("test")
        );
        assert!(hit.atom.source_id.starts_with("remember/"));

        let err = brain.remember(" ", "ok", &[], &[]).await.unwrap_err();
        assert!(err.to_string().contains("non-empty"));
    }
}
