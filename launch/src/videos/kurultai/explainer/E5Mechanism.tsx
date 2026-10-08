import React from 'react';
import {interpolate, spring, useCurrentFrame} from 'remotion';
import {color, font, radius} from '../../../brands/kurultai/tokens';
import {StageBox, DArrow, MTag} from './Diagram';
import {SynthCursor} from '../../../core/SynthCursor';

const AGENTS = ['claude', 'codex', 'cursor', 'devin', 'aider'];

/**
 * S5: the mechanism — one daemon, every agent over MCP; each memory carries
 * source + trust tier; agents propose, humans approve.
 */
export const E5Mechanism: React.FC = () => {
  const f = useCurrentFrame();

  const ringIn = interpolate(f, [20, 60], [0, 1], {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'});
  const cardIn = spring({frame: f - 330, fps: 60, config: {damping: 200}});
  const propPulse = 0.5 + 0.5 * Math.sin((f - 560) * 0.15);
  const approved = f > 720;
  const propO = interpolate(f, [560, 590], [0, 1], {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'});
  const endO = interpolate(f, [900, 940], [0, 1], {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'});

  const cx = 960;
  const cy = 470;
  const R = 330;

  return (
    <div style={{position: 'absolute', inset: 0, background: color.canvas}}>
      {/* agent ring -> daemon */}
      {AGENTS.map((a, i) => {
        const ang = (-90 + i * 72) * (Math.PI / 180);
        const x = cx + Math.cos(ang) * R - 78;
        const y = cy + Math.sin(ang) * R - 22;
        return (
          <React.Fragment key={a}>
            <DArrow x1={x + 78} y1={y + 22} x2={cx} y2={cy} at={100 + i * 15} bend={i % 2 ? 30 : -30} />
            <div
              style={{
                position: 'absolute',
                left: x,
                top: y,
                padding: '10px 22px',
                borderRadius: radius.md,
                border: `1.5px solid ${color.hairline}`,
                background: color.surface,
                fontFamily: font.mono,
                fontSize: 20,
                color: color.ink,
                opacity: ringIn,
                transform: `scale(${0.7 + 0.3 * ringIn})`,
              }}
            >
              {a}
            </div>
          </React.Fragment>
        );
      })}

      <div
        style={{
          position: 'absolute',
          left: cx - 130,
          top: cy - 46,
          width: 260,
          padding: '18px 0',
          textAlign: 'center',
          borderRadius: radius.lg,
          border: `2px solid ${color.accent}`,
          background: color.accentTint,
          boxShadow: `0 0 70px ${color.accent}44`,
          fontFamily: font.mono,
          fontSize: 22,
          color: color.accentSoft,
          opacity: ringIn,
        }}
      >
        kurultai
        <div style={{fontSize: 14, color: color.ink3, marginTop: 4}}>one daemon · MCP</div>
      </div>

      {/* memory card: source + trust tier */}
      <div
        style={{
          position: 'absolute',
          left: 300,
          top: 780,
          display: 'flex',
          alignItems: 'center',
          gap: 14,
          opacity: cardIn,
          transform: `translateY(${(1 - cardIn) * 24}px)`,
          padding: '16px 22px',
          borderRadius: radius.lg,
          border: `1.5px solid ${color.hairline}`,
          background: color.surface,
          fontFamily: font.mono,
          fontSize: 19,
        }}
      >
        <span style={{color: color.ink}}>“timeout was pgBouncer”</span>
        <span style={{color: color.ink3}}>source: PR #318</span>
        <span
          style={{
            padding: '3px 12px',
            borderRadius: 999,
            background: color.passedTint,
            border: `1px solid ${color.passed}55`,
            color: color.passed,
            fontSize: 15,
          }}
        >
          trusted
        </span>
      </div>

      {/* proposed edge -> human approves */}
      <svg style={{position: 'absolute', inset: 0}} viewBox="0 0 1920 1080" opacity={propO}>
        <path
          d="M 1240 830 Q 1330 760 1420 830"
          fill="none"
          stroke={approved ? color.passed : color.caution}
          strokeWidth={2.5}
          strokeDasharray={approved ? 'none' : '8 8'}
          opacity={approved ? 1 : 0.4 + 0.5 * propPulse}
        />
      </svg>
      <div
        style={{
          position: 'absolute',
          left: 1240,
          top: 850,
          display: 'flex',
          gap: 18,
          alignItems: 'center',
          opacity: propO,
          fontFamily: font.mono,
          fontSize: 18,
        }}
      >
        <span style={{color: color.ink2}}>new link proposed</span>
        <span style={{color: color.ink3}}>prop:9ca0a523</span>
        <span
          style={{
            padding: '4px 14px',
            borderRadius: 999,
            border: `1.5px solid ${approved ? color.passed : color.caution}`,
            color: approved ? color.passed : color.caution,
            background: approved ? color.passedTint : color.cautionTint,
            fontSize: 16,
            transition: 'none',
          }}
        >
          {approved ? '✓ approved' : 'awaiting human'}
        </span>
      </div>
      {f > 640 && f < 780 && (
        <SynthCursor
          keys={[
            {at: 640, x: 1700, y: 700},
            {at: 700, x: 1560, y: 880},
            {at: 720, x: 1560, y: 880, click: true},
            {at: 770, x: 1700, y: 960},
          ]}
        />
      )}

      <MTag x={120} y={200} at={80}>every agent reads & writes</MTag>
      <MTag x={320} y={740} at={cardIn ? 350 : 9999}>source + trust tier on every memory</MTag>
      <MTag x={1260} y={930} at={580} tone={color.caution}>agents propose — humans decide</MTag>

      {endO > 0 && (
        <div
          style={{
            position: 'absolute',
            bottom: 60,
            width: '100%',
            textAlign: 'center',
            fontFamily: font.grotesk,
            fontSize: 38,
            fontWeight: 600,
            color: color.ink,
            opacity: endO,
          }}
        >
          five agents. <span style={{color: color.accentSoft}}>one brain.</span>
        </div>
      )}
    </div>
  );
};
