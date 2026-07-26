// brain.js — dashboard + whole-brain synaptic storm (neurons / electrons / zaps).
document.addEventListener("DOMContentLoaded", () => {
    const listContainer = document.getElementById("atoms-list-container");
    const inspector = document.getElementById("atom-inspector");
    const searchInput = document.getElementById("brain-search");
    const searchClearBtn = document.getElementById("search-clear");
    const stage = document.getElementById("3d-synapse-graph");
    const hint = document.getElementById("brain-hud-hint");
    const layoutForceBtn = document.getElementById("layout-force");
    const layoutBrainBtn = document.getElementById("layout-brain");

    const LIST_HOT_CAP = 400;
    const GRAPH_TIER_LIMIT = 20000;
    const BRAIN_MAX_NODES = 1400;
    const AMBIENT_PARTICLE_LINKS = 0.18;
    const FORMATION_RADIUS = 92;

    let currentAtoms = [];
    let activeAtom = null;
    let Graph = null;
    let graphBuiltForSig = "";
    let isSearchMode = false;
    let layoutMode = "force"; // force | brain
    let formationAngle = 0;
    let formationSpinTimer = null;

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

    function renderMemoryStats(memory) {
        const el = document.getElementById("stat-memory");
        if (!el || !memory) return;
        el.textContent = `${memory.hot ?? 0} / ${memory.warm ?? 0} / ${memory.cold ?? 0}`;
    }

    async function loadDashboard() {
        const statusPromise = fetch("/api/status")
            .then(async r => {
                const j = await r.json();
                document.getElementById("stat-status").textContent = j.ok ? "Online" : "Offline";
                document.getElementById("stat-atoms").textContent = j.atoms ?? "—";
                document.getElementById("stat-env").textContent =
                    (j.scheduler && j.scheduler.env) || "dev";
                if (j.memory) renderMemoryStats(j.memory);
            })
            .catch(e => {
                document.getElementById("stat-status").textContent = "Daemon unreachable";
                console.error("status fetch failed:", e);
            });
        await Promise.all([statusPromise, triggerLoadAtoms()]);
    }

    async function fetchGraphTier(tier) {
        const r = await fetch(`/api/graph?tier=${tier}&limit=${GRAPH_TIER_LIMIT}`);
        if (!r.ok) throw new Error(`graph ${tier} failed: ${r.status}`);
        return r.json();
    }

    async function triggerLoadAtoms() {
        isSearchMode = false;
        updateSearchClearVisibility();
        try {
            const hot = await fetchGraphTier("hot");
            mergeGraphNodes(hot.nodes || [], { replace: true });
            if (hint) hint.textContent = `hot ${hot.count ?? 0} · loading warm / cold…`;
            const [warm, cold] = await Promise.all([
                fetchGraphTier("warm"),
                fetchGraphTier("cold")
            ]);
            mergeGraphNodes([...(warm.nodes || []), ...(cold.nodes || [])], { replace: false });
            if (hint) {
                hint.textContent =
                    `hot ${hot.count ?? 0} · warm ${warm.count ?? 0} · cold ${cold.count ?? 0} · drag · scroll · click a neuron`;
            }
            if (layoutMode !== "force") applyFormationLayout(layoutMode, { animate: false });
        } catch (e) {
            console.error("graph fetch failed:", e);
            if (hint) hint.textContent = "could not reach /api/graph";
            listContainer.innerHTML = `<div style="text-align:center;color:var(--text-muted);padding:20px;">Could not reach the local daemon.</div>`;
        }
    }

    function updateSearchClearVisibility() {
        if (!searchClearBtn) return;
        const show = isSearchMode || !!(searchInput && searchInput.value.trim());
        searchClearBtn.hidden = !show;
    }

    async function clearSearch() {
        if (searchInput) searchInput.value = "";
        isSearchMode = false;
        updateSearchClearVisibility();
        await triggerLoadAtoms();
    }

    async function triggerSearch(query) {
        if (!query) return clearSearch();
        isSearchMode = true;
        updateSearchClearVisibility();
        try {
            const r = await fetch("/api/search?q=" + encodeURIComponent(query) + "&limit=25");
            const results = await r.json();
            processResults(results);
            if (layoutMode !== "force") applyFormationLayout(layoutMode, { animate: false });
        } catch (e) {
            console.error("search fetch failed:", e);
            listContainer.innerHTML = `<div style="text-align:center;color:var(--text-muted);padding:20px;">Search request failed.</div>`;
        }
    }

    function graphNodeToAtom(n) {
        return {
            id: n.id || Math.random().toString(36).slice(2, 11),
            title: n.title || "(untitled)",
            source: n.source || "",
            source_id: n.source_id || "",
            summary: n.summary || "",
            content: "",
            question: "",
            resolution: "",
            tags: [],
            file_path: null,
            source_updated_at: "",
            indexed_at: n.indexed_at || "",
            last_accessed_at: n.last_accessed_at || "",
            tier: n.tier || "warm",
            stub: n.tier !== "hot",
            score: undefined
        };
    }

    function mergeGraphNodes(nodes, { replace }) {
        const mapped = (nodes || []).map(graphNodeToAtom);
        if (replace) currentAtoms = mapped;
        else {
            const byId = new Map(currentAtoms.map(a => [a.id, a]));
            for (const a of mapped) {
                if (!byId.has(a.id)) byId.set(a.id, a);
            }
            currentAtoms = Array.from(byId.values());
        }
        refreshFromCurrentAtoms({ rebuildBrain: true });
    }

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
                indexed_at: a.indexed_at || "",
                last_accessed_at: a.last_accessed_at || "",
                tier: "hot",
                stub: false,
                score: r.score
            };
        });
        refreshFromCurrentAtoms({ rebuildBrain: true });
    }

    function atomsForList() {
        const hot = currentAtoms.filter(a => a.tier === "hot");
        const pool = hot.length ? hot : currentAtoms;
        return pool.slice(0, LIST_HOT_CAP);
    }

    function refreshFromCurrentAtoms({ rebuildBrain } = { rebuildBrain: false }) {
        const sources = new Set(currentAtoms.map(a => a.source).filter(Boolean));
        const sourcesEl = document.getElementById("stat-sources");
        if (sourcesEl) sourcesEl.textContent = sources.size || "—";

        const listSlice = atomsForList();
        renderAtomsList(listSlice);

        if (currentAtoms.length > 0) {
            const keep = activeAtom && currentAtoms.find(a => a.id === activeAtom.id);
            activeAtom = keep || listSlice[0] || currentAtoms[0];
            inspectAtom(activeAtom);
            if (rebuildBrain) updateMainBrain(currentAtoms);
            else focusCameraOnAtom(activeAtom);
            refreshGraphPaint();
        } else {
            activeAtom = null;
            if (Graph) Graph.graphData({ nodes: [], links: [] });
        }
    }

    function hydrateFromServerAtom(a) {
        const meta = a.metadata || {};
        return {
            id: a.id,
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
            indexed_at: a.indexed_at || "",
            last_accessed_at: a.last_accessed_at || "",
            tier: "hot",
            stub: false,
            score: undefined
        };
    }

    async function selectAtom(atom, { scrollList } = {}) {
        if (!atom) return;
        activeAtom = atom;
        renderAtomsList(atomsForList());
        inspectAtom(atom);
        focusCameraOnAtom(atom);
        refreshGraphPaint();
        if (scrollList) {
            const el = listContainer.querySelector(".atom-item.active");
            if (el) el.scrollIntoView({ block: "nearest" });
        }
        try {
            const r = await fetch("/api/touch", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ atom_id: atom.id })
            });
            if (!r.ok) return;
            const j = await r.json();
            if (!j.atom) return;
            const full = hydrateFromServerAtom(j.atom);
            const idx = currentAtoms.findIndex(a => a.id === full.id);
            if (idx >= 0) currentAtoms[idx] = { ...currentAtoms[idx], ...full };
            else currentAtoms.push(full);
            if (activeAtom && activeAtom.id === full.id) {
                activeAtom = currentAtoms.find(a => a.id === full.id) || full;
                inspectAtom(activeAtom);
                renderAtomsList(atomsForList());
                refreshGraphPaint();
            }
        } catch (e) {
            console.error("touch failed:", e);
        }
    }

    function renderAtomsList(atoms) {
        listContainer.innerHTML = "";
        if (!atoms || atoms.length === 0) {
            listContainer.innerHTML = `<div style="text-align:center;color:var(--text-muted);padding:20px;">No atoms in the local store.</div>`;
            return;
        }
        atoms.forEach(atom => {
            const div = document.createElement("div");
            div.className = `atom-item ${activeAtom && activeAtom.id === atom.id ? "active" : ""}`;
            const tier = atom.tier
                ? `<span class="tier-chip tier-${escapeHtml(atom.tier)}">${escapeHtml(atom.tier)}</span>`
                : "";
            div.innerHTML = `
                <h4>${escapeHtml(atom.title)} ${tier}</h4>
                <div class="atom-meta">
                    <span>${escapeHtml(atom.source)}/${escapeHtml(atom.source_id)}</span>
                    <span>${atom.score !== undefined ? atom.score.toFixed(3) : atom.id.slice(0, 8)}</span>
                </div>
            `;
            div.addEventListener("click", () => selectAtom(atom));
            listContainer.appendChild(div);
        });
    }

    function inspectAtom(atom) {
        if (!atom) {
            inspector.innerHTML = `<div style="text-align:center;color:var(--text-muted);padding-top:100px;"><p>Select an atom to inspect its structure</p></div>`;
            return;
        }
        const tagPills = (atom.tags || []).map(t => `<span class="tag-pill">${escapeHtml(t)}</span>`).join("");
        const sourceLabel = isTechnical ? "Source Context" : "Memory Origin";
        const contentLabel = isTechnical ? "Raw Database Content (content)" : "Excerpt / Content";
        const summaryLabel = isTechnical ? "LLM-Distilled Summary (summary)" : "Summary";
        const updatedSuffix = isTechnical && atom.source_updated_at
            ? ` (updated: ${escapeHtml(atom.source_updated_at)})`
            : "";
        const idHeader = isTechnical
            ? `<div class="detail-label" style="text-align:right;">ID: ${escapeHtml(atom.id)}</div>`
            : "";
        const openFileBtn = isTechnical && atom.file_path
            ? `<button onclick="openFileInEditor('${escapeHtml(atom.file_path)}')" style="padding:4px 12px;font-size:0.75rem;border-radius:9999px;background:rgba(255,255,255,0.08);border:1px solid rgba(160,200,255,0.4);color:#fff;cursor:pointer;font-family:var(--font-mono);">Open File</button>`
            : "";
        const routingRow = isTechnical && atom.question
            ? `<div class="detail-row"><div class="detail-label">Routing Queries</div><div class="detail-val"><strong>Q:</strong> ${escapeHtml(atom.question)}<br/><strong>A:</strong> ${escapeHtml(atom.resolution || "")}</div></div>`
            : "";
        const body = atom.content || atom.summary || "(stub — click again after focus to hydrate)";

        inspector.innerHTML = `
            <div class="detail-header">
                <div>
                    <h3 class="detail-title">${escapeHtml(atom.title)}</h3>
                    <div style="margin-top:8px;">${tagPills}</div>
                </div>
                ${idHeader}
            </div>
            <div class="detail-layout">
                <div class="detail-row">
                    <div class="detail-label">${sourceLabel}</div>
                    <div class="detail-val" style="font-family:var(--font-mono);display:flex;justify-content:space-between;align-items:center;gap:12px;">
                        <span>${escapeHtml(atom.source)} / ${escapeHtml(atom.source_id)}${updatedSuffix}</span>
                        ${openFileBtn}
                    </div>
                </div>
                <div class="detail-row">
                    <div class="detail-label">${contentLabel}</div>
                    <div class="detail-val">${escapeHtml(body)}</div>
                </div>
                <div class="detail-row">
                    <div class="detail-label">${summaryLabel}</div>
                    <div class="detail-val">${escapeHtml(atom.summary || "—")}</div>
                </div>
                ${routingRow}
            </div>
        `;
    }

    window.openFileInEditor = async function (filePath) {
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

    let searchTimeout = null;
    if (searchInput) {
        searchInput.addEventListener("input", e => {
            clearTimeout(searchTimeout);
            const q = e.target.value.trim();
            updateSearchClearVisibility();
            searchTimeout = setTimeout(() => triggerSearch(q), 300);
        });
        searchInput.addEventListener("keydown", e => {
            if (e.key === "Escape") {
                e.preventDefault();
                clearSearch();
            }
        });
    }
    if (searchClearBtn) {
        searchClearBtn.addEventListener("click", () => clearSearch());
    }

    function setLayoutMode(mode) {
        layoutMode = mode;
        [layoutForceBtn, layoutBrainBtn].forEach(btn => {
            if (!btn) return;
            const id = btn.id;
            const active =
                (id === "layout-force" && mode === "force") ||
                (id === "layout-brain" && mode === "brain");
            btn.classList.toggle("active", active);
            btn.setAttribute("aria-pressed", String(active));
        });
        const controls = Graph && Graph.controls();
        if (mode === "force") {
            clearFormationPins();
            stopFormationSpin();
            if (controls) controls.autoRotate = true;
            if (hint) {
                hint.textContent = "organic force · drag · scroll · click a neuron";
            }
        } else {
            if (controls) controls.autoRotate = false;
            applyFormationLayout(mode, { animate: true });
            startFormationSpin();
            if (hint) {
                hint.textContent = "brain formation · slow spin · organic to release";
            }
        }
    }

    if (layoutForceBtn) layoutForceBtn.addEventListener("click", () => setLayoutMode("force"));
    if (layoutBrainBtn) layoutBrainBtn.addEventListener("click", () => setLayoutMode("brain"));

    function hash01(i) {
        let h = (i * 2654435761) >>> 0;
        h ^= h >>> 16;
        h = Math.imul(h, 2246822507);
        h ^= h >>> 13;
        return (h >>> 0) / 4294967296;
    }

    /** Two-hemisphere cloud with cortical folding — from neural patch. */
    function brainPoint(i, n, radius) {
        const side = i % 2 === 0 ? -1 : 1;
        const u = hash01(i);
        const v = hash01(i + 7919);
        const interior = hash01(i + 104729) < 0.18;
        const shell = interior ? 0.55 + hash01(i + 3) * 0.4 : 0.97 + hash01(i + 5) * 0.05;
        const a = 1.0, b = 0.82, c = 1.12;
        const theta = u * Math.PI * 2;
        const phi = Math.acos(1 - 2 * Math.min(0.999, Math.max(0.001, v)));
        const folds = 1 + 0.06 * Math.sin(phi * 9.0) * Math.sin(theta * 7.0);
        const r = shell * folds;
        const x = side * 0.42 + side * a * r * Math.sin(phi) * Math.cos(theta) * 0.82;
        const y = b * r * Math.cos(phi) * 0.92;
        const z = c * r * Math.sin(phi) * Math.sin(theta) * 0.88;
        return {
            x: x * radius,
            y: y * radius,
            z: z * radius
        };
    }

    function clearFormationPins() {
        if (!Graph) return;
        const nodes = Graph.graphData().nodes || [];
        for (const node of nodes) {
            node.fx = undefined;
            node.fy = undefined;
            node.fz = undefined;
        }
        const charge = Graph.d3Force("charge");
        if (charge) charge.strength(-38);
        const linkF = Graph.d3Force("link");
        if (linkF) linkF.distance(40);
        Graph.d3ReheatSimulation();
    }

    function applyFormationLayout(mode, { animate } = { animate: true }) {
        if (!Graph || mode !== "brain") return;
        const nodes = Graph.graphData().nodes || [];
        const n = nodes.length;
        if (!n) return;
        const radius = FORMATION_RADIUS * Math.min(1.35, 0.55 + Math.sqrt(n) / 28);
        for (let i = 0; i < n; i++) {
            const p = brainPoint(i, n, radius);
            // Apply current spin angle so layout doesn't jump on refresh
            const c = Math.cos(formationAngle);
            const s = Math.sin(formationAngle);
            const x = p.x * c - p.z * s;
            const z = p.x * s + p.z * c;
            nodes[i].fx = x;
            nodes[i].fy = p.y;
            nodes[i].fz = z;
            if (!animate) {
                nodes[i].x = x;
                nodes[i].y = p.y;
                nodes[i].z = z;
            }
        }
        const charge = Graph.d3Force("charge");
        if (charge) charge.strength(-2);
        const linkF = Graph.d3Force("link");
        if (linkF) linkF.distance(18);
        Graph.d3ReheatSimulation();
        if (animate) {
            setTimeout(() => {
                try { Graph.zoomToFit(900, 70); } catch (_) {}
            }, 400);
        }
    }

    function startFormationSpin() {
        stopFormationSpin();
        formationSpinTimer = setInterval(() => {
            if (!Graph || layoutMode !== "brain") return;
            formationAngle += 0.008; // slow spin
            const nodes = Graph.graphData().nodes || [];
            const n = nodes.length;
            const radius = FORMATION_RADIUS * Math.min(1.35, 0.55 + Math.sqrt(n) / 28);
            const c = Math.cos(formationAngle);
            const s = Math.sin(formationAngle);
            for (let i = 0; i < n; i++) {
                const p = brainPoint(i, n, radius);
                nodes[i].fx = p.x * c - p.z * s;
                nodes[i].fy = p.y;
                nodes[i].fz = p.x * s + p.z * c;
            }
        }, 40);
    }

    function stopFormationSpin() {
        if (formationSpinTimer) {
            clearInterval(formationSpinTimer);
            formationSpinTimer = null;
        }
    }

    // ——— Synaptic brain (neurons + lightning zaps) ———

    let baseLinks = [];
    let adjacency = new Map();
    let hoverNodeId = null;
    let hoverNeighborIds = new Set();
    let simNodeIds = new Set();
    let simLinkKeys = new Set();
    let simTimer = null;
    let simClearTimer = null;
    let ambientPulse = 0;
    let zapBurstUntil = 0;

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

    function pickBrainAtoms(atoms) {
        const hot = atoms.filter(a => a.tier === "hot");
        const warm = atoms.filter(a => a.tier === "warm");
        const cold = atoms.filter(a => a.tier === "cold");
        const out = [];
        const pushCap = arr => {
            for (const a of arr) {
                if (out.length >= BRAIN_MAX_NODES) break;
                out.push(a);
            }
        };
        pushCap(hot);
        pushCap(warm);
        pushCap(cold);
        return out;
    }

    function atomsToNodes(atoms) {
        return atoms.map(atom => {
            const tags = Array.isArray(atom.tags) ? atom.tags : [];
            const tier = atom.tier || "warm";
            let val = tier === "hot" ? 4 : tier === "cold" ? 1.4 : 2.4;
            return {
                id: atom.id,
                title: atom.title,
                source: atom.source,
                source_id: atom.source_id,
                tags,
                tier,
                kind: "atom",
                val
            };
        });
    }

    function buildBrainLinks(nodes) {
        const links = [];
        const seen = new Set();
        const add = (a, b) => {
            if (!a || !b || a === b) return;
            const k = sortedLinkKey(a, b);
            if (seen.has(k)) return;
            seen.add(k);
            links.push({ source: a, target: b });
        };

        const bySourceId = new Map();
        const bySource = new Map();
        for (const n of nodes) {
            if (n.source_id) {
                if (!bySourceId.has(n.source_id)) bySourceId.set(n.source_id, []);
                bySourceId.get(n.source_id).push(n);
            }
            if (!bySource.has(n.source)) bySource.set(n.source, []);
            bySource.get(n.source).push(n);
        }

        for (const group of bySourceId.values()) {
            group.sort((a, b) => a.id.localeCompare(b.id));
            for (let i = 0; i < group.length - 1; i++) add(group[i].id, group[i + 1].id);
        }

        for (const group of bySource.values()) {
            const hub = group.find(g => g.tier === "hot") || group[0];
            if (!hub) continue;
            const cap = Math.min(group.length, 26);
            for (let i = 0; i < cap; i++) {
                if (group[i].id !== hub.id) add(hub.id, group[i].id);
            }
            const hots = group.filter(g => g.tier === "hot").slice(0, 14);
            for (let i = 0; i < hots.length; i++) {
                add(hots[i].id, hots[(i + 1) % hots.length].id);
            }
        }
        return links;
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

    function linkAmbientIndex(link) {
        const k = sortedLinkKey(linkId(link.source), linkId(link.target));
        let h = 0;
        for (let i = 0; i < k.length; i++) h = (h * 31 + k.charCodeAt(i)) >>> 0;
        return (h % 1000) / 1000;
    }

    function inZapBurst() {
        return performance.now() < zapBurstUntil;
    }

    /** Small core + soft halo — neuron, not a fat white orb. */
    function makeNeuronObject(node) {
        const THREE = window.THREE;
        const tier = node.tier || "warm";
        const group = new THREE.Group();

        const coreR = tier === "hot" ? 0.55 : tier === "cold" ? 0.28 : 0.4;
        const haloR = coreR * (tier === "hot" ? 3.2 : tier === "cold" ? 2.2 : 2.6);

        const coreMat = new THREE.MeshBasicMaterial({
            color: tier === "hot" ? 0xeaf2ff : tier === "cold" ? 0x6a7890 : 0xb8c8e0,
            transparent: true,
            opacity: tier === "cold" ? 0.55 : 0.92
        });
        const core = new THREE.Mesh(new THREE.SphereGeometry(coreR, 10, 10), coreMat);
        group.add(core);

        const haloMat = new THREE.MeshBasicMaterial({
            color: tier === "hot" ? 0x9ec4ff : 0x5a6a88,
            transparent: true,
            opacity: tier === "hot" ? 0.16 : 0.07,
            depthWrite: false
        });
        const halo = new THREE.Mesh(new THREE.SphereGeometry(haloR, 12, 12), haloMat);
        group.add(halo);

        group.userData = { coreMat, haloMat, tier, baseHalo: haloMat.opacity };
        return group;
    }

    function applyGraphStyle(g) {
        g.backgroundColor("#000000")
            .showNavInfo(false)
            .nodeThreeObject(node => makeNeuronObject(node))
            .nodeThreeObjectExtend(false)
            .nodeLabel(node => {
                const tags = (node.tags || []).map(t => escapeHtml(t)).join(", ");
                return `
                    <div style="background:rgba(0,0,0,0.92);border:1px solid rgba(160,200,255,0.4);border-radius:10px;padding:12px 14px;font-family:Share Tech Mono,monospace;font-size:0.8rem;color:#fff;box-shadow:0 0 28px rgba(120,170,255,0.2);">
                        <strong style="font-size:0.9rem;">${escapeHtml(node.title)}</strong><br/>
                        <span style="color:#8899aa;">${escapeHtml(node.source)}/${escapeHtml(node.source_id)}</span>
                        ${node.tier ? `<br/><span style="color:#b8d0ff;text-transform:uppercase;letter-spacing:0.08em;font-size:0.68rem;">${escapeHtml(node.tier)}</span>` : ""}
                        ${tags ? `<br/><span style="color:#ccd;">${tags}</span>` : ""}
                    </div>`;
            })
            .linkColor(link => {
                if (linkIsHovered(link)) return "rgba(220, 235, 255, 0.98)";
                if (isHoverActive()) return "rgba(100, 140, 200, 0.04)";
                if (linkIsSim(link) || inZapBurst()) return "rgba(190, 220, 255, 0.85)";
                if (linkAmbientIndex(link) < AMBIENT_PARTICLE_LINKS) return "rgba(140, 180, 255, 0.28)";
                return "rgba(90, 120, 170, 0.12)";
            })
            .linkWidth(link => {
                if (linkIsHovered(link)) return 1.7;
                if (linkIsSim(link)) return 1.35;
                if (inZapBurst() && linkAmbientIndex(link) < 0.35) return 0.9;
                if (linkAmbientIndex(link) < AMBIENT_PARTICLE_LINKS) return 0.45;
                return 0.22;
            })
            .linkOpacity(0.75)
            .linkDirectionalParticles(link => {
                if (linkIsHovered(link)) return 7;
                if (linkIsSim(link)) return 5;
                if (inZapBurst() && linkAmbientIndex(link) < 0.4) return 3;
                if (linkAmbientIndex(link) < AMBIENT_PARTICLE_LINKS) return 2;
                return 0;
            })
            .linkDirectionalParticleSpeed(link => {
                if (linkIsHovered(link) || linkIsSim(link)) return 0.012;
                if (inZapBurst()) return 0.01;
                return 0.0045;
            })
            .linkDirectionalParticleWidth(link => (linkIsSim(link) || linkIsHovered(link) ? 2.4 : 1.6))
            .linkDirectionalParticleColor(link => {
                if (linkIsSim(link) || linkIsHovered(link)) return "#eaf2ff";
                return "#00d8e8"; // synaptic cyan (neural patch accent)
            })
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
                if (atom) selectAtom(atom, { scrollList: true });
            });

        const charge = g.d3Force("charge");
        if (charge) charge.strength(-38);
        const linkF = g.d3Force("link");
        if (linkF) linkF.distance(40);
    }

    function refreshGraphPaint() {
        if (!Graph) return;
        Graph.linkColor(Graph.linkColor());
        Graph.linkWidth(Graph.linkWidth());
        Graph.linkDirectionalParticles(Graph.linkDirectionalParticles());
        Graph.linkDirectionalParticleSpeed(Graph.linkDirectionalParticleSpeed());
        Graph.linkDirectionalParticleColor(Graph.linkDirectionalParticleColor());

        // Pulse neuron halos
        const nodes = Graph.graphData().nodes || [];
        for (const n of nodes) {
            const obj = n.__threeObj;
            if (!obj || !obj.userData || !obj.userData.haloMat) continue;
            const { haloMat, coreMat, tier, baseHalo } = obj.userData;
            const selected = activeAtom && n.id === activeAtom.id;
            const hovered = isHoverActive() && n.id === hoverNodeId;
            const neigh = isHoverActive() && hoverNeighborIds.has(n.id);
            const firing = simNodeIds.has(n.id);

            let halo = baseHalo * (1 + 0.35 * Math.sin(ambientPulse + (n.val || 0)));
            let coreOp = tier === "cold" ? 0.55 : 0.92;
            if (selected || hovered) {
                halo = Math.min(0.42, halo + 0.22);
                coreOp = 1;
                if (coreMat) coreMat.color.setHex(0xffffff);
            } else if (neigh || firing) {
                halo = Math.min(0.32, halo + 0.14);
                if (coreMat) coreMat.color.setHex(0xeaf2ff);
            } else if (isHoverActive()) {
                halo *= 0.35;
                coreOp *= 0.35;
                if (coreMat) {
                    coreMat.color.setHex(tier === "hot" ? 0x4a5870 : 0x2a3040);
                }
            } else if (coreMat) {
                coreMat.color.setHex(tier === "hot" ? 0xeaf2ff : tier === "cold" ? 0x6a7890 : 0xb8c8e0);
            }
            haloMat.opacity = halo;
            if (coreMat) coreMat.opacity = coreOp;
        }
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
        const hops = 4 + Math.floor(Math.random() * Math.max(1, maxHops - 3));
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
        // Lightning burst: one primary bolt + brief ambient shimmer
        const walk = randomWalkPath(8);
        simNodeIds = new Set(walk.nodes);
        simLinkKeys = new Set(walk.linkKeys);
        zapBurstUntil = performance.now() + 420;
        refreshGraphPaint();

        const holdMs = 900 + Math.floor(Math.random() * 1400);
        if (simClearTimer) clearTimeout(simClearTimer);
        simClearTimer = setTimeout(() => {
            clearSimOverlay();
            scheduleNextSim();
        }, holdMs);
    }

    function scheduleNextSim() {
        if (simTimer) clearTimeout(simTimer);
        // Storm cadence — frequent but irregular
        const delay = 900 + Math.floor(Math.random() * 2200);
        simTimer = setTimeout(pulseSimActivity, delay);
    }

    function startSimActivity() {
        if (simTimer) clearTimeout(simTimer);
        simTimer = setTimeout(pulseSimActivity, 700);
    }

    function stageSize() {
        if (!stage) return { w: 800, h: 560 };
        return {
            w: stage.clientWidth || 800,
            h: stage.clientHeight || 560
        };
    }

    function focusCameraOnAtom(atom) {
        if (!Graph || !atom) return;
        const nodes = Graph.graphData().nodes || [];
        const node = nodes.find(n => n.id === atom.id);
        if (!node || node.x == null) return;
        const dist = 160;
        const cam = Graph.cameraPosition();
        const len = Math.hypot(cam.x || 1, cam.y || 1, cam.z || 1) || 1;
        Graph.cameraPosition(
            {
                x: node.x + (cam.x / len) * dist,
                y: node.y + (cam.y / len) * dist,
                z: node.z + (cam.z / len) * dist
            },
            { x: node.x, y: node.y, z: node.z },
            850
        );
    }

    function updateMainBrain(atoms) {
        if (!stage || typeof ForceGraph3D === "undefined" || typeof THREE === "undefined") return;
        const picked = pickBrainAtoms(atoms);
        const nodes = atomsToNodes(picked);
        const sig = nodes.map(n => n.id).sort().join("|").slice(0, 2000);
        baseLinks = buildBrainLinks(nodes);
        rebuildAdjacency(baseLinks);
        clearSimOverlay();

        const { w, h } = stageSize();

        if (!Graph) {
            Graph = ForceGraph3D()(stage);
            applyGraphStyle(Graph);
            Graph.width(w).height(h);
            Graph.graphData({ nodes, links: baseLinks.slice() });
            ensureStarfield(Graph);

            const controls = Graph.controls();
            if (controls) {
                controls.autoRotate = true;
                controls.autoRotateSpeed = 0.4;
                controls.enableDamping = true;
                controls.dampingFactor = 0.07;
            }

            window.addEventListener("resize", () => {
                if (!Graph || !stage) return;
                const s = stageSize();
                Graph.width(s.w).height(s.h);
            });

            setInterval(() => {
                ambientPulse += 0.1;
                if (Graph) refreshGraphPaint();
            }, 70);

            stage.addEventListener("pointerdown", () => {
                const c = Graph && Graph.controls();
                if (c) c.autoRotate = false;
            });
            stage.addEventListener("pointerleave", () => {
                const c = Graph && Graph.controls();
                if (c) c.autoRotate = true;
            });

            setTimeout(() => {
                try { Graph.zoomToFit(1000, 70); } catch (_) {}
            }, 1000);
            startSimActivity();
            graphBuiltForSig = sig;
            if (layoutMode !== "force") {
                applyFormationLayout(layoutMode, { animate: false });
                startFormationSpin();
            }
            return;
        }

        if (sig !== graphBuiltForSig) {
            Graph.graphData({ nodes, links: baseLinks.slice() });
            graphBuiltForSig = sig;
            setTimeout(() => {
                try { Graph.zoomToFit(850, 60); } catch (_) {}
            }, 500);
        } else {
            refreshGraphPaint();
        }
        startSimActivity();
        if (layoutMode !== "force") applyFormationLayout(layoutMode, { animate: false });
        if (activeAtom) setTimeout(() => focusCameraOnAtom(activeAtom), 650);
    }

    let starfieldAdded = false;

    function ensureStarfield(g) {
        if (starfieldAdded || !g || typeof THREE === "undefined") return;
        const scene = typeof g.scene === "function" ? g.scene() : null;
        if (!scene) return;
        const COUNT = 900;
        const pos = new Float32Array(COUNT * 3);
        for (let i = 0; i < COUNT; i++) {
            const r = 220 + Math.random() * 280;
            const theta = Math.random() * Math.PI * 2;
            const phi = Math.acos(1 - 2 * Math.random());
            pos[i * 3] = r * Math.sin(phi) * Math.cos(theta);
            pos[i * 3 + 1] = r * Math.cos(phi);
            pos[i * 3 + 2] = r * Math.sin(phi) * Math.sin(theta);
        }
        const geo = new THREE.BufferGeometry();
        geo.setAttribute("position", new THREE.BufferAttribute(pos, 3));
        const mat = new THREE.PointsMaterial({
            size: 1.1,
            color: 0x8fa8bd,
            transparent: true,
            opacity: 0.55,
            depthWrite: false,
            sizeAttenuation: true
        });
        scene.add(new THREE.Points(geo, mat));
        starfieldAdded = true;
    }

    // ——— Memory stream + Ask (neural patch, correct API contracts) ———

    const streamFeed = document.getElementById("stream-feed");
    const liveToggle = document.getElementById("live-toggle");
    const askInput = document.getElementById("ask-input");
    const askOutput = document.getElementById("ask-output");
    const MAX_STREAM = 40;
    let liveStream = true;
    let activitySince = 0;

    function pushStream(kind, label, body) {
        if (!streamFeed) return;
        const el = document.createElement("div");
        el.className = `stream-item kind-${kind}`;
        el.innerHTML = `
            <div class="s-meta"><span>${escapeHtml(label)}</span><span>${new Date().toLocaleTimeString()}</span></div>
            <div class="s-body">${escapeHtml(body)}</div>`;
        streamFeed.prepend(el);
        while (streamFeed.children.length > MAX_STREAM) streamFeed.lastChild.remove();
    }

    async function pollActivity() {
        if (!liveStream) return;
        try {
            const r = await fetch(`/api/activity?since=${activitySince}`);
            if (!r.ok) return;
            const j = await r.json();
            const events = Array.isArray(j.events) ? j.events : [];
            if (typeof j.next_seq === "number") activitySince = j.next_seq;
            for (const ev of events) {
                const tool = ev.tool || "memory";
                const kind = /ask|think/i.test(tool) ? "thought"
                    : /search|cite|mcp|tool/i.test(tool) ? "tool" : "memory";
                const body = [ev.query, ev.detail].filter(Boolean).join(" — ")
                    || (ev.atom_ids && ev.atom_ids.length ? `${ev.atom_ids.length} atoms` : tool);
                pushStream(kind, tool, body);
            }
        } catch (_) { /* daemon offline */ }
    }

    if (liveToggle) {
        liveToggle.addEventListener("click", () => {
            liveStream = !liveStream;
            liveToggle.setAttribute("aria-pressed", String(liveStream));
            liveToggle.textContent = liveStream ? "❚❚" : "▶";
            liveToggle.title = liveStream ? "Pause live stream" : "Resume live stream";
        });
    }

    if (askInput) {
        askInput.addEventListener("keydown", async e => {
            if (e.key !== "Enter" || !askInput.value.trim()) return;
            const q = askInput.value.trim();
            if (askOutput) askOutput.textContent = "thinking…";
            pushStream("thought", "ask", q);
            try {
                const r = await fetch(`/api/ask?question=${encodeURIComponent(q)}`);
                const j = await r.json();
                if (!r.ok) {
                    if (askOutput) askOutput.textContent = j.error || j.message || `ask failed (${r.status})`;
                    return;
                }
                const text = j.answer || j.response || JSON.stringify(j).slice(0, 400);
                if (askOutput) askOutput.textContent = text;
                pushStream("thought", "answer", String(text).slice(0, 160));
            } catch (_) {
                if (askOutput) askOutput.textContent = "ask failed — daemon offline?";
            }
        });
    }

    applyViewMode();
    loadDashboard().then(() => {
        pushStream("memory", "lattice", "synaptic brain online");
    });
    setInterval(pollActivity, 2500);
});
