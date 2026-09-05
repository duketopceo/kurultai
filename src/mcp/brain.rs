//! Brain facade — AgentRead / AgentWrite over the SQLite store.

use crate::activity::ActivityLog;
use crate::brain::{AgentAtomView, DEFAULT_EXCERPT_CAP};
use crate::embed::Embedder;
use crate::error::{KurultaiError, Result};
use crate::hashutil::atom_id;
use crate::mcp::interface::{AgentRead, AgentWrite};
use crate::memory::{GraphNode, MemoryTier, TierPolicy};
use crate::project::{normalize_project, resolve_project, PROJECT_METADATA_KEY};
use crate::quality::{apply_gate, evaluate, promote_atom, PromoteResult};
use crate::query::{expand_markdown_context, hybrid_search_filtered};
use crate::rerank::Reranker;
use crate::store::{SearchFilter, Store};
use crate::synthesize::{who_knows_from_hits, Synthesizer, WhoKnowsEntry};
use crate::types::{Answer, Citation, KnowledgeAtom, SearchResult};
use crate::write_policy::{WriteContext, WriteTransport};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

static REMEMBER_SEQ: AtomicU64 = AtomicU64::new(1);

/// Default connector names sequestered from unscoped hot retrieval (pond session noise).
pub const DEFAULT_NOISY_SOURCES: &[&str] = &["pond"];

