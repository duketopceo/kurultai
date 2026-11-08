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
  AdditiveBlending, BufferAttribute, BufferGeometry, Color, IcosahedronGeometry, LineSegments,
  LineBasicMaterial, PerspectiveCamera, Points, Scene, ShaderMaterial, Vector3, WebGLRenderer,
} from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import type { Atom, Link } from '../../types.ts';
import { buildAdjacency, shortestPath, type Adjacency } from './path.ts';
import { HULL, lodTier, parcellate, placeAll, type LodTier, type Parcellation, nearestCenter } from './regions.ts';

const EMBER = 0xe8a33d;
const PURPLE = 0x8b6cf0;
const FOV = 45;
const START_DISTANCE = 3.1;
const PICK_RADIUS_PX = 12;

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

const VERT = /* glsl */ `
attribute float aFlag;
attribute float aPhase;
uniform float uSize;
uniform float uLod;
uniform float uTime;
varying float vFlag;
varying float vTw;
void main() {
  vFlag = aFlag;
  vTw = 0.5 + 0.5 * sin(uTime * 2.1 + aPhase * 6.2831);
  vec4 mv = modelViewMatrix * vec4(position, 1.0);
  float base = mix(1.1, 2.4, uLod);
  float boost = aFlag > 0.5 ? 2.2 : 1.0;
  gl_PointSize = clamp(uSize * base * boost / -mv.z, 1.0, 48.0);
  gl_Position = projectionMatrix * mv;
}`;

const FRAG = /* glsl */ `
uniform float uLod;
varying float vFlag;
varying float vTw;
void main() {
  vec2 c = gl_PointCoord - 0.5;
  float d = length(c);
  if (d > 0.5) discard;
  float core = smoothstep(0.5, 0.0, d);
  vec3 white = vec3(0.92, 0.9, 0.96);
  vec3 purple = vec3(0.545, 0.424, 0.941);
  vec3 col = mix(purple, white, 0.35 + 0.5 * vTw);
  float a = core * mix(0.30, 0.95, uLod);
  if (vFlag > 0.5 && vFlag < 1.5) { col = white; a = core; }
  if (vFlag > 1.5 && vFlag < 2.5) { col = vec3(0.91, 0.64, 0.24); a = core; }
  if (vFlag > 2.5 && vFlag < 3.5) { col = mix(white, purple, 1.0 - vTw); a = core; }
  if (vFlag > 3.5) { col = vec3(0.91, 0.64, 0.24); a = core; }
  gl_FragColor = vec4(col * a, a);
}`;

const GLOW_VERT = /* glsl */ `
attribute float aStrength;
uniform float uFar;
varying float vS;
void main() {
  vS = aStrength;
  vec4 mv = modelViewMatrix * vec4(position, 1.0);
  gl_PointSize = clamp((24.0 + 150.0 * aStrength) * uFar / -mv.z, 0.0, 220.0);
  gl_Position = projectionMatrix * mv;
}`;
const GLOW_FRAG = /* glsl */ `
uniform float uFar;
varying float vS;
void main() {
  float d = length(gl_PointCoord - 0.5);
  if (d > 0.5) discard;
  float a = pow(1.0 - d * 2.0, 2.0) * (0.10 + 0.32 * vS) * uFar;
  gl_FragColor = vec4(vec3(0.545, 0.424, 0.941) * a, a);
}`;

/** Low-poly brain shell: deformed icosphere with a longitudinal fissure + gyri ripple. */
export function brainShellSegments(detail = 2): Float32Array {
  const geo = new IcosahedronGeometry(1, detail);
  const pos = geo.getAttribute('position');
  const p = new Vector3();
  const warp = (v: Vector3) => {
    const x = v.x * HULL.x;
    const y = v.y * HULL.y;
    const z = v.z * HULL.z;
    const ripple = 1 + 0.035 * Math.sin(7 * v.x + 1) * Math.sin(6 * v.y) * Math.sin(5 * v.z + 2);
    let wx = x * ripple;
    const wy = y * ripple;
    const wz = z * ripple;
    if (wy > 0.15) wx += (wx >= 0 ? 1 : -1) * 0.05 * Math.min(1, (wy - 0.15) * 4);
    v.set(wx, wy, wz);
  };
  const seen = new Set<string>();
  const out: number[] = [];
  const key = (v: Vector3) => `${Math.round(v.x * 1e4)},${Math.round(v.y * 1e4)},${Math.round(v.z * 1e4)}`;
  const tri = [new Vector3(), new Vector3(), new Vector3()];
  for (let f = 0; f < pos.count; f += 3) {
    for (let k = 0; k < 3; k++) {
      p.fromBufferAttribute(pos, f + k);
      warp(p);
      tri[k].copy(p);
    }
    for (let k = 0; k < 3; k++) {
      const a = tri[k];
      const b = tri[(k + 1) % 3];
      const ka = key(a);
      const kb = key(b);
      const ek = ka < kb ? `${ka}|${kb}` : `${kb}|${ka}`;
      if (seen.has(ek)) continue;
      seen.add(ek);
      out.push(a.x, a.y, a.z, b.x, b.y, b.z);
    }
  }
  geo.dispose();
  return new Float32Array(out);
}

export class BrainV2 {
  private host: HTMLElement;
  private opts: BrainV2Options;
  private renderer!: WebGLRenderer;
  private scene = new Scene();
  private camera = new PerspectiveCamera(FOV, 1, 0.05, 50);
  private controls!: OrbitControls;
  private shell!: LineSegments;
  private shellSegs: Float32Array;
  private cloud: Points | null = null;
  private glow: Points | null = null;
  private hoverLines: LineSegments;
  private pathLines: LineSegments;
  private labelLayer: HTMLDivElement;
  private labels: HTMLSpanElement[] = [];
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

