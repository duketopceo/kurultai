import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import brainUrl from '../assets/brain.glb?url';
import type { Atom, Link, Theme, LayoutMode } from '../types';
import { hashId } from '../state';

/*
 * Particle-cortex renderer. The anatomical brain mesh is dissolved into GPU
 * point sprites (gl.POINTS) — one per mesh vertex. Memories are pinned to
 * cortex vertices by FNV-1a hash; shared-tag or shared-source synapses arc
 * between them. Cursor proximity drives a local electric flare through
 * nearby particles via uPointer/uHover uniforms.
 */

const PARTICLE_VERTEX = /* glsl */ `
attribute vec3 aOffset;
attribute float aRotation;
attribute float aSize;
attribute vec3 aColor;
attribute float aSeed;
uniform vec3 uPointer;
uniform float uHover;
uniform float uTime;
uniform float uIntro;
varying vec3 vColor;
varying float vAlpha;

void main() {
  float d = distance(uPointer, aOffset);
  float c = smoothstep(0.45, 0.08, d);
  float scale = (aSize + c * 3.5 * uHover) * uIntro;
  float drift = uTime * (0.2 + aSeed * 0.3);
  vec3 pos = aOffset;
  pos.x += sin(drift + aSeed * 6.28) * 0.004 * aRotation;
  pos.y += cos(drift * 0.7 + aSeed * 3.14) * 0.004 * aRotation;
  pos.z += sin(drift * 0.5 + aSeed * 4.71) * 0.004 * aRotation;
  vec4 mvPosition = modelViewMatrix * vec4(pos, 1.0);
  gl_Position = projectionMatrix * mvPosition;
  gl_PointSize = scale * (500.0 / -mvPosition.z);
  float flicker = 0.72 + 0.28 * sin(uTime * 2.6 + aSeed * 39.0);
  vColor = mix(aColor, vec3(1.0), c * uHover * 0.3);
  vAlpha = flicker * (0.65 + 0.2 * c * uHover);
}
`;

const PARTICLE_FRAGMENT = /* glsl */ `
varying vec3 vColor;
varying float vAlpha;
void main() {
  float dist = distance(gl_PointCoord, vec2(0.5));
  if (dist > 0.5) discard;
  float soft = 1.0 - smoothstep(0.35, 0.5, dist);
  gl_FragColor = vec4(vColor, vAlpha * soft);
}
`;

/*
 * Node point-sprite cloud (U3 / KTD3). A second instance of the cortex
 * circular-falloff idiom, used when the memory-node count exceeds the
 * sphere-mesh budget. Per-node size is driven by the same degree formula as
 * the sphere radius (NODE_SPRITE_SIZE_SCALE converts world radius → the
 * gl_PointSize pixel curve the cortex shader uses). uPointer/uHover are shared
 * with the cortex uniforms so the pointer flare routes through the hovered
 * node exactly as it does for meshes (highlightConnections sets pointerTarget).
 */
const NODE_SPRITE_VERTEX = /* glsl */ `
attribute float aSize;
attribute vec3 aColor;
attribute float aAlpha;
uniform vec3 uPointer;
uniform float uHover;
uniform float uIntro;
varying vec3 vColor;
varying float vAlpha;

void main() {
  vec4 mvPosition = modelViewMatrix * vec4(position, 1.0);
  gl_Position = projectionMatrix * mvPosition;
  float d = distance(uPointer, position);
  float c = smoothstep(0.22, 0.04, d);
  float scale = (aSize + c * 2.5 * uHover) * uIntro;
  gl_PointSize = scale * (500.0 / -mvPosition.z);
  vColor = mix(aColor, vec3(1.0), c * uHover * 0.25);
  vAlpha = aAlpha * (0.85 + 0.1 * c * uHover);
}
`;

const NODE_SPRITE_FRAGMENT = /* glsl */ `
varying vec3 vColor;
varying float vAlpha;
void main() {
  float dist = distance(gl_PointCoord, vec2(0.5));
  if (dist > 0.5) discard;
  float soft = 1.0 - smoothstep(0.35, 0.5, dist);
  gl_FragColor = vec4(vColor, vAlpha * soft);
}
`;

interface Palette {
  nodeBase: number;
  nodeHot: number;
  nodeUnfocus: number;
  edgeRest: number;
  edgeActive: number;
  edgeDim: number;
  particles: number[];
}

const DARK_PALETTE: Palette = {
  nodeBase: 0xa855f7,
  nodeHot: 0xffffff,
  nodeUnfocus: 0x5b2b8a,
  edgeRest: 0x6d28d9,
  edgeActive: 0xc084fc,
  edgeDim: 0x2a1050,
  particles: [0x7c3aed, 0xa855f7, 0x6d28d9, 0xc084fc, 0xffffff],
};

const LIGHT_PALETTE: Palette = {
  nodeBase: 0x7c3aed,
  nodeHot: 0x4c1d95,
  nodeUnfocus: 0xc4b5fd,
  edgeRest: 0x7c3aed,
  edgeActive: 0x6d28d9,
  edgeDim: 0xe9d5ff,
  particles: [0x7c3aed, 0x6d28d9, 0x9333ea, 0x4c1d95, 0xa855f7],
};

export type Region = 'left' | 'right' | 'stem';

const MAX_NODES = 2500;
const MAX_EDGES = 3000;

// Hybrid node rendering (KTD3 / R4): above this count nodes render as a single
// THREE.Points draw call; at or below it the sphere+halo+label meshes render.
const NODE_SPRITE_CUTOFF = 500;
// Converts the sphere radius (world units) to the cortex gl_PointSize curve.
// Tuned so the sprite visual size matches the sphere at the 500-node crossover.
const NODE_SPRITE_SIZE_SCALE = 5;
// Raycaster threshold (world units) for picking individual node sprites.
const NODE_RAYCAST_THRESHOLD = 0.02;

// Force-directed layout constants (vanilla JS — no physics library).
const FORCE_MAX_ITERATIONS = 300;
const FORCE_DAMPING = 0.85;
const FORCE_SPRING_REST = 0.35;
const FORCE_SPRING_K = 0.05;
const FORCE_REPULSION_K = 0.02;
const FORCE_REPULSION_MAX_DIST = 1.5; // pairs farther than this don't repel
const FORCE_REPULSION_EPSILON = 0.01; // guards divide-by-zero on coincident nodes
const FORCE_CENTER_K = 0.01; // weak pull toward origin
const FORCE_SETTLE_THRESHOLD = 0.0005; // avg displacement/iter below this ⇒ settled
// Large-graph iteration cap (U4): at n > FORCE_BIG_GRAPH_N a full
// FORCE_MAX_ITERATIONS sim is ~25-50s of background slices (n²/2 ≈ 3.1M pair
// evals/iter at n=2500, ~80-160ms/iter), so the galaxy/cluster shape takes
// far too long to emerge. 150 iterations still resolves the macro shape, and
// FORCE_SETTLE_THRESHOLD early-exits warm-started refreshes before either cap.
const FORCE_BIG_GRAPH_N = 1500;
const FORCE_MAX_ITERATIONS_BIG_GRAPH = 150;

// Dev-only perf instrumentation (U4): flip to true locally to log setData and
// force-sim timings. Kept behind a module-level const because the repo
// tsconfig has no vite/client types (import.meta.env doesn't typecheck); when
// false the guards are dead-code-eliminated and no timing calls ever run.
const PERF_DEBUG = false;

export interface BrainViewOptions {
  theme: Theme;
  reducedMotion: boolean;
  onHoverAtom: (atom: Atom, clientX: number, clientY: number) => void;
  onSelectAtom: (atom: Atom) => void;
  onClearHover: () => void;
  onZoomOut: () => void;
  onReady: () => void;
  onError: (message: string) => void;
}

export class BrainView {
  private container: HTMLElement;
  private opts: BrainViewOptions;
  private palette: Palette;

  private renderer!: THREE.WebGLRenderer;
  private scene = new THREE.Scene();
  private camera!: THREE.PerspectiveCamera;
  private brainGroup = new THREE.Group();
  private nodeGroup = new THREE.Group();
  private edgeGroup = new THREE.Group();
  private proxy?: THREE.Mesh;

  private uniforms = {
    uPointer: { value: new THREE.Vector3(999, 999, 999) },
    uHover: { value: 0 },
    uTime: { value: 0 },
    uIntro: { value: 0 },
  };
  private pointerTarget = new THREE.Vector3(999, 999, 999);
  private hoverTarget = 0;
  private focusPulse: { pos: THREE.Vector3; until: number } | null = null;

  private verts: Float32Array = new Float32Array(0);
  private norms: Float32Array = new Float32Array(0);
  private particleColorIndex: number[] = [];
  private particleColorAttr?: THREE.BufferAttribute;

  private sphereGeo = new THREE.SphereGeometry(1, 10, 10);
  private haloTexture: THREE.Texture;
  private nodeObjects: THREE.Mesh[] = [];
  private nodeMap = new Map<string, THREE.Mesh>();
  private haloMap = new Map<string, THREE.Sprite>();
  private labelMap = new Map<string, THREE.Sprite>();
  private links: Link[] = [];
  // Top-MAX_EDGES links by strength, computed once per setData and shared by
  // the edge builder and the force-sim spring setup.
  private sortedLinks: Link[] = [];
  private atomsById = new Map<string, Atom>();