/// MCP-facing brain bound to the app store + embedder.
#[derive(Clone)]
pub struct BrainService {
    store: Arc<dyn Store>,
    embedder: Arc<dyn Embedder>,
    reranker: Arc<dyn Reranker>,
    synthesizer: Arc<dyn Synthesizer>,
    activity: Arc<ActivityLog>,
    /// Sources excluded from unscoped search/ask/who_knows/recall (pin `source=` bypasses).
    noisy_sources: Arc<Vec<String>>,
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
            noisy_sources: Arc::new(
                DEFAULT_NOISY_SOURCES
                    .iter()
                    .map(|s| (*s).to_string())
                    .collect(),
            ),
        }
    }

    /// Override noisy-source denylist (empty = sequester nothing).
    pub fn with_noisy_sources(mut self, sources: Vec<String>) -> Self {
        self.noisy_sources = Arc::new(sources);
        self
    }

    pub fn activity(&self) -> Arc<ActivityLog> {
        Arc::clone(&self.activity)
    }

    fn is_noisy_source(&self, source: &str) -> bool {
        self.noisy_sources.iter().any(|s| s == source)
    }

    /// Unscoped: drop noisy sources then diversify. Pinned `source=` keeps that source only.
    fn apply_retrieval_policy(
        &self,
        mut results: Vec<SearchResult>,
        limit: usize,
        source: Option<&str>,
    ) -> Vec<SearchResult> {
        if let Some(src) = source {
            results.retain(|r| r.atom.source == src);
            results.truncate(limit);
            return results;
        }
        results.retain(|r| !self.is_noisy_source(&r.atom.source));
        results = diversify_by_source(results, limit);
        results.truncate(limit);
        results
    }

    async fn touch_non_noisy(&self, results: &[SearchResult]) {
        let ids: Vec<String> = results
            .iter()
            .filter(|r| !self.is_noisy_source(&r.atom.source))
            .map(|r| r.atom.id.clone())
            .collect();
        self.touch_access_many(&ids).await;
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
        self.search_views_scoped(query, limit, include_quarantine, None)
            .await
    }

    pub async fn search_views_scoped(
        &self,
        query: &str,
        limit: usize,
        include_quarantine: bool,
        source: Option<&str>,
    ) -> Result<Vec<AgentAtomView>> {
        let results = self
            .search_scoped(query, limit, include_quarantine, source)
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
            .list_atoms(limit, SearchFilter::trusted(!include_quarantine))
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
        self.search_scoped(query, limit, include_quarantine, None)
            .await
    }

    /// Search, optionally pinned to one connector `source`.
    /// Unscoped results are diversified so pond/dayflow cannot fill every slot.
    pub async fn search_scoped(
        &self,
        query: &str,
        limit: usize,
        include_quarantine: bool,
        source: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        self.search_scoped_hub(query, limit, include_quarantine, source, None)
            .await
    }

    pub async fn search_scoped_hub(
        &self,
        query: &str,
        limit: usize,
        include_quarantine: bool,
        source: Option<&str>,
        hub_team_id: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        let filter = SearchFilter::trusted(!include_quarantine).with_hub_team(hub_team_id);
        let src = source.map(str::trim).filter(|s| !s.is_empty());
        let fetch = if src.is_some() {
            (limit.max(1) * 8).min(80)
        } else {
            (limit.max(1) * 4).min(40)
        };
        let mut results = self.hybrid_hits_filtered(query, fetch, filter).await?;
        results = self.apply_retrieval_policy(results, limit, src);
        let ids: Vec<String> = results.iter().map(|r| r.atom.id.clone()).collect();
        self.touch_non_noisy(&results).await;
        self.activity.record("search", query, ids, None);
        Ok(results)
    }

    pub async fn ask_with_team(&self, question: &str, hub_team_id: Option<&str>) -> Result<Answer> {
        let filter = SearchFilter::default().with_hub_team(hub_team_id);
        let primary = self
            .hybrid_hits_filtered(question, 16, filter.clone())
            .await?;
        let primary = self.apply_retrieval_policy(primary, 8, None);
        let hits = multi_hop_expand(self, primary, 8).await?;
        let hits = self.apply_retrieval_policy(hits, 8, None);
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
        self.touch_non_noisy(&hits).await;
        self.activity.record("ask", question, ids, detail);
        Ok(answer)
    }

    pub async fn who_knows_with_team(
        &self,
        topic: &str,
        limit: usize,
        hub_team_id: Option<&str>,
    ) -> Result<Vec<WhoKnowsEntry>> {
        let filter = SearchFilter::default().with_hub_team(hub_team_id);
        let limit = limit.max(1);
        let mut hits = self
            .hybrid_hits_filtered(topic, (limit * 4).min(40), filter)
            .await?;
        hits = self.apply_retrieval_policy(hits, limit, None);
        let ids: Vec<String> = hits.iter().map(|r| r.atom.id.clone()).collect();
        self.touch_non_noisy(&hits).await;
        self.activity.record("who_knows", topic, ids, None);
        Ok(who_knows_from_hits(&hits))
    }

    /// Agent-optimized recall: project-scoped search returning token-capped views.
    ///
    /// The `project_id` predicate is pushed into SQL (see [`SearchFilter::with_project`])
    /// so scoping happens before any candidate truncation. Filtering after truncation —
    /// the previous prototype behaviour — returned `[]` whenever other sessions'
    /// atoms filled the candidate window, which is exactly the shared-box case.
    ///
    /// This is namespacing, not isolation: any local process running as the same
    /// unix user can pass any project string.
    pub async fn recall_for_agent(
        &self,
        project: &str,
        query: &str,
        limit: usize,
        include_quarantine: bool,
    ) -> Result<Vec<AgentAtomView>> {
        let project = normalize_project(project);
        let filter = SearchFilter::trusted(!include_quarantine).with_project(Some(&project));
        let limit = limit.max(1);
        let mut results = self
            .hybrid_hits_filtered(query, (limit * 4).min(40), filter)
            .await?;
        // Belt and braces: SQL already scoped, this catches any store impl that ignores it.
        results.retain(|r| r.atom.project_id() == project);
        results = self.apply_retrieval_policy(results, limit, None);
        let ids: Vec<String> = results.iter().map(|r| r.atom.id.clone()).collect();
        self.touch_non_noisy(&results).await;
        self.activity
            .record("recall", query, ids, Some(format!("project={project}")));
        Ok(results
            .into_iter()
            .map(|r| AgentAtomView::from_atom(&r.atom, r.score, DEFAULT_EXCERPT_CAP))
            .collect())
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
        source: Option<&str>,
        exclude_source: Option<&str>,
    ) -> Result<Vec<GraphNode>> {
        self.store
            .list_graph_nodes(
                tier,
                limit,
                SearchFilter::trusted(!include_quarantine)
                    .with_source(source)
                    .with_exclude_source(exclude_source),
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

/// Cap how many hits any one connector can take in an unscoped search.
const MAX_HITS_PER_SOURCE: usize = 3;

pub(crate) fn diversify_by_source(results: Vec<SearchResult>, limit: usize) -> Vec<SearchResult> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    let mut kept = Vec::new();
    let mut overflow = Vec::new();
    for r in results {
        let n = counts.get(&r.atom.source).copied().unwrap_or(0);
        if n < MAX_HITS_PER_SOURCE {
            counts.insert(r.atom.source.clone(), n + 1);
            kept.push(r);
            if kept.len() >= limit {
                return kept;
            }
        } else {
            overflow.push(r);
        }
    }
    for r in overflow {
        if kept.len() >= limit {
            break;
        }
        kept.push(r);
    }
    kept
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
        self.ask_with_team(question, None).await
    }

    async fn who_knows(&self, topic: &str, limit: usize) -> Result<Vec<WhoKnowsEntry>> {
        self.who_knows_with_team(topic, limit, None).await
    }
}

