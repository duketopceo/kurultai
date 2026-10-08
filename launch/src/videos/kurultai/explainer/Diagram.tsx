import React from 'react';
import {interpolate, useCurrentFrame} from 'remotion';
import {color, font, radius} from '../../../brands/kurultai/tokens';

/**
 * Explainer diagram primitives — KodeKloud-style annotation vocabulary:
 * thin-bordered stage boxes, hand-drawn dashed arrows that draw on,
 * small mono callout labels. Dark canvas, no skin font (product's own
 * type system per the explainer-track rule).
 */

export const StageBox: React.FC<{
  x: number;
  y: number;
  w: number;
  h: number;
  label?: string;
  tone?: string;
  glow?: boolean;
  opacity?: number;
  children?: React.ReactNode;
}> = ({x, y, w, h, label, tone = color.accent, glow = false, opacity = 1, children}) => (
  <div
    style={{
      position: 'absolute',
      left: x,
      top: y,
      width: w,
      height: h,
      borderRadius: radius.lg,
      border: `1.5px solid ${tone}66`,
      background: color.surface,
      boxShadow: glow ? `0 0 80px ${tone}33, inset 0 0 40px ${tone}0d` : 'none',
      opacity,
      padding: label ? '40px 28px 24px' : 24,
    }}
  >
    {label && (
      <div
        style={{
          position: 'absolute',
          top: -14,
          left: 24,
          padding: '2px 12px',
          borderRadius: radius.sm,
          background: color.canvas,
          border: `1.5px solid ${tone}88`,
          color: tone,
          fontFamily: font.mono,
          fontSize: 17,
          letterSpacing: 1.5,
          textTransform: 'uppercase',
        }}
      >
        {label}
      </div>
    )}
    {children}
  </div>
);

/** Dashed arrow that draws itself on between `at` and `at+dur` frames. */
export const DArrow: React.FC<{
  x1: number;
  y1: number;
  x2: number;
  y2: number;
  at: number;
  dur?: number;
  tone?: string;
  bend?: number;
}> = ({x1, y1, x2, y2, at, dur = 30, tone = color.accentSoft, bend = 0}) => {
  const f = useCurrentFrame();
  const p = interpolate(f, [at, at + dur], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  const mx = (x1 + x2) / 2 + bend;
  const my = (y1 + y2) / 2;
  const path = `M ${x1} ${y1} Q ${mx} ${my} ${x2} ${y2}`;
  // rough length estimate for dash offset
  const len = Math.hypot(x2 - x1, y2 - y1) * 1.15;
  const ax = x2 - (x2 - mx) * 0.001;
  return (
    <svg
      style={{position: 'absolute', left: 0, top: 0, width: 1920, height: 1080, pointerEvents: 'none'}}
      viewBox="0 0 1920 1080"
    >
      <path
        d={path}
        fill="none"
        stroke={tone}
        strokeWidth={2.5}
        strokeDasharray={`10 9 ${len}`}
        strokeDashoffset={len * (1 - p) * -1}
        strokeLinecap="round"
        opacity={p <= 0 ? 0 : 1}
      />
      {p > 0.9 && (
        <circle cx={x2} cy={y2} r={5} fill={tone} opacity={(p - 0.9) * 10} />
      )}
    </svg>
  );
};

/** Small marker-style callout label. */
export const MTag: React.FC<{
  x: number;
  y: number;
  at: number;
  tone?: string;
  children: React.ReactNode;
}> = ({x, y, at, tone = color.ink2, children}) => {
  const f = useCurrentFrame();
  const o = interpolate(f, [at, at + 12], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  return (
    <div
      style={{
        position: 'absolute',
        left: x,
        top: y,
        opacity: o,
        transform: `translateY(${(1 - o) * 8}px)`,
        color: tone,
        fontFamily: font.mono,
        fontSize: 20,
        fontStyle: 'italic',
        letterSpacing: 0.5,
      }}
    >
      {children}
    </div>
  );
};
