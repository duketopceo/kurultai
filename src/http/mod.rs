//! Thin local HTTP API mirroring MCP read tools (Phase 3 / #7).
//!
//! Bind to localhost only — no auth in this slice.

use crate::daemon::DaemonStatus;
use crate::mcp::brain::BrainService;
use crate::mcp::interface::AgentRead;
use crate::synthesize::WhoKnowsEntry;
use crate::types::{Answer, Citation, SearchResult};
use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
struct AppState {
    brain: Arc<BrainService>,
    status: Arc<DaemonStatus>,
}

/// Serve search/ask/cite/who_knows on `127.0.0.1:port` until cancelled.
pub async fn serve(brain: BrainService, status: Arc<DaemonStatus>, port: u16) -> crate::Result<()> {
    let state = AppState {
        brain: Arc::new(brain),
        status,
    };
    let app = router(state);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| crate::KurultaiError::Other(anyhow::anyhow!("bind {addr}: {e}")))?;
    tracing::info!(%addr, "http daemon listening (localhost only)");
    axum::serve(listener, app)
        .await
        .map_err(|e| crate::KurultaiError::Other(anyhow::anyhow!("http serve: {e}")))?;
    Ok(())
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/status", get(api_status))
        .route("/api/search", get(search_get).post(search_post))
        .route("/api/open", get(api_open))
        .route("/ui", get(ui_dashboard))
        .route("/ui/", get(ui_dashboard))
        .route("/search", get(search_get).post(search_post))
        .route("/ask", get(ask_get).post(ask_post))
        .route("/cite", post(cite_post))
        .route("/who_knows", post(who_knows_post))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true, "service": "kurultai" }))
}

async fn api_status(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.brain.atom_count().await {
        Ok(atoms) => Ok(Json(serde_json::json!({
            "ok": true,
            "service": "kurultai",
            "atoms": atoms,
            "scheduler": state.status.snapshot(),
        }))),
        Err(e) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "ok": false,
                "service": "kurultai",
                "atoms": null,
                "error": e.to_string(),
                "scheduler": state.status.snapshot(),
            })),
        )),
    }
}

async fn ui_dashboard() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        Html(DASHBOARD_HTML),
    )
}

