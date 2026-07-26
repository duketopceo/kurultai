// brain.js — wired to the local kurultai daemon.
// /api/* is proxied to http://127.0.0.1:8421 by vite.config.js so there are no CORS issues.
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

    // 1. Boot: pull status + initial atom list from the daemon
    async function loadDashboard() {
        try {
            const r = await fetch("/api/status");
            const j = await r.json();
            document.getElementById("stat-status").textContent = j.ok ? "Online" : "Offline";
            document.getElementById("stat-atoms").textContent = j.atoms ?? "—";
            document.getElementById("stat-env").textContent =
                (j.scheduler && j.scheduler.env) || "dev";
            // sources: derived from atom list below (daemon status doesn't expose source count directly)
        } catch (e) {
            document.getElementById("stat-status").textContent = "Daemon unreachable";
            console.error("status fetch failed:", e);
        }
        await triggerLoadAtoms();
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
        const seen = new Set();
        currentAtoms = (results || []).map(r => {
            const a = r.atom || r;
            return {
                id: a.id || a.title_hash || r.rank || Math.random().toString(36).slice(2, 11),
                title: a.title || "(untitled)",
                source: a.source || "",
                source_id: a.source_id || "",
                summary: a.summary || "",
                content: a.content || "",
                question: a.question || "",
                resolution: a.resolution || "",
                tags: a.tags || [],
                file_path: a.file_path || null,
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

    // 7. 3D synapse graph (3d-force-graph via CDN, same as before)
    function update3DGraph(atoms) {
        if (!graphContainer || typeof ForceGraph3D === "undefined") return;

        const nodes = atoms.map(atom => ({
            id: atom.id,
            title: atom.title,
            source: atom.source,
            source_id: atom.source_id,
            tags: atom.tags,
            // High-contrast palette: white default, purple for tagged/highlighted atoms
            color: (atom.tags && atom.tags.length > 0) ? "#c084fc" : "#ffffff",
            val: 5
        }));

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

        if (!Graph) {
            Graph = ForceGraph3D()(graphContainer)
                .graphData({ nodes, links })
                .backgroundColor("#000000")
                .nodeColor(node => node.color)
                .nodeLabel(node => `
                    <div style="background: rgba(0, 0, 0, 0.9); border: 1px solid #c084fc; border-radius: 8px; padding: 12px; font-family: var(--font-mono); font-size: 0.85rem; color: #ffffff; pointer-events: none;">
                        <strong style="color: #c084fc; font-size: 0.9rem;">${node.title}</strong><br/>
                        <span style="color: #888888;">Source: ${node.source}/${node.source_id}</span><br/>
                        <span style="color: #c084fc;">Tags: ${node.tags.join(", ")}</span>
                    </div>
                `)
                .nodeRelSize(3)
                .linkColor(() => "rgba(255, 255, 255, 0.12)")
                .linkWidth(0.5)
                .linkDirectionalParticles(2)
                .linkDirectionalParticleSpeed(0.006)
                .linkDirectionalParticleWidth(1.5)
                .linkDirectionalParticleColor(() => "#c084fc")
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

    // Kick off
    applyViewMode();
    loadDashboard();
});
