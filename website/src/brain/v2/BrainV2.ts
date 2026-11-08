/**
 * Brain v2: explorable, optimized wireframe cortex (opt in via `?brain=v2`).
 *
 * Perf design: every atom is one vertex of a single THREE.Points draw call;
 * placement is a deterministic O(n) hash inside the atom's tag region (no FDG);
 * edges exist only for the hovered node and the highlighted path; far zoom
 * shows region glow + counts, near zoom shows individual atoms.
 *
 * Palette: deep black, black/white + slight purple. Ember (#e8a33d) is used
 * ONLY for the selected path and the search target (DESIGN.md accent rule).
 */
import {
  AdditiveBlending, BufferAttribute, BufferGeometry, DoubleSide, LineSegments, Mesh,
  PerspectiveCamera, Points, Scene, ShaderMaterial, Vector3, WebGLRenderer,
} from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import type { Atom, Link } from '../../types.ts';
import { buildAdjacency, shortestPath, type Adjacency } from './path.ts';
import { LOBES, MESH_IDX, MESH_POS, MESH_VERTEX_LOBE } from './cortex.ts';
import { lodTier, parcellate, placeAll, type LodTier, type Parcellation } from './regions.ts';

const FOV = 45;
const START_DISTANCE = 2.85;
const START_DIR = new Vector3(-0.42, 0.5, 1).normalize(); // front-lateral, slightly above: lobes + both hemispheres read
const PICK_RADIUS_PX = 12;
const MAX_LABELS = 10;
const AMBIENT_SYNAPSES = 900;
const SEG = 8; // sub-segments per highlighted synapse (electric jitter)
const LINE_CAP = 1024;

export const FLAG = { none: 0, hover: 1, path: 2, fresh: 3, target: 4 } as const;

export interface BrainV2Options {
  reducedMotion: boolean;
  onHoverAtom: (atom: Atom, clientX: number, clientY: number) => void;
  onSelectAtom: (atom: Atom) => void;
  onClearHover: () => void;
  onReady: () => void;
  onError: (message: string) => void;
}

export interface BrainV2Metrics {
  nodes: number;
  renderedEdges: number;
  fps: number;
  regions: number;
  lod: LodTier;
  drawCalls: number;
}

/* ── atoms: one draw call. Bright additive sprite + 4-ray sparkle, time-based shimmer + spike flashes ── */
const VERT = /* glsl */ `
attribute float aFlag;
attribute float aPhase;
attribute float aImp;
uniform float uSize;
uniform float uLod;
uniform float uTime;
varying float vFlag;
varying float vTw;
varying float vSpike;
void main() {
  vFlag = aFlag;
  vTw = 0.5 + 0.5 * sin(uTime * (1.6 + 2.2 * aPhase) + aPhase * 40.0);
  // occasional spike: a short bright flash a few times a minute per atom
  float g = sin(uTime * (0.35 + 0.5 * aPhase) + aPhase * 91.0);
  vSpike = pow(max(0.0, g), 28.0);
  vec4 mv = modelViewMatrix * vec4(position, 1.0);
  float base = mix(1.0, 2.4, uLod) * (0.6 + 0.9 * aImp);
  float boost = aFlag > 0.5 ? 2.2 : 1.0;
  gl_PointSize = clamp(uSize * base * boost * (1.0 + 1.2 * vSpike) / -mv.z, 2.0, 56.0);
  gl_Position = projectionMatrix * mv;
}`;

const FRAG = /* glsl */ `
uniform float uLod;
varying float vFlag;
varying float vTw;
varying float vSpike;
void main() {
  vec2 c = gl_PointCoord - 0.5;
  float d = length(c) * 2.0;
  if (d > 1.0) discard;
  float core = exp(-d * d * 14.0);
  float halo = pow(1.0 - d, 2.2) * 0.55;
  float rays = (exp(-abs(c.x) * 70.0) + exp(-abs(c.y) * 70.0)) * pow(1.0 - d, 1.5) * (0.25 + 0.9 * vSpike);
  vec3 white = vec3(0.96, 0.94, 1.0);
  vec3 purple = vec3(0.60, 0.46, 1.0);
  vec3 col = mix(purple, white, 0.25 + 0.6 * vTw + 0.5 * vSpike);
  float a = (core * 1.2 + halo * (0.5 + 0.5 * vTw) + rays) * mix(0.8, 1.0, uLod);
  if (vFlag > 0.5 && vFlag < 1.5) { col = white; a = core * 1.4 + halo + rays; }
  if (vFlag > 1.5 && vFlag < 2.5) { col = vec3(0.91, 0.64, 0.24); a = core * 1.4 + halo + rays; }
  if (vFlag > 2.5 && vFlag < 3.5) { col = mix(white, purple, 1.0 - vTw); a = core * 1.3 + halo + rays; }
  if (vFlag > 3.5) { col = vec3(0.91, 0.64, 0.24); a = core * 1.4 + halo + rays; }
  gl_FragColor = vec4(col * a, a);
}`;

