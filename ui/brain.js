(() => {
  "use strict";
  const $ = (id) => document.getElementById(id);

  const state = {
    atoms: [], visible: [], selected: null,
    query: "", since: 0, live: true,
    playing: false, timer: null, graph: null,
    maxMode: false, atomTotal: 0,
    loadGen: 0, loadAbort: null
  };

  const DEFAULT_ATOM_LIMIT = 450;
  const MAX_GRAPH_LIMIT = 10000;
  const MAX_RENDER_CAP = 2500;
  const GHOST_PREVIEW_CAP = 2500;
  const GRAPH_TIERS = ["hot", "warm", "cold"];

  const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  const elements = {
    status:         $("daemon-status"),
    caption:        $("brain-caption"),
    canvas:         $("brain-canvas"),
    fallback:       $("lattice-fallback"),
    stage:          $("brain-stage"),
    tooltip:        $("node-tooltip"),
    inspector:      $("node-inspector"),
    activity:       $("activity-stream"),
    streamToggle:   $("stream-toggle"),
    search:         $("brain-search"),
    dropdown:       $("search-dropdown"),
    timeline:       $("timeline-range"),
    timelineOutput: $("timeline-output"),
    play:           $("timeline-play"),
    askForm:        $("ask-form"),
    askInput:       $("ask-input"),
    askOutput:      $("ask-output"),
    theme:          $("theme-toggle"),
    maxToggle:      $("max-toggle"),
    layoutToggle:   $("layout-toggle")
  };

  /* ── Utilities ─────────────────────────────────────────────── */
  function text(value, fallback = "—") {
    return value == null || value === "" ? fallback : String(value);
  }
  function dateValue(atom) {
    const d = Date.parse(atom.indexed_at || atom.source_updated_at || "");
    return Number.isFinite(d) ? d : 0;
  }
  function normalize(result) {
    const atom = result && result.atom ? result.atom : result || {};
    const meta = atom.metadata || {};
    return {
      id:             text(atom.id, crypto.randomUUID ? crypto.randomUUID() : Math.random().toString(36).slice(2)),
      title:          text(atom.title, "Untitled memory"),
      summary:        text(atom.summary || atom.content || meta.summary, "No local summary available."),
      source:         text(atom.source),
      sourceId:       text(atom.source_id),
      tags:           Array.isArray(atom.tags) ? atom.tags : Array.isArray(meta.tags) ? meta.tags : [],
      file:           atom.file_path || meta.file_path || "",
      indexed_at:     atom.indexed_at || meta.indexed_at || "",
      last_accessed_at: atom.last_accessed_at || meta.last_accessed_at || "",
      score:          Number(result && result.score) || 0,
      tier:           atom.tier || meta.tier || "warm",
      lean:           false
    };
  }

  function normalizeGraphNode(node) {
    const n = node || {};
    return {
      id:             text(n.id, crypto.randomUUID ? crypto.randomUUID() : Math.random().toString(36).slice(2)),
      title:          text(n.title, "Untitled memory"),
      summary:        text(n.summary, n.tier === "hot" ? "No local summary available." : "Lean stub — select to load details."),
      source:         text(n.source),
      sourceId:       text(n.source_id),
      tags:           Array.isArray(n.tags) ? n.tags : [],
      file:           "",
      indexed_at:     n.indexed_at || "",
      last_accessed_at: n.last_accessed_at || "",
      score:          n.tier === "hot" ? 1 : n.tier === "warm" ? 0.55 : 0.25,
      tier:           text(n.tier, "warm"),
      lean:           true
    };
  }

  async function getJson(path, signal) {
    const r = await fetch(path, {
      headers: { Accept: "application/json" },
      signal
    });
    if (!r.ok) throw new Error(`${path} failed (${r.status})`);
    return r.json();
  }
  function setStatus(value, online) {
    elements.status.textContent = value;
    document.querySelector(".status-dot").style.background =
      online ? "var(--electric-dim)" : "var(--danger)";
  }

  /* ── Theme ──────────────────────────────────────────────────── */
  function initialTheme() {
    const saved = localStorage.getItem("kurultai-theme");
    if (saved === "light" || saved === "dark") return saved;
    return window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
  }
  function applyTheme(theme) {
    document.documentElement.dataset.theme = theme;
    const light = theme === "light";
    elements.theme.setAttribute("aria-pressed", String(light));
    elements.theme.setAttribute("aria-label", `Switch to ${light ? "dark" : "light"} theme`);
    localStorage.setItem("kurultai-theme", theme);
    if (state.graph) state.graph.recolor();
  }

  /* ── Graph relationships ────────────────────────────────────── */
  function buildRelationships(atoms) {
    const links = new Map();
    atoms.forEach((a, i) => atoms.slice(i + 1).forEach((b) => {
      const shared = a.tags.filter((tag) => b.tags.includes(tag));
      if (shared.length || (a.source && a.source === b.source)) {
        links.set([a.id, b.id].sort().join("|"), {
          a: a.id, b: b.id,
          strength: shared.length + (a.source === b.source ? 1 : 0)
        });
      }
    }));
    return [...links.values()];
  }

  function filteredAtoms() {
    const horizon = Number(elements.timeline.value) / 100;
    if (horizon >= 1 || !state.atoms.length) return state.atoms;
    const dates = state.atoms.map(dateValue).filter(Boolean);
    const min   = Math.min(...dates, Date.now());
    const cutoff = min + (Date.now() - min) * (1 - horizon);
    return state.atoms.filter((atom) => !dateValue(atom) || dateValue(atom) >= cutoff);
  }

  function refreshLattice() {
    state.visible = filteredAtoms();
    const renderCap = state.maxMode ? MAX_RENDER_CAP : DEFAULT_ATOM_LIMIT;
    if (state.graph) state.graph.update(state.visible, renderCap);
    const loaded = state.visible.length;
    const shown = Math.min(loaded, renderCap);
    const total = Math.max(state.atomTotal || 0, state.atoms.length);
    const mode = state.maxMode ? "max" : "standard";
    if (state.maxMode && loaded > shown) {
      elements.caption.textContent =
        `${shown} shown · ${loaded} loaded (cap ${renderCap}) · ${mode}`;
    } else if (total > loaded && !state.maxMode) {
      elements.caption.textContent =
        `${loaded} of ${total} memories · ${mode} · hover max for ghost preview`;
    } else {
      elements.caption.textContent =
        `${shown} memories · ${mode} · hover to trace connections`;
    }
  }

  function applyMaxToggleUi() {
    if (!elements.maxToggle) return;
    elements.maxToggle.setAttribute("aria-pressed", String(state.maxMode));
    elements.maxToggle.textContent = "max";
    elements.maxToggle.setAttribute(
      "aria-label",
      state.maxMode
        ? "Exit max mode and reload the standard memory subset"
        : "Load up to 10,000 memories via tiered graph. Hover to preview as ghosts."
    );
  }

  async function setMaxMode(next) {
    if (state.maxMode === next) return;
    state.maxMode = next;
    localStorage.setItem("kurultai-max-mode", next ? "1" : "0");
    applyMaxToggleUi();
    if (state.graph) state.graph.clearGhostPreview();
    if (elements.maxToggle) elements.maxToggle.classList.remove("is-ghosting");
    elements.caption.textContent = next
      ? "loading up to 10,000 memories (hot → warm → cold)…"
      : "loading standard memory subset…";
    await loadAtoms();
  }

  /* ── Inspector ──────────────────────────────────────────────── */
  async function enrichAtomDetails(atom) {
    if (!atom || !atom.lean || !atom.id) return atom;
    try {
      const r = await fetch("/api/touch", {
        method: "POST",
        headers: {
          Accept: "application/json",
          "Content-Type": "application/json"
        },
        body: JSON.stringify({ atom_id: atom.id })
      });
      if (!r.ok) return atom;
      const data = await r.json();
      atom.title = text(data.title, atom.title);
      atom.summary = text(data.summary, atom.summary);
      atom.source = text(data.source, atom.source);
      atom.tags = Array.isArray(data.tags) ? data.tags : atom.tags;
      atom.file = data.file_path || atom.file || "";
      atom.indexed_at = data.indexed_at || atom.indexed_at;
      atom.last_accessed_at = data.last_accessed_at || atom.last_accessed_at;
      atom.lean = false;
      const idx = state.atoms.findIndex((a) => a.id === atom.id);
      if (idx >= 0) state.atoms[idx] = atom;
    } catch (_) { /* keep stub */ }
    return atom;
  }

  async function renderInspector(atom) {
    state.selected = atom;
    if (!atom) {
      elements.inspector.innerHTML = "";
      const p = document.createElement("p");
      p.className = "empty-state";
      p.textContent = "Hover or select a memory node to reveal its place in the lattice.";
      elements.inspector.append(p);
      return;
    }
    if (atom.lean) {
      elements.inspector.innerHTML = "";
      const loading = document.createElement("p");
      loading.className = "empty-state";
      loading.textContent = "Loading memory details…";
      elements.inspector.append(loading);
      atom = await enrichAtomDetails(atom);
      if (state.selected && state.selected.id !== atom.id) return;
    }
    elements.inspector.innerHTML = "";
    const title = document.createElement("h3");
    title.className = "inspector-title";
    title.textContent = atom.title;
    const summary = document.createElement("p");
    summary.className = "inspector-summary";
    summary.textContent = atom.summary;
    const metrics = document.createElement("div");
    metrics.className = "node-metrics";
    const relationCount = buildRelationships(state.visible)
      .filter((link) => link.a === atom.id || link.b === atom.id).length;
    [["weight", atom.score.toFixed(2)], ["recency", atom.last_accessed_at || atom.indexed_at || "unknown"],
     ["relations", relationCount], ["tier", atom.tier]].forEach(([name, value]) => {
      const metric = document.createElement("span");
      metric.className = "metric";
      const label = document.createTextNode(`${name} `);
      const strong = document.createElement("b");
      strong.textContent = value;
      metric.append(label, strong);
      metrics.append(metric);
    });
    elements.inspector.append(title, summary, metrics);
    if (atom.tags.length) {
      const tags = document.createElement("div");
      tags.className = "inspector-tags";
      atom.tags.forEach((tag) => {
        const chip = document.createElement("span");
        chip.className = "tag-chip";
        chip.textContent = `#${tag}`;
        chip.style.background = labelGradientCss(tag);
        tags.append(chip);
      });
      elements.inspector.append(tags);
    }
    if (atom.file) {
      const open = document.createElement("button");
      open.className = "open-button";
      open.type = "button";
      open.textContent = "Open source file ↗";
      open.addEventListener("click", () => openFile(atom.file));
      elements.inspector.append(open);
    }
  }

  async function openFile(file) {
    try {
      const r = await fetch(`/api/open?file=${encodeURIComponent(file)}`);
      if (!r.ok) throw new Error("open failed");
    } catch (_) {
      elements.askOutput.textContent = "Could not ask the daemon to open that file.";
    }
  }

  /* ── Search ─────────────────────────────────────────────────── */
  function renderSearchResults(results) {
    elements.dropdown.innerHTML = "";
    if (!results.length) { elements.dropdown.hidden = true; return; }
    results.slice(0, 6).forEach((atom) => {
      const option = document.createElement("button");
      option.type = "button";
      option.className = "search-result";
      option.setAttribute("role", "option");
      const title = document.createElement("strong");
      title.textContent = atom.title;
      const detail = document.createElement("span");
      detail.textContent = atom.summary;
      option.append(title, detail);
      option.addEventListener("click", () => {
        elements.search.value = atom.title;
        elements.dropdown.hidden = true;
        state.atoms = results;
        refreshLattice();
        renderInspector(atom);
      });
      elements.dropdown.append(option);
    });
    elements.dropdown.hidden = false;
  }

  let searchDelay;
  async function search(query) {
    if (!query.trim()) {
      state.query = "";
      renderSearchResults([]);
      await loadAtoms();
      return;
    }
    try {
      const results = (await getJson(`/api/search?q=${encodeURIComponent(query)}&limit=80`)).map(normalize);
      state.query = query;
      state.atoms = results;
      renderSearchResults(results);
      refreshLattice();
    } catch (_) {
      renderSearchResults([]);
      elements.caption.textContent = "Search is unavailable while the daemon is offline.";
    }
  }

  /* ── Activity ───────────────────────────────────────────────── */
  function addActivity(event) {
    if (elements.activity.querySelector(".empty-state")) elements.activity.innerHTML = "";
    const row = document.createElement("article");
    row.className = "activity-item";
    const meta = document.createElement("div");
    meta.className = "activity-meta";
    const tool = document.createElement("span");
    tool.textContent = text(event.tool, "memory");
    const time = document.createElement("time");
    time.textContent = new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
    meta.append(tool, time);
    const body = document.createElement("p");
    body.textContent = text(event.query || event.detail || (event.atom_ids || []).join(", "), "local activity");
    row.append(meta, body);
    elements.activity.prepend(row);
    while (elements.activity.children.length > 30) elements.activity.lastElementChild.remove();
  }

  async function pollActivity() {
    if (!state.live) return;
    try {
      const data = await getJson(`/api/activity?since=${state.since}`);
      state.since = Number(data.next_seq) || state.since;
      (data.events || []).forEach(addActivity);
    } catch (_) { /* Offline — represented in header. */ }
  }

  /* ── Purple gradient from labels ────────────────────────────── */
  /*
   * Map each atom's primary tag onto a purple-only spectrum so related
   * labels share a hue family while staying black/white/purple compliant.
   */
  const COLOUR = {
    nodeBase:    0xa855f7,   // electric purple
    nodeHot:     0xffffff,   // white on hover
    nodeUnfocus: 0x5b2b8a,   // dimmed purple when another node is hovered
    edgeRest:    0x6d28d9,   // dim purple edge
    edgeActive:  0xc084fc,   // bright purple on hover connection
    edgeDim:     0x2a1050,   // near-invisible when unfocused
    pointLight:  0xa855f7
  };
  const PURPLE_STOPS = [0x4c1d95, 0x6d28d9, 0xa855f7, 0xc084fc, 0xe9d5ff];

  function hashLabel(value) {
    const s = String(value || "").toLowerCase();
    let h = 2166136261;
    for (let i = 0; i < s.length; i++) {
      h ^= s.charCodeAt(i);
      h = Math.imul(h, 16777619);
    }
    return (h >>> 0) / 4294967295;
  }

  function lerpChannel(a, b, t) {
    return Math.round(a + (b - a) * t);
  }

  function lerpHex(a, b, t) {
    const ar = (a >> 16) & 255, ag = (a >> 8) & 255, ab = a & 255;
    const br = (b >> 16) & 255, bg = (b >> 8) & 255, bb = b & 255;
    return (lerpChannel(ar, br, t) << 16)
      | (lerpChannel(ag, bg, t) << 8)
      | lerpChannel(ab, bb, t);
  }

  function primaryLabel(atom) {
    const tags = atom && Array.isArray(atom.tags) ? atom.tags : [];
    if (tags.length) return String(tags[0]);
    return text(atom && atom.source, "");
  }

  function purpleForLabel(label) {
    if (!label) return COLOUR.nodeBase;
    const t = hashLabel(label);
    const scaled = t * (PURPLE_STOPS.length - 1);
    const i = Math.floor(scaled);
    const f = scaled - i;
    const a = PURPLE_STOPS[i];
    const b = PURPLE_STOPS[Math.min(i + 1, PURPLE_STOPS.length - 1)];
    return lerpHex(a, b, f);
  }

  function purpleForAtom(atom) {
    return purpleForLabel(primaryLabel(atom));
  }

  function hexToCss(hex) {
    return `#${(hex >>> 0).toString(16).padStart(6, "0")}`;
  }

  function labelGradientCss(label) {
    const mid = purpleForLabel(label);
    const deep = lerpHex(mid, 0x2e1065, .45);
    const light = lerpHex(mid, 0xffffff, .28);
    return `linear-gradient(135deg, ${hexToCss(deep)}, ${hexToCss(mid)} 55%, ${hexToCss(light)})`;
  }

  /* ── 3-D graph (Three.js) ───────────────────────────────────── */
  /*
   * Soft electric orbs (not faceted cubes) + dashed synapse charge on edges.
   * Node fill sits on a purple gradient keyed by the atom's primary label.
   * Hover: brighter node pulse + faster zap on connected edges only.
   * Palette: white = hot, purple family = base. No background particle field.
   * Graph stays compact (radius ≤ 8) so the whole lattice fits the camera.
   */

  function makeGraph() {
    if (!window.THREE || !elements.canvas) return null;
    const THREE = window.THREE;
    const host = elements.canvas;
    const stage = elements.stage;

    const scene    = new THREE.Scene();
    const camera   = new THREE.PerspectiveCamera(44, 1, .1, 1000);
    camera.position.set(0, 0, 24);

    const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    host.append(renderer.domElement);

    const nodes = new THREE.Group();
    const edges = new THREE.Group();
    scene.add(nodes, edges);

    const pointLight = new THREE.PointLight(COLOUR.pointLight, 1.6, 80);
    pointLight.position.set(0, 0, 12);
    scene.add(pointLight);

    const raycaster = new THREE.Raycaster();
    const pointer   = new THREE.Vector2();
    let objects = [], nodeMap = new Map(), links = [];
    let hover = null, hoverConnected = new Set(), dragging = false, last = null;
    let yaw = 0, pitch = 0, distance = 24;
    let lastFrameAt = 0;
    let ghostMesh = null;
    let layoutMode = localStorage.getItem("kurultai-layout") === "solar" ? "solar" : "lattice";
    let lastAtoms = [];
    let transition = null;

    /* Solar layout cache — refreshed once per data change, O(1) per atom during frame */
    let _solarSunId   = null;
    let _solarSources = [];
    let _solarSiblings = new Map();

    function refreshSolarCache(atoms) {
      let best = atoms[0], bestScore = -Infinity;
      atoms.forEach((a) => {
        const s = Number(a.score) || 0;
        if (s > bestScore) { bestScore = s; best = a; }
      });
      _solarSunId = best ? best.id : null;
      _solarSources = [];
      _solarSiblings = new Map();
      atoms.forEach((a) => {
        if (!_solarSources.includes(a.source)) _solarSources.push(a.source);
        if (a.id === _solarSunId) return;
        if (!_solarSiblings.has(a.source)) _solarSiblings.set(a.source, []);
        _solarSiblings.get(a.source).push(a.id);
      });
    }

    function disposeGroup(group) {
      group.traverse((child) => {
        if (child.geometry) child.geometry.dispose();
        if (child.material) {
          if (Array.isArray(child.material)) child.material.forEach((m) => m.dispose());
          else child.material.dispose();
        }
      });
    }

    function nodeMaterial(hex) {
      return new THREE.MeshBasicMaterial({
        color:       hex == null ? COLOUR.nodeBase : hex,
        transparent: true,
        opacity:     .82
      });
    }

    function haloMaterial(hex) {
      return new THREE.MeshBasicMaterial({
        color:       hex == null ? COLOUR.nodeBase : hex,
        transparent: true,
        opacity:     .2,
        depthWrite:  false,
        blending:    THREE.AdditiveBlending
      });
    }

    function edgeMaterial(active) {
      return new THREE.LineDashedMaterial({
        color:       active ? COLOUR.edgeActive : COLOUR.edgeRest,
        transparent: true,
        opacity:     active ? .9 : .22,
        dashSize:    active ? .18 : .11,
        gapSize:     active ? .06 : .1,
        scale:       1
      });
    }

    function animateEdgeCharge(line, speed, deltaSeconds) {
      if (!line.geometry.attributes.lineDistance) line.computeLineDistances();
      const distances = line.geometry.attributes.lineDistance;
      if (!line.userData.baseLineDistances || line.userData.baseLineDistances.length !== distances.array.length) {
        line.userData.baseLineDistances = Array.from(distances.array);
        line.userData.dashPhase = 0;
      }

      const cycle = line.material.dashSize + line.material.gapSize;
      line.userData.dashPhase = ((line.userData.dashPhase || 0) + speed * deltaSeconds) % cycle;
      distances.array.forEach((_, index) => {
        distances.array[index] = line.userData.baseLineDistances[index] - line.userData.dashPhase;
      });
      distances.needsUpdate = true;
    }

    function resize() {
      const rect = stage.getBoundingClientRect();
      camera.aspect = rect.width / rect.height;
      camera.updateProjectionMatrix();
      renderer.setSize(rect.width, rect.height, false);
    }

    /* Fibonacci-sphere layout capped to a tight radius (8.5 max) */
    function positionLattice(index, total) {
      const phi   = Math.acos(1 - 2 * (index + .5) / Math.max(total, 1));
      const theta = Math.PI * (1 + Math.sqrt(5)) * index;
      const r     = 6.5 * (0.7 + .28 * Math.sin(phi) ** 1.4);
      return new THREE.Vector3(
        r * Math.cos(theta) * Math.sin(phi),
        4.8 * Math.cos(phi),
        r * Math.sin(theta) * Math.sin(phi) * .68
      );
    }

    /* Solar: hottest memory as sun, one orbital ring per source */
    function positionSolar(atom) {
      if (atom.id === _solarSunId) return new THREE.Vector3(0, 0, 0);
      const sourceIndex = Math.max(0, _solarSources.indexOf(atom.source));
      const orbitCount  = Math.max(_solarSources.length, 1);
      const orbit       = 2.4 + (sourceIndex / Math.max(orbitCount - 1, 1)) * 4.6;
      const siblings    = _solarSiblings.get(atom.source) || [];
      const i     = Math.max(0, siblings.indexOf(atom.id));
      const n     = Math.max(siblings.length, 1);
      const angle = (i / n) * Math.PI * 2 + sourceIndex * 0.35;
      const y     = ((i % 5) - 2) * 0.12;
      return new THREE.Vector3(Math.cos(angle) * orbit, y, Math.sin(angle) * orbit);
    }

    function positionFor(index, total, atom) {
      if (layoutMode === "solar" && atom) return positionSolar(atom);
      return positionLattice(index, total);
    }

    function clearGhostPreview() {
      if (!ghostMesh) return;
      scene.remove(ghostMesh);
      if (ghostMesh.geometry) ghostMesh.geometry.dispose();
      if (ghostMesh.material) ghostMesh.material.dispose();
      ghostMesh = null;
    }

    /* Cheap InstancedMesh preview of not-yet-loaded atoms (no fetch, no edges). */
    function showGhostPreview(loadedCount, totalCount) {
      clearGhostPreview();
      if (layoutMode === "solar") return;
      const extra = Math.max(0, (totalCount || 0) - (loadedCount || 0));
      if (extra <= 0 || !window.THREE) return;
      const count = Math.min(extra, GHOST_PREVIEW_CAP);
      const geo = new THREE.SphereGeometry(0.07, 8, 8);
      const mat = new THREE.MeshBasicMaterial({
        color: COLOUR.edgeActive,
        transparent: true,
        opacity: reducedMotion ? .1 : .14,
        depthWrite: false
      });
      const mesh = new THREE.InstancedMesh(geo, mat, count);
      mesh.raycast = () => {};
      const dummy = new THREE.Object3D();
      const totalForLayout = Math.max(totalCount, loadedCount + count);
      for (let i = 0; i < count; i++) {
        const index = loadedCount + i;
        dummy.position.copy(positionFor(index, totalForLayout));
        dummy.updateMatrix();
        mesh.setMatrixAt(i, dummy.matrix);
      }
      mesh.instanceMatrix.needsUpdate = true;
      scene.add(mesh);
      ghostMesh = mesh;
    }

    function update(atoms, renderCap = DEFAULT_ATOM_LIMIT) {
      transition = null;
      clearHover();
      clearGhostPreview();
      disposeGroup(nodes);
      disposeGroup(edges);
      nodes.clear(); edges.clear();
      objects = []; nodeMap = new Map();
      const cap = Math.max(1, Math.min(Number(renderCap) || DEFAULT_ATOM_LIMIT, MAX_RENDER_CAP));
      const rendered = atoms.slice(0, cap);
      lastAtoms = rendered.slice();
      refreshSolarCache(lastAtoms);
      links = buildRelationships(rendered);

      rendered.forEach((atom, index) => {
        const size = .12 + Math.min(atom.score, 1) * .10;
        const baseColor = purpleForAtom(atom);
        /* A: smooth orb (24 segments) — avoids faceted “cube” look from 9×9 */
        const mesh = new THREE.Mesh(
          new THREE.SphereGeometry(size, 24, 24),
          nodeMaterial(baseColor)
        );
        mesh.position.copy(positionFor(index, rendered.length, atom));
        mesh.userData.atom = atom;
        mesh.userData.baseColor = baseColor;
        mesh.userData.pulsePhase = Math.random() * Math.PI * 2;
        mesh.userData.baseOpacity = .82;

        const halo = new THREE.Mesh(
          new THREE.SphereGeometry(size * 2.15, 16, 16),
          haloMaterial(baseColor)
        );
        halo.raycast = () => {};
        halo.userData.isHalo = true;
        mesh.add(halo);
        mesh.userData.halo = halo;

        nodes.add(mesh);
        objects.push(mesh);
        nodeMap.set(atom.id, mesh);
      });

      links.forEach((link) => {
        const a = nodeMap.get(link.a);
        const b = nodeMap.get(link.b);
        if (!a || !b) return;
        const geo = new THREE.BufferGeometry().setFromPoints([a.position, b.position]);
        const line = new THREE.Line(geo, edgeMaterial(false));
        line.computeLineDistances();
        line.userData = link;
        edges.add(line);
      });

      /* Hide fallback once nodes are loaded */
      if (elements.fallback) {
        elements.fallback.style.opacity = rendered.length ? "0" : "1";
      }
    }

    function recolor() {
      renderer.setClearColor(0x000000, 0);
    }

    /* Hover: highlight hovered node + connected edges/nodes */
    function placeTooltip(event) {
      const rect = stage.getBoundingClientRect();
      elements.tooltip.style.left = `${Math.min(event.clientX - rect.left + 12, rect.width - 190)}px`;
      elements.tooltip.style.top  = `${Math.min(event.clientY - rect.top  + 12, rect.height - 56)}px`;
    }

    function styleEdge(line, mode) {
      /* mode: 'rest' | 'active' | 'dim' */
      if (mode === "active") {
        line.material.color.setHex(COLOUR.edgeActive);
        line.material.opacity = .95;
        line.material.dashSize = .2;
        line.material.gapSize = .05;
      } else if (mode === "dim") {
        line.material.color.setHex(COLOUR.edgeDim);
        line.material.opacity = .04;
        line.material.dashSize = .08;
        line.material.gapSize = .14;
      } else {
        line.material.color.setHex(COLOUR.edgeRest);
        line.material.opacity = .22;
        line.material.dashSize = .11;
        line.material.gapSize = .1;
      }
    }

    function showHover(mesh, event) {
      if (hover === mesh) {
        placeTooltip(event);
        return;
      }
      hover = mesh;
      const nextId = mesh.userData.atom.id;
      hoverConnected = new Set();
      links.forEach((link) => {
        if (link.a === nextId) hoverConnected.add(link.b);
        else if (link.b === nextId) hoverConnected.add(link.a);
      });

      objects.forEach((node) => {
        const halo = node.userData.halo;
        const base = node.userData.baseColor ?? COLOUR.nodeBase;
        if (node === mesh) {
          node.material.color.setHex(COLOUR.nodeHot);
          node.material.opacity = 1;
          if (halo) {
            halo.material.color.setHex(COLOUR.nodeHot);
            halo.material.opacity = .5;
          }
        } else if (hoverConnected.has(node.userData.atom.id)) {
          node.material.color.setHex(base);
          node.material.opacity = .92;
          if (halo) {
            halo.material.color.setHex(lerpHex(base, COLOUR.edgeActive, .35));
            halo.material.opacity = .32;
          }
        } else {
          node.material.color.setHex(lerpHex(base, COLOUR.nodeUnfocus, .7));
          node.material.opacity = .3;
          if (halo) {
            halo.material.color.setHex(COLOUR.nodeUnfocus);
            halo.material.opacity = .06;
          }
        }
      });

      edges.children.forEach((line) => {
        const linked = line.userData.a === nextId || line.userData.b === nextId;
        styleEdge(line, linked ? "active" : "dim");
      });

      const atom = mesh.userData.atom;
      elements.tooltip.hidden = false;
      elements.tooltip.innerHTML = "";
      const titleEl = document.createElement("strong");
      titleEl.textContent = atom.title;
      const metaEl = document.createElement("span");
      metaEl.textContent = `${atom.tags.length} tags · ${atom.tier}`;
      elements.tooltip.append(titleEl, metaEl);
      placeTooltip(event);
      renderInspector(atom);
    }

    function clearHover() {
      hover = null;
      hoverConnected = new Set();
      elements.tooltip.hidden = true;
      objects.forEach((node) => {
        const base = node.userData.baseColor ?? COLOUR.nodeBase;
        node.material.color.setHex(base);
        node.material.opacity = node.userData.baseOpacity ?? .82;
        node.scale.setScalar(1);
        const halo = node.userData.halo;
        if (halo) {
          halo.material.color.setHex(base);
          halo.material.opacity = .2;
        }
      });
      edges.children.forEach((line) => styleEdge(line, "rest"));
    }

    function hit(event) {
      const rect = renderer.domElement.getBoundingClientRect();
      pointer.x = ((event.clientX - rect.left) / rect.width)  *  2 - 1;
      pointer.y = -((event.clientY - rect.top)  / rect.height) *  2 + 1;
      raycaster.setFromCamera(pointer, camera);
      return raycaster.intersectObjects(objects, false)[0]?.object || null;
    }

    renderer.domElement.addEventListener("pointermove", (e) => {
      if (dragging && last) {
        yaw   += (e.clientX - last.x) * .008;
        pitch  = Math.max(-.8, Math.min(.8, pitch + (e.clientY - last.y) * .008));
        last   = { x: e.clientX, y: e.clientY };
        return;
      }
      const obj = hit(e);
      if (obj)        showHover(obj, e);
      else if (hover) clearHover();
    });
    renderer.domElement.addEventListener("pointerdown", (e) => {
      dragging = true; last = { x: e.clientX, y: e.clientY };
    });
    window.addEventListener("pointerup", () => { dragging = false; last = null; });
    renderer.domElement.addEventListener("wheel", (e) => {
      e.preventDefault();
      distance = Math.max(12, Math.min(50, distance + e.deltaY * .015));
    }, { passive: false });
    renderer.domElement.addEventListener("click", (e) => {
      const obj = hit(e);
      if (obj) renderInspector(obj.userData.atom);
    });
    stage.addEventListener("keydown", (e) => { if (e.key === "Escape") clearHover(); });

    function frame(now) {
      const deltaSeconds = lastFrameAt ? Math.min(Math.max((now - lastFrameAt) / 1000, 0), .05) : 0;
      lastFrameAt = now;

      /* A: soft heartbeat on orbs; C: hotter pulse on hover + neighbors */
      if (!reducedMotion && objects.length) {
        objects.forEach((mesh) => {
          const phase = mesh.userData.pulsePhase || 0;
          const isHot = hover === mesh;
          const isLinked = hover && hoverConnected.has(mesh.userData.atom.id);
          const amp = isHot ? .16 : isLinked ? .1 : .055;
          const speed = isHot ? .0042 : .0028;
          const t = Math.sin(now * speed + phase) * .5 + .5;
          mesh.scale.setScalar(1 + t * amp);
          const halo = mesh.userData.halo;
          if (halo && !hover) {
            halo.material.opacity = .14 + t * .12;
          } else if (halo && isHot) {
            halo.material.opacity = .38 + t * .22;
          } else if (halo && isLinked) {
            halo.material.opacity = .24 + t * .14;
          }
        });
      }

      /* Solar layout transition */
      if (transition) {
        const t = Math.min(1, (performance.now() - transition.start) / transition.duration);
        const e = t < .5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
        transition.items.forEach((item) => item.mesh.position.lerpVectors(item.from, item.to, e));
        edges.children.forEach((line) => {
          const a = nodeMap.get(line.userData.a), b = nodeMap.get(line.userData.b);
          if (!a || !b) return;
          const pos = line.geometry.attributes.position;
          pos.setXYZ(0, a.position.x, a.position.y, a.position.z);
          pos.setXYZ(1, b.position.x, b.position.y, b.position.z);
          pos.needsUpdate = true;
          line.computeLineDistances();
          delete line.userData.baseLineDistances;
        });
        if (t >= 1) transition = null;
      } else if (layoutMode === "solar" && !reducedMotion && objects.length) {
        const angle = 0.13 * deltaSeconds;
        const cos = Math.cos(angle), sin = Math.sin(angle);
        objects.forEach((mesh) => {
          if (mesh.userData.atom && mesh.userData.atom.id === _solarSunId) return;
          const x = mesh.position.x, z = mesh.position.z;
          mesh.position.x = x * cos - z * sin;
          mesh.position.z = x * sin + z * cos;
        });
        edges.children.forEach((line) => {
          const a = nodeMap.get(line.userData.a), b = nodeMap.get(line.userData.b);
          if (!a || !b) return;
          const pos = line.geometry.attributes.position;
          pos.setXYZ(0, a.position.x, a.position.y, a.position.z);
          pos.setXYZ(1, b.position.x, b.position.y, b.position.z);
          pos.needsUpdate = true;
          line.computeLineDistances();
          delete line.userData.baseLineDistances;
        });
      }

      /* B: synapse charge along edges; C: faster zap on hover arcs */
      if (!reducedMotion && edges.children.length) {
        edges.children.forEach((line) => {
          const hot = hover && (
            line.userData.a === hover.userData.atom.id ||
            line.userData.b === hover.userData.atom.id
          );
          /* Per-second equivalents of the original 60fps frame-step rates. */
          animateEdgeCharge(line, hot ? 3.3 : .84, deltaSeconds);
        });
      }

      camera.position.set(
        Math.sin(yaw) * distance,
        Math.sin(pitch) * distance * .48,
        Math.cos(yaw) * distance
      );
      camera.lookAt(0, 0, 0);
      renderer.render(scene, camera);
      requestAnimationFrame(frame);
    }

    function setLayout(next) {
      if (next !== "lattice" && next !== "solar") return;
      if (layoutMode === next) return;
      layoutMode = next;
      localStorage.setItem("kurultai-layout", next);
      if (!lastAtoms.length || !objects.length) return;
      refreshSolarCache(lastAtoms);
      const items = [];
      lastAtoms.forEach((atom, index) => {
        const mesh = nodeMap.get(atom.id);
        if (!mesh) return;
        items.push({ mesh, from: mesh.position.clone(), to: positionFor(index, lastAtoms.length, atom) });
      });
      transition = { start: performance.now(), duration: reducedMotion ? 1 : 850, items };
    }

    new ResizeObserver(resize).observe(stage);
    resize(); recolor();
    requestAnimationFrame(frame);
    return { update, recolor, showGhostPreview, clearGhostPreview, setLayout, getLayout: () => layoutMode };
  }

  /* ── CDN Three.js load guard ────────────────────────────────── */
  /*
   * If Three.js fails to load (e.g. unpkg.com blocked), window.THREE will be
   * undefined and makeGraph() returns null. In that case we keep the static
   * SVG fallback fully visible and skip the 3-D canvas entirely.
   */
  function initGraph() {
    if (!window.THREE) {
      // CDN blocked — keep fallback at full opacity, just update caption
      if (elements.fallback) elements.fallback.style.opacity = "1";
      if (elements.canvas)  elements.canvas.style.display   = "none";
      elements.caption.textContent = "3-D graph offline (CDN blocked) — static view active";
      return null;
    }
    return makeGraph();
  }

  /* ── API calls ──────────────────────────────────────────────── */
  async function loadStatus() {
    try {
      const data = await getJson("/api/status");
      setStatus(data.ok ? "daemon online" : "daemon recovering", Boolean(data.ok));
      const total = Number(data.atoms);
      state.atomTotal = Number.isFinite(total) ? total : state.atomTotal;
      $("stat-atoms").textContent    = text(data.atoms);
      const memory = data.memory || {};
      $("stat-tiers").textContent   = `${memory.hot ?? 0} / ${memory.warm ?? 0} / ${memory.cold ?? 0}`;
      $("stat-trusted").textContent  = text(data.brain && data.brain.trusted_count);
      if (!state.maxMode) refreshLattice();
    } catch (_) {
      setStatus("daemon offline", false);
    }
  }

  async function loadAtoms() {
    const gen = ++state.loadGen;
    if (state.loadAbort) state.loadAbort.abort();
    const controller = new AbortController();
    state.loadAbort = controller;
    const { signal } = controller;
    const wantedMax = state.maxMode;

    try {
      if (!wantedMax) {
        const data = await getJson(`/api/atoms?limit=${DEFAULT_ATOM_LIMIT}`, signal);
        if (gen !== state.loadGen || state.maxMode !== wantedMax) return;
        state.atoms = data.map(normalize);
        if (state.atoms.length > state.atomTotal) state.atomTotal = state.atoms.length;
        refreshLattice();
        return;
      }

      /* Max mode: progressive lean graph tiers — full details only on inspect. */
      const byId = new Map();
      for (const tier of GRAPH_TIERS) {
        if (gen !== state.loadGen || state.maxMode !== wantedMax) return;
        const data = await getJson(
          `/api/graph?tier=${tier}&limit=${MAX_GRAPH_LIMIT}`,
          signal
        );
        if (gen !== state.loadGen || state.maxMode !== wantedMax) return;
        (data.nodes || []).forEach((node) => {
          const atom = normalizeGraphNode(node);
          if (!byId.has(atom.id)) byId.set(atom.id, atom);
        });
        state.atoms = Array.from(byId.values());
        if (state.atoms.length > state.atomTotal) state.atomTotal = state.atoms.length;
        elements.caption.textContent =
          `loading ${tier} · ${state.atoms.length} memories so far (≤${MAX_GRAPH_LIMIT})…`;
        refreshLattice();
      }
    } catch (err) {
      if (err && err.name === "AbortError") return;
      if (gen !== state.loadGen) return;
      state.atoms = [];
      refreshLattice();
      elements.caption.textContent = "Local daemon unavailable — lattice standby.";
    }
  }

  async function ask(event) {
    event.preventDefault();
    const question = elements.askInput.value.trim();
    if (!question) return;
    elements.askOutput.textContent = "Thinking locally…";
    try {
      const answer = await getJson(`/api/ask?question=${encodeURIComponent(question)}`);
      elements.askOutput.textContent = text(
        answer.answer || answer.response || answer.text,
        "The daemon returned an empty answer."
      );
      addActivity({ tool: "ask", query: question });
    } catch (_) {
      elements.askOutput.textContent = "The local daemon could not answer right now.";
    }
  }

  function setTimelineLabel() {
    const value = Number(elements.timeline.value);
    elements.timelineOutput.value =
      value === 100 ? "all time" : value > 66 ? "recent" : value > 33 ? "focused" : "now";
    refreshLattice();
  }

  function togglePlayback() {
    state.playing = !state.playing;
    elements.play.setAttribute("aria-pressed", String(state.playing));
    elements.play.textContent = state.playing ? "Ⅱ" : "▶";
    elements.play.setAttribute("aria-label", state.playing ? "Pause timeline" : "Play timeline");
    clearInterval(state.timer);
    if (state.playing) {
      state.timer = setInterval(() => {
        const value = Number(elements.timeline.value);
        elements.timeline.value = value >= 100 ? 0 : value + 2;
        setTimelineLabel();
      }, reducedMotion ? 1500 : 650);
    }
  }

  /* ── Event bindings ─────────────────────────────────────────── */
  function bind() {
    elements.theme.addEventListener("click", () =>
      applyTheme(document.documentElement.dataset.theme === "dark" ? "light" : "dark")
    );
    elements.streamToggle.addEventListener("click", () => {
      state.live = !state.live;
      elements.streamToggle.setAttribute("aria-pressed", String(state.live));
      elements.streamToggle.textContent = state.live ? "live" : "paused";
    });
    if (elements.maxToggle) {
      elements.maxToggle.addEventListener("mouseenter", () => {
        if (state.maxMode || !state.graph) return;
        const loaded = state.atoms.length;
        const total = Math.max(state.atomTotal || 0, loaded);
        if (total <= loaded) return;
        elements.maxToggle.classList.add("is-ghosting");
        state.graph.showGhostPreview(loaded, total);
        elements.caption.textContent =
          `ghost preview · ~${Math.min(total - loaded, GHOST_PREVIEW_CAP)} more of ${total} (click max to load)`;
      });
      elements.maxToggle.addEventListener("mouseleave", () => {
        elements.maxToggle.classList.remove("is-ghosting");
        if (state.graph) state.graph.clearGhostPreview();
        if (!state.maxMode) refreshLattice();
      });
      elements.maxToggle.addEventListener("focus", () => {
        elements.maxToggle.dispatchEvent(new Event("mouseenter"));
      });
      elements.maxToggle.addEventListener("blur", () => {
        elements.maxToggle.dispatchEvent(new Event("mouseleave"));
      });
      elements.maxToggle.addEventListener("click", () => {
        setMaxMode(!state.maxMode);
      });
    }
    if (elements.layoutToggle) {
      const syncLayoutToggle = () => {
        const mode = state.graph
          ? state.graph.getLayout()
          : (localStorage.getItem("kurultai-layout") === "solar" ? "solar" : "lattice");
        const solar = mode === "solar";
        elements.layoutToggle.setAttribute("aria-pressed", String(solar));
        elements.layoutToggle.textContent = solar ? "lattice" : "solar";
        elements.layoutToggle.setAttribute("aria-label",
          solar ? "Switch memory layout to lattice" : "Switch memory layout to solar system");
      };
      syncLayoutToggle();
      state._syncLayoutToggle = syncLayoutToggle;
      elements.layoutToggle.addEventListener("click", () => {
        if (!state.graph) return;
        const next = state.graph.getLayout() === "solar" ? "lattice" : "solar";
        state.graph.setLayout(next);
        syncLayoutToggle();
      });
    }
    elements.timeline.addEventListener("input", setTimelineLabel);
    elements.play.addEventListener("click", togglePlayback);
    elements.askForm.addEventListener("submit", ask);
    elements.search.addEventListener("input", () => {
      clearTimeout(searchDelay);
      searchDelay = setTimeout(() => search(elements.search.value), 200);
    });
    elements.search.addEventListener("keydown", (e) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") e.preventDefault();
      if (e.key === "Escape") elements.dropdown.hidden = true;
    });
    document.addEventListener("keydown", (e) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        elements.search.focus();
      }
    });
  }

  /* ── Bootstrap ──────────────────────────────────────────────── */
  async function init() {
    state.maxMode = localStorage.getItem("kurultai-max-mode") === "1";
    applyTheme(initialTheme());
    applyMaxToggleUi();
    bind();
    state.graph = initGraph();
    if (state._syncLayoutToggle) state._syncLayoutToggle();
    await Promise.all([loadStatus(), loadAtoms(), pollActivity()]);
    window.setInterval(loadStatus,   20000);
    window.setInterval(pollActivity,  2500);
  }

  document.addEventListener("DOMContentLoaded", init);
})();
