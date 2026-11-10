/* Background field for Brain v2: a Kurultai variant of the "Ribbon Field" background from
   ThreeUI Community (https://github.com/MengTo/threeui, src/shaders/ribbon-field/, MIT,
   Copyright (c) 2026 Meng To; notice in website/public/credits.txt).
   Changes from upstream: monochrome (ink on paper, no hue, no bloom); fewer, thinner, slower
   strands with a fine fibre ripple; coverage drives halftone dot SIZE instead of alpha; masked
   to near-zero around the brain's projected ellipse and to zero under the right panel and top bar.
   The ribbon-with-gaussian-falloff and halftone-grid structure is upstream's. */

export const BG_VERT = 'varying vec2 vUv; void main() { vUv = uv; gl_Position = vec4(position.xy, 0.9999, 1.0); }';

/** Top-bar height (CSS px) that the field stays clear of. */
export const TOP_BAR_PX = 56;
/** Peak ink alpha of the field far from the brain (the brain keeps the focal contrast). */
export const FIELD_PEAK = { light: 0.5, dark: 0.42 } as const;
/** Field time offset so a frozen field (reduced motion / reduce effects) is a composed frame, not t=0. */
export const STILL_PHASE = 1.7;

export const BG_FRAG = /* glsl */ `
uniform vec3 uBg;
uniform vec3 uInkC;
uniform float uTime;
uniform float uPeak;
uniform vec2 uCss;     // canvas size in CSS px
uniform vec4 uBrain;   // ellipse centre x,y (CSS px, y up) and radii rx,ry
uniform vec2 uInsets;  // right panel width, top bar height (CSS px)
uniform float uCell;   // halftone cell, CSS px
varying vec2 vUv;
float h(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
// a thin strand: gaussian across, with a fine fibre ripple along it
float strand(vec2 uv, float y0, float amp, float freq, float width, float phase, float rip) {
  float y = y0 + amp * sin(uv.x * freq + phase) + 0.03 * sin(uv.x * freq * 2.7 - phase * 0.8) + rip * sin(uv.x * 23.0 + phase * 1.9);
  float d = uv.y - y;
  return exp(-(d * d) / width);
}
void main() {
  vec2 px = vUv * uCss;
  vec2 uv = vec2(vUv.x * uCss.x / uCss.y, vUv.y);
  float t = uTime * 0.18 + ${STILL_PHASE.toFixed(1)};
  float cov = 0.0;
  cov += strand(uv, 0.30, 0.10, 1.9, 0.00008, t + 0.9, 0.004);
  cov += strand(uv, 0.46, 0.13, 1.4, 0.00006, t + 3.2, 0.003) * 0.9;
  cov += strand(uv, 0.62, 0.11, 2.3, 0.00007, t + 1.8, 0.004);
  cov += strand(uv, 0.78, 0.09, 1.7, 0.00006, t + 4.6, 0.003) * 0.8;
  cov += strand(uv, 0.54, 0.22, 1.1, 0.00010, t + 5.4, 0.006) * 0.55;
  cov = clamp(cov, 0.0, 1.0);

  // mask: quiet inside / near the brain's ellipse, zero under the panel and the top bar
  float d = length((px - uBrain.xy) / uBrain.zw);
  float m = smoothstep(0.8, 2.0, d);
  m *= m * sqrt(m);                                   // pow 2.5: ~2% at the ellipse edge
  float panel = 1.0 - smoothstep(uCss.x - uInsets.x - 48.0, uCss.x - uInsets.x, px.x);
  m *= mix(1.0, panel, step(0.5, uInsets.x));
  m *= 1.0 - smoothstep(uCss.y - uInsets.y - 28.0, uCss.y - uInsets.y, px.y);

  // halftone: coverage sets dot radius
  vec2 g = px / uCell;
  float r = 0.5 * sqrt(cov * m) * (0.75 + 0.25 * h(floor(g)));
  float dotShape = 1.0 - smoothstep(r - 0.16, r, length(fract(g) - 0.5));
  float a = dotShape * uPeak * step(0.02, cov * m);
  gl_FragColor = vec4(mix(uBg, uInkC, a), 1.0);
}`;