/* ── far-zoom region glow ── */
const GLOW_VERT = /* glsl */ `
attribute float aStrength;
uniform float uFar;
varying float vS;
void main() {
  vS = aStrength;
  vec4 mv = modelViewMatrix * vec4(position, 1.0);
  gl_PointSize = clamp((20.0 + 110.0 * aStrength) * uFar / -mv.z, 0.0, 190.0);
  gl_Position = projectionMatrix * mv;
}`;
const GLOW_FRAG = /* glsl */ `
uniform float uFar;
varying float vS;
void main() {
  float d = length(gl_PointCoord - 0.5);
  if (d > 0.5) discard;
  float a = pow(1.0 - d * 2.0, 2.0) * (0.04 + 0.22 * vS) * uFar;
  gl_FragColor = vec4(vec3(0.545, 0.424, 0.941) * a, a);
}`;

/* ── wire shell: thin edges coloured by the owning region's fill; lobe boundaries drawn brighter ── */
const SHELL_VERT = /* glsl */ `
attribute float aFill;
attribute float aEdge;
attribute vec3 aNormal;
uniform float uTime;
varying float vFill;
varying float vEdge;
varying float vShim;
varying float vFace;
void main() {
  vFill = aFill;
  vEdge = aEdge;
  vec4 mv = modelViewMatrix * vec4(position, 1.0);
  // near-side edges read bright, far-side edges recede: kills the wire tangle, keeps the silhouette
  vFace = mix(0.14, 1.0, smoothstep(-0.15, 0.35, dot(normalize(normalMatrix * aNormal), normalize(-mv.xyz))));
  vShim = 0.5 + 0.5 * sin(uTime * 1.3 + position.x * 5.0 + position.y * 4.0 + position.z * 6.0);
  gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
}`;
const SHELL_FRAG = /* glsl */ `
varying float vFill;
varying float vEdge;
varying float vShim;
varying float vFace;
void main() {
  vec3 dim = vec3(0.24, 0.20, 0.38);
  vec3 hot = vec3(0.78, 0.70, 1.0);
  vec3 col = mix(dim, hot, vFill);
  float a = (0.17 + 0.42 * vFill + 0.22 * vEdge * (0.5 + vFill) + 0.12 * vFill * vShim) * vFace;
  gl_FragColor = vec4(col * a, a);
}`;
const FRESNEL_VERT = /* glsl */ `
attribute float aFill;
varying float vFill;
varying float vRim;
void main() {
  vFill = aFill;
  vec4 mv = modelViewMatrix * vec4(position, 1.0);
  vec3 n = normalize(normalMatrix * normal);
  vRim = pow(1.0 - abs(dot(n, normalize(-mv.xyz))), 2.4);
  gl_Position = projectionMatrix * mv;
}`;
const FRESNEL_FRAG = /* glsl */ `
varying float vFill;
varying float vRim;
void main() {
  vec3 col = mix(vec3(0.30, 0.22, 0.60), vec3(0.72, 0.62, 1.0), vFill);
  float a = vRim * (0.10 + 0.45 * vFill) + 0.008 + 0.025 * vFill;
  gl_FragColor = vec4(col * a, a);
}`;

/* ── synapses: electric zig-zag + traveling spike pulses ── */
const SYN_VERT = /* glsl */ `
attribute float aT;
attribute float aSeed;
uniform float uTime;
uniform float uJitter;
varying float vT;
varying float vSeed;
float h(vec2 p) { return fract(sin(dot(p, vec2(12.9898, 78.233))) * 43758.5453); }
void main() {
  vT = aT;
  vSeed = aSeed;
  vec3 p = position;
  float env = sin(3.14159 * aT);
  float tick = floor(uTime * 14.0);
  p += (vec3(h(vec2(aSeed, aT * 9.0 + tick)), h(vec2(aSeed + 1.3, aT * 9.0 + tick)), h(vec2(aSeed + 2.7, aT * 9.0 + tick))) - 0.5) * uJitter * env;
  gl_Position = projectionMatrix * modelViewMatrix * vec4(p, 1.0);
}`;
const SYN_FRAG = /* glsl */ `
uniform float uTime;
uniform vec3 uColor;
uniform float uBase;
uniform float uPulse;
varying float vT;
varying float vSeed;
void main() {
  float flick = 0.55 + 0.45 * sin(uTime * (3.0 + 5.0 * vSeed) + vSeed * 60.0 + vT * 6.0);
  // a pulse runs along the synapse every few seconds, per-synapse phase
  float cyc = fract(uTime * (0.16 + 0.2 * vSeed) + vSeed * 7.0);
  float spike = exp(-pow((vT - cyc * 1.4 + 0.2) / 0.07, 2.0)) * uPulse;
  float a = uBase * flick + spike;
  vec3 col = mix(uColor, vec3(0.97, 0.95, 1.0), clamp(spike, 0.0, 1.0));
  gl_FragColor = vec4(col * a, a);
}`;