    this.camera.position.set(0.9, 0.7, START_DISTANCE);
    this.controls = new OrbitControls(this.camera, this.renderer.domElement);
    this.controls.enableDamping = !opts.reducedMotion;
    this.controls.minDistance = 0.35;
    this.controls.maxDistance = 6;
    this.controls.enablePan = true;

    this.shellSegs = brainShellSegments(2);
    const shellGeo = new BufferGeometry();
    shellGeo.setAttribute('position', new BufferAttribute(this.shellSegs, 3));
    shellGeo.setAttribute('color', new BufferAttribute(new Float32Array(this.shellSegs.length), 3));
    this.shell = new LineSegments(
      shellGeo,
      new LineBasicMaterial({ vertexColors: true, blending: AdditiveBlending, depthWrite: false, transparent: true }),
    );
    this.scene.add(this.shell);
    this.paintShell(null);

    this.hoverLines = this.makeLines(PURPLE, 0.8);
    this.pathLines = this.makeLines(EMBER, 1);
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

  private makeLines(color: number, opacity: number): LineSegments {
    const g = new BufferGeometry();
    g.setAttribute('position', new BufferAttribute(new Float32Array(4096 * 6), 3));
    g.setDrawRange(0, 0);
    const l = new LineSegments(g, new LineBasicMaterial({ color, transparent: true, opacity, blending: AdditiveBlending, depthWrite: false }));
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

  /** Wire chunks: each shell edge takes its region's brightness; fuller regions glow more. */
  private paintShell(p: Parcellation | null) {
    const segs = this.shellSegs;
    const col = this.shell.geometry.getAttribute('color') as BufferAttribute;
    const base = new Color(0x2a2540);
    const hot = new Color(0xb8a6ff);
    const centers = p?.regions.map((r) => r.center) ?? [];
    const c = new Color();
    for (let s = 0; s < segs.length; s += 6) {
      let fill = 0;
      if (p && centers.length) {
        const mx = (segs[s] + segs[s + 3]) / 2;
        const my = (segs[s + 1] + segs[s + 4]) / 2;
        const mz = (segs[s + 2] + segs[s + 5]) / 2;
        fill = p.regions[nearestCenter(mx, my, mz, centers)].fill;
      }
      c.copy(base).lerp(hot, 0.1 + 0.5 * fill * fill).multiplyScalar(0.1 + 0.3 * fill);
      for (let k = 0; k < 2; k++) col.setXYZ(s / 3 + k, c.r, c.g, c.b);
    }
    col.needsUpdate = true;
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
    this.cloud = new Points(g, new ShaderMaterial({
      vertexShader: VERT, fragmentShader: FRAG, transparent: true, depthWrite: false, blending: AdditiveBlending,
      uniforms: { uSize: { value: 0 }, uLod: { value: 0 }, uTime: { value: 0 } },
    }));
    this.cloud.frustumCulled = false;
    this.scene.add(this.cloud);

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
      const s = document.createElement('span');
      s.textContent = `${r.key} ${r.count.toLocaleString()}`;
      Object.assign(s.style, {
        position: 'absolute', left: '0', top: '0', font: '11px ui-monospace, "JetBrains Mono", monospace',
        color: '#9a968c', whiteSpace: 'nowrap', transform: 'translate(-50%,-50%)', opacity: '0',
      });
      this.labelLayer.appendChild(s);
      return s;
    });
    this.hovered = -1;
    this.applyFlags();
    this.clearLines(this.hoverLines);
    this.resetView(true);
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
    const attr = lines.geometry.getAttribute('position') as BufferAttribute;
    const arr = attr.array as Float32Array;
    const n = Math.min(pairs.length, 4096);
    for (let k = 0; k < n; k++) {
      const [a, b] = pairs[k];
      arr.set(this.xyz.subarray(a * 3, a * 3 + 3), k * 6);
      arr.set(this.xyz.subarray(b * 3, b * 3 + 3), k * 6 + 3);
    }
    attr.needsUpdate = true;
    lines.geometry.setDrawRange(0, n * 2);
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
    const pos = new Vector3(0.9, 0.7, START_DISTANCE).setLength(START_DISTANCE);
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
      u.uSize.value = h * 0.012;
      u.uLod.value = lodU;
      u.uTime.value = this.time;
    }
    if (this.glow) {
      const far = this.lod === 'far' ? 1 : this.lod === 'mid' ? 0.45 : 0;
      (this.glow.material as ShaderMaterial).uniforms.uFar.value = far * (h / 700);
      this.glow.visible = far > 0;
    }
    this.updateLabels(this.lod === 'far');

    if (this.pointer.dirty && this.pointer.inside) {
      this.pointer.dirty = false;
      this.setHover(this.pick(this.pointer.x, this.pointer.y));
    }
    this.renderer.render(this.scene, this.camera);
  }

  private updateLabels(show: boolean) {
    if (!this.parc) return;
    const w = this.host.clientWidth;
    const h = this.host.clientHeight;
    this.parc.regions.forEach((r, i) => {
      const el = this.labels[i];
      if (!el) return;
      if (!show) { el.style.opacity = '0'; return; }
      const c = new Vector3(...r.center);
      const facing = c.dot(this.camera.position) > 0; // hide labels on the far side of the hull
      const v = c.project(this.camera);
      el.style.opacity = v.z > 1 || !facing || i >= 14 ? '0' : '0.85';
      el.style.left = `${(v.x * 0.5 + 0.5) * w}px`;
      el.style.top = `${(-v.y * 0.5 + 0.5) * h}px`;
    });
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
