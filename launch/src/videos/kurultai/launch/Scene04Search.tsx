import React from 'react';
import {Easing, interpolate, useCurrentFrame} from 'remotion';
import {KineticType, MonoLine} from '../../../core/KineticType';
import {color, font, radius} from '../../../brands/kurultai/tokens';

const HITS = [
  {id: '02-retrieval-tiers.md', title: 'Retrieval tiers', tier: 'hot'},
  {id: '05-brain-ui.md', title: 'Brain UI', tier: 'hot'},
  {id: '11-meridian-architecture.md', title: 'Meridian architecture', tier: 'hot'},
];

/**
 * Scene 04 — Search (12–17s).
 * Query types itself → cited hits DEAL in with source pills.
 */
export const Scene04Search: React.FC = () => {
  const frame = useCurrentFrame();
  return (
    <div style={{position: 'absolute', inset: 0, background: color.canvas}}>
      <div style={{position: 'absolute', top: 64, left: 0, right: 0}}>
        <KineticType text="Ask where a claim came from." delay={2} size={64} />
      </div>

      {/* query bar */}
      <div
        style={{
          position: 'absolute',
          left: 460,
          top: 260,
          width: 1000,
          padding: '22px 30px',
          borderRadius: radius.lg,
          background: color.surface,
          border: `1.5px solid ${color.accent}55`,
        }}
      >
        <MonoLine text="kurultai search 'retrieval tiers'" size={28} ink={color.ink} delay={16} />
      </div>

      {/* cited hits */}
      {HITS.map((h, i) => {
        const t = interpolate(frame, [90 + i * 16, 114 + i * 16], [0, 1], {
          extrapolateLeft: 'clamp',
          extrapolateRight: 'clamp',
          easing: Easing.bezier(0.2, 0, 0, 1),
        });
        return (
          <div
            key={h.id}
            style={{
              position: 'absolute',
              left: 460,
              top: 380 + i * 150,
              width: 1000,
              padding: '22px 30px',
              borderRadius: radius.lg,
              background: color.raised,
              border: `1.5px solid ${color.hairline}`,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              opacity: t,
              transform: `translateY(${(1 - t) * 46}px)`,
            }}
          >
            <div>
              <div style={{fontFamily: font.grotesk, fontWeight: 700, fontSize: 30, color: color.ink}}>
                {h.title}
              </div>
              <div style={{fontFamily: font.mono, fontSize: 19, color: color.ink3, marginTop: 6}}>
                demo_corpus/{h.id}
              </div>
            </div>
            <div style={{display: 'flex', alignItems: 'center', gap: 16}}>
              <span style={{fontFamily: font.mono, fontSize: 18, color: color.accentSoft}}>
                tier:{h.tier}
              </span>
              <span
                style={{
                  fontFamily: font.mono,
                  fontSize: 17,
                  padding: '5px 14px',
                  borderRadius: 999,
                  color: color.passed,
                  border: `1px solid ${color.passed}66`,
                }}
              >
                cited
              </span>
            </div>
          </div>
        );
      })}
    </div>
  );
};