function synapseMaterial(color: [number, number, number], base: number, pulse: number, jitter: number): ShaderMaterial {
  return new ShaderMaterial({
    vertexShader: SYN_VERT, fragmentShader: SYN_FRAG, transparent: true, depthWrite: false, blending: AdditiveBlending,
    uniforms: {
      uTime: { value: 0 }, uJitter: { value: jitter }, uBase: { value: base }, uPulse: { value: pulse },
      uColor: { value: new Vector3(...color) },
    },
  });
}

/** Unique triangle edges of the baked cortex as line positions, plus the lobe of each endpoint. */
export function cortexEdges(): { segs: Float32Array; lobeA: Uint8Array; lobeB: Uint8Array; va: Uint16Array; vb: Uint16Array } {
  const seen = new Set<number>();
  const pts: number[] = [];
  const la: number[] = [];
  const lb: number[] = [];
  const va: number[] = [];
  const vb: number[] = [];
  const n = MESH_POS.length / 3;
  for (let f = 0; f < MESH_IDX.length; f += 3) {
    for (let k = 0; k < 3; k++) {
      const a = MESH_IDX[f + k];
      const b = MESH_IDX[f + ((k + 1) % 3)];
      const key = a < b ? a * n + b : b * n + a;
      if (seen.has(key)) continue;
      seen.add(key);
      pts.push(MESH_POS[a * 3], MESH_POS[a * 3 + 1], MESH_POS[a * 3 + 2], MESH_POS[b * 3], MESH_POS[b * 3 + 1], MESH_POS[b * 3 + 2]);
      va.push(a);
      vb.push(b);
      la.push(MESH_VERTEX_LOBE[a]);
      lb.push(MESH_VERTEX_LOBE[b]);
    }
  }
  return { segs: new Float32Array(pts), lobeA: Uint8Array.from(la), lobeB: Uint8Array.from(lb), va: Uint16Array.from(va), vb: Uint16Array.from(vb) };
}

export class BrainV2 {
  private host: HTMLElement;
  private opts: BrainV2Options;
  private renderer!: WebGLRenderer;
  private scene = new Scene();
  private camera = new PerspectiveCamera(FOV, 1, 0.05, 50);
  private controls!: OrbitControls;
  private shell!: LineSegments;
  private shellLobes: { a: Uint8Array; b: Uint8Array };
  private fresnel!: Mesh;
  private ambient: LineSegments | null = null;
  private labelRects: Array<{ x: number; y: number; w: number; h: number }> = [];
  private cloud: Points | null = null;
  private glow: Points | null = null;
  private hoverLines: LineSegments;
  private pathLines: LineSegments;
  private labelLayer: HTMLDivElement;
  private labels: HTMLSpanElement[] = [];
  private labelOrder: number[] = [];
  private ro: ResizeObserver;
  private raf = 0;
  private disposed = false;
  private atoms: Atom[] = [];
  private idIndex = new Map<string, number>();
  private xyz: Float32Array = new Float32Array(0);
  private flags: Float32Array = new Float32Array(0);
  private adj: Adjacency = new Map();
  private parc: Parcellation | null = null;
  private hovered = -1;
  private pathIds: string[] = [];
  private targetId: string | null = null;
  private fresh = new Set<string>();
  private time = 0;
  private lastFrame = performance.now();
  private fpsEma = 60;
  private lod: LodTier = 'far';
  private tween: { t0: number; dur: number; fromP: Vector3; toP: Vector3; fromT: Vector3; toT: Vector3 } | null = null;
  private pointer = { x: 0, y: 0, inside: false, dirty: false };
  private downAt = { x: 0, y: 0 };
  private edgeCount = 0;
  private unsubs: Array<() => void> = [];

