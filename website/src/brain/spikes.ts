import * as THREE from 'three';

/*
 * Traveling axon spikes (plan 2026-09-12-001, U2/U3/KTD4).
 * The neural behavior signature is firing: sparse, discrete, directed pulses
 * — not ambient twinkle. `heatOf`/`emissionRate`/`nextEmission` are pure so
 * node:test can cover the encoding; `SpikePool` is the thin GL side (a fixed
 * THREE.Points cloud, no per-frame allocation).
 */

/** Minimal atom shape for heat scoring — matches `Atom`'s fields. */
export interface HeatInput {
  score?: number;
  tier?: string;
  last_accessed_at?: string;
}

const TIER_HEAT: Record<string, number> = { hot: 1, warm: 0.55, cold: 0.25 };
/** Recency half-life in days — a note touched today fires hot, one untouched
 *  for a month contributes almost nothing. */
const RECENCY_HALF_LIFE_DAYS = 30;
const DEGREE_SATURATION = 20;

/** Activity heat 0..1: score + tier + access recency + connectivity. */
export function heatOf(atom: HeatInput, now: number, degree: number): number {
  const score = Math.min(Math.max(atom.score ?? 0, 0), 1);
  const tier = TIER_HEAT[atom.tier ?? ''] ?? 0.4;
  let recency = 0;
  const accessed = atom.last_accessed_at ? Date.parse(atom.last_accessed_at) : NaN;
  if (Number.isFinite(accessed)) {
    const ageDays = Math.max(0, (now - accessed) / 86_400_000);
    recency = Math.pow(0.5, ageDays / RECENCY_HALF_LIFE_DAYS);
  }
  const deg = Math.min(Math.max(degree, 0), DEGREE_SATURATION) / DEGREE_SATURATION;
  const heat = score * 0.35 + tier * 0.3 + recency * 0.25 + deg * 0.1;
  return Math.min(Math.max(heat, 0), 1);
}

export type EdgeHover = 'none' | 'towardA' | 'towardB' | 'unrelated';

/** Spikes/sec an edge emits. A floor keeps the cortex faintly alive; hover
 *  stimulation saturates connected edges; unrelated edges go quiet. */
export function emissionRate(heatA: number, heatB: number, hover: EdgeHover): number {
  if (hover === 'unrelated') return 0.02;
  if (hover === 'towardA' || hover === 'towardB') return 2.5;
  return 0.05 + (heatA + heatB) * 0.5;
}

/** Fractional accumulator emission — deterministic (no rng), so tests can
 *  step time exactly. Call each tick; `fired` means emit one spike now. */
export function nextEmission(
  acc: number,
  rate: number,
  dt: number,
): { acc: number; fired: boolean } {
  const next = acc + rate * dt;
  if (next >= 1) return { acc: next - 1, fired: true };
  return { acc: next, fired: false };
}

const SPIKE_VERTEX = /* glsl */ `
attribute float aSize;
attribute float aAlpha;
varying float vAlpha;
void main() {
  vec4 mvPosition = modelViewMatrix * vec4(position, 1.0);
  gl_Position = projectionMatrix * mvPosition;
  gl_PointSize = aSize * (500.0 / -mvPosition.z);
  vAlpha = aAlpha;
}
`;

const SPIKE_FRAGMENT = /* glsl */ `
uniform vec3 uColor;
varying float vAlpha;
void main() {
  float dist = distance(gl_PointCoord, vec2(0.5));
  if (dist > 0.5) discard;
  float soft = 1.0 - smoothstep(0.15, 0.5, dist);
  gl_FragColor = vec4(uColor, vAlpha * soft);
}
`;

export const SPIKE_POOL_CAPACITY = 150;
/** World-ish size on the cortex gl_PointSize curve. */
const SPIKE_SIZE = 0.09;
/** Traversal duration in seconds per unit of curve length — shorter edges
 *  conduct faster, like real axons. */
const SPIKE_SPEED = 0.9;

/** Fixed-size pool of spikes riding edge curves. Free slots have aAlpha 0. */
export class SpikePool {
  readonly points: THREE.Points;
  private posAttr: THREE.BufferAttribute;
  private alphaAttr: THREE.BufferAttribute;
  private sizeAttr: THREE.BufferAttribute;
  private free: number[] = [];
  private activeCount = 0;
  // Per active slot: which edge curve it rides, progress 0→1, and direction
  // (+1 a→b, -1 b→a).
  private slotEdge = new Int32Array(SPIKE_POOL_CAPACITY);
  private slotT = new Float32Array(SPIKE_POOL_CAPACITY);
  private slotDir = new Int8Array(SPIKE_POOL_CAPACITY);
  private slotDur = new Float32Array(SPIKE_POOL_CAPACITY);