  /* ── Hybrid node rendering (U3) ────────────────────────────── */
  // Authoritative per-atom positions in brainGroup-local space. BOTH render
  // paths write to this; the edge builder, layout animation, hover flare and
  // camera focus all read from it. Mesh position and the sprite position
  // attribute are both just sinks that mirror this map.
  private atomPositions = new Map<string, THREE.Vector3>();
  // SetData-time lattice (vertexForRegion) positions, cached so the layout
  // animation can target them without re-probing/saturating usedVerts every
  // frame (U4: at 2500 nodes vs ~2.9k mesh verts the probe saturates within
  // one animation frame, turning the lattice switch into multi-second scans).
  private latticePositions = new Map<string, THREE.Vector3>();
  // Atom ids currently passing the connectionThreshold filter (both modes).
  private visibleAtomIds = new Set<string>();
  // Shown atom ids in render order (the slice(0, MAX_NODES) result).
  private _shownIds: string[] = [];
  // True when nodes render as a THREE.Points cloud instead of sphere meshes.
  private spriteMode = false;
  private nodeSpriteCloud: THREE.Points | null = null;
  private spritePosAttr?: THREE.BufferAttribute;
  private spriteColorAttr?: THREE.BufferAttribute;
  private spriteAlphaAttr?: THREE.BufferAttribute;
  // atomId → brain region lookup (sprite mode only).
  private spriteRegionOf = new Map<string, Region>();
  // On-demand label sprite for the hovered node in sprite mode.
  private spriteHoverLabel: THREE.Sprite | null = null;
  // Unified hover/zoom trackers (work in both modes).
  private hoverAtomId: string | null = null;
  private hoverConnected = new Set<string>();
  private zoomAtomId: string | null = null;
  private regionVerts: Record<Region, number[]> = { left: [], right: [], stem: [] };
  private usedVerts = new Set<number>();
  private degrees = new Map<string, number>();
  private layoutMode: LayoutMode = 'brain';
  private _solarSunId = '';
  private _solarPlanets = new Map<string, { orbitR: number; angle: number; tilt: number }>();
  private _solarMoons = new Map<string, { planetId: string; moonR: number; moonAngle: number }>();
  private _solarAsteroids = new Map<string, { r: number; angle: number; y: number }>();

  /* ── Force-directed layout state ──────────────────────────── */
  // Authoritative settled positions (synced once on sim completion).
  private forcePositions = new Map<string, THREE.Vector3>();
  // Flat arrays for the O(n²) hot loop — avoids per-iteration Vector3/GC churn.
  private _forcePosArr = new Float32Array(0);
  private _forceVelArr = new Float32Array(0);
  private _forceIds: string[] = [];
  private _forceIndex = new Map<string, number>();
  private _forceLinkPairs: { a: number; b: number; strength: number }[] = [];
  private _forceSimHandle: ReturnType<typeof setTimeout> | null = null;
  private _forceIterations = 0;
  // PERF_DEBUG-only sim timers (never written when PERF_DEBUG is false).
  private _perfSimT0 = 0;
  private _perfSimActive = 0;

  /* ── Interactive control state ─────────────────────────────── */
  private connectionThreshold = 0;
  private showSynapses = true;
  private showLabels = false;
  private showRegions = false;
  private manualDistance: number | null = null;

  private raycaster = new THREE.Raycaster();
  private pointerNdc = new THREE.Vector2();
  private parallax = new THREE.Vector2();
  private parallaxTarget = new THREE.Vector2();

  private yaw = 0;
  private pitch = 0;
  private distance = 1.75;
  // Scratch vectors for the zoom camera branch of loop() (no per-frame allocs).
  private _zoomLocal = new THREE.Vector3();
  private _zoomWorld = new THREE.Vector3();
  private _zoomDir = new THREE.Vector3();
  private dragging = false;
  private last: { x: number; y: number } | null = null;

  private clock = new THREE.Clock();
  private raf = 0;
  private layoutAnimRaf = 0;
  private resizeObserver?: ResizeObserver;
  private disposed = false;

  constructor(container: HTMLElement, opts: BrainViewOptions) {
    this.container = container;
    this.opts = opts;
    this.palette = opts.theme === 'light' ? LIGHT_PALETTE : DARK_PALETTE;
    this.haloTexture = this.makeHaloTexture();

    try {
      this.renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
    } catch {
      opts.onError('WebGL is unavailable in this browser.');
      return;
    }
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    this.renderer.setClearColor(0x000000, 0);
    container.appendChild(this.renderer.domElement);

    this.camera = new THREE.PerspectiveCamera(45, 1, 0.05, 60);
    // Points-picking threshold for the node sprite cloud (set once; no other
    // Points objects are raycast in this view, so a constant is safe).
    this.raycaster.params.Points.threshold = NODE_RAYCAST_THRESHOLD;
    this.brainGroup.add(this.nodeGroup, this.edgeGroup);
    // The brain mesh's vertical centroid sits below the origin; recenter it.
    this.brainGroup.position.y = 0.08;
    this.scene.add(this.brainGroup);

    this.uniforms.uIntro.value = opts.reducedMotion ? 1 : 0;

    this.loadModel();
    this.addListeners();
    this.resizeObserver = new ResizeObserver(() => this.resize());
    this.resizeObserver.observe(container);
    this.resize();
    this.loop();
  }

  /* ── Model / particles ─────────────────────────────────────── */

  private loadModel() {
    new GLTFLoader().load(
      brainUrl,
      (gltf) => {
        if (this.disposed) return;
        let source: THREE.Mesh | null = null;
        gltf.scene.traverse((child) => {
          if (!source && (child as THREE.Mesh).isMesh) source = child as THREE.Mesh;
        });
        if (!source) {
          this.opts.onError('Brain model failed to parse.');
          return;
        }
        const mesh = source as THREE.Mesh;
        const geometry = mesh.geometry;
        const pos = geometry.getAttribute('position');
        const norm = geometry.getAttribute('normal');
        this.verts = new Float32Array(pos.array as Float32Array);
        this.norms = norm
          ? new Float32Array(norm.array as Float32Array)
          : this.deriveNormals(this.verts);

        this.classifyVertices();

        this.proxy = new THREE.Mesh(geometry, new THREE.MeshBasicMaterial());
        this.proxy.visible = false;
        this.brainGroup.add(this.proxy);

        this.buildParticles(pos.count);
        this.opts.onReady();
      },
      undefined,
      () => this.opts.onError('Brain model failed to load.'),
    );
  }

  private deriveNormals(positions: Float32Array): Float32Array {
    const out = new Float32Array(positions.length);
    const v = new THREE.Vector3();
    for (let i = 0; i < positions.length; i += 3) {
      v.set(positions[i], positions[i + 1], positions[i + 2]).normalize();
      out[i] = v.x;
      out[i + 1] = v.y;
      out[i + 2] = v.z;
    }
    return out;
  }

  /** Partition mesh vertices into left hemisphere, right hemisphere, and stem. */
  private classifyVertices() {
    this.regionVerts = { left: [], right: [], stem: [] };
    const count = this.verts.length / 3;
    if (!count) return;

    let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
    for (let i = 0; i < this.verts.length; i += 3) {
      minX = Math.min(minX, this.verts[i]);
      maxX = Math.max(maxX, this.verts[i]);
      minY = Math.min(minY, this.verts[i + 1]);
      maxY = Math.max(maxY, this.verts[i + 1]);
    }
    const xRange = maxX - minX;
    const yRange = maxY - minY;
    const xCenter = (minX + maxX) / 2;
    // Stem: bottom ~13% of the mesh, within ~12% of the X centre.
    const stemYThreshold = minY + 0.13 * yRange;
    const stemXThreshold = 0.12 * xRange;

    for (let i = 0, idx = 0; i < this.verts.length; i += 3, idx++) {
      const x = this.verts[i];
      const y = this.verts[i + 1];
      if (y < stemYThreshold && Math.abs(x - xCenter) < stemXThreshold) {
        this.regionVerts.stem.push(idx);
      } else if (x < xCenter) {
        this.regionVerts.left.push(idx);
      } else {
        this.regionVerts.right.push(idx);
      }
    }
  }

  /**
   * Pin an atom to a cortex vertex within its assigned region.
   * Falls back to adjacent regions if the primary region is saturated.
   */
  private vertexForRegion(atom: Atom, region: Region): THREE.Vector3 {
    const count = this.verts.length / 3;
    if (!count) return new THREE.Vector3();

    const order: Region[] = [region, region === 'left' ? 'right' : 'left', 'stem'];
    const h = hashId(atom.id);

    for (const candidateRegion of order) {
      const indices = this.regionVerts[candidateRegion];
      if (!indices.length) continue;
      const slot = h % indices.length;
      // Linear-probe for the nearest unused vertex in this region.
      for (let offset = 0; offset < indices.length; offset++) {
        const vertIdx = indices[(slot + offset) % indices.length];
        if (this.usedVerts.has(vertIdx)) continue;
        this.usedVerts.add(vertIdx);
        return this.vertexFromIndex(vertIdx, h);
      }
    }

    // All regions saturated — fall back to the original hash vertex (no exclusivity).
    return this.vertexFromIndex((h % count) * 3, h);
  }

