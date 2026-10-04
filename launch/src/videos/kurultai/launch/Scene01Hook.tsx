import React from 'react';
import {useCurrentFrame, interpolate} from 'remotion';
import {SynapseMark} from '../../../brands/kurultai/BrainMark';
import {KineticType} from '../../../core/KineticType';
import {color} from '../../../brands/kurultai/tokens';

/**
 * Scene 01 — Hook (0–2.5s, 150f @60).
 * Black canvas → synapse cluster draws itself + shimmer → headline lands
 * word by word. Purple spent only on the mark.
 */
export const Scene01Hook: React.FC = () => {
  const frame = useCurrentFrame();
  const lift = interpolate(frame, [120, 150], [0, -18], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  return (
    <div
      style={{
        position: 'absolute',
        inset: 0,
        background: color.canvas,
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        gap: 64,
        transform: `translateY(${lift}px)`,
      }}
    >
      <SynapseMark size={190} delay={2} tint={color.accent} />
      <KineticType
        text="One brain. Every agent."
        delay={48}
        size={104}
        staggerMs={60}
      />
    </div>
  );
};