  /** Density scale: shrink spikes alongside somas so they stay spark-sized
   *  relative to the graph instead of blobbing the core. */
  setSizeScale(scale: number) {
    this.sizeAttr.array.fill(SPIKE_SIZE * scale);
    this.sizeAttr.needsUpdate = true;
  }

  constructor(color: number) {
    const positions = new Float32Array(SPIKE_POOL_CAPACITY * 3);
    const alphas = new Float32Array(SPIKE_POOL_CAPACITY);
    const sizes = new Float32Array(SPIKE_POOL_CAPACITY).fill(SPIKE_SIZE);
    const geometry = new THREE.BufferGeometry();
    this.posAttr = new THREE.BufferAttribute(positions, 3);
    this.alphaAttr = new THREE.BufferAttribute(alphas, 1);
    this.sizeAttr = new THREE.BufferAttribute(sizes, 1);
    geometry.setAttribute('position', this.posAttr);
    geometry.setAttribute('aAlpha', this.alphaAttr);
    geometry.setAttribute('aSize', this.sizeAttr);
    const material = new THREE.ShaderMaterial({
      vertexShader: SPIKE_VERTEX,
      fragmentShader: SPIKE_FRAGMENT,
      uniforms: { uColor: { value: new THREE.Color(color) } },
      transparent: true,
      depthWrite: false,
      blending: THREE.AdditiveBlending,
    });
    this.points = new THREE.Points(geometry, material);
    this.points.frustumCulled = false;
    for (let i = SPIKE_POOL_CAPACITY - 1; i >= 0; i--) this.free.push(i);
  }

  get active() {
    return this.activeCount;
  }

  /** Start a spike on `edgeIndex` traveling `dir` over `duration` seconds.
   *  Returns false when the pool is saturated (drop, never allocate). */
  claim(edgeIndex: number, dir: 1 | -1, duration: number, origin: THREE.Vector3): boolean {
    const slot = this.free.pop();
    if (slot === undefined) return false;
    this.slotEdge[slot] = edgeIndex;
    this.slotT[slot] = 0;
    this.slotDir[slot] = dir;
    this.slotDur[slot] = Math.max(duration, 0.05);
    this.posAttr.setXYZ(slot, origin.x, origin.y, origin.z);
    this.alphaAttr.setX(slot, 1);
    this.posAttr.needsUpdate = true;
    this.alphaAttr.needsUpdate = true;
    this.activeCount++;
    return true;
  }

  /** Advance every active spike; arrival frees the slot and reports the
   *  destination edge+dir so the caller can flare the target soma.
   *  `curveAt(edgeIndex)` must return the edge's curve (or null). */
  advance(
    dt: number,
    curveAt: (edgeIndex: number) => THREE.Curve<THREE.Vector3> | null,
    onArrive: (edgeIndex: number, dir: 1 | -1) => void,
  ) {
    if (this.activeCount === 0) return;
    const scratch = _scratch;
    for (let slot = 0; slot < SPIKE_POOL_CAPACITY; slot++) {
      if (this.alphaAttr.getX(slot) === 0) continue;
      const curve = curveAt(this.slotEdge[slot]);
      if (!curve) {
        this.release(slot);
        continue;
      }
      const t = (this.slotT[slot] += dt / this.slotDur[slot]);
      if (t >= 1) {
        const edge = this.slotEdge[slot];
        const dir = this.slotDir[slot] as 1 | -1;
        this.release(slot);
        onArrive(edge, dir);
        continue;
      }
      const tt = this.slotDir[slot] === 1 ? t : 1 - t;
      curve.getPointAt(tt, scratch);
      // Fade in over the first 15% of travel, fade out approaching the soma.
      const fade = Math.min(t / 0.15, 1) * Math.min((1 - t) / 0.1, 1);
      this.posAttr.setXYZ(slot, scratch.x, scratch.y, scratch.z);
      this.alphaAttr.setX(slot, Math.max(fade, 0.15));
    }
    this.posAttr.needsUpdate = true;
    this.alphaAttr.needsUpdate = true;
  }

  private release(slot: number) {
    this.alphaAttr.setX(slot, 0);
    this.free.push(slot);
    this.activeCount--;
  }

  /** Drop every in-flight spike — call when edges rebuild so slot→edge
   *  indices can't point at stale lines. */
  clear() {
    for (let i = 0; i < SPIKE_POOL_CAPACITY; i++) this.alphaAttr.setX(i, 0);
    this.alphaAttr.needsUpdate = true;
    this.free.length = 0;
    for (let i = SPIKE_POOL_CAPACITY - 1; i >= 0; i--) this.free.push(i);
    this.activeCount = 0;
  }

  /** Seconds a spike spends on an edge — proportional to curve length. */
  static durationFor(curve: THREE.Curve<THREE.Vector3>): number {
    return curve.getLength() / SPIKE_SPEED;
  }

  dispose() {
    this.points.geometry.dispose();
    (this.points.material as THREE.Material).dispose();
  }
}

const _scratch = new THREE.Vector3();
