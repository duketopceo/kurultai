// brain.js — wired to the local kurultai daemon (served at GET /ui).
// Absolute /api/* paths hit the same origin when loaded from the daemon.
// Overview: 2D atlas map (Three.js Points). Focus: 3D synapse for neighborhood only.
document.addEventListener("DOMContentLoaded", () => {
    const listContainer = document.getElementById("atoms-list-container");
    const inspector = document.getElementById("atom-inspector");
    const searchInput = document.getElementById("brain-search");
    const graphContainer = document.getElementById("3d-synapse-graph");
    const atlasContainer = document.getElementById("brain-atlas-map");
    const atlasHud = document.getElementById("atlas-hud");
    const atlasTooltip = document.getElementById("atlas-tooltip");

    const ATOMS_LIMIT = 200;
    const FOCUS_MAX_NODES = 120;
    const LAYOUT_CACHE_KEY = "kurultai-brain-atlas-layout-v1";

    let currentAtoms = [];
    let activeAtom = null;
    let Graph = null;
    let layoutById = new Map(); // id -> {x, y}

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
            const r = await fetch(`/api/atoms?limit=${ATOMS_LIMIT}`);
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
            const layoutX = parseFloat(meta.layout_x);
            const layoutY = parseFloat(meta.layout_y);
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
                score: r.score,
                layout_x: Number.isFinite(layoutX) ? layoutX : null,
                layout_y: Number.isFinite(layoutY) ? layoutY : null
            };
        });

        const sources = new Set(currentAtoms.map(a => a.source).filter(Boolean));
        const sourcesEl = document.getElementById("stat-sources");
        if (sourcesEl) sourcesEl.textContent = sources.size || "—";

        layoutById = ensureAtlasLayout(currentAtoms);
        renderAtomsList(currentAtoms);
        updateAtlasMap(currentAtoms);

        if (currentAtoms.length > 0) {
            const keep = activeAtom && currentAtoms.find(a => a.id === activeAtom.id);
            activeAtom = keep || currentAtoms[0];
            renderAtomsList(currentAtoms);
            inspectAtom(activeAtom);
            updateFocusSynapse(activeAtom);
            highlightAtlasSelection(activeAtom.id);
        } else {
            activeAtom = null;
            updateFocusSynapse(null);
        }
    }

    function selectAtom(atom, { scrollList } = { scrollList: false }) {
        if (!atom) return;
        activeAtom = atom;
        renderAtomsList(currentAtoms);
        inspectAtom(atom);
        updateFocusSynapse(atom);
        highlightAtlasSelection(atom.id);
        if (scrollList) {
            const el = listContainer.querySelector(".atom-item.active");
            if (el) el.scrollIntoView({ block: "nearest" });
        }
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
            div.addEventListener("click", () => selectAtom(atom));
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

    // ——— Atlas layout (metadata → cache → force) ———

    function layoutFingerprint(atoms) {
        return atoms.map(a => a.id).slice().sort().join("|");
    }

    function loadLayoutCache(fp) {
        try {
            const raw = localStorage.getItem(LAYOUT_CACHE_KEY);
            if (!raw) return null;
            const parsed = JSON.parse(raw);
            if (!parsed || parsed.fp !== fp || !parsed.pos) return null;
            return parsed.pos;
        } catch (_) {
            return null;
        }
    }

    function saveLayoutCache(fp, pos) {
        try {
            localStorage.setItem(LAYOUT_CACHE_KEY, JSON.stringify({ fp, pos, t: Date.now() }));
        } catch (_) {
            // quota / private mode — ignore
        }
    }

    function hashSeed(str) {
        let h = 2166136261;
        for (let i = 0; i < str.length; i++) {
            h ^= str.charCodeAt(i);
            h = Math.imul(h, 16777619);
        }
        return (h >>> 0) / 4294967296;
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

    function runForceLayout2d(atoms, iterations) {
        const n = atoms.length;
        if (n === 0) return {};
        const pos = {};
        atoms.forEach(a => {
            if (a.layout_x != null && a.layout_y != null) {
                pos[a.id] = { x: a.layout_x, y: a.layout_y };
            } else {
                const t = hashSeed(a.id) * Math.PI * 2;
                const r = 40 + hashSeed(a.id + ":r") * 120;
                pos[a.id] = { x: Math.cos(t) * r, y: Math.sin(t) * r };
            }
        });

        const links = buildStructuralLinks(atoms);
        const ids = atoms.map(a => a.id);
        const kRep = 900;
        const kSpring = 0.035;
        const ideal = 55;
        const damp = 0.82;

        const vx = {};
        const vy = {};
        ids.forEach(id => { vx[id] = 0; vy[id] = 0; });

        for (let iter = 0; iter < iterations; iter++) {
            for (let i = 0; i < n; i++) {
                for (let j = i + 1; j < n; j++) {
                    const a = ids[i];
                    const b = ids[j];
                    let dx = pos[a].x - pos[b].x;
                    let dy = pos[a].y - pos[b].y;
                    let dist2 = dx * dx + dy * dy + 0.01;
                    const dist = Math.sqrt(dist2);
                    const force = kRep / dist2;
                    const fx = (dx / dist) * force;
                    const fy = (dy / dist) * force;
                    vx[a] += fx;
                    vy[a] += fy;
                    vx[b] -= fx;
                    vy[b] -= fy;
                }
            }
            for (const link of links) {
                const a = link.source;
                const b = link.target;
                let dx = pos[b].x - pos[a].x;
                let dy = pos[b].y - pos[a].y;
                const dist = Math.sqrt(dx * dx + dy * dy) + 0.01;
                const stretch = dist - ideal;
                const fx = (dx / dist) * stretch * kSpring;
                const fy = (dy / dist) * stretch * kSpring;
                vx[a] += fx;
                vy[a] += fy;
                vx[b] -= fx;
                vy[b] -= fy;
            }
            // mild centering
            for (const id of ids) {
                vx[id] -= pos[id].x * 0.002;
                vy[id] -= pos[id].y * 0.002;
                vx[id] *= damp;
                vy[id] *= damp;
                pos[id].x += vx[id];
                pos[id].y += vy[id];
            }
        }

        // Normalize to roughly [-1, 1]
        let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
        for (const id of ids) {
            minX = Math.min(minX, pos[id].x);
            maxX = Math.max(maxX, pos[id].x);
            minY = Math.min(minY, pos[id].y);
            maxY = Math.max(maxY, pos[id].y);
        }
        const spanX = Math.max(1e-6, maxX - minX);
        const spanY = Math.max(1e-6, maxY - minY);
        const out = {};
        for (const id of ids) {
            out[id] = {
                x: ((pos[id].x - minX) / spanX) * 2 - 1,
                y: ((pos[id].y - minY) / spanY) * 2 - 1
            };
        }
        return out;
    }

    function ensureAtlasLayout(atoms) {
        const map = new Map();
        if (!atoms.length) return map;

        const allHaveMeta = atoms.every(a => a.layout_x != null && a.layout_y != null);
        if (allHaveMeta) {
            atoms.forEach(a => map.set(a.id, { x: a.layout_x, y: a.layout_y }));
            if (atlasHud) atlasHud.textContent = `${atoms.length} pts · layout from metadata · scroll/drag/click`;
            return map;
        }

        const fp = layoutFingerprint(atoms);
        const cached = loadLayoutCache(fp);
        if (cached) {
            for (const a of atoms) {
                if (cached[a.id]) map.set(a.id, cached[a.id]);
            }
            if (map.size === atoms.length) {
                if (atlasHud) atlasHud.textContent = `${atoms.length} pts · cached force layout · scroll/drag/click`;
                return map;
            }
        }

        const iters = Math.min(120, 40 + Math.floor(atoms.length / 3));
        const computed = runForceLayout2d(atoms, iters);
        saveLayoutCache(fp, computed);
        for (const a of atoms) {
            if (computed[a.id]) map.set(a.id, computed[a.id]);
        }
        if (atlasHud) atlasHud.textContent = `${atoms.length} pts · force layout · scroll/drag/click`;
        return map;
    }

    // ——— Overview atlas (Three.js Points) ———

    let atlasState = null;

    function initAtlas() {
        if (!atlasContainer || typeof THREE === "undefined") return null;

        const width = atlasContainer.clientWidth || 640;
        const height = atlasContainer.clientHeight || 560;

        const scene = new THREE.Scene();
        scene.background = new THREE.Color(0x000000);

        const camera = new THREE.OrthographicCamera(-1.4, 1.4, 1.4, -1.4, 0.1, 10);
        camera.position.z = 2;

        const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: false });
        renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
        renderer.setSize(width, height);
        atlasContainer.appendChild(renderer.domElement);

        const geometry = new THREE.BufferGeometry();
        const material = new THREE.PointsMaterial({
            size: 7,
            sizeAttenuation: false,
            vertexColors: true,
            transparent: true,
            opacity: 0.95
        });
        const points = new THREE.Points(geometry, material);
        scene.add(points);

        const state = {
            scene,
            camera,
            renderer,
            geometry,
            material,
            points,
            atomIds: [],
            selectedId: null,
            hoverId: null,
            view: { cx: 0, cy: 0, zoom: 1 },
            pointerDown: false,
            didDrag: false,
            lastX: 0,
            lastY: 0,
            needsRender: true
        };

        const canvas = renderer.domElement;

        canvas.addEventListener("wheel", (e) => {
            e.preventDefault();
            const factor = e.deltaY > 0 ? 0.9 : 1.1;
            state.view.zoom = Math.min(8, Math.max(0.35, state.view.zoom * factor));
            applyAtlasCamera(state);
            state.needsRender = true;
        }, { passive: false });

        canvas.addEventListener("pointerdown", (e) => {
            state.pointerDown = true;
            state.didDrag = false;
            state.lastX = e.clientX;
            state.lastY = e.clientY;
            canvas.setPointerCapture(e.pointerId);
        });

        canvas.addEventListener("pointermove", (e) => {
            if (state.pointerDown) {
                const moveX = e.clientX - state.lastX;
                const moveY = e.clientY - state.lastY;
                if (!state.didDrag && (Math.abs(moveX) > 3 || Math.abs(moveY) > 3)) {
                    state.didDrag = true;
                }
                if (state.didDrag) {
                    const rect = canvas.getBoundingClientRect();
                    const worldW = (state.camera.right - state.camera.left);
                    const worldH = (state.camera.top - state.camera.bottom);
                    const dx = moveX / rect.width * worldW;
                    const dy = -moveY / rect.height * worldH;
                    state.view.cx -= dx;
                    state.view.cy -= dy;
                    state.lastX = e.clientX;
                    state.lastY = e.clientY;
                    applyAtlasCamera(state);
                    state.needsRender = true;
                }
                return;
            }
            const hit = pickAtlasPoint(state, e);
            if (hit !== state.hoverId) {
                state.hoverId = hit;
                updateAtlasTooltip(state, e, hit);
                paintAtlasColors(state);
                state.needsRender = true;
            } else if (hit && atlasTooltip) {
                atlasTooltip.style.left = `${e.offsetX + 14}px`;
                atlasTooltip.style.top = `${e.offsetY + 14}px`;
            }
        });

        canvas.addEventListener("pointerup", (e) => {
            const clicked = state.pointerDown && !state.didDrag;
            state.pointerDown = false;
            try { canvas.releasePointerCapture(e.pointerId); } catch (_) {}
            if (!clicked) return;
            const hit = pickAtlasPoint(state, e);
            if (hit) {
                const atom = currentAtoms.find(a => a.id === hit);
                if (atom) selectAtom(atom, { scrollList: true });
            }
        });

        canvas.addEventListener("pointerleave", () => {
            state.hoverId = null;
            if (atlasTooltip) atlasTooltip.style.display = "none";
            paintAtlasColors(state);
            state.needsRender = true;
        });

        window.addEventListener("resize", () => {
            if (!atlasContainer || !state) return;
            const w = atlasContainer.clientWidth;
            const h = atlasContainer.clientHeight;
            state.renderer.setSize(w, h);
            state.needsRender = true;
        });

        function loop() {
            requestAnimationFrame(loop);
            if (state.needsRender) {
                state.renderer.render(state.scene, state.camera);
                state.needsRender = false;
            }
        }
        loop();

        return state;
    }

    function applyAtlasCamera(state) {
        const z = state.view.zoom;
        const half = 1.4 / z;
        state.camera.left = state.view.cx - half;
        state.camera.right = state.view.cx + half;
        state.camera.top = state.view.cy + half;
        state.camera.bottom = state.view.cy - half;
        state.camera.updateProjectionMatrix();
    }

    function ndcFromEvent(state, e) {
        const rect = state.renderer.domElement.getBoundingClientRect();
        const nx = ((e.clientX - rect.left) / rect.width) * 2 - 1;
        const ny = -(((e.clientY - rect.top) / rect.height) * 2 - 1);
        const x = state.view.cx + nx * (1.4 / state.view.zoom);
        const y = state.view.cy + ny * (1.4 / state.view.zoom);
        return { x, y };
    }

    function pickAtlasPoint(state, e) {
        if (!state.atomIds.length) return null;
        const { x, y } = ndcFromEvent(state, e);
        const pos = state.geometry.attributes.position;
        const thresh = 0.06 / state.view.zoom;
        let best = null;
        let bestD = thresh * thresh;
        for (let i = 0; i < state.atomIds.length; i++) {
            const dx = pos.getX(i) - x;
            const dy = pos.getY(i) - y;
            const d = dx * dx + dy * dy;
            if (d < bestD) {
                bestD = d;
                best = state.atomIds[i];
            }
        }
        return best;
    }

    function updateAtlasTooltip(state, e, atomId) {
        if (!atlasTooltip) return;
        if (!atomId) {
            atlasTooltip.style.display = "none";
            return;
        }
        const atom = currentAtoms.find(a => a.id === atomId);
        if (!atom) {
            atlasTooltip.style.display = "none";
            return;
        }
        atlasTooltip.innerHTML = `<strong>${escapeHtml(atom.title)}</strong><br/><span style="color:#888">${escapeHtml(atom.source)}/${escapeHtml(atom.source_id)}</span>`;
        atlasTooltip.style.display = "block";
        atlasTooltip.style.left = `${e.offsetX + 14}px`;
        atlasTooltip.style.top = `${e.offsetY + 14}px`;
    }

    function paintAtlasColors(state) {
        const colors = state.geometry.attributes.color;
        if (!colors) return;
        for (let i = 0; i < state.atomIds.length; i++) {
            const id = state.atomIds[i];
            let r = 1, g = 1, b = 1;
            if (state.selectedId && id === state.selectedId) {
                r = 1; g = 1; b = 1;
            } else if (state.hoverId && id === state.hoverId) {
                r = 0.92; g = 0.92; b = 0.92;
            } else if (state.selectedId) {
                r = 0.22; g = 0.22; b = 0.22;
            } else {
                r = 0.85; g = 0.85; b = 0.85;
            }
            colors.setXYZ(i, r, g, b);
        }
        colors.needsUpdate = true;
        state.material.size = state.selectedId ? 8 : 7;
    }

    function highlightAtlasSelection(atomId) {
        if (!atlasState) return;
        atlasState.selectedId = atomId;
        paintAtlasColors(atlasState);
        atlasState.needsRender = true;
    }

    function updateAtlasMap(atoms) {
        if (!atlasContainer || typeof THREE === "undefined") return;
        if (!atlasState) atlasState = initAtlas();
        if (!atlasState) return;

        const ids = [];
        const positions = [];
        const colors = [];

        for (const atom of atoms) {
            const p = layoutById.get(atom.id) || { x: 0, y: 0 };
            ids.push(atom.id);
            positions.push(p.x, p.y, 0);
            colors.push(0.85, 0.85, 0.85);
        }

        atlasState.atomIds = ids;
        atlasState.geometry.setAttribute("position", new THREE.Float32BufferAttribute(positions, 3));
        atlasState.geometry.setAttribute("color", new THREE.Float32BufferAttribute(colors, 3));
        atlasState.geometry.computeBoundingSphere();
        paintAtlasColors(atlasState);
        atlasState.needsRender = true;
    }

    // ——— Focus 3D synapse (neighborhood only) ———

    let baseLinks = [];
    let adjacency = new Map();

    let hoverNodeId = null;
    let hoverNeighborIds = new Set();

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

    function collectNeighborhood(focusAtom, allAtoms) {
        if (!focusAtom) return [];
        const byId = new Map(allAtoms.map(a => [a.id, a]));
        const globalLinks = buildStructuralLinks(allAtoms);
        const adj = new Map();
        for (const link of globalLinks) {
            if (!adj.has(link.source)) adj.set(link.source, new Set());
            if (!adj.has(link.target)) adj.set(link.target, new Set());
            adj.get(link.source).add(link.target);
            adj.get(link.target).add(link.source);
        }

        const selected = new Set([focusAtom.id]);
        const hop1 = [...(adj.get(focusAtom.id) || [])];
        hop1.forEach(id => selected.add(id));

        if (selected.size < 8) {
            for (const id of hop1) {
                for (const n2 of (adj.get(id) || [])) {
                    selected.add(n2);
                    if (selected.size >= FOCUS_MAX_NODES) break;
                }
                if (selected.size >= FOCUS_MAX_NODES) break;
            }
        }

        // Always include focus; cap size
        const out = [];
        out.push(focusAtom);
        for (const id of selected) {
            if (id === focusAtom.id) continue;
            const a = byId.get(id);
            if (a) out.push(a);
            if (out.length >= FOCUS_MAX_NODES) break;
        }

        // If isolated, still show focus alone (plus a few same-source if any)
        if (out.length === 1) {
            for (const a of allAtoms) {
                if (a.id === focusAtom.id) continue;
                if (a.source === focusAtom.source) out.push(a);
                if (out.length >= Math.min(12, FOCUS_MAX_NODES)) break;
            }
        }
        return out;
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
                if (activeAtom && node.id === activeAtom.id) return "#ffffff";
                if (isHoverActive()) {
                    if (node.id === hoverNodeId) return "#ffffff";
                    if (hoverNeighborIds.has(node.id)) return "#e8e8e8";
                    return "#1a1a1a";
                }
                if (simNodeIds.has(node.id)) return "#f5f5f5";
                return "#cccccc";
            })
            .nodeRelSize(4)
            .nodeVal(node => {
                if (activeAtom && node.id === activeAtom.id) return 12;
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
                if (atom) selectAtom(atom, { scrollList: true });
            });
        const charge = g.d3Force("charge");
        if (charge) charge.strength(-28);
        const linkF = g.d3Force("link");
        if (linkF) linkF.distance(36);
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
        const delay = 2800 + Math.floor(Math.random() * 5200);
        simTimer = setTimeout(pulseSimActivity, delay);
    }

    function startSimActivity() {
        if (simTimer) clearTimeout(simTimer);
        simTimer = setTimeout(pulseSimActivity, 2200);
    }

    function clearFocusEmpty() {
        const empty = document.getElementById("focus-empty");
        if (empty) empty.remove();
    }

    function showFocusEmpty() {
        if (!graphContainer) return;
        if (Graph) {
            Graph.graphData({ nodes: [], links: [] });
        }
        if (!document.getElementById("focus-empty")) {
            const div = document.createElement("div");
            div.id = "focus-empty";
            div.className = "focus-empty";
            div.textContent = "Select an atom to load its neighborhood hologram";
            graphContainer.appendChild(div);
        }
    }

    function updateFocusSynapse(focusAtom) {
        if (!graphContainer || typeof ForceGraph3D === "undefined") return;

        if (!focusAtom) {
            showFocusEmpty();
            return;
        }

        clearFocusEmpty();
        const neighborhood = collectNeighborhood(focusAtom, currentAtoms);
        const nodes = atomsToNodes(neighborhood);
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
            setTimeout(() => { try { Graph.zoomToFit(800, 60); } catch (e) {} }, 900);
            startSimActivity();
        } else {
            Graph.graphData({ nodes, links: baseLinks.slice() });
            refreshGraphPaint();
            setTimeout(() => { try { Graph.zoomToFit(600, 50); } catch (e) {} }, 400);
            startSimActivity();
        }
    }

    // Kick off
    applyViewMode();
    loadDashboard();
});
