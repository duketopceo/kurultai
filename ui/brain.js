(() => {
  "use strict";
  const $ = (id) => document.getElementById(id);

  const state = {
    atoms: [], visible: [], selected: null,
    query: "", since: 0, live: true,
    playing: false, timer: null, graph: null
  };

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
    theme:          $("theme-toggle")
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
      tier:           atom.tier || meta.tier || "warm"
    };
  }
  async function getJson(path) {
    const r = await fetch(path, { headers: { Accept: "application/json" } });
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
    if (state.graph) state.graph.update(state.visible);
    elements.caption.textContent = `${state.visible.length} memories · hover to trace connections`;
  }

  /* ── Inspector ──────────────────────────────────────────────── */
  function renderInspector(atom) {
    state.selected = atom;
    if (!atom) {
      elements.inspector.innerHTML = "";
      const p = document.createElement("p");
      p.className = "empty-state";
      p.textContent = "Hover or select a memory node to reveal its place in the lattice.";
      elements.inspector.append(p);
      return;
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
      const tags = document.createElement("p");
      tags.className = "inspector-summary";
      tags.textContent = atom.tags.map((tag) => `#${tag}`).join(" ");
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
      elements.dropdown.hidden = true;
      elements.dropdown.innerHTML = "";
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

  /* ── 3-D graph (Three.js) ───────────────────────────────────── */
  /*
   * Node colour palette: white = hovered/selected, purple = base.
   * Edge colour:         dim purple at rest, bright on hover highlight.
   * No floating background particles (avoided per AGENTS.md clean aesthetic).
   * Graph stays compact (radius ≤ 8) to prevent far-out dots.
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
    let hover = null, hoveredId = null, dragging = false, last = null;
    let yaw = 0, pitch = 0, distance = 24;

    function disposeGroup(group) {
      group.traverse((child) => {
        if (child.geometry) child.geometry.dispose();
        if (child.material) {
          if (Array.isArray(child.material)) child.material.forEach((m) => m.dispose());
          else child.material.dispose();
        }
      });
    }

    function nodeMaterial(active) {
      return new THREE.MeshBasicMaterial({
        color:       active ? COLOUR.nodeHot : COLOUR.nodeBase,
        transparent: true,
        opacity:     active ? 1 : .82
      });
    }

    function resize() {
      const rect = stage.getBoundingClientRect();
      camera.aspect = rect.width / rect.height;
      camera.updateProjectionMatrix();
      renderer.setSize(rect.width, rect.height, false);
    }

    /* Fibonacci-sphere layout capped to a tight radius (8.5 max) */
    function positionFor(index, total) {
      const phi   = Math.acos(1 - 2 * (index + .5) / Math.max(total, 1));
      const theta = Math.PI * (1 + Math.sqrt(5)) * index;
      const r     = 6.5 * (0.7 + .28 * Math.sin(phi) ** 1.4);
      return new THREE.Vector3(
        r * Math.cos(theta) * Math.sin(phi),
        4.8 * Math.cos(phi),
        r * Math.sin(theta) * Math.sin(phi) * .68
      );
    }

    function update(atoms) {
      disposeGroup(nodes);
      disposeGroup(edges);
      nodes.clear(); edges.clear();
      objects = []; nodeMap = new Map();
      links = buildRelationships(atoms);

      atoms.slice(0, 450).forEach((atom, index) => {
        const size = .12 + Math.min(atom.score, 1) * .10;
        const mesh = new THREE.Mesh(
          new THREE.SphereGeometry(size, 9, 9),
          nodeMaterial(false)
        );
        mesh.position.copy(positionFor(index, Math.min(atoms.length, 450)));
        mesh.userData.atom = atom;
        nodes.add(mesh);
        objects.push(mesh);
        nodeMap.set(atom.id, mesh);
      });

      links.forEach((link) => {
        const a = nodeMap.get(link.a);
        const b = nodeMap.get(link.b);
        if (!a || !b) return;
        const geo = new THREE.BufferGeometry().setFromPoints([a.position, b.position]);
        const line = new THREE.Line(geo, new THREE.LineBasicMaterial({
          color: COLOUR.edgeRest, transparent: true, opacity: .18
        }));
        line.userData = link;
        edges.add(line);
      });

      /* Hide fallback once nodes are loaded */
      if (elements.fallback) {
        elements.fallback.style.opacity = atoms.length ? "0" : "1";
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

    function showHover(mesh, event) {
      const nextId = mesh.userData.atom.id;
      if (hoveredId === nextId) {
        placeTooltip(event);
        return;
      }
      hover = mesh;
      hoveredId = nextId;
      const connectedIds = new Set();
      links.forEach((link) => {
        if (link.a === hoveredId) connectedIds.add(link.b);
        else if (link.b === hoveredId) connectedIds.add(link.a);
      });

      objects.forEach((node) => {
        if (node === mesh) {
          node.material.color.setHex(COLOUR.nodeHot);
          node.material.opacity = 1;
        } else if (connectedIds.has(node.userData.atom.id)) {
          node.material.color.setHex(COLOUR.nodeBase);
          node.material.opacity = .9;
        } else {
          node.material.color.setHex(COLOUR.nodeUnfocus);
          node.material.opacity = .3;
        }
      });

      edges.children.forEach((line) => {
        const linked = line.userData.a === hoveredId || line.userData.b === hoveredId;
        line.material.opacity = linked ? .9 : .04;
        line.material.color.setHex(linked ? COLOUR.edgeActive : COLOUR.edgeDim);
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
      hoveredId = null;
      elements.tooltip.hidden = true;
      objects.forEach((node) => {
        node.material.color.setHex(COLOUR.nodeBase);
        node.material.opacity = .82;
      });
      edges.children.forEach((line) => {
        line.material.opacity = .18;
        line.material.color.setHex(COLOUR.edgeRest);
      });
    }

    function hit(event) {
      const rect = renderer.domElement.getBoundingClientRect();
      pointer.x = ((event.clientX - rect.left) / rect.width)  *  2 - 1;
      pointer.y = -((event.clientY - rect.top)  / rect.height) *  2 + 1;
      raycaster.setFromCamera(pointer, camera);
      return raycaster.intersectObjects(objects)[0]?.object || null;
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
      camera.position.set(
        Math.sin(yaw) * distance,
        Math.sin(pitch) * distance * .48,
        Math.cos(yaw) * distance
      );
      camera.lookAt(0, 0, 0);
      renderer.render(scene, camera);
      requestAnimationFrame(frame);
    }

    new ResizeObserver(resize).observe(stage);
    resize(); recolor();
    requestAnimationFrame(frame);
    return { update, recolor };
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
      $("stat-atoms").textContent    = text(data.atoms);
      const memory = data.memory || {};
      $("stat-tiers").textContent   = `${memory.hot ?? 0} / ${memory.warm ?? 0} / ${memory.cold ?? 0}`;
      $("stat-trusted").textContent  = text(data.brain && data.brain.trusted_count);
    } catch (_) {
      setStatus("daemon offline", false);
    }
  }

  async function loadAtoms() {
    try {
      const data   = await getJson("/api/atoms?limit=450");
      state.atoms  = data.map(normalize);
      refreshLattice();
    } catch (_) {
      state.atoms  = [];
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
    applyTheme(initialTheme());
    bind();
    state.graph = initGraph();
    await Promise.all([loadStatus(), loadAtoms(), pollActivity()]);
    window.setInterval(loadStatus,   20000);
    window.setInterval(pollActivity,  2500);
  }

  document.addEventListener("DOMContentLoaded", init);
})();
