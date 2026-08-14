//! Acceptance tests — MCP tool surface (KHAN-251).
//!
//! Covers all 8 MCP tools via the in-process `handle_message` JSON-RPC path:
//! search, cite, remember, ask, who_knows, promote, ontology_get,
//! ontology_promote. Also covers the read-only (HTTP/SSE) surface gate.

use chrono::Utc;
use kurultai::connectors::markdown::MarkdownConnector;
use kurultai::connectors::Connector;
use kurultai::embed::NullEmbedder;
use kurultai::mcp::brain::BrainService;
use kurultai::mcp::interface::AgentRead;
use kurultai::mcp::server::{handle_message, ToolSurface};
use kurultai::pipeline::IndexPipeline;
use kurultai::rerank::NullReranker;
use kurultai::store::{SqliteVecStore, Store};
use kurultai::synthesize::ExtractiveSynthesizer;
use kurultai::types::{SourceConfig, SourceKind, TrustLane};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

fn fixture_vault() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vault")
}

async fn brain() -> BrainService {
    static N: AtomicU64 = AtomicU64::new(0);
    let db_dir = std::env::temp_dir().join(format!(
        "khan251-mcp-{}-{}-{}",
        std::process::id(),
        Utc::now().timestamp_nanos_opt().unwrap_or(0),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&db_dir).unwrap();
    let store = Arc::new(SqliteVecStore::open(db_dir.join("store.db"), 4).unwrap());
    let embedder: Arc<dyn kurultai::embed::Embedder> = Arc::new(NullEmbedder::new(4));
    let pipeline = IndexPipeline::new(Arc::clone(&store) as Arc<dyn Store>, Arc::clone(&embedder));
    let mut connector = MarkdownConnector::new();
    let mut extra = HashMap::new();
    extra.insert(
        "root_path".into(),
        fixture_vault().to_string_lossy().into_owned(),
    );
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

async fn call(brain: &BrainService, name: &str, args: serde_json::Value) -> serde_json::Value {
    let resp = handle_message(
        brain,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": name, "arguments": args }
        }),
        ToolSurface::Full,
    )
    .await
    .unwrap()
    .expect("response");
    resp
}

fn text_of(resp: &serde_json::Value) -> &str {
    resp["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_else(|| panic!("no text in response: {resp}"))
}

// ── initialize + tools/list ──────────────────────────────────────────────────

#[tokio::test]
async fn initialize_returns_server_info() {
    let brain = brain().await;
    let resp = handle_message(
        &brain,
        json!({"jsonrpc":"2.0","id":1,"method":"initialize"}),
        ToolSurface::Full,
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(resp["result"]["serverInfo"]["name"], "kurultai");
    assert!(resp["result"]["protocolVersion"].is_string());
}

#[tokio::test]
async fn tools_list_exposes_all_eight_tools() {
    let brain = brain().await;
    let resp = handle_message(
        &brain,
        json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}),
        ToolSurface::Full,
    )
    .await
    .unwrap()
    .unwrap();
    let names: Vec<String> = resp["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|t| t["name"].as_str().map(String::from))
        .collect();
    for expected in [
        "search",
        "cite",
        "remember",
        "ask",
        "who_knows",
        "promote",
        "ontology_get",
        "ontology_promote",
    ] {
        assert!(
            names.iter().any(|n| n == expected),
            "tools/list missing {expected}: {names:?}"
        );
    }
}

// ── search ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn tool_search_returns_capped_views() {
    let brain = brain().await;
    let resp = call(
        &brain,
        "search",
        json!({"query":"KNOWN_PHRASE_KURULTAI_42","limit":3}),
    )
    .await;
    let text = text_of(&resp);
    assert!(
        text.contains("KNOWN_PHRASE_KURULTAI_42") || text.contains("notes"),
        "search result must reference the phrase or source: {text}"
    );
    // Token cap: no 500-char dump.
    assert!(!text.contains(&"x".repeat(500)));
}

// ── cite ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn tool_cite_miss_returns_no_atom_message() {
    let brain = brain().await;
    let resp = call(
        &brain,
        "cite",
        json!({"source":"agent","source_id":"missing"}),
    )
    .await;
    assert!(text_of(&resp).contains("No atom"));
}

#[tokio::test]
async fn tool_cite_hit_returns_citation() {
    let brain = brain().await;
    let hits = brain.search("KNOWN_PHRASE_KURULTAI_42", 1).await.unwrap();
    let atom = &hits[0].atom;
    let resp = call(
        &brain,
        "cite",
        json!({"source":atom.source,"source_id":atom.source_id}),
    )
    .await;
    let text = text_of(&resp);
    assert!(text.contains(&atom.source));
    assert!(!text.contains("No atom"));
}

