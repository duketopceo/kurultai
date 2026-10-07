import React from 'react';
import {interpolate, useCurrentFrame} from 'remotion';
import {color, font, radius} from '../../../brands/kurultai/tokens';
import {StageBox, MTag} from './Diagram';

const TOKENS = [
  'read db.ts — pool config',
  'grep max_connections',
  'pgBouncer docs lookup',
  'test: timeout repro',
  'fix: pool size 10 → 50',
  'decision: batch ticks 500ms',
];

/** S2: the context window — entire short-term memory, resets on session end. */
export const E2Context: React.FC = () => {
  const f = useCurrentFrame();

  const fill = interpolate(f, [20, 160], [0, 1], {extrapolateRight: 'clamp'});
  const wipe = interpolate(f, [260, 340], [0, 1], {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'});
  const visible = Math.floor(fill * TOKENS.length);

  return (
    <div style={{position: 'absolute', inset: 0, background: color.canvas}}>
      <StageBox x={560} y={200} w={800} h={620} label="context window" tone={color.accent} glow>
        <div style={{fontFamily: font.mono, fontSize: 20, lineHeight: 2.2, color: color.ink2, position: 'relative'}}>
          {TOKENS.slice(0, visible).map((t, i) => (
            <div
              key={t}
              style={{
                opacity: 1 - wipe,
                transform: `translateY(${wipe * -14}px)`,
                filter: `blur(${wipe * 4}px)`,
              }}
            >
              <span style={{color: color.accentDeep}}>▸ </span>{t}
            </div>
          ))}
          {wipe > 0 && wipe < 1 && (
            <div
              style={{
                position: 'absolute',
                left: -40,
                right: -40,
                top: wipe * 420 - 20,
                height: 3,
                background: color.failed,
                boxShadow: `0 0 24px ${color.failed}`,
              }}
            />
          )}
          {wipe >= 1 && <div style={{color: color.ink3, fontStyle: 'italic'}}>— empty —</div>}
        </div>
      </StageBox>

      <MTag x={560} y={160} at={30}>the agent's ENTIRE short-term memory</MTag>
      {wipe > 0.05 && wipe < 1 && (
        <MTag x={1000} y={600} at={270} tone={color.failed}>session end</MTag>
      )}
      {wipe >= 1 && (
        <div
          style={{
            position: 'absolute',
            bottom: 130,
            width: '100%',
            textAlign: 'center',
            fontFamily: font.grotesk,
            fontSize: 40,
            fontWeight: 600,
            color: color.ink,
            opacity: interpolate(f, [360, 400], [0, 1], {extrapolateLeft: 'clamp'}),
          }}
        >
          everything it learned — <span style={{color: color.failed}}>gone.</span>
        </div>
      )}
    </div>
  );
};
