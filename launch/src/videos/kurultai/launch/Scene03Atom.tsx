import React from 'react';
import {Easing, interpolate, useCurrentFrame} from 'remotion';
import {KineticType, MonoLine} from '../../../core/KineticType';
import {color, font, radius} from '../../../brands/kurultai/tokens';

/**
 * Scene 03 — Atom lifecycle (7–12s).
 * A raw markdown note slides in → SPLITS into an indexed atom card →
 * the quarantine tag FLIPS to trusted. Verbatim strings from the live demo.
 */
export const Scene03Atom: React.FC = () => {
  const frame = useCurrentFrame();

  const noteIn = interpolate(frame, [6, 26], [1, 0], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: Easing.bezier(0.2, 0, 0, 1),
  });
  const noteOut = interpolate(frame, [80, 96], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: Easing.bezier(0.4, 0, 0.6, 1),
  });
  const atomIn = interpolate(frame, [88, 110], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: Easing.bezier(0.2, 0, 0, 1),
  });
  const flip = interpolate(frame, [170, 196], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: Easing.bezier(0.4, 0, 0.2, 1),
  });
  const trusted = flip > 0.5;

  return (
    <div style={{position: 'absolute', inset: 0, background: color.canvas}}>
      <div style={{position: 'absolute', top: 64, left: 0, right: 0}}>
        <KineticType text="Notes become atoms." delay={2} size={72} />
      </div>

      {/* raw note → exits left */}
      <div
        style={{
          position: 'absolute',
          left: 240 + (1 - noteIn) * -260 - noteOut * 320,
          top: 320,
          width: 560,
          padding: 28,
          borderRadius: radius.lg,
          background: color.raised,
          border: `1.5px solid ${color.hairline}`,
          opacity: noteIn * (1 - noteOut),
        }}
      >
        <MonoLine text="12-meridian-adr-tick-batching.md" size={22} ink={color.ink2} delay={14} />
        <div style={{fontFamily: font.grotesk, fontSize: 26, color: color.ink, marginTop: 14, lineHeight: 1.35}}>
          # ADR-004 — tick batching{'\n'}
          Batch ingest ticks at 250ms…
        </div>
      </div>

      {/* atom card → deals in center-right */}
      <div
        style={{
          position: 'absolute',
          left: 960,
          top: 290,
          width: 640,
          padding: 30,
          borderRadius: radius.lg,
          background: color.surface,
          border: `1.5px solid ${color.accent}44`,
          opacity: atomIn,
          transform: `scale(${0.7 + atomIn * 0.3})`,
        }}
      >
        <div style={{display: 'flex', justifyContent: 'space-between', alignItems: 'center'}}>
          <span style={{fontFamily: font.mono, fontSize: 20, color: color.ink3}}>atom</span>
          {/* status pill — quarantine → trusted */}
          <span
            style={{
              fontFamily: font.mono,
              fontSize: 18,
              padding: '6px 16px',
              borderRadius: 999,
              color: trusted ? color.passed : color.caution,
              background: trusted ? color.passedTint : color.cautionTint,
              border: `1.5px solid ${trusted ? color.passed : color.caution}`,
              transform: `rotateX(${(1 - flip) * 0}deg)`,
            }}
          >
            {trusted ? 'trusted' : 'quarantine'}
          </span>
        </div>
        <div style={{fontFamily: font.grotesk, fontWeight: 700, fontSize: 34, color: color.ink, marginTop: 16}}>
          12 meridian adr tick batching
        </div>
        <MonoLine text="source: demo_corpus · f54ee4bb…7185" size={18} ink={color.ink3} delay={120} />
        <div style={{display: 'flex', gap: 10, marginTop: 18, opacity: atomIn}}>
          {['#meridian', '#adr', '#decision'].map((t) => (
            <span
              key={t}
              style={{
                fontFamily: font.mono,
                fontSize: 17,
                padding: '5px 14px',
                borderRadius: 999,
                color: color.accentSoft,
                border: `1px solid ${color.accentDeep}`,
              }}
            >
              {t}
            </span>
          ))}
        </div>
      </div>

      <div style={{position: 'absolute', bottom: 120, left: 0, right: 0, textAlign: 'center', opacity: Math.max(0, flip - 0.3) * 1.4}}>
        <MonoLine text="235 atoms · 220 trusted · 15 quarantine" size={26} ink={color.ink2} delay={200} />
      </div>
    </div>
  );
};
