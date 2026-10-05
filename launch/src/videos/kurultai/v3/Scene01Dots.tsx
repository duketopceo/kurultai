import React from 'react';
import {Easing, interpolate, useCurrentFrame} from 'remotion';
import {KineticType} from '../../../core/KineticType';
import {color, font} from '../../../brands/kurultai/tokens';

/**
 * v3 Scene 01 — SiloDots (0–3s).
 * Five glowing dots drift apart; each keeps a private memory orbit.
 * No names — "your tools." Beat: "five agents. five memories."
 */
export const Scene01Dots: React.FC = () => {
  const frame = useCurrentFrame();
  const centers = [340, 650, 960, 1270, 1580];

  return (
    <div style={{position: 'absolute', inset: 0, background: color.canvas}}>
      {centers.map((cx, i) => {
        const inT = interpolate(frame, [i * 6, 18 + i * 6], [0, 1], {
          extrapolateLeft: 'clamp',
          extrapolateRight: 'clamp',
          easing: Easing.bezier(0.2, 0, 0, 1),
        });
        const drift = frame * (0.4 + i * 0.12);
        const cy = 500 + Math.sin(frame / 24 + i) * 24;
        // private memory orbits — one small moon per dot, never shared
        const ma = frame / 26 + i * 1.3;
        return (
          <React.Fragment key={i}>
            <div
              style={{
                position: 'absolute',
                left: cx - 13 + Math.cos(drift / 60) * 10,
                top: cy - 13,
                width: 26,
                height: 26,
                borderRadius: '50%',
                background: color.accent,
                boxShadow: `0 0 26px ${color.accent}`,
                opacity: inT,
              }}
            />
            <div
              style={{
                position: 'absolute',
                left: cx - 7 + Math.cos(ma) * 62 + Math.cos(drift / 60) * 10,
                top: cy - 7 + Math.sin(ma) * 40,
                width: 14,
                height: 14,
                borderRadius: '50%',
                background: color.surface,
                border: `1.5px dashed ${color.ink3}`,
                opacity: inT * 0.9,
              }}
            />
          </React.Fragment>
        );
      })}

      <div style={{position: 'absolute', bottom: 150, left: 0, right: 0}}>
        <KineticType text="five agents. five memories." delay={30} size={58} />
      </div>
    </div>
  );
};