  private vertexFromIndex(vertexNumber: number, h: number): THREE.Vector3 {
    const i = vertexNumber * 3;
    const pos = new THREE.Vector3(this.verts[i], this.verts[i + 1], this.verts[i + 2]);
    const normal = new THREE.Vector3(this.norms[i], this.norms[i + 1], this.norms[i + 2]);
    const jitter = (((h >> 8) % 100) / 100 - 0.5) * 0.03;
    const jitter2 = (((h >> 16) % 100) / 100 - 0.5) * 0.03;
    return pos.add(normal.multiplyScalar(0.018 + jitter)).addScalar(jitter2 * 0.01);
  }

  /**
   * Lattice animation target: the atom's setData-time vertex position, cached.
   * Re-calling vertexForRegion per animation frame would re-probe (and keep
   * growing) usedVerts — at 2500 nodes the regions saturate almost immediately
   * and every subsequent frame degenerates into full-region scans (U4 audit),
   * so the cached position is both faster and stable across frames.
   */
  private latticePos(atom: Atom, region: Region, out: THREE.Vector3): THREE.Vector3 {
    const cached = this.latticePositions.get(atom.id);
    if (cached) return out.copy(cached);
    return out.copy(this.vertexForRegion(atom, region));
  }

  private buildParticles(count: number) {
    const geometry = new THREE.BufferGeometry();

    const rotations = new Float32Array(count);
    const sizes = new Float32Array(count);
    const colors = new Float32Array(count * 3);
    const seeds = new Float32Array(count);
    const color = new THREE.Color();

    this.particleColorIndex = [];
    for (let i = 0; i < count; i++) {
      rotations[i] = Math.random() * 2 - 1;
      sizes[i] = 1.0 + Math.random() * 4.0;
      seeds[i] = Math.random();
      // White is rare; purples dominate.
      const idx = Math.random() < 0.08 ? this.palette.particles.length - 1 : Math.floor(Math.random() * (this.palette.particles.length - 1));
      this.particleColorIndex.push(idx);
      color.setHex(this.palette.particles[idx]);
      colors[i * 3] = color.r;
      colors[i * 3 + 1] = color.g;
      colors[i * 3 + 2] = color.b;
    }

    geometry.setAttribute('aOffset', new THREE.BufferAttribute(this.verts, 3));
    geometry.setAttribute('aRotation', new THREE.BufferAttribute(rotations, 1));
    geometry.setAttribute('aSize', new THREE.BufferAttribute(sizes, 1));
    this.particleColorAttr = new THREE.BufferAttribute(colors, 3);
    geometry.setAttribute('aColor', this.particleColorAttr);
    geometry.setAttribute('aSeed', new THREE.BufferAttribute(seeds, 1));

    const material = new THREE.ShaderMaterial({
      vertexShader: PARTICLE_VERTEX,
      fragmentShader: PARTICLE_FRAGMENT,
      uniforms: this.uniforms,
      transparent: true,
      depthWrite: false,
      blending: THREE.AdditiveBlending,
    });

    const particles = new THREE.Points(geometry, material);
    particles.frustumCulled = false;
    this.brainGroup.add(particles);
  }

  /* ── Memory nodes + synapses ───────────────────────────────── */

  private regionColor(region: Region): number {
    if (region === 'left') return 0x60a5fa; // blue tint
    if (region === 'right') return 0xa855f7; // purple tint
    return 0xffffff; // stem — white
  }

  setData(atoms: Atom[], links: Link[]) {
    // Cancel any in-flight layout transition so a mid-animation refresh
    // doesn't compete with the rebuild below (mirrors dispose()).
    if (this.layoutAnimRaf) { cancelAnimationFrame(this.layoutAnimRaf); this.layoutAnimRaf = 0; }
    const perfT0 = PERF_DEBUG ? performance.now() : 0;
    this.atomsById = new Map(atoms.map((a) => [a.id, a]));
    this.links = links;
    this.sortedLinks = links.slice().sort((a, b) => b.strength - a.strength).slice(0, MAX_EDGES);
    // Notify the consumer before orphaning the React hover tooltip.
    if (this.hoverAtomId) this.opts.onClearHover();
    this.hoverAtomId = null;
    this.hoverConnected = new Set();
    this.zoomAtomId = null;

    // Compute connection counts (degrees) for region assignment.
    this.degrees = new Map();
    links.forEach((link) => {
      this.degrees.set(link.a, (this.degrees.get(link.a) || 0) + 1);
      this.degrees.set(link.b, (this.degrees.get(link.b) || 0) + 1);
    });

    // Hybrid path selection (KTD3 / R4): > cutoff → sprite cloud, else meshes.
    this.spriteMode = Math.min(atoms.length, MAX_NODES) > NODE_SPRITE_CUTOFF;
    // Release the previous cloud whenever we rebuild (mode switch or refresh).
    this.disposeSpriteHoverLabel();
    this.disposeNodeSpriteCloud();
    // Dispose mesh-mode + edge GPU resources before detaching children —
    // clear() alone orphans geometries/materials every refresh.
    this.disposeNodeAndEdgeGroups();
    this.nodeGroup.clear();
    this.edgeGroup.clear();
    this.nodeObjects = [];
    this.nodeMap = new Map();
    this.haloMap = new Map();
    this.labelMap = new Map();
    this.atomPositions = new Map();
    this.latticePositions = new Map();
    this.visibleAtomIds = new Set();
    this._shownIds = [];
    this.spriteRegionOf = new Map();
    this.usedVerts = new Set();
    if (!this.verts.length) return;

    const shown = atoms.slice(0, MAX_NODES);
    this._shownIds = shown.map((a) => a.id);

    // Sort by connection count descending to assign brain regions.
    // Top 10% (hubs) → stem, next 40% (more connected) → right, bottom 50% → left.
    const sorted = [...shown].sort(
      (a, b) => (this.degrees.get(b.id) || 0) - (this.degrees.get(a.id) || 0),
    );
    const n = sorted.length;
    const stemCount = Math.max(1, Math.ceil(n * 0.10));
    const rightCount = Math.ceil(n * 0.40);
    const regionOf: Map<string, Region> = new Map();
    sorted.slice(0, stemCount).forEach((a) => regionOf.set(a.id, 'stem'));
    sorted.slice(stemCount, stemCount + rightCount).forEach((a) => regionOf.set(a.id, 'right'));
    sorted.slice(stemCount + rightCount).forEach((a) => regionOf.set(a.id, 'left'));

    if (this.spriteMode) {
      this.buildNodeSpriteCloud(shown, regionOf);
      this.nodeGroup.visible = false;
    } else {
      this.buildNodeMeshes(shown, regionOf);
      this.nodeGroup.visible = true;
    }

    this.buildEdges();

    // Apply current interactive-control state to the freshly built graph.
    // Sprite mode: applyFilters routes through refreshSpriteAttributes, which
    // already recomputes region colors + alphas — a second applyRegionColors
    // would just redo the same attribute pass.
    this.applyFilters();
    if (!this.spriteMode) this.applyRegionColors();

    // Kick off the force-directed layout sim in the background.
    // Nodes stay at lattice positions until the sim settles; on completion
    // the existing setLayout transition animates to force positions if the
    // user is currently in force mode.
    this.computeForceLayout();

    // Re-apply a non-'brain' layout so live refreshes don't leave new atoms
    // at lattice positions. 'ontology' is handled by finalizeForceLayout; 'galaxy'
    // rebuilds role assignment here. 'brain' needs nothing (lattice is the
    // setData default).
    if (this.layoutMode === 'galaxy') this.applySolarLayout();

    if (PERF_DEBUG) {
      console.log(
        `[brain-perf] setData n=${shown.length} edges=${this.edgeGroup.children.length} sprite=${this.spriteMode}: ${(performance.now() - perfT0).toFixed(1)}ms`,
      );
    }
  }

  /** Shared node radius (degree- and score-scaled) used by both node builders. */
  private nodeRadius(atom: Atom): number {
    const degree = this.degrees.get(atom.id) || 0;
    const base = 0.006 + Math.min(degree, 20) * 0.0012 + Math.min(atom.score, 1) * 0.003;
    return atom.source === 'code' ? base * 0.45 : base;
  }

  /** Sphere+halo+label mesh path (≤ NODE_SPRITE_CUTOFF nodes). Byte-identical to
   *  the original builder; additionally mirrors positions into atomPositions. */
  private buildNodeMeshes(shown: Atom[], regionOf: Map<string, Region>) {
    shown.forEach((atom) => {
      const region = regionOf.get(atom.id) || 'left';
      const radius = this.nodeRadius(atom);
      const mesh = new THREE.Mesh(this.sphereGeo, this.nodeMaterial(false));
      mesh.scale.setScalar(radius);
      mesh.position.copy(this.vertexForRegion(atom, region));
      mesh.userData.atomId = atom.id;
      mesh.userData.region = region;

      const halo = new THREE.Sprite(this.haloMaterial(false));
      halo.scale.setScalar(radius * 3.4);
      halo.position.copy(mesh.position);
      halo.raycast = () => undefined;

      const label = new THREE.Sprite(
        new THREE.SpriteMaterial({
          map: this.makeLabelTexture(atom.title),
          transparent: true,
          depthWrite: false,
          depthTest: true,
        }),
      );
      label.scale.setScalar(0.09);
      label.position.copy(mesh.position);
      label.position.y += radius * 2.2;
      label.visible = false;
      label.raycast = () => undefined;

      this.nodeGroup.add(mesh, halo, label);
      this.nodeObjects.push(mesh);
      this.nodeMap.set(atom.id, mesh);
      this.haloMap.set(atom.id, halo);
      this.labelMap.set(atom.id, label);
      this.atomPositions.set(atom.id, mesh.position.clone());
      this.latticePositions.set(atom.id, mesh.position.clone());
    });
  }

