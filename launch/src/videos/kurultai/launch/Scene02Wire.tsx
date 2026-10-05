import React from 'react';
import {Easing, interpolate, useCurrentFrame} from 'remotion';
import {KineticType, MonoLine} from '../../../core/KineticType';
import {color, font, radius} from '../../../brands/kurultai/tokens';
import {SynapseMark} from '../../../brands/kurultai/BrainMark';

const AGENTS = ['cursor', 'claude', 'codex', 'hermes', 'devin'];

/**
 * Scene 02 v2 — the wiring.
 * `kurultai mcp` types into a config line; the five silo bubbles
 * collapse and every agent fires a beam into ONE shared core.
 * Beat: "One daemon. Every agent."
 */
export const Scene02Wire: React.FC = () => {
  const frame = useCurrentFrame();
  const CX = 960;
  const CY = 640;

  // config line types itself, then the collapse begins
  const typed = interpolate(frame, [10, 60], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  const collapse = interpolate(frame, [80, 130], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: Easing.bezier(0.4, 0, 0.2, 1),
  });
  const coreIn = interpolate(frame, [110, 145], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: Easing.bezier(0.2, 0, 0, 1),
  });
  const beamT = interpolate(frame, [150, 195], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: Easing.bezier(0.2, 0, 0, 1),
  });

  return (
    <div style={{position: 'absolute', inset: 0, background: color.canvas}}>
      <div style={{position: 'absolute', top: 72, left: 0, right: 0}}>
        <KineticType text="One daemon. Every agent." delay={8} size={64} />
      </div>

      {/* config line */}
      <div
        style={{
          position: 'absolute',
          left: 560,
          top: 210,
          width: 800,
          padding: '18px 28px',
          borderRadius: radius.lg,
          background: color.surface,
          border: `1.5px solid ${color.hairline}`,
          opacity: typed > 0 ? 1 : 0,
        }}
      >
        <MonoLine text='"command": "kurultai mcp"' size={28} ink={color.ink} delay={14} />
      </div>

      {/* shared core */}
      <div
        style={{
          position: 'absolute',
          left: CX - 90,
          top: CY - 90,
          opacity: coreIn,
          transform: `scale(${0.3 + coreIn * 0.7})`,
        }}
      >
        <SynapseMark size={180} delay={0} tint={color.accent} />
      </div>

      {/* five agents — bubbles collapse, beams fire to core */}
      {AGENTS.map((a, i) => {
        const angle = -Math.PI / 2 + (i - 2) * 0.62;
        const R = 300;
        const x = CX + Math.cos(angle) * R;
        const y = CY + Math.sin(angle) * R;
        const beam = Math.max(0, Math.min(1, beamT * 1.4 - i * 0.1));
        const ex = x + (CX - x) * 0.3;
        const ey = y + (CY - y) * 0.3;
        // chips fly from their Scene-01 silo slots to ring positions
        const siloLeft = 384 * i + 192 - 90;
        const siloTop = 380;
        const left = siloLeft + (x - 90 - siloLeft) * collapse;
        const top = siloTop + (y - 24 - siloTop) * collapse;
        return (
          <React.Fragment key={a}>
            <svg
              style={{position: 'absolute', inset: 0, overflow: 'visible'}}
              width="1920"
              height="1080"
            >
              <line
                x1={ex}
                y1={ey}
                x2={ex + (CX - ex) * beam}
                y2={ey + (CY - ey) * beam}
                stroke={color.accent}
                strokeWidth={2.5}
                opacity={beam > 0 ? 0.9 : 0}
                strokeLinecap="round"
              />
            </svg>
            <div
              style={{
                position: 'absolute',
                left,
                top,
                width: 180,
                padding: '16px 0',
                borderRadius: radius.lg,
                background: color.raised,
                border: `1.5px solid ${color.accent}55`,
                textAlign: 'center',
                fontFamily: font.mono,
                fontSize: 24,
                color: color.ink,
                opacity: 1,
                transform: `scale(${0.9 + collapse * 0.1})`,
              }}
            >
              {a}
            </div>
          </React.Fragment>
        );
      })}

      <div style={{position: 'absolute', bottom: 120, left: 0, right: 0, textAlign: 'center'}}>
        <MonoLine
          text="memory · citations · coordination — over MCP"
          size={26}
          ink={color.ink2}
          delay={205}
        />
      </div>
    </div>
  );
};
