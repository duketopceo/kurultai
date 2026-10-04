import React from 'react';
import {Easing, interpolate, useCurrentFrame} from 'remotion';
import {KineticType, MonoLine} from '../../../core/KineticType';
import {color, font, radius} from '../../../brands/kurultai/tokens';

/**
 * Scene 06 — Ontology proposal (22–27s).
 * wright's promote_atom proposal card lands with pending status —
 * the agents-propose / humans-decide gate, verbatim payload from the demo.
 */
export const Scene06Ontology: React.FC = () => {
  const frame = useCurrentFrame();

  const cardIn = interpolate(frame, [14, 40], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: Easing.bezier(0.2, 0, 0, 1),
  });
  const pendingPulse = 0.72 + 0.28 * Math.sin(frame / 9);
  const stampIn = interpolate(frame, [150, 168], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: Easing.bezier(0.1, 0, 0.2, 1),
  });

  return (
    <div style={{position: 'absolute', inset: 0, background: color.canvas}}>
      <div style={{position: 'absolute', top: 64, left: 0, right: 0}}>
        <KineticType text="Agents propose. Humans decide." delay={2} size={72} />
      </div>

      <div
        style={{
          position: 'absolute',
          left: 560,
          top: 260,
          width: 800,
          padding: 34,
          borderRadius: radius.lg,
          background: color.surface,
          border: `1.5px solid ${color.accent}55`,
          opacity: cardIn,
          transform: `scale(${0.75 + cardIn * 0.25})`,
        }}
      >
        <div style={{display: 'flex', justifyContent: 'space-between', alignItems: 'center'}}>
          <span style={{fontFamily: font.mono, fontSize: 20, color: color.ink3}}>
            prop:9ca0a523-9352-…
          </span>
          <span
            style={{
              fontFamily: font.mono,
              fontSize: 19,
              padding: '6px 18px',
              borderRadius: 999,
              color: color.accentSoft,
              border: `1.5px solid ${color.accentSoft}`,
              background: color.accentTint,
              opacity: pendingPulse,
            }}
          >
            pending
          </span>
        </div>

        <div style={{fontFamily: font.grotesk, fontWeight: 700, fontSize: 36, color: color.ink, marginTop: 18}}>
          promote_atom → class:decision
        </div>
        <MonoLine
          text='reason: "ADR-004 is a load-bearing architecture decision"'
          size={21}
          ink={color.ink2}
          delay={50}
        />
        <div style={{fontFamily: font.mono, fontSize: 20, color: color.ink3, marginTop: 20}}>
          proposed_by: <span style={{color: color.accentSoft}}>wright</span>
        </div>
      </div>

      {/* decision gate stamp */}
      <div
        style={{
          position: 'absolute',
          left: 760,
          top: 640,
          padding: '16px 36px',
          borderRadius: radius.lg,
          border: `2px solid ${color.ink}`,
          fontFamily: font.grotesk,
          fontWeight: 700,
          fontSize: 30,
          color: color.ink,
          letterSpacing: '0.08em',
          opacity: stampIn,
          transform: `scale(${1.6 - stampIn * 0.6})`,
        }}
      >
        AWAITING HUMAN
      </div>
    </div>
  );
};