  /** GPU point-sprite cloud path (> NODE_SPRITE_CUTOFF nodes): one draw call. */
  private buildNodeSpriteCloud(shown: Atom[], regionOf: Map<string, Region>) {
    const count = shown.length;
    const positions = new Float32Array(count * 3);
    const sizes = new Float32Array(count);
    const colors = new Float32Array(count * 3);
    const alphas = new Float32Array(count);
    const color = new THREE.Color();

    shown.forEach((atom, i) => {
      const region = regionOf.get(atom.id) || 'left';
      const radius = this.nodeRadius(atom);
      const pos = this.vertexForRegion(atom, region);
      positions[i * 3] = pos.x;
      positions[i * 3 + 1] = pos.y;
      positions[i * 3 + 2] = pos.z;
      // Same degree formula as the sphere radius, converted to the gl_PointSize
      // curve the cortex shader uses (crossover stays visually continuous).
      sizes[i] = radius * NODE_SPRITE_SIZE_SCALE;
      const c = this.showRegions ? this.regionColor(region) : this.palette.nodeBase;
      color.setHex(c);
      colors[i * 3] = color.r;
      colors[i * 3 + 1] = color.g;
      colors[i * 3 + 2] = color.b;
      alphas[i] = 1;

      this.atomPositions.set(atom.id, pos);
      // Separate instance: atomPositions is mutated by layout animations.
      this.latticePositions.set(atom.id, pos.clone());
      this.spriteRegionOf.set(atom.id, region);
    });

    const geometry = new THREE.BufferGeometry();
    this.spritePosAttr = new THREE.BufferAttribute(positions, 3);
    geometry.setAttribute('position', this.spritePosAttr);
    geometry.setAttribute('aSize', new THREE.BufferAttribute(sizes, 1));
    this.spriteColorAttr = new THREE.BufferAttribute(colors, 3);
    geometry.setAttribute('aColor', this.spriteColorAttr);
    this.spriteAlphaAttr = new THREE.BufferAttribute(alphas, 1);
    geometry.setAttribute('aAlpha', this.spriteAlphaAttr);

    const material = new THREE.ShaderMaterial({
      vertexShader: NODE_SPRITE_VERTEX,
      fragmentShader: NODE_SPRITE_FRAGMENT,
      uniforms: this.uniforms,
      transparent: true,
      depthWrite: false,
      blending: THREE.AdditiveBlending,
    });

    const cloud = new THREE.Points(geometry, material);
    cloud.frustumCulled = false;
    this.nodeSpriteCloud = cloud;
    this.brainGroup.add(cloud);
  }

  /** Edge builder — reads endpoint positions from the authoritative
   *  atomPositions map so it works identically in mesh and sprite modes. */
  private buildEdges() {
    this.sortedLinks
      .forEach((link) => {
        const a = this.atomPositions.get(link.a);
        const b = this.atomPositions.get(link.b);
        if (!a || !b) return;

        // Build a surface-conformant Catmull-Rom curve through 4-6
        // intermediate points projected onto the brain mesh surface.
        const points: THREE.Vector3[] = [a.clone()];
        const numIntermediate = 4;
        for (let i = 1; i <= numIntermediate; i++) {
          const t = i / (numIntermediate + 1);
          const p = a.clone().lerp(b, t);

          // U4: sprite mode skips the per-edge surface raycast — 4 raycasts
          // per edge × up to MAX_EDGES edges against the ~5.6k-tri proxy costs
          // ~1-2ms per raycast (multi-second setData at 2500 nodes), blowing
          // the 500ms render budget. At sprite densities surface conformity is
          // invisible (the force/galaxy layouts leave the brain hull anyway),
          // so fall through to the cheap outward lift.
          if (this.proxy && !this.spriteMode) {
            // Raycast from brain center through p onto the mesh surface.
            const worldOrigin = new THREE.Vector3();
            this.brainGroup.localToWorld(worldOrigin);
            const worldP = p.clone();
            this.brainGroup.localToWorld(worldP);
            const dir = worldP.clone().sub(worldOrigin).normalize();
            this.raycaster.set(worldOrigin, dir);
            const hit = this.raycaster.intersectObject(this.proxy, false)[0];
            if (hit) {
              const localHit = hit.point.clone();
              this.brainGroup.worldToLocal(localHit);
              // Push slightly outward along the radial normal.
              const normal = localHit.clone().normalize();
              localHit.add(normal.multiplyScalar(0.005));
              points.push(localHit);
            } else {
              // Fallback: push outward from origin.
              const outward = p.clone().normalize().multiplyScalar(p.length() + 0.01);
              points.push(outward);
            }
          } else {
            // No proxy (or sprite mode) — simple outward lift.
            const lift = p.length() * 0.18;
            points.push(p.add(p.clone().normalize().multiplyScalar(lift)));
          }
        }
        points.push(b.clone());

        const curve = new THREE.CatmullRomCurve3(points);
        const geo = new THREE.BufferGeometry().setFromPoints(curve.getPoints(28));
        const line = new THREE.Line(
          geo,
          new THREE.LineBasicMaterial({
            color: this.palette.edgeRest,
            transparent: true,
            opacity: 0.2,
            depthWrite: false,
          }),
        );
        line.userData = link;
        this.edgeGroup.add(line);
      });
  }

  /** Release the node sprite cloud's GPU resources and clear lookups. */
  private disposeNodeSpriteCloud() {
    if (this.nodeSpriteCloud) {
      this.brainGroup.remove(this.nodeSpriteCloud);
      this.nodeSpriteCloud.geometry.dispose();
      (this.nodeSpriteCloud.material as THREE.Material).dispose();
      this.nodeSpriteCloud = null;
    }
    this.spritePosAttr = undefined;
    this.spriteColorAttr = undefined;
    this.spriteAlphaAttr = undefined;
  }

  /** Dispose mesh-mode node + edge GPU resources before group.clear() detaches
   *  them. sphereGeo is SHARED across all node meshes — never disposed here.
   *  haloTexture is shared across halo sprites; label sprites own their
   *  CanvasTexture maps, which MUST be disposed to avoid per-refresh leaks. */
  private disposeNodeAndEdgeGroups() {
    this.edgeGroup.children.forEach((child) => {
      const line = child as THREE.Line;
      line.geometry?.dispose();
      const mat = line.material as THREE.Material | THREE.Material[] | undefined;
      if (Array.isArray(mat)) mat.forEach((m) => m.dispose());
      else mat?.dispose();
    });
    this.nodeGroup.children.forEach((child) => {
      const mesh = child as THREE.Mesh;
      if (mesh.isMesh) {
        // sphereGeo is shared — dispose material only.
        const mat = mesh.material as THREE.Material | THREE.Material[] | undefined;
        if (Array.isArray(mat)) mat.forEach((m) => m.dispose());
        else mat?.dispose();
        return;
      }
      const sprite = child as THREE.Sprite;
      if (sprite.isSprite) {
        const spriteMat = sprite.material as THREE.SpriteMaterial;
        // haloTexture is shared; label sprites own their CanvasTexture maps.
        if (spriteMat.map && spriteMat.map !== this.haloTexture) spriteMat.map.dispose();
        spriteMat.dispose();
      }
    });
  }

  /** Create the on-demand hover label for a sprite node (destroyed on clear). */
  private showSpriteHoverLabel(atomId: string) {
    const atom = this.atomsById.get(atomId);
    const pos = this.atomPositions.get(atomId);
    if (!atom || !pos) return;
    // Skip the label for nodes hidden by the connectionThreshold filter.
    if (!this.visibleAtomIds.has(atomId)) return;
    this.disposeSpriteHoverLabel();
    const label = new THREE.Sprite(
      new THREE.SpriteMaterial({
        map: this.makeLabelTexture(atom.title),
        transparent: true,
        depthWrite: false,
        depthTest: true,
      }),
    );
    label.scale.setScalar(0.09);
    label.position.copy(pos);
    label.position.y += 0.02;
    label.raycast = () => undefined;
    this.brainGroup.add(label);
    this.spriteHoverLabel = label;
  }

  private disposeSpriteHoverLabel() {
    if (this.spriteHoverLabel) {
      this.brainGroup.remove(this.spriteHoverLabel);
      const mat = this.spriteHoverLabel.material as THREE.SpriteMaterial;
      mat.map?.dispose();
      mat.dispose();
      this.spriteHoverLabel = null;
    }
  }

  /** Recompute sprite aColor/aAlpha from current state (preserves an active hover). */
  private refreshSpriteAttributes() {
    if (!this.spriteMode || !this.spriteColorAttr || !this.spriteAlphaAttr) return;
    if (this.hoverAtomId) {
      this.applySpriteHoverColors(this.hoverAtomId, this.hoverConnected);
    } else {
      this.applySpriteBaseColors();
    }
  }

  /** Base (non-hover) sprite colors/alphas from region palette + filter. */
  private applySpriteBaseColors() {
    if (!this.spriteColorAttr || !this.spriteAlphaAttr) return;
    const colorAttr = this.spriteColorAttr;
    const alphaAttr = this.spriteAlphaAttr;
    const color = new THREE.Color();
    this._shownIds.forEach((id, i) => {
      const visible = this.isAtomVisible(id);
      alphaAttr.setX(i, visible ? 1 : 0);
      const c = this.showRegions ? this.regionColor(this.spriteRegionOf.get(id) ?? 'left') : this.palette.nodeBase;
      color.setHex(c);
      colorAttr.setXYZ(i, color.r, color.g, color.b);
    });
    alphaAttr.needsUpdate = true;
    colorAttr.needsUpdate = true;
  }

