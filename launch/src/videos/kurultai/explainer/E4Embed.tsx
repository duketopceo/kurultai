import React from 'react';
import {interpolate, spring, useCurrentFrame} from 'remotion';
import {color, font, radius} from '../../../brands/kurultai/tokens';
import {StageBox, DArrow, MTag} from './Diagram';

/**
 * S4: the trick — memories become points in space; similar meaning lands
 * near each other; a query lands and lights its neighbors.
 */

// Plot space (1920x1080 canvas): box at x 600-1320, y 300-820 → plot area inset
const P = {x: 660, y: 360, w: 600, h: 420};
const dot = (px: number, py: number) => ({x: P.x + px * P.w, y: P.y + py * P.h});

const POINTS = [
  {px: 0.28, py: 0.30, label: 'batch interval settled: 500ms', at: 200, hot: true},
  {px: 0.38, py: 0.42, label: '500ms batches, decided w/ infra', at: 260, hot: true},
  {px: 0.72, py: 0.72, label: 'pgBouncer, not the network', at: 320, hot: false},
  {px: 0.20, py: 0.70, label: 'max_connections = 50', at: 380, hot: false},
];

const QUERY = {px: 0.30, py: 0.50};

export const E4Embed: React.FC = () => {
  const f = useCurrentFrame();

  const vecO = interpolate(f, [60, 90], [0, 1], {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'});
  const queryIn = spring({frame: f - 470, fps: 60, config: {damping: 160}});
  const linkO = interpolate(f, [520, 560], [0, 1], {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'});

  return (
    <div style={{position: 'absolute', inset: 0, background: color.canvas}}>
      {/* memory sentence -> vector */}
      <div
        style={{
          position: 'absolute',
          left: 420,
          top: 120,
          width: 1080,
          display: 'flex',
          alignItems: 'center',
          gap: 26,
          fontFamily: font.mono,
          fontSize: 20,
        }}
      >
        <div style={{padding: '12px 18px', border: `1.5px solid ${color.hairline}`, borderRadius: radius.md, background: color.surface, color: color.ink}}>
          “batch interval settled: 500ms”
        </div>
        <div style={{color: color.ink3, opacity: vecO}}>→</div>
        <div style={{padding: '12px 18px', border: `1.5px solid ${color.accent}66`, borderRadius: radius.md, background: color.accentTint, color: color.accentSoft, opacity: vecO}}>
          [0.21, −0.44, 0.87, …]
        </div>
      </div>

      {/* embedding space */}
      <StageBox x={560} y={280} w={800} h={560} label="meaning space" tone={color.accentSoft} glow>
        <svg style={{position: 'absolute', inset: 0, width: '100%', height: '100%'}} viewBox="0 0 800 560">
          <line x1={40} y1={520} x2={760} y2={520} stroke={color.hairline} strokeWidth={1.5} />
          <line x1={40} y1={520} x2={40} y2={40} stroke={color.hairline} strokeWidth={1.5} />
          {/* query -> hot neighbors links */}
          {linkO > 0 &&
            POINTS.filter((p) => p.hot).map((p) => {
              const a = dot(p.px, p.py);
              const q = dot(QUERY.px, QUERY.py);
              return (
                <line
                  key={p.label}
                  x1={q.x - 560 - 100}
                  y1={q.y - 280 - 80}
                  x2={a.x - 560 - 100}
                  y2={a.y - 280 - 80}
                  stroke={color.accent}
                  strokeWidth={2}
                  strokeDasharray="6 6"
                  opacity={linkO}
                />
              );
            })}
        </svg>
        {POINTS.map((p) => {
          const a = dot(p.px, p.py);
          const o = interpolate(f, [p.at, p.at + 20], [0, 1], {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'});
          const lit = p.hot ? linkO : 0;
          return (
            <div key={p.label}>
              <div
                style={{
                  position: 'absolute',
                  left: a.x - 560 - 100 + 6,
                  top: a.y - 280 - 80 - 6,
                  width: 12,
                  height: 12,
                  borderRadius: 6,
                  background: lit ? color.accentSoft : color.ink3,
                  boxShadow: lit ? `0 0 ${24 * lit}px ${color.accent}` : 'none',
                  opacity: o,
                }}
              />
              <div
                style={{
                  position: 'absolute',
                  left: a.x - 560 - 100 + 22,
                  top: a.y - 280 - 80 - 14,
                  fontFamily: font.mono,
                  fontSize: 15,
                  color: lit ? color.accentSoft : color.ink3,
                  opacity: o,
                  whiteSpace: 'nowrap',
                }}
              >
                {p.label}
              </div>
            </div>
          );
        })}
        {/* query point */}
        {queryIn > 0 && (
          <>
            <div
              style={{
                position: 'absolute',
                left: QUERY.px * 600 + 90,
                top: QUERY.py * 560 + 40,
                width: 16,
                height: 16,
                borderRadius: 8,
                border: `2.5px solid ${color.caution}`,
                background: 'transparent',
                opacity: queryIn,
                boxShadow: `0 0 ${20 * queryIn}px ${color.caution}88`,
              }}
            />
            <div
              style={{
                position: 'absolute',
                left: QUERY.px * 600 + 118,
                top: QUERY.py * 560 + 28,
                fontFamily: font.mono,
                fontSize: 16,
                color: color.caution,
                opacity: queryIn,
                whiteSpace: 'nowrap',
              }}
            >
              “tick batching?”
            </div>
          </>
        )}
      </StageBox>

      <MTag x={120} y={300} at={70}>each memory → a point</MTag>
      <MTag x={1400} y={560} at={400}>similar meaning → nearby</MTag>
      {linkO > 0 && (
        <div
          style={{
            position: 'absolute',
            bottom: 120,
            width: '100%',
            textAlign: 'center',
            fontFamily: font.grotesk,
            fontSize: 40,
            fontWeight: 600,
            color: color.ink,
            opacity: linkO,
          }}
        >
          search by <span style={{color: color.accentSoft}}>meaning</span>, not keywords.
        </div>
      )}
    </div>
  );
};
