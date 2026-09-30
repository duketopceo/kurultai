import * as THREE from 'three';

/*
 * Neuron-motif node dressing (plan 2026-09-12-001, U1/KTD1/KTD5).
 * A neuron reads as neural from three elements: soma, a short dendritic
 * corona, and an axon that conducts spikes. This module owns the first two:
 * `somaGeometry` gives the faceted cell body and `coronaTexture` paints the
 * radial starburst that replaces the old round halo. The texture is drawn
 * procedurally at boot — no binary assets, constants stay tweakable.
 */

/** Corona silhouette for a node: filament count grows with connectivity. */
export interface CoronaParams {
  filaments: number;
  /** Radians of angular jitter applied per filament (organic irregularity). */
  jitter: number;
  /** Sprite scale multiplier vs the plain halo (filaments extend past it). */
  scale: number;
}

const CORONA_MIN_FILAMENTS = 5;
const CORONA_MAX_FILAMENTS = 9;
const CORONA_DEGREE_SATURATION = 20;

/** Pure degree → corona shape mapping. Higher-degree neurons sprout more
 *  filaments; saturated at CORONA_DEGREE_SATURATION so hubs don't turn into
 *  sunbursts. */
export function coronaParams(degree: number): CoronaParams {
  const t = Math.min(Math.max(degree, 0), CORONA_DEGREE_SATURATION) / CORONA_DEGREE_SATURATION;
  return {
    filaments: Math.round(CORONA_MIN_FILAMENTS + t * (CORONA_MAX_FILAMENTS - CORONA_MIN_FILAMENTS)),
    jitter: 0.16 + t * 0.1,
    scale: 2.4 + t * 0.6,
  };
}

/** Faceted soma — an icosahedron reads as a cell body under flat shading,
 *  where a perfect sphere reads as a generic orb. */
export function somaGeometry(): THREE.BufferGeometry {
  return new THREE.IcosahedronGeometry(1, 1);
}

/** Draw the corona sprite: thin tapered filaments radiating from a soft core.
 *  Returns null without a DOM (tests, SSR) — callers keep the halo fallback. */
export function coronaTexture(params: CoronaParams, size = 128): THREE.CanvasTexture | null {
  if (typeof document === 'undefined') return null;
  const canvas = document.createElement('canvas');
  canvas.width = canvas.height = size;
  const ctx = canvas.getContext('2d');
  if (!ctx) return null;
  const c = size / 2;
  const inner = size * 0.08;
  const outer = size * 0.48;

  // Soft soma core — keeps the node center bright inside the filaments.
  const core = ctx.createRadialGradient(c, c, 0, c, c, inner * 2.2);
  core.addColorStop(0, 'rgba(255,255,255,0.95)');
  core.addColorStop(0.5, 'rgba(255,255,255,0.28)');
  core.addColorStop(1, 'rgba(255,255,255,0)');
  ctx.fillStyle = core;
  ctx.fillRect(0, 0, size, size);

  // Filaments: tapered quads from inner radius to outer, slight angular
  // jitter + length variation so the corona reads grown, not stamped.
  ctx.globalCompositeOperation = 'lighter';
  for (let i = 0; i < params.filaments; i++) {
    const baseAngle = (i / params.filaments) * Math.PI * 2;
    const angle = baseAngle + (Math.sin(i * 12.9898) * 0.5 + 0.5 - 0.5) * params.jitter * 2;
    const len = outer * (0.72 + 0.28 * ((Math.sin(i * 78.233) * 0.5 + 0.5)));
    const wInner = size * 0.014;
    const wOuter = size * 0.002;
    const cos = Math.cos(angle);
    const sin = Math.sin(angle);
    const px = -sin;
    const py = cos;

    const g = ctx.createLinearGradient(
      c + cos * inner,
      c + sin * inner,
      c + cos * len,
      c + sin * len,
    );
    g.addColorStop(0, 'rgba(255,255,255,0.55)');
    g.addColorStop(0.6, 'rgba(255,255,255,0.18)');
    g.addColorStop(1, 'rgba(255,255,255,0)');
    ctx.fillStyle = g;
    ctx.beginPath();
    ctx.moveTo(c + cos * inner + px * wInner, c + sin * inner + py * wInner);
    ctx.lineTo(c + cos * inner - px * wInner, c + sin * inner - py * wInner);
    ctx.lineTo(c + cos * len - px * wOuter, c + sin * len - py * wOuter);
    ctx.lineTo(c + cos * len + px * wOuter, c + sin * len + py * wOuter);
    ctx.closePath();
    ctx.fill();
  }
  ctx.globalCompositeOperation = 'source-over';
  return new THREE.CanvasTexture(canvas);
}