  /** Hover-state sprite colors/alphas: hot hovered node, bright connected, dim rest. */
  private applySpriteHoverColors(atomId: string, connected: Set<string>) {
    if (!this.spriteColorAttr || !this.spriteAlphaAttr) return;
    const colorAttr = this.spriteColorAttr;
    const alphaAttr = this.spriteAlphaAttr;
    const color = new THREE.Color();
    this._shownIds.forEach((id, i) => {
      const visible = this.isAtomVisible(id);
      let c: number;
      let alpha: number;
      if (id === atomId) {
        c = this.palette.nodeHot;
        // A hidden hovered node (only reachable via the randomConnection
        // all-hidden fallback) stays hidden — mirrors mesh mode where
        // node.visible=false hides the hovered mesh.
        alpha = visible ? 1 : 0;
      } else if (connected.has(id)) {
        c = this.showRegions ? this.regionColor(this.spriteRegionOf.get(id) ?? 'left') : this.palette.nodeBase;
        alpha = visible ? 0.9 : 0;
      } else {
        c = this.palette.nodeUnfocus;
        alpha = visible ? 0.3 : 0;
      }
      color.setHex(c);
      colorAttr.setXYZ(i, color.r, color.g, color.b);
      alphaAttr.setX(i, alpha);
    });
    colorAttr.needsUpdate = true;
    alphaAttr.needsUpdate = true;
  }

  /** Resolve a pointer raycast to an atom id in whichever path is active. */
  private pickAtomId(): string | null {
    if (this.spriteMode && this.nodeSpriteCloud) {
      const hit = this.raycaster.intersectObject(this.nodeSpriteCloud, false)[0];
      if (hit && hit.index !== undefined) {
        const id = this._shownIds[hit.index];
        // Skip nodes hidden by the connectionThreshold (alpha-0 sprites are
        // still raycast, unlike invisible meshes).
        if (id && this.visibleAtomIds.has(id)) return id;
      }
      return null;
    }
    const nodeHit = this.raycaster.intersectObjects(this.nodeObjects, false)[0]?.object as
      | THREE.Mesh
      | undefined;
    return nodeHit ? (nodeHit.userData.atomId as string) : null;
  }

  private makeLabelTexture(text: string): THREE.Texture {
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d')!;
    const fontSize = 34;
    const font = `${fontSize}px "JetBrains Mono", monospace`;
    ctx.font = font;
    const padding = 10;
    const metrics = ctx.measureText(text);
    canvas.width = Math.ceil(metrics.width + padding * 2);
    canvas.height = fontSize + padding;
    ctx.font = font;
    ctx.textBaseline = 'top';
    ctx.fillStyle = 'rgba(244, 240, 255, 0.92)';
    ctx.fillText(text, padding, padding * 0.5);
    const texture = new THREE.CanvasTexture(canvas);
    texture.minFilter = THREE.LinearFilter;
    return texture;
  }

  /* ── Interactive control setters ───────────────────────────── */

  setConnectionThreshold(threshold: number) {
    this.connectionThreshold = threshold;
    this.applyFilters();
  }

  setZoomLevel(level: number) {
    // level 0–100 → distance 1.0 (close) – 5.0 (far).
    this.manualDistance = 1.0 + (level / 100) * 4.0;
  }

  setShowSynapses(show: boolean) {
    this.showSynapses = show;
    this.applyFilters();
  }

  setShowLabels(show: boolean) {
    this.showLabels = show;
    this.applyFilters();
  }

  setShowRegions(show: boolean) {
    this.showRegions = show;
    this.applyRegionColors();
  }

  /** Smoothly fly the camera to focus on a specific memory node. */
  zoomToAtom(atom: Atom) {
    if (this.atomPositions.has(atom.id)) this.zoomAtomId = atom.id;
  }

  zoomOut() {
    if (this.zoomAtomId) {
      this.zoomAtomId = null;
      this.opts.onZoomOut();
      this.clearHover();
    }
  }

  /** Pick a random node, highlight its connections, and zoom the brain to face it. */
  randomConnection() {
    if (!this._shownIds.length) return;
    const visibleIds = [...this.visibleAtomIds];
    const pool = visibleIds.length ? visibleIds : this._shownIds;
    const atomId = pool[Math.floor(Math.random() * pool.length)];
    const atom = this.atomsById.get(atomId);
    if (!atom) return;
    this.highlightConnections(atomId);
    this.zoomAtomId = atomId;
    this.opts.onSelectAtom(atom);
  }

  /* ── Filtering + region colouring ──────────────────────────── */

  /** Shared degree-visibility predicate (connectionThreshold filter). */
  private isAtomVisible(id: string): boolean {
    return (this.degrees.get(id) || 0) >= this.connectionThreshold;
  }

  private applyFilters() {
    this.visibleAtomIds = new Set();
    if (this.spriteMode) {
      this._shownIds.forEach((id) => {
        if (this.isAtomVisible(id)) this.visibleAtomIds.add(id);
      });
      this.refreshSpriteAttributes();
      // A hovered node filtered to alpha 0 keeps its floating label — drop it.
      if (this.hoverAtomId && !this.visibleAtomIds.has(this.hoverAtomId)) {
        this.disposeSpriteHoverLabel();
      }
    } else {
      this.nodeObjects.forEach((node) => {
        const id = node.userData.atomId as string;
        const visible = this.isAtomVisible(id);
        node.visible = visible;
        if (visible) this.visibleAtomIds.add(id);
        const halo = this.haloMap.get(id);
        if (halo) halo.visible = visible;
        const label = this.labelMap.get(id);
        if (label) label.visible = visible && this.showLabels;
      });
    }
    this.edgeGroup.children.forEach((line) => {
      const link = line.userData as Link;
      (line as THREE.Line).visible =
        this.showSynapses && this.isAtomVisible(link.a) && this.isAtomVisible(link.b);
    });
  }

  private applyRegionColors() {
    if (this.spriteMode) {
      this.refreshSpriteAttributes();
      return;
    }
    if (this.showRegions) {
      this.nodeObjects.forEach((node) => {
        const region = node.userData.region as Region;
        const color = this.regionColor(region);
        (node.material as THREE.MeshBasicMaterial).color.setHex(color);
        (this.haloMap.get(node.userData.atomId as string)!.material as THREE.SpriteMaterial).color.setHex(
          color,
        );
      });
    } else {
      this.nodeObjects.forEach((node) => {
        (node.material as THREE.MeshBasicMaterial).color.setHex(this.palette.nodeBase);
        (this.haloMap.get(node.userData.atomId as string)!.material as THREE.SpriteMaterial).color.setHex(
          this.palette.nodeBase,
        );
      });
    }
  }

  private nodeMaterial(active: boolean) {
    return new THREE.MeshBasicMaterial({
      color: active ? this.palette.nodeHot : this.palette.nodeBase,
      transparent: true,
      opacity: active ? 1 : 0.85,
    });
  }

  private haloMaterial(active: boolean) {
    return new THREE.SpriteMaterial({
      map: this.haloTexture,
      color: active ? this.palette.nodeHot : this.palette.nodeBase,
      transparent: true,
      opacity: active ? 0.7 : 0.32,
      depthWrite: false,
      blending: THREE.AdditiveBlending,
    });
  }

  private makeHaloTexture(): THREE.Texture {
    const size = 64;
    const canvas = document.createElement('canvas');
    canvas.width = canvas.height = size;
    const ctx = canvas.getContext('2d')!;
    const gradient = ctx.createRadialGradient(size / 2, size / 2, 0, size / 2, size / 2, size / 2);
    gradient.addColorStop(0, 'rgba(255,255,255,1)');
    gradient.addColorStop(0.35, 'rgba(255,255,255,.35)');
    gradient.addColorStop(1, 'rgba(255,255,255,0)');
    ctx.fillStyle = gradient;
    ctx.fillRect(0, 0, size, size);
    return new THREE.CanvasTexture(canvas);
  }

  /* ── Hover / selection ─────────────────────────────────────── */

  private highlightConnections(atomId: string) {
    // Already highlighting this atom — skip the (expensive in sprite mode)
    // re-rasterization of the label texture and attribute buffer rewrites.
    if (atomId === this.hoverAtomId) return;
    this.hoverAtomId = atomId;
    const connected = new Set<string>();
    this.links.forEach((link) => {
      if (link.a === atomId) connected.add(link.b);
      else if (link.b === atomId) connected.add(link.a);
    });
    this.hoverConnected = connected;

    if (this.spriteMode) {
      this.applySpriteHoverColors(atomId, connected);
      this.showSpriteHoverLabel(atomId);
    } else {
      const mesh = this.nodeMap.get(atomId);
      if (!mesh) return;
      this.nodeObjects.forEach((node) => {
        const id = node.userData.atomId as string;
        const nodeMat = node.material as THREE.MeshBasicMaterial;
        const haloMat = this.haloMap.get(id)!.material as THREE.SpriteMaterial;
        if (node === mesh) {
          nodeMat.color.setHex(this.palette.nodeHot);
          nodeMat.opacity = 1;
          haloMat.color.setHex(this.palette.nodeHot);
          haloMat.opacity = 0.7;
        } else if (connected.has(id)) {
          const color = this.showRegions
            ? this.regionColor(node.userData.region as Region)
            : this.palette.nodeBase;
          nodeMat.color.setHex(color);
          nodeMat.opacity = 0.9;
          haloMat.color.setHex(color);
          haloMat.opacity = 0.32;
        } else {
          nodeMat.color.setHex(this.palette.nodeUnfocus);
          nodeMat.opacity = 0.3;
          haloMat.opacity = 0.1;
        }
      });
    }

    this.edgeGroup.children.forEach((line) => {
      const link = line.userData as Link;
      const linked = link.a === atomId || link.b === atomId;
      const mat = (line as THREE.Line).material as THREE.LineBasicMaterial;
      mat.opacity = linked ? 0.9 : 0.04;
      mat.color.setHex(linked ? this.palette.edgeActive : this.palette.edgeDim);
    });

    // Route the particle flare through the highlighted memory.
    const pos = this.atomPositions.get(atomId);
    if (pos) {
      this.pointerTarget.copy(pos);
      this.hoverTarget = 1;
    }
  }

