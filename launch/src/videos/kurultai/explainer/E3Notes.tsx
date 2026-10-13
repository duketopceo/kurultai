import React from 'react';
import {interpolate, useCurrentFrame} from 'remotion';
import {color, font, radius} from '../../../brands/kurultai/tokens';
import {StageBox, DArrow, MTag} from './Diagram';

const NOTES = [
  '- pools default to 10; ours need 50',
  '- timeout was pgBouncer, not network',
  '- batch interval settled: 500ms',
  '- ask infra before touching c2',
];

/** S3: the notes-file instinct — you can't grep a question. */
export const E3Notes: React.FC = () => {
  const f = useCurrentFrame();

  const queryO = interpolate(f, [80, 110], [0, 1], {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'});
  const missO = interpolate(f, [300, 330], [0, 1], {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'});
  const reveal = interpolate(f, [400, 440], [0, 1], {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'});

  return (
    <div style={{position: 'absolute', inset: 0, background: color.canvas}}>
      {/* query */}
      <div
        style={{
          position: 'absolute',
          left: 560,
          top: 150,
          width: 800,
          opacity: queryO,
          transform: `translateY(${(1 - queryO) * 16}px)`,
          border: `1.5px solid ${color.hairline}`,
          borderRadius: radius.lg,
          background: color.surfaceSunk,
          padding: '18px 24px',
          fontFamily: font.mono,
          fontSize: 22,
          color: color.ink,
        }}
      >
        <span style={{color: color.ink3}}>› </span>
        what did we decide about tick batching?
      </div>

      <DArrow x1={960} y1={230} x2={960} y2={330} at={140} tone={color.ink3} />

      <StageBox x={560} y={330} w={800} h={420} label="notes.txt" tone={color.ink3}>
        <div style={{fontFamily: font.mono, fontSize: 21, lineHeight: 2.3, color: color.ink2}}>
          {NOTES.map((n, i) => {
            const isAnswer = i === 2;
            return (
              <div
                key={n}
                style={{
                  borderRadius: 6,
                  padding: '0 10px',
                  marginLeft: -10,
                  background: isAnswer ? `${color.accent}1f` : 'transparent',
                  outline: isAnswer ? `1.5px solid ${color.accent}88` : 'none',
                  opacity: isAnswer && reveal < 0.5 ? 0.45 : 1,
                }}
              >
                {n}
                {isAnswer && reveal > 0.5 && (
                  <span style={{color: color.accentSoft, fontSize: 16}}>  ← it was right there</span>
                )}
              </div>
            );
          })}
        </div>
      </StageBox>

      {missO > 0 && (
        <div
          style={{
            position: 'absolute',
            left: 560,
            top: 790,
            opacity: missO,
            fontFamily: font.mono,
            fontSize: 26,
            color: color.failed,
          }}
        >
          ✗ 0 matches
        </div>
      )}

      <MTag x={120} y={360} at={60}>first instinct: a notes file</MTag>
      <MTag x={1430} y={330} at={330} tone={color.failed}>keywords can't find meaning</MTag>
    </div>
  );
};