const DASHBOARD_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Kurultai — Brain Synapse & Ingested Data View</title>
    <link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Syncopate:wght@400;700&family=Share+Tech+Mono&family=Inter:wght@300;400;500;600;700&display=swap">
    <script src="https://unpkg.com/3d-force-graph"></script>
    <style>
        :root {
            --bg-dark: #000000;
            --bg-card: #080808;
            --border-color: #222222;
            --border-glow: #ffffff;
            --text-primary: #ffffff;
            --text-secondary: #cccccc;
            --text-muted: #888888;
            --primary: #ffffff;
            --secondary: #888888;
            --font-heading: 'Syncopate', sans-serif;
            --font-body: 'Inter', sans-serif;
            --font-mono: 'Share Tech Mono', monospace;
            --transition-smooth: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
        }

        * {
            box-sizing: border-box;
            margin: 0;
            padding: 0;
        }

        body {
            background-color: var(--bg-dark);
            color: var(--text-primary);
            font-family: var(--font-body);
            line-height: 1.6;
            padding: 40px 20px;
        }

        .container {
            max-width: 1400px;
            margin: 0 auto;
        }

        .glass-panel {
            background: var(--bg-card);
            backdrop-filter: blur(16px);
            border: 1px solid var(--border-color);
            border-radius: 12px;
            box-shadow: 0 8px 32px 0 rgba(0, 0, 0, 0.5);
        }

        header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 40px;
            padding-bottom: 20px;
            border-bottom: 1px solid var(--border-color);
        }

        h1, h2, h3, h4 {
            font-family: var(--font-heading);
            font-weight: 700;
            letter-spacing: -0.02em;
        }

        .logo {
            font-size: 1.5rem;
            color: var(--primary);
            text-decoration: none;
            display: flex;
            align-items: center;
            gap: 10px;
        }

        .badge {
            display: inline-block;
            padding: 6px 14px;
            border-radius: 9999px;
            background: rgba(255, 255, 255, 0.08);
            border: 1px solid rgba(255, 255, 255, 0.15);
            color: var(--primary);
            font-family: var(--font-mono);
            font-size: 0.8rem;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            margin-bottom: 12px;
        }

        .stats-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
            gap: 24px;
            margin-bottom: 40px;
        }

        .stat-card {
            padding: 24px;
            text-align: center;
        }

        .stat-val {
            font-size: 2.2rem;
            font-weight: 800;
            color: var(--primary);
            font-family: var(--font-mono);
            margin-top: 8px;
        }

        .db-layout {
            display: grid;
            grid-template-columns: 1fr 2fr;
            gap: 32px;
            align-items: start;
        }

        .atoms-list {
            display: flex;
            flex-direction: column;
            gap: 16px;
            max-height: 600px;
            overflow-y: auto;
            padding-right: 8px;
        }

        .atom-item {
            padding: 16px 20px;
            cursor: pointer;
            border-radius: 12px;
            background: rgba(255, 255, 255, 0.02);
            border: 1px solid var(--border-color);
            transition: var(--transition-smooth);
        }

        .atom-item:hover, .atom-item.active {
            border-color: var(--primary);
            background: rgba(255, 255, 255, 0.05);
        }

        .atom-item h4 {
            font-size: 1.05rem;
            margin-bottom: 6px;
        }

        .atom-meta {
            font-size: 0.8rem;
            color: var(--text-muted);
            display: flex;
            justify-content: space-between;
            font-family: var(--font-mono);
        }

        .atom-details {
            min-height: 500px;
            padding: 32px;
        }

        .detail-header {
            display: flex;
            justify-content: space-between;
            align-items: start;
            border-bottom: 1px solid var(--border-color);
            padding-bottom: 20px;
            margin-bottom: 24px;
        }

        .detail-title {
            font-size: 1.6rem;
            font-weight: 700;
        }

        .detail-row {
            margin-bottom: 20px;
        }

        .detail-label {
            font-size: 0.8rem;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            color: var(--text-muted);
            margin-bottom: 6px;
            font-weight: 600;
            font-family: var(--font-mono);
        }

        .detail-val {
            font-size: 0.95rem;
            color: var(--text-secondary);
            background: rgba(0, 0, 0, 0.4);
            padding: 12px 16px;
            border-radius: 8px;
            border: 1px solid var(--border-color);
            white-space: pre-wrap;
        }

        .tag-pill {
            display: inline-block;
            padding: 4px 12px;
            border-radius: 9999px;
            background: rgba(255, 255, 255, 0.08);
            border: 1px solid rgba(255, 255, 255, 0.15);
            color: var(--text-secondary);
            font-size: 0.8rem;
            margin-right: 6px;
            font-family: var(--font-mono);
        }

        .search-box {
            display: flex;
            gap: 12px;
            margin-bottom: 24px;
        }

        .search-input {
            flex: 1;
            background: rgba(17, 24, 39, 0.8);
            border: 1px solid var(--border-color);
            padding: 14px 28px;
            border-radius: 9999px;
            color: var(--text-primary);
            font-family: var(--font-body);
            font-size: 1rem;
            outline: none;
            transition: var(--transition-smooth);
        }

        .search-input:focus {
            border-color: var(--primary);
            box-shadow: 0 0 15px rgba(255, 255, 255, 0.15);
        }

        .vector-space-section {
            margin-top: 80px;
            margin-bottom: 40px;
        }

        #3d-synapse-graph {
            width: 100%;
            height: 500px;
            border-radius: 12px;
            border: 1px solid var(--border-color);
            overflow: hidden;
            background-color: #030712;
        }

        @media (max-width: 992px) {
            .db-layout {
                grid-template-columns: 1fr;
            }
        }
    </style>
