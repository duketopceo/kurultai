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
  float scale = (aSize + c * 7.0 * uHover) * uIntro;
  float drift = uTime * (0.2 + aSeed * 0.3);
  vec3 pos = aOffset;
  pos.x += sin(drift + aSeed * 6.28) * 0.003 * aRotation;
  pos.y += cos(drift * 0.7 + aSeed * 3.14) * 0.003 * aRotation;
  pos.z += sin(drift * 0.5 + aSeed * 4.71) * 0.003 * aRotation;
  vec4 mvPosition = modelViewMatrix * vec4(pos, 1.0);
  gl_Position = projectionMatrix * mvPosition;
  gl_PointSize = scale * (300.0 / -mvPosition.z);
  float flicker = 0.72 + 0.28 * sin(uTime * 2.6 + aSeed * 39.0);
  vColor = mix(aColor, vec3(1.0), c * uHover * 0.9);
  vAlpha = flicker * (0.55 + 0.45 * c * uHover);
}
`;

const PARTICLE_FRAGMENT = /* glsl */ `
varying vec3 vColor;
varying float vAlpha;
void main() {
  float dist = distance(gl_PointCoord, vec2(0.5));
  if (dist > 0.5) discard;
  float soft = 1.0 - smoothstep(0.2, 0.5, dist);
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

const MAX_NODES = 450;
const MAX_EDGES = 800;

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
  private _solarSources: string[] = [];
  private _solarSiblings = new Map<string, string[]>();

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
      sizes[i] = 0.3 + Math.random() * 2.7;
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
      const radius = 0.0095 + Math.min(atom.score, 1) * 0.0075;
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
      const sorted = [...atoms].sort((a, b) => (b.score ?? 0) - (a.score ?? 0));
      this._solarSunId = sorted[0]?.id ?? '';
      const sourceSet = new Set(atoms.map((a) => a.source));
      this._solarSources = [...sourceSet];
      this._solarSiblings = new Map();
      this._solarSources.forEach((src) => {
        this._solarSiblings.set(src, atoms.filter((a) => a.source === src).map((a) => a.id));
      });
    }
    const DURATION = 850;
    const start = performance.now();
    const froms = new Map<string, THREE.Vector3>();
    this.nodeObjects.forEach((mesh) => {
      froms.set(mesh.userData.atomId as string, mesh.position.clone());
    });
    const animate = () => {
      const t = Math.min((performance.now() - start) / DURATION, 1);
      const e = t < 0.5 ? 2 * t * t : -1 + (4 - 2 * t) * t;
      this.nodeObjects.forEach((mesh, idx) => {
        const atom = this.atomsById.get(mesh.userData.atomId as string);
        if (!atom) return;
        const from = froms.get(atom.id) ?? mesh.position;
        const to = mode === 'solar' ? this.solarPos(atom) : this.vertexForRegion(atom, mesh.userData.region as Region);
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
    const srcIdx = Math.max(0, this._solarSources.indexOf(atom.source));
    const orbit = 2.5 + (srcIdx / Math.max(this._solarSources.length - 1, 1)) * 5;
    const siblings = this._solarSiblings.get(atom.source) ?? [];
    const i = Math.max(0, siblings.indexOf(atom.id));
    const n = Math.max(siblings.length, 1);
    const angle = (i / n) * Math.PI * 2 + srcIdx * 0.4;
    const y = ((i % 5) - 2) * 0.15;
    return new THREE.Vector3(Math.cos(angle) * orbit, y, Math.sin(angle) * orbit);
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