  constructor(host: HTMLElement, opts: BrainV2Options) {
    this.host = host;
    this.opts = opts;
    try {
      this.renderer = new WebGLRenderer({ antialias: false, alpha: false, powerPreference: 'low-power' });
    } catch (e) {
      opts.onError(`WebGL unavailable: ${(e as Error).message}`);
      throw e;
    }
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 1.5));
    this.renderer.setClearColor(0x000000, 1); // deep black
    host.appendChild(this.renderer.domElement);
    this.renderer.domElement.style.display = 'block';
    this.renderer.domElement.dataset.brain = 'v2';

    this.camera.position.copy(START_DIR).multiplyScalar(START_DISTANCE);
    this.controls = new OrbitControls(this.camera, this.renderer.domElement);
    this.controls.enableDamping = !opts.reducedMotion;
    this.controls.minDistance = 0.35;
    this.controls.maxDistance = 6;
    this.controls.enablePan = true;

    const edges = cortexEdges();
    const fg = new BufferGeometry();
    fg.setAttribute('position', new BufferAttribute(MESH_POS, 3));
    fg.setIndex(new BufferAttribute(MESH_IDX, 1));
    fg.computeVertexNormals();
    fg.setAttribute('aFill', new BufferAttribute(new Float32Array(MESH_POS.length / 3), 1));
    this.shellLobes = { a: edges.lobeA, b: edges.lobeB };
    const nrm = fg.getAttribute('normal') as BufferAttribute;
    const en = new Float32Array(edges.va.length * 6);
    for (let e = 0; e < edges.va.length; e++) {
      en.set((nrm.array as Float32Array).subarray(edges.va[e] * 3, edges.va[e] * 3 + 3), e * 6);
      en.set((nrm.array as Float32Array).subarray(edges.vb[e] * 3, edges.vb[e] * 3 + 3), e * 6 + 3);
    }
    const shellGeo = new BufferGeometry();
    shellGeo.setAttribute('position', new BufferAttribute(edges.segs, 3));
    shellGeo.setAttribute('aFill', new BufferAttribute(new Float32Array(edges.lobeA.length * 2), 1));
    const boundary = new Float32Array(edges.lobeA.length * 2);
    for (let e = 0; e < edges.lobeA.length; e++) if (edges.lobeA[e] !== edges.lobeB[e]) boundary[e * 2] = boundary[e * 2 + 1] = 1;
    shellGeo.setAttribute('aNormal', new BufferAttribute(en, 3));
    shellGeo.setAttribute('aEdge', new BufferAttribute(boundary, 1));
    this.shell = new LineSegments(shellGeo, new ShaderMaterial({
      vertexShader: SHELL_VERT, fragmentShader: SHELL_FRAG, transparent: true, depthWrite: false, blending: AdditiveBlending,
      uniforms: { uTime: { value: 0 } },
    }));
    this.shell.frustumCulled = false;
    this.scene.add(this.shell);

    this.fresnel = new Mesh(fg, new ShaderMaterial({
      vertexShader: FRESNEL_VERT, fragmentShader: FRESNEL_FRAG, transparent: true, depthWrite: false, blending: AdditiveBlending, side: DoubleSide,
    }));
    this.fresnel.frustumCulled = false;
    this.scene.add(this.fresnel);
    this.paintShell(null);

    this.hoverLines = this.makeLines([0.62, 0.5, 1.0], 0.55, 1.4, 0.014);
    this.pathLines = this.makeLines([0.91, 0.64, 0.24], 0.9, 1.6, 0.012);
    this.scene.add(this.hoverLines, this.pathLines);

    this.labelLayer = document.createElement('div');
    Object.assign(this.labelLayer.style, { position: 'absolute', inset: '0', pointerEvents: 'none', overflow: 'hidden' });
    host.style.position = host.style.position || 'relative';
    host.appendChild(this.labelLayer);

    this.ro = new ResizeObserver(() => this.resize());
    this.ro.observe(host);
    this.resize();

    const el = this.renderer.domElement;
    const move = (e: PointerEvent) => {
      const r = el.getBoundingClientRect();
      this.pointer = { x: e.clientX - r.left, y: e.clientY - r.top, inside: true, dirty: true };
    };
    const leave = () => {
      this.pointer.inside = false;
      this.setHover(-1);
    };
    const down = (e: PointerEvent) => { this.downAt = { x: e.clientX, y: e.clientY }; };
    const up = (e: PointerEvent) => {
      if (Math.hypot(e.clientX - this.downAt.x, e.clientY - this.downAt.y) > 4) return; // a drag, not a click
      const r = el.getBoundingClientRect();
      const i = this.pick(e.clientX - r.left, e.clientY - r.top);
      if (i >= 0) this.opts.onSelectAtom(this.atoms[i]);
    };
    el.addEventListener('pointermove', move);
    el.addEventListener('pointerleave', leave);
    el.addEventListener('pointerdown', down);
    el.addEventListener('pointerup', up);
    this.unsubs.push(() => {
      el.removeEventListener('pointermove', move);
      el.removeEventListener('pointerleave', leave);
      el.removeEventListener('pointerdown', down);
      el.removeEventListener('pointerup', up);
    });

    this.loop = this.loop.bind(this);
    this.raf = requestAnimationFrame(this.loop);
    queueMicrotask(() => { if (!this.disposed) opts.onReady(); });
  }

  /** Electric highlight lines: each synapse is SEG sub-segments so the shader can zig-zag it. */
  private makeLines(color: [number, number, number], base: number, pulse: number, jitter: number): LineSegments {
    const g = new BufferGeometry();
    const verts = LINE_CAP * SEG * 2;
    g.setAttribute('position', new BufferAttribute(new Float32Array(verts * 3), 3));
    g.setAttribute('aT', new BufferAttribute(new Float32Array(verts), 1));
    g.setAttribute('aSeed', new BufferAttribute(new Float32Array(verts), 1));
    g.setDrawRange(0, 0);
    const l = new LineSegments(g, synapseMaterial(color, base, pulse, jitter));
    l.frustumCulled = false;
    return l;
  }

  private resize() {
    const w = Math.max(1, this.host.clientWidth);
    const h = Math.max(1, this.host.clientHeight);
    this.renderer.setSize(w, h, false);
    this.renderer.domElement.style.width = '100%';
    this.renderer.domElement.style.height = '100%';
    this.camera.aspect = w / h;
    this.camera.updateProjectionMatrix();
  }

  /** Each region's fill drives its wire edges + fresnel shell; unassigned lobes stay dim chunks. */
  private paintShell(p: Parcellation | null) {
    const fillOfLobe = new Float32Array(LOBES);
    p?.regions.forEach((r) => { fillOfLobe[r.lobe] = r.fill; });
    const sf = this.shell.geometry.getAttribute('aFill') as BufferAttribute;
    const { a, b } = this.shellLobes;
    for (let e = 0; e < a.length; e++) {
      sf.setX(e * 2, fillOfLobe[a[e]]);
      sf.setX(e * 2 + 1, fillOfLobe[b[e]]);
    }
    sf.needsUpdate = true;
    const ff = this.fresnel.geometry.getAttribute('aFill') as BufferAttribute;
    for (let v = 0; v < MESH_VERTEX_LOBE.length; v++) ff.setX(v, fillOfLobe[MESH_VERTEX_LOBE[v]]);
    ff.needsUpdate = true;
  }

  setData(atoms: Atom[], links: Link[]) {
    this.atoms = atoms;
    this.idIndex = new Map(atoms.map((a, i) => [a.id, i]));
    this.adj = buildAdjacency(links);
    this.edgeCount = links.length;
    this.parc = parcellate(atoms);
    this.xyz = placeAll(atoms, this.parc);
    this.flags = new Float32Array(atoms.length);
    this.paintShell(this.parc);

    if (this.cloud) { this.scene.remove(this.cloud); this.cloud.geometry.dispose(); (this.cloud.material as ShaderMaterial).dispose(); }
    const g = new BufferGeometry();
    g.setAttribute('position', new BufferAttribute(this.xyz, 3));
    g.setAttribute('aFlag', new BufferAttribute(this.flags, 1));
    const phase = new Float32Array(atoms.length);
    for (let i = 0; i < phase.length; i++) phase[i] = ((i * 2654435761) >>> 0) / 4294967296;
    g.setAttribute('aPhase', new BufferAttribute(phase, 1));
    const imp = new Float32Array(atoms.length);
    for (let i = 0; i < imp.length; i++) {
      const a = atoms[i];
      const tier = a.tier === 'hot' ? 1 : a.tier === 'warm' ? 0.6 : 0.3;
      imp[i] = Math.min(1, tier * (0.65 + 0.35 * Math.max(0, Math.min(1, a.score || 0))));
    }
    g.setAttribute('aImp', new BufferAttribute(imp, 1));
    this.cloud = new Points(g, new ShaderMaterial({
      vertexShader: VERT, fragmentShader: FRAG, transparent: true, depthWrite: false, blending: AdditiveBlending,
      uniforms: { uSize: { value: 0 }, uLod: { value: 0 }, uTime: { value: 0 } },
    }));
    this.cloud.frustumCulled = false;
    this.scene.add(this.cloud);
    this.buildAmbient(links);

    if (this.glow) { this.scene.remove(this.glow); this.glow.geometry.dispose(); (this.glow.material as ShaderMaterial).dispose(); }
    const regs = this.parc.regions;
    const maxCount = Math.max(1, ...regs.map((r) => r.count));
    const gg = new BufferGeometry();
    const gp = new Float32Array(regs.length * 3);
    const gs = new Float32Array(regs.length);
    regs.forEach((r, i) => {
      gp.set(r.center, i * 3);
      gs[i] = Math.sqrt(r.count / maxCount);
    });
    gg.setAttribute('position', new BufferAttribute(gp, 3));
    gg.setAttribute('aStrength', new BufferAttribute(gs, 1));
    this.glow = new Points(gg, new ShaderMaterial({
      vertexShader: GLOW_VERT, fragmentShader: GLOW_FRAG, transparent: true, depthWrite: false, blending: AdditiveBlending,
      uniforms: { uFar: { value: 1 } },
    }));
    this.glow.frustumCulled = false;
    this.scene.add(this.glow);

    this.labelLayer.replaceChildren();
    this.labels = regs.map((r) => {
      const el = document.createElement('span');
      el.textContent = `${r.key} \u00b7 ${r.count.toLocaleString()}`;
      Object.assign(el.style, {
        position: 'absolute', left: '0', top: '0', font: '11px ui-monospace, "JetBrains Mono", monospace',
        color: '#d9d2f3', whiteSpace: 'nowrap', transform: 'translate(-50%,-50%)', opacity: '0',
        textShadow: '0 0 6px #000, 0 0 2px #000', letterSpacing: '0.02em',
      });
      this.labelLayer.appendChild(el);
      return el;
    });
    this.labelOrder = regs.map((_, i) => i).filter((i) => regs[i].count > 0).sort((a, b) => regs[b].count - regs[a].count).slice(0, MAX_LABELS);
    this.hovered = -1;
    this.applyFlags();
    this.clearLines(this.hoverLines);
    this.resetView(true);
  }

  /** Faint always-on synapses (strongest links, capped): one draw call, shimmer + spike pulses in the shader. */
  private buildAmbient(links: Link[]) {
    if (this.ambient) { this.scene.remove(this.ambient); this.ambient.geometry.dispose(); (this.ambient.material as ShaderMaterial).dispose(); this.ambient = null; }
    const picked: Array<[number, number]> = [];
    const stride = Math.max(1, Math.ceil(links.length / (AMBIENT_SYNAPSES * 2)));
    const cand = links.filter((_, i) => i % stride === 0).sort((x, y) => (y.strength ?? 0) - (x.strength ?? 0));
    for (const l of cand) {
      const a = this.idIndex.get(l.a);
      const b = this.idIndex.get(l.b);
      if (a === undefined || b === undefined) continue;
      const d = Math.hypot(this.xyz[a * 3] - this.xyz[b * 3], this.xyz[a * 3 + 1] - this.xyz[b * 3 + 1], this.xyz[a * 3 + 2] - this.xyz[b * 3 + 2]);
      if (d > 0.7) continue; // keep synapses local: no long spaghetti across the brain
      picked.push([a, b]);
      if (picked.length >= AMBIENT_SYNAPSES) break;
    }
    if (!picked.length) return;
    const n = picked.length;
    const pos = new Float32Array(n * 6);
    const tt = new Float32Array(n * 2);
    const sd = new Float32Array(n * 2);
    picked.forEach(([a, b], k) => {
      pos.set(this.xyz.subarray(a * 3, a * 3 + 3), k * 6);
      pos.set(this.xyz.subarray(b * 3, b * 3 + 3), k * 6 + 3);
      tt[k * 2] = 0;
      tt[k * 2 + 1] = 1;
      sd[k * 2] = sd[k * 2 + 1] = (((a * 73856093) ^ (b * 19349663)) >>> 0) % 997 / 997;
    });
    const g = new BufferGeometry();
    g.setAttribute('position', new BufferAttribute(pos, 3));
    g.setAttribute('aT', new BufferAttribute(tt, 1));
    g.setAttribute('aSeed', new BufferAttribute(sd, 1));
    this.ambient = new LineSegments(g, synapseMaterial([0.55, 0.45, 1.0], 0.12, 0.9, 0.0));
    this.ambient.frustumCulled = false;
    this.scene.add(this.ambient);
  }

  /* ── flags / highlights ───────────────────────────────────── */
  private applyFlags() {
    if (!this.cloud) return;
    this.flags.fill(0);
    for (const id of this.fresh) { const i = this.idIndex.get(id); if (i !== undefined) this.flags[i] = FLAG.fresh; }
    if (this.hovered >= 0) {
      this.flags[this.hovered] = FLAG.hover;
      for (const n of this.adj.get(this.atoms[this.hovered].id) ?? []) {
        const j = this.idIndex.get(n);
        if (j !== undefined) this.flags[j] = FLAG.hover;
      }
    }
    for (const id of this.pathIds) { const i = this.idIndex.get(id); if (i !== undefined) this.flags[i] = FLAG.path; }
    if (this.targetId) { const i = this.idIndex.get(this.targetId); if (i !== undefined) this.flags[i] = FLAG.target; }
    (this.cloud.geometry.getAttribute('aFlag') as BufferAttribute).needsUpdate = true;
  }

  private fillLines(lines: LineSegments, pairs: Array<[number, number]>) {
    const g = lines.geometry;
    const pos = (g.getAttribute('position') as BufferAttribute).array as Float32Array;
    const tt = (g.getAttribute('aT') as BufferAttribute).array as Float32Array;
    const sd = (g.getAttribute('aSeed') as BufferAttribute).array as Float32Array;
    const n = Math.min(pairs.length, LINE_CAP);
    let v = 0;
    for (let k = 0; k < n; k++) {
      const [a, b] = pairs[k];
      const seed = ((a * 73856093) ^ (b * 19349663)) % 997 / 997;
      for (let q = 0; q < SEG; q++) {
        for (let e = 0; e < 2; e++) {
          const t = (q + e) / SEG;
          for (let c = 0; c < 3; c++) pos[v * 3 + c] = this.xyz[a * 3 + c] + (this.xyz[b * 3 + c] - this.xyz[a * 3 + c]) * t;
          tt[v] = t;
          sd[v] = Math.abs(seed);
          v++;
        }
      }
    }
    (g.getAttribute('position') as BufferAttribute).needsUpdate = true;
    (g.getAttribute('aT') as BufferAttribute).needsUpdate = true;
    (g.getAttribute('aSeed') as BufferAttribute).needsUpdate = true;
    g.setDrawRange(0, v);
  }
  private clearLines(lines: LineSegments) { lines.geometry.setDrawRange(0, 0); }

  private setHover(i: number) {
    if (i === this.hovered) return;
    this.hovered = i;
    if (i < 0) {
      this.clearLines(this.hoverLines);
      this.opts.onClearHover();
    } else {
      const pairs: Array<[number, number]> = [];
      for (const n of this.adj.get(this.atoms[i].id) ?? []) {
        const j = this.idIndex.get(n);
        if (j !== undefined) pairs.push([i, j]);
      }
      this.fillLines(this.hoverLines, pairs);
      const r = this.renderer.domElement.getBoundingClientRect();
      const s = this.project(i);
      this.opts.onHoverAtom(this.atoms[i], r.left + (s?.x ?? 0), r.top + (s?.y ?? 0));
    }
    this.applyFlags();
  }

  /** Highlight a chain of atom ids (ember). Empty clears. */
  setPath(ids: string[]) {
    this.pathIds = ids.filter((id) => this.idIndex.has(id));
    const pairs: Array<[number, number]> = [];
    for (let k = 0; k + 1 < this.pathIds.length; k++) pairs.push([this.idIndex.get(this.pathIds[k])!, this.idIndex.get(this.pathIds[k + 1])!]);
    if (pairs.length) this.fillLines(this.pathLines, pairs); else this.clearLines(this.pathLines);
    this.applyFlags();
  }

  /** Client-side shortest path over the loaded edges (no server traverse API yet). */
  findPath(from: string, to: string): string[] | null {
    return shortestPath(this.adj, from, to);
  }

  setTarget(id: string | null) { this.targetId = id; this.applyFlags(); }
  /** "What's new": light these atoms up (purple-white pulse; ember stays reserved). */
  setFresh(ids: Iterable<string>) { this.fresh = new Set(ids); this.applyFlags(); }

  /* ── camera ───────────────────────────────────────────────── */
  private flyTo(pos: Vector3, target: Vector3, dur = 600) {
    if (this.opts.reducedMotion) {
      this.camera.position.copy(pos);
      this.controls.target.copy(target);
      this.controls.update();
      return;
    }
    this.tween = { t0: performance.now(), dur, fromP: this.camera.position.clone(), toP: pos, fromT: this.controls.target.clone(), toT: target };
  }

  /** Fly to an atom and mark it as the (ember) search target. */
  flyToAtom(id: string): boolean {
    const i = this.idIndex.get(id);
    if (i === undefined) return false;
    const t = new Vector3(this.xyz[i * 3], this.xyz[i * 3 + 1], this.xyz[i * 3 + 2]);
    const dir = t.clone().normalize();
    if (dir.lengthSq() < 1e-6) dir.set(0, 0, 1);
    this.setTarget(id);
    this.flyTo(t.clone().add(dir.multiplyScalar(0.7)), t);
    return true;
  }

  /** Clear search target + path, return to the whole-graph view. */
  resetView(instant = false) {
    this.setTarget(null);
    this.setPath([]);
    const pos = START_DIR.clone().multiplyScalar(START_DISTANCE);
    if (instant) { this.camera.position.copy(pos); this.controls.target.set(0, 0, 0); this.controls.update(); this.tween = null; }
    else this.flyTo(pos, new Vector3(0, 0, 0));
  }

  focusAtom(atom: Atom) { this.flyToAtom(atom.id); }
  zoomToAtom(atom: Atom) { this.flyToAtom(atom.id); }
  randomConnection() {
    if (!this.atoms.length) return;
    const a = this.atoms[Math.floor(Math.random() * this.atoms.length)];
    this.flyToAtom(a.id);
    this.opts.onSelectAtom(a);
  }
  setOntology(_doc: unknown) { /* v2 shows tag regions; ontology lives in its own view */ }
  setLayout(_mode: string) { /* single volumetric layout */ }

  /* ── picking ──────────────────────────────────────────────── */
  private project(i: number): { x: number; y: number; z: number } | null {
    const v = new Vector3(this.xyz[i * 3], this.xyz[i * 3 + 1], this.xyz[i * 3 + 2]).project(this.camera);
    if (v.z > 1) return null;
    const w = this.host.clientWidth;
    const h = this.host.clientHeight;
    return { x: (v.x * 0.5 + 0.5) * w, y: (-v.y * 0.5 + 0.5) * h, z: v.z };
  }

  /** Nearest atom to a screen point, by manual projection (cheap at 50k; no Raycaster). */
  private pick(px: number, py: number): number {
    const n = this.atoms.length;
    if (!n) return -1;
    this.camera.updateMatrixWorld();
    const m = this.camera.projectionMatrix.clone().multiply(this.camera.matrixWorldInverse).elements;
    const w = this.host.clientWidth;
    const h = this.host.clientHeight;
    const xyz = this.xyz;
    let best = -1;
    let bd = PICK_RADIUS_PX * PICK_RADIUS_PX;
    for (let i = 0; i < n; i++) {
      const x = xyz[i * 3], y = xyz[i * 3 + 1], z = xyz[i * 3 + 2];
      const cw = m[3] * x + m[7] * y + m[11] * z + m[15];
      if (cw <= 0) continue;
      const sx = ((m[0] * x + m[4] * y + m[8] * z + m[12]) / cw * 0.5 + 0.5) * w;
      const sy = (-(m[1] * x + m[5] * y + m[9] * z + m[13]) / cw * 0.5 + 0.5) * h;
      const d = (sx - px) ** 2 + (sy - py) ** 2;
      if (d < bd) {
        bd = d;
        best = i;
      }
    }
    return best;
  }

  /* ── loop ─────────────────────────────────────────────────── */
  private loop(now: number) {
    if (this.disposed) return;
    this.raf = requestAnimationFrame(this.loop);
    if (document.hidden) return;
    const dt = Math.min(100, now - this.lastFrame);
    this.lastFrame = now;
    if (dt > 0) this.fpsEma += (1000 / dt - this.fpsEma) * 0.1;
    if (!this.opts.reducedMotion) this.time += dt / 1000;

    if (this.tween) {
      const k = Math.min(1, (now - this.tween.t0) / this.tween.dur);
      const e = 1 - (1 - k) ** 3;
      this.camera.position.lerpVectors(this.tween.fromP, this.tween.toP, e);
      this.controls.target.lerpVectors(this.tween.fromT, this.tween.toT, e);
      if (k >= 1) this.tween = null;
    }
    this.controls.update();

    this.lod = lodTier(this.camera.position.distanceTo(this.controls.target));
    const lodU = this.lod === 'far' ? 0 : this.lod === 'mid' ? 0.55 : 1;
    const h = this.host.clientHeight || 1;
    if (this.cloud) {
      const u = (this.cloud.material as ShaderMaterial).uniforms;
      u.uSize.value = h * 0.017;
      u.uLod.value = lodU;
      u.uTime.value = this.time;
    }
    for (const o of [this.shell, this.ambient, this.hoverLines, this.pathLines]) {
      if (o) (o.material as ShaderMaterial).uniforms.uTime.value = this.time;
    }
    this.fresnel.visible = this.lod !== 'near';
    if (this.glow) {
      const far = this.lod === 'far' ? 1 : this.lod === 'mid' ? 0.45 : 0;
      (this.glow.material as ShaderMaterial).uniforms.uFar.value = far * (h / 700);
      this.glow.visible = far > 0;
    }
    this.updateLabels(this.lod !== 'near');

    if (this.pointer.dirty && this.pointer.inside) {
      this.pointer.dirty = false;
      this.setHover(this.pick(this.pointer.x, this.pointer.y));
    }
    this.renderer.render(this.scene, this.camera);
  }

  /** Top-N region labels only; greedy screen-space collision (largest region wins); fade with depth + facing. */
  private updateLabels(show: boolean) {
    if (!this.parc) return;
    const w = this.host.clientWidth;
    const h = this.host.clientHeight;
    const regs = this.parc.regions;
    const shown = new Set<number>();
    this.labelRects.length = 0;
    const cam = this.camera.position;
    const camLen = cam.length() || 1;
    if (show) {
      for (const i of this.labelOrder) {
        const r = regs[i];
        const c = new Vector3(...r.center);
        const facing = (c.x * cam.x + c.y * cam.y + c.z * cam.z) / ((c.length() || 1) * camLen);
        const fade = Math.max(0, Math.min(1, (facing + 0.1) / 0.5));
        if (fade <= 0.02) continue;
        const v = c.project(this.camera);
        if (v.z > 1) continue;
        const x = (v.x * 0.5 + 0.5) * w;
        const y = (-v.y * 0.5 + 0.5) * h;
        const tw = this.labels[i].textContent!.length * 6.6 + 10;
        const rect = { x: x - tw / 2, y: y - 9, w: tw, h: 18 };
        if (rect.x < 4 || rect.x + rect.w > w - 4 || rect.y < 4 || rect.y + rect.h > h - 4) continue;
        if (this.labelRects.some((o) => rect.x < o.x + o.w + 6 && rect.x + rect.w + 6 > o.x && rect.y < o.y + o.h + 4 && rect.y + rect.h + 4 > o.y)) continue;
        this.labelRects.push(rect);
        const el = this.labels[i];
        el.style.left = `${x}px`;
        el.style.top = `${y}px`;
        el.style.opacity = String(0.9 * fade * (this.lod === 'far' ? 1 : 0.6));
        shown.add(i);
      }
    }
    this.labels.forEach((el, i) => { if (!shown.has(i)) el.style.opacity = '0'; });
  }

  metrics(): BrainV2Metrics {
    return {
      nodes: this.atoms.length,
      renderedEdges: this.edgeCount,
      fps: Math.round(this.fpsEma),
      regions: this.parc?.regions.length ?? 0,
      lod: this.lod,
      drawCalls: this.renderer.info.render.calls,
    };
  }

  dispose() {
    this.disposed = true;
    cancelAnimationFrame(this.raf);
    this.ro.disconnect();
    this.unsubs.forEach((u) => u());
    this.controls.dispose();
    this.scene.traverse((o) => {
      const m = o as unknown as { geometry?: BufferGeometry; material?: { dispose(): void } };
      m.geometry?.dispose();
      m.material?.dispose();
    });
    this.renderer.dispose();
    this.renderer.domElement.remove();
    this.labelLayer.remove();
  }
}