</head>
<body>

    <div class="container">
        <!-- Header -->
        <header>
            <a href="#" class="logo">
                <svg height="24" width="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" style="color: var(--primary);"><circle cx="12" cy="12" r="3"/><circle cx="19" cy="5" r="2"/><circle cx="5" cy="19" r="2"/><circle cx="19" cy="19" r="2"/><circle cx="5" cy="5" r="2"/><line x1="12" y1="12" x2="19" y2="5"/><line x1="12" y1="12" x2="5" y2="19"/><line x1="12" y1="12" x2="19" y2="19"/><line x1="12" y1="12" x2="5" y2="5"/></svg>
                kurultai
            </a>
            <span class="badge">Local Daemon UI</span>
        </header>

        <!-- Mode Toggle -->
        <div style="display: flex; justify-content: flex-end; align-items: center; margin-bottom: 24px; font-family: var(--font-mono); font-size: 0.85rem; gap: 12px;">
            <span style="color: var(--text-muted);">View Mode:</span>
            <button id="view-mode-btn" class="glass-panel" style="padding: 8px 16px; border-radius: 9999px; border: 1px solid var(--border-color); color: var(--text-primary); cursor: pointer; font-family: var(--font-mono); background: rgba(255,255,255,0.04); transition: var(--transition-smooth);">Executive View</button>
        </div>

        <!-- Stats -->
        <section class="stats-grid">
            <div class="stat-card glass-panel">
                <div class="detail-label">Daemon Status</div>
                <div id="stat-status" class="stat-val" style="font-size: 1.5rem; margin-top: 16px;">Online</div>
            </div>
            <div class="stat-card glass-panel">
                <div id="label-atoms" class="detail-label">Stored Memories</div>
                <div id="stat-atoms" class="stat-val">0</div>
            </div>
            <div class="stat-card glass-panel">
                <div id="label-sources" class="detail-label">Data Sources</div>
                <div id="stat-sources" class="stat-val">0</div>
            </div>
            <div class="stat-card glass-panel">
                <div id="label-env" class="detail-label">Server Mode</div>
                <div id="stat-env" class="stat-val" style="font-size: 1.5rem; margin-top: 16px; text-transform: uppercase;">dev</div>
            </div>
        </section>

        <!-- Search / Browser -->
        <section style="display: flex; flex-direction: column; gap: 16px; margin-bottom: 24px;">
            <div class="search-box" style="margin-bottom: 0;">
                <input type="text" id="brain-search" class="search-input" placeholder="Query the local brain (FTS + Vector search)...">
            </div>
            <div class="glass-panel" style="display: flex; align-items: center; justify-content: space-between; padding: 12px 24px; border-radius: 9999px; font-family: var(--font-mono); font-size: 0.85rem;">
                <span style="color: var(--text-muted);">Connection Threshold</span>
                <input type="range" id="threshold-slider" min="0.0" max="1.0" step="0.05" value="0.0" style="flex: 1; margin: 0 20px; accent-color: var(--primary); cursor: pointer;">
                <span id="threshold-val" style="color: var(--primary); min-width: 32px; text-align: right;">0.00</span>
            </div>
        </section>

        <section class="db-layout">
            <!-- Left Pane: Atoms -->
            <div class="atoms-list" id="atoms-list-container">
                <div style="text-align: center; color: var(--text-muted); padding: 20px;">Fetching local store...</div>
            </div>

            <!-- Right Pane: Inspector -->
            <div class="atom-details glass-panel" id="atom-inspector">
                <div style="text-align: center; color: var(--text-muted); padding-top: 100px;">
                    <p>Select an atom to inspect its vector-spaced contents</p>
                </div>
            </div>
        </section>

        <!-- 3D Synapse Graph -->
        <section class="vector-space-section">
            <div style="margin-bottom: 24px;">
                <h2>3D Synaptic Network Map</h2>
                <p style="color: var(--text-muted); font-size: 0.95rem; margin-top: 4px;">Dynamic force-directed WebGL space representing semantic vectors and code relations.</p>
            </div>
            <div id="3d-synapse-graph"></div>
        </section>
    </div>

    <script>
        document.addEventListener("DOMContentLoaded", () => {
            const listContainer = document.getElementById("atoms-list-container");
            const inspector = document.getElementById("atom-inspector");
            const searchInput = document.getElementById("brain-search");
            const graphContainer = document.getElementById("3d-synapse-graph");
            const slider = document.getElementById("threshold-slider");
            const sliderVal = document.getElementById("threshold-val");
            const viewModeBtn = document.getElementById("view-mode-btn");

            let currentAtoms = [];
            let activeAtom = null;
            let currentThreshold = 0.0;
            let isTechnical = false;

            viewModeBtn.addEventListener("click", () => {
                isTechnical = !isTechnical;
                viewModeBtn.textContent = isTechnical ? "Technical View" : "Executive View";
                viewModeBtn.style.borderColor = isTechnical ? "var(--primary)" : "var(--border-color)";
                viewModeBtn.style.background = isTechnical ? "rgba(255,255,255,0.12)" : "rgba(255,255,255,0.04)";
                updateDashboardMode();
            });

            function updateDashboardMode() {
                document.getElementById("label-atoms").textContent = isTechnical ? "Indexed Atoms" : "Stored Memories";
                document.getElementById("label-sources").textContent = isTechnical ? "Active Sources" : "Data Sources";
                document.getElementById("label-env").textContent = isTechnical ? "Environment" : "Server Mode";
                renderAtomsList(currentAtoms);
                inspectAtom(activeAtom);
            }

            async function loadDashboard() {
                // 1. Fetch Status Info
                try {
                    const statusRes = await fetch("/api/status");
                    const statusData = await statusRes.json();
                    document.getElementById("stat-atoms").textContent = statusData.atoms || 0;
                    document.getElementById("stat-env").textContent = statusData.scheduler?.env || "dev";
                    
                    const sourcesCount = statusData.scheduler?.last_sync_duration_ms !== undefined ? 1 : 0;
                    document.getElementById("stat-sources").textContent = sourcesCount;
                } catch (e) {
                    console.error("Failed to load status details:", e);
                }

                // 2. Fetch Initial Search (all atoms)
                await triggerSearch("");
            }

            async function triggerSearch(query) {
                try {
                    const searchRes = await fetch("/api/search?q=" + encodeURIComponent(query) + "&limit=25");
                    const results = await searchRes.json();
                    
                    currentAtoms = results.map(r => ({
                        id: r.atom.id || r.title_hash || Math.random().toString(36).substr(2, 9),
                        title: r.atom.title || r.title,
                        source: r.atom.source || r.source,
                        source_id: r.atom.source_id || r.source_id,
                        summary: r.atom.summary || r.excerpt,
                        content: r.atom.content || r.excerpt,
                        question: r.atom.question || "",
                        resolution: r.atom.resolution || "",
                        tags: r.atom.tags || [],
                        source_updated_at: r.atom.source_updated_at || "",
                        file_path: r.file_path || r.atom.file_path || r.atom.metadata?.file_path || "",
                        score: r.score
                    }));

                    renderAtomsList(currentAtoms);
                    if (currentAtoms.length > 0 && !activeAtom) {
                        activeAtom = currentAtoms[0];
                        inspectAtom(activeAtom);
                    }
                    update3DGraph(currentAtoms);
                } catch (e) {
                    console.error("Failed search query execution:", e);
                    listContainer.innerHTML = `<div style="text-align: center; color: var(--text-muted); padding: 20px;">Failed to fetch active atoms from server.</div>`;
                }
            }

            function renderAtomsList(atoms) {
                listContainer.innerHTML = "";
                if (atoms.length === 0) {
                    listContainer.innerHTML = `<div style="text-align: center; color: var(--text-muted); padding: 20px;">No matching records found.</div>`;
                    return;
                }

                atoms.forEach(atom => {
                    const div = document.createElement("div");
                    div.className = `atom-item ${activeAtom && activeAtom.id === atom.id ? "active" : ""}`;
                    div.innerHTML = `
                        <h4>${escapeHtml(atom.title)}</h4>
                        <div class="atom-meta">
                            <span>${escapeHtml(atom.source)}/${escapeHtml(atom.source_id)}</span>
                            <span>Score: ${atom.score !== undefined ? atom.score.toFixed(3) : "N/A"}</span>
                        </div>
                    `;
                    div.addEventListener("click", () => {
                        activeAtom = atom;
                        document.querySelectorAll(".atom-item").forEach(el => el.classList.remove("active"));
                        div.classList.add("active");
                        inspectAtom(atom);
                    });
                    listContainer.appendChild(div);
                });
            }

            function inspectAtom(atom) {
                if (!atom) return;
                const tagPills = atom.tags.map(t => `<span class="tag-pill">${escapeHtml(t)}</span>`).join("");
                
                inspector.innerHTML = `
                    <div class="detail-header">
                        <div>
                            <h3 class="detail-title">${escapeHtml(atom.title)}</h3>
                            <div style="margin-top: 8px;">${tagPills}</div>
                        </div>
                        ${isTechnical ? `<div class="detail-label" style="text-align: right;">ID: ${escapeHtml(atom.id)}</div>` : ""}
                    </div>
                    
                    <div class="detail-layout">
                        <div class="detail-row">
                            <div class="detail-label">${isTechnical ? "Source Context" : "Memory Origin"}</div>
                            <div class="detail-val" style="font-family: var(--font-mono); display: flex; justify-content: space-between; align-items: center;">
                                <span>${escapeHtml(atom.source)} / ${escapeHtml(atom.source_id)}</span>
                                ${isTechnical && atom.file_path ? `<button onclick="openFileInEditor('${escapeHtml(atom.file_path)}')" style="padding: 4px 12px; font-size: 0.75rem; border-radius: 9999px; background: rgba(255,255,255,0.08); border: 1px solid var(--border-color); color: var(--text-primary); cursor: pointer; font-family: var(--font-mono);">Open File</button>` : ""}
                            </div>
                        </div>
                        <div class="detail-row">
                            <div class="detail-label">Excerpt / Content</div>
                            <div class="detail-val">${escapeHtml(atom.content)}</div>
                        </div>
                        <div class="detail-row">
                            <div class="detail-label">Summary</div>
                            <div class="detail-val">${escapeHtml(atom.summary)}</div>
                        </div>
                        ${isTechnical && atom.question ? `
                        <div class="detail-row">
                            <div class="detail-label">Routing Query Mapping</div>
                            <div class="detail-val"><strong>Q:</strong> ${escapeHtml(atom.question)}<br/><strong>A:</strong> ${escapeHtml(atom.resolution)}</div>
                        </div>` : ""}
                    </div>
                `;
            }

            window.openFileInEditor = async function(filePath) {
                try {
                    await fetch("/api/open?file=" + encodeURIComponent(filePath));
                } catch (e) {
                    console.error("Failed to trigger file open API:", e);
                }
            };

            function escapeHtml(text) {
                if (!text) return "";
                return text
                    .replace(/&/g, "&amp;")
                    .replace(/</g, "&lt;")
                    .replace(/>/g, "&gt;")
                    .replace(/"/g, "&quot;")
                    .replace(/'/g, "&#039;");
            }

            // Real-Time Search Handler
            let searchTimeout = null;
            searchInput.addEventListener("input", (e) => {
                clearTimeout(searchTimeout);
                searchTimeout = setTimeout(() => {
                    triggerSearch(e.target.value.trim());
                }, 300);
            });

            // Threshold Range Change Listener
            slider.addEventListener("input", (e) => {
                currentThreshold = parseFloat(e.target.value);
                sliderVal.textContent = currentThreshold.toFixed(2);
                update3DGraph(currentAtoms);
            });

            // 3D Graph Instance Handler
            let Graph = null;
            function update3DGraph(atoms) {
                if (!graphContainer || typeof ForceGraph3D === 'undefined') return;

                const nodes = atoms.map(atom => ({
                    id: atom.id,
                    title: atom.title,
                    source: atom.source,
                    source_id: atom.source_id,
                    tags: atom.tags,
                    color: atom.source_id.includes('guidelines') ? '#ffffff' : (atom.source_id.includes('migration') ? '#888888' : '#333333'),
                    val: 5
                }));

                const links = [];
                for (let i = 0; i < nodes.length; i++) {
                    for (let j = i + 1; j < nodes.length; j++) {
                        const sharesSource = nodes[i].source_id === nodes[j].source_id;
                        const sharesTag = nodes[i].tags.some(t => nodes[j].tags.includes(t));
                        
                        let score = 0.0;
                        if (sharesSource && sharesTag) score = 1.0;
                        else if (sharesSource) score = 0.8;
                        else if (sharesTag) score = 0.5;

                        if (score >= currentThreshold) {
                            links.push({
                                source: nodes[i].id,
                                target: nodes[j].id,
                                score: score
                            });
                        }
                    }
                }

                if (!Graph) {
                    Graph = ForceGraph3D()(graphContainer)
                        .graphData({ nodes, links })
                        .backgroundColor('#000000')
                        .nodeColor(node => node.color)
                        .nodeLabel(node => `
                            <div style="background: rgba(17, 24, 39, 0.9); border: 1px solid var(--border-color); border-radius: 8px; padding: 12px; font-family: var(--font-mono); font-size: 0.85rem; color: var(--text-primary);">
                                <strong style="color: var(--primary); font-size: 0.9rem;">${node.title}</strong><br/>
                                <span style="color: var(--text-muted);">Source: ${node.source}/${node.source_id}</span><br/>
                                <span style="color: var(--secondary);">Tags: ${node.tags.join(', ')}</span>
                            </div>
                        `)
                        .nodeRelSize(3)
                        .linkColor(() => 'rgba(255, 255, 255, 0.04)')
                        .linkWidth(0.5)
                        .linkDirectionalParticles(2)
                        .linkDirectionalParticleSpeed(0.005)
                        .linkDirectionalParticleWidth(1.2)
                        .linkDirectionalParticleColor(() => '#ffffff')
                        .onNodeClick(node => {
                            const atom = currentAtoms.find(a => a.id === node.id);
                            if (atom) {
                                activeAtom = atom;
                                renderAtomsList(currentAtoms);
                                inspectAtom(atom);
                            }
                        });
                        
                    Graph.width(graphContainer.clientWidth);
                    Graph.height(500);
                    window.addEventListener("resize", () => Graph.width(graphContainer.clientWidth));
                } else {
                    Graph.graphData({ nodes, links });
                }
            }

            // Boot Dashboard
            loadDashboard();
        });
    </script>
</body>
</html>
"##;

fn default_limit() -> usize {
    10
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: String,
    #[serde(default = "default_limit")]
    limit: usize,
}

#[derive(Debug, Deserialize)]
struct AskQuery {
    question: String,
}

#[derive(Debug, Deserialize)]
struct SearchBody {
    query: String,
    #[serde(default = "default_limit")]
    limit: usize,
}

async fn search_post(
    State(state): State<AppState>,
    Json(body): Json<SearchBody>,
) -> Result<Json<Vec<SearchResult>>, (StatusCode, String)> {
    state.status.touch_client_activity();
    state
        .brain
        .search(&body.query, body.limit)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn search_get(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<SearchResult>>, (StatusCode, String)> {
    state.status.touch_client_activity();
    state
        .brain
        .search(&query.q, query.limit)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[derive(Debug, Deserialize)]
struct AskBody {
    question: String,
}

async fn ask_post(
    State(state): State<AppState>,
    Json(body): Json<AskBody>,
) -> Result<Json<Answer>, (StatusCode, String)> {
    state.status.touch_client_activity();
    state
        .brain
        .ask(&body.question)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn ask_get(
    State(state): State<AppState>,
    Query(query): Query<AskQuery>,
) -> Result<Json<Answer>, (StatusCode, String)> {
    state.status.touch_client_activity();
    state
        .brain
        .ask(&query.question)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[derive(Debug, Deserialize)]
struct CiteBody {
    source: String,
    source_id: String,
}

async fn cite_post(
    State(state): State<AppState>,
    Json(body): Json<CiteBody>,
) -> Result<Json<Option<Citation>>, (StatusCode, String)> {
    state
        .brain
        .cite(&body.source, &body.source_id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[derive(Debug, Deserialize)]
struct WhoKnowsBody {
    topic: String,
    #[serde(default = "default_limit")]
    limit: usize,
}

async fn who_knows_post(
    State(state): State<AppState>,
    Json(body): Json<WhoKnowsBody>,
) -> Result<Json<Vec<WhoKnowsEntry>>, (StatusCode, String)> {
    state
        .brain
        .who_knows(&body.topic, body.limit)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn api_open(
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(file) = params.get("file") {
        let path = std::path::Path::new(file);
        if path.exists() {
            let _ = std::process::Command::new("open").arg(path).status();
        }
    }
    StatusCode::OK
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connectors::markdown::MarkdownConnector;
    use crate::connectors::Connector;
    use crate::embed::{Embedder, NullEmbedder};
    use crate::pipeline::IndexPipeline;
    use crate::rerank::NullReranker;
    use crate::store::{SqliteVecStore, Store};
    use crate::synthesize::ExtractiveSynthesizer;
    use crate::synthesize::Synthesizer;
    use crate::types::{SourceConfig, SourceKind};
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use tower::ServiceExt;

    static HTTP_FIXTURE_SEQ: AtomicU64 = AtomicU64::new(1);

    fn test_brain() -> BrainService {
        let dir = std::env::temp_dir().join(format!(
            "kurultai-http-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let store = Arc::new(SqliteVecStore::open(dir.join("store.db"), 4).unwrap());
        let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
        let synth: Arc<dyn Synthesizer> = Arc::new(ExtractiveSynthesizer::new());
        BrainService::new(store, embedder, Arc::new(NullReranker::new()), synth)
    }

    #[tokio::test]
    async fn health_ok() {
        let app = router(AppState {
            brain: Arc::new(test_brain()),
            status: Arc::new(crate::daemon::DaemonStatus::default()),
        });
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn ask_empty_store_json() {
        let app = router(AppState {
            brain: Arc::new(test_brain()),
            status: Arc::new(crate::daemon::DaemonStatus::default()),
        });
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/ask")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"question":"anything?"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let answer: Answer = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(answer.confidence, 0.0);
        assert!(answer.citations.is_empty());
    }

    async fn fixture_brain_app() -> (Router, tempfile::TempDir) {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vault");
        let db_dir = tempfile::tempdir().unwrap();
        let store = Arc::new(SqliteVecStore::open(db_dir.path().join("store.db"), 4).unwrap());
        let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
        let pipeline =
            IndexPipeline::new(Arc::clone(&store) as Arc<dyn Store>, Arc::clone(&embedder));

        let mut connector = MarkdownConnector::new();
        let mut extra = HashMap::new();
        extra.insert("root_path".into(), fixture.to_string_lossy().into_owned());
        let source_name = format!(
            "notes-http-{}",
            HTTP_FIXTURE_SEQ.fetch_add(1, Ordering::Relaxed)
        );
        connector
            .init(&SourceConfig {
                name: source_name.clone(),
                kind: SourceKind::Markdown,
                enabled: true,
                poll_interval_secs: 60,
                extra,
            })
            .await
            .unwrap();
        pipeline
            .index_connector(&source_name, &connector, true)
            .await
            .unwrap();

        let brain = BrainService::new(
            store,
            embedder,
            Arc::new(NullReranker::new()),
            Arc::new(ExtractiveSynthesizer::new()),
        );
        let app = router(AppState {
            brain: Arc::new(brain),
            status: Arc::new(crate::daemon::DaemonStatus::default()),
        });
        (app, db_dir)
    }

    #[tokio::test]
    async fn fixture_vault_search_ask_who_knows() {
        let (app, _db_dir) = fixture_brain_app().await;

        // POST /search
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/search")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"query":"KNOWN_PHRASE_KURULTAI_42","limit":5}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let results: Vec<SearchResult> = serde_json::from_slice(&bytes).unwrap();
        assert!(!results.is_empty());

        // GET /search
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/search?q=KNOWN_PHRASE_KURULTAI_42&limit=5")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let results: Vec<SearchResult> = serde_json::from_slice(&bytes).unwrap();
        assert!(!results.is_empty());

        // POST /ask
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/ask")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"question":"what is KNOWN_PHRASE_KURULTAI_42"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let answer: Answer = serde_json::from_slice(&bytes).unwrap();
        assert!(answer.confidence > 0.0);
        assert!(!answer.citations.is_empty());

        // GET /ask
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/ask?question=what%20is%20KNOWN_PHRASE_KURULTAI_42")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let answer: Answer = serde_json::from_slice(&bytes).unwrap();
        assert!(answer.confidence > 0.0);
        assert!(!answer.citations.is_empty());

        // POST /who_knows
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/who_knows")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"topic":"KNOWN_PHRASE_KURULTAI_42","limit":10}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let entries: Vec<WhoKnowsEntry> = serde_json::from_slice(&bytes).unwrap();
        assert!(!entries.is_empty());
    }

    /// Store stub: only `count` matters (always Err) for `/api/status` failure path.
    struct FailCountStore;

    #[async_trait::async_trait]
    impl Store for FailCountStore {
        async fn upsert(&self, _atom: &crate::types::KnowledgeAtom) -> crate::Result<()> {
            Ok(())
        }
        async fn upsert_batch(&self, _atoms: &[crate::types::KnowledgeAtom]) -> crate::Result<()> {
            Ok(())
        }
        async fn vector_search(
            &self,
            _query_embed: &[f32],
            _limit: usize,
        ) -> crate::Result<Vec<(crate::types::KnowledgeAtom, f64)>> {
            Ok(vec![])
        }
        async fn fts_search(
            &self,
            _query: &str,
            _limit: usize,
        ) -> crate::Result<Vec<(crate::types::KnowledgeAtom, f64)>> {
            Ok(vec![])
        }
        async fn fts_search_ids(
            &self,
            _query: &str,
            _limit: usize,
        ) -> crate::Result<Vec<(String, f64)>> {
            Ok(vec![])
        }
        async fn vector_search_ids(
            &self,
            _query_embed: &[f32],
            _limit: usize,
        ) -> crate::Result<Vec<(String, f64)>> {
            Ok(vec![])
        }
        async fn get_many(
            &self,
            _ids: &[String],
        ) -> crate::Result<Vec<crate::types::KnowledgeAtom>> {
            Ok(vec![])
        }
        async fn delete_source(&self, _source: &str) -> crate::Result<()> {
            Ok(())
        }
        async fn count(&self) -> crate::Result<u64> {
            Err(crate::KurultaiError::Store("count failed".into()))
        }
        async fn get_by_source_id(
            &self,
            _source: &str,
            _source_id: &str,
        ) -> crate::Result<Option<crate::types::KnowledgeAtom>> {
            Ok(None)
        }
        async fn get_by_chunk_meta(
            &self,
            _source: &str,
            _rel_path: &str,
            _chunk_index: u32,
        ) -> crate::Result<Option<crate::types::KnowledgeAtom>> {
            Ok(None)
        }
        async fn has_fresh_embedding(&self, _id: &str, _content_hash: &str) -> crate::Result<bool> {
            Ok(false)
        }
    }

    #[tokio::test]
    async fn api_status_ok_includes_scheduler() {
        let status = Arc::new(crate::daemon::DaemonStatus::default());
        let app = router(AppState {
            brain: Arc::new(test_brain()),
            status: Arc::clone(&status),
        });
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["ok"], true);
        assert!(v["atoms"].is_number());
        assert!(v["scheduler"]["last_client_activity_unix"].is_number());
    }

    #[tokio::test]
    async fn api_status_store_failure_is_503() {
        let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
        let synth: Arc<dyn Synthesizer> = Arc::new(ExtractiveSynthesizer::new());
        let brain = BrainService::new(
            Arc::new(FailCountStore),
            embedder,
            Arc::new(NullReranker::new()),
            synth,
        );
        let app = router(AppState {
            brain: Arc::new(brain),
            status: Arc::new(crate::daemon::DaemonStatus::default()),
        });
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["ok"], false);
        assert!(v["atoms"].is_null());
        assert!(v["error"].as_str().unwrap_or("").contains("count failed"));
    }

    #[tokio::test]
    async fn search_and_ask_refresh_client_activity() {
        let status = Arc::new(crate::daemon::DaemonStatus::default());
        assert_eq!(
            status
                .last_client_activity_unix
                .load(std::sync::atomic::Ordering::Relaxed),
            0
        );
        let app = router(AppState {
            brain: Arc::new(test_brain()),
            status: Arc::clone(&status),
        });

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/search?q=hello&limit=1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let after_get = status
            .last_client_activity_unix
            .load(std::sync::atomic::Ordering::Relaxed);
        assert!(after_get > 0);

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/search")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"query":"hello","limit":1}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let after_post = status
            .last_client_activity_unix
            .load(std::sync::atomic::Ordering::Relaxed);
        assert!(after_post > 0);

        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/ask")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"question":"anything?"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(
            status
                .last_client_activity_unix
                .load(std::sync::atomic::Ordering::Relaxed)
                >= after_post
        );
    }
}
