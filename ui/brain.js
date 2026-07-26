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
                // sources: derived from atom list below (daemon status doesn't expose source count directly)
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

        // distinct source count for the stat card
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
            ? `<button onclick="openFileInEditor('${escapeHtml(atom.file_path)}')" style="padding: 4px 12px; font-size: 0.75rem; border-radius: 9999px; background: rgba(168,85,247,0.1); border: 1px solid #c084fc; color: #c084fc; cursor: pointer; font-family: var(--font-mono);">Open File</button>`
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

    // 7. 3D synapse graph — stock ForceGraph3D styling + pathway overlays
    const AGENT_NODE_ID = "__mcp_agent__";
    let baseLinks = [];
    let pathwayIds = new Set();
    let pathwayClearTimer = null;
    let liveSince = 0;
    let liveTimer = null;
    let showcaseBusy = false;

    const pathwayStatusEl = document.getElementById("pathway-status");
    const answerChip = document.getElementById("answer-chip");
    const answerChipText = document.getElementById("answer-chip-text");
    const showcaseBtn = document.getElementById("showcase-btn");
    const liveBtn = document.getElementById("live-btn");

    function setPathwayStatus(text, active) {
        if (!pathwayStatusEl) return;
        pathwayStatusEl.textContent = text;
        pathwayStatusEl.classList.toggle("active", !!active);
    }

    function showAnswerChip(text) {
        if (!answerChip || !answerChipText) return;
        answerChipText.textContent = text || "";
        answerChip.classList.toggle("visible", !!text);
    }

    function sleep(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }

    function buildStructuralLinks(nodes) {
        const links = [];
        for (let i = 0; i < nodes.length; i++) {
            for (let j = i + 1; j < nodes.length; j++) {
                const sharesSource = nodes[i].source_id === nodes[j].source_id;
                const sharesTag = nodes[i].tags.some(t => nodes[j].tags.includes(t));
                if (sharesSource || sharesTag) {
                    links.push({ source: nodes[i].id, target: nodes[j].id, path: false });
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
                color: tags.length > 0 ? "#c084fc" : "#ffffff",
                val: Math.max(2, Math.min(6, tags.length + 2))
            };
        });
    }

    function applyGraphStyle(g) {
        g.backgroundColor("#000000")
            .nodeColor(node => {
                if (node.id === AGENT_NODE_ID) return "#22c55e";
                if (pathwayIds.has(node.id)) return "#fbbf24";
                if (pathwayIds.size > 0 && node.kind === "atom") return "#444444";
                return node.color || "#ffffff";
            })
            .nodeRelSize(4)
            .nodeVal(node => (pathwayIds.has(node.id) || node.id === AGENT_NODE_ID ? 10 : node.val || 3))
            .nodeOpacity(0.95)
            .nodeLabel(node => {
                const tags = (node.tags || []).map(t => escapeHtml(t)).join(", ");
                return `
                    <div style="background: rgba(0, 0, 0, 0.9); border: 1px solid #c084fc; border-radius: 8px; padding: 12px; font-family: var(--font-mono); font-size: 0.85rem; color: #ffffff; pointer-events: none;">
                        <strong style="color: #c084fc; font-size: 0.9rem;">${escapeHtml(node.title)}</strong><br/>
                        <span style="color: #888888;">Source: ${escapeHtml(node.source)}/${escapeHtml(node.source_id)}</span><br/>
                        <span style="color: #c084fc;">Tags: ${tags}</span>
                    </div>`;
            })
            .linkColor(link => (link.path ? "rgba(251, 191, 36, 0.85)" : "rgba(168, 85, 247, 0.22)"))
            .linkWidth(link => (link.path ? 1.8 : 0.45))
            .linkDirectionalParticles(link => (link.path ? 3 : 0))
            .linkDirectionalParticleSpeed(0.006)
            .linkDirectionalParticleWidth(2)
            .linkDirectionalParticleColor(() => "#fbbf24")
            .onNodeClick(node => {
                if (node.id === AGENT_NODE_ID) return;
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

    function clearPathway(keepStatus) {
        pathwayIds = new Set();
        if (pathwayClearTimer) {
            clearTimeout(pathwayClearTimer);
            pathwayClearTimer = null;
        }
        if (!Graph) return;
        const data = Graph.graphData();
        const nodes = data.nodes.filter(n => n.id !== AGENT_NODE_ID);
        Graph.graphData({ nodes, links: baseLinks.slice() });
        refreshGraphPaint();
        if (!keepStatus) {
            setPathwayStatus("idle", false);
            showAnswerChip("");
        }
    }

    async function playPathway({ atomIds, mode, holdMs, agentId }) {
        if (!Graph || !atomIds || !atomIds.length) return;
        const present = new Set(Graph.graphData().nodes.map(n => n.id));
        const ids = atomIds.filter(id => present.has(id));
        if (!ids.length) return;

        pathwayIds = new Set(ids);
        const pathLinks = [];
        if (agentId && present.has(agentId)) {
            pathLinks.push({ source: agentId, target: ids[0], path: true });
        }
        for (let i = 0; i < ids.length - 1; i++) {
            pathLinks.push({ source: ids[i], target: ids[i + 1], path: true });
        }
        const data = Graph.graphData();
        Graph.graphData({
            nodes: data.nodes,
            links: baseLinks.concat(pathLinks)
        });
        refreshGraphPaint();

        try {
            const focusNodes = data.nodes.filter(n => pathwayIds.has(n.id) || n.id === agentId);
            if (focusNodes.length) Graph.zoomToFit(800, 60, n => pathwayIds.has(n.id) || n.id === agentId);
        } catch (e) { /* ignore */ }

        const hold = holdMs != null ? holdMs : (mode === "live" ? 2200 : 1600);
        if (pathwayClearTimer) clearTimeout(pathwayClearTimer);
        if (mode === "live") {
            pathwayClearTimer = setTimeout(() => {
                pathwayIds = new Set();
                if (!Graph) return;
                const d = Graph.graphData();
                Graph.graphData({
                    nodes: d.nodes.filter(n => n.id !== AGENT_NODE_ID),
                    links: baseLinks.slice()
                });
                refreshGraphPaint();
            }, hold);
        }
        await sleep(hold);
    }

    function update3DGraph(atoms) {
        if (!graphContainer || typeof ForceGraph3D === "undefined") return;

        const nodes = atomsToNodes(atoms);
        baseLinks = buildStructuralLinks(nodes);

        if (!Graph) {
            Graph = ForceGraph3D()(graphContainer);
            applyGraphStyle(Graph);
            Graph.graphData({ nodes, links: baseLinks.slice() });
            Graph.width(graphContainer.clientWidth);
            Graph.height(500);
            window.addEventListener("resize", () => {
                if (!Graph || !graphContainer) return;
                Graph.width(graphContainer.clientWidth);
                if (!graphContainer.classList.contains("fullscreen")) {
                    Graph.height(500);
                }
            });
            setTimeout(() => { try { Graph.zoomToFit(1000, 80); } catch (e) {} }, 1400);
        } else {
            const agent = Graph.graphData().nodes.find(n => n.id === AGENT_NODE_ID);
            const nextNodes = agent ? nodes.concat([agent]) : nodes;
            Graph.graphData({ nodes: nextNodes, links: baseLinks.slice() });
            refreshGraphPaint();
        }
    }

    // 8. Fullscreen + Suggested
    const fullscreenBtn = document.getElementById("fullscreen-btn");
    const suggestBtn = document.getElementById("suggest-btn");

    if (fullscreenBtn) {
        fullscreenBtn.addEventListener("click", () => {
            if (!graphContainer) return;
            const isFs = graphContainer.classList.toggle("fullscreen");
            fullscreenBtn.textContent = isFs ? "Exit Fullscreen" : "Fullscreen";
            fullscreenBtn.classList.toggle("primary", isFs);
            requestAnimationFrame(() => {
                if (!Graph) return;
                Graph.width(graphContainer.clientWidth);
                Graph.height(graphContainer.clientHeight);
            });
        });
    }

    function suggestAtoms() {
        if (!currentAtoms.length || !Graph) return;
        const scoreMap = new Map();
        for (const a of currentAtoms) {
            let score = 0;
            for (const b of currentAtoms) {
                if (a.id === b.id) continue;
                const sharesSource = a.source_id && a.source_id === b.source_id;
                const sharesTag = (a.tags || []).some(t => (b.tags || []).includes(t));
                if (sharesSource || sharesTag) score++;
            }
            scoreMap.set(a.id, score);
        }
        const top = [...currentAtoms]
            .sort((a, b) => (scoreMap.get(b.id) || 0) - (scoreMap.get(a.id) || 0))
            .slice(0, 2);
        if (!top.length) return;
        playPathway({ atomIds: top.map(a => a.id), mode: "live", holdMs: 2500 });
        activeAtom = top[0];
        renderAtomsList(currentAtoms);
        inspectAtom(top[0]);
    }

    if (suggestBtn) {
        suggestBtn.addEventListener("click", suggestAtoms);
    }

    // 9. Showcase MCP walk
    async function runShowcase() {
        if (showcaseBusy || !Graph) return;
        showcaseBusy = true;
        if (showcaseBtn) showcaseBtn.disabled = true;
        showAnswerChip("");
        clearPathway(true);

        const q = (searchInput && searchInput.value.trim()) || "kurultai";
        setPathwayStatus(`showcase · entering · ${q}`, true);

        const data = Graph.graphData();
        const agentNode = {
            id: AGENT_NODE_ID,
            title: "MCP agent",
            source: "mcp",
            source_id: "showcase",
            tags: [],
            kind: "agent",
            color: "#22c55e",
            val: 12,
            x: 80,
            y: 80,
            z: 80
        };
        Graph.graphData({ nodes: data.nodes.concat([agentNode]), links: baseLinks.slice() });
        refreshGraphPaint();
        await sleep(600);

        let searchIds = [];
        try {
            setPathwayStatus(`showcase · search · ${q}`, true);
            const r = await fetch("/api/search?q=" + encodeURIComponent(q) + "&limit=8");
            const results = await r.json();
            searchIds = (results || []).map(row => (row.atom || row).id).filter(Boolean);
            await playPathway({
                atomIds: searchIds,
                mode: "showcase",
                holdMs: 1400,
                agentId: AGENT_NODE_ID
            });
        } catch (e) {
            console.error("showcase search failed:", e);
            setPathwayStatus("showcase · search failed", true);
        }

        try {
            setPathwayStatus(`showcase · ask · ${q}`, true);
            const r = await fetch("/api/ask", {
                method: "POST",
                headers: { "content-type": "application/json" },
                body: JSON.stringify({ question: q })
            });
            const answer = await r.json();
            const citeIds = searchIds.slice(0, 5);
            showAnswerChip(answer.answer || "(no answer)");
            await playPathway({
                atomIds: citeIds,
                mode: "showcase",
                holdMs: 1800,
                agentId: AGENT_NODE_ID
            });
            setPathwayStatus("showcase · answered", true);
        } catch (e) {
            console.error("showcase ask failed:", e);
            setPathwayStatus("showcase · ask failed", true);
        }

        await sleep(700);
        setPathwayStatus("showcase · leaving", true);
        const after = Graph.graphData();
        Graph.graphData({
            nodes: after.nodes.filter(n => n.id !== AGENT_NODE_ID),
            links: baseLinks.slice()
        });
        pathwayIds = new Set();
        refreshGraphPaint();
        await sleep(400);
        setPathwayStatus("idle", false);

        showcaseBusy = false;
        if (showcaseBtn) showcaseBtn.disabled = false;
    }

    if (showcaseBtn) {
        showcaseBtn.addEventListener("click", () => { runShowcase(); });
    }

    // 10. Live toggle — poll /api/activity
    async function handleLiveEvent(ev) {
        const tool = ev.tool || "";
        const q = ev.query || "";
        if (tool === "search_hop") {
            setPathwayStatus(`live · hop · ${q}`, true);
            await playPathway({ atomIds: ev.atom_ids || [], mode: "live", holdMs: 900 });
            return;
        }
        if (tool === "search") {
            setPathwayStatus(`live · search · ${q}`, true);
            await playPathway({ atomIds: ev.atom_ids || [], mode: "live", holdMs: 2000 });
            return;
        }
        if (tool === "ask") {
            setPathwayStatus(`live · ask · ${q}`, true);
            if (ev.detail) showAnswerChip(ev.detail);
            await playPathway({ atomIds: ev.atom_ids || [], mode: "live", holdMs: 2400 });
            return;
        }
        if (tool === "remember") {
            setPathwayStatus(`live · remembered id=${(ev.atom_ids || [])[0] || "?"}`, true);
            await triggerLoadAtoms();
            await playPathway({ atomIds: ev.atom_ids || [], mode: "live", holdMs: 2600 });
            return;
        }
        if (tool === "cite" || tool === "who_knows") {
            setPathwayStatus(`live · ${tool} · ${q}`, true);
            await playPathway({ atomIds: ev.atom_ids || [], mode: "live", holdMs: 1600 });
        }
    }

    async function pollLive() {
        try {
            const r = await fetch("/api/activity?since=" + liveSince);
            const j = await r.json();
            liveSince = j.next_seq ?? liveSince;
            const events = j.events || [];
            for (const ev of events) {
                await handleLiveEvent(ev);
            }
        } catch (e) {
            console.error("live poll failed:", e);
        }
    }

    function setLive(on) {
        if (liveBtn) {
            liveBtn.classList.toggle("live-on", on);
            liveBtn.setAttribute("aria-pressed", String(on));
            liveBtn.textContent = on ? "Live · ON" : "Live";
        }
        if (liveTimer) {
            clearInterval(liveTimer);
            liveTimer = null;
        }
        if (on) {
            setPathwayStatus("live · watching", true);
            // Seed since to current tip so we only see new traffic
            fetch("/api/activity?since=0")
                .then(r => r.json())
                .then(j => {
                    liveSince = j.next_seq || 0;
                    liveTimer = setInterval(pollLive, 400);
                })
                .catch(() => {
                    liveTimer = setInterval(pollLive, 400);
                });
        } else {
            clearPathway(false);
        }
    }

    if (liveBtn) {
        liveBtn.addEventListener("click", () => {
            const on = liveBtn.getAttribute("aria-pressed") !== "true";
            setLive(on);
        });
    }

    // Kick off
    applyViewMode();
    loadDashboard();
});
