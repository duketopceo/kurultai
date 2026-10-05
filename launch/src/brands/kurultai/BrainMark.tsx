import React from 'react';
import {useCurrentFrame, interpolate} from 'remotion';
import {color} from './tokens';

/**
 * Kurultai mark — a synapse cluster drawing itself: central neuron with
 * four dendrite arcs sparking outward. Electric zap shimmer, not plain orbs.
 */
export const SynapseMark: React.FC<{
  size?: number;
  delay?: number;
  tint?: string;
}> = ({size = 190, delay = 0, tint = color.accent}) => {
  const frame = useCurrentFrame();
  const f = frame - delay;
  const draw = interpolate(f, [0, 42], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  const shimmer = f > 42 ? 0.75 + 0.25 * Math.sin(f / 4) : draw;
  const dash = (len: number) => `${len * draw} ${len}`;
  return (
    <svg width={size} height={size} viewBox="0 0 100 100" fill="none">
      {/* dendrites */}
      <path d="M50 50 L20 22" stroke={tint} strokeWidth="2.4" strokeDasharray={dash(40)} opacity={shimmer} />
      <path d="M50 50 L80 26" stroke={tint} strokeWidth="2.4" strokeDasharray={dash(38)} opacity={shimmer} />
      <path d="M50 50 L18 74" stroke={tint} strokeWidth="2.4" strokeDasharray={dash(40)} opacity={shimmer} />
      <path d="M50 50 L82 76" stroke={tint} strokeWidth="2.4" strokeDasharray={dash(40)} opacity={shimmer} />
      <path d="M20 22 L80 26" stroke={tint} strokeWidth="1.2" strokeDasharray={dash(60)} opacity={shimmer * 0.5} />
      <path d="M18 74 L82 76" stroke={tint} strokeWidth="1.2" strokeDasharray={dash(64)} opacity={shimmer * 0.5} />
      {/* outer neurons */}
      {[
        [20, 22],
        [80, 26],
        [18, 74],
        [82, 76],
      ].map(([x, y], i) => (
        <circle key={i} cx={x} cy={y} r={4.5 * draw} fill={color.ink} opacity={shimmer} />
      ))}
      {/* core neuron */}
      <circle cx="50" cy="50" r={9 * draw} fill={tint} opacity={shimmer} />
      <circle cx="50" cy="50" r={15 * draw} stroke={tint} strokeWidth="1.6" fill="none" opacity={shimmer * 0.6} />
    </svg>
  );
};

export default SynapseMark;