  private showHover(atomId: string, event: PointerEvent) {
    this.highlightConnections(atomId);
    const atom = this.atomsById.get(atomId);
    if (atom) this.opts.onHoverAtom(atom, event.clientX, event.clientY);
  }

  private clearHover() {
    if (!this.hoverAtomId) return;
    this.hoverAtomId = null;
    this.hoverConnected = new Set();
    if (this.spriteMode) {
      this.disposeSpriteHoverLabel();
      this.applySpriteBaseColors();
    } else {
      this.nodeObjects.forEach((node) => {
        const nodeMat = node.material as THREE.MeshBasicMaterial;
        const haloMat = this.haloMap.get(node.userData.atomId as string)!.material as THREE.SpriteMaterial;
        const color = this.showRegions
          ? this.regionColor(node.userData.region as Region)
          : this.palette.nodeBase;
        nodeMat.color.setHex(color);
        nodeMat.opacity = 0.85;
        haloMat.color.setHex(color);
        haloMat.opacity = 0.32;
      });
    }
    this.edgeGroup.children.forEach((line) => {
      const mat = (line as THREE.Line).material as THREE.LineBasicMaterial;
      mat.opacity = 0.2;
      mat.color.setHex(this.palette.edgeRest);
    });
    this.opts.onClearHover();
  }

  /** External focus (e.g. search pick): pulse the flare at the atom's vertex. */
  focusAtom(atom: Atom) {
    const pos = this.atomPositions.get(atom.id);
    if (!pos) return;
    this.focusPulse = { pos: pos.clone(), until: performance.now() + 1400 };
  }

  setLayout(mode: LayoutMode) {
    if (this.layoutMode === mode) return;
    this.layoutMode = mode;
    if (mode === 'galaxy') {
      this.applySolarLayout();
    } else {
      this.animateLayoutTo(mode);
    }
  }

  /**
   * Rebuild the solar role assignment (sun/planets/moons/asteroids) from the
   * current atomsById and animate to the 'galaxy' layout. Extracted from
   * setLayout so setData can re-apply it after a live refresh (otherwise new
   * atoms stay at lattice positions in solar mode). Preserves the exact
   * role-assignment logic: degree-descending sort, MAX_PLANETS cap, moon
   * assignment via shared-link planet lookup, and asteroid scatter for the rest.
   */
  private applySolarLayout() {
    const atoms = [...this.atomsById.values()];
    const sorted = [...atoms].sort(
      (a, b) => (this.degrees.get(b.id) || 0) - (this.degrees.get(a.id) || 0),
    );
    this._solarSunId = sorted[0]?.id ?? '';
    const MAX_PLANETS = 14;
    const planets = sorted.slice(1, MAX_PLANETS + 1);
    this._solarPlanets = new Map();
    planets.forEach((atom, i) => {
      const orbitR = 0.8 + (i / Math.max(planets.length - 1, 1)) * 3.5;
      const angle = (i / planets.length) * Math.PI * 2;
      const tilt = (Math.random() - 0.5) * 0.6;
      this._solarPlanets.set(atom.id, { orbitR, angle, tilt });
    });
    const planetIds = new Set(planets.map((a) => a.id));
    const assigned = new Set<string>([this._solarSunId, ...planetIds]);
    this._solarMoons = new Map();
    this._solarAsteroids = new Map();
    const moonCounts = new Map<string, number>();
    sorted.slice(MAX_PLANETS + 1).forEach((atom) => {
      // Find the best planet for this atom (shared link).
      const link = this.links.find(
        (l) => (l.a === atom.id && planetIds.has(l.b)) || (l.b === atom.id && planetIds.has(l.a)),
      );
      if (link) {
        const planetId = planetIds.has(link.b) ? link.b : link.a;
        const mc = (moonCounts.get(planetId) || 0) + 1;
        moonCounts.set(planetId, mc);
        this._solarMoons.set(atom.id, {
          planetId,
          moonR: 0.15 + (mc % 5) * 0.08,
          moonAngle: (mc * 2.4) % (Math.PI * 2),
        });
        assigned.add(atom.id);
      }
    });
    // Remaining atoms become asteroids scattered between orbits.
    let ai = 0;
    atoms.forEach((atom) => {
      if (assigned.has(atom.id)) return;
      const r = 1.2 + Math.random() * 3.0;
      const angle = (ai * 2.399963) % (Math.PI * 2);
      const y = (Math.random() - 0.5) * 0.4;
      this._solarAsteroids.set(atom.id, { r, angle, y });
      ai++;
    });
    this.animateLayoutTo('galaxy');
  }

  /**
   * Eased lerp of every node from its current position to the per-mode target.
   * atomPositions is the authoritative store; mesh.position (mesh mode) and
   * the sprite position attribute (sprite mode) are sinks that mirror it.
   * Extracted from setLayout so the force sim completion can trigger the same
   * transition without re-entering the layoutMode guard.
   */
  private animateLayoutTo(mode: LayoutMode) {
    // Cancel any in-flight layout transition so two RAF loops never compete.
    if (this.layoutAnimRaf) cancelAnimationFrame(this.layoutAnimRaf);
    const DURATION = 850;
    const start = performance.now();
    const froms = new Map<string, THREE.Vector3>();
    this._shownIds.forEach((id) => {
      const p = this.atomPositions.get(id);
      froms.set(id, p ? p.clone() : new THREE.Vector3());
    });
    const animate = () => {
      if (this.disposed) return;
      const t = Math.min((performance.now() - start) / DURATION, 1);
      const e = t < 0.5 ? 2 * t * t : -1 + (4 - 2 * t) * t;
      // Scratch target vector — the per-mode pos functions write into it,
      // avoiding ~n Vector3 allocations per animation frame.
      const to = new THREE.Vector3();
      if (this.spriteMode && this.spritePosAttr) {
        const posAttr = this.spritePosAttr;
        this._shownIds.forEach((id, i) => {
          const atom = this.atomsById.get(id);
          if (!atom) return;
          const from = froms.get(id) ?? new THREE.Vector3();
          const region = this.spriteRegionOf.get(id) || 'left';
          if (mode === 'galaxy') this.solarPos(atom, to);
          else if (mode === 'ontology') this.forcePos(atom, to);
          else this.latticePos(atom, region, to);
          const p = this.atomPositions.get(id);
          if (!p) return;
          p.lerpVectors(from, to, e);
          posAttr.setXYZ(i, p.x, p.y, p.z);
        });
        posAttr.needsUpdate = true;
        // Keep the on-demand hover label glued to its node.
        if (this.spriteHoverLabel && this.hoverAtomId) {
          const hp = this.atomPositions.get(this.hoverAtomId);
          if (hp) {
            this.spriteHoverLabel.position.copy(hp);
            this.spriteHoverLabel.position.y += 0.02;
          }
        }
      } else {
        this.nodeObjects.forEach((mesh) => {
          const atom = this.atomsById.get(mesh.userData.atomId as string);
          if (!atom) return;
          const from = froms.get(atom.id) ?? mesh.position;
          if (mode === 'galaxy') this.solarPos(atom, to);
          else if (mode === 'ontology') this.forcePos(atom, to);
          else this.latticePos(atom, mesh.userData.region as Region, to);
          mesh.position.lerpVectors(from, to, e);
          const halo = this.haloMap.get(atom.id);
          if (halo) halo.position.copy(mesh.position);
          const label = this.labelMap.get(atom.id);
          if (label) { label.position.copy(mesh.position); label.position.y += 0.02; }
          // Mirror the sink back into the authoritative store.
          this.atomPositions.get(atom.id)?.copy(mesh.position);
        });
      }
      if (t < 1) {
        this.layoutAnimRaf = requestAnimationFrame(animate);
      } else {
        this.layoutAnimRaf = 0;
      }
    };
    this.layoutAnimRaf = requestAnimationFrame(animate);
  }

