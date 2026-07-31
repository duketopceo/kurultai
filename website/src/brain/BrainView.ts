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
  float scale = (aSize + c * 10.0 * uHover) * uIntro;
  float drift = uTime * (0.2 + aSeed * 0.3);
  vec3 pos = aOffset;
  pos.x += sin(drift + aSeed * 6.28) * 0.004 * aRotation;
  pos.y += cos(drift * 0.7 + aSeed * 3.14) * 0.004 * aRotation;
  pos.z += sin(drift * 0.5 + aSeed * 4.71) * 0.004 * aRotation;
  vec4 mvPosition = modelViewMatrix * vec4(pos, 1.0);
  gl_Position = projectionMatrix * mvPosition;
  gl_PointSize = scale * (500.0 / -mvPosition.z);
  float flicker = 0.72 + 0.28 * sin(uTime * 2.6 + aSeed * 39.0);
  vColor = mix(aColor, vec3(1.0), c * uHover * 0.9);
  vAlpha = flicker * (0.65 + 0.35 * c * uHover);
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
  private atomsById = new Map<string, Atom>();
  private regionVerts: Record<Region, number[]> = { left: [], right: [], stem: [] };
  private usedVerts = new Set<number>();
  private degrees = new Map<string, number>();
  private layoutMode: LayoutMode = 'regions';
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

  /* ── Interactive control state ─────────────────────────────── */
  private connectionThreshold = 0;
  private showSynapses = true;
  private showLabels = false;
  private showRegions = false;
  private zoomNode: THREE.Mesh | null = null;
  private manualDistance: number | null = null;

  private raycaster = new THREE.Raycaster();
  private pointerNdc = new THREE.Vector2();
  private parallax = new THREE.Vector2();
  private parallaxTarget = new THREE.Vector2();

  private yaw = 0;
  private pitch = 0;
  private distance = 1.75;
  private dragging = false;
  private last: { x: number; y: number } | null = null;
  private hover: THREE.Mesh | null = null;

  private clock = new THREE.Clock();
  private raf = 0;
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
    this.atomsById = new Map(atoms.map((a) => [a.id, a]));
    this.links = links;
    this.hover = null;
    this.zoomNode = null;

    // Compute connection counts (degrees) for region assignment.
    this.degrees = new Map();
    links.forEach((link) => {
      this.degrees.set(link.a, (this.degrees.get(link.a) || 0) + 1);
      this.degrees.set(link.b, (this.degrees.get(link.b) || 0) + 1);
    });

    this.nodeGroup.clear();
    this.edgeGroup.clear();
    this.nodeObjects = [];
    this.nodeMap = new Map();
    this.haloMap = new Map();
    this.labelMap = new Map();
    this.usedVerts = new Set();
    if (!this.verts.length) return;

    const shown = atoms.slice(0, MAX_NODES);

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

    shown.forEach((atom) => {
      const region = regionOf.get(atom.id) || 'left';
      const degree = this.degrees.get(atom.id) || 0;
      const radius = 0.006 + Math.min(degree, 20) * 0.0012 + Math.min(atom.score, 1) * 0.003;
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
    });

    links
      .slice()
      .sort((a, b) => b.strength - a.strength)
      .slice(0, MAX_EDGES)
      .forEach((link) => {
        const a = this.nodeMap.get(link.a);
        const b = this.nodeMap.get(link.b);
        if (!a || !b) return;

        // Build a surface-conformant Catmull-Rom curve through 4-6
        // intermediate points projected onto the brain mesh surface.
        const points: THREE.Vector3[] = [a.position.clone()];
        const numIntermediate = 4;
        for (let i = 1; i <= numIntermediate; i++) {
          const t = i / (numIntermediate + 1);
          const p = a.position.clone().lerp(b.position, t);

          if (this.proxy) {
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
            // No proxy — simple outward lift.
            const lift = p.length() * 0.18;
            points.push(p.add(p.clone().normalize().multiplyScalar(lift)));
          }
        }
        points.push(b.position.clone());

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

    // Apply current interactive-control state to the freshly built graph.
    this.applyFilters();
    this.applyRegionColors();

    // Kick off the force-directed layout sim in the background.
    // Nodes stay at lattice positions until the sim settles; on completion
    // the existing setLayout transition animates to force positions if the
    // user is currently in force mode.
    this.computeForceLayout();
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
    const mesh = this.nodeMap.get(atom.id);
    if (mesh) this.zoomNode = mesh;
  }

  zoomOut() {
    if (this.zoomNode) {
      this.zoomNode = null;
      this.opts.onZoomOut();
      this.clearHover();
    }
  }

  /** Pick a random node, highlight its connections, and zoom the brain to face it. */
  randomConnection() {
    if (!this.nodeObjects.length) return;
    const visibleNodes = this.nodeObjects.filter((n) => n.visible);
    const pool = visibleNodes.length ? visibleNodes : this.nodeObjects;
    const node = pool[Math.floor(Math.random() * pool.length)];
    const atomId = node.userData.atomId as string;
    const atom = this.atomsById.get(atomId);
    if (!atom) return;
    this.highlightConnections(node);
    this.zoomNode = node;
    this.opts.onSelectAtom(atom);
  }

  /* ── Filtering + region colouring ──────────────────────────── */

  private applyFilters() {
    this.nodeObjects.forEach((node) => {
      const id = node.userData.atomId as string;
      const deg = this.degrees.get(id) || 0;
      const visible = deg >= this.connectionThreshold;
      node.visible = visible;
      const halo = this.haloMap.get(id);
      if (halo) halo.visible = visible;
      const label = this.labelMap.get(id);
      if (label) label.visible = visible && this.showLabels;
    });
    this.edgeGroup.children.forEach((line) => {
      const link = line.userData as Link;
      const aDeg = this.degrees.get(link.a) || 0;
      const bDeg = this.degrees.get(link.b) || 0;
      (line as THREE.Line).visible =
        this.showSynapses && aDeg >= this.connectionThreshold && bDeg >= this.connectionThreshold;
    });
  }

  private applyRegionColors() {
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

  private highlightConnections(mesh: THREE.Mesh) {
    this.hover = mesh;
    const hoveredId = mesh.userData.atomId as string;
    const connected = new Set<string>();
    this.links.forEach((link) => {
      if (link.a === hoveredId) connected.add(link.b);
      else if (link.b === hoveredId) connected.add(link.a);
    });

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

    this.edgeGroup.children.forEach((line) => {
      const link = line.userData as Link;
      const linked = link.a === hoveredId || link.b === hoveredId;
      const mat = (line as THREE.Line).material as THREE.LineBasicMaterial;
      mat.opacity = linked ? 0.9 : 0.04;
      mat.color.setHex(linked ? this.palette.edgeActive : this.palette.edgeDim);
    });

    // Route the particle flare through the highlighted memory.
    this.pointerTarget.copy(mesh.position);
    this.hoverTarget = 1;
  }

  private showHover(mesh: THREE.Mesh, event: PointerEvent) {
    this.highlightConnections(mesh);
    const atom = this.atomsById.get(mesh.userData.atomId as string);
    if (atom) this.opts.onHoverAtom(atom, event.clientX, event.clientY);
  }

  private clearHover() {
    if (!this.hover) return;
    this.hover = null;
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
    this.edgeGroup.children.forEach((line) => {
      const mat = (line as THREE.Line).material as THREE.LineBasicMaterial;
      mat.opacity = 0.2;
      mat.color.setHex(this.palette.edgeRest);
    });
    this.opts.onClearHover();
  }

  /** External focus (e.g. search pick): pulse the flare at the atom's vertex. */
  focusAtom(atom: Atom) {
    const mesh = this.nodeMap.get(atom.id);
    if (!mesh) return;
    this.focusPulse = { pos: mesh.position.clone(), until: performance.now() + 1400 };
  }

  setLayout(mode: LayoutMode) {
    if (this.layoutMode === mode) return;
    this.layoutMode = mode;
    const atoms = [...this.atomsById.values()];
    if (mode === 'solar') {
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
    }
    this.animateLayoutTo(mode);
  }

  /**
   * Eased lerp of every mesh from its current position to the per-mode target.
   * Extracted from setLayout so the force sim completion can trigger the same
   * transition without re-entering the layoutMode guard.
   */
  private animateLayoutTo(mode: LayoutMode) {
    const DURATION = 850;
    const start = performance.now();
    const froms = new Map<string, THREE.Vector3>();
    this.nodeObjects.forEach((mesh) => {
      froms.set(mesh.userData.atomId as string, mesh.position.clone());
    });
    const animate = () => {
      if (this.disposed) return;
      const t = Math.min((performance.now() - start) / DURATION, 1);
      const e = t < 0.5 ? 2 * t * t : -1 + (4 - 2 * t) * t;
      this.nodeObjects.forEach((mesh) => {
        const atom = this.atomsById.get(mesh.userData.atomId as string);
        if (!atom) return;
        const from = froms.get(atom.id) ?? mesh.position;
        const to =
          mode === 'solar' ? this.solarPos(atom)
          : mode === 'force' ? this.forcePos(atom)
          : this.vertexForRegion(atom, mesh.userData.region as Region);
        mesh.position.lerpVectors(from, to, e);
        const halo = this.haloMap.get(atom.id);
        if (halo) halo.position.copy(mesh.position);
        const label = this.labelMap.get(atom.id);
        if (label) { label.position.copy(mesh.position); label.position.y += 0.02; }
      });
      if (t < 1) requestAnimationFrame(animate);
    };
    requestAnimationFrame(animate);
  }

  private solarPos(atom: Atom): THREE.Vector3 {
    if (atom.id === this._solarSunId) return new THREE.Vector3(0, 0, 0);
    const planet = this._solarPlanets.get(atom.id);
    if (planet) {
      const y = Math.sin(planet.tilt) * planet.orbitR * 0.3;
      return new THREE.Vector3(
        Math.cos(planet.angle) * planet.orbitR,
        y,
        Math.sin(planet.angle) * planet.orbitR,
      );
    }
    const moon = this._solarMoons.get(atom.id);
    if (moon) {
      const planetMesh = this.nodeMap.get(moon.planetId);
      if (planetMesh) {
        return new THREE.Vector3(
          planetMesh.position.x + Math.cos(moon.moonAngle) * moon.moonR,
          planetMesh.position.y + Math.sin(moon.moonAngle * 0.7) * moon.moonR * 0.4,
          planetMesh.position.z + Math.sin(moon.moonAngle) * moon.moonR,
        );
      }
    }
    const asteroid = this._solarAsteroids.get(atom.id);
    if (asteroid) {
      return new THREE.Vector3(
        Math.cos(asteroid.angle) * asteroid.r,
        asteroid.y,
        Math.sin(asteroid.angle) * asteroid.r,
      );
    }
    return new THREE.Vector3(1.5, 0, 0);
  }

  /* ── Force-directed layout ─────────────────────────────────── */

  /** Target position for the force layout — mirrors the solarPos/vertexForRegion pattern. */
  private forcePos(atom: Atom): THREE.Vector3 {
    const idx = this._forceIndex.get(atom.id);
    if (idx !== undefined) {
      return new THREE.Vector3(
        this._forcePosArr[idx * 3],
        this._forcePosArr[idx * 3 + 1],
        this._forcePosArr[idx * 3 + 2],
      );
    }
    // Sim not yet seeded or atom absent — fall back to the lattice position.
    const mesh = this.nodeMap.get(atom.id);
    if (mesh) return mesh.position.clone();
    return new THREE.Vector3();
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
        const mesh = this.nodeMap.get(id);
        if (mesh) {
          this._forcePosArr[i * 3] = mesh.position.x;
          this._forcePosArr[i * 3 + 1] = mesh.position.y;
          this._forcePosArr[i * 3 + 2] = mesh.position.z;
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
    const sortedLinks = this.links.slice().sort((a, b) => b.strength - a.strength).slice(0, MAX_EDGES);
    for (const link of sortedLinks) {
      const ai = this._forceIndex.get(link.a);
      const bi = this._forceIndex.get(link.b);
      if (ai !== undefined && bi !== undefined && ai !== bi) {
        this._forceLinkPairs.push({ a: ai, b: bi, strength: link.strength || 0 });
      }
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

    const pos = this._forcePosArr;
    const vel = this._forceVelArr;
    const iters = this.forceItersPerSlice(n);
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
      if (avgDisp < FORCE_SETTLE_THRESHOLD || this._forceIterations >= FORCE_MAX_ITERATIONS) {
        this.finalizeForceLayout();
        return;
      }
    }

    this.scheduleForceSlice();
  }

  /** Sync flat arrays to the forcePositions map and animate if currently in force mode. */
  private finalizeForceLayout() {
    this.cancelForceSim();
    this.forcePositions = new Map();
    for (let i = 0; i < this._forceIds.length; i++) {
      this.forcePositions.set(
        this._forceIds[i],
        new THREE.Vector3(
          this._forcePosArr[i * 3],
          this._forcePosArr[i * 3 + 1],
          this._forcePosArr[i * 3 + 2],
        ),
      );
    }
    // If the user is already viewing force mode, animate to the settled positions.
    if (this.layoutMode === 'force') {
      this.animateLayoutTo('force');
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
    this.hover = null;
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
    const nodeHit = this.raycaster.intersectObjects(this.nodeObjects, false)[0]?.object as
      | THREE.Mesh
      | undefined;
    if (nodeHit) {
      this.showHover(nodeHit, event);
      return;
    }
    if (this.hover) this.clearHover();
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
    const hit = this.raycaster.intersectObjects(this.nodeObjects, false)[0]?.object as
      | THREE.Mesh
      | undefined;
    const atom = hit && this.atomsById.get(hit.userData.atomId as string);
    if (atom) {
      if (this.zoomNode === hit) {
        this.zoomOut();
      } else {
        this.zoomNode = hit;
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
      if (!this.dragging && !this.zoomNode) this.brainGroup.rotation.y += dt * 0.05;
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

    if (this.zoomNode && this.zoomNode.visible) {
      // Smoothly fly the camera to the focused node, looking at it.
      const worldPos = new THREE.Vector3();
      this.zoomNode.getWorldPosition(worldPos);
      const dir = new THREE.Vector3().subVectors(this.camera.position, worldPos);
      if (dir.lengthSq() < 0.0001) dir.set(0, 0, 1);
      dir.normalize();
      const desiredCamPos = worldPos.clone().add(dir.multiplyScalar(0.35));
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
    this.cancelForceSim();
    this.resizeObserver?.disconnect();
    this.removeListeners();
    this.haloTexture.dispose();
    this.sphereGeo.dispose();
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
