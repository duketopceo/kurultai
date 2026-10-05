import React from 'react';
import {Easing, interpolate, useCurrentFrame} from 'remotion';
import {UIZoom} from '../../../core/UIZoom';
import {KineticType} from '../../../core/KineticType';
import {color, font, radius} from '../../../brands/kurultai/tokens';
import {SynapseMark} from '../../../brands/kurultai/BrainMark';

const AGENTS = ['cursor', 'claude', 'codex', 'antigravity', 'hermes', 'devin'];
const CX = 960;
const CY = 540;
const R = 280;

/**
 * Scene 02 — Fleet (2.5–7s).
 * Six agent chips DEAL onto a ring around the core neuron; each fires a
 * beam into the brain. Camera punches from wide to the core.
 */
export const Scene02Fleet: React.FC = () => {
  const frame = useCurrentFrame();
  return (
    <div style={{position: 'absolute', inset: 0, background: color.canvas}}>
      <UIZoom
        width={1920}
        height={1080}
        keys={[
          {at: 0, x: CX, y: CY, scale: 1},
          {at: 40, x: CX, y: CY, scale: 1},
          {at: 90, x: CX, y: CY, scale: 1.45},
          {at: 150, x: CX, y: CY, scale: 1.45},
        ]}
      >
        {/* beams */}
        <svg width={1920} height={1080} style={{position: 'absolute', inset: 0}}>
          {AGENTS.map((_, i) => {
            const a = (i / AGENTS.length) * Math.PI * 2 - Math.PI / 2 + Math.PI / 6;
            const x = CX + Math.cos(a) * R;
            const y = CY + Math.sin(a) * R;
            const start = 60 + i * 10;
            const t = interpolate(frame, [start, start + 12], [0, 1], {
              extrapolateLeft: 'clamp',
              extrapolateRight: 'clamp',
            });
            const flicker = t === 1 ? 0.55 + 0.35 * Math.sin(frame / 3 + i) : 0;
            // beams start at the chip edge (84px toward the core), not centers
            const ex = x + (CX - x) * 0.26;
            const ey = y + (CY - y) * 0.26;
            return (
              <line
                key={i}
                x1={ex}
                y1={ey}
                x2={CX + (x - CX) * (1 - t)}
                y2={CY + (y - CY) * (1 - t)}
                stroke={color.accent}
                strokeWidth={2.5}
                opacity={Math.max(t, flicker)}
              />
            );
          })}
        </svg>
        {/* core neuron */}
        <div style={{position: 'absolute', left: CX - 95, top: CY - 95}}>
          <SynapseMark size={190} delay={10} tint={color.accent} />
        </div>
        {/* agent chips */}
        {AGENTS.map((name, i) => {
          const a = (i / AGENTS.length) * Math.PI * 2 - Math.PI / 2 + Math.PI / 6;
          const x = CX + Math.cos(a) * R;
          const y = CY + Math.sin(a) * R;
          const deal = interpolate(frame, [12 + i * 7, 34 + i * 7], [0, 1], {
            extrapolateLeft: 'clamp',
            extrapolateRight: 'clamp',
            easing: Easing.bezier(0.2, 0, 0, 1),
          });
          return (
            <div
              key={name}
              style={{
                position: 'absolute',
                left: x - 74,
                top: y - 22,
                padding: '10px 22px',
                borderRadius: radius.lg,
                background: color.raised,
                border: `1.5px solid ${color.hairline}`,
                fontFamily: font.mono,
                fontSize: 26,
                color: color.ink,
                opacity: deal,
                transform: `scale(${0.6 + deal * 0.4})`,
              }}
            >
              {name}
            </div>
          );
        })}
      </UIZoom>
      <div style={{position: 'absolute', top: 64, left: 0, right: 0}}>
        <KineticType text="Every agent, one memory." delay={4} size={64} />
      </div>
    </div>
  );
};