  private solarPos(atom: Atom, out: THREE.Vector3): THREE.Vector3 {
    if (atom.id === this._solarSunId) return out.set(0, 0, 0);
    const planet = this._solarPlanets.get(atom.id);
    if (planet) {
      const y = Math.sin(planet.tilt) * planet.orbitR * 0.3;
      return out.set(
        Math.cos(planet.angle) * planet.orbitR,
        y,
        Math.sin(planet.angle) * planet.orbitR,
      );
    }
    const moon = this._solarMoons.get(atom.id);
    if (moon) {
      // Read from the authoritative store so moons track their planet in
      // both mesh and sprite modes (nodeMap is empty in sprite mode).
      const planetPos = this.atomPositions.get(moon.planetId);
      if (planetPos) {
        return out.set(
          planetPos.x + Math.cos(moon.moonAngle) * moon.moonR,
          planetPos.y + Math.sin(moon.moonAngle * 0.7) * moon.moonR * 0.4,
          planetPos.z + Math.sin(moon.moonAngle) * moon.moonR,
        );
      }
    }
    const asteroid = this._solarAsteroids.get(atom.id);
    if (asteroid) {
      return out.set(
        Math.cos(asteroid.angle) * asteroid.r,
        asteroid.y,
        Math.sin(asteroid.angle) * asteroid.r,
      );
    }
    return out.set(1.5, 0, 0);
  }

  /* ── Force-directed layout ─────────────────────────────────── */

  /** Target position for the force layout — mirrors the solarPos/vertexForRegion pattern. */
  private forcePos(atom: Atom, out: THREE.Vector3): THREE.Vector3 {
    const idx = this._forceIndex.get(atom.id);
    if (idx !== undefined) {
      return out.fromArray(this._forcePosArr, idx * 3);
    }
    // Sim not yet seeded or atom absent — fall back to the lattice position.
    const pos = this.atomPositions.get(atom.id);
    if (pos) return out.copy(pos);
    return out.set(0, 0, 0);
  }

  /** Iterations per idle slice — throttled for large n to keep each slice <500ms (R5). */
  private forceItersPerSlice(n: number): number {
    if (n <= 500) return 40;
    if (n <= 1000) return 15;
    if (n <= 1500) return 8;
    return 3;
  }

  /**
   * Seed the force sim from the current graph and run it in time-sliced chunks.
   * Warm-starts from previously settled positions for atoms that persist across
   * setData() refreshes; new atoms inherit their lattice (vertexForRegion) position.
   * Stale entries (atoms no longer present) are dropped by rebuilding the maps.
   */
  private computeForceLayout() {
    this.cancelForceSim();

    const ids = [...this.atomsById.keys()].slice(0, MAX_NODES);
    const n = ids.length;

    this._forceIds = ids;
    this._forceIndex = new Map(ids.map((id, i) => [id, i]));
    this._forcePosArr = new Float32Array(n * 3);
    this._forceVelArr = new Float32Array(n * 3);
    this._forceIterations = 0;

    // Seed positions: warm start from settled forcePositions, else lattice.
    for (let i = 0; i < n; i++) {
      const id = ids[i];
      const settled = this.forcePositions.get(id);
      if (settled) {
        this._forcePosArr[i * 3] = settled.x;
        this._forcePosArr[i * 3 + 1] = settled.y;
        this._forcePosArr[i * 3 + 2] = settled.z;
      } else {
        const pos = this.atomPositions.get(id);
        if (pos) {
          this._forcePosArr[i * 3] = pos.x;
          this._forcePosArr[i * 3 + 1] = pos.y;
          this._forcePosArr[i * 3 + 2] = pos.z;
        }
      }
      // Deterministic jitter from atom hash (stable across re-seeds, no Math.random).
      const h = hashId(id);
      this._forcePosArr[i * 3] += ((h % 1000) / 1000 - 0.5) * 0.02;
      this._forcePosArr[i * 3 + 1] += (((h >>> 8) % 1000) / 1000 - 0.5) * 0.02;
      this._forcePosArr[i * 3 + 2] += (((h >>> 16) % 1000) / 1000 - 0.5) * 0.02;
    }

    // Build link index pairs for spring forces (same MAX_EDGES cap as the edge renderer).
    this._forceLinkPairs = [];
    for (const link of this.sortedLinks) {
      const ai = this._forceIndex.get(link.a);
      const bi = this._forceIndex.get(link.b);
      if (ai !== undefined && bi !== undefined && ai !== bi) {
        this._forceLinkPairs.push({ a: ai, b: bi, strength: link.strength || 0 });
      }
    }

    if (PERF_DEBUG) {
      this._perfSimT0 = performance.now();
      this._perfSimActive = 0;
    }

    // Trivial cases: 0 or 1 node — no sim needed, just center.
    if (n <= 1) {
      if (n === 1) {
        this._forcePosArr[0] = 0;
        this._forcePosArr[1] = 0;
        this._forcePosArr[2] = 0;
      }
      this.finalizeForceLayout();
      return;
    }

    this.scheduleForceSlice();
  }

  private scheduleForceSlice() {
    this._forceSimHandle = setTimeout(() => {
      this._forceSimHandle = null;
      if (this.disposed) return;
      this.forceSimSlice();
    }, 0);
  }

  /** Run a chunk of sim iterations, then either finalize or schedule the next slice. */
  private forceSimSlice() {
    const n = this._forceIds.length;
    if (n <= 1) {
      this.finalizeForceLayout();
      return;
    }

    const sliceT0 = PERF_DEBUG ? performance.now() : 0;
    const pos = this._forcePosArr;
    const vel = this._forceVelArr;
    const iters = this.forceItersPerSlice(n);
    // U4: cap total iterations lower on big graphs — the settle threshold
    // already early-exits; this only bounds the worst-case wall-clock.
    const maxIters = n > FORCE_BIG_GRAPH_N ? FORCE_MAX_ITERATIONS_BIG_GRAPH : FORCE_MAX_ITERATIONS;
    const maxD2 = FORCE_REPULSION_MAX_DIST * FORCE_REPULSION_MAX_DIST;
    const eps = FORCE_REPULSION_EPSILON;

    for (let iter = 0; iter < iters; iter++) {
      this._forceIterations++;

      // Repulsion: O(n²) inverse-square, capped distance.
      for (let i = 0; i < n; i++) {
        const ix = i * 3, iy = i * 3 + 1, iz = i * 3 + 2;
        const px = pos[ix], py = pos[iy], pz = pos[iz];
        let fx = 0, fy = 0, fz = 0;
        for (let j = i + 1; j < n; j++) {
          const jx = j * 3, jy = j * 3 + 1, jz = j * 3 + 2;
          const dx = px - pos[jx];
          const dy = py - pos[jy];
          const dz = pz - pos[jz];
          const d2 = dx * dx + dy * dy + dz * dz;
          if (d2 >= maxD2 || d2 < eps) continue;
          const d = Math.sqrt(d2);
          const f = FORCE_REPULSION_K / d2;
          const nx = dx / d, ny = dy / d, nz = dz / d;
          fx += nx * f;
          fy += ny * f;
          fz += nz * f;
          vel[jx] -= nx * f;
          vel[jy] -= ny * f;
          vel[jz] -= nz * f;
        }
        vel[ix] += fx;
        vel[iy] += fy;
        vel[iz] += fz;
      }

      // Spring attraction along links (rest length, strength-proportional).
      for (const link of this._forceLinkPairs) {
        const ax = link.a * 3, ay = link.a * 3 + 1, az = link.a * 3 + 2;
        const bx = link.b * 3, by = link.b * 3 + 1, bz = link.b * 3 + 2;
        const dx = pos[bx] - pos[ax];
        const dy = pos[by] - pos[ay];
        const dz = pos[bz] - pos[az];
        const d = Math.sqrt(dx * dx + dy * dy + dz * dz) || eps;
        const displacement = d - FORCE_SPRING_REST;
        const f = FORCE_SPRING_K * displacement * (link.strength || 1);
        const nx = dx / d, ny = dy / d, nz = dz / d;
        vel[ax] += nx * f;
        vel[ay] += ny * f;
        vel[az] += nz * f;
        vel[bx] -= nx * f;
        vel[by] -= ny * f;
        vel[bz] -= nz * f;
      }

      // Centering + damping + position update; track displacement for settle check.
      let totalDisp = 0;
      for (let i = 0; i < n; i++) {
        const ix = i * 3, iy = i * 3 + 1, iz = i * 3 + 2;
        vel[ix] -= pos[ix] * FORCE_CENTER_K;
        vel[iy] -= pos[iy] * FORCE_CENTER_K;
        vel[iz] -= pos[iz] * FORCE_CENTER_K;
        vel[ix] *= FORCE_DAMPING;
        vel[iy] *= FORCE_DAMPING;
        vel[iz] *= FORCE_DAMPING;
        pos[ix] += vel[ix];
        pos[iy] += vel[iy];
        pos[iz] += vel[iz];
        totalDisp += Math.abs(vel[ix]) + Math.abs(vel[iy]) + Math.abs(vel[iz]);
      }

      const avgDisp = totalDisp / (n * 3);
      if (avgDisp < FORCE_SETTLE_THRESHOLD || this._forceIterations >= maxIters) {
        if (PERF_DEBUG) this._perfSimActive += performance.now() - sliceT0;
        this.finalizeForceLayout();
        return;
      }
    }

    if (PERF_DEBUG) this._perfSimActive += performance.now() - sliceT0;
    this.scheduleForceSlice();
  }

