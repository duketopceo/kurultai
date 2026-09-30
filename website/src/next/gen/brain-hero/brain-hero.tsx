// brain-hero.tsx
import { forwardRef, memo } from 'react';
import './brain-hero.css';

export type BrainTier = 'min' | 'low' | 'mid' | 'high' | 'max' | (string & {});
export type BrainLayout = 'force' | 'ontology' | (string & {});

export interface BrainHeroProps {
  /** React subtree rendered inside the cortex mount (the BrainStage canvas). */
  children?: import('react').ReactNode;
  /** Atoms currently rendered as neurons. */
  atomCount: number;
  /** Atoms in the store. May exceed atomCount when the tier caps rendering. */
  atomTotal: number;
  /** Render tier label, e.g. 'max'. */
  tier: BrainTier;
  /** Active layout. 'ontology' shows the ONTOLOGY badge. */
  layout: BrainLayout;
  /** Measured frames per second. null while not yet measured. */
  fps: number | null;
  /** Synapse (edge) count. null while unknown. */
  synapses: number | null;
  /** True when the store has no memories. */
  empty: boolean;
}

const DASH = '—';
const LOW_FPS = 24;
const intFmt = new Intl.NumberFormat('en-US');

function fmtInt(n: number | null | undefined): string {
  if (n == null || !Number.isFinite(n)) return DASH;
  return intFmt.format(Math.max(0, Math.round(n)));
}

function fmtCompact(n: number | null | undefined): string {
  if (n == null || !Number.isFinite(n)) return DASH;
  const v = Math.max(0, Math.round(n));
  if (v < 1_000) return String(v);
  const [div, unit] = v < 1_000_000 ? [1_000, 'k'] : [1_000_000, 'M'];
  const scaled = v / div;
  const text = scaled < 10 ? scaled.toFixed(1) : Math.round(scaled).toString();
  return text.replace(/\.0$/, '') + unit;
}

const Sep = () => (
  <span className="brain-hero__sep" aria-hidden="true">
    ·
  </span>
);

/**
 * Chrome around the cortex canvas. The forwarded ref is the 3D mount point.
 * In the empty state no mount is rendered, so ref.current stays null and
 * the renderer must not initialize.
 */
export const BrainHero = memo(
  forwardRef<HTMLDivElement, BrainHeroProps>(function BrainHero(
    { children, atomCount, atomTotal, tier, layout, fps, synapses, empty },
    mountRef,
  ) {
    const isEmpty = empty || !(atomTotal > 0);
    const isOntology = layout === 'ontology';
    const capped =
      Number.isFinite(atomCount) &&
      Number.isFinite(atomTotal) &&
      atomCount < atomTotal;
    const fpsKnown = fps != null && Number.isFinite(fps);
    const fpsState = !fpsKnown ? 'unknown' : fps! < LOW_FPS ? 'low' : 'ok';
    const tierLabel = String(tier ?? '').trim().toLowerCase();

    const neuronLabel = capped
      ? `${fmtInt(atomCount)} of ${fmtInt(atomTotal)} neurons`
      : `${fmtInt(atomCount)} neurons`;

    return (
      <section
        className="brain-hero"
        data-empty={isEmpty || undefined}
        data-layout={layout}
        aria-label="Cortex"
      >
        {isEmpty ? (
          <div className="brain-hero__empty" role="status">
            <span className="brain-hero__empty-dot" aria-hidden="true" />
            <p className="brain-hero__empty-text">
              <span>no memories yet</span>
              <span className="brain-hero__empty-dash" aria-hidden="true">
                {' — '}
              </span>
              <code className="brain-hero__empty-cmd">kurultai init --docs</code>
              <span className="brain-hero__caret" aria-hidden="true" />
            </p>
          </div>
        ) : (
          <>
            <div
              ref={mountRef}
              className="brain-hero__mount"
              role="img"
              aria-label={`Cortex graph, ${neuronLabel}`}
            >
              {children}
            </div>
            <div className="brain-hero__vignette" aria-hidden="true" />

            <div className="brain-hero__overlay">
              <div
                className="brain-hero__chip"
                data-fps={fpsState}
                aria-label={`fps ${fpsKnown ? Math.round(fps!) : 'not measured'}, nodes ${fmtInt(atomCount)}, synapses ${synapses == null ? 'unknown' : fmtInt(synapses)}`}
              >
                <span className="brain-hero__stat" aria-hidden="true">
                  <span className="brain-hero__k">fps</span>
                  <span className="brain-hero__v brain-hero__v--fps">
                    {fpsKnown ? Math.round(fps!) : DASH}
                  </span>
                </span>
                <Sep />
                <span className="brain-hero__stat" aria-hidden="true">
                  <span className="brain-hero__k">nodes</span>
                  <span className="brain-hero__v">{fmtInt(atomCount)}</span>
                </span>
                <Sep />
                <span
                  className="brain-hero__stat"
                  aria-hidden="true"
                  title={synapses == null ? undefined : `${fmtInt(synapses)} synapses`}
                >
                  <span className="brain-hero__k">syn</span>
                  <span className="brain-hero__v">{fmtCompact(synapses)}</span>
                </span>
              </div>

              <div className="brain-hero__caption-row">
                {isOntology && (
                  <span className="brain-hero__badge">ONTOLOGY</span>
                )}
                <p className="brain-hero__caption">
                  <span className="brain-hero__caption-count">{neuronLabel}</span>
                  {tierLabel && (
                    <>
                      <Sep />
                      <span>{tierLabel}</span>
                    </>
                  )}
                  <Sep />
                  <span className="brain-hero__hint brain-hero__hint--hover">
                    hover to trace connections
                  </span>
                  <span className="brain-hero__hint brain-hero__hint--touch">
                    tap to trace connections
                  </span>
                </p>
              </div>
            </div>
          </>
        )}
      </section>
    );
  }),
);

export default BrainHero;
