document.addEventListener("DOMContentLoaded", () => {
    const listContainer = document.getElementById("atoms-list-container");
    const inspector = document.getElementById("atom-inspector");
    const searchInput = document.getElementById("brain-search");
    const graphContainer = document.getElementById("3d-synapse-graph");

    let activeAtom = brainAtoms[0];

    // 1. Populate Lists and Inspect Actions
    function renderAtomsList(filteredAtoms) {
        listContainer.innerHTML = "";
        if (filteredAtoms.length === 0) {
            listContainer.innerHTML = `<div style="text-align: center; color: var(--text-muted); padding: 20px;">No matching atoms in SQLite store.</div>`;
            return;
        }

        filteredAtoms.forEach(atom => {
            const div = document.createElement("div");
            div.className = `atom-item ${activeAtom && activeAtom.id === atom.id ? "active" : ""}`;
            div.innerHTML = `
                <h4>${atom.title}</h4>
                <div class="atom-meta">
                    <span>${atom.source}/${atom.source_id}</span>
                    <span>${atom.id}</span>
                </div>
            `;
            div.addEventListener("click", () => {
                activeAtom = atom;
                document.querySelectorAll(".atom-item").forEach(el => el.classList.remove("active"));
                div.classList.add("active");
                inspectAtom(atom);
                if (window.Graph) {
                    window.Graph.centerAt(atom.x, atom.y, atom.z, 1000);
                    window.Graph.zoomToFit(1000, 100);
                }
            });
            listContainer.appendChild(div);
        });
    }

    function inspectAtom(atom) {
        if (!atom) {
            inspector.innerHTML = `
                <div style="text-align: center; color: var(--text-muted); padding-top: 100px;">
                    <p>Select an atom from the list to inspect its database structure</p>
                </div>
            `;
            return;
        }

        const tagPills = atom.tags.map(t => `<span class="tag-pill">${t}</span>`).join("");
        const pseudoEmbedding = Array.from({length: 8}, (_, i) => (Math.sin(atom.id.charCodeAt(0) + i) * 0.1).toFixed(6)).join(", ") + ", ...";

        inspector.innerHTML = `
            <div class="detail-header">
                <div>
                    <h3 class="detail-title">${atom.title}</h3>
                    <div style="margin-top: 8px;">${tagPills}</div>
                </div>
                <div class="detail-label" style="text-align: right;">ID: ${atom.id}</div>
            </div>
            
            <div class="detail-layout">
                <div class="detail-row">
                    <div class="detail-label">Source Context</div>
                    <div class="detail-val" style="font-family: var(--font-mono);">${atom.source} / ${atom.source_id} (updated: ${atom.source_updated_at})</div>
                </div>
                <div class="detail-row">
                    <div class="detail-label">Raw Database Content (content)</div>
                    <div class="detail-val">${escapeHtml(atom.content)}</div>
                </div>
                <div class="detail-row">
                    <div class="detail-label">LLM-Distilled Summary (summary)</div>
                    <div class="detail-val">${escapeHtml(atom.summary)}</div>
                </div>
                <div class="detail-row">
                    <div class="detail-label">Routing Queries (question / resolution)</div>
                    <div class="detail-val"><strong>Q:</strong> ${escapeHtml(atom.question)}\n<strong>A:</strong> ${escapeHtml(atom.resolution)}</div>
                </div>
                <div class="detail-row">
                    <div class="detail-label">SQLite-Vec Embedding (float[3072])</div>
                    <div class="detail-val" style="font-family: var(--font-mono); font-size: 0.85rem; color: var(--primary);">[${pseudoEmbedding}]</div>
                </div>
            </div>
        `;
    }

    function escapeHtml(text) {
        if (!text) return "";
        return text
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/"/g, "&quot;")
            .replace(/'/g, "&#039;");
    }

    // 2. Interactive FTS/Semantic Search
    if (searchInput) {
        searchInput.addEventListener("input", (e) => {
            const query = e.target.value.toLowerCase().trim();
            if (!query) {
                renderAtomsList(brainAtoms);
                return;
            }

            const matches = brainAtoms.filter(atom => 
                atom.title.toLowerCase().includes(query) ||
                atom.content.toLowerCase().includes(query) ||
                atom.summary.toLowerCase().includes(query) ||
                atom.tags.some(t => t.toLowerCase().includes(query))
            );
            renderAtomsList(matches);
        });
    }

    // Initialize list and inspector
    renderAtomsList(brainAtoms);
    inspectAtom(activeAtom);

    // 3. 3D Synaptic Force Graph Projection (Using 3d-force-graph library via CDN)
    if (graphContainer && typeof ForceGraph3D !== 'undefined') {
        const nodes = brainAtoms.map(atom => ({
            id: atom.id,
            title: atom.title,
            source: atom.source,
            source_id: atom.source_id,
            tags: atom.tags,
            // Non-standard high-tech neon colors matching theme
            color: atom.source_id.includes('guidelines') ? '#c084fc' : (atom.source_id.includes('migration') ? '#38bdf8' : '#10b981'),
            val: 5
        }));

        const links = [];
        for (let i = 0; i < nodes.length; i++) {
            for (let j = i + 1; j < nodes.length; j++) {
                const sharesSource = nodes[i].source_id === nodes[j].source_id;
                const sharesTag = nodes[i].tags.some(t => nodes[j].tags.includes(t));
                if (sharesSource || sharesTag) {
                    links.push({
                        source: nodes[i].id,
                        target: nodes[j].id
                    });
                }
            }
        }

        window.Graph = ForceGraph3D()(graphContainer)
            .graphData({ nodes, links })
            .backgroundColor('#030712')
            .nodeColor(node => node.color)
            .nodeLabel(node => `
                <div style="background: rgba(17, 24, 39, 0.85); backdrop-filter: blur(8px); border: 1px solid var(--border-color); border-radius: 8px; padding: 12px; font-family: var(--font-mono); font-size: 0.85rem; color: var(--text-primary); pointer-events: none;">
                    <strong style="color: var(--primary); font-size: 0.9rem;">${node.title}</strong><br/>
                    <span style="color: var(--text-muted);">Source: ${node.source}/${node.source_id}</span><br/>
                    <span style="color: var(--secondary);">Tags: ${node.tags.join(', ')}</span>
                </div>
            `)
            .nodeRelSize(3)
            .linkColor(() => 'rgba(255, 255, 255, 0.08)')
            .linkWidth(0.5)
            // Particle electrons transmission effect through synapses
            .linkDirectionalParticles(2)
            .linkDirectionalParticleSpeed(0.006)
            .linkDirectionalParticleWidth(1.5)
            .linkDirectionalParticleColor(() => '#38bdf8')
            .onNodeClick(node => {
                const atom = brainAtoms.find(a => a.id === node.id);
                if (atom) {
                    activeAtom = atom;
                    renderAtomsList(brainAtoms);
                    inspectAtom(atom);
                }
            });

        // Set dimensions match the glass-panel dimensions
        window.Graph.width(graphContainer.clientWidth);
        window.Graph.height(500);

        window.addEventListener("resize", () => {
            window.Graph.width(graphContainer.clientWidth);
        });
    }
});