  /** Sync flat arrays to the forcePositions map and animate if currently in force mode. */
  private finalizeForceLayout() {
    this.cancelForceSim();
    if (PERF_DEBUG) {
      console.log(
        `[brain-perf] forceSim n=${this._forceIds.length} iters=${this._forceIterations}: wall=${(performance.now() - this._perfSimT0).toFixed(0)}ms active=${this._perfSimActive.toFixed(0)}ms`,
      );
    }
    this.forcePositions = new Map();
    for (let i = 0; i < this._forceIds.length; i++) {
      this.forcePositions.set(
        this._forceIds[i],
        new THREE.Vector3().fromArray(this._forcePosArr, i * 3),
      );
    }
    // If the user is already viewing force mode, animate to the settled positions.
    if (this.layoutMode === 'ontology') {
      this.animateLayoutTo('ontology');
    }
  }

  private cancelForceSim() {
    if (this._forceSimHandle !== null) {
      clearTimeout(this._forceSimHandle);
      this._forceSimHandle = null;
    }
  }

  /* ── Theme ─────────────────────────────────────────────────── */

  setTheme(theme: Theme) {
    this.palette = theme === 'light' ? LIGHT_PALETTE : DARK_PALETTE;
    if (this.particleColorAttr) {
      const color = new THREE.Color();
      this.particleColorIndex.forEach((idx, i) => {
        color.setHex(this.palette.particles[idx]);
        this.particleColorAttr!.setXYZ(i, color.r, color.g, color.b);
      });
      this.particleColorAttr.needsUpdate = true;
    }
    this.clearHoverSilent();
    this.applyRegionColors();
    this.edgeGroup.children.forEach((line) => {
      ((line as THREE.Line).material as THREE.LineBasicMaterial).color.setHex(this.palette.edgeRest);
    });
  }

  private clearHoverSilent() {
    // Notify the consumer so the React hover tooltip isn't orphaned on
    // theme toggles (the only call site of this method).
    if (this.hoverAtomId) this.opts.onClearHover();
    this.hoverAtomId = null;
    this.hoverConnected = new Set();
    this.disposeSpriteHoverLabel();
  }

  /* ── Input ─────────────────────────────────────────────────── */

  private addListeners() {
    const canvas = this.renderer.domElement;
    canvas.addEventListener('pointermove', this.onPointerMove);
    canvas.addEventListener('pointerdown', this.onPointerDown);
    window.addEventListener('pointerup', this.onPointerUp);
    canvas.addEventListener('wheel', this.onWheel, { passive: false });
    canvas.addEventListener('click', this.onClick);
    this.container.addEventListener('keydown', this.onKeydown);
  }

  private removeListeners() {
    const canvas = this.renderer?.domElement;
    if (canvas) {
      canvas.removeEventListener('pointermove', this.onPointerMove);
      canvas.removeEventListener('pointerdown', this.onPointerDown);
      canvas.removeEventListener('wheel', this.onWheel);
      canvas.removeEventListener('click', this.onClick);
    }
    window.removeEventListener('pointerup', this.onPointerUp);
    this.container.removeEventListener('keydown', this.onKeydown);
  }

  private updateNdc(event: PointerEvent | MouseEvent) {
    const rect = this.renderer.domElement.getBoundingClientRect();
    this.pointerNdc.set(
      ((event.clientX - rect.left) / rect.width) * 2 - 1,
      -((event.clientY - rect.top) / rect.height) * 2 + 1,
    );
    this.parallaxTarget.set(this.pointerNdc.x * 0.07, this.pointerNdc.y * 0.045);
  }

  private onPointerMove = (event: PointerEvent) => {
    this.updateNdc(event);
    if (this.dragging && this.last) {
      this.yaw += (event.clientX - this.last.x) * 0.008;
      this.pitch = Math.max(-0.8, Math.min(0.8, this.pitch + (event.clientY - this.last.y) * 0.008));
      this.last = { x: event.clientX, y: event.clientY };
      return;
    }
    this.raycaster.setFromCamera(this.pointerNdc, this.camera);
    const atomId = this.pickAtomId();
    if (atomId) {
      this.showHover(atomId, event);
      return;
    }
    if (this.hoverAtomId) this.clearHover();
    if (this.proxy) {
      const hit = this.raycaster.intersectObject(this.proxy, false)[0];
      if (hit) {
        this.pointerTarget.copy(this.brainGroup.worldToLocal(hit.point.clone()));
        this.hoverTarget = Math.max(this.hoverTarget, 0.65);
        return;
      }
    }
    this.hoverTarget = 0;
  };

  private onPointerDown = (event: PointerEvent) => {
    this.dragging = true;
    this.last = { x: event.clientX, y: event.clientY };
  };

  private onPointerUp = () => {
    this.dragging = false;
    this.last = null;
  };

  private onWheel = (event: WheelEvent) => {
    event.preventDefault();
    this.manualDistance = null;
    this.distance = Math.max(1.0, Math.min(5.0, this.distance + event.deltaY * 0.0012));
  };

  private onClick = (event: MouseEvent) => {
    this.updateNdc(event);
    this.raycaster.setFromCamera(this.pointerNdc, this.camera);
    const atomId = this.pickAtomId();
    const atom = atomId && this.atomsById.get(atomId);
    if (atom) {
      if (this.zoomAtomId === atomId) {
        this.zoomOut();
      } else {
        this.zoomAtomId = atomId;
      }
      this.opts.onSelectAtom(atom);
    } else {
      // Click empty space — return to orbit view.
      this.zoomOut();
    }
  };

  private onKeydown = (event: KeyboardEvent) => {
    if (event.key === 'Escape') {
      this.clearHover();
      this.hoverTarget = 0;
      this.zoomOut();
    }
  };

  /* ── Frame loop ────────────────────────────────────────────── */

  private resize() {
    const rect = this.container.getBoundingClientRect();
    if (!rect.width || !rect.height) return;
    this.camera.aspect = rect.width / rect.height;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(rect.width, rect.height, false);
  }

  private loop = () => {
    if (this.disposed) return;
    const dt = Math.min(this.clock.getDelta(), 0.1);
    const now = performance.now();

    if (this.focusPulse) {
      if (now < this.focusPulse.until) {
        this.pointerTarget.copy(this.focusPulse.pos);
        this.hoverTarget = 1;
      } else {
        this.focusPulse = null;
        this.hoverTarget = 0;
      }
    }

    if (!this.opts.reducedMotion) {
      this.uniforms.uTime.value += dt;
      if (!this.dragging && !this.zoomAtomId) this.brainGroup.rotation.y += dt * 0.05;
      if (this.uniforms.uIntro.value < 1) {
        this.uniforms.uIntro.value = Math.min(1, this.uniforms.uIntro.value + dt * 0.8);
      }
    }

    this.uniforms.uPointer.value.lerp(this.pointerTarget, 0.18);
    this.uniforms.uHover.value += (this.hoverTarget - this.uniforms.uHover.value) * 0.2;
    this.parallax.lerp(this.parallaxTarget, 0.06);

    if (this.manualDistance !== null) {
      this.distance += (this.manualDistance - this.distance) * 0.15;
    }

    // Smoothly fly the camera to the focused node, looking at it.
    // atomPositions is in brainGroup-local space; localToWorld mirrors the
    // mesh.getWorldPosition path used in mesh mode.
    const zoomPos =
      this.zoomAtomId && this.visibleAtomIds.has(this.zoomAtomId)
        ? this.atomPositions.get(this.zoomAtomId)
        : undefined;
    if (zoomPos) {
      const worldPos = this.brainGroup.localToWorld(this._zoomWorld.copy(zoomPos));
      const dir = this._zoomDir.subVectors(this.camera.position, worldPos);
      if (dir.lengthSq() < 0.0001) dir.set(0, 0, 1);
      dir.normalize();
      const desiredCamPos = this._zoomLocal.copy(worldPos).add(dir.multiplyScalar(0.35));
      this.camera.position.lerp(desiredCamPos, 0.06);
      this.camera.lookAt(worldPos);
    } else {
      this.camera.position.set(
        Math.sin(this.yaw) * this.distance + this.parallax.x,
        Math.sin(this.pitch) * this.distance * 0.6 + this.parallax.y,
        Math.cos(this.yaw) * this.distance,
      );
      this.camera.lookAt(0, 0, 0);
    }

    this.renderer.render(this.scene, this.camera);
    this.raf = requestAnimationFrame(this.loop);
  };

  dispose() {
    this.disposed = true;
    cancelAnimationFrame(this.raf);
    cancelAnimationFrame(this.layoutAnimRaf);
    this.cancelForceSim();
    this.resizeObserver?.disconnect();
    this.removeListeners();
    this.haloTexture.dispose();
    this.sphereGeo.dispose();
    // Release hybrid node-sprite resources explicitly (the brainGroup.traverse
    // below would also dispose them, but removing first avoids double-dispose).
    this.disposeSpriteHoverLabel();
    this.disposeNodeSpriteCloud();
    this.brainGroup.traverse((obj) => {
      const mesh = obj as THREE.Mesh;
      if (mesh.isMesh) {
        mesh.geometry?.dispose?.();
        const mat = mesh.material as THREE.Material | THREE.Material[] | undefined;
        if (Array.isArray(mat)) mat.forEach((m) => m.dispose());
        else mat?.dispose?.();
      }
      const points = obj as THREE.Points;
      if (points.isPoints) {
        points.geometry?.dispose?.();
        (points.material as THREE.Material)?.dispose?.();
      }
      const sprite = obj as THREE.Sprite;
      if (sprite.isSprite) {
        const spriteMat = sprite.material as THREE.SpriteMaterial;
        spriteMat.map?.dispose();
        spriteMat.dispose();
      }
    });
    this.renderer?.dispose();
    this.renderer?.domElement.remove();
  }
}
