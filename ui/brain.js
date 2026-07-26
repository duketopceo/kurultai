// brain.js — wired to the local kurultai daemon (served at GET /ui).
// Absolute /api/* paths hit the same origin when loaded from the daemon.
document.addEventListener("DOMContentLoaded", () => {
    const listContainer = document.getElementById("atoms-list-container");
    const inspector = document.getElementById("atom-inspector");
    const searchInput = document.getElementById("brain-search");
    const graphContainer = document.getElementById("3d-synapse-graph");

    let currentAtoms = [];
    let activeAtom = null;
    let Graph = null;

    // View mode: Simple (default) vs Technical
    const viewModeBtn = document.getElementById("view-mode-btn");
    let isTechnical = localStorage.getItem("kurultai-brain-view") === "technical";

    function applyViewMode() {
        if (!viewModeBtn) return;
        viewModeBtn.textContent = isTechnical ? "Simple View" : "Technical View";
        viewModeBtn.classList.toggle("technical", isTechnical);
        viewModeBtn.setAttribute("aria-pressed", String(isTechnical));
        const lAtoms = document.getElementById("label-atoms");
        const lSources = document.getElementById("label-sources");
        const lEnv = document.getElementById("label-env");
        if (lAtoms) lAtoms.textContent = isTechnical ? "Indexed Atoms" : "Stored Memories";
        if (lSources) lSources.textContent = isTechnical ? "Active Sources" : "Data Sources";
        if (lEnv) lEnv.textContent = isTechnical ? "Environment" : "Server Mode";
        if (activeAtom) inspectAtom(activeAtom);
    }

    if (viewModeBtn) {
        viewModeBtn.addEventListener("click", () => {
            isTechnical = !isTechnical;
            localStorage.setItem("kurultai-brain-view", isTechnical ? "technical" : "simple");
            applyViewMode();
        });
    }

    // 1. Boot: pull status + initial atom list from the daemon in parallel
    async function loadDashboard() {
        const statusPromise = fetch("/api/status")
            .then(async r => {
                const j = await r.json();
                document.getElementById("stat-status").textContent = j.ok ? "Online" : "Offline";
                document.getElementById("stat-atoms").textContent = j.atoms ?? "—";
                document.getElementById("stat-env").textContent =
                    (j.scheduler && j.scheduler.env) || "dev";
            })
            .catch(e => {
                document.getElementById("stat-status").textContent = "Daemon unreachable";
                console.error("status fetch failed:", e);
            });
        await Promise.all([statusPromise, triggerLoadAtoms()]);
    }

    // 2. Fetch atoms (no query = list all) or run a real FTS/vector search
    async function triggerLoadAtoms() {
        try {
            const r = await fetch("/api/atoms?limit=200");
            const results = await r.json();
            processResults(results);
        } catch (e) {
            console.error("atoms fetch failed:", e);
            listContainer.innerHTML = `<div style="text-align: center; color: var(--text-muted); padding: 20px;">Could not reach the local daemon at /api/atoms.</div>`;
        }
    }

    async function triggerSearch(query) {
        if (!query) return triggerLoadAtoms();
        try {
            const r = await fetch("/api/search?q=" + encodeURIComponent(query) + "&limit=25");
            const results = await r.json();
            processResults(results);
        } catch (e) {
            console.error("search fetch failed:", e);
            listContainer.innerHTML = `<div style="text-align: center; color: var(--text-muted); padding: 20px;">Search request failed.</div>`;
        }
    }

    // 3. Normalize daemon SearchResult -> atom shape the UI expects
    function processResults(results) {
        currentAtoms = (results || []).map(r => {
            const a = r.atom || r;
            const meta = a.metadata || {};
            return {
                id: a.id || Math.random().toString(36).slice(2, 11),
                title: a.title || "(untitled)",
                source: a.source || "",
                source_id: a.source_id || "",
                summary: a.summary || "",
                content: a.content || "",
                question: a.question || "",
                resolution: a.resolution || "",
                tags: a.tags || [],
                file_path: meta.file_path || meta.rel_path || null,
                source_updated_at: a.source_updated_at || "",
                score: r.score
            };
        });

        const sources = new Set(currentAtoms.map(a => a.source).filter(Boolean));
        const sourcesEl = document.getElementById("stat-sources");
        if (sourcesEl) sourcesEl.textContent = sources.size || "—";

        renderAtomsList(currentAtoms);
        if (currentAtoms.length > 0 && !activeAtom) {
            activeAtom = currentAtoms[0];
            inspectAtom(activeAtom);
        }
        update3DGraph(currentAtoms);
    }

    // 4. Render the left-pane atom list
    function renderAtomsList(atoms) {
        listContainer.innerHTML = "";
        if (!atoms || atoms.length === 0) {
            listContainer.innerHTML = `<div style="text-align: center; color: var(--text-muted); padding: 20px;">No atoms in the local store.</div>`;
            return;
        }

        atoms.forEach(atom => {
            const div = document.createElement("div");
            div.className = `atom-item ${activeAtom && activeAtom.id === atom.id ? "active" : ""}`;
            div.innerHTML = `
                <h4>${escapeHtml(atom.title)}</h4>
                <div class="atom-meta">
                    <span>${escapeHtml(atom.source)}/${escapeHtml(atom.source_id)}</span>
                    <span>${atom.score !== undefined ? atom.score.toFixed(3) : atom.id.slice(0, 8)}</span>
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

    // 5. Inspector pane
    function inspectAtom(atom) {
        if (!atom) {
            inspector.innerHTML = `<div style="text-align: center; color: var(--text-muted); padding-top: 100px;"><p>Select an atom to inspect its database structure</p></div>`;
            return;
        }

        const tagPills = (atom.tags || []).map(t => `<span class="tag-pill">${escapeHtml(t)}</span>`).join("");
        const sourceLabel = isTechnical ? "Source Context" : "Memory Origin";
        const contentLabel = isTechnical ? "Raw Database Content (content)" : "Excerpt / Content";
        const summaryLabel = isTechnical ? "LLM-Distilled Summary (summary)" : "Summary";
        const updatedSuffix = isTechnical && atom.source_updated_at ? ` (updated: ${escapeHtml(atom.source_updated_at)})` : "";
        const idHeader = isTechnical ? `<div class="detail-label" style="text-align: right;">ID: ${escapeHtml(atom.id)}</div>` : "";
        const openFileBtn = isTechnical && atom.file_path
            ? `<button onclick="openFileInEditor('${escapeHtml(atom.file_path)}')" style="padding: 4px 12px; font-size: 0.75rem; border-radius: 9999px; background: rgba(255,255,255,0.08); border: 1px solid rgba(255,255,255,0.35); color: #ffffff; cursor: pointer; font-family: var(--font-mono);">Open File</button>`
            : "";
        const routingRow = isTechnical && atom.question ? `
            <div class="detail-row">
                <div class="detail-label">Routing Queries (question / resolution)</div>
                <div class="detail-val"><strong>Q:</strong> ${escapeHtml(atom.question)}<br/><strong>A:</strong> ${escapeHtml(atom.resolution || "")}</div>
            </div>` : "";

        inspector.innerHTML = `
            <div class="detail-header">
                <div>
                    <h3 class="detail-title">${escapeHtml(atom.title)}</h3>
                    <div style="margin-top: 8px;">${tagPills}</div>
                </div>
                ${idHeader}
            </div>

            <div class="detail-layout">
                <div class="detail-row">
                    <div class="detail-label">${sourceLabel}</div>
                    <div class="detail-val" style="font-family: var(--font-mono); display: flex; justify-content: space-between; align-items: center; gap: 12px;">
                        <span>${escapeHtml(atom.source)} / ${escapeHtml(atom.source_id)}${updatedSuffix}</span>
                        ${openFileBtn}
                    </div>
                </div>
                <div class="detail-row">
                    <div class="detail-label">${contentLabel}</div>
                    <div class="detail-val">${escapeHtml(atom.content)}</div>
                </div>
                <div class="detail-row">
                    <div class="detail-label">${summaryLabel}</div>
                    <div class="detail-val">${escapeHtml(atom.summary)}</div>
                </div>
                ${routingRow}
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
        return String(text)
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/"/g, "&quot;")
            .replace(/'/g, "&#039;");
    }

    // 6. Search input — debounced, hits /api/search on the daemon
    let searchTimeout = null;
    if (searchInput) {
        searchInput.addEventListener("input", (e) => {
            clearTimeout(searchTimeout);
            const q = e.target.value.trim();
            searchTimeout = setTimeout(() => triggerSearch(q), 300);
        });
    }

    // 7. 3D synapse graph — white/electric styling, hover focus, simulated activity
    let baseLinks = [];
    let adjacency = new Map(); // id -> Set of neighbor ids

    let hoverNodeId = null;
    let hoverNeighborIds = new Set();

    // Simulated algorithm overlays (visual only — never hits the API)
    let simNodeIds = new Set();
    let simLinkKeys = new Set();
    let simTimer = null;
    let simClearTimer = null;

    function linkId(end) {
        return typeof end === "object" && end ? end.id : end;
    }

    function sortedLinkKey(a, b) {
        return a < b ? `${a}|${b}` : `${b}|${a}`;
    }

    function rebuildAdjacency(links) {
        adjacency = new Map();
        for (const link of links) {
            const s = linkId(link.source);
            const t = linkId(link.target);
            if (!s || !t) continue;
            if (!adjacency.has(s)) adjacency.set(s, new Set());
            if (!adjacency.has(t)) adjacency.set(t, new Set());
            adjacency.get(s).add(t);
            adjacency.get(t).add(s);
        }
    }

    function buildStructuralLinks(nodes) {
        const links = [];
        for (let i = 0; i < nodes.length; i++) {
            for (let j = i + 1; j < nodes.length; j++) {
                const sharesSource = nodes[i].source_id === nodes[j].source_id;
                const sharesTag = nodes[i].tags.some(t => nodes[j].tags.includes(t));
                if (sharesSource || sharesTag) {
                    links.push({ source: nodes[i].id, target: nodes[j].id });
                }
            }
        }
        return links;
    }

    function atomsToNodes(atoms) {
        return atoms.map(atom => {
            const tags = Array.isArray(atom.tags) ? atom.tags : [];
            return {
                id: atom.id,
                title: atom.title,
                source: atom.source,
                source_id: atom.source_id,
                tags,
                kind: "atom",
                val: Math.max(2, Math.min(6, tags.length + 2))
            };
        });
    }

    function isHoverActive() {
        return hoverNodeId != null;
    }

    function linkIsHovered(link) {
        if (!isHoverActive()) return false;
        const s = linkId(link.source);
        const t = linkId(link.target);
        return (s === hoverNodeId && hoverNeighborIds.has(t))
            || (t === hoverNodeId && hoverNeighborIds.has(s));
    }

    function linkIsSim(link) {
        const s = linkId(link.source);
        const t = linkId(link.target);
        return simLinkKeys.has(sortedLinkKey(s, t));
    }

    function applyGraphStyle(g) {
        g.backgroundColor("#000000")
            .nodeColor(node => {
                if (isHoverActive()) {
                    if (node.id === hoverNodeId) return "#ffffff";
                    if (hoverNeighborIds.has(node.id)) return "#e8e8e8";
                    return "#1a1a1a";
                }
                if (simNodeIds.has(node.id)) return "#f5f5f5";
                return "#ffffff";
            })
            .nodeRelSize(4)
            .nodeVal(node => {
                if (isHoverActive() && node.id === hoverNodeId) return 11;
                if (isHoverActive() && hoverNeighborIds.has(node.id)) return 8;
                if (simNodeIds.has(node.id)) return 7;
                return node.val || 3;
            })
            .nodeOpacity(0.95)
            .nodeLabel(node => {
                const tags = (node.tags || []).map(t => escapeHtml(t)).join(", ");
                return `
                    <div style="background: rgba(0, 0, 0, 0.92); border: 1px solid rgba(255,255,255,0.35); border-radius: 8px; padding: 12px; font-family: var(--font-mono); font-size: 0.85rem; color: #ffffff; pointer-events: none; box-shadow: 0 0 18px rgba(255,255,255,0.12);">
                        <strong style="color: #ffffff; font-size: 0.9rem;">${escapeHtml(node.title)}</strong><br/>
                        <span style="color: #888888;">Source: ${escapeHtml(node.source)}/${escapeHtml(node.source_id)}</span><br/>
                        <span style="color: #cccccc;">Tags: ${tags}</span>
                    </div>`;
            })
            .linkColor(link => {
                if (linkIsHovered(link)) return "rgba(255, 255, 255, 0.92)";
                if (isHoverActive()) return "rgba(255, 255, 255, 0.04)";
                if (linkIsSim(link)) return "rgba(255, 255, 255, 0.55)";
                return "rgba(255, 255, 255, 0.14)";
            })
            .linkWidth(link => {
                if (linkIsHovered(link)) return 1.6;
                if (linkIsSim(link)) return 1.1;
                return 0.35;
            })
            .linkOpacity(0.65)
            .linkDirectionalParticles(link => {
                if (linkIsHovered(link)) return 4;
                if (linkIsSim(link)) return 2;
                return 0;
            })
            .linkDirectionalParticleSpeed(0.005)
            .linkDirectionalParticleWidth(1.8)
            .linkDirectionalParticleColor(() => "#ffffff")
            .onNodeHover(node => {
                if (!node) {
                    hoverNodeId = null;
                    hoverNeighborIds = new Set();
                } else {
                    hoverNodeId = node.id;
                    hoverNeighborIds = new Set(adjacency.get(node.id) || []);
                }
                refreshGraphPaint();
            })
            .onNodeClick(node => {
                const atom = currentAtoms.find(a => a.id === node.id);
                if (atom) {
                    activeAtom = atom;
                    renderAtomsList(currentAtoms);
                    inspectAtom(atom);
                }
            });
        const charge = g.d3Force("charge");
        if (charge) charge.strength(-18);
        const linkF = g.d3Force("link");
        if (linkF) linkF.distance(30);
    }

    function refreshGraphPaint() {
        if (!Graph) return;
        Graph.nodeColor(Graph.nodeColor());
        Graph.nodeVal(Graph.nodeVal());
        Graph.linkColor(Graph.linkColor());
        Graph.linkWidth(Graph.linkWidth());
        Graph.linkDirectionalParticles(Graph.linkDirectionalParticles());
    }

    function clearSimOverlay() {
        simNodeIds = new Set();
        simLinkKeys = new Set();
        if (simClearTimer) {
            clearTimeout(simClearTimer);
            simClearTimer = null;
        }
        refreshGraphPaint();
    }

    function randomWalkPath(maxHops) {
        const nodeIds = [...adjacency.keys()];
        if (nodeIds.length < 2) return { nodes: [], linkKeys: [] };

        const start = nodeIds[Math.floor(Math.random() * nodeIds.length)];
        const pathNodes = [start];
        const pathLinks = [];
        let current = start;
        const hops = 2 + Math.floor(Math.random() * Math.max(1, maxHops - 1));

        for (let i = 0; i < hops; i++) {
            const neighbors = [...(adjacency.get(current) || [])];
            if (!neighbors.length) break;
            const next = neighbors[Math.floor(Math.random() * neighbors.length)];
            pathLinks.push(sortedLinkKey(current, next));
            pathNodes.push(next);
            current = next;
        }

        return { nodes: pathNodes, linkKeys: pathLinks };
    }

    function pulseSimActivity() {
        if (!Graph || adjacency.size < 2) {
            scheduleNextSim();
            return;
        }

        // Soft delayed "algorithm" traversal — visual only
        const walk = randomWalkPath(5);
        simNodeIds = new Set(walk.nodes);
        simLinkKeys = new Set(walk.linkKeys);
        refreshGraphPaint();

        const holdMs = 1600 + Math.floor(Math.random() * 1400);
        if (simClearTimer) clearTimeout(simClearTimer);
        simClearTimer = setTimeout(() => {
            clearSimOverlay();
            scheduleNextSim();
        }, holdMs);
    }

    function scheduleNextSim() {
        if (simTimer) clearTimeout(simTimer);
        // Delayed / staggered so activity feels like queued potential algorithms, not spam
        const delay = 2800 + Math.floor(Math.random() * 5200);
        simTimer = setTimeout(pulseSimActivity, delay);
    }

    function startSimActivity() {
        if (simTimer) clearTimeout(simTimer);
        // First pulse after a short settle so the graph can layout
        simTimer = setTimeout(pulseSimActivity, 2200);
    }

    function update3DGraph(atoms) {
        if (!graphContainer || typeof ForceGraph3D === "undefined") return;

        const nodes = atomsToNodes(atoms);
        baseLinks = buildStructuralLinks(nodes);
        rebuildAdjacency(baseLinks);
        clearSimOverlay();

        if (!Graph) {
            Graph = ForceGraph3D()(graphContainer);
            applyGraphStyle(Graph);
            Graph.graphData({ nodes, links: baseLinks.slice() });
            Graph.width(graphContainer.clientWidth);
            Graph.height(560);
            window.addEventListener("resize", () => {
                if (!Graph || !graphContainer) return;
                Graph.width(graphContainer.clientWidth);
                Graph.height(560);
            });
            setTimeout(() => { try { Graph.zoomToFit(1000, 80); } catch (e) {} }, 1400);
            startSimActivity();
        } else {
            Graph.graphData({ nodes, links: baseLinks.slice() });
            refreshGraphPaint();
            startSimActivity();
        }
    }

    // Kick off
    applyViewMode();
    loadDashboard();
});