// ── remember ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn tool_remember_creates_searchable_atom() {
    let brain = brain().await;
    let resp = call(
        &brain,
        "remember",
        json!({
            "title":"MCP Remembered Fact",
            "summary":"remember via tools/call creates a durable agent atom for recall and future retrieval sessions",
            "tags":["mcp","acceptance"]
        }),
    )
    .await;
    let text = text_of(&resp);
    assert!(text.contains("remembered atom id="));
    assert_eq!(resp["result"]["isError"], false);

    // The remembered atom is searchable.
    let hits = brain.search("durable agent atom", 5).await.unwrap();
    assert!(
        hits.iter()
            .any(|h| h.atom.source == "agent" && h.atom.title == "MCP Remembered Fact"),
        "remembered atom must be searchable"
    );
}

// ── ask ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn tool_ask_returns_answer_json() {
    let brain = brain().await;
    let resp = call(
        &brain,
        "ask",
        json!({"question":"KNOWN_PHRASE_KURULTAI_42"}),
    )
    .await;
    let answer: kurultai::types::Answer = serde_json::from_str(text_of(&resp)).unwrap();
    assert!(answer.confidence > 0.0);
    assert!(!answer.citations.is_empty());
}

// ── who_knows ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn tool_who_knows_returns_sources() {
    let brain = brain().await;
    let resp = call(
        &brain,
        "who_knows",
        json!({"topic":"KNOWN_PHRASE_KURULTAI_42","limit":5}),
    )
    .await;
    let text = text_of(&resp);
    assert!(
        text.contains("notes"),
        "who_knows must list notes source: {text}"
    );
}

// ── promote ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn tool_promote_refuses_untagged_quarantine() {
    let brain = brain().await;
    // Inject an untagged quarantine atom.
    use kurultai::quality::gate::{apply_gate, GateOutcome};
    use kurultai::types::KnowledgeAtom;
    let mut atom = KnowledgeAtom::default();
    atom.id = "mcp-q1".into();
    atom.source = "agent".into();
    atom.source_id = "/mcp-q1".into();
    atom.title = "Quarantined".into();
    atom.content = "untagged quarantine content body with operational detail".into();
    atom.tags = vec![];
    apply_gate(
        &mut atom,
        GateOutcome::Quarantine {
            reason: "untagged".into(),
        },
    );
    brain.store().upsert(&atom).await.unwrap();

    let resp = call(&brain, "promote", json!({"atom_id":"mcp-q1"})).await;
    // Must be a JSON-RPC error (read gate refuses untagged).
    assert!(
        resp.get("error").is_some() || resp["result"]["isError"] == true,
        "promote must refuse untagged quarantine: {resp}"
    );
}

#[tokio::test]
async fn tool_promote_succeeds_after_tags_added() {
    let brain = brain().await;
    use kurultai::quality::gate::{apply_gate, GateOutcome};
    use kurultai::types::KnowledgeAtom;
    let mut atom = KnowledgeAtom::default();
    atom.id = "mcp-q2".into();
    atom.source = "agent".into();
    atom.source_id = "/mcp-q2".into();
    atom.title = "Promotable".into();
    atom.content = "promotable content body with sufficient operational detail for the gate to accept on promote after tags added".into();
    atom.tags = vec!["ops".into()];
    apply_gate(
        &mut atom,
        GateOutcome::Quarantine {
            reason: "test".into(),
        },
    );
    brain.store().upsert(&atom).await.unwrap();
    brain
        .store()
        .set_trust_lane("mcp-q2", TrustLane::Quarantine, Some("test"))
        .await
        .unwrap();

    let resp = call(
        &brain,
        "promote",
        json!({"atom_id":"mcp-q2","reason":"acceptance test"}),
    )
    .await;
    let text = text_of(&resp);
    assert!(text.contains("promoted atom id=mcp-q2"));

    let promoted = brain.store().get("mcp-q2").await.unwrap().unwrap();
    assert_eq!(promoted.trust_lane, TrustLane::Trusted);
}

// ── ontology_get ────────────────────────────────────────────────────────────

