import React from 'react';
import {interpolate, spring, useCurrentFrame} from 'remotion';
import {color, font, radius} from '../../../brands/kurultai/tokens';
import {StageBox, MTag} from './Diagram';

/** S1: two agents — one spent the session learning, the other knows nothing. */
export const E1Hook: React.FC = () => {
  const f = useCurrentFrame();

  const fill = interpolate(f, [40, 200], [0, 1], {extrapolateRight: 'clamp'});
  const agentB = spring({frame: f - 220, fps: 60, config: {damping: 200}});
  const questionO = interpolate(f, [330, 360], [0, 1], {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'});
  const ellipsis = '·'.repeat(1 + Math.floor(Math.max(0, f - 380) / 20) % 3);

  const laptop = (x: number, name: string, known: boolean, o: number) => (
    <div style={{position: 'absolute', left: x, top: 260, width: 560, opacity: o}}>
      <div
        style={{
          border: `1.5px solid ${color.hairline}`,
          borderRadius: radius.lg,
          background: color.surface,
          overflow: 'hidden',
        }}
      >
        <div style={{display: 'flex', gap: 7, padding: '12px 16px', borderBottom: `1px solid ${color.hairline}`}}>
          {[color.failed, color.caution, color.passed].map((c) => (
            <div key={c} style={{width: 11, height: 11, borderRadius: 6, background: c, opacity: 0.7}} />
          ))}
          <div style={{marginLeft: 'auto', fontFamily: font.mono, fontSize: 15, color: color.ink3}}>{name}</div>
        </div>
        <div style={{padding: 22, height: 240, fontFamily: font.mono, fontSize: 18, lineHeight: 1.9, color: color.ink2}}>
          {known ? (
            <>
              <div style={{opacity: Math.min(1, fill * 3)}}>$ pool size was the problem</div>
              <div style={{opacity: Math.min(1, fill * 3 - 1)}}>$ pgBouncer, not the network</div>
              <div style={{opacity: Math.min(1, fill * 3 - 2)}}>$ set max_connections = 50</div>
              <div style={{color: color.accentSoft, opacity: Math.min(1, fill * 3 - 3)}}>✓ fixed at 4:40pm</div>
            </>
          ) : (
            <div style={{color: color.ink3, fontStyle: 'italic'}}>new session — empty</div>
          )}
        </div>
        {/* context meter */}
        <div style={{padding: '0 22px 20px'}}>
          <div style={{display: 'flex', justifyContent: 'space-between', fontFamily: font.mono, fontSize: 13, color: color.ink3, marginBottom: 6}}>
            <span>context</span>
            <span>{known ? `${Math.round(fill * 96)}%` : '0%'}</span>
          </div>
          <div style={{height: 8, borderRadius: 4, background: color.surfaceSunk}}>
            <div
              style={{
                height: '100%',
                borderRadius: 4,
                width: `${(known ? fill : 0) * 100}%`,
                background: `linear-gradient(90deg, ${color.accentDeep}, ${color.accent})`,
              }}
            />
          </div>
        </div>
      </div>
    </div>
  );

  return (
    <div style={{position: 'absolute', inset: 0, background: color.canvas}}>
      {laptop(300, 'agent you used yesterday', true, 1)}
      {laptop(1060, 'agent you open tomorrow', false, agentB)}

      {f > 330 && (
        <div
          style={{
            position: 'absolute',
            left: 1090,
            top: 580,
            opacity: questionO,
            transform: `translateY(${(1 - questionO) * 14}px)`,
            fontFamily: font.grotesk,
            fontSize: 30,
            color: color.ink,
          }}
        >
          “what did we fix?” {f > 380 && <span style={{color: color.ink3}}>{ellipsis}</span>}
        </div>
      )}

      <MTag x={330} y={590} at={60}>learned the hard way</MTag>
      <MTag x={1090} y={230} at={agentB ? 220 + 20 : 9999} tone={color.failed}>knows nothing</MTag>

      <div
        style={{
          position: 'absolute',
          bottom: 130,
          width: '100%',
          textAlign: 'center',
          fontFamily: font.grotesk,
          fontSize: 44,
          fontWeight: 600,
          color: color.ink,
          opacity: interpolate(f, [480, 520], [0, 1], {extrapolateLeft: 'clamp'}),
        }}
      >
        how does an agent <span style={{color: color.accentSoft}}>remember</span> anything?
      </div>
    </div>
  );
};
