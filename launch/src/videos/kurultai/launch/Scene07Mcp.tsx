import React from 'react';
import {Easing, interpolate, useCurrentFrame} from 'remotion';
import {KineticType, MonoLine} from '../../../core/KineticType';
import {color, font, radius} from '../../../brands/kurultai/tokens';

const TOOLS = [
  'search', 'cite', 'remember', 'ask',
  'who_knows', 'promote', 'ontology_get', 'ontology_promote',
];

/**
 * Scene 07 — MCP surface (27–32s).
 * The eight real MCP tools lock into a grid, two columns of four.
 */
export const Scene07Mcp: React.FC = () => {
  const frame = useCurrentFrame();
  return (
    <div style={{position: 'absolute', inset: 0, background: color.canvas}}>
      <div style={{position: 'absolute', top: 64, left: 0, right: 0}}>
        <KineticType text="Eight tools. Zero plumbing." delay={2} size={72} />
      </div>

      <div
        style={{
          position: 'absolute',
          left: 460,
          top: 240,
          width: 1000,
          display: 'grid',
          gridTemplateColumns: 'repeat(2, minmax(0, 1fr))',
          gap: 18,
        }}
      >
        {TOOLS.map((t, i) => {
          const s = interpolate(frame, [20 + i * 8, 38 + i * 8], [0, 1], {
            extrapolateLeft: 'clamp',
            extrapolateRight: 'clamp',
            easing: Easing.bezier(0.2, 0, 0, 1),
          });
          return (
            <div
              key={t}
              style={{
                padding: '20px 28px',
                borderRadius: radius.lg,
                background: color.raised,
                border: `1.5px solid ${color.hairline}`,
                fontFamily: font.mono,
                fontSize: 28,
                color: color.ink,
                opacity: s,
                transform: `translateY(${(1 - s) * 34}px)`,
                display: 'flex',
                alignItems: 'center',
                gap: 16,
              }}
            >
              <span style={{color: color.accent}}>◈</span>
              {t}
            </div>
          );
        })}
      </div>

      <div style={{position: 'absolute', bottom: 130, left: 0, right: 0, textAlign: 'center'}}>
        <MonoLine text="kurultai mcp — one stdio server, every agent" size={24} ink={color.ink3} delay={110} />
      </div>
    </div>
  );
};