#[tokio::test]
async fn tool_ontology_get_lists_seeded_classes() {
    let brain = brain().await;
    let resp = call(&brain, "ontology_get", json!({})).await;
    let payload: serde_json::Value = serde_json::from_str(text_of(&resp)).unwrap();
    let entities = payload["entities"].as_array().expect("entities array");
    assert!(entities.len() >= 6, "seeded class tree must have 6 classes");
    let ids: Vec<&str> = entities.iter().filter_map(|e| e["id"].as_str()).collect();
    for class in ["class:memory", "class:note", "class:code", "class:decision"] {
        assert!(ids.contains(&class), "missing {class}: {ids:?}");
    }
    let links = payload["links"].as_array().expect("links array");
    assert!(links.len() >= 5, "seeded is_a links must exist");
}

#[tokio::test]
async fn tool_ontology_get_by_entity_id_returns_entity_and_links() {
    let brain = brain().await;
    let resp = call(&brain, "ontology_get", json!({"entity_id":"class:note"})).await;
    let payload: serde_json::Value = serde_json::from_str(text_of(&resp)).unwrap();
    assert_eq!(payload["entity"]["id"], "class:note");
    assert!(payload["links"].is_array());
}

// ── ontology_promote ────────────────────────────────────────────────────────

#[tokio::test]
async fn tool_ontology_promote_maps_atom_to_instance() {
    let brain = brain().await;
    // Use a fixture atom.
    let hits = brain.search("KNOWN_PHRASE_KURULTAI_42", 1).await.unwrap();
    let atom_id = hits[0].atom.id.clone();

    let resp = call(
        &brain,
        "ontology_promote",
        json!({"atom_id":atom_id,"class_id":"class:note"}),
    )
    .await;
    let entity: serde_json::Value = serde_json::from_str(text_of(&resp)).unwrap();
    assert_eq!(entity["kind"], "instance");
    assert_eq!(entity["atom_id"], atom_id);
    assert!(entity["id"].as_str().unwrap().starts_with("ent:"));

    // The instance_of link now exists.
    let links = brain
        .store()
        .list_ontology_links(Some(entity["id"].as_str().unwrap()))
        .await
        .unwrap();
    assert!(
        links.iter().any(|l| {
            l.from_id == entity["id"].as_str().unwrap()
                && l.to_id == "class:note"
                && l.rel == kurultai::types::OntologyLinkType::InstanceOf
        }),
        "instance_of link must exist"
    );
}

// ── Read-only surface gate (HTTP/SSE) ───────────────────────────────────────

#[tokio::test]
async fn readonly_surface_exposes_only_read_tools() {
    let brain = brain().await;
    let resp = handle_message(
        &brain,
        json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}),
        ToolSurface::ReadOnly,
    )
    .await
    .unwrap()
    .unwrap();
    let mut names: Vec<String> = resp["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|t| t["name"].as_str().map(String::from))
        .collect();
    names.sort();
    assert_eq!(
        names,
        vec!["ask", "cite", "ontology_get", "search", "who_knows"],
        "read-only surface must exclude remember/promote/ontology_promote"
    );
}

#[tokio::test]
async fn readonly_surface_rejects_remember() {
    let brain = brain().await;
    let resp = call_with_surface(
        &brain,
        "remember",
        json!({"title":"x","summary":"y"}),
        ToolSurface::ReadOnly,
    )
    .await;
    let msg = resp["error"]["message"].as_str().unwrap_or("");
    assert!(
        msg.contains("read-only"),
        "remember must be rejected: {resp}"
    );
}

#[tokio::test]
async fn readonly_surface_rejects_ontology_promote() {
    let brain = brain().await;
    let resp = call_with_surface(
        &brain,
        "ontology_promote",
        json!({"atom_id":"x","class_id":"class:note"}),
        ToolSurface::ReadOnly,
    )
    .await;
    let msg = resp["error"]["message"].as_str().unwrap_or("");
    assert!(
        msg.contains("read-only"),
        "ontology_promote must be rejected: {resp}"
    );
}

async fn call_with_surface(
    brain: &BrainService,
    name: &str,
    args: serde_json::Value,
    surface: ToolSurface,
) -> serde_json::Value {
    handle_message(
        brain,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": name, "arguments": args }
        }),
        surface,
    )
    .await
    .unwrap()
    .expect("response")
}

#[tokio::test]
async fn unknown_method_returns_jsonrpc_error_code() {
    let brain = brain().await;
    let resp = handle_message(
        &brain,
        json!({"jsonrpc":"2.0","id":9,"method":"nope"}),
        ToolSurface::Full,
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(resp["error"]["code"], -32601);
}