#[async_trait::async_trait]
impl AgentWrite for BrainService {
    /// Legacy entry point. Resolves the write context from the environment so the
    /// closed-write policy applies to callers that predate [`WriteContext`].
    async fn remember(
        &self,
        title: &str,
        summary: &str,
        tags: &[String],
        metadata: &[(&str, &str)],
    ) -> Result<String> {
        self.remember_with(
            title,
            summary,
            tags,
            metadata,
            &WriteContext::from_env(WriteTransport::Mcp),
        )
        .await
    }
}

impl BrainService {
    /// Store an agent-authored atom, stamped with the caller's provenance.
    ///
    /// Under [`crate::write_policy::WriteMode::SharedClosed`] the resulting atom is
    /// forced into quarantine **regardless of the quality gate outcome**: on a shared
    /// store no agent-reachable path may write to the globally-searchable lane.
    /// Only `kurultai promote`, run by the operator, moves it out.
    pub async fn remember_with(
        &self,
        title: &str,
        summary: &str,
        tags: &[String],
        metadata: &[(&str, &str)],
        ctx: &WriteContext,
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
        // Always stamp a namespace so agent writes are recallable by project.
        // Caller-supplied project_id wins; otherwise KURULTAI_PROJECT, else "default".
        let project = resolve_project(meta.get(PROJECT_METADATA_KEY).map(String::as_str));
        meta.insert(PROJECT_METADATA_KEY.to_string(), project.clone());
        // Stamp last: caller-supplied agent_id / project_id must not forge provenance.
        ctx.stamp(&mut meta);

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

        // Containment: agent-reachable writes never land in the trusted lane on a
        // shared store, even when the quality gate would have passed them.
        if ctx.contains_writes() && atom.trust_lane == crate::types::TrustLane::Trusted {
            apply_gate(
                &mut atom,
                crate::quality::GateOutcome::Quarantine {
                    reason: crate::write_policy::CONTAINED_REASON.to_string(),
                },
            );
        }

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

    #[test]
    fn diversify_caps_one_source_then_fills() {
        fn hit(id: &str, source: &str) -> SearchResult {
            SearchResult {
                atom: KnowledgeAtom {
                    id: id.into(),
                    source: source.into(),
                    ..Default::default()
                },
                score: 1.0,
                rank: 0,
                matched_by: vec!["fts".into()],
            }
        }
        let pond: Vec<_> = (0..8).map(|i| hit(&format!("p{i}"), "pond")).collect();
        let notes = hit("n1", "notes");
        let mut input = pond;
        input.insert(3, notes);
        let out = super::diversify_by_source(input, 5);
        let pond_n = out.iter().filter(|r| r.atom.source == "pond").count();
        let notes_n = out.iter().filter(|r| r.atom.source == "notes").count();
        assert_eq!(notes_n, 1);
        assert_eq!(out.len(), 5);
        assert!(pond_n <= 4);
    }

    #[tokio::test]
    async fn unscoped_search_excludes_default_noisy_pond() {
        let brain = brain_with_fixture().await;
        let mut pond = crate::types::KnowledgeAtom {
            id: "pond-noise".into(),
            source: "pond".into(),
            source_id: "p1".into(),
            title: "tool call noise LANENOISETOKEN".into(),
            content: "LANENOISETOKEN external_agent_tool_call bash".into(),
            summary: "noise".into(),
            tags: vec!["pond".into()],
            ..Default::default()
        };
        pond.trust_lane = crate::types::TrustLane::Trusted;
        brain.store().upsert(&pond).await.unwrap();

        let hits = brain
            .search_scoped("LANENOISETOKEN", 10, false, None)
            .await
            .unwrap();
        assert!(
            hits.iter().all(|h| h.atom.source != "pond"),
            "unscoped must sequester pond: {:?}",
            hits.iter().map(|h| &h.atom.source).collect::<Vec<_>>()
        );

        let pinned = brain
            .search_scoped("LANENOISETOKEN", 10, false, Some("pond"))
            .await
            .unwrap();
        assert!(
            pinned.iter().any(|h| h.atom.source == "pond"),
            "source=pond pin must still return pond"
        );
    }
}
