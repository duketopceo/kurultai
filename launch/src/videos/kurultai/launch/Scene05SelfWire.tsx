import React from 'react';
import {Easing, interpolate, useCurrentFrame} from 'remotion';
import {KineticType, MonoLine} from '../../../core/KineticType';
import {color, font, radius} from '../../../brands/kurultai/tokens';

const NODES = [
  {id: 'adr-004', x: 480, y: 380, superseded: true},
  {id: 'adr-005', x: 760, y: 300},
  {id: 'meridian', x: 1060, y: 420},
  {id: 'runbook-14', x: 780, y: 620},
  {id: 'exp-e13', x: 1240, y: 600},
];

const EDGES: Array<[number, number, string]> = [
  [0, 1, 'supersedes'],
  [1, 2, 'associates_with'],
  [2, 3, 'associates_with'],
  [2, 4, 'triggered_by'],
];

/**
 * Scene 05B — the brain wires itself.
 * Edges extract at index time (zero-LLM); a superseded atom fades as its
 * replacement takes over; an --as-of scrubber rewinds the graph.
 * Beat: "It organizes itself. And remembers what it knew."
 */
export const Scene05SelfWire: React.FC = () => {
  const frame = useCurrentFrame();

  const edgeIn = (i: number) =>
    interpolate(frame, [30 + i * 14, 55 + i * 14], [0, 1], {
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
      easing: Easing.bezier(0.2, 0, 0, 1),
    });
  // supersede: old node fades, new node brightens
  const sup = interpolate(frame, [110, 145], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: Easing.bezier(0.2, 0, 0, 1),
  });
  // as-of scrubber sweeps a timeline
  const scrub = interpolate(frame, [170, 230], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: Easing.bezier(0.3, 0, 0.3, 1),
  });

  return (
    <div style={{position: 'absolute', inset: 0, background: color.canvas}}>
      <div style={{position: 'absolute', top: 72, left: 0, right: 0}}>
        <KineticType text="It wires itself — no LLM in the loop." delay={4} size={56} />
      </div>

      {/* edges extract at index time */}
      <svg style={{position: 'absolute', inset: 0}} width="1920" height="1080">
        {EDGES.map(([a, b, rel], i) => {
          const t = edgeIn(i);
          const na = NODES[a];
          const nb = NODES[b];
          return (
            <g key={rel}>
              <line
                x1={na.x}
                y1={na.y}
                x2={na.x + (nb.x - na.x) * t}
                y2={na.y + (nb.y - na.y) * t}
                stroke={a === 0 ? color.caution : color.accentDeep}
                strokeWidth={2.5}
                opacity={0.85}
                strokeDasharray={a === 0 ? '8 6' : 'none'}
              />
              {t > 0.6 && (
                <text
                  x={(na.x + nb.x) / 2}
                  y={(na.y + nb.y) / 2 - 10}
                  fill={color.ink3}
                  fontSize={18}
                  fontFamily={font.mono}
                  textAnchor="middle"
                >
                  {rel}
                </text>
              )}
            </g>
          );
        })}
      </svg>

      {/* nodes */}
      {NODES.map((n, i) => {
        const t = interpolate(frame, [16 + i * 6, 34 + i * 6], [0, 1], {
          extrapolateLeft: 'clamp',
          extrapolateRight: 'clamp',
          easing: Easing.bezier(0.2, 0, 0, 1),
        });
        const fading = n.superseded ? 1 - sup * 0.75 : 1;
        const highlighted = i === 1 ? sup : 0;
        return (
          <div
            key={n.id}
            style={{
              position: 'absolute',
              left: n.x - 80,
              top: n.y - 26,
              width: 160,
              padding: '14px 0',
              borderRadius: radius.lg,
              background: color.surface,
              border: `1.5px solid ${i === 1 && highlighted > 0.4 ? color.accent : color.hairline}`,
              textAlign: 'center',
              fontFamily: font.mono,
              fontSize: 19,
              color: color.ink,
              opacity: t * fading,
              transform: `scale(${0.6 + t * 0.4})`,
            }}
          >
            {n.id}
            {n.superseded && sup > 0.5 && (
              <div style={{fontSize: 14, color: color.caution, marginTop: 4}}>superseded</div>
            )}
          </div>
        );
      })}

      {/* --as-of scrubber */}
      <div
        style={{
          position: 'absolute',
          left: 560,
          top: 830,
          width: 800,
          opacity: scrub > 0 ? 1 : 0,
        }}
      >
        <div style={{fontFamily: font.mono, fontSize: 22, color: color.ink2, marginBottom: 14}}>
          kurultai search --as-of 2026-09-30 'tick batching'
        </div>
        <div style={{position: 'relative', height: 6, borderRadius: 3, background: color.hairline}}>
          <div
            style={{
              position: 'absolute',
              left: 0,
              top: 0,
              height: 6,
              borderRadius: 3,
              width: `${scrub * 100}%`,
              background: color.accent,
            }}
          />
        </div>
        <div style={{display: 'flex', justifyContent: 'space-between', fontFamily: font.mono, fontSize: 16, color: color.ink3, marginTop: 10}}>
          <span>yesterday</span>
          <span>now</span>
        </div>
      </div>
    </div>
  );
};
